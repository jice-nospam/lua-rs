use crate::LuaState;

pub type PanicFunction = fn(&LuaState) -> i32;

#[derive(Default)]
pub struct GlobalState {
    pub panic: Option<PanicFunction>,
}

