//! Standard I/O (and system) library

use std::io::{stdout, Write};

use crate::{api, luaL, state::LuaState, LuaError};

use super::LibReg;

const IO_FUNCS: [LibReg; 11] = [
    LibReg {
        name: "close",
        func: io_close,
    },
    LibReg {
        name: "flush",
        func: io_flush,
    },
    LibReg {
        name: "input",
        func: io_input,
    },
    LibReg {
        name: "lines",
        func: io_lines,
    },
    LibReg {
        name: "open",
        func: io_open,
    },
    LibReg {
        name: "output",
        func: io_output,
    },
    LibReg {
        name: "popen",
        func: io_popen,
    },
    LibReg {
        name: "read",
        func: io_read,
    },
    LibReg {
        name: "tmpfile",
        func: io_tmpfile,
    },
    LibReg {
        name: "type",
        func: io_type,
    },
    LibReg {
        name: "write",
        func: io_write,
    },
];

const FILE_FUNCS: [LibReg; 9] = [
    LibReg {
        name: "close",
        func: io_close,
    },
    LibReg {
        name: "flush",
        func: f_flush,
    },
    LibReg {
        name: "lines",
        func: f_lines,
    },
    LibReg {
        name: "read",
        func: f_read,
    },
    LibReg {
        name: "seek",
        func: f_seek,
    },
    LibReg {
        name: "setvbuf",
        func: f_setvbuf,
    },
    LibReg {
        name: "write",
        func: f_write,
    },
    LibReg {
        name: "__gc",
        func: io_gc,
    },
    LibReg {
        name: "__tostring",
        func: io_tostring,
    },
];

fn create_metatable(state: &mut LuaState) {
    luaL::new_metatable(state, "FILE"); // create metatable for file handles
    api::push_value(state, -1); // push metatable
    api::set_field(state, -2, "__index"); // metatable.__index = metatable
    luaL::set_funcs(state, &FILE_FUNCS, 0); // add file methods to new metatable
    api::pop(state, 1); // pop new metatable
}

pub fn io_close(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn io_flush(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn io_input(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn io_lines(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn io_open(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn io_output(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn io_popen(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn io_read(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn io_tmpfile(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn io_type(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}

fn g_write(state: &mut LuaState, out: &mut dyn Write, arg: isize) -> i32 {
    let mut nargs = api::get_top(state) as isize; // number of arguments
                                                  //let mut status = true;
                                                  // TODO handle formatting errors
    let mut arg = arg;
    while nargs > 0 {
        write!(out, "{}", state.index2adr(arg)).unwrap();
        arg += 1;
        nargs -= 1;
    }
    api::push_boolean(state, true);
    1
}

pub fn io_write(state: &mut LuaState) -> Result<i32, LuaError> {
    Ok(g_write(state, &mut stdout(), 1))
}
pub fn f_flush(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn f_lines(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn f_read(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn f_seek(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn f_setvbuf(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn f_write(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn io_gc(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn io_tostring(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}

pub fn lib_open_io(state: &mut LuaState) -> Result<i32, LuaError> {
    luaL::new_lib(state, &IO_FUNCS);
    create_metatable(state);
    Ok(1)
}
