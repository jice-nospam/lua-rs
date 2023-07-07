//! Code generator for Lua

use crate::{
    api::LuaError,
    lex::LexState,
    object::TValue,
    opcodes::{
        create_abck, create_abx, create_ax, create_sj, get_arg_a, get_arg_b, get_arg_k, get_arg_sj,
        get_opcode, set_arg_a, set_arg_b, set_arg_c, set_arg_k, set_arg_sj, set_opcode, OpCode,
        LFIELDS_PER_FLUSH, MAXARG_A, MAXARG_AX, MAXARG_B, MAXARG_BX, MAXARG_C, MAXARG_SJ,
        MAX_INDEX_RK, NO_JUMP, NO_REG, OFFSET_SBX, OFFSET_SC, OFFSET_SJ,
    },
    parser::{BinaryOp, ExpressionDesc, ExpressionKind, UnaryOp},
    state::LuaState,
    tm::TagMethod,
    LuaFloat, LuaInteger, LUA_MULTRET,
};

/// Maximum number of registers in a Lua function (must fit in 8 bits)
pub(crate) const MAX_REGS: usize = 255;

/// Ensure that expression 'e' is not a variable (nor a <const>).
/// (Expression still may have jump lists.)
pub(crate) fn discharge_vars<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    match exp.k {
        ExpressionKind::CompileTimeConst => const2exp(const2val(lex, exp), exp),
        ExpressionKind::LocalRegister => {
            // already in a register
            // becomes a non-relocatable value
            exp.init(ExpressionKind::NonRelocable, exp.var_ridx);
        }
        ExpressionKind::UpValue => {
            //  move value to some (pending) register
            exp.info = code_abc(lex, state, OpCode::GetUpVal as u32, 0, exp.info, 0)? as i32;
            exp.k = ExpressionKind::Relocable;
        }
        ExpressionKind::IndexedUpvalue => {
            exp.info = code_abc(
                lex,
                state,
                OpCode::GetTabUp as u32,
                0,
                exp.ind_t as i32,
                exp.ind_idx,
            )? as i32;
            exp.k = ExpressionKind::Relocable;
        }
        ExpressionKind::IndexedInteger => {
            free_reg(lex, exp.ind_t as u32);
            exp.info = code_abc(
                lex,
                state,
                OpCode::GetI as u32,
                0,
                exp.ind_t as i32,
                exp.ind_idx,
            )? as i32;
            exp.k = ExpressionKind::Relocable;
        }
        ExpressionKind::IndexedString => {
            free_reg(lex, exp.ind_t as u32);
            exp.info = code_abc(
                lex,
                state,
                OpCode::GetField as u32,
                0,
                exp.ind_t as i32,
                exp.ind_idx,
            )? as i32;
            exp.k = ExpressionKind::Relocable;
        }
        ExpressionKind::Indexed => {
            free_regs(lex, exp.ind_t as u32, exp.ind_idx as u32);
            exp.info = code_abc(
                lex,
                state,
                OpCode::GetTable as u32,
                0,
                exp.ind_t as i32,
                exp.ind_idx,
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

/// Free two registers in proper order
fn free_regs<T>(lex: &mut LexState<T>, r1: u32, r2: u32) {
    if r1 > r2 {
        free_reg(lex, r1);
        free_reg(lex, r2);
    } else {
        free_reg(lex, r2);
        free_reg(lex, r1);
    }
}

/// Convert a constant in 'v' into an expression description 'e'
fn const2exp(v: TValue, e: &mut ExpressionDesc) {
    match v {
        TValue::Integer(i) => {
            e.k = ExpressionKind::IntegerConstant;
            e.ival = i;
        }
        TValue::Float(n) => {
            e.k = ExpressionKind::FloatConstant;
            e.nval = n;
        }
        TValue::Boolean(b) => {
            e.k = if b {
                ExpressionKind::True
            } else {
                ExpressionKind::False
            };
        }
        TValue::Nil => {
            e.k = ExpressionKind::Nil;
        }
        TValue::String(s) => {
            e.k = ExpressionKind::StringConstant;
            e.strval = (*s).to_owned();
        }
        _ => unreachable!(),
    }
}

/// Get the constant value from a constant expression
fn const2val<T>(lex: &mut LexState<T>, exp: &ExpressionDesc) -> TValue {
    debug_assert!(exp.k == ExpressionKind::CompileTimeConst);
    lex.dyd.actvar[exp.info as usize].k.clone()
}

/// Fix an expression to return one result.
/// If expression is not a multi-ret expression (function call or
/// vararg), it already returns one result, so nothing needs to be done.
/// Function calls become NonRelocable expressions (as its result comes
/// fixed in the base register of the call), while vararg expressions
/// become Relocable (as OpCode::VarArg puts its results where it wants).
/// (Calls are created returning one result, so that does not need
/// to be fixed.)
pub(crate) fn set_one_ret<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) {
    if exp.k == ExpressionKind::Call {
        // expression is an open function call?
        // already returns 1 value
        exp.k = ExpressionKind::NonRelocable;
        exp.info = get_arg_a(lex.get_code(state, exp.info as usize)) as i32;
    } else if exp.k == ExpressionKind::VarArg {
        set_arg_c(lex.borrow_mut_code(state, exp.info as usize), 2);
        exp.k = ExpressionKind::Relocable; // can relocate its simple resul
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
    code_abck(lex, state, op, a, b, c, 0)
}

/// Format and emit an 'iABx' instruction.
pub(crate) fn code_abx<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    o: u32,
    a: i32,
    bx: u32,
) -> Result<u32, LuaError> {
    debug_assert!(OpCode::try_from(o).unwrap().is_abx());
    debug_assert!(a as usize <= MAXARG_A && bx as usize <= MAXARG_BX);
    let o = create_abx(o, a, bx);
    code(lex, state, o)
}

/// Emit a "load constant" instruction, using either 'OpCode::LoadK'
/// (if constant index 'k' fits in 18 bits) or an 'OpCode::LoadKX'
/// instruction with "extra argument".
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

fn code_abrk<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    o: u32,
    a: i32,
    b: i32,
    ec: &mut ExpressionDesc,
) -> Result<u32, LuaError> {
    let k = exp2rk(lex, state, ec)? as u32;
    code_abck(lex, state, o, a, b, ec.info, k)
}

/// Format and emit an 'iABC' instruction. (Assertions check consistency
/// of parameters versus opcode.)
fn code_abck<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    o: u32,
    a: i32,
    b: i32,
    c: i32,
    k: u32,
) -> Result<u32, LuaError> {
    debug_assert!({
        let op = OpCode::try_from(o).unwrap();
        op.is_abc() || op.is_a() || op.is_ak() || op.is_absc() || op.is_ac()
    });
    debug_assert!(
        a <= MAXARG_A as i32 && b <= MAXARG_B as i32 && c <= MAXARG_C as i32 && (k & !1) == 0
    );
    code(lex, state, create_abck(o, a, b, c, k))
}

/// Emit an "extra argument" instruction (format 'iAx')
fn code_extra_arg<T>(lex: &mut LexState<T>, state: &mut LuaState, a: u32) -> Result<u32, LuaError> {
    debug_assert!(a <= MAXARG_AX as u32);
    code(lex, state, create_ax(OpCode::ExtraArg as u32, a))
}

/// Format and emit an 'isJ' instruction.
pub(crate) fn code_sj<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    o: u32,
    sj: i32,
    k: i32,
) -> Result<u32, LuaError> {
    let j = sj + OFFSET_SJ;
    debug_assert!(OpCode::try_from(o).unwrap().is_sj());
    debug_assert!(j as usize <= MAXARG_SJ && (k & !1) == 0);
    let o = create_sj(o, j as u32, k as u32);
    code(lex, state, o)
}
/// Format and emit an 'iAsBx' instruction.
pub(crate) fn code_asbx<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    o: u32,
    a: i32,
    bc: i32,
) -> Result<u32, LuaError> {
    let b = bc + OFFSET_SBX;
    debug_assert!(OpCode::try_from(o).unwrap().is_asbx());
    debug_assert!(a as usize <= MAXARG_A && b as usize <= MAXARG_BX);
    let o = create_abx(o, a, b as u32);
    code(lex, state, o)
}

/// Emit instruction 'i', checking for array sizes and saving also its
/// line information. Return 'i' position.
pub(crate) fn code<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    o: u32,
) -> Result<u32, LuaError> {
    let line = lex.lastline;
    let proto = lex.borrow_mut_proto(state, None);
    let pc = proto.next_pc() as u32;
    proto.code.push(o); // put new instruction in code array
    proto.lineinfo.push(line);
    Ok(pc) // index of new instruction
}

/// Path all jumps in 'list' to jump to 'target'.
/// (The assert means that we cannot fix a jump to a forward address
/// because we only know addresses once code is generated.)
pub(crate) fn patch_list<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    list: i32,
    target: i32,
) -> Result<(), LuaError> {
    let pc = lex.next_pc(state) as i32;
    debug_assert!(target <= pc);
    patch_list_aux(lex, state, list, target, NO_REG, target)
}

/// Traverse a list of tests, patching their destination address and
/// registers: tests producing values jump to 'vtarget' (and put their
/// values in 'reg'), other tests jump to 'dtarget'.
fn patch_list_aux<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    list: i32,
    vtarget: i32,
    reg: u32,
    dtarget: i32,
) -> Result<(), LuaError> {
    let mut jpc = list;
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

/// Fix jump instruction at position 'pc' to jump to 'dest'.
/// (Jump addresses are relative in Lua)
fn fix_jump<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    pc: i32,
    dest: i32,
) -> Result<(), LuaError> {
    let jmp = lex.borrow_mut_code(state, pc as usize);
    let offset = dest - (pc + 1);
    debug_assert!(dest != NO_JUMP);
    if !(offset >= -OFFSET_SJ && offset <= (MAXARG_SJ - OFFSET_SJ as usize) as i32) {
        return lex.syntax_error(state, "controle structure too long");
    }
    debug_assert!(get_opcode(*jmp) == OpCode::Jmp);
    set_arg_sj(jmp, offset);
    Ok(())
}

/// Patch destination register for a TESTSET instruction.
/// If instruction in position 'node' is not a TESTSET, return 0 ("fails").
/// Otherwise, if 'reg' is not 'NO_REG', set it as the destination
/// register. Otherwise, change instruction to a simple 'TEST' (produces
/// no register value)
fn patch_test_reg<T>(lex: &mut LexState<T>, state: &mut LuaState, node: i32, reg: u32) -> bool {
    let i = get_jump_control(lex, state, node);
    if get_opcode(*i) != OpCode::TestSet {
        return false; // cannot patch other instructions
    } else if reg != NO_REG && reg != get_arg_b(*i) {
        set_arg_a(i, reg);
    } else {
        // no register to put value or register already has the value
        // change instruction to simple test
        *i = create_abck(
            OpCode::Test as u32,
            get_arg_b(*i) as i32,
            0,
            0,
            get_arg_k(*i),
        );
    }
    true
}

/// Generate code to store result of expression 'ex' into variable 'var'.
pub(crate) fn store_var<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    var: &ExpressionDesc,
    ex: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    match var.k {
        ExpressionKind::LocalRegister => {
            free_exp(lex, ex);
            return exp2reg(lex, state, ex, var.var_ridx as u32); // compute 'ex' into proper place
        }
        ExpressionKind::UpValue => {
            let e = exp2anyreg(lex, state, ex)?;
            code_abc(lex, state, OpCode::SetupVal as u32, e as i32, var.info, 0)?;
        }
        ExpressionKind::IndexedUpvalue => {
            code_abrk(
                lex,
                state,
                OpCode::SetTabUp as u32,
                var.ind_t as i32,
                var.ind_idx,
                ex,
            )?;
        }
        ExpressionKind::IndexedInteger => {
            code_abrk(
                lex,
                state,
                OpCode::SetI as u32,
                var.ind_t as i32,
                var.ind_idx,
                ex,
            )?;
        }
        ExpressionKind::IndexedString => {
            code_abrk(
                lex,
                state,
                OpCode::SetField as u32,
                var.ind_t as i32,
                var.ind_idx,
                ex,
            )?;
        }
        ExpressionKind::Indexed => {
            code_abrk(
                lex,
                state,
                OpCode::SetTable as u32,
                var.ind_t as i32,
                var.ind_idx,
                ex,
            )?;
        }
        _ => {
            unreachable!(); // invalid var kind to store
        }
    }
    free_exp(lex, ex);
    Ok(())
}

/// Create expression 't[k]'. 't' must have its final result already in a
/// register or upvalue. Upvalues can only be indexed by literal strings.
/// Keys can be literal strings in the constant table or arbitrary
/// values in registers.
pub(crate) fn indexed<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    t: &mut ExpressionDesc,
    k: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    if k.k == ExpressionKind::StringConstant {
        str2k(lex, state, k);
    }
    debug_assert!(
        !t.has_jumps()
            && (t.k == ExpressionKind::LocalRegister
                || t.k == ExpressionKind::NonRelocable
                || t.k == ExpressionKind::UpValue)
    );
    if t.k == ExpressionKind::UpValue && !is_k_str(lex, state, k) {
        // upvalue indexed by non 'Kstr'?
        exp2anyreg(lex, state, t)?; // put it in a register
    }
    if t.k == ExpressionKind::UpValue {
        t.ind_t = t.info as usize; // upvalue index
        t.ind_idx = k.info; // literal string
        t.k = ExpressionKind::IndexedUpvalue;
    } else {
        // register index of the table
        t.ind_t = if t.k == ExpressionKind::LocalRegister {
            t.var_ridx as usize
        } else {
            t.info as usize
        };
        if is_k_str(lex, state, k) {
            t.ind_idx = k.info; // literal string
            t.k = ExpressionKind::IndexedString;
        } else if is_c_int(k) {
            t.ind_idx = k.ival as i32; // int constant in proper range
            t.k = ExpressionKind::IndexedInteger;
        } else {
            t.ind_idx = exp2anyreg(lex, state, k)? as i32; // register
            t.k = ExpressionKind::Indexed;
        }
    }
    Ok(())
}

/// Check whether expression 'e' is a literal integer in
/// proper range to fit in register C
fn is_c_int(e: &mut ExpressionDesc) -> bool {
    is_k_int(e) && e.ival as usize <= MAXARG_C
}

/// Check whether expression 'e' is a literal integer.
fn is_k_int(e: &mut ExpressionDesc) -> bool {
    e.k == ExpressionKind::IntegerConstant && !e.has_jumps()
}

/// Check whether expression 'e' is a small literal string
fn is_k_str<T>(lex: &mut LexState<T>, state: &mut LuaState, e: &mut ExpressionDesc) -> bool {
    if e.k == ExpressionKind::Constant && !e.has_jumps() && e.info as usize <= MAXARG_B {
        let protoid = lex.borrow_fs(None).f;
        let is_string = state.protos[protoid].k[e.info as usize].is_string();
        is_string
    } else {
        false
    }
}

/// Convert a StringConstant to a Constant
fn str2k<T>(lex: &mut LexState<T>, state: &mut LuaState, e: &mut ExpressionDesc) {
    debug_assert!(e.k == ExpressionKind::StringConstant);
    e.info = string_constant(lex, state, &e.strval) as i32;
    e.k = ExpressionKind::Constant;
}

/// Ensures final expression result is in a valid R/K index
/// (that is, it is either in a register or in 'k' with an index
/// in the range of R/K indices).
/// Returns 1 if expression is K.
pub(crate) fn exp2rk<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    e: &mut ExpressionDesc,
) -> Result<bool, LuaError> {
    if exp2k(lex, state, e) {
        Ok(true)
    } else {
        // not a constant in the right range: put it in a register
        exp2anyreg(lex, state, e)?;
        Ok(false)
    }
}

/// Try to make 'e' a K expression with an index in the range of R/K
/// indices. Return true if succeeded.
fn exp2k<T>(lex: &mut LexState<T>, state: &mut LuaState, e: &mut ExpressionDesc) -> bool {
    if !e.has_jumps() {
        let info = match e.k {
            ExpressionKind::True => bool_constant(lex, state, true),
            ExpressionKind::False => bool_constant(lex, state, false),
            ExpressionKind::Nil => nil_constant(lex, state),
            ExpressionKind::IntegerConstant => integer_constant(lex, state, e.ival),
            ExpressionKind::FloatConstant => float_constant(lex, state, e.nval),
            ExpressionKind::StringConstant => string_constant(lex, state, &e.strval),
            ExpressionKind::Constant => e.info as usize,
            _ => {
                return false; // not a constant
            }
        };
        if info <= MAX_INDEX_RK {
            // does constant fit in 'argC'?
            e.k = ExpressionKind::Constant;
            e.info = info as i32;
            return true;
        }
    }
    // else, expression doesn't fit; leave it unchanged
    false
}

/// Add a boolean to list of constants and return its index.
fn bool_constant<T>(lex: &mut LexState<T>, state: &mut LuaState, val: bool) -> usize {
    let o = TValue::Boolean(val);
    lex.borrow_mut_fs(None).add_constant(state, o.clone(), o)
}

/// Add a float to list of constants and return its index.
pub(crate) fn float_constant<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    val: LuaFloat,
) -> usize {
    lex.borrow_mut_fs(None).float_constant(state, val)
}

/// Add an integer to list of constants and return its index.
pub(crate) fn integer_constant<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    val: LuaInteger,
) -> usize {
    lex.borrow_mut_fs(None).integer_constant(state, val)
}

/// Add a string to list of constants and return its index.
pub(crate) fn string_constant<T>(lex: &mut LexState<T>, state: &mut LuaState, val: &str) -> usize {
    lex.borrow_mut_fs(None).string_constant(state, val)
}

/// Add nil to list of constants and return its index.
fn nil_constant<T>(lex: &mut LexState<T>, state: &mut LuaState) -> usize {
    // cannot use nil as key; instead use table itself to represent nil
    lex.borrow_mut_fs(None)
        .add_constant(state, TValue::Nil, TValue::Nil)
}

/// Ensures final expression result is either in a register
/// or it is a constant.
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

/// Ensures final expression result is in some (any) register
/// and return that register.
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
        if ex.info as usize >= lex.get_nvar_stack() {
            // reg. is not a local?
            exp2reg(lex, state, ex, ex.info as u32)?; // put value on it
            return Ok(ex.info as u32);
        }
        // else expression has jumps and cannot change its register
        // to hold the jump values, because it is a local variable.
        // Go through to the default case.
    }
    exp2nextreg(lex, state, ex)?; // default: use next available register
    Ok(ex.info as u32)
}

/// Returns the position of the instruction "controlling" a given
/// jump (that is, its condition), or the jump itself if it is
/// unconditional.
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

/// Gets the destination address of a jump instruction. Used to traverse
/// a list of jumps.
fn get_jump<T>(lex: &mut LexState<T>, state: &LuaState, pc: i32) -> i32 {
    let i = lex.get_code(state, pc as usize);
    let offset = get_arg_sj(i);
    if offset == NO_JUMP {
        // point to itself represents end of list
        NO_JUMP // end of list
    } else {
        pc + 1 + offset // turn offset into absolute position
    }
}

/// Ensures final expression result is in next available register.
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

/// Ensures final expression result is either in a register
/// or in an upvalue.
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

/// Ensures final expression result (which includes results from its
/// jump lists) is in register 'reg'.
/// If expression has jumps, need to patch these jumps either to
/// its final position or to "load" instructions (for those tests
/// that do not produce values).
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
                jump(lex, state)? as i32
            };
            p_f = code_load_bool(lex, state, reg, OpCode::LoadFalseSkip)? as i32;
            p_t = code_load_bool(lex, state, reg, OpCode::LoadTrue)? as i32;
            // jump around these booleans if 'e' is not a test
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

fn code_load_bool<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    a: u32,
    op: OpCode,
) -> Result<u32, LuaError> {
    get_label(lex, state); // those instructions may be jump targets
    code_abc(lex, state, op as u32, a as i32, 0, 0)
}

/// Create a jump instruction and return its position, so its destination
/// can be fixed later (with 'fixjump').
pub(crate) fn jump<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<u32, LuaError> {
    code_sj(lex, state, OpCode::Jmp as u32, NO_JUMP, 0)
}

pub(crate) fn set_mult_ret<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    set_returns(lex, state, exp, LUA_MULTRET)
}

/// Fix an expression to return the number of results 'nresults'.
/// 'e' must be a multi-ret expression (function call or vararg).
pub(crate) fn set_returns<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
    nresults: i32,
) -> Result<(), LuaError> {
    let pc = lex.borrow_mut_code(state, exp.info as usize);
    if exp.k == ExpressionKind::Call {
        // expression is an open function call?
        set_arg_c(pc, (nresults + 1) as u32);
    } else {
        debug_assert!(exp.k == ExpressionKind::VarArg);
        set_arg_b(pc, (nresults + 1) as u32);
        let freereg = lex.borrow_fs(None).freereg as u32;
        set_arg_a(pc, freereg);
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
    let hr = get_label(lex, state); // mark "here" as a jump target
    patch_list(lex, state, list, hr)
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

/// Concatenate jump-list 'l2' into jump-list 'l1'
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
            // find last element
            list = next;
            next = get_jump(lex, state, list)
        }
        fix_jump(lex, state, list, l2)?; // last element links to 'l2'
    }
    Ok(())
}

/// Ensure expression value is in register 'reg', making 'exp' a
/// non-relocatable expression.
/// (Expression still may have jump lists.)
fn discharge2reg<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
    reg: u32,
) -> Result<(), LuaError> {
    discharge_vars(lex, state, exp)?;
    match exp.k {
        ExpressionKind::Nil => nil(lex, state, reg, 1)?,
        ExpressionKind::False => {
            code_abc(lex, state, OpCode::LoadFalse as u32, reg as i32, 0, 0)?;
        }
        ExpressionKind::True => {
            code_abc(lex, state, OpCode::LoadTrue as u32, reg as i32, 0, 0)?;
        }
        ExpressionKind::StringConstant => {
            str2k(lex, state, exp);
            code_k(lex, state, reg as i32, exp.info as u32)?;
        }
        ExpressionKind::Constant => {
            code_k(lex, state, reg as i32, exp.info as u32)?;
        }
        ExpressionKind::FloatConstant => {
            if exp.nval.fract() == 0.0
                && exp.nval as i64 >= -OFFSET_SBX as i64
                && exp.nval as i64 <= MAXARG_BX as i64 - OFFSET_SBX as i64
            {
                code_asbx(
                    lex,
                    state,
                    OpCode::LoadF as u32,
                    reg as i32,
                    exp.nval as i32,
                )?;
            } else {
                let k = float_constant(lex, state, exp.nval) as u32;
                code_k(lex, state, reg as i32, k)?;
            }
        }
        ExpressionKind::IntegerConstant => {
            if exp.ival >= -OFFSET_SBX as i64 && exp.ival <= MAXARG_BX as i64 - OFFSET_SBX as i64 {
                code_asbx(
                    lex,
                    state,
                    OpCode::LoadI as u32,
                    reg as i32,
                    exp.ival as i32,
                )?;
            } else {
                let k = integer_constant(lex, state, exp.ival) as u32;
                code_k(lex, state, reg as i32, k)?;
            }
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
            debug_assert!(exp.k == ExpressionKind::Jump);
            return Ok(()); //nothing to do...
        }
    }
    exp.info = reg as i32;
    exp.k = ExpressionKind::NonRelocable;
    Ok(())
}

/// Emit a SETLIST instruction.
/// 'base' is register that keeps table;
/// 'nelems' is #table plus those to be stored now;
/// 'tostore' is number of values (in registers 'base + 1',...) to add to
/// table (or LUA_MULTRET to add up to stack top).
pub(crate) fn set_list<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    base: i32,
    nelems: i32,
    to_store: i32,
) -> Result<(), LuaError> {
    debug_assert!(to_store != 0 && to_store <= LFIELDS_PER_FLUSH as i32);
    let to_store = if to_store == LUA_MULTRET { 0 } else { to_store };
    if nelems <= MAXARG_C as i32 {
        code_abc(lex, state, OpCode::SetList as u32, base, to_store, nelems)?;
    } else {
        let extra = nelems / (MAXARG_C as i32 + 1);
        let nelems = nelems % MAXARG_C as i32 + 1;
        code_abck(
            lex,
            state,
            OpCode::SetList as u32,
            base,
            to_store,
            nelems,
            1,
        )?;
        code_extra_arg(lex, state, extra as u32)?;
    }
    // free registers with list values
    lex.borrow_mut_fs(None).freereg = (base + 1) as usize;
    Ok(())
}

/// Create a OP_LOADNIL instruction, but try to optimize: if the previous
/// instruction is also OP_LOADNIL and ranges are compatible, adjust
/// range of previous instruction instead of emitting a new one. (For
/// instance, 'local a; local b' will generate a single opcode.)
pub(crate) fn nil<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    from: u32,
    n: i32,
) -> Result<(), LuaError> {
    let l = (from as i32 + n - 1) as u32; // last register to set nil
    if let Some(previous) = lex.try_borrow_mut_previous_code(state) {
        if get_opcode(*previous) == OpCode::LoadNil {
            let pfrom = get_arg_a(*previous);
            let pl = pfrom + get_arg_b(*previous);
            if (pfrom <= from && from <= pl + 1) || (from <= pfrom && pfrom <= l + 1) {
                let from = from.min(pfrom);
                let l = l.max(pl);
                set_arg_a(previous, from);
                set_arg_b(previous, l - from);
                return Ok(());
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

/// Code a 'return' instruction
pub(crate) fn ret<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    first: u32,
    nret: u32,
) -> Result<(), LuaError> {
    let op = match nret {
        0 => OpCode::Return0,
        1 => OpCode::Return1,
        _ => OpCode::Return,
    };
    code_abc(lex, state, op as u32, first as i32, nret as i32 + 1, 0)?;
    Ok(())
}

/// Reserve 'n' registers in register stack
pub(crate) fn reserve_regs<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    n: usize,
) -> Result<(), LuaError> {
    check_stack(lex, state, n)?;
    lex.borrow_mut_fs(None).freereg += n;
    Ok(())
}

/// Check register-stack level, keeping track of its maximum size
/// in field 'maxstacksize'
pub(crate) fn check_stack<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    n: usize,
) -> Result<(), LuaError> {
    let proto = lex.borrow_mut_proto(state, None);
    let new_stack = lex.borrow_fs(None).freereg + n;
    if new_stack > proto.maxstacksize {
        if new_stack >= MAX_REGS {
            return lex.syntax_error(state, "function or expression needs too many registers");
        }
        proto.maxstacksize = new_stack;
    }
    Ok(())
}

/// Free register used by expression 'e' (if any)
fn free_exp<T>(lex: &mut LexState<T>, e: &mut ExpressionDesc) {
    if let ExpressionKind::NonRelocable = e.k {
        free_reg(lex, e.info as u32);
    }
}

/// Free register 'reg', if it is neither a constant index nor
/// a local variable.
fn free_reg<T>(lex: &mut LexState<T>, reg: u32) {
    if reg >= lex.get_nvar_stack() as u32 {
        lex.borrow_mut_fs(None).freereg -= 1;
        debug_assert!(reg == lex.borrow_fs(None).freereg as u32);
    }
}

/// Change line information associated with current position, by removing
/// previous info and adding it again with new line.
pub(crate) fn fix_line<T>(lex: &mut LexState<T>, state: &mut LuaState, line: usize) {
    save_line_info(lex, state, line);
}

/// Save line info for a new instruction.
fn save_line_info<T>(lex: &mut LexState<T>, state: &mut LuaState, line: usize) {
    let pc = lex.next_pc(state) - 1;
    lex.borrow_mut_proto(state, None).lineinfo[pc] = line;
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
    discharge_vars(lex, state, e)?;
    match op {
        UnaryOp::Minus => {
            // use e2 as fake 2nd operand
            if !const_folding(OpCode::UnaryMinus, e, &mut e2) {
                code_unexpval(lex, state, OpCode::UnaryMinus, e, line)?
            }
        }
        UnaryOp::BinaryNot => {
            if !const_folding(OpCode::BinaryNot, e, &mut e2) {
                code_unexpval(lex, state, OpCode::BinaryNot, e, line)?
            }
        }
        UnaryOp::Len => code_unexpval(lex, state, OpCode::Len, e, line)?,
        UnaryOp::Not => {
            code_not(lex, state, e)?;
        }
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
        | ExpressionKind::StringConstant
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
    // interchange true and false lists
    (exp.t, exp.f) = (exp.f, exp.t);
    remove_values(lex, state, exp.f);
    remove_values(lex, state, exp.t);
    Ok(())
}

/// Traverse a list of tests ensuring no one produces a value
fn remove_values<T>(lex: &mut LexState<T>, state: &mut LuaState, list: i32) {
    let mut list = list;
    while list != NO_JUMP {
        patch_test_reg(lex, state, list, NO_REG);
        list = get_jump(lex, state, list);
    }
}

/// Process 1st operand 'v' of binary operation 'op' before reading
/// 2nd operand.
pub(crate) fn infix<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: BinaryOp,
    v: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    discharge_vars(lex, state, v)?;
    match op {
        BinaryOp::And => go_if_true(lex, state, v), // go ahead only if 'v' is true
        BinaryOp::Or => go_if_false(lex, state, v), // go ahead only if 'v' is false
        BinaryOp::Concat => exp2nextreg(lex, state, v), // operand must be on the 'stack'
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
            if !v.is_numeral() {
                exp2anyreg(lex, state, v)?;
            }
            // else keep numeral, which may be folded or used as an immediate
            // operand
            Ok(())
        }
        BinaryOp::Eq | BinaryOp::Ne => {
            if !v.is_numeral() {
                exp2rk(lex, state, v)?;
            }
            //else keep numeral, which may be an immediate operand
            Ok(())
        }
        BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
            let mut pi = 0;
            let mut is_float = false;
            if !is_sc_number(v, &mut pi, &mut is_float) {
                exp2anyreg(lex, state, v)?;
            }
            // else keep numeral, which may be an immediate operand
            Ok(())
        }
    }
}

/// Check whether expression 'e' is a literal integer or float in
/// proper range to fit in a register (sB or sC).
fn is_sc_number(e: &mut ExpressionDesc, pi: &mut i32, is_float: &mut bool) -> bool {
    let i;
    if e.k == ExpressionKind::IntegerConstant {
        i = e.ival;
    } else if e.k == ExpressionKind::FloatConstant && e.nval.fract() == 0.0 {
        i = e.nval as LuaInteger;
        *is_float = true;
    } else {
        return false; // not a number
    }
    if !e.has_jumps() && i >= 0 && i as usize <= MAXARG_C {
        *pi = i as i32 + OFFSET_SC;
        true
    } else {
        false
    }
}

/// Emit code to go through if 'e' is false, jump otherwise.
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
        ExpressionKind::Jump => exp.info, // already jump if true
        _ => jump_on_cond(lex, state, exp, 1)?, // jump if true
    };
    concat_jump(lex, state, &mut exp.t, pc)?; // insert new jump in 't' list
    patch_to_here(lex, state, exp.f)?; // false list jumps to here (to go through)
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
        | ExpressionKind::StringConstant
        | ExpressionKind::IntegerConstant
        | ExpressionKind::True => {
            // always true; do nothing
            NO_JUMP
        }
        ExpressionKind::Jump => {
            negate_condition(lex, state, exp); // jump when it is false
            exp.info // save jump position
        }
        _ => jump_on_cond(lex, state, exp, 0)?, // jump when false
    };
    concat_jump(lex, state, &mut exp.f, pc)?; // insert new jump in false list
    patch_to_here(lex, state, exp.t)?; // true list jumps to here (to go through)
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
                remove_last_instruction(lex, state);
                return cond_jump(
                    lex,
                    state,
                    OpCode::Test,
                    get_arg_b(ie) as i32,
                    0,
                    0,
                    1 - cond as u32,
                );
            }
            // else go through
        }
    }
    discharge2any_reg(lex, state, exp)?;
    free_exp(lex, exp);
    cond_jump(
        lex,
        state,
        OpCode::TestSet,
        NO_REG as i32,
        exp.info,
        0,
        cond as u32,
    )
}

/// Remove the last instruction created, correcting line information
/// accordingly.
fn remove_last_instruction<T>(lex: &mut LexState<T>, state: &mut LuaState) {
    let proto = lex.borrow_mut_proto(state, None);
    proto.lineinfo.pop();
    proto.code.pop();
}

/// Code a "conditional jump", that is, a test or comparison opcode
/// followed by a jump. Return jump position.
fn cond_jump<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: OpCode,
    a: i32,
    b: i32,
    c: i32,
    k: u32,
) -> Result<i32, LuaError> {
    code_abck(lex, state, op as u32, a, b, c, k)?;
    jump(lex, state).map(|v| v as i32)
}

/// Ensure expression value is in a register, making 'e' a
/// non-relocatable expression.
/// (Expression still may have jump lists.)
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

/// Emit SELF instruction (convert expression 'e' into 'e:key(e,').
pub(crate) fn op_self<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    e: &mut ExpressionDesc,
    key: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    exp2anyreg(lex, state, e)?;
    let ereg = e.info; // register where 'e' was placed
    free_exp(lex, e);
    let func = lex.borrow_fs(None).freereg as i32; // base register for op_self
    e.info = func;
    e.k = ExpressionKind::NonRelocable; // self expression has a fixed register
    reserve_regs(lex, state, 2)?; // function and 'self' produced by op_self
    code_abrk(lex, state, OpCode::OpSelf as u32, func, ereg, key)?;
    free_exp(lex, key);
    Ok(())
}

/// Negate condition 'e' (where 'e' is a comparison).
fn negate_condition<T>(lex: &mut LexState<T>, state: &mut LuaState, exp: &mut ExpressionDesc) {
    let pcref = get_jump_control(lex, state, exp.info);
    debug_assert!({
        let i = get_opcode(*pcref);
        i.is_test() && i != OpCode::Test && i != OpCode::TestSet
    });
    set_arg_k(pcref, get_arg_k(*pcref) ^ 1);
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
    discharge_vars(lex, state, exp2)?;
    if let Ok(op) = op.try_into() {
        if const_folding(op, exp1, exp2) {
            return Ok(()); // done by folding
        }
    }
    match op {
        BinaryOp::And => {
            debug_assert!(exp1.t == NO_JUMP); // list closed by 'in_fix'
            concat_jump(lex, state, &mut exp2.f, exp1.f)?;
            *exp1 = (*exp2).clone();
        }
        BinaryOp::Or => {
            debug_assert!(exp1.f == NO_JUMP); // list closed by 'in_fix'
            concat_jump(lex, state, &mut exp2.t, exp1.t)?;
            *exp1 = (*exp2).clone();
        }
        BinaryOp::Concat => {
            exp2nextreg(lex, state, exp2)?;
            code_concat(lex, state, exp1, exp2, line)?;
        }
        BinaryOp::Add | BinaryOp::Mul => code_commutative(lex, state, op, exp1, exp2, line)?,
        BinaryOp::Sub => {
            if !finish_bin_exp_neg(
                lex,
                state,
                exp1,
                exp2,
                OpCode::AddI,
                line,
                TagMethod::Sub as i32,
            )? {
                code_arith(lex, state, OpCode::Sub, exp1, exp2, false, line)?
            }
        }
        BinaryOp::Div | BinaryOp::IntDiv | BinaryOp::Mod | BinaryOp::Pow => {
            code_arith(lex, state, op.try_into().unwrap(), exp1, exp2, false, line)?
        }
        BinaryOp::BinaryAnd | BinaryOp::BinaryOr | BinaryOp::BinaryXor => {
            code_bitwise(lex, state, op.try_into().unwrap(), exp1, exp2, line)?
        }
        BinaryOp::Shl => {
            if is_sc_int(exp1) {
                std::mem::swap(exp1, exp2);
                code_bini(
                    lex,
                    state,
                    OpCode::ShlI,
                    exp1,
                    exp2,
                    true,
                    line,
                    TagMethod::Shl as i32,
                )?;
            // I << r2
            } else if !finish_bin_exp_neg(
                lex,
                state,
                exp1,
                exp2,
                OpCode::ShrI,
                line,
                TagMethod::Shl as i32,
            )? {
                // coded as (r1 >> -I)
                // regular case (two registers)
                code_bin_expval(lex, state, OpCode::Shl, exp1, exp2, line)?;
            }
        }
        BinaryOp::Shr => {
            if is_sc_int(exp2) {
                code_bini(
                    lex,
                    state,
                    OpCode::ShrI,
                    exp1,
                    exp2,
                    false,
                    line,
                    TagMethod::Shr as i32,
                )?;
            // r1 >> I
            } else {
                // regular case (two registers)
                code_bin_expval(lex, state, OpCode::Shr, exp1, exp2, line)?;
            }
        }
        BinaryOp::Eq | BinaryOp::Ne => code_eq(lex, state, op, exp1, exp2)?,
        BinaryOp::Gt | BinaryOp::Ge => {
            // '(a > b)' => '(b < a)';  '(a >= b)' => '(b <= a)'
            std::mem::swap(exp1, exp2);
            let op = match op {
                BinaryOp::Gt => BinaryOp::Lt,
                BinaryOp::Ge => BinaryOp::Le,
                _ => unreachable!(),
            };
            code_order(lex, state, op, exp1, exp2)?;
        }
        BinaryOp::Lt | BinaryOp::Le => {
            code_order(lex, state, op, exp1, exp2)?;
        }
    }
    Ok(())
}

/// Emit code for order comparisons. When using an immediate operand,
/// 'isfloat' tells whether the original value was a float.
fn code_order<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: BinaryOp,
    exp1: &mut ExpressionDesc,
    exp2: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    let mut im: i32 = 0;
    let mut is_float: bool = false;
    let (r1, r2, op) = if is_sc_number(exp2, &mut im, &mut is_float) {
        // use immediate operand
        (
            exp2anyreg(lex, state, exp1)? as i32,
            im,
            match op {
                BinaryOp::Le => OpCode::LeI,
                BinaryOp::Lt => OpCode::LtI,
                _ => unreachable!(),
            },
        )
    } else if is_sc_number(exp1, &mut im, &mut is_float) {
        // transform (A < B) to (B > A) and (A <= B) to (B >= A)
        (
            exp2anyreg(lex, state, exp2)? as i32,
            im,
            match op {
                BinaryOp::Le => OpCode::GeI,
                BinaryOp::Lt => OpCode::GtI,
                _ => unreachable!(),
            },
        )
    } else {
        // regular case, compare two registers
        (
            exp2anyreg(lex, state, exp1)? as i32,
            exp2anyreg(lex, state, exp2)? as i32,
            match op {
                BinaryOp::Le => OpCode::Le,
                BinaryOp::Lt => OpCode::Lt,
                _ => unreachable!(),
            },
        )
    };
    free_exps(lex, exp1, exp2);
    exp1.init(
        ExpressionKind::Jump,
        cond_jump(lex, state, op, r1, r2, if is_float { 1 } else { 0 }, 1)?,
    );
    Ok(())
}

/// Emit code for equality comparisons ('==', '~=').
/// 'exp1' was already put as RK by 'in_fix'.
fn code_eq<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    opr: BinaryOp,
    exp1: &mut ExpressionDesc,
    exp2: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    if exp1.k != ExpressionKind::NonRelocable {
        debug_assert!(
            exp1.k == ExpressionKind::Constant
                || exp1.k == ExpressionKind::IntegerConstant
                || exp1.k == ExpressionKind::FloatConstant
        );
        std::mem::swap(exp1, exp2);
    }
    let r1 = exp2anyreg(lex, state, exp1)?; // 1st expression must be in register
    let mut im: i32 = 0;
    let mut is_float: bool = false;
    let (op, r2) = if is_sc_number(exp2, &mut im, &mut is_float) {
        (OpCode::EqI, im) // immediate operand
    } else if exp2rk(lex, state, exp2)? {
        // 2nd expression is constant?
        (OpCode::EqK, exp2.info) // constant index
    } else {
        (OpCode::Eq, exp2anyreg(lex, state, exp2)? as i32) // will compare two registers
    };
    free_exps(lex, exp1, exp2);
    exp1.init(
        ExpressionKind::Jump,
        cond_jump(
            lex,
            state,
            op,
            r1 as i32,
            r2,
            if is_float { 1 } else { 0 },
            if opr == BinaryOp::Eq { 1 } else { 0 },
        )?,
    );
    Ok(())
}

/// Code bitwise operations; they are all commutative, so the function
/// tries to put an integer constant as the 2nd operand (a K operand).
fn code_bitwise<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: OpCode,
    exp1: &mut ExpressionDesc,
    exp2: &mut ExpressionDesc,
    line: usize,
) -> Result<(), LuaError> {
    let mut flip = false;
    if exp1.k == ExpressionKind::IntegerConstant {
        std::mem::swap(exp1, exp2); // 'e2' will be the constant operand
        flip = true;
    }
    if exp2.k == ExpressionKind::IntegerConstant && exp2k(lex, state, exp2) {
        // K operand?
        code_bin_k(lex, state, op, exp1, exp2, flip, line)?;
    } else {
        code_bin_nok(lex, state, op, exp1, exp2, flip, line)?;
    }
    Ok(())
}

/// Code binary operators with no constant operand.
fn code_bin_nok<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: OpCode,
    exp1: &mut ExpressionDesc,
    exp2: &mut ExpressionDesc,
    flip: bool,
    line: usize,
) -> Result<(), LuaError> {
    if flip {
        std::mem::swap(exp1, exp2); // back to original order
    }
    code_bin_expval(lex, state, op, exp1, exp2, line) // use standard operators
}

/// Code binary operators with K operand.
fn code_bin_k<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: OpCode,
    exp1: &mut ExpressionDesc,
    exp2: &mut ExpressionDesc,
    flip: bool,
    line: usize,
) -> Result<(), LuaError> {
    let event = TagMethod::try_from(op).unwrap() as i32;
    let op = op.to_k();
    let v2 = exp2.info; // K index
    finish_bin_expval(
        lex,
        state,
        exp1,
        exp2,
        op,
        v2,
        flip,
        line,
        OpCode::MMBinK,
        event,
    )
}

/// Try to code a binary operator negating its second operand.
/// For the metamethod, 2nd operand must keep its original value.
fn finish_bin_exp_neg<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp1: &mut ExpressionDesc,
    exp2: &mut ExpressionDesc,
    op: OpCode,
    line: usize,
    event: i32,
) -> Result<bool, LuaError> {
    if !exp2.is_k_int() {
        return Ok(false); // not an integer constant
    }
    let i2 = exp2.ival;
    if !(fit_sc(i2) && fit_sc(-i2)) {
        return Ok(false); // not in the proper range
    }
    // operating a small integer constant
    let v2 = i2 as i32;
    finish_bin_expval(
        lex,
        state,
        exp1,
        exp2,
        op,
        OFFSET_SC - v2,
        false,
        line,
        OpCode::MMBinI,
        event,
    )?;
    // correct metamethod argument
    let pc = lex.try_borrow_mut_previous_code(state).unwrap();
    set_arg_b(pc, (OFFSET_SC + v2) as u32);
    Ok(true)
}

/// Code commutative operators ('+', '*'). If first operand is a
/// numeric constant, change order of operands to try to use an
/// immediate or K operator.
fn code_commutative<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: BinaryOp,
    exp1: &mut ExpressionDesc,
    exp2: &mut ExpressionDesc,
    line: usize,
) -> Result<(), LuaError> {
    let mut flip = false;
    if exp1.to_numeral(None) {
        // is first operand a numeric constant?
        std::mem::swap(exp1, exp2); // change order
        flip = true;
    }
    if op == BinaryOp::Add && is_sc_int(exp2) {
        // immediate operand?
        code_bini(
            lex,
            state,
            OpCode::AddI,
            exp1,
            exp2,
            flip,
            line,
            TagMethod::Add as i32,
        )?;
    } else {
        code_arith(lex, state, op.try_into().unwrap(), exp1, exp2, flip, line)?;
    }
    Ok(())
}

/// Code binary operators with immediate operands.
#[allow(clippy::too_many_arguments)]
fn code_bini<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: OpCode,
    exp1: &mut ExpressionDesc,
    exp2: &mut ExpressionDesc,
    flip: bool,
    line: usize,
    event: i32,
) -> Result<(), LuaError> {
    let v2 = exp2.ival as i32 + OFFSET_SC;
    debug_assert!(exp2.k == ExpressionKind::IntegerConstant);
    finish_bin_expval(
        lex,
        state,
        exp1,
        exp2,
        op,
        v2,
        flip,
        line,
        OpCode::MMBinI,
        event,
    )
}

/// Emit code for binary expressions that "produce values"
/// (everything but logical operators 'and'/'or' and comparison
/// operators).
/// Expression to produce final result will be encoded in 'exp1'.
#[allow(clippy::too_many_arguments)]
fn finish_bin_expval<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp1: &mut ExpressionDesc,
    exp2: &mut ExpressionDesc,
    op: OpCode,
    v2: i32,
    flip: bool,
    line: usize,
    mmop: OpCode,
    event: i32,
) -> Result<(), LuaError> {
    let v1 = exp2anyreg(lex, state, exp1)?;
    let pc = code_abck(lex, state, op as u32, 0, v1 as i32, v2, 0)?;
    free_exps(lex, exp1, exp2);
    exp1.info = pc as i32;
    exp1.k = ExpressionKind::Relocable;
    fix_line(lex, state, line);
    code_abck(
        lex,
        state,
        mmop as u32,
        v1 as i32,
        v2,
        event,
        if flip { 1 } else { 0 },
    )?;
    fix_line(lex, state, line);
    Ok(())
}

/// Check whether expression 'e' is a literal integer in
/// proper range to fit in register sC
fn is_sc_int(e: &mut ExpressionDesc) -> bool {
    e.is_k_int() && fit_sc(e.ival)
}

/// Check whether 'i' can be stored in an 'sC' operand.
fn fit_sc(ival: LuaInteger) -> bool {
    ival >= 0 && ival <= MAXARG_C as LuaInteger - OFFSET_SC as LuaInteger
}

/// Check whether 'i' can be stored in an 'sBx' operand.
fn fit_sbx(ival: LuaInteger) -> bool {
    ival >= OFFSET_SBX as i64 && ival <= MAXARG_BX as i64 - OFFSET_SBX as i64
}

/// Create code for '(e1 .. e2)'.
/// For '(e1 .. e2.1 .. e2.2)' (which is '(e1 .. (e2.1 .. e2.2))',
/// because concatenation is right associative), merge both CONCATs.
fn code_concat<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp1: &mut ExpressionDesc,
    exp2: &mut ExpressionDesc,
    line: usize,
) -> Result<(), LuaError> {
    let ie2 = lex.get_previous_code(state);
    if get_opcode(ie2) == OpCode::Concat {
        // is 'e2' a concatenation?
        let n = get_arg_b(ie2); // # of elements concatenated in 'e2'
        debug_assert!(exp1.info + 1 == get_arg_a(ie2) as i32);
        free_exp(lex, exp2);
        let ie2 = lex.try_borrow_mut_previous_code(state).unwrap();
        set_arg_a(ie2, exp1.info as u32); // correct first element ('e1')
        set_arg_b(ie2, n + 1); // will concatenate one more element
    } else {
        // 'e2' is not a concatenation
        code_abc(lex, state, OpCode::Concat as u32, exp1.info, 2, 0)?; // new concat opcode
        free_exp(lex, exp2);
        fix_line(lex, state, line);
    }
    Ok(())
}

/// Code arithmetic operators ('+', '-', ...). If second operand is a
/// constant in the proper range, use variant opcodes with K operands.
fn code_arith<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: OpCode,
    exp1: &mut ExpressionDesc,
    exp2: &mut ExpressionDesc,
    flip: bool,
    line: usize,
) -> Result<(), LuaError> {
    if exp2.to_numeral(None) && exp2k(lex, state, exp2) {
        // K operand ?
        code_bin_k(lex, state, op, exp1, exp2, flip, line)
    } else {
        // 'e2' is neither an immediate nor a K operand
        code_bin_nok(lex, state, op, exp1, exp2, flip, line)
    }
}

/// Emit code for binary expressions that "produce values" over
/// two registers.
fn code_bin_expval<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    op: OpCode,
    exp1: &mut ExpressionDesc,
    exp2: &mut ExpressionDesc,
    line: usize,
) -> Result<(), LuaError> {
    let v2 = exp2anyreg(lex, state, exp2)?; // make sure 'e2' is in a register
                                            // 'e1' must be already in a register or it is a constant
    debug_assert!(op as u32 >= OpCode::Add as u32 && op as u32 <= OpCode::Shr as u32);
    finish_bin_expval(
        lex,
        state,
        exp1,
        exp2,
        op,
        v2 as i32,
        false,
        line,
        OpCode::MMBin,
        TagMethod::try_from(op).unwrap() as i32,
    )?;
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
    if !exp1.to_numeral(Some(&mut v1))
        || !exp2.to_numeral(Some(&mut v2))
        || !is_op_valid(op, &v1, &v2)
    {
        return false; // non-numeric operands or not safe to fold
    }
    let mut res: TValue = TValue::Nil;
    raw_arith(op, &v1, &v2, &mut res);
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

pub(crate) fn arith(op: OpCode, v1: &TValue, v2: &TValue, res: &mut TValue) {
    if !raw_arith(op, v1, v2, res) {
        // TODO metamethods
        todo!()
    }
}

pub(crate) fn raw_arith(op: OpCode, v1: &TValue, v2: &TValue, res: &mut TValue) -> bool {
    match op {
        OpCode::BinaryAnd
        | OpCode::BinaryOr
        | OpCode::BinaryXor
        | OpCode::Shl
        | OpCode::Shr
        | OpCode::BinaryNot => {
            // operate only on integer
            if let (Some(i1), Some(i2)) = (v1.into_integer(), v2.into_integer()) {
                *res = TValue::Integer(int_arith(op, i1, i2));
                true
            } else {
                false
            }
        }
        OpCode::Div | OpCode::Pow => {
            // operate only on floats
            if let (Some(f1), Some(f2)) = (v1.into_float(), v2.into_float()) {
                *res = TValue::Float(num_arith(op, f1, f2));
                true
            } else {
                false
            }
        }
        _ => {
            if v1.is_integer() && v2.is_integer() {
                *res = TValue::Integer(int_arith(
                    op,
                    v1.get_integer_value(),
                    v2.get_integer_value(),
                ));
                true
            } else if let (Some(f1), Some(f2)) = (v1.into_float(), v2.into_float()) {
                *res = TValue::Float(num_arith(op, f1, f2));
                true
            } else {
                false
            }
        }
    }
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
        _ => unreachable!(),
    }
}

fn num_arith(op: OpCode, i1: LuaFloat, i2: LuaFloat) -> LuaFloat {
    match op {
        OpCode::Add => i1 + i2,
        OpCode::Sub => i1 - i2,
        OpCode::Mul => i1 * i2,
        OpCode::Div => i1 / i2,
        OpCode::Pow => i1.powf(i2),
        OpCode::IntegerDiv => (i1 / i2).floor(),
        OpCode::UnaryMinus => -i1,
        OpCode::Mod => i1 % i2,
        _ => unreachable!(),
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
            v1.into_integer().is_some() && v2.into_integer().is_some()
            // conversion errors
        }
        OpCode::Div | OpCode::IntegerDiv | OpCode::Mod => match v1.into_integer() {
            Some(i) if i == 0 => false, // division by 0
            Some(_) => true,
            None => false,
        },
        _ => true, // everything else is valid
    }
}

/// Concatenate jump-list 'l2' into jump-list 'l1'
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

/// Do a final pass over the code of a function, doing small peephole
/// optimizations and adjustments.
pub(crate) fn finish<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let protoid = lex.borrow_fs(None).f;
    let is_vararg = state.protos[protoid].is_vararg;
    let numparams = state.protos[protoid].numparams;
    let code_len = state.protos[protoid].code.len();
    for i in 0..code_len {
        let pc = state.protos[protoid].code[i];
        let op = get_opcode(pc);
        match op {
            OpCode::Return0 | OpCode::Return1 => {
                if lex.borrow_fs(None).need_close || is_vararg {
                    let pc = &mut state.protos[protoid].code[i];
                    set_opcode(pc, OpCode::Return as u32);
                    if lex.borrow_fs(None).need_close {
                        set_arg_k(pc, 1);
                    }
                    if is_vararg {
                        set_arg_c(pc, numparams as u32 + 1);
                    }
                }
            }
            OpCode::Return | OpCode::TailCall => {
                let pc = &mut state.protos[protoid].code[i];
                if lex.borrow_fs(None).need_close {
                    set_arg_k(pc, 1);
                }
                if is_vararg {
                    set_arg_c(pc, numparams as u32 + 1);
                }
            }
            OpCode::Jmp => {
                let target = final_target(lex, state, i);
                fix_jump(lex, state, i as i32, target)?;
            }
            _ => (),
        }
    }
    Ok(())
}

/// return the final target of a jump (skipping jumps to jumps)
fn final_target<T>(lex: &mut LexState<T>, state: &LuaState, i: usize) -> i32 {
    let mut i = i as i32;
    for _ in 0..100 {
        // avoid infinite loops
        let pc = lex.get_code(state, i as usize);
        if get_opcode(pc) != OpCode::Jmp {
            break;
        }
        i += get_arg_sj(pc) + 1;
    }
    i
}

pub(crate) fn set_table_size<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    pc: usize,
    ra: i32,
) -> Result<(), LuaError> {
    let inst = lex.borrow_mut_code(state, pc);
    *inst = create_abck(OpCode::NewTable as u32, ra, 0, 0, 0);
    // TODO ? extraarg
    Ok(())
}

/// If expression is a constant, fills 'v' with its value
/// and returns true. Otherwise, returns false.
pub(crate) fn exp2const<T>(
    lex: &mut LexState<T>,
    e: &ExpressionDesc,
    v: &mut TValue,
) -> Result<bool, LuaError> {
    if e.has_jumps() {
        return Ok(false);
    }
    match e.k {
        ExpressionKind::False => {
            *v = TValue::Boolean(false);
            Ok(true)
        }
        ExpressionKind::True => {
            *v = TValue::Boolean(true);
            Ok(true)
        }
        ExpressionKind::Nil => {
            *v = TValue::Nil;
            Ok(true)
        }
        ExpressionKind::StringConstant => {
            *v = TValue::from(e.strval.to_owned());
            Ok(true)
        }
        ExpressionKind::Constant => {
            *v = const2val(lex, e);
            Ok(true)
        }
        _ => Ok(e.to_numeral(Some(v))),
    }
}

pub(crate) fn integer<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    reg: u32,
    i: LuaInteger,
) -> Result<(), LuaError> {
    if fit_sbx(i) {
        code_asbx(lex, state, OpCode::LoadI as u32, reg as i32, i as i32)?;
    } else {
        let k = integer_constant(lex, state, i) as u32;
        code_k(lex, state, reg as i32, k)?;
    }
    Ok(())
}
