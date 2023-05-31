//! Standard library for string operations and pattern-matching

use crate::{luaL, state::LuaState};

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

pub fn lib_open_string(state: &mut LuaState) -> Result<i32, ()> {
    luaL::register(state, "string", &STR_FUNCS).unwrap();
    // TODO
    //create_metatable(state);
    Ok(1)
}

fn create_metatable(state: &mut LuaState) {
    todo!();
}

pub fn str_byte(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn str_char(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn str_dump(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn str_find(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn str_format(s: &mut LuaState) -> Result<i32, ()> {
    let value = luaL::check_string(s, 1).map_err(|_| ())?;
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
                        'c' => {
                            match char::from_u32(luaL::check_integer(s, arg).map_err(|_| ())? as u32)
                            {
                                Some(char_val) => res.push_str(&format!("{}", char_val)),
                                None => res.push('�'),
                            }
                        }
                        'd' | 'i' => res
                            .push_str(&format!("{}", luaL::check_integer(s, arg).map_err(|_| ())?)),
                        'o' => res.push_str(&format!(
                            "{:o}",
                            luaL::check_integer(s, arg).map_err(|_| ())?
                        )),
                        'u' => res.push_str(&format!(
                            "{}",
                            luaL::check_integer(s, arg).map_err(|_| ())? as u64
                        )),
                        'x' => res.push_str(&format!(
                            "{:x}",
                            luaL::check_integer(s, arg).map_err(|_| ())?
                        )),
                        'X' => res.push_str(&format!(
                            "{:X}",
                            luaL::check_integer(s, arg).map_err(|_| ())?
                        )),
                        'e' => res.push_str(&format!(
                            "{:e}",
                            luaL::check_number(s, arg).map_err(|_| ())?
                        )),
                        'E' => res.push_str(&format!(
                            "{:E}",
                            luaL::check_number(s, arg).map_err(|_| ())?
                        )),
                        'f' => res
                            .push_str(&format!("{}", luaL::check_number(s, arg).map_err(|_| ())?)),
                        'g' => {
                            let n = luaL::check_number(s, arg).map_err(|_| ())?;
                            if n.abs() <= 1E-5 || n.abs() >= 1E6 {
                                res.push_str(&format!("{:e}", n));
                            } else {
                                res.push_str(&format!("{}", n));
                            }
                        }
                        'G' => {
                            let n = luaL::check_number(s, arg).map_err(|_| ())?;
                            if n.abs() <= 1E-5 || n.abs() >= 1E6 {
                                res.push_str(&format!("{:E}", n));
                            } else {
                                res.push_str(&format!("{}", n));
                            }
                        }
                        's' => {
                            let s = luaL::check_string(s, arg).map_err(|_| ())?;
                            res.push_str(&s);
                        }
                        _ => {
                            luaL::error(s, &format!("invalid option '%{}' to 'format'", c))
                                .map_err(|_| ())?;
                            unreachable!()
                        }
                    }
                }
                None => {
                    luaL::error(s, &format!("invalid conversion '%' to 'format'"))
                        .map_err(|_| ())?;
                    unreachable!()
                }
            }
        }
    }
    s.push_string(&res);
    Ok(1)
}
pub fn str_gfind(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn str_gmatch(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn str_gsub(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn str_len(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn str_lower(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn str_match(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn str_rep(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn str_reverse(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn str_sub(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}
pub fn str_upper(_state: &mut LuaState) -> Result<i32, ()> {
    todo!();
}

#[cfg(test)]
mod tests {
    use crate::{api, luaL, object::TValue};
    #[test]
    fn string_format_d() {
        let mut state = luaL::newstate();
        luaL::open_libs(&mut state).unwrap();
        luaL::dostring(&mut state, "s=string.format('%d',14.3)").unwrap();

        api::get_global(&mut state, "s");
        assert_eq!(
            state.stack.last().unwrap(),
            &TValue::new_string("14")
        );
    }
}
