//! Lua virtual machine

use std::{cell::RefCell, rc::Rc};

use crate::{
    api,
    api::LuaError,
    luaD::PrecallStatus,
    luaG,
    object::{Closure, LClosure, StkId, TValue},
    opcodes::{
        get_arg_a, get_arg_b, get_arg_bx, get_arg_c, get_arg_sbx, get_opcode, rk_is_k, OpCode,
        BIT_RK, LFIELDS_PER_FLUSH,
    },
    state::LuaState,
    LuaNumber, LUA_MULTRET,
};

#[cfg(feature = "debug_logs")]
use crate::{debug_println, limits::Instruction, opcodes::OPCODE_NAME};

macro_rules! arith_op {
    ($op: tt, $opcode: expr, $protoid: expr, $state: expr,$i:expr,$base:expr,$ra: expr,$pc:expr) => {
        {
            let b=get_arg_b($i);
            let rbi = ($base + b) as usize;
            let rb = if rk_is_k(b) {
                $state.get_lua_constant($protoid, (b&!BIT_RK) as usize)
            } else {
                $state.stack[rbi].clone()
            };
            let c=get_arg_c($i);
            let rci = ($base + c) as usize;
            let rc = if rk_is_k(c) {
                $state.get_lua_constant($protoid,(c&!BIT_RK) as usize)
            } else {
                $state.stack[rci].clone()
            };
            if rb.is_number() && rc.is_number() {
                let val = rb.get_number_value() $op rc.get_number_value();
                if $ra as usize == $state.stack.len() {
                    $state.stack.push(TValue::Number(val));
                } else {
                    $state.stack[$ra as usize] = TValue::Number(val);
                }
            } else {
                $state.saved_pc = $pc;
                arith($state, $ra, rbi, rci, $opcode)?;
                $base = $state.base as u32;
            }
        }
    }
}

macro_rules! arith_func {
    ($func: tt, $opcode: expr, $protoid: expr, $state: expr,$i:expr,$base:expr,$ra: expr,$pc:expr) => {{
        let b = get_arg_b($i);
        let rbi = ($base + b) as usize;
        let rb = if rk_is_k(b) {
            $state.get_lua_constant($protoid, (b & !BIT_RK) as usize)
        } else {
            $state.stack[rbi].clone()
        };
        let c = get_arg_c($i);
        let rci = ($base + c) as usize;
        let rc = if rk_is_k(c) {
            $state.get_lua_constant($protoid, (c & !BIT_RK) as usize)
        } else {
            $state.stack[rci].clone()
        };
        if rb.is_number() && rc.is_number() {
            let val = rb.get_number_value().$func(rc.get_number_value());
            $state.stack[$ra as usize] = TValue::Number(val);
        } else {
            $state.saved_pc = $pc;
            arith($state, $ra, rbi, rci, $opcode)?;
            $base = $state.base as u32;
        }
    }};
}

impl LuaState {
    #[cfg(feature = "debug_logs")]
    /// disassemble current instruction
    fn dump_debug_log(&self, func: usize, first: bool, pc: usize, i: u32) {
        let cl = if let TValue::Function(cl) = &self.stack[func] {
            cl.borrow()
        } else {
            unreachable!()
        };
        if let Closure::Lua(cl_lua) = &*cl {
            if first {
                dump_function_header(self, cl_lua);
            }
        } else {
            unreachable!()
        };
        if let Closure::Lua(cl_lua) = &*cl {
            debug_println!("[{:04x}] {}", pc, &disassemble(self, i, cl_lua));
        }
    }

    pub(crate) fn vexecute(&mut self, nexec_calls: i32) -> Result<(), LuaError> {
        let mut nexec_calls = nexec_calls;
        'reentry: loop {
            let func = self.base_ci[self.ci].func;
            let mut pc = self.saved_pc;
            let protoid = self.get_lua_closure_protoid(func);
            let mut base = self.base as u32;
            #[cfg(feature = "debug_logs")]
            let mut first = true;
            // main loop of interpreter
            loop {
                let i = self.protos[protoid].code[pc];
                #[cfg(feature = "debug_logs")]
                {
                    self.dump_debug_log(func, first, pc, i);
                    first = false;
                }
                pc += 1;
                // TODO handle hooks
                let ra = base + get_arg_a(i);
                debug_assert!(
                    self.base == base as usize && self.base == self.base_ci[self.ci].base
                );
                match get_opcode(i) {
                    OpCode::Move => {
                        let rb = (base + get_arg_b(i)) as usize;
                        let rai = ra as usize;
                        if rai == self.stack.len() {
                            self.stack.push(self.stack[rb].clone());
                        } else {
                            self.stack[rai] = self.stack[rb].clone();
                        }
                    }
                    OpCode::LoadK => {
                        let kid = get_arg_bx(i);
                        let kname = self.get_lua_constant(protoid, kid as usize);
                        let rai = ra as usize;
                        if rai == self.stack.len() {
                            self.stack.push(kname.clone());
                        } else {
                            self.stack[rai] = kname.clone();
                        }
                    }
                    OpCode::LoadBool => {
                        let b = get_arg_b(i);
                        self.stack[ra as usize] = TValue::Boolean(b != 0);
                        let c = get_arg_c(i);
                        if c != 0 {
                            pc += 1; // skip next instruction (if C)
                        }
                    }
                    OpCode::LoadNil => {
                        let mut b = base + get_arg_b(i);
                        loop {
                            self.stack[b as usize] = TValue::Nil;
                            b -= 1;
                            if b < ra {
                                break;
                            }
                        }
                    }
                    OpCode::GetUpVal => {
                        let b = get_arg_b(i);
                        self.stack[ra as usize] = self.get_lua_closure_upvalue(func, b as usize);
                    }
                    OpCode::GetGlobal => {
                        let kid = get_arg_bx(i);
                        let kname = self.get_lua_constant(protoid, kid as usize);
                        self.saved_pc = pc;
                        let env = self.get_lua_closure_env_value(func);
                        Self::get_tablev2(&mut self.stack, &env, &kname, Some(ra as usize));
                        base = self.base as u32;
                    }
                    OpCode::GetTable => {
                        self.saved_pc = pc;
                        let tableid = (base + get_arg_b(i)) as usize;
                        let key = self.get_rkc(i, base, protoid);
                        Self::get_tablev(&mut self.stack, tableid, &key, Some(ra as usize));
                        base = self.base as u32;
                    }
                    OpCode::SetGlobal => {
                        let g = self.get_lua_closure_env(func);
                        let kid = get_arg_bx(i) as usize;
                        let key = self.get_lua_constant(protoid, kid);
                        self.saved_pc = pc;
                        let value = self.stack[ra as usize].clone();
                        self.set_tablev(&TValue::from(&g), key, value);
                        base = self.base as u32;
                    }
                    OpCode::SetupVal => {
                        let b = get_arg_b(i);
                        self.set_lua_closure_upvalue(
                            func,
                            b as usize,
                            self.stack[ra as usize].clone(),
                        );
                    }
                    OpCode::SetTable => {
                        self.saved_pc = pc;
                        let key = self.get_rkb(i, base, protoid);
                        let value = self.get_rkc(i, base, protoid);
                        self.set_tablev(&self.stack[ra as usize], key, value);
                        base = self.base as u32;
                    }
                    OpCode::NewTable => {
                        self.stack[ra as usize] = TValue::new_table();
                        self.saved_pc = pc;
                        base = self.base as u32;
                    }
                    OpCode::OpSelf => {
                        let rb = base + get_arg_b(i);
                        self.stack[ra as usize + 1] = self.stack[rb as usize].clone();
                        self.saved_pc = pc;
                        let key = self.get_rkc(i, base, protoid);
                        let value = if let TValue::Table(tref) = &self.stack[rb as usize] {
                            let mut t = tref.borrow_mut();
                            t.get(&key).unwrap_or(&TValue::Nil).clone()
                        } else {
                            unreachable!()
                        };
                        self.stack[ra as usize] = value;
                        base = self.base as u32;
                    }
                    OpCode::Add => arith_op!(+,OpCode::Add,protoid,self,i,base,ra,pc),
                    OpCode::Sub => arith_op!(-,OpCode::Sub,protoid,self,i,base,ra,pc),
                    OpCode::Mul => arith_op!(*,OpCode::Mul,protoid,self,i,base,ra,pc),
                    OpCode::Div => arith_op!(/,OpCode::Div,protoid,self,i,base,ra,pc),
                    OpCode::Mod => arith_op!(%,OpCode::Mod,protoid,self,i,base,ra,pc),
                    OpCode::Pow => arith_func!(powf, OpCode::Pow, protoid, self, i, base, ra, pc),
                    OpCode::UnaryMinus => {
                        let rb = (base + get_arg_b(i)) as usize;
                        match &self.stack[rb] {
                            TValue::Number(n) => {
                                self.stack[ra as usize] = TValue::Number(-n);
                            }
                            _ => {
                                self.saved_pc = pc;
                                arith(self, ra, rb, rb, OpCode::UnaryMinus)?;
                                base = self.base as u32;
                            }
                        }
                    }
                    OpCode::Not => {
                        let b=base + get_arg_b(i);
                        let res=self.stack[b as usize].is_false();
                        self.stack[ra as usize] = TValue::Boolean(res);
                    },
                    OpCode::Len => {
                        let rb = (base + get_arg_b(i)) as usize;
                        match &self.stack[rb] {
                            TValue::Table(tref) => {
                                let len = tref.borrow().len() as LuaNumber;
                                self.stack[ra as usize] = TValue::Number(len);
                            }
                            TValue::String(s) => {
                                self.stack[ra as usize] = TValue::Number(s.len() as LuaNumber);
                            }
                            _ => {
                                self.saved_pc = pc;
                                // try metamethod
                                if !call_bin_tm(self, rb, 0, ra, OpCode::Len)? {
                                    return luaG::type_error(self, rb, "get length of");
                                }
                                base = self.base as u32;
                            }
                        }
                    }
                    OpCode::Concat => {
                        let b = get_arg_b(i);
                        let c = get_arg_c(i);
                        self.saved_pc = pc;
                        concat(self, (c - b + 1) as usize, c as usize)?;
                        base = self.base as u32;
                        self.stack[ra as usize] = self.stack[(base + b) as usize].clone();
                    }
                    OpCode::Jmp => {
                        let jmp = get_arg_sbx(i);
                        pc = ((pc as i32) + jmp) as usize;
                    }
                    OpCode::Eq => {
                        let rkb = self.get_rkb(i, base, protoid);
                        let rkc = self.get_rkc(i, base, protoid);
                        self.saved_pc = pc;
                        let a = get_arg_a(i) > 0;
                        if equal_obj(self, rkb, rkc) == a {
                            let i2 = self.protos[protoid].code[pc];
                            let jmp = get_arg_sbx(i2);
                            pc = ((pc as i32) + jmp) as usize;
                        }
                        pc += 1;
                        base = self.base as u32;
                    }
                    OpCode::Lt => {
                        self.saved_pc = pc;
                        let rkb = self.get_rkb(i, base, protoid);
                        let rkc = self.get_rkc(i, base, protoid);
                        let a = get_arg_a(i) > 0;
                        if less_than(self, rkb, rkc)? == a {
                            let i2 = self.protos[protoid].code[pc];
                            let jmp = get_arg_sbx(i2);
                            pc = ((pc as i32) + jmp) as usize;
                        }
                        pc += 1;
                        base = self.base as u32;
                    }
                    OpCode::Le => {
                        self.saved_pc = pc;
                        let rkb = self.get_rkb(i, base, protoid);
                        let rkc = self.get_rkc(i, base, protoid);
                        let a = get_arg_a(i) > 0;
                        if less_equal(self, rkb, rkc)? == a {
                            let i2 = self.protos[protoid].code[pc];
                            let jmp = get_arg_sbx(i2);
                            pc = ((pc as i32) + jmp) as usize;
                        }
                        pc += 1;
                        base = self.base as u32;
                    }
                    OpCode::Test => {
                        let is_false = get_arg_c(i) != 0;
                        if self.stack[ra as usize].is_false() != is_false {
                            let i2 = self.protos[protoid].code[pc];
                            let jump = get_arg_sbx(i2);
                            pc = (pc as i32 + jump) as usize;
                        }
                        pc += 1;
                    }
                    OpCode::TestSet => {
                        let b=base+get_arg_b(i);
                        let c=get_arg_c(i) > 0;
                        if self.stack[b as usize].is_false() != c {
                            self.stack[ra as usize] = self.stack[b as usize].clone();
                            let pci =  self.protos[protoid].code[pc];
                            pc = (pc as i32 + get_arg_sbx(pci)) as usize;
                        }
                        pc+=1;
                    },
                    OpCode::Call => {
                        let b = get_arg_b(i);
                        let nresults = get_arg_c(i) as i32 - 1;
                        if b != 0 {
                            self.stack.resize((ra + b) as usize, TValue::Nil); // top = ra+b
                        } // else previous instruction set top
                        self.saved_pc = pc;
                        match self.dprecall(ra as usize, nresults) {
                            Ok(PrecallStatus::Lua) => {
                                nexec_calls += 1;
                                // restart luaV_execute over new Lua function
                                continue 'reentry;
                            }
                            Ok(PrecallStatus::Rust) => {
                                // it was a Rust function (`precall' called it); adjust results
                                if nresults > 0 && self.stack.len() > self.base_ci[self.ci].top {
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
                    OpCode::TailCall => {
                        let b = get_arg_b(i);
                        if b != 0 {
                            self.stack.resize((ra + b) as usize, TValue::Nil); // top = ra+b
                        } // else previous instruction set top
                        self.saved_pc = pc;    
                        match self.dprecall(ra as usize, LUA_MULTRET) {
                            Ok(PrecallStatus::Lua) => {
                                // tail call: put new frame in place of previous one
                                let pbase = self.base_ci[self.ci].base; // previous base
                                let pfunc=func; // previous function index
                                let nbase = self.base_ci[self.ci-1].base; // new base
                                if !self.open_upval.is_empty() {
                                    self.close_func(nbase);
                                }
                                let mut prevci = &mut self.base_ci[self.ci-1];
                                let func = prevci.func;
                                self.base = func + pbase - pfunc;
                                prevci.base = self.base;
                                let mut aux=0;
                                while pfunc+aux < self.stack.len() {
                                    // move frame down
                                    self.stack[(func+aux) as usize] = self.stack[(pfunc+aux) as usize].clone();
                                    aux+=1;
                                }
                                self.stack.resize((func+aux) as usize, TValue::Nil);
                                prevci.top = self.stack.len(); // correct top
                                prevci.saved_pc = self.saved_pc;
                                prevci.tailcalls+=1; // one more call lost
                                self.base_ci.pop(); // remove new frame
                                self.ci-=1;
                                continue 'reentry;
                            }
                            Ok(PrecallStatus::Rust) => {
                                // it was a Rust function (`precall' called it); adjust results
                                base = self.base as u32;
                            }
                            Ok(PrecallStatus::RustYield) => {
                                return Ok(()); // yield
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }                                            
                    },
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
                        let step = self.stack[ra as usize + 2].get_number_value();
                        let idx = self.stack[ra as usize].get_number_value() + step;
                        let limit = self.stack[ra as usize + 1].get_number_value();
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
                            let rai = ra as usize + 3;
                            if rai == self.stack.len() {
                                self.stack.push(TValue::Number(idx));
                            } else {
                                self.stack[rai] = TValue::Number(idx); // ...and external index
                            }
                        }
                    }
                    OpCode::ForPrep => {
                        self.saved_pc = pc;
                        if Self::to_number(&mut self.stack, ra as usize, Some(ra as usize))
                            .is_none()
                        {
                            return self.run_error("'for' initial value must be a number");
                        }
                        if Self::to_number(&mut self.stack, ra as usize + 1, Some(ra as usize + 1))
                            .is_none()
                        {
                            return self.run_error("'for' limit must be a number");
                        }
                        if Self::to_number(&mut self.stack, ra as usize + 2, Some(ra as usize + 2))
                            .is_none()
                        {
                            return self.run_error("'for' step must be a number");
                        }
                        // init = init - step
                        self.stack[ra as usize] = TValue::Number(
                            self.stack[ra as usize].get_number_value()
                                - self.stack[ra as usize + 2].get_number_value(),
                        );
                        let jump = get_arg_sbx(i);
                        pc = (pc as i32 + jump) as usize;
                    }
                    OpCode::TForLoop => {
                        let mut cb = ra + 3; // call base
                        self.stack[cb as usize + 2] = self.stack[ra as usize + 2].clone();
                        self.stack[cb as usize + 1] = self.stack[ra as usize + 1].clone();
                        self.stack[cb as usize] = self.stack[ra as usize].clone();
                        self.saved_pc = pc;
                        let nresults = get_arg_c(i) as i32;
                        self.dcall(cb as usize, nresults)?;
                        base = self.base as u32;
                        self.stack.resize(self.base_ci[self.ci].top, TValue::Nil);
                        cb = base + get_arg_a(i) + 3;
                        if !self.stack[cb as usize].is_nil() {
                            // continue loop ?
                            self.stack[cb as usize - 1] = self.stack[cb as usize].clone();
                            let pci = self.protos[protoid].code[pc];
                            let jmp = get_arg_sbx(pci);
                            pc = (pc as i32 + jmp) as usize;
                        }
                        pc += 1;
                    }
                    OpCode::SetList => {
                        let mut n = get_arg_b(i);
                        let mut c = get_arg_c(i);
                        if n == 0 {
                            n = self.stack.len() as u32 - ra - 1;
                            self.stack.resize(self.base_ci[self.ci].top, TValue::Nil);
                        }
                        if c == 0 {
                            c = self.protos[protoid].code[pc];
                            pc += 1;
                        }
                        let mut last = (c - 1) * LFIELDS_PER_FLUSH + n;
                        if let TValue::Table(tref) = &self.stack[ra as usize] {
                            let mut t = tref.borrow_mut();
                            while n > 0 {
                                t.set(
                                    TValue::Number(last as LuaNumber),
                                    self.stack[(ra + n) as usize].clone(),
                                );
                                last -= 1;
                                n -= 1;
                            }
                        }
                    }
                    OpCode::Close => todo!(),
                    OpCode::Closure => {
                        let pid = get_arg_bx(i);
                        let pid = self.protos[protoid].p[pid as usize];
                        let p = &self.protos[pid];
                        let nup = p.nups;
                        let ncl = Rc::new(RefCell::new(Closure::Lua(LClosure::new(
                            pid,
                            self.get_lua_closure_env(func),
                        ))));
                        self.stack[ra as usize] = TValue::Function(ncl.clone());
                        let mut ncl = ncl.borrow_mut();
                        for _ in 0..nup {
                            let pci = self.protos[protoid].code[pc];
                            if get_opcode(pci) == OpCode::GetUpVal {
                                let upvalid = get_arg_b(pci);
                                let upval = self.get_lua_closure_upval_desc(func, upvalid as usize);
                                ncl.add_lua_upvalue(upval);
                            } else {
                                debug_assert!(get_opcode(pci) == OpCode::Move);
                                let b = get_arg_b(pci);
                                ncl.add_lua_upvalue(Self::find_upval(
                                    &mut self.open_upval,
                                    &mut self.stack,
                                    base + b,
                                ));
                            }
                            pc += 1;
                        }
                        self.saved_pc = pc;
                        base = self.base as u32;
                    }
                    OpCode::VarArg => todo!(),
                }
            }
        }
    }
}

fn equal_obj(_state: &mut LuaState, rkb: TValue, rkc: TValue) -> bool {
    rkb == rkc
    // TODO metamethod
}

fn less_than(state: &mut LuaState, rkb: TValue, rkc: TValue) -> Result<bool, LuaError> {
    if rkb.get_type_name() != rkc.get_type_name() {
        luaG::order_error(state, &rkb, &rkc)?;
    } else if rkb.is_number() {
        return Ok(rkb.get_number_value() < rkc.get_number_value());
    } else if rkb.is_string() {
        return Ok(rkb.borrow_string_value() < rkc.borrow_string_value());
    }
    // TODO metamethods
    luaG::order_error(state, &rkb, &rkc)?;
    unreachable!()
}

fn less_equal(state: &mut LuaState, rkb: TValue, rkc: TValue) -> Result<bool, LuaError> {
    if rkb.get_type_name() != rkc.get_type_name() {
        luaG::order_error(state, &rkb, &rkc)?;
    } else if rkb.is_number() {
        return Ok(rkb.get_number_value() <= rkc.get_number_value());
    } else if rkb.is_string() {
        return Ok(rkb.borrow_string_value() <= rkc.borrow_string_value());
    }
    // TODO metamethods
    luaG::order_error(state, &rkb, &rkc)?;
    unreachable!()
}

fn arith(state: &mut LuaState, ra: u32, rb: usize, rc: usize, op: OpCode) -> Result<(), LuaError> {
    if let (Some(b), Some(c)) = (
        LuaState::to_number(&mut state.stack, rb, None),
        LuaState::to_number(&mut state.stack, rc, None),
    ) {
        match op {
            OpCode::Add => {
                state.stack[ra as usize] = TValue::Number(b + c);
            }
            OpCode::Sub => {
                state.stack[ra as usize] = TValue::Number(b - c);
            }
            OpCode::Mul => {
                state.stack[ra as usize] = TValue::Number(b * c);
            }
            OpCode::Div => {
                state.stack[ra as usize] = TValue::Number(b / c);
            }
            OpCode::Mod => {
                state.stack[ra as usize] = TValue::Number(b % c);
            }
            OpCode::Pow => {
                state.stack[ra as usize] = TValue::Number(b.powf(c));
            }
            OpCode::UnaryMinus => {
                state.stack[ra as usize] = TValue::Number(-b);
            }
            _ => unreachable!(),
        }
    } else if !call_bin_tm(state, rb, rc, ra, op)? {
        return luaG::arith_error(state, rb, rc);
    }
    Ok(())
}

fn call_bin_tm(
    _state: &mut LuaState,
    _rb: usize,
    _rc: usize,
    _ra: u32,
    _op: OpCode,
) -> Result<bool, LuaError> {
    todo!()
}

#[cfg(feature = "debug_logs")]
fn dump_function_header(state: &LuaState, cl: &LClosure) {
    let nup = cl.upvalues.len();
    let proto = &state.protos[cl.proto];
    let nk = proto.k.len();
    if proto.linedefined == proto.lastlinedefined {
        println!("; function [{}] ", proto.linedefined);
    } else {
        println!(
            "; function [{}-{}] ",
            proto.linedefined, proto.lastlinedefined
        );
    }
    println!(
        "; {} upvalues   {} params   {} stack   {}",
        nup,
        proto.numparams,
        proto.maxstacksize,
        if proto.is_vararg { "vararg" } else { "" }
    );
    for (i, loc) in proto.locvars.iter().enumerate() {
        println!(".local {:<10} ; {}", format!("\"{}\"", loc.name), i);
    }
    for i in 0..nk {
        if proto.k[i].is_string() {
            println!(".const {:<10} ; {}", &format!("{:?}", proto.k[i]), i);
        } else {
            println!(".const {:<10} ; {}", &format!("{}", proto.k[i]), i);
        }
    }
}

#[cfg(feature = "debug_logs")]
fn disassemble(state: &LuaState, i: Instruction, cl: &LClosure) -> String {
    let o = get_opcode(i);
    let a = get_arg_a(i);
    let b = get_arg_b(i);
    let c = get_arg_c(i);
    let sbx = get_arg_sbx(i);
    let bx = get_arg_bx(i);
    let proto = &state.protos[cl.proto];
    let mut res = if o.is_asbx() {
        format!("{:10} {:>5} {:>5}", OPCODE_NAME[o as usize], a, sbx)
    } else if o.is_sbx() {
        format!("{:10} {:>5}", OPCODE_NAME[o as usize], sbx)
    } else if o.is_abx() {
        format!("{:10} {:>5} {:>5}", OPCODE_NAME[o as usize], a, bx)
    } else {
        format!("{:10} {:>5} {:>5} {:>5}", OPCODE_NAME[o as usize], a, b, c)
    };
    match o {
        OpCode::LoadK | OpCode::SetGlobal | OpCode::GetGlobal => {
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

pub(crate) fn concat(state: &mut LuaState, total: usize, last: usize) -> Result<(), LuaError> {
    let top = state.base as usize + last + 1;
    if api::to_string(state, top as isize - 2).is_none()
        || api::to_string(state, top as isize - 1).is_none()
    {
        // TODO metamethods
        return luaG::concat_error(state, top as isize - 2, top as isize - 1);
    } else {
        let mut res = String::new();
        let first = top - total;
        for i in first..top {
            res.push_str(&state.stack[i as usize].to_string());
        }
        state.stack[first] = TValue::from(res);
        Ok(())
    }
}
