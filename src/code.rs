//! Code generator for Lua

use crate::{
    api::LuaError,
    lex::LexState,
    limits::MAX_LUA_STACK,
    object::TValue,
    opcodes::{
        create_abc, create_abx, get_arg_a, get_arg_b, get_arg_c, get_arg_sbx, get_opcode,
        is_reg_constant, rk_as_k, set_arg_a, set_arg_b, set_arg_c, set_arg_sbx, OpCode,
        LFIELDS_PER_FLUSH, MAXARG_C, MAXARG_SBX, MAX_INDEX_RK, NO_JUMP, NO_REG,
    },
    parser::{BinaryOp, ExpressionDesc, ExpressionKind, UnaryOp},
    state::LuaState,
    LuaNumber, LUA_MULTRET,
};

pub(crate) fn discharge_vars<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    match exp.k {
        ExpressionKind::LocalRegister => exp.k = ExpressionKind::NonRelocable,
        ExpressionKind::UpValue => {
            exp.info = code_abc(lex, state, OpCode::GetUpVal as u32, 0, exp.info, 0)? as i32;
            exp.k = ExpressionKind::Relocable;
        }
        ExpressionKind::GlobalVar => {
            exp.info = code_abx(lex, state, OpCode::GetGlobal as u32, 0, exp.info as u32)? as i32;
            exp.k = ExpressionKind::Relocable;
        }
        ExpressionKind::Indexed => {
            free_reg(lex, exp.aux as u32);
            free_reg(lex, exp.info as u32);
            exp.info = code_abc(lex, state, OpCode::GetTable as u32, 0, exp.info, exp.aux)? as i32;
            exp.k = ExpressionKind::Relocable;
        }
        ExpressionKind::Call | ExpressionKind::VarArg => {
            set_one_ret(lex, exp);
        }
        _ => (), // there is one value available (somewhere)
    }
    Ok(())
}

pub(crate) fn set_one_ret<T>(lex: &mut LexState<T>, exp: &mut ExpressionDesc) {
    let fs = lex.borrow_fs(None);
    if exp.k == ExpressionKind::Call {
        // expression is an open function call?
        exp.k = ExpressionKind::NonRelocable;
        exp.info = get_arg_a(fs.f.code[exp.info as usize]) as i32;
    } else if exp.k == ExpressionKind::VarArg {
        set_arg_b(lex.borrow_mut_code(exp.info as usize), 2);
        exp.k = ExpressionKind::Relocable;
    }
}

pub(crate) fn code_abc<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: u32,
    a: i32,
    b: i32,
    c: i32,
) -> Result<u32, LuaError> {
    let o = create_abc(op, a, b, c);
    code(lex, state, o, lex.lastline)
}

pub(crate) fn code_abx<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: u32,
    a: i32,
    bx: u32,
) -> Result<u32, LuaError> {
    let o = create_abx(op, a, bx);
    code(lex, state, o, lex.lastline)
}

pub(crate) fn code_asbx<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: u32,
    a: i32,
    sbx: i32,
) -> Result<u32, LuaError> {
    code_abx(lex, state, op, a, (sbx + MAXARG_SBX) as u32)
}

fn code<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    o: u32,
    line: usize,
) -> Result<u32, LuaError> {
    discharge_jpc(lex, state)?; // pc' will change
    let fs = lex.borrow_mut_fs(None);
    let pc = fs.next_pc() as u32;
    let f = &mut fs.f;
    f.code.push(o);
    f.lineinfo.push(line);
    Ok(pc)
}

fn discharge_jpc<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let pc = lex.borrow_fs(None).next_pc();
    let jpc = lex.borrow_fs(None).jpc;
    patch_list_aux(lex, state, jpc, pc, NO_REG, pc)?;
    lex.borrow_mut_fs(None).jpc = NO_JUMP;
    Ok(())
}

pub(crate) fn patch_list<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    list: i32,
    target: i32,
) -> Result<(), LuaError> {
    let pc = lex.borrow_fs(None).next_pc();
    if target == pc {
        patch_to_here(lex, state, list)
    } else {
        debug_assert!(target < pc);
        patch_list_aux(lex, state, list, target, NO_REG, target)
    }
}

fn patch_list_aux<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    jpc: i32,
    vtarget: i32,
    reg: u32,
    dtarget: i32,
) -> Result<(), LuaError> {
    let mut jpc = jpc;
    while jpc != NO_JUMP {
        let next = get_jump(lex, jpc);
        if patch_test_reg(lex, jpc, reg) {
            fix_jump(lex, state, jpc, vtarget)?;
        } else {
            fix_jump(lex, state, jpc, dtarget)?;
        }
        jpc = next;
    }
    Ok(())
}

fn fix_jump<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    pc: i32,
    dest: i32,
) -> Result<(), LuaError> {
    let jmp = &mut lex.borrow_mut_fs(None).f.code[pc as usize];
    let offset = dest - (pc + 1);
    debug_assert!(dest != NO_JUMP);
    if offset.abs() > MAXARG_SBX {
        return lex.syntax_error(state, "controle structure too long");
    }
    set_arg_sbx(jmp, offset);
    Ok(())
}

fn patch_test_reg<T>(lex: &mut LexState<T>, node: i32, reg: u32) -> bool {
    let i = get_jump_control(lex, node);
    if get_opcode(*i) != OpCode::TestSet {
        return false; // cannot patch other instructions
    } else if reg != NO_REG && reg != get_arg_b(*i) {
        set_arg_a(i, reg);
    } else {
        // no register to put value or register already has the value
        *i = create_abc(
            OpCode::Test as u32,
            get_arg_b(*i) as i32,
            0,
            get_arg_c(*i) as i32,
        );
    }
    true
}

pub(crate) fn store_var<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    var: &ExpressionDesc,
    ex: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    match var.k {
        ExpressionKind::LocalRegister => {
            free_exp(lex, ex);
            return exp2reg(lex, state, ex, var.info as u32);
        }
        ExpressionKind::UpValue => {
            let e = exp2anyreg(lex, state, ex)?;
            code_abc(lex, state, OpCode::SetupVal as u32, e as i32, var.info, 0)?;
        }
        ExpressionKind::GlobalVar => {
            let e = exp2anyreg(lex, state, ex)?;
            code_abx(
                lex,
                state,
                OpCode::SetGlobal as u32,
                e as i32,
                var.info as u32,
            )?;
        }
        ExpressionKind::Indexed => {
            let e = exp2rk(lex, state, ex)?;
            code_abc(
                lex,
                state,
                OpCode::SetTable as u32,
                var.info,
                var.aux,
                e as i32,
            )?;
        }
        _ => {
            debug_assert!(false); // invalid var kind to store
        }
    }
    free_exp(lex, ex);
    Ok(())
}

pub(crate) fn indexed<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    t: &mut ExpressionDesc,
    k: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    t.aux = exp2rk(lex, state, k)? as i32;
    t.k = ExpressionKind::Indexed;
    Ok(())
}

pub(crate) fn exp2rk<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    ex: &mut ExpressionDesc,
) -> Result<u32, LuaError> {
    exp2val(lex, state, ex)?;
    match ex.k {
        ExpressionKind::NumberConstant
        | ExpressionKind::True
        | ExpressionKind::False
        | ExpressionKind::Nil => {
            if lex.borrow_fs(None).f.k.len() <= MAX_INDEX_RK {
                ex.info = match ex.k {
                    ExpressionKind::Nil => nil_constant(lex) as i32,
                    ExpressionKind::NumberConstant => number_constant(lex, ex.nval) as i32,
                    _ => bool_constant(lex, ex.k == ExpressionKind::True) as i32,
                };
                ex.k = ExpressionKind::Constant;
                return Ok(rk_as_k(ex.info as u32));
            }
        }
        ExpressionKind::Constant => {
            if ex.info <= MAX_INDEX_RK as i32 {
                // constant fit in argC?
                return Ok(rk_as_k(ex.info as u32));
            }
        }
        _ => (),
    }
    // not a constant in the right range: put it in a register
    exp2anyreg(lex, state, ex)
}

fn bool_constant<T>(lex: &mut LexState<T>, val: bool) -> usize {
    let o = TValue::Boolean(val);
    lex.borrow_mut_fs(None).add_constant(o.clone(), o)
}

pub(crate) fn number_constant<T>(lex: &mut LexState<T>, val: f64) -> usize {
    let o = TValue::Number(val);
    lex.borrow_mut_fs(None).add_constant(o.clone(), o)
}

fn nil_constant<T>(lex: &mut LexState<T>) -> usize {
    // cannot use nil as key; instead use table itself to represent nil
    lex.borrow_mut_fs(None)
        .add_constant(TValue::Nil, TValue::Nil)
}

pub(crate) fn exp2val<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    ex: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    if has_jumps(ex) {
        exp2anyreg(lex, state, ex)?;
    } else {
        discharge_vars(lex, state, ex)?;
    }
    Ok(())
}

pub(crate) fn exp2anyreg<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    ex: &mut ExpressionDesc,
) -> Result<u32, LuaError> {
    discharge_vars(lex, state, ex)?;
    if ex.k == ExpressionKind::NonRelocable {
        if !has_jumps(ex) {
            return Ok(ex.info as u32); // exp is already in a register
        }
        if ex.info == lex.borrow_fs(None).nactvar as i32 {
            // reg. is not a local?
            exp2reg(lex, state, ex, ex.info as u32)?; // put value on it
            return Ok(ex.info as u32);
        }
    }
    exp2nextreg(lex, state, ex)?;
    Ok(ex.info as u32)
}

fn get_jump_control<T>(lex: &mut LexState<T>, pc: i32) -> &mut u32 {
    let fs = lex.borrow_mut_fs(None);
    let i = fs.f.code[pc as usize - 1];
    let op = get_opcode(i);
    if pc >= 1 && op.is_test() {
        lex.borrow_mut_code(pc as usize - 1)
    } else {
        lex.borrow_mut_code(pc as usize)
    }
}

/// get the new value of pc for this jump instruction
fn get_jump<T>(lex: &mut LexState<T>, jpc: i32) -> i32 {
    let fs = lex.borrow_mut_fs(None);
    let offset = get_arg_sbx(fs.f.code[jpc as usize]);
    if offset == NO_JUMP {
        // point to itself represents end of list
        NO_JUMP // end of list
    } else {
        jpc + 1 + offset // turn offset into absolute position
    }
}

pub(crate) fn exp2nextreg<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    discharge_vars(lex, state, exp)?;
    free_exp(lex, exp);
    reserve_regs(lex, state, 1)?;
    exp2reg(lex, state, exp, lex.borrow_fs(None).freereg as u32 - 1)
}

fn exp2reg<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
    reg: u32,
) -> Result<(), LuaError> {
    discharge2reg(lex, state, exp, reg)?;
    if let ExpressionKind::Jump = exp.k {
        concat_jump(lex, state, &mut exp.t, exp.info)?; // put this jump in `t' list
    }
    if has_jumps(exp) {
        let mut p_f = NO_JUMP; // position of an eventual LOAD false
        let mut p_t = NO_JUMP; // position of an eventual LOAD true
        if need_value(lex, exp.t) || need_value(lex, exp.f) {
            let fj = if let ExpressionKind::Jump = exp.k {
                NO_JUMP
            } else {
                jump(lex, state)?
            };
            p_f = code_label(lex, state, reg, 0, 1)?;
            p_t = code_label(lex, state, reg, 1, 0)?;
            patch_to_here(lex, state, fj)?;
        }
        let final_pc = get_label(lex); // position after whole expression
        patch_list_aux(lex, state, exp.f, final_pc, reg, p_f)?;
        patch_list_aux(lex, state, exp.t, final_pc, reg, p_t)?;
    }
    exp.f = NO_JUMP;
    exp.t = NO_JUMP;
    exp.info = reg as i32;
    exp.k = ExpressionKind::NonRelocable;
    Ok(())
}

pub(crate) fn jump<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<i32, LuaError> {
    let jpc = {
        let fs = lex.borrow_mut_fs(None);
        let jpc = fs.jpc;
        fs.jpc = NO_JUMP;
        jpc
    };
    let mut j = code_asbx(lex, state, OpCode::Jmp as u32, 0, NO_JUMP)? as i32;
    concat_jump(lex, state, &mut j, jpc)?;
    Ok(j)
}

pub(crate) fn set_mult_ret<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    set_returns(lex, state, exp, LUA_MULTRET)
}

pub(crate) fn set_returns<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
    nresults: i32,
) -> Result<(), LuaError> {
    if exp.k == ExpressionKind::Call {
        // expression is an open function call?
        let pc = exp.info as usize;
        set_arg_c(
            &mut lex.borrow_mut_fs(None).f.code[pc],
            (nresults + 1) as u32,
        );
    } else if exp.k == ExpressionKind::VarArg {
        let pc = exp.info as usize;
        {
            set_arg_b(lex.borrow_mut_code(pc), (nresults + 1) as u32);
            let freereg = lex.borrow_fs(None).freereg as u32;
            set_arg_a(lex.borrow_mut_code(pc), freereg);
        }
        reserve_regs(lex, state, 1)?;
    }
    Ok(())
}

/// returns current `pc' and marks it as a jump target (to avoid wrong
///  optimizations with consecutive instructions not in the same basic block).
pub(crate) fn get_label<T>(lex: &mut LexState<T>) -> i32 {
    let fs = lex.borrow_mut_fs(None);
    fs.last_target = fs.next_pc();
    fs.last_target
}

pub(crate) fn patch_to_here<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    list: i32,
) -> Result<(), LuaError> {
    get_label(lex);
    let mut jpc = lex.borrow_fs(None).jpc;
    concat_jump(lex, state, &mut jpc, list)?;
    lex.borrow_mut_fs(None).jpc = jpc;
    Ok(())
}

fn code_label<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    a: u32,
    b: i32,
    jump: i32,
) -> Result<i32, LuaError> {
    get_label(lex); // those instructions may be jump targets
    Ok(code_abc(lex, state, OpCode::LoadBool as u32, a as i32, b, jump)? as i32)
}

/// check whether list has any jump that do not produce a value
/// (or produce an inverted value)
fn need_value<T>(lex: &mut LexState<T>, list: i32) -> bool {
    let mut list = list;
    while list != NO_JUMP {
        let i = get_jump_control(lex, list);
        if get_opcode(*i) != OpCode::TestSet {
            return true;
        }
        list = get_jump(lex, list);
    }
    false
}

#[inline]
fn has_jumps(exp: &mut ExpressionDesc) -> bool {
    exp.t != exp.f
}

fn concat_jump<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    l1: &mut i32,
    l2: i32,
) -> Result<(), LuaError> {
    if l2 == NO_JUMP {
        return Ok(());
    }
    if *l1 == NO_JUMP {
        *l1 = l2;
    } else {
        let mut list = *l1;
        let mut next = get_jump(lex, list);
        while next != NO_JUMP {
            list = next;
            next = get_jump(lex, list)
        }
        fix_jump(lex, state, list, l2)?;
    }
    Ok(())
}

fn discharge2reg<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
    reg: u32,
) -> Result<(), LuaError> {
    discharge_vars(lex, state, exp)?;
    match exp.k {
        ExpressionKind::Nil => nil(lex, state, reg, 1)?,
        ExpressionKind::True | ExpressionKind::False => {
            code_abc(
                lex,
                state,
                OpCode::LoadBool as u32,
                reg as i32,
                if exp.k == ExpressionKind::True { 1 } else { 0 },
                0,
            )?;
        }
        ExpressionKind::Constant => {
            code_abx(
                lex,
                state,
                OpCode::LoadK as u32,
                reg as i32,
                exp.info as u32,
            )?;
        }
        ExpressionKind::NumberConstant => {
            let kid = lex
                .borrow_mut_fs(None)
                .number_constant(exp.nval as LuaNumber) as u32;
            code_abx(lex, state, OpCode::LoadK as u32, reg as i32, kid)?;
        }
        ExpressionKind::Relocable => {
            let pc = exp.info as usize;
            set_arg_a(&mut lex.vfs[lex.fs].f.code[pc], reg);
        }
        ExpressionKind::NonRelocable => {
            if reg != exp.info as u32 {
                code_abc(lex, state, OpCode::Move as u32, reg as i32, exp.info, 0)?;
            }
        }
        _ => {
            debug_assert!(exp.k == ExpressionKind::Void || exp.k == ExpressionKind::Jump);
            return Ok(()); //nothing to do...
        }
    }
    exp.info = reg as i32;
    exp.k = ExpressionKind::NonRelocable;
    Ok(())
}

pub(crate) fn set_list<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    base: i32,
    nelems: i32,
    to_store: i32,
) -> Result<(), LuaError> {
    let c = (nelems - 1) / LFIELDS_PER_FLUSH as i32 + 1;
    let b = if to_store == LUA_MULTRET { 0 } else { to_store };
    debug_assert!(to_store != 0);
    if c <= MAXARG_C as i32 {
        code_abc(lex, state, OpCode::SetList as u32, base, b, c)?;
    } else {
        code_abc(lex, state, OpCode::SetList as u32, base, b, 0)?;
        code(lex, state, c as u32, lex.lastline)?;
    }
    // free registers with list values
    lex.borrow_mut_fs(None).freereg = (base + 1) as usize;
    Ok(())
}

pub(crate) fn nil<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    from: u32,
    n: i32,
) -> Result<(), LuaError> {
    {
        let (pc, last_target, nactvar) = {
            let fs = lex.borrow_fs(None);
            (fs.next_pc(), fs.last_target, fs.nactvar)
        };
        if pc > last_target {
            //  no jumps to current position?
            if pc == 0 {
                // function start
                if from >= nactvar as u32 {
                    return Ok(());
                }
            } else {
                let previous = lex.borrow_mut_code(pc as usize - 1);
                if get_opcode(*previous) == OpCode::LoadNil {
                    let pfrom = get_arg_a(*previous);
                    let pto = get_arg_b(*previous);
                    if pfrom < from && from < pto + 1 {
                        // can connect both?
                        if from as i32 + n + 1 > pto as i32 {
                            set_arg_b(previous, (from as i32 + n) as u32 - 1);
                        }
                        return Ok(());
                    }
                }
            }
        }
    }
    // else no optimisation
    code_abc(
        lex,
        state,
        OpCode::LoadNil as u32,
        from as i32,
        from as i32 + n - 1,
        0,
    )?;
    Ok(())
}

pub(crate) fn ret<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    first: u32,
    nret: u32,
) -> Result<(), LuaError> {
    code_abc(
        lex,
        state,
        OpCode::Return as u32,
        first as i32,
        nret as i32 + 1,
        0,
    )?;
    Ok(())
}

pub(crate) fn reserve_regs<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    count: usize,
) -> Result<(), LuaError> {
    check_stack(lex, state, count)?;
    lex.borrow_mut_fs(None).freereg += count;
    Ok(())
}

pub(crate) fn check_stack<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    count: usize,
) -> Result<(), LuaError> {
    let fs = lex.borrow_mut_fs(None);
    let new_stack = fs.freereg + count;
    if new_stack > fs.f.maxstacksize {
        if new_stack > MAX_LUA_STACK {
            return lex.syntax_error(state, "function or expression too complex");
        }
        fs.f.maxstacksize = new_stack;
    }
    Ok(())
}

fn free_exp<T>(lex: &mut LexState<T>, exp: &mut ExpressionDesc) {
    if let ExpressionKind::NonRelocable = exp.k {
        free_reg(lex, exp.info as u32);
    }
}

fn free_reg<T>(lex: &mut LexState<T>, reg: u32) {
    let fs = lex.borrow_mut_fs(None);
    if !is_reg_constant(reg) && reg >= fs.nactvar as u32 {
        fs.freereg -= 1;
        debug_assert!(reg == fs.freereg as u32);
    }
}

pub(crate) fn fix_line<T>(lex: &mut LexState<T>, line: usize) {
    let fs = lex.borrow_mut_fs(None);
    let pc = fs.next_pc() as usize;
    fs.f.lineinfo[pc - 1] = line;
}

pub(crate) fn prefix<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: UnaryOp,
    e: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    let mut e2 = ExpressionDesc::default();
    e2.init(ExpressionKind::NumberConstant, 0);
    match op {
        UnaryOp::Minus => {
            if ! is_numeral(e) {
                // cannot operate on non-numeric constants
                exp2anyreg(lex, state, e)?;
            }
            code_arith(lex, state, OpCode::UnaryMinus, e, &mut e2)?;
        }
        UnaryOp::Not => {
            code_not(lex, state, e)?;
        }
        UnaryOp::Len => {
            exp2anyreg(lex, state, e)?; // cannot operate on constants
            code_arith(lex, state, OpCode::Len, e, &mut e2)?;
        }
    }
    Ok(())
}

pub(crate) fn is_numeral(e: &ExpressionDesc) -> bool {
    e.k == ExpressionKind::NumberConstant && e.t == NO_JUMP && e.f == NO_JUMP
}

fn code_not<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    discharge_vars(lex, state, exp)?;
    match exp.k {
        ExpressionKind::Nil | ExpressionKind::False => {
            exp.k = ExpressionKind::True;
        }
        ExpressionKind::Constant | ExpressionKind::NumberConstant | ExpressionKind::True => {
            exp.k = ExpressionKind::False;
        }
        ExpressionKind::Jump => {
            invert_jump(lex, exp);
        }
        ExpressionKind::Relocable | ExpressionKind::NonRelocable => {
            discharge2any_reg(lex, state, exp)?;
            free_exp(lex, exp);
            exp.info = code_abc(lex, state, OpCode::Not as u32, 0, exp.info, 0)? as i32;
            exp.k = ExpressionKind::Relocable;
        }
        _ => unreachable!(),
    }
    Ok(())
}

pub(crate) fn infix<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: BinaryOp,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    match op {
        BinaryOp::And => go_if_true(lex, state, exp),
        BinaryOp::Or => go_if_false(lex, state, exp),
        BinaryOp::Concat => exp2nextreg(lex, state, exp),
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod | BinaryOp::Pow => {
            if !exp.is_numeral() {
                exp2rk(lex, state, exp)?;
            }
            Ok(())
        },
        _ => {
            exp2rk(lex, state, exp)?;
            Ok(())
        }
    }
}

fn go_if_false<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    discharge_vars(lex, state, exp)?;
    let pc = match exp.k {
        ExpressionKind::Nil | ExpressionKind::False => {
            // always false, do nothing
            NO_JUMP
        }
        ExpressionKind::True => {
            // always jump
            jump(lex, state)?
        }
        ExpressionKind::Jump => exp.info,
        _ => jump_on_cond(lex, state, exp, 1)?,
    };
    concat_jump(lex, state, &mut exp.t, pc)?; // insert last jump in `t' list
    patch_to_here(lex, state, exp.f)?;
    exp.f = NO_JUMP;
    Ok(())
}

pub(crate) fn go_if_true<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    discharge_vars(lex, state, exp)?;
    let pc = match exp.k {
        ExpressionKind::Constant | ExpressionKind::NumberConstant | ExpressionKind::True => {
            // always true; do nothing
            NO_JUMP
        }
        ExpressionKind::False => {
            // always jump
            jump(lex, state)?
        }
        ExpressionKind::Jump => {
            invert_jump(lex, exp);
            exp.info
        }
        _ => jump_on_cond(lex, state, exp, 0)?,
    };
    concat_jump(lex, state, &mut exp.f, pc)?; // insert last jump in `f' list
    patch_to_here(lex, state, exp.t)?;
    exp.t = NO_JUMP;
    Ok(())
}

fn jump_on_cond<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
    cond: i32,
) -> Result<i32, LuaError> {
    {
        let fs = lex.borrow_mut_fs(None);
        if exp.k == ExpressionKind::Relocable {
            let ie = fs.f.code[exp.info as usize];
            if get_opcode(ie) == OpCode::Not {
                // remove previous OP_NOT
                fs.f.code.pop();
                return cond_jump(
                    lex,
                    state,
                    OpCode::Test,
                    get_arg_b(ie) as i32,
                    0,
                    if cond == 0 { 1 } else { 0 },
                );
            }
            // else go through
        }
    }
    discharge2any_reg(lex, state, exp)?;
    free_exp(lex, exp);
    cond_jump(lex, state, OpCode::TestSet, NO_REG as i32, exp.info, cond)
}

fn cond_jump<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: OpCode,
    a: i32,
    b: i32,
    c: i32,
) -> Result<i32, LuaError> {
    code_abc(lex, state, op as u32, a, b, c)?;
    jump(lex, state)
}

fn discharge2any_reg<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    if exp.k != ExpressionKind::NonRelocable {
        reserve_regs(lex, state, 1)?;
        discharge2reg(lex, state, exp, lex.borrow_fs(None).freereg as u32 - 1)?;
    }
    Ok(())
}

pub(crate) fn op_self<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    e: &mut ExpressionDesc,
    key: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    exp2anyreg(lex, state, e)?;
    free_exp(lex, e);
    let func = lex.borrow_fs(None).freereg as i32;
    reserve_regs(lex, state, 2)?;
    let c = exp2rk(lex, state, key)? as i32;
    code_abc(lex, state, OpCode::OpSelf as u32, func, e.info, c)?;
    free_exp(lex, key);
    e.info = func;
    e.k = ExpressionKind::NonRelocable;
    Ok(())
}

fn invert_jump<T>(lex: &mut LexState<T>, exp: &mut ExpressionDesc) {
    let pcref = get_jump_control(lex, exp.info);
    set_arg_a(pcref, if get_arg_a(*pcref) == 0 { 1 } else { 0 });
}

pub(crate) fn postfix<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: BinaryOp,
    exp1: &mut ExpressionDesc,
    exp2: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    match op {
        BinaryOp::And => {
            debug_assert!(exp1.t == NO_JUMP); // list must be closed
            discharge_vars(lex, state, exp2)?;
            concat_jump(lex, state, &mut exp2.f, exp1.f)?;
            *exp1 = (*exp2).clone();
        }
        BinaryOp::Or => {
            debug_assert!(exp1.f == NO_JUMP); // list must be closed
            discharge_vars(lex, state, exp2)?;
            concat_jump(lex, state, &mut exp2.t, exp1.t)?;
            *exp1 = (*exp2).clone();
        }
        BinaryOp::Concat => {
            exp2val(lex, state, exp2)?;
            let i2 = lex.get_code(exp2.info as usize);
            if exp2.k == ExpressionKind::Relocable && get_opcode(i2) == OpCode::Concat {
                debug_assert!(exp1.info as u32 == get_arg_b(i2) - 1);
                free_exp(lex, exp1);
                set_arg_b(lex.borrow_mut_code(exp2.info as usize), exp1.info as u32);
                exp1.k = ExpressionKind::Relocable;
                exp1.info = exp2.info;
            } else {
                exp2nextreg(lex, state, exp2)?; // operand must be on the 'stack'
                code_arith(lex, state, OpCode::Concat, exp1, exp2)?;
            }
        }
        BinaryOp::Add => code_arith(lex, state, OpCode::Add, exp1, exp2)?,
        BinaryOp::Sub => code_arith(lex, state, OpCode::Sub, exp1, exp2)?,
        BinaryOp::Mul => code_arith(lex, state, OpCode::Mul, exp1, exp2)?,
        BinaryOp::Div => code_arith(lex, state, OpCode::Div, exp1, exp2)?,
        BinaryOp::Mod => code_arith(lex, state, OpCode::Mod, exp1, exp2)?,
        BinaryOp::Pow => code_arith(lex, state, OpCode::Pow, exp1, exp2)?,
        BinaryOp::Eq => code_comp(lex, state, OpCode::Eq, 1, exp1, exp2)?,
        BinaryOp::Ne => code_comp(lex, state, OpCode::Eq, 0, exp1, exp2)?,
        BinaryOp::Lt => code_comp(lex, state, OpCode::Lt, 1, exp1, exp2)?,
        BinaryOp::Le => code_comp(lex, state, OpCode::Le, 1, exp1, exp2)?,
        BinaryOp::Gt => code_comp(lex, state, OpCode::Lt, 0, exp1, exp2)?,
        BinaryOp::Ge => code_comp(lex, state, OpCode::Le, 0, exp1, exp2)?,
    }
    Ok(())
}

fn code_comp<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: OpCode,
    cond: i32,
    exp1: &mut ExpressionDesc,
    exp2: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    let mut o1 = exp2rk(lex, state, exp1)?;
    let mut o2 = exp2rk(lex, state, exp2)?;
    free_exp(lex, exp2);
    free_exp(lex, exp1);
    let cond = if cond == 0 && op != OpCode::Eq {
        // exchange args to replace by `<' or `<='
        std::mem::swap(&mut o1, &mut o2);
        1
    } else {
        cond
    };
    exp1.info = cond_jump(lex, state, op, cond, o1 as i32, o2 as i32)?;
    exp1.k = ExpressionKind::Jump;
    Ok(())
}

fn code_arith<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: OpCode,
    exp1: &mut ExpressionDesc,
    exp2: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    if const_folding(op, exp1, exp2) {
        return Ok(());
    }
    let o2 = if op != OpCode::UnaryMinus && op != OpCode::Len {
        exp2rk(lex, state, exp2)?
    } else {
        0
    };
    let o1 = exp2rk(lex, state, exp1)?;
    if o1 > o2 {
        free_exp(lex, exp1);
        free_exp(lex, exp2);    
    } else {
        free_exp(lex, exp2);
        free_exp(lex, exp1);
    }
    exp1.info = code_abc(lex, state, op as u32, 0, o1 as i32, o2 as i32)? as i32;
    exp1.k = ExpressionKind::Relocable;
    Ok(())
}

fn const_folding(op: OpCode, exp1: &mut ExpressionDesc, exp2: &mut ExpressionDesc) -> bool {
    if !exp1.is_numeral() || !exp2.is_numeral() {
        return false;
    }
    let v1 = exp1.nval;
    let v2 = exp2.nval;
    let r = match op {
        OpCode::Add => v1 + v2,
        OpCode::Sub => v1 - v2,
        OpCode::Mul => v1 * v2,
        OpCode::Div => {
            if v2 == 0.0 {
                return false; // do not attempt to divide by 0
            } else {
                v1 / v2
            }
        }
        OpCode::Mod => {
            if v2 == 0.0 {
                return false;
            } else {
                v1 % v2
            }
        }
        OpCode::Pow => v1.powf(v2),
        OpCode::UnaryMinus => -v1,
        OpCode::Len => {
            return false;
        } // no constant folding for `len`
        _ => unreachable!(),
    };
    if r.is_nan() {
        return false;
    }
    exp1.nval = r;
    true
}

pub(crate) fn concat<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    l1: &mut i32,
    l2: i32,
) -> Result<(), LuaError> {
    if l2 == NO_JUMP {
        Ok(())
    } else if *l1 == NO_JUMP {
        *l1 = l2;
        Ok(())
    } else {
        let mut list = *l1;
        loop {
            let next = get_jump(lex, list);
            if next != NO_JUMP {
                list = next;
            } else {
                break;
            }
        }
        fix_jump(lex, state, list, l2)
    }
}
