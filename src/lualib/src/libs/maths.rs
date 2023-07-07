//! Standard mathematical library

use crate::{api, luaL, state::LuaState, LuaError, LuaInteger, TValue};

use super::LibReg;

const MATH_FUNCS: [LibReg; 23] = [
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
        name: "atan",
        func: math_atan,
    },
    LibReg {
        name: "ceil",
        func: math_ceil,
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
        name: "tointeger",
        func: math_toint,
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
        name: "ult",
        func: math_ult,
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
        name: "sin",
        func: math_sin,
    },
    LibReg {
        name: "sqrt",
        func: math_sqrt,
    },
    LibReg {
        name: "tan",
        func: math_tan,
    },
    LibReg {
        name: "type",
        func: math_type,
    },
];

pub fn math_abs(s: &mut LuaState) -> Result<i32, LuaError> {
    let value = luaL::check_numeral(s, 1)?;
    if api::is_integer(s, 1) {
        api::push_integer(s, value.abs() as LuaInteger);
    } else {
        api::push_number(s, value.abs());
    }
    Ok(1)
}
pub fn math_acos(s: &mut LuaState) -> Result<i32, LuaError> {
    let value = luaL::check_number(s, 1)?;
    api::push_number(s, value.acos());
    Ok(1)
}
pub fn math_asin(s: &mut LuaState) -> Result<i32, LuaError> {
    let value = luaL::check_number(s, 1)?;
    api::push_number(s, value.asin());
    Ok(1)
}
pub fn math_atan(s: &mut LuaState) -> Result<i32, LuaError> {
    let y = luaL::check_number(s, 1)?;
    let x = luaL::opt_number(s, 2).unwrap_or(1.0);
    api::push_number(s, y.atan2(x));
    Ok(1)
}
pub fn math_ceil(s: &mut LuaState) -> Result<i32, LuaError> {
    let value = luaL::check_number(s, 1)?;
    api::push_number(s, value.ceil());
    Ok(1)
}
pub fn math_cos(s: &mut LuaState) -> Result<i32, LuaError> {
    let value = luaL::check_number(s, 1)?;
    api::push_number(s, value.cos());
    Ok(1)
}
pub fn math_deg(s: &mut LuaState) -> Result<i32, LuaError> {
    let value = luaL::check_number(s, 1)?;
    api::push_number(s, value.to_degrees());
    Ok(1)
}
pub fn math_exp(s: &mut LuaState) -> Result<i32, LuaError> {
    let value = luaL::check_number(s, 1)?;
    api::push_number(s, value.exp());
    Ok(1)
}
pub fn math_floor(s: &mut LuaState) -> Result<i32, LuaError> {
    let value = luaL::check_number(s, 1)?;
    api::push_number(s, value.floor());
    Ok(1)
}
pub fn math_fmod(s: &mut LuaState) -> Result<i32, LuaError> {
    let x = luaL::check_number(s, 1)?;
    let y = luaL::check_number(s, 2)?;
    api::push_number(s, x % y);
    Ok(1)
}
pub fn math_toint(s: &mut LuaState) -> Result<i32, LuaError> {
    match api::to_integer(s, 1) {
        None => api::push_nil(s),
        Some(i) => api::push_integer(s, i),
    }
    Ok(1)
}
pub fn math_ult(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!()
}

pub fn math_type(s: &mut LuaState) -> Result<i32, LuaError> {
    match s.index2adr(1) {
        TValue::Float(_) => api::push_literal(s, "float"),
        TValue::Integer(_) => api::push_literal(s, "integer"),
        _ => api::push_nil(s),
    }
    Ok(1)
}

pub fn math_log(s: &mut LuaState) -> Result<i32, LuaError> {
    let value = luaL::check_number(s, 1)?;
    let res = if api::is_none_or_nil(s, 2) {
        value.ln()
    } else {
        let base = luaL::check_number(s, 2)?;
        if base == 10.0 {
            value.log10()
        } else {
            value.log(base)
        }
    };
    api::push_number(s, res);
    Ok(1)
}
pub fn math_max(s: &mut LuaState) -> Result<i32, LuaError> {
    let n = api::get_top(s) as isize; // number of arguments
    let mut dmax = luaL::check_numeral(s, 1)?;
    let mut maxi = 1;
    for i in 2..=n {
        let d = luaL::check_numeral(s, i)?;
        if d > dmax {
            dmax = d;
            maxi = i;
        }
    }
    if api::is_integer(s, maxi) {
        api::push_integer(s, dmax as LuaInteger);
    } else {
        api::push_number(s, dmax);
    }
    Ok(1)
}

/// Returns the minimum value among its arguments.
pub fn math_min(s: &mut LuaState) -> Result<i32, LuaError> {
    let n = api::get_top(s) as isize; // number of arguments
    let mut dmin = luaL::check_numeral(s, 1)?;
    let mut imin = 1;
    for i in 2..=n {
        let d = luaL::check_numeral(s, i)?;
        if d < dmin {
            dmin = d;
            imin = i;
        }
    }
    if api::is_integer(s, imin) {
        api::push_integer(s, dmin as LuaInteger);
    } else {
        api::push_number(s, dmin);
    }
    Ok(1)
}
pub fn math_modf(s: &mut LuaState) -> Result<i32, LuaError> {
    let value = luaL::check_number(s, 1)?;
    api::push_number(s, value.floor());
    api::push_number(s, value.fract());
    Ok(2)
}
pub fn math_rad(s: &mut LuaState) -> Result<i32, LuaError> {
    let value = luaL::check_number(s, 1)?;
    api::push_number(s, value.to_radians());
    Ok(1)
}
pub fn math_random(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn math_randomseed(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn math_sin(s: &mut LuaState) -> Result<i32, LuaError> {
    let value = luaL::check_number(s, 1)?;
    api::push_number(s, value.sin());
    Ok(1)
}
pub fn math_sqrt(s: &mut LuaState) -> Result<i32, LuaError> {
    let value = luaL::check_number(s, 1)?;
    api::push_number(s, value.sqrt());
    Ok(1)
}
pub fn math_tan(s: &mut LuaState) -> Result<i32, LuaError> {
    let value = luaL::check_number(s, 1)?;
    api::push_number(s, value.tan());
    Ok(1)
}

pub fn lib_open_math(state: &mut LuaState) -> Result<i32, LuaError> {
    luaL::new_lib(state, &MATH_FUNCS);
    api::push_number(state, std::f64::consts::PI);
    api::set_field(state, -2, "pi");
    api::push_number(state, f64::INFINITY);
    api::set_field(state, -2, "huge");
    api::push_integer(state, i64::MAX);
    api::set_field(state, -2, "maxinteger");
    api::push_integer(state, i64::MIN);
    api::set_field(state, -2, "mininteger");
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
        assert_eq!(state.stack.last().unwrap(), &TValue::Float(4.0));
    }
    #[test]
    fn sin() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(&mut state, "z=math.sin(math.pi/2)").unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Float(1.0));
    }
    #[test]
    fn min() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(&mut state, "z=math.min(3,2,5)").unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Integer(2));
    }
    #[test]
    fn max() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(&mut state, "z=math.max(3.0,5.2,2.0)").unwrap();

        api::get_global(&mut state, "z");
        assert_eq!(state.stack.last().unwrap(), &TValue::Float(5.2));
    }
}
