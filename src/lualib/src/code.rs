//! Code generator for Lua

use crate::{
    api::LuaError,
    lex::LexState,
    limits::MAX_LUA_STACK,
    object::TValue,
    opcodes::{
        create_abc, create_abx, create_ax, get_arg_a, get_arg_b, get_arg_c, get_arg_sbx,
        get_opcode, is_reg_constant, rk_as_k, set_arg_a, set_arg_b, set_arg_c, set_arg_sbx, OpCode,
        LFIELDS_PER_FLUSH, MAXARG_AX, MAXARG_BX, MAXARG_C, MAXARG_SBX, MAX_INDEX_RK, NO_JUMP,
        NO_REG,
    },
    parser::{BinaryOp, ExpressionDesc, ExpressionKind, UnaryOp},
    state::LuaState,
    LuaFloat, LuaInteger, LUA_MULTRET,
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
        ExpressionKind::Indexed => {
            let mut opcode = OpCode::GetTabUp; // assume 't' is an upvalue
            free_reg(lex, exp.ind.idx);
            if !exp.ind.is_t_upval {
                // 't' is in a register?
                free_reg(lex, exp.ind.t);
                opcode = OpCode::GetTable;
            }
            exp.info = code_abc(
                lex,
                state,
                opcode as u32,
                0,
                exp.ind.t as i32,
                exp.ind.idx as i32,
            )? as i32;
            exp.k = ExpressionKind::Relocable;
        }
        ExpressionKind::Call | ExpressionKind::VarArg => {
            set_one_ret(lex, state, exp);
        }
        _ => (), // there is one value available (somewhere)
    }
    Ok(())
}

pub(crate) fn set_one_ret<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) {
    if exp.k == ExpressionKind::Call {
        // expression is an open function call?
        exp.k = ExpressionKind::NonRelocable;
        exp.info = get_arg_a(lex.get_code(state, exp.info as usize)) as i32;
    } else if exp.k == ExpressionKind::VarArg {
        set_arg_b(lex.borrow_mut_code(state, exp.info as usize), 2);
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
    code(lex, state, o)
}

pub(crate) fn code_abx<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: u32,
    a: i32,
    bx: u32,
) -> Result<u32, LuaError> {
    let o = create_abx(op, a, bx);
    code(lex, state, o)
}

pub(crate) fn code_k<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    reg: i32,
    k: u32,
) -> Result<u32, LuaError> {
    if k <= MAXARG_BX as u32 {
        code_abx(lex, state, OpCode::LoadK as u32, reg, k)
    } else {
        let p = code_abx(lex, state, OpCode::LoadKx as u32, reg, 0)?;
        code_extra_arg(lex, state, k)?;
        Ok(p)
    }
}

fn code_extra_arg<T>(lex: &mut LexState<T>, state: &mut LuaState, k: u32) -> Result<u32, LuaError> {
    debug_assert!(k <= MAXARG_AX as u32);
    code(lex, state, create_ax(OpCode::ExtraArg as u32, k))
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

fn code<T>(lex: &mut LexState<T>, state: &mut LuaState, o: u32) -> Result<u32, LuaError> {
    discharge_jpc(lex, state)?; // pc' will change
    let line = lex.lastline;
    let proto = lex.borrow_mut_proto(state, None);
    let pc = proto.next_pc() as u32;
    proto.code.push(o);
    proto.lineinfo.push(line);
    Ok(pc)
}

fn discharge_jpc<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let pc = lex.next_pc(state) as i32;
    let jpc = lex.borrow_fs(None).jpc;
    patch_list_aux(lex, state, jpc, pc, NO_REG, pc)?;
    lex.borrow_mut_fs(None).jpc = NO_JUMP;
    Ok(())
}

pub(crate) fn patch_close<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    list: i32,
    level: usize,
) -> Result<(), LuaError> {
    // argument is +1 to reserve 0 as non-op
    let level = level + 1;
    let mut list = list;
    while list != NO_JUMP {
        let next = get_jump(lex, state, list);
        let to_fix = lex.borrow_mut_code(state, list as usize);
        set_arg_a(to_fix, level as u32);
        list = next;
    }
    Ok(())
}

pub(crate) fn patch_list<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    list: i32,
    target: i32,
) -> Result<(), LuaError> {
    let pc = lex.next_pc(state) as i32;
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
        let next = get_jump(lex, state, jpc);
        if patch_test_reg(lex, state, jpc, reg) {
            fix_jump(lex, state, jpc, vtarget)?;
        } else {
            fix_jump(lex, state, jpc, dtarget)?; // jump to default target
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
    let jmp = lex.borrow_mut_code(state, pc as usize);
    let offset = dest - (pc + 1);
    debug_assert!(dest != NO_JUMP);
    if offset.abs() > MAXARG_SBX {
        return lex.syntax_error(state, "controle structure too long");
    }
    set_arg_sbx(jmp, offset);
    Ok(())
}

fn patch_test_reg<T>(lex: &mut LexState<T>, state: &mut LuaState, node: i32, reg: u32) -> bool {
    let i = get_jump_control(lex, state, node);
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
        ExpressionKind::Indexed => {
            let opcode = if var.ind.is_t_upval {
                OpCode::SetTabUp
            } else {
                OpCode::SetTable
            };
            let e = exp2rk(lex, state, ex)?;
            code_abc(
                lex,
                state,
                opcode as u32,
                var.ind.t as i32,
                var.ind.idx as i32,
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
    debug_assert!(t.t == t.f);
    t.ind.t = t.info as u32;
    t.ind.idx = exp2rk(lex, state, k)?;
    t.ind.is_t_upval = t.k == ExpressionKind::UpValue;
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
        ExpressionKind::FloatConstant
        | ExpressionKind::IntegerConstant
        | ExpressionKind::True
        | ExpressionKind::False
        | ExpressionKind::Nil => {
            if lex.borrow_proto(state, None).k.len() <= MAX_INDEX_RK {
                ex.info = match ex.k {
                    ExpressionKind::Nil => nil_constant(lex, state) as i32,
                    ExpressionKind::FloatConstant => float_constant(lex, state, ex.nval) as i32,
                    ExpressionKind::IntegerConstant => integer_constant(lex, state, ex.ival) as i32,
                    _ => bool_constant(lex, state, ex.k == ExpressionKind::True) as i32,
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

fn bool_constant<T>(lex: &mut LexState<T>, state: &mut LuaState, val: bool) -> usize {
    let o = TValue::Boolean(val);
    lex.borrow_mut_fs(None).add_constant(state, o.clone(), o)
}

pub(crate) fn float_constant<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    val: LuaFloat,
) -> usize {
    lex.borrow_mut_fs(None).float_constant(state, val)
}
pub(crate) fn integer_constant<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    val: LuaInteger,
) -> usize {
    lex.borrow_mut_fs(None).integer_constant(state, val)
}

fn nil_constant<T>(lex: &mut LexState<T>, state: &mut LuaState) -> usize {
    // cannot use nil as key; instead use table itself to represent nil
    lex.borrow_mut_fs(None)
        .add_constant(state, TValue::Nil, TValue::Nil)
}

pub(crate) fn exp2val<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    ex: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    if ex.has_jumps() {
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
        if !ex.has_jumps() {
            return Ok(ex.info as u32); // exp is already in a register
        }
        if ex.info == lex.borrow_fs(None).nactvar as i32 {
            // reg. is not a local?
            exp2reg(lex, state, ex, ex.info as u32)?; // put value on it
            return Ok(ex.info as u32);
        }
    }
    exp2nextreg(lex, state, ex)?; // default
    Ok(ex.info as u32)
}

fn get_jump_control<'a, T>(
    lex: &'a mut LexState<T>,
    state: &'a mut LuaState,
    pc: i32,
) -> &'a mut u32 {
    let i = lex.get_code(state, pc as usize - 1);
    let op = get_opcode(i);
    if pc >= 1 && op.is_test() {
        lex.borrow_mut_code(state, pc as usize - 1)
    } else {
        lex.borrow_mut_code(state, pc as usize)
    }
}

/// get the new value of pc for this jump instruction
fn get_jump<T>(lex: &mut LexState<T>, state: &LuaState, jpc: i32) -> i32 {
    let i = lex.get_code(state, jpc as usize);
    let offset = get_arg_sbx(i);
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

pub(crate) fn exp2anyregup<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    if exp.k != ExpressionKind::UpValue || exp.has_jumps() {
        exp2anyreg(lex, state, exp).map(|_| ())
    } else {
        Ok(())
    }
}

fn exp2reg<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
    reg: u32,
) -> Result<(), LuaError> {
    discharge2reg(lex, state, exp, reg)?;
    if exp.k == ExpressionKind::Jump {
        concat_jump(lex, state, &mut exp.t, exp.info)?; // put this jump in `t' list
    }
    if exp.has_jumps() {
        let mut p_f = NO_JUMP; // position of an eventual LOAD false
        let mut p_t = NO_JUMP; // position of an eventual LOAD true
        if need_value(lex, state, exp.t) || need_value(lex, state, exp.f) {
            let fj = if let ExpressionKind::Jump = exp.k {
                NO_JUMP
            } else {
                jump(lex, state)?
            };
            p_f = code_label(lex, state, reg, 0, 1)?;
            p_t = code_label(lex, state, reg, 1, 0)?;
            patch_to_here(lex, state, fj)?;
        }
        let final_pc = get_label(lex, state); // position after whole expression
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
        set_arg_c(lex.borrow_mut_code(state, pc), (nresults + 1) as u32);
    } else if exp.k == ExpressionKind::VarArg {
        let pc = exp.info as usize;
        {
            set_arg_b(lex.borrow_mut_code(state, pc), (nresults + 1) as u32);
            let freereg = lex.borrow_fs(None).freereg as u32;
            set_arg_a(lex.borrow_mut_code(state, pc), freereg);
        }
        reserve_regs(lex, state, 1)?;
    }
    Ok(())
}

/// returns current `pc' and marks it as a jump target (to avoid wrong
///  optimizations with consecutive instructions not in the same basic block).
pub(crate) fn get_label<T>(lex: &mut LexState<T>, state: &mut LuaState) -> i32 {
    let pc = lex.next_pc(state) as i32;
    let fs = lex.borrow_mut_fs(None);
    fs.last_target = pc;
    fs.last_target
}

pub(crate) fn patch_to_here<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    list: i32,
) -> Result<(), LuaError> {
    get_label(lex, state);
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
    get_label(lex, state); // those instructions may be jump targets
    Ok(code_abc(lex, state, OpCode::LoadBool as u32, a as i32, b, jump)? as i32)
}

/// check whether list has any jump that do not produce a value
/// (or produce an inverted value)
fn need_value<T>(lex: &mut LexState<T>, state: &mut LuaState, list: i32) -> bool {
    let mut list = list;
    while list != NO_JUMP {
        let i = get_jump_control(lex, state, list);
        if get_opcode(*i) != OpCode::TestSet {
            return true;
        }
        list = get_jump(lex, state, list);
    }
    false
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
        let mut next = get_jump(lex, state, list);
        while next != NO_JUMP {
            list = next;
            next = get_jump(lex, state, list)
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
        ExpressionKind::FloatConstant => {
            let kid = lex.borrow_mut_fs(None).float_constant(state, exp.nval) as u32;
            code_abx(lex, state, OpCode::LoadK as u32, reg as i32, kid)?;
        }
        ExpressionKind::IntegerConstant => {
            let kid = lex.borrow_mut_fs(None).integer_constant(state, exp.ival) as u32;
            code_abx(lex, state, OpCode::LoadK as u32, reg as i32, kid)?;
        }
        ExpressionKind::Relocable => {
            let pc = exp.info as usize;
            set_arg_a(lex.borrow_mut_code(state, pc), reg);
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
        code(lex, state, c as u32)?;
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
            let pc = lex.next_pc(state) as i32;
            (pc, fs.last_target, fs.nactvar)
        };
        if pc > last_target {
            //  no jumps to current position?
            if pc == 0 {
                // function start
                if from >= nactvar as u32 {
                    return Ok(());
                }
            } else {
                let previous = lex.borrow_mut_code(state, pc as usize - 1);
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
    let proto = lex.borrow_mut_proto(state, None);
    let new_stack = lex.borrow_fs(None).freereg + count;
    if new_stack > proto.maxstacksize {
        if new_stack > MAX_LUA_STACK {
            return lex.syntax_error(state, "function or expression too complex");
        }
        proto.maxstacksize = new_stack;
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

pub(crate) fn fix_line<T>(lex: &mut LexState<T>, state: &mut LuaState, line: usize) {
    let proto = lex.borrow_mut_proto(state, None);
    let pc = proto.next_pc() as usize;
    proto.lineinfo[pc - 1] = line;
}

/// Apply prefix operation 'op' to expression 'e'
pub(crate) fn prefix<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: UnaryOp,
    e: &mut ExpressionDesc,
    line: usize,
) -> Result<(), LuaError> {
    let mut e2 = ExpressionDesc::default();
    e2.init(ExpressionKind::IntegerConstant, 0);
    match op {
        UnaryOp::Minus => {
            if !const_folding(OpCode::UnaryMinus, e, &mut e2) {
                code_unexpval(lex, state, OpCode::UnaryMinus, e, line)?
            }
        }
        UnaryOp::BinaryNot => {
            if !const_folding(OpCode::BinaryNot, e, &mut e2) {
                code_unexpval(lex, state, OpCode::BinaryNot, e, line)?
            }
        }
        UnaryOp::Not => {
            code_not(lex, state, e)?;
        }
        UnaryOp::Len => code_unexpval(lex, state, OpCode::Len, e, line)?,
    }
    Ok(())
}

/// Emit code for unary expressions that "produce values"
/// (everything but 'not').
/// Expression to produce final result will be encoded in 'e'.
fn code_unexpval<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: OpCode,
    e: &mut ExpressionDesc,
    line: usize,
) -> Result<(), LuaError> {
    let r = exp2anyreg(lex, state, e)? as i32; // opcodes operate only on registers
    free_exp(lex, e);
    e.info = code_abc(lex, state, op as u32, 0, r, 0)? as i32;
    e.k = ExpressionKind::Relocable; // all those operations are relocatable
    fix_line(lex, state, line);
    Ok(())
}

/// Code 'not e', doing constant folding.
fn code_not<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    discharge_vars(lex, state, exp)?;
    match exp.k {
        ExpressionKind::Nil | ExpressionKind::False => {
            exp.k = ExpressionKind::True; // true == not nil == not false
        }
        ExpressionKind::Constant
        | ExpressionKind::FloatConstant
        | ExpressionKind::IntegerConstant
        | ExpressionKind::True => {
            exp.k = ExpressionKind::False; // false == not "x" == not 0.5 == not 1 == not true
        }
        ExpressionKind::Jump => {
            negate_condition(lex, state, exp);
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

/// Process 1st operand 'v' of binary operation 'op' before reading
/// 2nd operand.
pub(crate) fn infix<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: BinaryOp,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    match op {
        BinaryOp::And => go_if_true(lex, state, exp), // go ahead only if 'v' is true
        BinaryOp::Or => go_if_false(lex, state, exp), // go ahead only if 'v' is false
        BinaryOp::Concat => exp2nextreg(lex, state, exp), // operand must be on the 'stack'
        BinaryOp::Add
        | BinaryOp::Sub
        | BinaryOp::Mul
        | BinaryOp::Div
        | BinaryOp::IntDiv
        | BinaryOp::Mod
        | BinaryOp::Pow
        | BinaryOp::BinaryAnd
        | BinaryOp::BinaryOr
        | BinaryOp::BinaryXor
        | BinaryOp::Shl
        | BinaryOp::Shr => {
            if !exp.is_numeral() {
                exp2rk(lex, state, exp)?;
            }
            // else keep numeral, which may be folded with 2nd operand
            Ok(())
        }
        _ => {
            exp2rk(lex, state, exp)?;
            Ok(())
        }
    }
}

pub(crate) fn go_if_false<T>(
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
        ExpressionKind::Jump => exp.info,
        _ => jump_on_cond(lex, state, exp, 1)?,
    };
    concat_jump(lex, state, &mut exp.t, pc)?; // insert last jump in `t' list
    patch_to_here(lex, state, exp.f)?;
    exp.f = NO_JUMP;
    Ok(())
}

/// Emit code to go through if 'e' is true, jump otherwise.
pub(crate) fn go_if_true<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    discharge_vars(lex, state, exp)?;
    let pc = match exp.k {
        ExpressionKind::Constant
        | ExpressionKind::FloatConstant
        | ExpressionKind::IntegerConstant
        | ExpressionKind::True => {
            // always true; do nothing
            NO_JUMP
        }
        ExpressionKind::Jump => {
            negate_condition(lex, state, exp); // jump when it is false
            exp.info // save jump position
        }
        _ => jump_on_cond(lex, state, exp, 0)?,
    };
    concat_jump(lex, state, &mut exp.f, pc)?; // insert last jump in `f' list
    patch_to_here(lex, state, exp.t)?;
    exp.t = NO_JUMP;
    Ok(())
}

/// Emit instruction to jump if 'e' is 'cond' (that is, if 'cond'
/// is true, code will jump if 'e' is true.) Return jump position.
/// Optimize when 'e' is 'not' something, inverting the condition
/// and removing the 'not'.
fn jump_on_cond<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
    cond: i32,
) -> Result<i32, LuaError> {
    {
        let proto = lex.borrow_mut_proto(state, None);
        if exp.k == ExpressionKind::Relocable {
            let ie = proto.code[exp.info as usize];
            if get_opcode(ie) == OpCode::Not {
                // remove previous OP_NOT
                proto.code.pop();
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
    let ereg = e.info;
    free_exp(lex, e);
    let func = lex.borrow_fs(None).freereg as i32;
    e.info = func;
    e.k = ExpressionKind::NonRelocable;
    reserve_regs(lex, state, 2)?;
    let c = exp2rk(lex, state, key)? as i32;
    code_abc(lex, state, OpCode::OpSelf as u32, func, ereg, c)?;
    free_exp(lex, key);
    Ok(())
}

/// Negate condition 'e' (where 'e' is a comparison).
fn negate_condition<T>(lex: &mut LexState<T>, state: &mut LuaState, exp: &mut ExpressionDesc) {
    let pcref = get_jump_control(lex, state, exp.info);
    set_arg_a(pcref, 1 - get_arg_a(*pcref));
}

/// Finalize code for binary operation, after reading 2nd operand.
/// For '(a .. b .. c)' (which is '(a .. (b .. c))', because
/// concatenation is right associative), merge second CONCAT into first
/// one.
pub(crate) fn pos_fix<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: BinaryOp,
    exp1: &mut ExpressionDesc,
    exp2: &mut ExpressionDesc,
    line: usize,
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
            let i2 = lex.get_code(state, exp2.info as usize);
            if exp2.k == ExpressionKind::Relocable && get_opcode(i2) == OpCode::Concat {
                debug_assert!(exp1.info as u32 == get_arg_b(i2) - 1);
                free_exp(lex, exp1);
                set_arg_b(
                    lex.borrow_mut_code(state, exp2.info as usize),
                    exp1.info as u32,
                );
                exp1.init(ExpressionKind::Relocable, exp2.info);
            } else {
                exp2nextreg(lex, state, exp2)?; // operand must be on the 'stack'
                code_bin_expval(lex, state, OpCode::Concat, exp1, exp2, line)?;
            }
        }
        BinaryOp::Add => code_arith(lex, state, OpCode::Add, exp1, exp2, line)?,
        BinaryOp::Sub => code_arith(lex, state, OpCode::Sub, exp1, exp2, line)?,
        BinaryOp::Mul => code_arith(lex, state, OpCode::Mul, exp1, exp2, line)?,
        BinaryOp::Div => code_arith(lex, state, OpCode::Div, exp1, exp2, line)?,
        BinaryOp::IntDiv => code_arith(lex, state, OpCode::IntegerDiv, exp1, exp2, line)?,
        BinaryOp::Mod => code_arith(lex, state, OpCode::Mod, exp1, exp2, line)?,
        BinaryOp::Pow => code_arith(lex, state, OpCode::Pow, exp1, exp2, line)?,
        BinaryOp::BinaryAnd => code_arith(lex, state, OpCode::BinaryAnd, exp1, exp2, line)?,
        BinaryOp::BinaryOr => code_arith(lex, state, OpCode::BinaryOr, exp1, exp2, line)?,
        BinaryOp::BinaryXor => code_arith(lex, state, OpCode::BinaryXor, exp1, exp2, line)?,
        BinaryOp::Shl => code_arith(lex, state, OpCode::Shl, exp1, exp2, line)?,
        BinaryOp::Shr => code_arith(lex, state, OpCode::Shr, exp1, exp2, line)?,
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
    line: usize,
) -> Result<(), LuaError> {
    if !const_folding(op, exp1, exp2) {
        code_bin_expval(lex, state, op, exp1, exp2, line)?;
    }
    Ok(())
}

/// Emit code for binary expressions that "produce values"
/// (everything but logical operators 'and'/'or' and comparison
/// operators).
/// Expression to produce final result will be encoded in 'e1'.
/// Because 'luaK_exp2RK' can free registers, its calls must be
/// in "stack order" (that is, first on 'e2', which may have more
/// recent registers to be released).
fn code_bin_expval<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: OpCode,
    exp1: &mut ExpressionDesc,
    exp2: &mut ExpressionDesc,
    line: usize,
) -> Result<(), LuaError> {
    let rk1 = exp2rk(lex, state, exp1)? as i32;
    let rk2 = exp2rk(lex, state, exp2)? as i32;
    free_exps(lex, exp1, exp2);
    exp1.init(
        ExpressionKind::Relocable,
        code_abc(lex, state, op as u32, 0, rk1, rk2)? as i32,
    );
    fix_line(lex, state, line);
    Ok(())
}

/// Free registers used by expressions 'e1' and 'e2' (if any) in proper
/// order.
fn free_exps<T>(lex: &mut LexState<T>, exp1: &mut ExpressionDesc, exp2: &mut ExpressionDesc) {
    let r1 = if exp1.k == ExpressionKind::NonRelocable {
        exp1.info
    } else {
        -1
    };
    let r2 = if exp2.k == ExpressionKind::NonRelocable {
        exp2.info
    } else {
        -1
    };
    if r1 > r2 {
        free_exp(lex, exp1);
        free_exp(lex, exp2);
    } else {
        free_exp(lex, exp2);
        free_exp(lex, exp1);
    }
}

/// Try to "constant-fold" an operation; return 1 iff successful.
/// (In this case, 'e1' has the final result.)
fn const_folding(op: OpCode, exp1: &mut ExpressionDesc, exp2: &mut ExpressionDesc) -> bool {
    let mut v1 = TValue::Nil;
    let mut v2 = TValue::Nil;
    if !exp1.to_numeral(&mut v1) || !exp2.to_numeral(&mut v2) || !is_op_valid(op, &v1, &v2) {
        return false; // non-numeric operands or not safe to fold
    }
    let res = arith(op, &v1, &v2);
    if res.is_float() {
        let v = res.get_float_value();
        if v.is_nan() || v == 0.0 {
            // folds neither NaN nor 0.0 (to avoid problems with -0.0)
            return false;
        }
        exp1.k = ExpressionKind::FloatConstant;
        exp1.nval = v;
    } else if res.is_integer() {
        let i = res.get_integer_value();
        exp1.k = ExpressionKind::IntegerConstant;
        exp1.ival = i;
    } else {
        unreachable!()
    }
    true
}

pub(crate) fn arith(op: OpCode, v1: &TValue, v2: &TValue) -> TValue {
    match op {
        OpCode::BinaryAnd
        | OpCode::BinaryOr
        | OpCode::BinaryXor
        | OpCode::Shl
        | OpCode::Shr
        | OpCode::BinaryNot => {
            // operate only on integer
            if let (Ok(i1), Ok(i2)) = (v1.into_integer(), v2.into_integer()) {
                return TValue::Integer(int_arith(op, i1, i2));
            }
        }
        OpCode::Div | OpCode::Pow => {
            // operate only on floats
            if let (Ok(f1), Ok(f2)) = (v1.into_float(), v2.into_float()) {
                return TValue::Float(num_arith(op, f1, f2));
            }
        }
        _ => {
            if v1.is_integer() && v2.is_integer() {
                return TValue::Integer(int_arith(
                    op,
                    v1.get_integer_value(),
                    v2.get_integer_value(),
                ));
            } else if let (Ok(f1), Ok(f2)) = (v1.into_float(), v2.into_float()) {
                return TValue::Float(num_arith(op, f1, f2));
            }
        }
    }
    // TODO metamethods
    todo!()
}

fn int_arith(op: OpCode, i1: LuaInteger, i2: LuaInteger) -> LuaInteger {
    match op {
        OpCode::Add => i1 + i2,
        OpCode::Sub => i1 - i2,
        OpCode::Mul => i1 * i2,
        OpCode::Mod => i1 % i2,
        OpCode::IntegerDiv => i1 / i2,
        OpCode::BinaryAnd => i1 & i2,
        OpCode::BinaryOr => i1 | i2,
        OpCode::BinaryXor => i1 ^ i2,
        OpCode::Shl => i1 << i2,
        OpCode::Shr => i1 >> i2,
        OpCode::UnaryMinus => -i1,
        OpCode::BinaryNot => !i1,
        _ => 0,
    }
}

fn num_arith(op: OpCode, i1: LuaFloat, i2: LuaFloat) -> LuaFloat {
    match op {
        OpCode::Add => i1 + i2,
        OpCode::Sub => i1 - i2,
        OpCode::Mul => i1 * i2,
        OpCode::Mod => i1 % i2,
        OpCode::Div => i1 / i2,
        OpCode::Pow => i1.powf(i2),
        OpCode::IntegerDiv => (i1 / i2).floor(),
        OpCode::UnaryMinus => -i1,
        _ => 0.0,
    }
}

/// Return false if folding can raise an error.
/// Bitwise operations need operands convertible to integers; division
/// operations cannot have 0 as divisor.
fn is_op_valid(op: OpCode, v1: &TValue, v2: &TValue) -> bool {
    match op {
        OpCode::BinaryAnd
        | OpCode::BinaryOr
        | OpCode::BinaryXor
        | OpCode::Shl
        | OpCode::Shr
        | OpCode::BinaryNot => {
            v1.into_integer().is_ok() && v2.into_integer().is_ok()
            // conversion errors
        }
        OpCode::Div | OpCode::IntegerDiv | OpCode::Mod => match v1.into_integer() {
            Ok(i) if i == 0 => false,
            Ok(_) => true,
            Err(_) => false,
        },
        _ => true, // everything else is valid
    }
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
            let next = get_jump(lex, state, list);
            if next != NO_JUMP {
                list = next;
            } else {
                break;
            }
        }
        fix_jump(lex, state, list, l2)
    }
}
