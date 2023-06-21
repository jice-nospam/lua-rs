//! Standard I/O (and system) library

use std::io::{Write, stdout};

use crate::{luaL, state::LuaState, is_number, api};

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

enum IoType {
    Input,
    Output,
    Error,
}

pub fn lib_open_io(state: &mut LuaState) -> Result<i32, ()> {
    //create_metatable(state);
    // create (private) environment (with fields IO_INPUT, IO_OUTPUT, __close)
    //api::create_table(state,2,1);
    //api::replace(state, LUA_ENVIRONINDEX);
    // open library
    luaL::register(state, "io", &IO_FUNCS).map_err(|_| ())?;
    // create and set default files
    //create_std_file(state,IoType::Input,std::io::stdin(),"stdin");
    //create_std_file(state,IoType::Output,std::io::stdout(),"stdout");
    //create_std_file(state,IoType::Error,std::io::stderr(),"stderr");
    // create environment for 'popen'
    Ok(1)
}

fn create_std_file<T>(_state: &mut LuaState, _input: IoType, _stream: T, _arg: &str) {
    todo!()
}

fn create_metatable(_state: &mut LuaState) {
    todo!()
}

pub fn io_close(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn io_flush(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn io_input(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn io_lines(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn io_open(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn io_output(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn io_popen(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn io_read(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn io_tmpfile(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn io_type(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}

fn g_write(state:&mut LuaState, out: &mut dyn Write, arg: isize) -> i32 {
    let mut nargs = api::get_top(state) as isize; // number of arguments
    //let mut status = true;
    // TODO handle formatting errors
    let mut arg=arg;
    while nargs > 0 {
        if is_number(state, arg) {
            write!(out, "{}",api::to_number(state,arg)).unwrap();
        } else {
            write!(out, "{}", state.index2adr(arg)).unwrap();
        }
        arg+=1;
        nargs-=1;
    }
    api::push_boolean(state,true);
    1
}

pub fn io_write(state: &mut LuaState) -> Result<i32, ()> {
    Ok(g_write(state,&mut stdout(),1))
}
pub fn f_flush(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn f_lines(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn f_read(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn f_seek(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn f_setvbuf(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn f_write(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn io_gc(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn io_tostring(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
