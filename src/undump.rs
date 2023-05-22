//! load precompiled Lua chunks

use crate::{api::LuaError, ldo::SParser, object::Proto, state::LuaStateRef};

pub fn undump<T>(_state: LuaStateRef, _parser: &mut SParser<T>) -> Result<Proto, LuaError> {
    Ok(Proto::new())
}
