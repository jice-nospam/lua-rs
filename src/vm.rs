//! Lua virtual machine

use std::rc::Rc;

use crate::{
    luaG,
    api, api::LuaError,
    luaD::PrecallStatus,
    object::{Closure, LClosure, TValue, StkId},
    opcodes::{get_arg_a, get_arg_b, get_arg_bx, get_arg_c, get_arg_sbx, get_opcode, OpCode, RK_IS_K, BIT_RK, OPCODE_NAME},
    state::LuaState, debug_println, limits::Instruction,
};

macro_rules! arith_op {
    ($op: tt, $opcode: expr, $cl: expr, $state: expr,$i:expr,$base:expr,$ra: expr,$pc:expr) => {
        {
            let b=get_arg_b($i);
            let rb = if RK_IS_K(b) {
                $cl.get_lua_constant((b&!BIT_RK) as usize)
            } else {
                $state.stack[($base + b) as usize].clone()
            };
            let c=get_arg_c($i);
            let rc = if RK_IS_K(c) {
                $cl.get_lua_constant((c&!BIT_RK) as usize)
            } else {
                $state.stack[($base + c) as usize].clone()
            };
            if rb.is_number() && rc.is_number() {
                let val = rb.get_number_value() $op rc.get_number_value();
                $state.stack[$ra as usize] = TValue::Number(val);
            } else {
                $state.saved_pc = $pc;
                todo!();
                $base = $state.base as u32;
            }
        }
    }
}

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
            let mut first=true;
            // main loop of interpreter
            loop {
                let i = if let Closure::Lua(cl_lua) = &*cl {
                    #[cfg(debug_assertions)] if first {dump_function_header(cl_lua);first=false;}
                    cl_lua.proto.borrow().code[pc]
                } else {
                    unreachable!()
                };
                #[cfg(debug_assertions)] if let Closure::Lua(cl_lua) = &*cl {
                    debug_println!("[{:04x}] {}",pc,&disassemble(i,cl_lua));
                }
                pc += 1;
                // TODO handle hooks
                let ra = base + get_arg_a(i);
                debug_assert!(
                    self.base == base as usize && self.base == self.base_ci[self.ci].base
                );
                match get_opcode(i) {
                    OpCode::Move => {
                        let rb=(base + get_arg_b(i)) as usize;
                        self.stack[ra as usize]=self.stack[rb].clone();
                    },
                    OpCode::LoadK => {
                        let kid = get_arg_bx(i);
                        let kname = cl.get_lua_constant(kid as usize);
                        self.stack[ra as usize] = kname.clone();
                    }
                    OpCode::LoadBool => {
                        let b=get_arg_b(i);
                        self.stack[ra as usize]=TValue::Boolean(b!=0);
                        let c=get_arg_c(i);
                        if c != 0 {
                            pc+=1; // skip next instruction (if C)
                        }
                    },
                    OpCode::LoadNil => todo!(),
                    OpCode::GetUpVal => {
                        let b=get_arg_b(i);
                        self.stack[ra as usize] = cl.get_lua_upvalue(b as usize);
                    },
                    OpCode::GetGlobal => {
                        let kid = get_arg_bx(i);
                        let kname = cl.get_lua_constant(kid as usize);
                        self.saved_pc = pc;
                        self.get_tablev(cl.get_envvalue(), &kname, Some(ra as usize));
                        base = self.base as u32;
                    }
                    OpCode::GetTable => {
                        self.saved_pc = pc;
                        let table = self.stack[(base + get_arg_b(i)) as usize].clone();
                        let c=get_arg_c(i);
                        let key = if RK_IS_K(c) {
                            cl.get_lua_constant((c & !BIT_RK) as usize)
                        } else {
                            self.stack[(base + c) as usize].clone()
                        };
                        self.get_tablev(&table, &key, Some(ra as usize));
                        base = self.base as u32;
                    },
                    OpCode::SetGlobal => {
                        let g= cl.get_env().clone();
                        let kid = get_arg_bx(i) as usize;
                        let key = cl.get_lua_constant(kid);
                        self.saved_pc = pc;
                        let value=self.stack[ra as usize].clone();
                        self.set_tablev(&TValue::Table(g), key, value);
                        base = self.base as  u32;
                    },
                    OpCode::SetupVal => todo!(),
                    OpCode::SetTable => {
                        self.saved_pc = pc;
                        let b=get_arg_b(i);
                        let c = get_arg_c(i);
                        let key = if RK_IS_K(b) {
                            cl.get_lua_constant((b &!BIT_RK) as usize)
                        } else {
                            self.stack[(base + b) as usize].clone()
                        };
                        let value = if RK_IS_K(c) {
                            cl.get_lua_constant((c &!BIT_RK) as usize)
                        } else {
                            self.stack[(base + c) as usize].clone()
                        };
                        self.set_tablev(&self.stack[ra as usize], key, value);
                        base=self.base as u32;
                    },
                    OpCode::NewTable => {
                        self.stack[ra as usize] = TValue::new_table();
                        self.saved_pc = pc;
                        base=  self.base as u32;
                    },
                    OpCode::OpSelf => todo!(),
                    OpCode::Add => arith_op!(+,OpCode::Add,cl,self,i,base,ra,pc),
                    OpCode::Sub => arith_op!(-,OpCode::Sub,cl,self,i,base,ra,pc),
                    OpCode::Mul => arith_op!(*,OpCode::Mul,cl,self,i,base,ra,pc),
                    OpCode::Div => arith_op!(/,OpCode::Div,cl,self,i,base,ra,pc),
                    OpCode::Mod => arith_op!(%,OpCode::Mod,cl,self,i,base,ra,pc),
                    OpCode::Pow => todo!(),
                    OpCode::UnaryMinus => todo!(),
                    OpCode::Not => todo!(),
                    OpCode::Len => todo!(),
                    OpCode::Concat => todo!(),
                    OpCode::Jmp => todo!(),
                    OpCode::Eq => todo!(),
                    OpCode::Lt => todo!(),
                    OpCode::Le => todo!(),
                    OpCode::Test => {
                        let is_false = if get_arg_c(i) == 0 { false } else { true };
                        let pci = if let Closure::Lua(cl_lua) = &*cl {
                            cl_lua.proto.borrow().code[pc]
                        } else {
                            unreachable!()
                        };
                        if self.stack[ra as usize].is_false() != is_false {
                            let jump = get_arg_sbx(pci);
                            pc = (pc as i32 + jump) as usize;
                        }
                        pc += 1;
                    }
                    OpCode::TestSet => todo!(),
                    OpCode::Call => {
                        let b = get_arg_b(i);
                        let nresults = get_arg_c(i) as i32 - 1;
                        if b != 0 {
                            self.stack.resize((ra + b) as usize, TValue::Nil); // top = ra+b
                        } // else previous instruction set top
                        self.saved_pc = pc;
                        match self.dprecall(ra as usize, nresults as i32) {
                            Ok(PrecallStatus::Lua) => {
                                nexec_calls += 1;
                                // restart luaV_execute over new Lua function
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
                    OpCode::TailCall => todo!(),
                    OpCode::Return => {
                        let b = get_arg_b(i);
                        if b != 0 {
                            self.stack.resize((ra + b - 1) as usize, TValue::Nil);
                        }
                        if !self.open_upval.is_empty() {
                            self.close_func(base as StkId);
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
                    OpCode::ForLoop => {
                        let step = self.stack[ra as usize+2].get_number_value();
                        let idx = self.stack[ra as usize].get_number_value() + step;
                        let limit = self.stack[ra as usize+1].get_number_value();
                        let end_loop = if step > 0.0 {
                            idx <= limit
                        } else {
                            limit <= idx
                        };
                        if end_loop {
                            // jump back
                            let jump = get_arg_sbx(i);
                            pc = (pc as i32 + jump) as usize;
                            self.stack[ra as usize] = TValue::Number(idx); // update internal index
                            self.stack[ra as usize+3] = TValue::Number(idx); // ...and external index
                        }
                    },
                    OpCode::ForPrep => {
                        self.saved_pc = pc;
                        if ! self.to_number(ra as usize,ra as usize) {
                            return self.run_error("'for' initial value must be a number");
                        }
                        if ! self.to_number(ra as usize+1, ra as usize+1) {
                            return self.run_error("'for' limit must be a number");
                        }
                        if ! self.to_number(ra as usize+2, ra as usize+2) {
                            return self.run_error("'for' step must be a number");
                        }
                        // init = init - step
                        self.stack[ra as usize] = TValue::Number(
                            self.stack[ra as usize].get_number_value()
                            - self.stack[ra as usize+2].get_number_value()
                        );
                        let jump = get_arg_sbx(i);
                        pc = (pc as i32 + jump) as usize;
                    },
                    OpCode::TForLoop => todo!(),
                    OpCode::SetList => todo!(),
                    OpCode::Close => todo!(),
                    OpCode::Closure => {
                        let pci = if let Closure::Lua(cl_lua) = &*cl {
                            cl_lua.proto.borrow().code[pc]
                        } else {
                            unreachable!()
                        };
                        if let Closure::Lua(cl) = &*cl {
                            let pid = get_arg_bx(i);
                            let p = &cl.proto.borrow().p[pid as usize];
                            let nup = p.borrow().nups;
                            let mut ncl = LClosure::new(p.clone(), cl.env.clone());
                            for _ in 0..nup {
                                if get_opcode(pci) == OpCode::GetUpVal {
                                    let upvalid = get_arg_b(pci);
                                    ncl.upvalues.push(cl.upvalues[upvalid as usize].clone());
                                } else {
                                    debug_assert!(get_opcode(pci) == OpCode::Move);
                                    let b = get_arg_b(pci);
                                    ncl.upvalues.push(self.find_upval(base + b));
                                }
                            }
                            self.stack[ra as usize] = TValue::Function(Rc::new(Closure::Lua(ncl)));
                            self.saved_pc = pc;
                            base = self.base as u32;
                        } else {
                            unreachable!()
                        }
                    }
                    OpCode::VarArg => todo!(),
                }
            }
        }
    }
}

fn dump_function_header(cl: &LClosure) {
    let nup = cl.upvalues.len();
    let proto = cl.proto.borrow();
    let nk = proto.k.len();
    if proto.linedefined == proto.lastlinedefined {
        println!("; function [{}] ", proto.linedefined);
    } else {
        println!("; function [{}-{}] ", proto.linedefined,proto.lastlinedefined);
    }
    println!("; {} upvalues   {} params   {} stack   {}",nup,proto.numparams,proto.maxstacksize, if proto.is_vararg {"vararg"} else {""});
    for (i,loc) in proto.locvars.iter().enumerate() {
        println!(".local {:<10} ; {}",format!("\"{}\"",loc.name),i);
    }
    for i in 0..nk {
        if proto.k[i].is_string() {
            println!(".const {:<10} ; {}",&format!("{:?}",proto.k[i]),i);
        } else {
            println!(".const {:<10} ; {}",&format!("{}",proto.k[i]),i);
        }
    }
}

fn disassemble(i: Instruction, cl : &LClosure) -> String {
    let o = get_opcode(i);
    let a = get_arg_a(i);
    let b = get_arg_b(i);
    let c = get_arg_c(i);
    let sbx = get_arg_sbx(i);
    let bx = get_arg_bx(i);
    let proto=cl.proto.borrow();
    let mut res = if o.is_asbx() {
        format!("{:10} {:>5} {:>5}", OPCODE_NAME[o as usize], a, sbx)
    } else if o.is_abx() {
        format!("{:10} {:>5} {:>5}", OPCODE_NAME[o as usize], a, bx)
    } else {
        format!("{:10} {:>5} {:>5} {:>5}", OPCODE_NAME[o as usize], a, b, c)
    };
    match o {
        OpCode::LoadK | OpCode::SetGlobal | OpCode::GetGlobal=> {
            if proto.k[bx as usize].is_string() {
                res.push_str(&format!("       ; {:?}", proto.k[bx as usize]));
            } else {
                res.push_str(&format!("       ; {}", proto.k[bx as usize]));
            }
        }
        _ => (),
    }
    res
}

pub(crate) fn concat(state: &mut LuaState, total: usize, last: usize) -> Result<(),LuaError> {
    let mut total=total;
    loop {
        let top = state.base as isize+ last as isize+ 1;
        let mut n=2; // number of elements handled in this pass (at least 2)
        if api::to_string(state, top -2).is_none() || api::to_string(state, top-1).is_none() {
            // TODO metamethods
            return luaG::concat_error(state,top-2,top-1);
        } else{
            todo!();
        }
    }
}

