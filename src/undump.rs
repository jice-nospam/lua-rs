//! load precompiled Lua chunks

use crate::{api::LuaError, ldo::SParser, object::Proto, state::LuaState};

pub fn undump<T>(_state: &mut LuaState, _parser: &mut SParser<T>) -> Result<Proto, LuaError> {
    todo!()
}
