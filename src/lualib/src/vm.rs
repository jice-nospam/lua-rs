//! Lua virtual machine

use std::{cell::RefCell, rc::Rc};

use crate::{
    api,
    api::LuaError,
    luaD::PrecallStatus,
    luaG,
    luaK::arith,
    object::{Closure, LClosure, StkId, TValue},
    opcodes::{
        get_arg_a, get_arg_ax, get_arg_b, get_arg_bx, get_arg_c, get_arg_sbx, get_opcode, OpCode,
        LFIELDS_PER_FLUSH,
    },
    state::{LuaState, CIST_FRESH, CIST_LUA, CIST_TAIL},
    LuaInteger, LUA_MULTRET,
};

#[cfg(feature = "debug_logs")]
use crate::{limits::Instruction, opcodes::OPCODE_NAME};

impl LuaState {
    #[cfg(feature = "debug_logs")]
    /// disassemble current instruction
    fn dump_debug_log(&mut self, func: usize, first: bool, pc: usize, i: u32) {
        if first {
            dump_function_header(self, func);
        }
        _ = writeln!(self.stdout, "[{:04x}] {}", pc, &disassemble(self, i, func));
    }

    pub(crate) fn vexecute(&mut self) -> Result<(), LuaError> {
        self.base_ci[self.ci].call_status |= CIST_FRESH;
        'new_frame: loop {
            let func = self.base_ci[self.ci].func;
            let protoid = self.get_lua_closure_protoid(func);
            let mut base = self.base_ci[self.ci].base as u32;
            #[cfg(feature = "debug_logs")]
            let mut first = true;
            // main loop of interpreter
            loop {
                let pc = self.base_ci[self.ci].saved_pc;
                let i = self.get_instruction(protoid, pc);
                #[cfg(feature = "debug_logs")]
                {
                    self.dump_debug_log(func, first, pc, i);
                    first = false;
                }
                self.base_ci[self.ci].saved_pc += 1;
                // TODO handle hooks
                let ra = base + get_arg_a(i);
                debug_assert!(base == self.base_ci[self.ci].base as u32);
                match get_opcode(i) {
                    OpCode::BinaryAnd => {
                        let rb = self.get_rkb(i, base, protoid);
                        let rc = self.get_rkc(i, base, protoid);
                        self.set_stack_from_value(ra as usize, arith(OpCode::BinaryAnd, &rb, &rc));
                    }
                    OpCode::BinaryOr => {
                        let rb = self.get_rkb(i, base, protoid);
                        let rc = self.get_rkc(i, base, protoid);
                        self.set_stack_from_value(ra as usize, arith(OpCode::BinaryOr, &rb, &rc));
                    }
                    OpCode::BinaryXor => {
                        let rb = self.get_rkb(i, base, protoid);
                        let rc = self.get_rkc(i, base, protoid);
                        self.set_stack_from_value(ra as usize, arith(OpCode::BinaryXor, &rb, &rc));
                    }
                    OpCode::Shl => {
                        let rb = self.get_rkb(i, base, protoid);
                        let rc = self.get_rkc(i, base, protoid);
                        self.set_stack_from_value(ra as usize, arith(OpCode::Shl, &rb, &rc));
                    }
                    OpCode::Shr => {
                        let rb = self.get_rkb(i, base, protoid);
                        let rc = self.get_rkc(i, base, protoid);
                        self.set_stack_from_value(ra as usize, arith(OpCode::Shr, &rb, &rc));
                    }
                    OpCode::IntegerDiv => {
                        let rb = self.get_rkb(i, base, protoid);
                        let rc = self.get_rkc(i, base, protoid);
                        self.set_stack_from_value(ra as usize, arith(OpCode::IntegerDiv, &rb, &rc));
                    }
                    OpCode::BinaryNot => {
                        let rb = get_rb(base, i);
                        let rb = &self.stack[rb];
                        self.set_stack_from_value(ra as usize, arith(OpCode::BinaryNot, rb, rb));
                    }
                    OpCode::Move => {
                        let rb = get_rb(base, i);
                        let rai = ra as usize;
                        self.set_or_push(rai, self.stack[rb].clone());
                    }
                    OpCode::LoadK => {
                        let kid = get_arg_bx(i);
                        let kval = self.get_lua_constant(protoid, kid as usize);
                        let rai = ra as usize;
                        self.set_or_push(rai, kval.clone());
                    }
                    OpCode::LoadKx => {
                        let ci_pc = self.base_ci[self.ci].saved_pc;
                        let ci_inst = self.get_instruction(protoid, ci_pc);
                        debug_assert!(get_opcode(ci_inst) == OpCode::ExtraArg);
                        let kid = get_arg_ax(ci_inst);
                        self.base_ci[self.ci].saved_pc += 1;
                        let kval = self.get_lua_constant(protoid, kid as usize);
                        let rai = ra as usize;
                        self.set_or_push(rai, kval.clone());
                    }
                    OpCode::LoadBool => {
                        let b = get_arg_b(i);
                        self.set_stack_from_value(ra as usize, TValue::Boolean(b != 0));
                        let c = get_arg_c(i);
                        if c != 0 {
                            self.base_ci[self.ci].saved_pc += 1; // skip next instruction (if C)
                        }
                    }
                    OpCode::LoadNil => {
                        let mut b = get_arg_b(i);
                        let mut ra = ra;
                        while b > 0 {
                            self.set_stack_from_value(ra as usize, TValue::Nil);
                            ra += 1;
                            b -= 1;
                        }
                    }
                    OpCode::GetUpVal => {
                        let b = get_arg_b(i);
                        self.set_stack_from_value(
                            ra as usize,
                            self.get_lua_closure_upvalue(func, b as usize),
                        );
                    }
                    OpCode::GetTabUp => {
                        let b = get_arg_b(i);
                        let key = self.get_rkc(i, base, protoid);
                        let table = self.get_lua_closure_upvalue(func, b as usize);
                        Self::get_tablev2(&mut self.stack, &table, &key, Some(ra as usize));
                        base = self.base_ci[self.ci].base as u32;
                    }
                    OpCode::GetTable => {
                        let tableid = get_rb(base, i);
                        let key = self.get_rkc(i, base, protoid);
                        Self::get_tablev(&mut self.stack, tableid, &key, Some(ra as usize));
                        base = self.base_ci[self.ci].base as u32;
                    }
                    OpCode::SetTabUp => {
                        let a = get_arg_a(i);
                        let key = self.get_rkb(i, base, protoid);
                        let val = self.get_rkc(i, base, protoid);
                        let table = self.get_lua_closure_upvalue(func, a as usize);
                        self.set_tablev(&table, key, val);
                        base = self.base_ci[self.ci].base as u32;
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
                        let key = self.get_rkb(i, base, protoid);
                        let value = self.get_rkc(i, base, protoid);
                        self.set_tablev(&self.stack[ra as usize], key, value);
                        base = self.base_ci[self.ci].base as u32;
                    }
                    OpCode::NewTable => {
                        self.set_stack_from_value(ra as usize, TValue::new_table());
                    }
                    OpCode::OpSelf => {
                        let rb = get_rb(base, i);
                        self.set_stack_from_idx(ra as usize + 1, rb as usize);
                        let key = self.get_rkc(i, base, protoid);
                        Self::get_tablev(&mut self.stack, rb, &key, Some(ra as usize));
                        base = self.base_ci[self.ci].base as u32;
                    }
                    OpCode::Add => {
                        let rb = self.get_rkb(i, base, protoid);
                        let rc = self.get_rkc(i, base, protoid);
                        self.set_stack_from_value(ra as usize, arith(OpCode::Add, &rb, &rc));
                    }
                    OpCode::Sub => {
                        let rb = self.get_rkb(i, base, protoid);
                        let rc = self.get_rkc(i, base, protoid);
                        self.set_stack_from_value(ra as usize, arith(OpCode::Sub, &rb, &rc));
                    }
                    OpCode::Mul => {
                        let rb = self.get_rkb(i, base, protoid);
                        let rc = self.get_rkc(i, base, protoid);
                        self.set_stack_from_value(ra as usize, arith(OpCode::Mul, &rb, &rc));
                    }
                    OpCode::Div => {
                        let rb = self.get_rkb(i, base, protoid);
                        let rc = self.get_rkc(i, base, protoid);
                        self.set_stack_from_value(ra as usize, arith(OpCode::Div, &rb, &rc));
                    }
                    OpCode::Mod => {
                        let rb = self.get_rkb(i, base, protoid);
                        let rc = self.get_rkc(i, base, protoid);
                        self.set_stack_from_value(ra as usize, arith(OpCode::Mod, &rb, &rc));
                    }
                    OpCode::Pow => {
                        let rb = self.get_rkb(i, base, protoid);
                        let rc = self.get_rkc(i, base, protoid);
                        self.set_stack_from_value(ra as usize, arith(OpCode::Pow, &rb, &rc));
                    }
                    OpCode::UnaryMinus => {
                        let rb = get_rb(base, i);
                        let rb = &self.stack[rb];
                        self.set_stack_from_value(ra as usize, arith(OpCode::UnaryMinus, rb, rb));
                    }
                    OpCode::Not => {
                        let b = get_rb(base, i);
                        let res = self.stack[b as usize].is_false(); // next assignment may change this value
                        self.set_stack_from_value(ra as usize, TValue::Boolean(res));
                    }
                    OpCode::Len => {
                        let rb = get_rb(base, i);
                        match &self.stack[rb] {
                            TValue::Table(tref) => {
                                let len = tref.borrow().len() as LuaInteger;
                                self.set_stack_from_value(ra as usize, TValue::Integer(len));
                            }
                            TValue::String(s) => {
                                self.set_stack_from_value(
                                    ra as usize,
                                    TValue::Integer(s.len() as LuaInteger),
                                );
                            }
                            _ => {
                                // try metamethod
                                if !call_bin_tm(self, rb, 0, ra, OpCode::Len)? {
                                    return luaG::type_error(self, rb, "get length of");
                                }
                            }
                        }
                        base = self.base_ci[self.ci].base as u32;
                    }
                    OpCode::Concat => {
                        let b = get_arg_b(i);
                        let c = get_arg_c(i);
                        self.stack
                            .resize(base as usize + c as usize + 1, TValue::Nil); // mark the end of concat operands
                        concat(self, (c + 1 - b) as usize)?;
                        let ra = base + get_arg_a(i);
                        let rb = base + b;
                        self.set_stack_from_idx(ra as StkId, rb as StkId);
                        self.stack.resize(self.base_ci[self.ci].top, TValue::Nil);
                        // restore top
                    }
                    OpCode::Jmp => {
                        self.do_jump(i, 0);
                    }
                    OpCode::Eq => {
                        let rkb = self.get_rkb(i, base, protoid);
                        let rkc = self.get_rkc(i, base, protoid);
                        let a = get_arg_a(i) > 0;
                        if equal_obj(self, rkb, rkc) != a {
                            self.base_ci[self.ci].saved_pc += 1;
                        } else {
                            self.do_next_jump(protoid);
                        }
                        base = self.base_ci[self.ci].base as u32;
                    }
                    OpCode::Lt => {
                        let rkb = self.get_rkb(i, base, protoid);
                        let rkc = self.get_rkc(i, base, protoid);
                        let a = get_arg_a(i) > 0;
                        if less_than(self, rkb, rkc)? != a {
                            self.base_ci[self.ci].saved_pc += 1;
                        } else {
                            self.do_next_jump(protoid);
                        }
                        base = self.base_ci[self.ci].base as u32;
                    }
                    OpCode::Le => {
                        let rkb = self.get_rkb(i, base, protoid);
                        let rkc = self.get_rkc(i, base, protoid);
                        let a = get_arg_a(i) > 0;
                        if less_equal(self, rkb, rkc)? != a {
                            self.base_ci[self.ci].saved_pc += 1;
                        } else {
                            self.do_next_jump(protoid);
                        }
                        base = self.base_ci[self.ci].base as u32;
                    }
                    OpCode::Test => {
                        let is_false = get_arg_c(i) != 0;
                        if self.stack[ra as usize].is_false() == is_false {
                            self.base_ci[self.ci].saved_pc += 1;
                        } else {
                            self.do_next_jump(protoid);
                        }
                    }
                    OpCode::TestSet => {
                        let rb = get_rb(base, i);
                        let c = get_arg_c(i) > 0;
                        if self.stack[rb].is_false() == c {
                            self.base_ci[self.ci].saved_pc += 1;
                        } else {
                            self.stack[ra as usize] = self.stack[rb].clone();
                            self.do_next_jump(protoid);
                        }
                    }
                    OpCode::Call => {
                        let b = get_arg_b(i);
                        let nresults = get_arg_c(i) as i32 - 1;
                        if b != 0 {
                            self.stack.resize((ra + b) as usize, TValue::Nil); // top = ra+b
                        } // else previous instruction set top
                        match self.dprecall(ra as usize, nresults) {
                            Ok(PrecallStatus::Lua) => {
                                // restart luaV_execute over new Lua function
                                continue 'new_frame;
                            }
                            Ok(PrecallStatus::Rust) => {
                                // it was a Rust function (`precall' called it); adjust results
                                if nresults > 0 && self.stack.len() > self.base_ci[self.ci].top {
                                    self.stack.resize(self.base_ci[self.ci].top, TValue::Nil);
                                }
                                base = self.base_ci[self.ci].base as u32;
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
                        match self.dprecall(ra as usize, LUA_MULTRET) {
                            Ok(PrecallStatus::Lua) => {
                                // tail call: put new frame in place of previous one
                                let nbase = self.base_ci[self.ci].base; // called base
                                let nfunc = func; // called function
                                let obase = self.base_ci[self.ci - 1].base; // caller base
                                if !self.open_upval.is_empty() {
                                    // close all upvalues from previous call
                                    self.close_func(obase);
                                }
                                let nsaved_pc = self.base_ci[self.ci].saved_pc;
                                // caller function
                                let mut oci = &mut self.base_ci[self.ci - 1];
                                let ofunc = oci.func;
                                oci.base = ofunc + nbase - nfunc;
                                let mut aux = 0;
                                while nfunc + aux < self.stack.len() {
                                    // move new frame into old one
                                    self.stack[(ofunc + aux) as usize] =
                                        self.stack[(nfunc + aux) as usize].clone();
                                    aux += 1;
                                }
                                self.stack.resize((ofunc + aux) as usize, TValue::Nil);
                                oci.top = self.stack.len(); // correct top
                                oci.saved_pc = nsaved_pc;
                                oci.call_status |= CIST_TAIL; // function was tail called
                                self.base_ci.pop(); // remove new frame
                                self.ci -= 1;
                                continue 'new_frame;
                            }
                            Ok(PrecallStatus::Rust) => {
                                // it was a Rust function (`precall' called it); adjust results
                                base = self.base_ci[self.ci].base as u32;
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                    OpCode::Return => {
                        let b = get_arg_b(i);
                        if !self.protos[protoid].p.is_empty() {
                            self.close_func(base as StkId);
                        }
                        let was_fresh = self.base_ci[self.ci].call_status & CIST_FRESH != 0;
                        let b = self.poscall(
                            ra as usize,
                            if b != 0 {
                                b as usize - 1
                            } else {
                                self.stack.len() - ra as usize
                            },
                        );
                        if was_fresh {
                            // 'ci' is still the called one
                            return Ok(()); // external invocation : return
                        }
                        if b {
                            // invocation via reentry: continue execution
                            self.stack.resize(self.base_ci[self.ci].top, TValue::Nil);
                        }
                        debug_assert!(self.base_ci[self.ci].call_status & CIST_LUA != 0);
                        debug_assert!({
                            let protoid = self.get_lua_closure_protoid(self.base_ci[self.ci].func);
                            get_opcode(
                                self.get_instruction(protoid, self.base_ci[self.ci].saved_pc - 1),
                            ) == OpCode::Call
                        });
                        continue 'new_frame; // restart luaV_execute over new Lua function
                    }
                    OpCode::ForLoop => {
                        if self.stack[ra as usize].is_integer() {
                            let step = self.stack[ra as usize + 2].get_integer_value();
                            let idx = self.stack[ra as usize].get_integer_value() + step;
                            let limit = self.stack[ra as usize + 1].get_integer_value();
                            let end_loop = if step > 0 { idx <= limit } else { limit <= idx };
                            if end_loop {
                                // jump back
                                let jump = get_arg_sbx(i);
                                self.base_ci[self.ci].saved_pc =
                                    (self.base_ci[self.ci].saved_pc as i32 + jump) as usize;
                                self.set_stack_from_value(ra as usize, TValue::Integer(idx)); // update internal index
                                let rai = ra as usize + 3;
                                self.set_or_push(rai, TValue::Integer(idx)); // ...and external index
                            }
                        } else {
                            let step = self.stack[ra as usize + 2].into_float().unwrap();
                            let idx = self.stack[ra as usize].get_float_value() + step;
                            let limit = self.stack[ra as usize + 1].into_float().unwrap();
                            let end_loop = if step > 0.0 {
                                idx <= limit
                            } else {
                                limit <= idx
                            };
                            if end_loop {
                                // jump back
                                let jump = get_arg_sbx(i);
                                self.base_ci[self.ci].saved_pc =
                                    (self.base_ci[self.ci].saved_pc as i32 + jump) as usize;
                                self.set_stack_from_value(ra as usize, TValue::Float(idx)); // update internal index
                                let rai = ra as usize + 3;
                                self.set_or_push(rai, TValue::Float(idx)); // ...and external index
                            }
                        };
                    }
                    OpCode::ForPrep => {
                        let ra = ra as usize;
                        let ilimit = self.stack[ra + 1].into_integer();
                        if self.stack[ra].is_integer()
                            && self.stack[ra + 2].is_integer()
                            && ilimit.is_ok()
                        {
                            let ilimit = ilimit.unwrap();
                            let initv = self.stack[ra].get_integer_value();
                            self.set_stack_from_value(ra + 1, TValue::from(ilimit));
                            self.set_stack_from_value(
                                ra,
                                TValue::from(initv - self.stack[ra + 2].get_integer_value()),
                            );
                        } else {
                            if Self::to_number(&mut self.stack, ra as usize, Some(ra as usize))
                                .is_none()
                            {
                                return self.run_error("'for' initial value must be a number");
                            }
                            if Self::to_number(
                                &mut self.stack,
                                ra as usize + 1,
                                Some(ra as usize + 1),
                            )
                            .is_none()
                            {
                                return self.run_error("'for' limit must be a number");
                            }
                            if Self::to_number(
                                &mut self.stack,
                                ra as usize + 2,
                                Some(ra as usize + 2),
                            )
                            .is_none()
                            {
                                return self.run_error("'for' step must be a number");
                            }
                            // init = init - step
                            self.stack[ra as usize] = TValue::Float(
                                self.stack[ra as usize].get_float_value()
                                    - self.stack[ra as usize + 2].get_float_value(),
                            );
                        }
                        let jump = get_arg_sbx(i);
                        self.base_ci[self.ci].saved_pc =
                            (self.base_ci[self.ci].saved_pc as i32 + jump) as usize;
                    }
                    OpCode::TForCall => {
                        let cb = ra as usize + 3; // call base
                        self.set_stack_from_idx(cb, ra as StkId);
                        self.set_stack_from_idx(cb + 1, ra as StkId + 1);
                        self.set_stack_from_idx(cb + 2, ra as StkId + 2);
                        let nresults = get_arg_c(i);
                        self.dcall(cb, nresults as i32)?;
                        self.stack.resize(self.base_ci[self.ci].top, TValue::Nil);
                        let ci_pc = self.base_ci[self.ci].saved_pc;
                        let i = self.get_instruction(protoid, ci_pc);
                        self.base_ci[self.ci].saved_pc += 1;
                        let ra = get_ra(base, i) as u32;
                        debug_assert!(get_opcode(i) == OpCode::TForLoop);
                        if !self.stack[ra as usize + 1].is_nil() {
                            // continue loop ?
                            self.set_stack_from_idx(ra as usize, ra as usize + 1); // save control variable
                            let jump = get_arg_sbx(i);
                            self.base_ci[self.ci].saved_pc =
                                (self.base_ci[self.ci].saved_pc as i32 + jump) as usize;
                            // jump back
                        }
                    }
                    OpCode::TForLoop => {
                        if !self.stack[ra as usize].is_nil() {
                            // continue loop ?
                            self.stack[ra as usize] = self.stack[ra as usize + 1].clone();
                            let jump = get_arg_sbx(i);
                            self.base_ci[self.ci].saved_pc =
                                (self.base_ci[self.ci].saved_pc as i32 + jump) as usize;
                        }
                    }
                    OpCode::SetList => {
                        let mut n = get_arg_b(i);
                        let mut c = get_arg_c(i);
                        if n == 0 {
                            n = self.stack.len() as u32 - ra - 1;
                        }
                        if c == 0 {
                            let ci_pc = self.base_ci[self.ci].saved_pc;
                            debug_assert!(
                                get_opcode(self.get_instruction(protoid, ci_pc))
                                    == OpCode::ExtraArg
                            );
                            c = get_arg_ax(self.get_instruction(protoid, ci_pc));
                            self.base_ci[self.ci].saved_pc += 1;
                        }
                        let mut last = (c - 1) * LFIELDS_PER_FLUSH + n;
                        if let TValue::Table(tref) = &self.stack[ra as usize] {
                            let mut t = tref.borrow_mut();
                            while n > 0 {
                                t.set(
                                    TValue::Integer(last as LuaInteger),
                                    self.stack[(ra + n) as usize].clone(),
                                );
                                last -= 1;
                                n -= 1;
                            }
                        }
                        self.stack.resize(self.base_ci[self.ci].top, TValue::Nil);
                    }
                    OpCode::Closure => {
                        let pid = get_arg_bx(i);
                        let new_protoid = self.protos[protoid].p[pid as usize];
                        let nup = self.protos[new_protoid].upvalues.len();
                        let ncl =
                            Rc::new(RefCell::new(Closure::Lua(LClosure::new(new_protoid, nup))));
                        self.set_stack_from_value(ra as usize, TValue::Function(ncl.clone()));
                        let mut ncl = ncl.borrow_mut();
                        for i in 0..nup {
                            let upvaldesc = &self.protos[new_protoid].upvalues[i];
                            if upvaldesc.in_stack {
                                let upval = self.find_upval(func, base as usize + upvaldesc.idx);
                                ncl.set_lua_upvalue(i, upval);
                            } else {
                                ncl.set_lua_upvalue(
                                    i,
                                    self.get_lua_closure_upval(func, upvaldesc.idx).clone(),
                                );
                            }
                        }
                    }
                    OpCode::VarArg => {
                        let ra = ra as usize;
                        let mut b = get_arg_b(i) as i32 - 1;
                        let cbase = self.base_ci[self.ci].base as i32;
                        let n = (cbase - func as i32 - self.protos[protoid].numparams as i32 - 1)
                            .max(0);
                        if b < 0 {
                            b = n;
                            self.stack.resize(ra + n as usize, TValue::Nil);
                        }
                        for j in 0..b.min(n) as usize {
                            self.set_stack_from_idx(ra + j, (cbase + j as i32 - n) as usize);
                        }
                        for j in b.min(n)..b {
                            self.set_stack_from_value(ra + j as usize, TValue::Nil);
                        }
                    }
                    OpCode::ExtraArg => {
                        unreachable!()
                    }
                }
            }
        }
    }
    pub(crate) fn do_jump(&mut self, i: u32, e: i32) {
        let a = get_arg_a(i) as usize;
        if a > 0 {
            self.close_func(self.base_ci[self.ci].base + a - 1);
        }
        self.base_ci[self.ci].saved_pc =
            (self.base_ci[self.ci].saved_pc as i32 + get_arg_sbx(i) + e) as usize;
    }
    pub(crate) fn do_next_jump(&mut self, protoid: usize) {
        let ci_pc = self.base_ci[self.ci].saved_pc;
        let inst = self.get_instruction(protoid, ci_pc);
        self.do_jump(inst, 1)
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
        return Ok(rkb.into_float().unwrap() < rkc.into_float().unwrap());
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
        return Ok(rkb.into_float().unwrap() <= rkc.into_float().unwrap());
    } else if rkb.is_string() {
        return Ok(rkb.borrow_string_value() <= rkc.borrow_string_value());
    }
    // TODO metamethods
    luaG::order_error(state, &rkb, &rkc)?;
    unreachable!()
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
fn dump_function_header(state: &mut LuaState, func: usize) {
    let cl = state.get_closure_ref(func);
    let cl = cl.borrow();
    let cl = cl.borrow_lua_closure();
    let nup = cl.upvalues.len();
    let proto = &state.protos[cl.proto];
    let nk = proto.k.len();
    if proto.linedefined == proto.lastlinedefined {
        _ = writeln!(state.stdout, "; function [{}] ", proto.linedefined);
    } else {
        _ = writeln!(
            state.stdout,
            "; function [{}-{}] ",
            proto.linedefined, proto.lastlinedefined
        );
    }
    _ = writeln!(
        state.stdout,
        "; {} upvalues   {} params   {} stack   {}",
        nup,
        proto.numparams,
        proto.maxstacksize,
        if proto.is_vararg { "vararg" } else { "" }
    );
    for (i, loc) in proto.locvars.iter().enumerate() {
        _ = writeln!(
            state.stdout,
            ".local {:<10} ; {}",
            format!("\"{}\"", loc.name),
            i
        );
    }
    for i in 0..nk {
        if proto.k[i].is_string() {
            _ = writeln!(
                state.stdout,
                ".const {:<10} ; {}",
                &format!("{:?}", proto.k[i]),
                i
            );
        } else {
            _ = writeln!(
                state.stdout,
                ".const {:<10} ; {}",
                &format!("{}", proto.k[i]),
                i
            );
        }
    }
}

#[cfg(feature = "debug_logs")]
fn disassemble(state: &LuaState, i: Instruction, func: usize) -> String {
    use crate::opcodes::{get_arg_sb, get_arg_sc};

    let o = get_opcode(i);
    let a = get_arg_a(i);
    let b = get_arg_sb(i);
    let c = get_arg_sc(i);
    let ax = get_arg_ax(i);
    let sbx = get_arg_sbx(i);
    let bx = get_arg_bx(i);
    let cl = state.get_closure_ref(func);
    let cl = cl.borrow();
    let cl = cl.borrow_lua_closure();
    let proto = &state.protos[cl.proto];
    let mut res = if o.is_asbx() {
        format!("{:10} {:>5} {:>5}", OPCODE_NAME[o as usize], a, sbx)
    } else if o.is_ax() {
        format!("{:10} {:>5}", OPCODE_NAME[o as usize], ax)
    } else if o.is_abx() {
        format!("{:10} {:>5} {:>5}", OPCODE_NAME[o as usize], a, bx)
    } else if o.is_ab() {
        format!("{:10} {:>5} {:>5}", OPCODE_NAME[o as usize], a, b)
    } else if o.is_ac() {
        format!("{:10} {:>5} {:>5}", OPCODE_NAME[o as usize], a, c)
    } else {
        format!("{:10} {:>5} {:>5} {:>5}", OPCODE_NAME[o as usize], a, b, c)
    };
    match o {
        OpCode::LoadK => {
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

pub(crate) fn concat(state: &mut LuaState, total: usize) -> Result<(), LuaError> {
    let top = state.stack.len();
    if !(api::is_string(state, top as isize - 2) || api::is_number(state, top as isize - 2))
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

#[inline]
fn get_ra(base: u32, i: u32) -> usize {
    (base + get_arg_a(i)) as usize
}

#[inline]
fn get_rb(base: u32, i: u32) -> usize {
    (base + get_arg_b(i)) as usize
}
