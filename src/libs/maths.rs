//! Standard mathematical library

use crate::{api, luaL, state::LuaState};

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

pub fn lib_open_math(state: &mut LuaState) -> Result<i32, ()> {
    luaL::register(state, "math", &MATH_FUNCS).map_err(|_| ())?;
    api::push_number(state, std::f64::consts::PI);
    api::set_field(state, -2, "pi");
    api::push_number(state, f64::INFINITY);
    api::set_field(state, -2, "huge");
    Ok(1)
}

pub fn math_abs(s: &mut LuaState) -> Result<i32, ()> {
    let value = luaL::check_number(s, 1).map_err(|_| ())?;
    api::push_number(s, value.abs());
    Ok(1)
}
pub fn math_acos(s: &mut LuaState) -> Result<i32, ()> {
    let value = luaL::check_number(s, 1).map_err(|_| ())?;
    api::push_number(s, value.acos());
    Ok(1)
}
pub fn math_asin(s: &mut LuaState) -> Result<i32, ()> {
    let value = luaL::check_number(s, 1).map_err(|_| ())?;
    api::push_number(s, value.asin());
    Ok(1)
}
pub fn math_atan2(s: &mut LuaState) -> Result<i32, ()> {
    let x = luaL::check_number(s, 1).map_err(|_| ())?;
    let y = luaL::check_number(s, 2).map_err(|_| ())?;
    api::push_number(s, y.atan2(x));
    Ok(1)
}
pub fn math_atan(s: &mut LuaState) -> Result<i32, ()> {
    let value = luaL::check_number(s, 1).map_err(|_| ())?;
    api::push_number(s, value.atan());
    Ok(1)
}
pub fn math_ceil(s: &mut LuaState) -> Result<i32, ()> {
    let value = luaL::check_number(s, 1).map_err(|_| ())?;
    api::push_number(s, value.ceil());
    Ok(1)
}
pub fn math_cosh(s: &mut LuaState) -> Result<i32, ()> {
    let value = luaL::check_number(s, 1).map_err(|_| ())?;
    api::push_number(s, value.cosh());
    Ok(1)
}
pub fn math_cos(s: &mut LuaState) -> Result<i32, ()> {
    let value = luaL::check_number(s, 1).map_err(|_| ())?;
    api::push_number(s, value.cos());
    Ok(1)
}
pub fn math_deg(s: &mut LuaState) -> Result<i32, ()> {
    let value = luaL::check_number(s, 1).map_err(|_| ())?;
    api::push_number(s, value.to_degrees());
    Ok(1)
}
pub fn math_exp(s: &mut LuaState) -> Result<i32, ()> {
    let value = luaL::check_number(s, 1).map_err(|_| ())?;
    api::push_number(s, value.exp());
    Ok(1)
}
pub fn math_floor(s: &mut LuaState) -> Result<i32, ()> {
    let value = luaL::check_number(s, 1).map_err(|_| ())?;
    api::push_number(s, value.floor());
    Ok(1)
}
pub fn math_fmod(s: &mut LuaState) -> Result<i32, ()> {
    let x = luaL::check_number(s, 1).map_err(|_| ())?;
    let y = luaL::check_number(s, 2).map_err(|_| ())?;
    api::push_number(s, x % y);
    Ok(1)
}
pub fn math_frexp(_state: &mut LuaState) -> Result<i32, ()> {
    todo!()
}
pub fn math_ldexp(s: &mut LuaState) -> Result<i32, ()> {
    let m = luaL::check_number(s, 1).map_err(|_| ())?;
    let e = luaL::check_integer(s, 2).map_err(|_| ())? as i32;
    api::push_number(s, m * (2.0_f64).powi(e));
    Ok(1)
}
pub fn math_log10(s: &mut LuaState) -> Result<i32, ()> {
    let value = luaL::check_number(s, 1).map_err(|_| ())?;
    api::push_number(s, value.log10());
    Ok(1)
}
pub fn math_log(s: &mut LuaState) -> Result<i32, ()> {
    let value = luaL::check_number(s, 1).map_err(|_| ())?;
    api::push_number(s, value.ln());
    Ok(1)
}
pub fn math_max(s: &mut LuaState) -> Result<i32, ()> {
    let n = api::get_top(s) as isize; // number of arguments
    let mut dmax = luaL::check_number(s, 1).map_err(|_| ())?;
    for i in 2..=n {
        let d = luaL::check_number(s, i).map_err(|_| ())?;
        if d > dmax {
            dmax = d;
        }
    }
    api::push_number(s, dmax);
    Ok(1)
}
pub fn math_min(s: &mut LuaState) -> Result<i32, ()> {
    let n = api::get_top(s) as isize; // number of arguments
    let mut dmin = luaL::check_number(s, 1).map_err(|_| ())?;
    for i in 2..=n {
        let d = luaL::check_number(s, i).map_err(|_| ())?;
        if d < dmin {
            dmin = d;
        }
    }
    api::push_number(s, dmin);
    Ok(1)
}
pub fn math_modf(s: &mut LuaState) -> Result<i32, ()> {
    let value = luaL::check_number(s, 1).map_err(|_| ())?;
    api::push_number(s, value.floor());
    api::push_number(s, value.fract());
    Ok(2)
}
pub fn math_pow(s: &mut LuaState) -> Result<i32, ()> {
    let x = luaL::check_number(s, 1).map_err(|_| ())?;
    let y = luaL::check_number(s, 2).map_err(|_| ())?;
    api::push_number(s, x.powf(y));
    Ok(1)
}
pub fn math_rad(s: &mut LuaState) -> Result<i32, ()> {
    let value = luaL::check_number(s, 1).map_err(|_| ())?;
    api::push_number(s, value.to_radians());
    Ok(1)
}
pub fn math_random(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn math_randomseed(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn math_sinh(s: &mut LuaState) -> Result<i32, ()> {
    let value = luaL::check_number(s, 1).map_err(|_| ())?;
    api::push_number(s, value.sinh());
    Ok(1)
}
pub fn math_sin(s: &mut LuaState) -> Result<i32, ()> {
    let value = luaL::check_number(s, 1).map_err(|_| ())?;
    api::push_number(s, value.sin());
    Ok(1)
}
pub fn math_sqrt(s: &mut LuaState) -> Result<i32, ()> {
    let value = luaL::check_number(s, 1).map_err(|_| ())?;
    api::push_number(s, value.sqrt());
    Ok(1)
}
pub fn math_tanh(s: &mut LuaState) -> Result<i32, ()> {
    let value = luaL::check_number(s, 1).map_err(|_| ())?;
    api::push_number(s, value.tanh());
    Ok(1)
}
pub fn math_tan(s: &mut LuaState) -> Result<i32, ()> {
    let value = luaL::check_number(s, 1).map_err(|_| ())?;
    api::push_number(s, value.tan());
    Ok(1)
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
    #[test]
    fn sin() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(&mut state, "z=math.sin(math.pi/4)").unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(0.5));
    }
    #[test]
    fn min() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(&mut state, "z=math.min(3,2,5)").unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(2.0));
    }
    #[test]
    fn max() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(&mut state, "z=math.max(3,5,2)").unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Number(5.0));
    }
}
