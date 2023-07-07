//! Standard library for string operations and pattern-matching

use crate::{api, luaL, state::LuaState, LuaError};

use super::LibReg;

const STR_FUNCS: [LibReg; 15] = [
    LibReg {
        name: "byte",
        func: str_byte,
    },
    LibReg {
        name: "char",
        func: str_char,
    },
    LibReg {
        name: "dump",
        func: str_dump,
    },
    LibReg {
        name: "find",
        func: str_find,
    },
    LibReg {
        name: "format",
        func: str_format,
    },
    LibReg {
        name: "gfind",
        func: str_gfind,
    },
    LibReg {
        name: "gmatch",
        func: str_gmatch,
    },
    LibReg {
        name: "gsub",
        func: str_gsub,
    },
    LibReg {
        name: "len",
        func: str_len,
    },
    LibReg {
        name: "lower",
        func: str_lower,
    },
    LibReg {
        name: "match",
        func: str_match,
    },
    LibReg {
        name: "rep",
        func: str_rep,
    },
    LibReg {
        name: "reverse",
        func: str_reverse,
    },
    LibReg {
        name: "sub",
        func: str_sub,
    },
    LibReg {
        name: "upper",
        func: str_upper,
    },
];

pub fn lib_open_string(state: &mut LuaState) -> Result<i32, LuaError> {
    luaL::new_lib(state, &STR_FUNCS);
    create_metatable(state);
    Ok(1)
}

fn create_metatable(state: &mut LuaState) {
    api::new_table(state); // create metatable for strings
    api::push_literal(state, ""); // dummy string
    api::push_value(state, -2);
    api::set_metatable(state, -2); // set string metatable
    api::pop(state, 1); // pop dummy string
    api::push_value(state, -2); // string library
    api::set_field(state, -2, "__index"); // ...is the __index metamethod
    api::pop(state, 1); // pop metatable
}

pub fn str_byte(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}

/// Receives zero or more integers.
/// Returns a string with length equal to the number of arguments, in which each character has the internal numerical code equal to its corresponding argument.
/// Note that numerical codes are not necessarily portable across platforms
pub fn str_char(state: &mut LuaState) -> Result<i32, LuaError> {
    let n = api::get_top(state) as isize; // number of arguments
    let mut s = String::new();
    for i in 1..=n {
        let c = luaL::check_integer(state, i)?;
        match char::from_u32(c as u32) {
            Some(c) => s.push(c),
            None => luaL::arg_error(state, i, "invalid value")?,
        }
    }
    state.push_string(&s);
    Ok(1)
}
pub fn str_dump(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn str_find(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn str_format(s: &mut LuaState) -> Result<i32, LuaError> {
    let value = luaL::check_string(s, 1)?;
    let mut ch = value.chars();
    let mut res = String::new();
    let mut arg = 1;
    while let Some(c) = ch.next() {
        if c != '%' {
            res.push(c);
        } else {
            match ch.next() {
                Some(c) if c == '%' => {
                    // %%
                    res.push('%');
                }
                Some(c) => {
                    // format item
                    arg += 1;
                    match c {
                        // TODO support complete printf format
                        'c' => match char::from_u32(luaL::check_integer(s, arg)? as u32) {
                            Some(char_val) => res.push_str(&format!("{}", char_val)),
                            None => res.push('�'),
                        },
                        'd' | 'i' => res.push_str(&format!("{}", luaL::check_integer(s, arg)?)),
                        'o' => res.push_str(&format!("{:o}", luaL::check_integer(s, arg)?)),
                        'u' => res.push_str(&format!("{}", luaL::check_integer(s, arg)? as u64)),
                        'x' => res.push_str(&format!("{:x}", luaL::check_integer(s, arg)?)),
                        'X' => res.push_str(&format!("{:X}", luaL::check_integer(s, arg)?)),
                        'e' => res.push_str(&format!("{:e}", luaL::check_number(s, arg)?)),
                        'E' => res.push_str(&format!("{:E}", luaL::check_number(s, arg)?)),
                        'f' => res.push_str(&format!("{}", luaL::check_number(s, arg)?)),
                        'g' => {
                            let n = luaL::check_number(s, arg)?;
                            if n.abs() <= 1E-5 || n.abs() >= 1E6 {
                                res.push_str(&format!("{:e}", n));
                            } else {
                                res.push_str(&format!("{}", n));
                            }
                        }
                        'G' => {
                            let n = luaL::check_number(s, arg)?;
                            if n.abs() <= 1E-5 || n.abs() >= 1E6 {
                                res.push_str(&format!("{:E}", n));
                            } else {
                                res.push_str(&format!("{}", n));
                            }
                        }
                        's' => {
                            let s = luaL::check_string(s, arg)?;
                            res.push_str(&s);
                        }
                        _ => {
                            luaL::error(s, &format!("invalid option '%{}' to 'format'", c))?;
                            unreachable!()
                        }
                    }
                }
                None => {
                    luaL::error(s, "invalid conversion '%' to 'format'")?;
                    unreachable!()
                }
            }
        }
    }
    s.push_string(&res);
    Ok(1)
}
pub fn str_gfind(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn str_gmatch(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn str_gsub(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn str_len(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn str_lower(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn str_match(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn str_rep(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn str_reverse(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn str_sub(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}
pub fn str_upper(_state: &mut LuaState) -> Result<i32, LuaError> {
    todo!();
}

#[cfg(test)]
mod tests {
    use crate::{api, luaL, object::TValue};
    #[test]
    fn string_format_d() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(&mut state, "s=string.format('%d',14)").unwrap();

        api::get_global(&mut state, "s");
        assert_eq!(state.stack.last().unwrap(), &TValue::from("14"));
    }
    #[test]
    fn string_char() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(&mut state, "s=string.char(72,101,108,108,111)").unwrap();

        api::get_global(&mut state, "s");
        assert_eq!(state.stack.last().unwrap(), &TValue::from("Hello"));
    }
}
