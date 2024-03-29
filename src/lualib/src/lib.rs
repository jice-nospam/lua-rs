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
mod table;
mod undump;
mod vm;
#[cfg(target_arch = "wasm32")]
pub mod wasm;
mod zio;

pub use api::*;
pub use auxlib as luaL;
pub(crate) use code as luaK;
pub(crate) use debug as luaG;
pub(crate) use ldo as luaD;
pub(crate) use parser as luaY;
pub(crate) use table as luaH;
pub(crate) use undump as luaU;
pub(crate) use vm as luaV;
pub(crate) use zio as luaZ;
pub use {object::TValue, state::LuaState};

pub type LuaFloat = f64;
pub type LuaInteger = i64;

pub type LuaRustFunction = fn(&mut LuaState) -> Result<i32, ()>;

/// lua bytecode dump header
pub(crate) const LUA_SIGNATURE: &str = "\x1BLua";
/// option for multiple returns in `lua_pcall' and `lua_call'
pub const LUA_MULTRET: i32 = -1;
/// minimum Lua stack available to a Rust function
pub(crate) const LUA_MINSTACK: usize = 20;
// predefined values in the registry
pub const LUA_RIDX_MAINTHREAD: usize = 1;
pub const LUA_RIDX_GLOBALS: usize = 2;
// pseudo-indices
pub const LUA_REGISTRYINDEX: isize = -1000000 - 1000;
pub const LUA_VERSION: &str = "Lua 5.3.6";

pub type Reader<T> = fn(&mut LuaState, &T, &mut Vec<char>) -> Result<(), ()>;

/// Prints to the standard ouput only in debug build.
/// In release build this macro is not compiled thanks to `#[cfg(debug_assertions)]`.
/// see [https://doc.rust-lang.org/std/macro.print.html](https://doc.rust-lang.org/std/macro.print.html) for more info.
#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => (#[cfg(debug_assertions)] print!($($arg)*));
}

#[cfg(test)]
mod tests {
    use crate::{api, luaL, object::TValue, LuaError};
    #[test]
    fn bitor_const() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "z=5|6").unwrap();
        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(7));
    }
    #[test]
    fn bitor() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "function bo(x,y) return x|y end z=bo(5,6)").unwrap();
        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(7));
    }
    #[test]
    fn bitxor_const() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "z=5~7").unwrap();
        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(2));
    }
    #[test]
    fn bitxor() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "function bo(x,y) return x~y end z=bo(5,7)").unwrap();
        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(2));
    }
    #[test]
    fn bitnot_const() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "z=~7").unwrap();
        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(-8));
    }
    #[test]
    fn bitnot() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "function bo(x) return ~x end z=bo(7)").unwrap();
        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(-8));
    }
    #[test]
    fn global_number() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "a=4").unwrap();

        api::get_global(&mut state, "a");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(4));
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
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(7));
    }
    #[test]
    fn add_var() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "a=3;b=4;z=a+b").unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(7));
    }
    #[test]
    fn func() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "function a() return 7; end z=a()").unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(7));
    }
    #[test]
    fn func_add() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "function a(x,y) return x+y; end z=a(3,4)").unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(7));
    }
    #[test]
    fn for_num() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "a=0 for i=1,10 do a=a+i end").unwrap();

        api::get_global(&mut state, "a");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(55));
    }
    #[test]
    fn for_with_step() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "a=0 for i=1,10,2 do a=a+i end").unwrap();

        api::get_global(&mut state, "a");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(25));
    }
    #[test]
    fn for_in() {
        let mut state = luaL::newstate();
        luaL::dostring(
            &mut state,
            "
            a=0
            t={1,3,5,8}
            function iter(t)
                local i=0
                return function()
                    i=i+1
                    return t[i]
                end
            end
            for i in iter(t) do a=a+i end",
        )
        .unwrap();

        api::get_global(&mut state, "a");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(17));
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
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(3025));
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
        assert_eq!(state.stack.last().unwrap(), &TValue::Float(0.5));
    }
    #[test]
    fn table() {
        let mut state = luaL::newstate();
        luaL::dostring(
            &mut state,
            "
            local N=10
            local u={}
            for i=1,N do 
                u[i]=i 
            end
            z=0
            for i=1,N do 
                z=z+u[i] 
            end",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(55));
    }
    #[test]
    fn func_call() {
        let mut state = luaL::newstate();
        luaL::dostring(
            &mut state,
            "
            local function A(x) 
                return x+1 
            end
            local function B(x) 
                return A(x)+2 
            end
            z=B(0)",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(3));
    }
    #[test]
    fn method() {
        let mut state = luaL::newstate();
        luaL::dostring(
            &mut state,
            "
            a={i=3}
            function a:t() return self.i+1 end
            z=a:t()
            ",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(4));
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
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(-3));
    }
    #[test]
    fn set_list() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "local q={2,4,6,8,10} z=q[3]").unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(6));
    }
    #[test]
    fn array_len() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "local q={2,4,6,8,10} z=#q").unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(5));
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
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(4));
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
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(4));
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
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(4));
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
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(4));
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
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(4));
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
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(4));
    }
    #[test]
    fn bool_cond() {
        let mut state = luaL::newstate();
        luaL::dostring(&mut state, "z=2 if nil then z=z+1 end if 3 then z=z+2 end if false then z=z+4 end if {} then z=z+8 end").unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(12));
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
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(1));
        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(2));
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
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(8));
    }
    #[test]
    fn nested_loops() {
        let mut state = luaL::newstate();
        luaL::dostring(
            &mut state,
            "z=0
            for i=0,3 do
                for j=0,3 do
                    for k=1,10 do
                        z=z+i*j
                        if k > 5 then
                            break;
                        end
                    end
                end
            end",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(216));
    }
    #[test]
    fn concat() {
        let mut state = luaL::newstate();
        luaL::dostring(
            &mut state,
            "a='hello'
            b='world'
            z=a..' '..b",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::from("hello world"));
    }
    #[test]
    fn tailcall() {
        let mut state = luaL::newstate();
        luaL::dostring(
            &mut state,
            "function facto(n,acc)
                if n==0 then
                    return acc
                else
                    return facto(n-1,acc*n);
                end
            end
            z=facto(7,1)",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(5040));
    }
    #[test]
    fn vararg() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(
            &mut state,
            "
            function sum(...)
                local args={...};
                return args[1]+args[2]+args[3]
            end
            z=sum(3,8,11)",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(22));
    }
    #[test]
    fn vararg_ipair() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(
            &mut state,
            "
            function sum(...)
                local args={...};
                local sum=0;
                for _,v in ipairs(args) do
                    sum = sum + v
                end
                return sum
            end
            z=sum(3,8,11)",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(22));
    }

    #[test]
    fn pairs_mixed() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(
            &mut state,
            "t={1,3,6}
            t.a=9
            z=0
            for k,v in pairs(t) do
                z = z + v
            end",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(19));
    }
    #[test]
    fn repeat() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(
            &mut state,
            "t=1
            z=0
            repeat
                z=z+t
                t=t*2
            until z > 256",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(511));
    }
    #[test]
    fn while_do() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(
            &mut state,
            "t=1
            z=0
            while z < 256 do
                z=z+t
                t=t*2
            end",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(511));
    }
    #[test]
    fn coercion() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(
            &mut state,
            "t=1
            a='4'
            z=3+t+a+'7'",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Float(15.0));
    }
    #[test]
    fn expression() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(
            &mut state,
            "function f() return 2,5 end
            a,b,c = f(), 1 -- results in 2,1,nil
            d,e,f = 1,f() -- results in 1,2,5",
        )
        .unwrap();

        api::get_global(&mut state, "a");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(2));
        api::get_global(&mut state, "b");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(1));
        api::get_global(&mut state, "c");
        assert_eq!(state.stack.last().unwrap(), &TValue::Nil);
        api::get_global(&mut state, "d");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(1));
        api::get_global(&mut state, "e");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(2));
        api::get_global(&mut state, "f");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(5));
    }
    #[test]
    fn numbers() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(&mut state, "z=3 + 1E1 + 0xa + 0xB").unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(34));
    }
    #[test]
    fn goto() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(
            &mut state,
            "
        t={1,3,5,7,9}
        z=0
        for _,v in ipairs(t) do
            z=z+v
            if v == 5 then
                goto exit
            end
        end
        ::exit::
        ",
        )
        .unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(9));
    }
}
