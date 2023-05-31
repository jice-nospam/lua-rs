//! Debug Interface

use crate::{LuaError, state::LuaState, object::{StkId, TValue, Closure}};

pub(crate) fn error_msg(state: &mut LuaState) ->Result<(),LuaError> {
    todo!()
}

pub(crate) fn concat_error(state: &mut LuaState, p1: isize, p2: isize) -> Result<(), LuaError> {
    let perr = if state.index2adr(p1).is_string() {
        p2
    } else {
        p1
    };
    type_error(state,perr as StkId, "concatenate")
}

pub(crate) fn type_error(state: &mut LuaState, id: StkId, operation: &str) -> Result<(), LuaError> {
    let (base,top) = {
        let ci = &state.base_ci[state.ci];
        (ci.base,ci.top)
    };
    let (kind, objname) = if id >= base && id < top {
        get_obj_name(state,id)
    } else {
        (None, None)
    };
    let tname = state.stack[id].get_type_name();
    if let (Some(kind), Some(objname)) = (kind, objname) {
        return state.run_error(&format!(
            "attempt to {} {} '{}' (a {} value)",
            operation, kind, &objname, tname
        ));
    } else {
        return state.run_error(&format!("attempt to {} a {} value", operation, tname));
    }
}

fn get_obj_name(state: &mut LuaState, id: usize) -> (Option<String>, Option<String>) {
    let ci = &state.base_ci[state.ci];
    let funcvalue = &state.stack[ci.func];
    match funcvalue {
        TValue::Function(cl) => {
            match cl.as_ref() {
                Closure::Lua(cl) => {
                    // TODO
                    (None, None)
                }
                _ => (None, None),
            }
        }
        _ => (None, None),
    }
}