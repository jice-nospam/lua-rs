//! load precompiled Lua chunks

use crate::{api::LuaError, ldo::SParser, object::LClosure, state::LuaState};

pub fn undump<T>(_state: &mut LuaState, _parser: &mut SParser<T>) -> Result<LClosure, LuaError> {
    todo!()
}
