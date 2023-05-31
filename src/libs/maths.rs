//! Standard mathematical library

use crate::{luaL,api,state::LuaState};

use super::LibReg;

const MATH_FUNCS: [LibReg; 28] = [
    LibReg {
        name: "abs",
        func: math_abs,
    },
    LibReg {
        name: "acos",
        func: math_acos,
    },
    LibReg {
        name: "asin",
        func: math_asin,
    },
    LibReg {
        name: "atan2",
        func: math_atan2,
    },
    LibReg {
        name: "atan",
        func: math_atan,
    },
    LibReg {
        name: "ceil",
        func: math_ceil,
    },
    LibReg {
        name: "cosh",
        func: math_cosh,
    },
    LibReg {
        name: "cos",
        func: math_cos,
    },
    LibReg {
        name: "deg",
        func: math_deg,
    },
    LibReg {
        name: "exp",
        func: math_exp,
    },
    LibReg {
        name: "floor",
        func: math_floor,
    },
    LibReg {
        name: "fmod",
        func: math_fmod,
    },
    LibReg {
        name: "frexp",
        func: math_frexp,
    },
    LibReg {
        name: "ldexp",
        func: math_ldexp,
    },
    LibReg {
        name: "log10",
        func: math_log10,
    },
    LibReg {
        name: "log",
        func: math_log,
    },
    LibReg {
        name: "max",
        func: math_max,
    },
    LibReg {
        name: "min",
        func: math_min,
    },
    LibReg {
        name: "modf",
        func: math_modf,
    },
    LibReg {
        name: "pow",
        func: math_pow,
    },
    LibReg {
        name: "rad",
        func: math_rad,
    },
    LibReg {
        name: "random",
        func: math_random,
    },
    LibReg {
        name: "randomseed",
        func: math_randomseed,
    },
    LibReg {
        name: "sinh",
        func: math_sinh,
    },
    LibReg {
        name: "sin",
        func: math_sin,
    },
    LibReg {
        name: "sqrt",
        func: math_sqrt,
    },
    LibReg {
        name: "tanh",
        func: math_tanh,
    },
    LibReg {
        name: "tan",
        func: math_tan,
    },
];

pub fn lib_open_math(state: &mut LuaState) -> Result<i32,()> {
    luaL::register(state, "math", &MATH_FUNCS).map_err(|_| ())?;
    api::push_number(state, std::f64::consts::PI);
    api::set_field(state, -2, "pi");
    api::push_number(state, f64::INFINITY);
    api::set_field(state,-2,"huge");
    Ok(1)
}

pub fn math_abs(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_acos(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_asin(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_atan2(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_atan(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_ceil(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_cosh(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_cos(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_deg(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_exp(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_floor(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_fmod(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_frexp(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_ldexp(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_log10(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_log(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_max(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_min(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_modf(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_pow(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_rad(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_random(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_randomseed(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_sinh(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_sin(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_sqrt(s: &mut LuaState) -> Result<i32,()> {
    let value = luaL::check_number(s,1).map_err(|_| ())?;
    api::push_number(s, value.sqrt());
    Ok(1)
}
pub fn math_tanh(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}
pub fn math_tan(_state: &mut LuaState) -> Result<i32,()> {
    todo!();
}

#[cfg(test)]
mod tests {
    use crate::{api, luaL, object::TValue};
    #[test]
    fn sqrt() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(&mut state, "z=math.sqrt(16)").unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(4.0));
    }
}

