//! Lua virtual machine

use crate::{
    api::LuaError,
    luaD::PrecallStatus,
    object::{Closure, TValue},
    opcodes::{get_arg_a, get_arg_b, get_arg_bx, get_arg_c, get_opcode},
    state::LuaState,
};

impl LuaState {
    pub(crate) fn vexecute(&mut self, nexec_calls: i32) -> Result<(), LuaError> {
        let mut nexec_calls = nexec_calls;
        'reentry: loop {
            let func = self.base_ci[self.ci].func;
            let mut pc = self.saved_pc;
            let cl = if let TValue::Function(cl) = &self.stack[func] {
                cl.clone()
            } else {
                unreachable!()
            };
            let mut base = self.base as u32;
            // main loop of interpreter
            loop {
                let i = if let Closure::Lua(cl_lua) = &*cl {
                    cl_lua.proto.code[pc]
                } else {
                    unreachable!()
                };
                pc += 1;
                // TODO handle hooks
                let ra = base + get_arg_a(i);
                debug_assert!(
                    self.base == base as usize && self.base == self.base_ci[self.ci].base
                );
                match get_opcode(i) {
                    crate::opcodes::OpCode::Move => todo!(),
                    crate::opcodes::OpCode::LoadK => {
                        let kid = get_arg_bx(i);
                        let kname = cl.get_lua_constant(kid as usize);
                        self.stack[ra as usize] = kname.clone();
                    }
                    crate::opcodes::OpCode::LoadBool => todo!(),
                    crate::opcodes::OpCode::LoadNil => todo!(),
                    crate::opcodes::OpCode::GetUpVal => todo!(),
                    crate::opcodes::OpCode::GetGlobal => {
                        let kid = get_arg_bx(i);
                        let kname = cl.get_lua_constant(kid as usize);
                        self.saved_pc = pc;
                        self.get_tablev(cl.get_envvalue(), kname, Some(ra as usize));
                        base = self.base as u32;
                    }
                    crate::opcodes::OpCode::GetTable => todo!(),
                    crate::opcodes::OpCode::SetGlobal => todo!(),
                    crate::opcodes::OpCode::SetupVal => todo!(),
                    crate::opcodes::OpCode::SetTable => todo!(),
                    crate::opcodes::OpCode::NewTable => todo!(),
                    crate::opcodes::OpCode::OpSelf => todo!(),
                    crate::opcodes::OpCode::Add => todo!(),
                    crate::opcodes::OpCode::Sub => todo!(),
                    crate::opcodes::OpCode::Mul => todo!(),
                    crate::opcodes::OpCode::Div => todo!(),
                    crate::opcodes::OpCode::Mod => todo!(),
                    crate::opcodes::OpCode::Pow => todo!(),
                    crate::opcodes::OpCode::UnaryMinus => todo!(),
                    crate::opcodes::OpCode::Not => todo!(),
                    crate::opcodes::OpCode::Len => todo!(),
                    crate::opcodes::OpCode::Concat => todo!(),
                    crate::opcodes::OpCode::Jmp => todo!(),
                    crate::opcodes::OpCode::Eq => todo!(),
                    crate::opcodes::OpCode::Lt => todo!(),
                    crate::opcodes::OpCode::Le => todo!(),
                    crate::opcodes::OpCode::Test => todo!(),
                    crate::opcodes::OpCode::TestSet => todo!(),
                    crate::opcodes::OpCode::Call => {
                        let b = get_arg_b(i);
                        let nresults = get_arg_c(i) - 1;
                        if b != 0 {
                            self.stack.resize((ra + b) as usize, TValue::Nil); // top = ra+b
                        }
                        self.saved_pc = pc;
                        match self.dprecall(ra as usize, nresults as i32) {
                            Ok(PrecallStatus::Lua) => {
                                nexec_calls += 1;
                                continue 'reentry;
                            }
                            Ok(PrecallStatus::Rust) => {
                                // it was a Rust function (`precall' called it); adjust results
                                if nresults > 0 {
                                    self.stack.resize(self.base_ci[self.ci].top, TValue::Nil);
                                }
                                base = self.base as u32;
                            }
                            Ok(PrecallStatus::RustYield) => {
                                return Ok(()); // yield
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                    crate::opcodes::OpCode::TailCall => todo!(),
                    crate::opcodes::OpCode::Return => {
                        let b = get_arg_b(i);
                        if b != 0 {
                            self.stack.resize((ra + b - 1) as usize, TValue::Nil);
                        }
                        self.saved_pc = pc;
                        let b = self.poscall(ra);
                        nexec_calls -= 1;
                        if nexec_calls == 0 {
                            return Ok(());
                        }
                        if b {
                            self.stack.resize(self.base_ci[self.ci].top, TValue::Nil);
                        }
                        continue 'reentry;
                    }
                    crate::opcodes::OpCode::ForLoop => todo!(),
                    crate::opcodes::OpCode::ForPrep => todo!(),
                    crate::opcodes::OpCode::TForLoop => todo!(),
                    crate::opcodes::OpCode::SetList => todo!(),
                    crate::opcodes::OpCode::Close => todo!(),
                    crate::opcodes::OpCode::Closure => todo!(),
                    crate::opcodes::OpCode::VarArg => todo!(),
                }
            }
        }
    }
}
