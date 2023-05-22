pub mod auxlib;
pub mod api;
pub mod state;

#[derive(Default)]
pub struct LuaState {
    pub g: state::GlobalState
}

pub fn tostring(state: &LuaState, idx: i32) -> String {
    api::tolstring(state,idx,None)
}
