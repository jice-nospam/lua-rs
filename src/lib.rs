pub mod api;
pub mod auxlib;
mod code;
mod debug;
mod func;
mod ldo;
mod lex;
mod libs;
mod limits;
mod luaconf;
mod object;
mod opcodes;
mod parser;
pub mod state;
mod string;
mod table;
mod undump;
mod vm;
mod zio;

pub use api::*;
pub use auxlib as luaL;
pub(crate) use code as luaK;
pub(crate) use debug as luaG;
pub(crate) use ldo as luaD;
pub(crate) use parser as luaY;
use state::LuaState;
pub(crate) use table as luaH;
pub(crate) use undump as luaU;
pub(crate) use vm as luaV;
pub(crate) use zio as luaZ;

pub type LuaNumber = f64;
pub type LuaInteger = i64;

pub type LuaRustFunction = fn(&mut LuaState) -> Result<i32, ()>;

/// lua bytecode dump header
pub const LUA_SIGNATURE: &str = "\x1BLua";
/// option for multiple returns in `lua_pcall' and `lua_call'
pub const LUA_MULTRET: i32 = -1;
/// minimum Lua stack available to a Rust function
pub const LUA_MINSTACK: usize = 20;
// pseudo-indices
pub const LUA_REGISTRYINDEX: isize = -10000;
pub const LUA_ENVIRONINDEX: isize = -10001;
pub const LUA_GLOBALSINDEX: isize = -10002;
pub const LUA_VERSION: &str = "Lua 5.1";

pub type Reader<T> = fn(&mut LuaState, &T, &mut Vec<char>) -> Result<(), ()>;

/// Prints to the standard ouput only in debug build.
/// In release build this macro is not compiled thanks to `#[cfg(debug_assertions)]`.
/// see [https://doc.rust-lang.org/std/macro.print.html](https://doc.rust-lang.org/std/macro.print.html) for more info.
#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => (#[cfg(debug_assertions)] print!($($arg)*));
}

/// Prints to the standard ouput only in debug build.
/// In release build this macro is not compiled thanks to `#[cfg(debug_assertions)]`.
/// see [https://doc.rust-lang.org/std/macro.println.html](https://doc.rust-lang.org/std/macro.println.html) for more info.
#[macro_export]
macro_rules! debug_println {
    ($($arg:tt)*) => (#[cfg(debug_assertions)] println!($($arg)*));
}

#[cfg(test)]
mod tests {
    use crate::{api, luaL, object::TValue, LuaError};
    #[test]
    fn hello_world() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(&mut state, "print('hello world from lua-rs!')").unwrap();
    }
    #[test]
    fn global_number() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "a=4").unwrap();

        api::get_global(&mut state, "a");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(4.0));
    }
    #[test]
    fn global_string() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "a='hello'").unwrap();

        api::get_global(&mut state, "a");
        assert_eq!(state.stack.last().unwrap(), &TValue::from("hello"));
    }
    #[test]
    fn global_bool() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "a=true;b=false").unwrap();

        api::get_global(&mut state, "a");
        assert_eq!(state.stack.last().unwrap(), &TValue::Boolean(true));
        api::get_global(&mut state, "b");
        assert_eq!(state.stack.last().unwrap(), &TValue::Boolean(false));
    }
    #[test]
    fn add_constant() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "z=3+4").unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(7.0));
    }
    #[test]
    fn add_var() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "a=3;b=4;z=a+b").unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(7.0));
    }
    #[test]
    fn func() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "function a() return 7; end z=a()").unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(7.0));
    }
    #[test]
    fn func_add() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "function a(x,y) return x+y; end z=a(3,4)").unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(7.0));
    }
    #[test]
    fn for_num() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "a=0 for i=1,10 do a=a+i end").unwrap();

        api::get_global(&mut state, "a");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(55.0));
    }
    #[test]
    fn nested_for() {
        let mut state = luaL::newstate();
        luaL::dostring(
            &mut state,
            "a=0 for i=1,10 do for j=1,10 do a=a+i*j end end ",
        )
        .unwrap();

        api::get_global(&mut state, "a");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(3025.0));
    }
    #[test]
    fn func_local() {
        let mut state = luaL::newstate();
        luaL::dostring(
            &mut state,
            "local function A(i, j)
                local ij = i+j-1
                return 1.0 / (ij * (ij-1) * 0.5 + i)
            end
            z= A(1,2)",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(0.5));
    }
    #[test]
    fn table() {
        let mut state = luaL::newstate();
        luaL::dostring(
            &mut state,
            "local N=10\nlocal u={}\nfor i=1,N do u[i]=i end\nz=0\nfor i=1,N do z=z+u[i] end",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(55.0));
    }
    #[test]
    fn func_call() {
        let mut state = luaL::newstate();
        luaL::dostring(
            &mut state,
            "local function A(x) return x+1 end\nlocal function B(x) return A(x)+2 end\nz=B(0)",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(3.0));
    }
    #[test]
    fn unknown_lib() {
        let mut state = luaL::newstate();
        let r = luaL::dostring(&mut state, "ia.write('hello')");

        match r {
            Err(e) => {
                assert_eq!(e, LuaError::RuntimeError);
            }
            _ => {
                assert!(false);
            }
        }
        let msg = api::to_string(&mut state, -1);
        assert_eq!(
            msg,
            Some("ia.write('hello'):1 attempt to call a nil value".to_owned())
        );
    }
    #[test]
    fn unary_minus() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "local a={x=-1,y=-2} z=a.x+a.y").unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(-3.0));
    }
    #[test]
    fn set_list() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "local q={2,4,6,8,10} z=q[3]").unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(6.0));
    }
    #[test]
    fn array_len() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "local q={2,4,6,8,10} z=#q").unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(5.0));
    }
    #[test]
    fn lt() {
        let mut state = luaL::newstate();
        luaL::dostring(
            &mut state,
            "z=2 if 5<3 then z=z+1 end if 3<5 then z=z+2 end",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(4.0));
    }
    #[test]
    fn gt() {
        let mut state = luaL::newstate();
        luaL::dostring(
            &mut state,
            "z=2 if 3>5 then z=z+1 end if 5>3 then z=z+2 end",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(4.0));
    }
    #[test]
    fn eq() {
        let mut state = luaL::newstate();
        luaL::dostring(
            &mut state,
            "z=2 if 5==3 then z=z+1 end if 3==3 then z=z+2 end",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(4.0));
    }
    #[test]
    fn ne() {
        let mut state = luaL::newstate();
        luaL::dostring(
            &mut state,
            "z=2 if 3~=3 then z=z+1 end if 3~=5 then z=z+2 end",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(4.0));
    }
    #[test]
    fn le() {
        let mut state = luaL::newstate();
        luaL::dostring(
            &mut state,
            "z=2 if 5<=3 then z=z+1 end if 3<=3 then z=z+2 end",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(4.0));
    }
    #[test]
    fn ge() {
        let mut state = luaL::newstate();
        luaL::dostring(
            &mut state,
            "z=2 if 3>=5 then z=z+1 end if 3>=3 then z=z+2 end",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(4.0));
    }
    #[test]
    fn test() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "z=2 if nil then z=z+1 end if 3 then z=z+2 end if false then z=z+4 end if {} then z=z+8 end").unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(12.0));
    }

    #[test]
    fn closure_upvalues() {
        let mut state = luaL::newstate();
        luaL::dostring(
            &mut state,
            "local function counter()
                local i=0
                return function() i=i+1 return i end
            end

            local c=counter()
            a=c()
            z=c()",
        )
        .unwrap();

        api::get_global(&mut state, "a");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(1.0));
        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(2.0));
    }
    #[test]
    fn recursion() {
        let mut state = luaL::newstate();
        luaL::dostring(
            &mut state,
            "local function fibo(n)
                if n<=2 then
                    return n
                else
                    return fibo(n-1)+fibo(n-2)
                end
            end

            z=fibo(5)",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(8.0));
    }
}
