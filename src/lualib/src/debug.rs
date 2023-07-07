//! Debug Interface

use crate::{
    object::{Closure, LClosure, StkId, TValue},
    opcodes::{get_arg_a, get_arg_b, get_arg_c, get_arg_sj, get_opcode, OpCode},
    state::LuaState,
    LuaError,
};

pub(crate) fn error_msg(_state: &mut LuaState) -> Result<(), LuaError> {
    todo!()
}

pub(crate) fn concat_error(state: &mut LuaState, p1: isize, p2: isize) -> Result<(), LuaError> {
    let perr = if state.index2adr(p1).is_string() || state.index2adr(p1).is_float() {
        p2
    } else {
        p1
    };
    type_error(state, perr as StkId, "concatenate")
}

pub(crate) fn for_error(state: &mut LuaState, ra: usize, what: &str) -> LuaError {
    state
        .run_error(&format!(
            "bad 'for' {} (number expected, got {})",
            what,
            state.stack[ra].get_type_name()
        ))
        .err()
        .unwrap()
}

/// Raise a type error with "standard" information about the faulty
/// object 'id' (using 'varinfo').
pub(crate) fn type_error(state: &mut LuaState, id: StkId, operation: &str) -> Result<(), LuaError> {
    let inf = var_info(state, id);
    internal_type_error(state, id, operation, &inf)
}

/// raise a type error
fn internal_type_error(
    state: &mut LuaState,
    id: StkId,
    operation: &str,
    extra: &str,
) -> Result<(), LuaError> {
    let typ = state.stack[id].get_type_name();
    state.run_error(&format!(
        "attempt to {} a {} value{}",
        operation, typ, extra
    ))
}

/// Build a string with a "description" for the value at position 'id', such as
/// "variable 'x'" or "upvalue 'y'".
fn var_info(state: &LuaState, id: usize) -> String {
    let ci = &state.base_ci[state.ci];
    let funcvalue = &state.stack[ci.func];
    match funcvalue {
        TValue::Function(cl) => match &*cl.borrow() {
            Closure::Lua(cl) => {
                if let Some((kind, name)) = get_upval_name(state, cl, id) {
                    return format!(" ({} '{}')", &kind, &name);
                } else {
                    if let Some(reg) = in_stack(state, id) {
                        if let Some((kind, name)) = get_obj_name(state, cl, ci.saved_pc, reg) {
                            return format!(" ({} '{}')", &kind, &name);
                        }
                    }
                }
            }
            _ => (),
        },
        _ => (),
    }
    "".to_owned()
}

fn get_obj_name(
    state: &LuaState,
    cl: &LClosure,
    last_pc: usize,
    reg: usize,
) -> Option<(String, String)> {
    if let Some(name) = get_local_name(state, cl, reg + 1, last_pc) {
        return Some(("local".to_owned(), name));
    }
    if let Some(pc) = find_set_reg(state, cl, last_pc, reg as u32) {
        let i = state.get_instruction(cl.proto, pc);
        let op = get_opcode(i);
        match op {
            OpCode::Move => {
                let b = get_arg_b(i);
                let a = get_arg_a(i);
                if b < a {
                    return get_obj_name(state, cl, pc, b as usize);
                }
            }
            OpCode::GetTabUp => {
                let _key_id = get_arg_c(i);
                todo!()
            }
            OpCode::GetTable => {
                todo!()
            }
            OpCode::GetI => {
                todo!()
            }
            OpCode::GetField => {
                todo!()
            }
            OpCode::GetUpVal => {
                todo!()
            }
            OpCode::LoadK | OpCode::LoadKx => {
                todo!()
            }
            OpCode::OpSelf => {
                todo!()
            }
            _ => (),
        }
    }
    None
}

/// Try to find last instruction before 'lastpc' that modified register 'reg'.
fn find_set_reg(state: &LuaState, cl: &LClosure, last_pc: usize, reg: u32) -> Option<usize> {
    let mut set_reg = None; // last instruction that changed 'reg'
    let mut jmp_target = 0; // any code before this address is conditional
    let i = state.get_instruction(cl.proto, last_pc);
    let last_pc = if get_opcode(i).is_mm() {
        last_pc - 1 // previous instruction was not actually executed
    } else {
        last_pc
    };
    for pc in 0..last_pc {
        let i = state.get_instruction(cl.proto, pc);
        let op = get_opcode(i);
        let a = get_arg_a(i);
        if match op {
            // check if current instruction changed 'reg'
            OpCode::LoadNil => {
                //  set registers from 'a' to 'a+b'
                let b = get_arg_b(i);
                a <= reg && reg <= a + b
            }
            OpCode::TForCall => {
                // affect all regs above its base
                reg >= a + 2
            }
            OpCode::Call | OpCode::TailCall => {
                // affect all registers above base
                reg >= a
            }
            OpCode::Jmp => {
                // doesn't change registers, but changes 'jmptarget'
                let b = get_arg_sj(i);
                let dest = (pc as isize + 1 + b as isize) as usize;
                // jump does not skip 'lastpc' and is larger than current one?
                if dest <= last_pc && dest > jmp_target {
                    jmp_target = dest; // update jmp_target
                }
                false
            }
            _ => {
                // any instruction that sets A
                reg == a && op.sets_a()
            }
        } && pc < jmp_target
        {
            set_reg = Some(pc)
        }
    }
    set_reg
}

/// Look for n-th local variable at line 'pc' in function 'cl'.
fn get_local_name(state: &LuaState, cl: &LClosure, n: usize, pc: usize) -> Option<String> {
    let mut n = n;
    let p = &state.protos[cl.proto];
    for var in p.locvars.iter() {
        if var.start_pc > pc {
            break;
        }
        if pc < var.end_pc {
            n -= 1;
            if n == 0 {
                return Some(var.name.to_owned());
            }
        }
    }
    None
}

/// Check whether stack position 'id' is in the stack frame of
/// the current function and, if so, returns its index.
fn in_stack(state: &LuaState, id: usize) -> Option<usize> {
    let func = state.base_ci[state.ci].func;
    let base = func + 1;
    if id >= base {
        Some(id)
    } else {
        None
    }
}

/// Checks whether value at position 'id' came from an upvalue. (That can only happen
/// with instructions OP_GETTABUP/OP_SETTABUP, which operate directly on
/// upvalues.)
fn get_upval_name(state: &LuaState, cl: &LClosure, id: usize) -> Option<(String, String)> {
    for (i, upv) in cl.upvalues.iter().enumerate() {
        if upv.v == id {
            return Some((
                "upvalue".to_owned(),
                state.protos[cl.proto].upvalues[i].name.to_owned(),
            ));
        }
    }
    None
}

pub(crate) fn order_error(
    state: &mut LuaState,
    v1_id: StkId,
    v2_id: StkId,
) -> Result<(), LuaError> {
    let t1 = state.stack[v1_id].get_type_name();
    let t2 = state.stack[v2_id].get_type_name();
    if t1 == t2 {
        state.run_error(&format!("attempt to compare two {} values", t1))
    } else {
        state.run_error(&format!("attempt to compare {} with {}", t1, t2))
    }
}
