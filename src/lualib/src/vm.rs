//! Lua virtual machine

use std::{cell::RefCell, rc::Rc};

use crate::{
    api::LuaError,
    debug::{for_error, order_error},
    luaD::PrecallStatus,
    luaG,
    object::{Closure, LClosure, TValue},
    opcodes::{
        get_arg_a, get_arg_ax, get_arg_b, get_arg_bx, get_arg_c, get_arg_k, get_arg_sb,
        get_arg_sbx, get_arg_sc, get_arg_sj, get_opcode, OpCode, MAXARG_C,
    },
    state::{LuaState, CIST_FRESH},
    LuaFloat, LuaInteger,
};

#[cfg(feature = "debug_logs")]
use crate::{limits::Instruction, opcodes::OPCODE_NAME};

pub(crate) const CLOSEKTOP: i32 = -1;

macro_rules! op_arith {
    ($s:expr, $ra: expr, $v1:expr, $v2:expr, $op: tt, $pc: expr) => {
        if let (TValue::Integer(iv1), TValue::Integer(iv2)) = ($v1, &$v2) {
            $pc += 1;
            $s.set_stack_from_value(
                $ra,TValue::Integer(iv1 $op iv2));
        } else if let (Some(f1), Some(f2)) = ($v1.into_float_ns(), $v2.into_float_ns()) {
            $pc += 1;
            $s.set_stack_from_value(
                $ra,TValue::Float(f1 $op f2));
        } else {
            () // need coercion. will be handled by MMBINx instruction
        }
    };
}

macro_rules! op_order {
    ($s:expr, $v1: expr, $v2:expr, $op: tt, $i:expr, $protoid: expr,$pc: expr) => {
        {
            let mut ok=true;
            let cond=if let (TValue::Integer(ia),TValue::Integer(ib)) = ($v1,$v2) {
                ia $op ib
            } else if let (TValue::Float(fa),TValue::Float(fb)) = ($v1,$v2) {
                fa $op fb
            } else if let (TValue::String(sa),TValue::String(sb)) = ($v1,$v2) {
                sa $op sb
            } else {
                // TODO metamethods
                ok=false;
                false
            };
            if ok {
                if cond != (get_arg_k($i)==1) {
                    *$pc += 1;
                } else {
                    $s.do_next_jump($protoid,$pc);
                }
            }
            ok
        }
    };
}

macro_rules! op_order_i {
    ($s:expr, $v1: expr, $v2:expr, $op: tt, $i:expr, $protoid: expr,$pc: expr) => {
        {
            let cond=if let TValue::Integer(ia) = $v1 {
                *ia $op $v2
            } else if let TValue::Float(fa) = $v1 {
                *fa $op $v2 as LuaFloat
            } else {
                // TODO metamethod
                todo!()
            };
            if cond != (get_arg_k($i)==1) {
                *$pc += 1;
            } else {
                $s.do_next_jump($protoid,$pc);
            }
        }
    };
}

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
        'new_frame: loop {
            let func = self.base_ci[self.ci].func;
            let protoid = self.get_lua_closure_protoid(func);
            let mut pc = self.base_ci[self.ci].saved_pc;
            #[cfg(feature = "debug_logs")]
            let mut first = true;
            // main loop of interpreter
            loop {
                let base = self.base_ci[self.ci].func as u32 + 1;
                let i = self.get_instruction(protoid, pc);
                #[cfg(feature = "debug_logs")]
                {
                    self.dump_debug_log(func, first, pc, i);
                    first = false;
                }
                pc += 1;
                // TODO handle hooks
                let ra = base + get_arg_a(i);
                debug_assert!(base <= self.stack.len() as u32);
                match get_opcode(i) {
                    OpCode::Move => {
                        let ra = get_ra(base, i);
                        let rb = get_rb(base, i);
                        self.set_stack_from_idx(ra, rb);
                    }
                    OpCode::LoadI => {
                        let ra = get_ra(base, i);
                        let b = get_arg_sbx(i) as LuaInteger;
                        self.set_stack_from_value(ra, TValue::Integer(b));
                    }
                    OpCode::LoadF => {
                        let ra = get_ra(base, i);
                        let b = get_arg_sbx(i) as LuaFloat;
                        self.set_stack_from_value(ra, TValue::Float(b));
                    }
                    OpCode::LoadK => {
                        let ra = get_ra(base, i);
                        let kid = get_arg_bx(i);
                        let kval = self.get_lua_constant(protoid, kid as usize);
                        self.set_stack_from_value(ra, kval);
                    }
                    OpCode::LoadKx => {
                        let ra = get_ra(base, i);
                        let i2 = self.get_instruction(protoid, pc);
                        let kid = get_arg_ax(i2);
                        pc += 1;
                        let kval = self.get_lua_constant(protoid, kid as usize);
                        self.set_stack_from_value(ra, kval);
                    }
                    OpCode::LoadFalse => {
                        let ra = get_ra(base, i);
                        self.set_stack_from_value(ra, TValue::Boolean(false));
                    }
                    OpCode::LoadFalseSkip => {
                        let ra = get_ra(base, i);
                        self.set_stack_from_value(ra, TValue::Boolean(false));
                        pc += 1; // skip next instruction
                    }
                    OpCode::LoadTrue => {
                        let ra = get_ra(base, i);
                        self.set_stack_from_value(ra, TValue::Boolean(true));
                    }
                    OpCode::LoadNil => {
                        let mut ra = get_ra(base, i);
                        let mut b = get_arg_b(i);
                        while b > 0 {
                            self.set_stack_from_value(ra, TValue::Nil);
                            ra += 1;
                            b -= 1;
                        }
                    }
                    OpCode::GetUpVal => {
                        let ra = get_ra(base, i);
                        let b = get_arg_b(i);
                        self.set_stack_from_value(
                            ra,
                            self.get_lua_closure_upvalue(func, b as usize),
                        );
                    }
                    OpCode::SetupVal => {
                        let ra = get_ra(base, i);
                        let b = get_arg_b(i);
                        self.set_lua_closure_upvalue(func, b as usize, self.stack[ra].clone());
                    }
                    OpCode::GetTabUp => {
                        let ra = get_ra(base, i);
                        let b = get_arg_b(i);
                        let key = self.get_kc(i, protoid);
                        let table = self.get_lua_closure_upvalue(func, b as usize);
                        // TODO metamethod
                        Self::get_tablev2(&mut self.stack, &table, &key, Some(ra));
                    }
                    OpCode::GetTable => {
                        let ra = get_ra(base, i);
                        let tableid = get_rb(base, i);
                        let rc = get_rc(base, i);
                        // TODO metamethod
                        self.get_table_value(tableid, rc, ra);
                    }
                    OpCode::GetI => {
                        let ra = get_ra(base, i);
                        let tableid: usize = get_rb(base, i);
                        let c = get_arg_c(i) as LuaInteger;
                        // TODO metamethod
                        self.get_table_value_by_key(tableid, &TValue::Integer(c), ra);
                    }
                    OpCode::GetField => {
                        let ra = get_ra(base, i);
                        let tableid = get_rb(base, i);
                        let key = self.get_kc(i, protoid);
                        // TODO metamethod
                        self.get_table_value_by_key(tableid, &key, ra);
                    }
                    OpCode::SetTabUp => {
                        let a = get_arg_a(i);
                        let key = self.get_kb(i, protoid);
                        let val = self.get_rkc(i, base, protoid);
                        let table = self.get_lua_closure_upvalue(func, a as usize);
                        // TODO metamethod
                        self.set_tablev(&table, key, val);
                    }
                    OpCode::SetTable => {
                        let ra = get_ra(base, i);
                        let rb = get_rb(base, i);
                        let key = self.stack[rb].clone();
                        let value = self.get_rkc(i, base, protoid);
                        // TODO metamethod
                        self.set_tablev(&self.stack[ra], key, value);
                    }
                    OpCode::SetI => {
                        let ra = get_ra(base, i);
                        let c = get_arg_c(i) as LuaInteger;
                        let rkc = self.get_rkc(i, base, protoid);
                        // TODO metamethod
                        self.set_tablev(&self.stack[ra], TValue::Integer(c), rkc);
                    }
                    OpCode::SetField => {
                        let ra = get_ra(base, i);
                        let rb = self.get_kb(i, protoid);
                        let rkc = self.get_rkc(i, base, protoid);
                        // TODO metamethod
                        self.set_tablev(&self.stack[ra], rb, rkc);
                    }
                    OpCode::NewTable => {
                        let ra = get_ra(base, i);
                        self.set_stack_from_value(ra, TValue::new_table());
                    }
                    OpCode::OpSelf => {
                        let ra = get_ra(base, i);
                        let rb = get_rb(base, i);
                        let rc = self.get_rkc(i, base, protoid);
                        self.set_stack_from_idx(ra + 1, rb);
                        // TODO metamethod
                        Self::get_tablev(&mut self.stack, rb, &rc, Some(ra));
                    }
                    OpCode::AddI => {
                        let ra = get_ra(base, i);
                        let v1 = get_rb(base, i);
                        let imm = get_arg_sc(i) as LuaInteger;
                        pc += 1;
                        self.set_stack_from_value(
                            ra,
                            match &self.stack[v1] {
                                TValue::Integer(iv1) => TValue::Integer(iv1 + imm),
                                TValue::Float(fv1) => TValue::Float(fv1 + imm as LuaFloat),
                                _ => unreachable!(),
                            },
                        );
                    }
                    OpCode::AddK => {
                        let v1 = &self.stack[get_rb(base, i)];
                        let v2 = self.get_kc(i, protoid);
                        debug_assert!(v2.is_number());
                        let ra = get_ra(base, i);
                        op_arith!(self, ra, v1, v2, +, pc);
                    }
                    OpCode::SubK => {
                        let v1 = &self.stack[get_rb(base, i)];
                        let v2 = self.get_kc(i, protoid);
                        debug_assert!(v2.is_number());
                        let ra = get_ra(base, i);
                        op_arith!(self, ra, v1, v2, -, pc);
                    }
                    OpCode::MulK => {
                        let v1 = &self.stack[get_rb(base, i)];
                        let v2 = self.get_kc(i, protoid);
                        debug_assert!(v2.is_number());
                        let ra = get_ra(base, i);
                        op_arith!(self, ra, v1, v2, *, pc);
                    }
                    OpCode::ModK => {
                        self.save_state(pc); // in case of division by 0
                        let v2 = self.get_kc(i, protoid);
                        debug_assert!(v2.is_number());
                        if v2.is_integer() && v2.get_integer_value() == 0 {
                            self.run_error("attempt to perform 'n%0'")?;
                        }
                        let v1 = &self.stack[get_rb(base, i)];
                        let ra = get_ra(base, i);
                        pc += 1;
                        let value = if let (TValue::Integer(iv1), TValue::Integer(iv2)) = (v1, &v2)
                        {
                            TValue::Integer(iv1 % iv2)
                        } else if let (Some(f1), Some(f2)) =
                            (v1.into_float_ns(), v2.into_float_ns())
                        {
                            TValue::Float(f1 % f2)
                        } else {
                            unreachable!()
                        };
                        self.set_stack_from_value(ra, value);
                    }
                    OpCode::PowK => {
                        let v1 = &self.stack[get_rb(base, i)];
                        let v2 = self.get_kc(i, protoid);
                        debug_assert!(v2.is_number());
                        let ra = get_ra(base, i);
                        pc += 1;
                        let value = if let (Some(f1), Some(f2)) =
                            (v1.into_float_ns(), v2.into_float_ns())
                        {
                            TValue::Float(f1.powf(f2))
                        } else {
                            unreachable!()
                        };
                        self.set_stack_from_value(ra, value);
                    }
                    OpCode::DivK => {
                        let v1 = &self.stack[get_rb(base, i)];
                        let v2 = self.get_kc(i, protoid);
                        debug_assert!(v2.is_number());
                        let ra = get_ra(base, i);
                        pc += 1;
                        let value = if let (Some(f1), Some(f2)) =
                            (v1.into_float_ns(), v2.into_float_ns())
                        {
                            TValue::Float(f1 / f2)
                        } else {
                            unreachable!()
                        };
                        self.set_stack_from_value(ra, value);
                    }
                    OpCode::IntegerDivK => {
                        self.save_state(pc); // in case of division by 0
                        let v2 = self.get_kc(i, protoid);
                        if v2.is_integer() && v2.get_integer_value() == 0 {
                            self.run_error("attempt to divide by zero")?;
                        }
                        let v1 = &self.stack[get_rb(base, i)];
                        debug_assert!(v2.is_number());
                        let ra = get_ra(base, i);
                        pc += 1;
                        let value = if let (TValue::Integer(iv1), TValue::Integer(iv2)) = (v1, &v2)
                        {
                            TValue::Integer(iv1 / iv2)
                        } else if let (Some(f1), Some(f2)) =
                            (v1.into_float_ns(), v2.into_float_ns())
                        {
                            TValue::Float((f1 / f2).floor())
                        } else {
                            unreachable!()
                        };
                        self.set_stack_from_value(ra, value);
                    }
                    OpCode::BinaryAndK => {
                        let ra = get_ra(base, i);
                        let v1 = &self.stack[get_rb(base, i)];
                        let v2 = self.get_kc(i, protoid);
                        let i2 = v2.get_integer_value();
                        if let Some(i1) = v1.into_integer_ns() {
                            pc += 1;
                            self.set_stack_from_value(ra, TValue::Integer(i1 & i2));
                        }
                    }
                    OpCode::BinaryOrK => {
                        let ra = get_ra(base, i);
                        let v1 = &self.stack[get_rb(base, i)];
                        let v2 = self.get_kc(i, protoid);
                        let i2 = v2.get_integer_value();
                        if let Some(i1) = v1.into_integer_ns() {
                            pc += 1;
                            self.set_stack_from_value(ra, TValue::Integer(i1 | i2));
                        }
                    }
                    OpCode::BinaryXorK => {
                        let ra = get_ra(base, i);
                        let v1 = &self.stack[get_rb(base, i)];
                        let v2 = self.get_kc(i, protoid);
                        let i2 = v2.get_integer_value();
                        if let Some(i1) = v1.into_integer_ns() {
                            pc += 1;
                            self.set_stack_from_value(ra, TValue::Integer(i1 ^ i2));
                        }
                    }
                    OpCode::ShrI => {
                        let ra = get_ra(base, i);
                        let v1 = &self.stack[get_rb(base, i)];
                        let v2 = get_arg_sc(i);
                        if let Some(i1) = v1.into_integer_ns() {
                            pc += 1;
                            self.set_stack_from_value(ra, TValue::Integer(i1 >> v2));
                        }
                    }
                    OpCode::ShlI => {
                        let ra = get_ra(base, i);
                        let v1 = &self.stack[get_rb(base, i)];
                        let v2 = get_arg_sc(i);
                        if let Some(i1) = v1.into_integer_ns() {
                            pc += 1;
                            self.set_stack_from_value(ra, TValue::Integer(i1 << v2));
                        }
                    }
                    OpCode::Add => {
                        let ra = get_ra(base, i);
                        let rb = get_rb(base, i);
                        let rc = get_rc(base, i);
                        op_arith!(self, ra, &self.stack[rb],&self.stack[rc], +, pc);
                    }
                    OpCode::Sub => {
                        let ra = get_ra(base, i);
                        let rb = get_rb(base, i);
                        let rc = get_rc(base, i);
                        op_arith!(self, ra, &self.stack[rb],&self.stack[rc], -, pc);
                    }
                    OpCode::Mul => {
                        let ra = get_ra(base, i);
                        let rb = get_rb(base, i);
                        let rc = get_rc(base, i);
                        op_arith!(self, ra, &self.stack[rb],&self.stack[rc], *, pc);
                    }
                    OpCode::Mod => {
                        self.save_state(pc); // in case of division by 0
                        let ra = get_ra(base, i);
                        let rb = get_rb(base, i);
                        let rc = get_rc(base, i);
                        if self.stack[rc].is_integer() && self.stack[rc].get_integer_value() == 0 {
                            self.run_error("attempt to perform 'n%0'")?;
                        }
                        let v1 = &self.stack[rb];
                        let v2 = &self.stack[rc];
                        pc += 1;
                        let value = if let (TValue::Integer(iv1), TValue::Integer(iv2)) = (v1, &v2)
                        {
                            TValue::Integer(iv1 % iv2)
                        } else if let (Some(f1), Some(f2)) =
                            (v1.into_float_ns(), v2.into_float_ns())
                        {
                            TValue::Float(f1 % f2)
                        } else {
                            unreachable!()
                        };
                        self.set_stack_from_value(ra, value);
                    }
                    OpCode::Pow => {
                        let ra = get_ra(base, i);
                        let rb = get_rb(base, i);
                        let rc = get_rc(base, i);
                        let v1 = &self.stack[rb];
                        let v2 = &self.stack[rc];
                        pc += 1;
                        let value = if let (Some(f1), Some(f2)) =
                            (v1.into_float_ns(), v2.into_float_ns())
                        {
                            TValue::Float(f1.powf(f2))
                        } else {
                            unreachable!()
                        };
                        self.set_stack_from_value(ra, value);
                    }
                    OpCode::Div => {
                        let ra = get_ra(base, i);
                        let rb = get_rb(base, i);
                        let rc = get_rc(base, i);
                        let v1 = &self.stack[rb];
                        let v2 = &self.stack[rc];
                        pc += 1;
                        let value = if let (Some(f1), Some(f2)) =
                            (v1.into_float_ns(), v2.into_float_ns())
                        {
                            TValue::Float(f1 / f2)
                        } else {
                            unreachable!()
                        };
                        self.set_stack_from_value(ra, value);
                    }
                    OpCode::IntegerDiv => {
                        self.save_state(pc); // in case of division by 0
                        let ra = get_ra(base, i);
                        let rb = get_rb(base, i);
                        let rc = get_rc(base, i);
                        if self.stack[rc].is_integer() && self.stack[rc].get_integer_value() == 0 {
                            self.run_error("attempt to divide by zero")?;
                        }
                        let v1 = &self.stack[rb];
                        let v2 = &self.stack[rc];
                        pc += 1;
                        let value = if let (TValue::Integer(iv1), TValue::Integer(iv2)) = (v1, &v2)
                        {
                            TValue::Integer(iv1 / iv2)
                        } else if let (Some(f1), Some(f2)) =
                            (v1.into_float_ns(), v2.into_float_ns())
                        {
                            TValue::Float((f1 / f2).floor())
                        } else {
                            unreachable!()
                        };
                        self.set_stack_from_value(ra, value);
                    }
                    OpCode::BinaryAnd => {
                        let ra = get_ra(base, i);
                        let v1 = &self.stack[get_rb(base, i)];
                        let v2 = &self.stack[get_rc(base, i)];
                        if let (Some(i1), Some(i2)) = (v1.into_integer_ns(), v2.into_integer_ns()) {
                            pc += 1;
                            self.set_stack_from_value(ra, TValue::Integer(i1 & i2));
                        }
                    }
                    OpCode::BinaryOr => {
                        let ra = get_ra(base, i);
                        let v1 = &self.stack[get_rb(base, i)];
                        let v2 = &self.stack[get_rc(base, i)];
                        if let (Some(i1), Some(i2)) = (v1.into_integer_ns(), v2.into_integer_ns()) {
                            pc += 1;
                            self.set_stack_from_value(ra, TValue::Integer(i1 | i2));
                        }
                    }
                    OpCode::BinaryXor => {
                        let ra = get_ra(base, i);
                        let v1 = &self.stack[get_rb(base, i)];
                        let v2 = &self.stack[get_rc(base, i)];
                        if let (Some(i1), Some(i2)) = (v1.into_integer_ns(), v2.into_integer_ns()) {
                            pc += 1;
                            self.set_stack_from_value(ra, TValue::Integer(i1 ^ i2));
                        }
                    }
                    OpCode::Shr => {
                        let ra = get_ra(base, i);
                        let v1 = &self.stack[get_rb(base, i)];
                        let v2 = &self.stack[get_rc(base, i)];
                        if let (Some(i1), Some(i2)) = (v1.into_integer_ns(), v2.into_integer_ns()) {
                            pc += 1;
                            self.set_stack_from_value(ra, TValue::Integer(i1 >> i2));
                        }
                    }
                    OpCode::Shl => {
                        let ra = get_ra(base, i);
                        let v1 = &self.stack[get_rb(base, i)];
                        let v2 = &self.stack[get_rc(base, i)];
                        if let (Some(i1), Some(i2)) = (v1.into_integer_ns(), v2.into_integer_ns()) {
                            pc += 1;
                            self.set_stack_from_value(ra, TValue::Integer(i1 << i2));
                        }
                    }
                    OpCode::MMBin => {
                        todo!()
                    }
                    OpCode::MMBinI => {
                        todo!()
                    }
                    OpCode::MMBinK => {
                        todo!()
                    }
                    OpCode::UnaryMinus => {
                        let ra = get_ra(base, i);
                        let vrb = &self.stack[get_rb(base, i)];
                        if let TValue::Integer(irb) = vrb {
                            self.set_stack_from_value(ra, TValue::Integer(-irb));
                        } else if let Some(frb) = vrb.into_float_ns() {
                            self.set_stack_from_value(ra, TValue::Float(-frb));
                        } else {
                            // metamethod
                            todo!()
                        }
                    }
                    OpCode::BinaryNot => {
                        let ra = get_ra(base, i);
                        let vrb = &self.stack[get_rb(base, i)];
                        if let Some(irb) = vrb.into_integer_ns() {
                            self.set_stack_from_value(ra, TValue::Integer(!irb));
                        } else {
                            // metamethod
                            todo!()
                        }
                    }
                    OpCode::Not => {
                        let ra = get_ra(base, i);
                        let vrb = &self.stack[get_rb(base, i)];
                        self.set_stack_from_value(ra, TValue::Boolean(vrb.is_false()));
                    }
                    OpCode::Len => {
                        let ra = get_ra(base, i);
                        let rb = get_rb(base, i);
                        if let Some(l) = self.stack[rb].try_len() {
                            self.set_stack_from_value(ra, TValue::Integer(l as LuaInteger));
                        } else {
                            // TODO metamethod
                            todo!()
                        }
                    }
                    OpCode::Concat => {
                        let ra = get_ra(base, i);
                        let n = get_arg_b(i) as usize; // number of elements to concatenate
                        self.stack.resize(ra + n, TValue::Nil); // mark the end of concat operands
                        self.base_ci[self.ci].saved_pc = pc;
                        concat(self, n)?;
                    }
                    OpCode::Close => {
                        let ra = get_ra(base, i);
                        self.save_state(pc);
                        f_close(self, ra, 0, true)?;
                    }
                    OpCode::ToBeClosed => {
                        todo!()
                    }
                    OpCode::Jmp => {
                        pc = (pc as i32 + get_arg_sj(i)) as usize;
                    }
                    OpCode::Eq => {
                        let ra = get_ra(base, i);
                        self.save_state(pc);
                        let vrb = &self.stack[get_rb(base, i)];
                        let cond = self.stack[ra] == *vrb;
                        if cond != (get_arg_k(i) == 1) {
                            pc += 1;
                        } else {
                            self.do_next_jump(protoid, &mut pc);
                        }
                    }
                    OpCode::Lt => {
                        let ra = get_ra(base, i);
                        let rb = get_rb(base, i);
                        let ok = {
                            let vra = &self.stack[ra];
                            let vrb = &self.stack[rb];
                            op_order!(self,vra,vrb,<,i,protoid,&mut pc)
                        };
                        if !ok {
                            order_error(self, ra, rb)?;
                        }
                    }
                    OpCode::Le => {
                        let ra = get_ra(base, i);
                        let rb = get_rb(base, i);
                        let ok = {
                            let vra = &self.stack[ra];
                            let vrb = &self.stack[rb];
                            op_order!(self,vra,vrb,<=,i,protoid,&mut pc)
                        };
                        if !ok {
                            order_error(self, ra, rb)?;
                        }
                    }
                    OpCode::EqK => {
                        let vra = &self.stack[get_ra(base, i)];
                        let vrb = &self.stack[get_rb(base, i)];
                        let cond = vra == vrb;
                        if cond != (get_arg_k(i) == 1) {
                            pc += 1;
                        } else {
                            self.do_next_jump(protoid, &mut pc);
                        }
                    }
                    OpCode::EqI => {
                        let vra = &self.stack[get_ra(base, i)];
                        let im = get_arg_sb(i) as LuaInteger;
                        let cond = match vra {
                            TValue::Integer(ia) => *ia == im,
                            TValue::Float(fa) => *fa == im as LuaFloat,
                            _ => false,
                        };
                        if cond != (get_arg_k(i) == 1) {
                            pc += 1;
                        } else {
                            self.do_next_jump(protoid, &mut pc);
                        }
                    }
                    OpCode::LtI => {
                        let vra = &self.stack[get_ra(base, i)];
                        let im = get_arg_sb(i) as LuaInteger;
                        op_order_i!(self,vra,im,<,i,protoid, &mut pc);
                    }
                    OpCode::LeI => {
                        let vra = &self.stack[get_ra(base, i)];
                        let im = get_arg_sb(i) as LuaInteger;
                        op_order_i!(self,vra,im,<=,i,protoid, &mut pc);
                    }
                    OpCode::GtI => {
                        let vra = &self.stack[get_ra(base, i)];
                        let im = get_arg_sb(i) as LuaInteger;
                        op_order_i!(self,vra,im,>,i,protoid, &mut pc);
                    }
                    OpCode::GeI => {
                        let vra = &self.stack[get_ra(base, i)];
                        let im = get_arg_sb(i) as LuaInteger;
                        op_order_i!(self,vra,im,>=,i,protoid, &mut pc);
                    }
                    OpCode::Test => {
                        let vra = &self.stack[get_ra(base, i)];
                        let cond = !vra.is_false();
                        if cond != (get_arg_k(i) == 1) {
                            pc += 1;
                        } else {
                            self.do_next_jump(protoid, &mut pc);
                        }
                    }
                    OpCode::TestSet => {
                        let vrb = &self.stack[get_rb(base, i)];
                        if vrb.is_false() == (get_arg_k(i) == 1) {
                            pc += 1;
                        } else {
                            self.stack[get_ra(base, i)] = vrb.clone();
                            self.do_next_jump(protoid, &mut pc);
                        }
                    }
                    OpCode::Call => {
                        let ra = get_ra(base, i);
                        let b = get_arg_b(i) as usize;
                        let nresults = get_arg_c(i) as i32 - 1;
                        if b != 0 {
                            // fixed number of arguments?
                            self.stack.resize(ra + b, TValue::Nil);
                            // top = ra+b
                        } // else previous instruction set top
                        self.base_ci[self.ci].saved_pc = pc;
                        match self.dprecall(ra, nresults) {
                            Ok(PrecallStatus::Lua) => {
                                // restart luaV_execute over new Lua function
                                continue 'new_frame;
                            }
                            Ok(PrecallStatus::Rust) => {
                                // it was a Rust function (`precall' called it); adjust results
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                    OpCode::TailCall => {
                        let ra = get_ra(base, i);
                        let mut b = get_arg_b(i) as usize; // number of arguments + 1 (function)
                        let nparams1 = get_arg_c(i) as usize;
                        // delta is virtual 'func' - real 'func' (vararg functions)
                        let delta = if nparams1 != 0 {
                            self.base_ci[self.ci].n_extra_args + nparams1
                        } else {
                            0
                        };
                        if b != 0 {
                            self.stack.resize(ra + b, TValue::Nil);
                        // top = ra+b
                        } else {
                            // else previous instruction set top
                            b = self.stack.len() - ra;
                        }
                        self.base_ci[self.ci].saved_pc = pc;
                        if get_arg_k(i) != 0 {
                            close_upval(self, base as usize);
                            debug_assert!(*self.tbc_list.last().unwrap() < base as usize);
                            debug_assert!(base == self.base_ci[self.ci].func as u32 + 1);
                        }
                        let mut nresults = 0;
                        match self.dpre_tailcall(ra, b, delta, &mut nresults) {
                            Ok(PrecallStatus::Lua) => {
                                // execute the callee
                                continue 'new_frame;
                            }
                            Ok(PrecallStatus::Rust) => {
                                // it was a Rust function (`precall' called it); adjust results
                                self.base_ci[self.ci].func -= delta; // restore func (if vararg)
                                self.poscall(nresults)?; // finish caller
                                if self.base_ci[self.ci].call_status & CIST_FRESH != 0 {
                                    // end this frame
                                    return Ok(());
                                } else {
                                    // continue running caller in this frame
                                    continue 'new_frame;
                                }
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }
                    OpCode::Return => {
                        let ra = get_ra(base, i);
                        let mut n = get_arg_b(i) as i32 - 1; // number of results
                        let nparams1 = get_arg_c(i);
                        if n < 0 {
                            // not fixed ?
                            n = (self.stack.len() - ra) as i32; // get what is available
                        }
                        self.base_ci[self.ci].saved_pc = pc;
                        if get_arg_k(i) != 0 {
                            // may there be open values ?
                            self.base_ci[self.ci].nresults = n; // save number of results
                            self.stack.resize(self.base_ci[self.ci].top, TValue::Nil);
                            f_close(self, base as usize, CLOSEKTOP, true)?;
                            // TODO trap
                        }
                        if nparams1 != 0 {
                            // vararg function ?
                            self.base_ci[self.ci].func -=
                                self.base_ci[self.ci].n_extra_args + nparams1 as usize;
                        }
                        self.stack.resize(ra + n as usize, TValue::Nil);
                        let call_status = self.base_ci[self.ci].call_status;
                        self.poscall(n)?;
                        if call_status & CIST_FRESH != 0 {
                            // end this frame
                            return Ok(());
                        } else {
                            // continue running caller in this frame
                            continue 'new_frame;
                        }
                    }
                    OpCode::Return0 => {
                        // TODO hooks
                        let nresults = self.base_ci[self.ci].nresults;
                        let call_status = self.base_ci[self.ci].call_status;
                        self.base_ci.pop();
                        self.ci -= 1;
                        self.stack.resize(base as usize - 1, TValue::Nil);
                        for _ in 0..nresults {
                            self.stack.push(TValue::Nil); // all results are nil
                        }
                        if call_status & CIST_FRESH != 0 {
                            // end this frame
                            return Ok(());
                        } else {
                            // continue running caller in this frame
                            continue 'new_frame;
                        }
                    }
                    OpCode::Return1 => {
                        // TODO hooks
                        let nres = self.base_ci[self.ci].nresults;
                        let call_status = self.base_ci[self.ci].call_status;
                        self.base_ci.pop();
                        self.ci -= 1;
                        if nres == 0 {
                            self.stack.resize(base as usize - 1, TValue::Nil);
                        } else {
                            let ra = get_ra(base, i);
                            self.set_stack_from_idx(base as usize - 1, ra); // at least this result
                            self.stack.resize(base as usize, TValue::Nil);
                            for _ in 0..nres - 1 {
                                self.stack.push(TValue::Nil); // complete missing results
                            }
                        }
                        if call_status & CIST_FRESH != 0 {
                            // end this frame
                            return Ok(());
                        } else {
                            // continue running caller in this frame
                            continue 'new_frame;
                        }
                    }
                    OpCode::ForLoop => {
                        let ra = get_ra(base, i);
                        if self.stack[ra + 2].is_integer() {
                            // integer loop?
                            let count = self.stack[ra + 1].get_integer_value();
                            if count > 0 {
                                let step = self.stack[ra + 2].get_integer_value();
                                let mut idx = self.stack[ra].get_integer_value(); // internal index
                                self.set_stack_from_value(ra + 1, TValue::Integer(count - 1)); // update counter
                                idx += step; // add step to index
                                self.set_stack_from_value(ra, TValue::Integer(idx)); // update internal index
                                self.set_stack_from_value(ra + 3, TValue::Integer(idx)); // and control variable
                                pc = (pc as isize - get_arg_bx(i) as isize) as usize;
                                //  jump back
                            }
                        } else if self.float_for_loop(ra) {
                            pc = (pc as isize - get_arg_bx(i) as isize) as usize;
                            //  jump back
                        }
                    }
                    OpCode::ForPrep => {
                        let ra = get_ra(base, i);
                        self.save_state(pc);
                        if self.for_prep(ra)? {
                            pc = (pc as isize + get_arg_bx(i) as isize + 1) as usize;
                            // skip the loop
                        }
                    }
                    OpCode::TForPrep => {
                        let ra = get_ra(base, i);
                        self.save_state(pc);
                        // create to-be-closed upvalue (if needed)
                        self.new_tbc_value(ra + 3);
                        pc = (pc as isize + get_arg_bx(i) as isize) as usize;
                        let i = self.get_instruction(protoid, pc);
                        pc += 1;
                        debug_assert!(get_opcode(i) == OpCode::TForCall && ra == get_ra(base, i));
                        self.tforcall(&mut pc, protoid, base, i)?;
                    }
                    OpCode::TForCall => {
                        self.tforcall(&mut pc, protoid, base, i)?;
                    }
                    OpCode::TForLoop => {
                        self.tforloop(&mut pc, base, i);
                    }
                    OpCode::SetList => {
                        let ra = get_ra(base, i);
                        let mut n = get_arg_b(i);
                        let mut last = get_arg_c(i);
                        if n == 0 {
                            n = (self.stack.len() - ra - 1) as u32; // get up to the top
                        } else {
                            self.stack.resize(self.base_ci[self.ci].top, TValue::Nil);
                        }
                        last += n;
                        if get_arg_k(i) != 0 {
                            let i2 = self.get_instruction(protoid, pc);
                            last += get_arg_ax(i2) * (MAXARG_C as u32 + 1);
                            pc += 1;
                        }
                        if let TValue::Table(tref) = &self.stack[ra] {
                            let mut t = tref.borrow_mut();
                            while n > 0 {
                                t.set(
                                    TValue::Integer(last as LuaInteger),
                                    self.stack[ra + n as usize].clone(),
                                );
                                last -= 1;
                                n -= 1;
                            }
                        }
                    }
                    OpCode::Closure => {
                        let ra = get_ra(base, i);
                        let pid = get_arg_bx(i);
                        let new_protoid = self.protos[protoid].p[pid as usize];
                        self.save_state(pc);

                        let nup = self.protos[new_protoid].upvalues.len();
                        let ncl =
                            Rc::new(RefCell::new(Closure::Lua(LClosure::new(new_protoid, nup))));
                        self.set_stack_from_value(ra, TValue::Function(ncl.clone()));
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
                        let mut wanted = get_arg_c(i) as i32 - 1; // required results
                        self.save_state(pc);
                        let nextra = self.base_ci[self.ci].n_extra_args as i32;
                        if wanted < 0 {
                            wanted = nextra; // get all extra arguments available
                            self.stack.resize(ra + nextra as usize, TValue::Nil);
                        }
                        let func = self.base_ci[self.ci].func as i32;
                        for i in 0..wanted.min(nextra) as usize {
                            self.set_stack_from_idx(ra + i, (func - nextra + i as i32) as usize);
                        }
                        for i in wanted.min(nextra)..wanted {
                            self.set_stack_from_value(ra + i as usize, TValue::Nil);
                        }
                    }
                    OpCode::VarArgPrep => {
                        self.save_state(pc);
                        let nfixparams = get_arg_a(i) as usize;
                        let func = self.base_ci[self.ci].func;
                        let actual = self.stack.len() - func - 1; // number of arguments
                        let nextra = actual - nfixparams; // number of extra arguments
                        self.base_ci[self.ci].n_extra_args = nextra;
                        let top = self.stack.len();
                        let max_stack_size = top + self.protos[protoid].maxstacksize;
                        self.stack.resize(max_stack_size + 1, TValue::Nil);
                        // copy function to the top of the stack
                        self.set_stack_from_idx(top, func);
                        // move fixed parameters to the top of the stack
                        for i in 1..=nfixparams {
                            self.set_stack_from_idx(top + i, func + i);
                        }
                        self.base_ci[self.ci].func += actual + 1;
                        self.base_ci[self.ci].top += actual + 1;
                        debug_assert!(self.stack.len() <= self.base_ci[self.ci].top);
                    }
                    OpCode::ExtraArg => {
                        unreachable!()
                    }
                }
            }
        }
    }

    /// Whenever code can raise errors, the global 'pc' and the global
    /// 'top' must be correct to report occasional errors.
    fn save_state(&mut self, pc: usize) {
        self.base_ci[self.ci].saved_pc = pc;
        self.stack.resize(self.base_ci[self.ci].top, TValue::Nil);
    }
    pub(crate) fn do_next_jump(&mut self, protoid: usize, pc: &mut usize) {
        let inst = self.get_instruction(protoid, *pc);
        self.do_jump(inst, pc, 1)
    }
    pub(crate) fn do_jump(&mut self, i: u32, pc: &mut usize, delta: usize) {
        *pc = (*pc as isize + get_arg_sj(i) as isize + delta as isize) as usize;
    }
    /// Execute a step of a float numerical for loop, returning
    /// true iff the loop must continue. (The integer case is
    /// written online with opcode OP_FORLOOP, for performance.)
    fn float_for_loop(&mut self, ra: usize) -> bool {
        let step = self.stack[ra + 2].get_float_value();
        let limit = self.stack[ra + 1].get_float_value();
        let idx = self.stack[ra].get_float_value();
        let idx = idx + step;
        let continue_loop = if step > 0.0 {
            idx <= limit
        } else {
            limit <= idx
        };
        if continue_loop {
            self.set_stack_from_value(ra, TValue::Float(idx)); // update internal index
            self.set_stack_from_value(ra + 3, TValue::Float(idx)); // and control variable
            return true;
        }
        false
    }

    /// Prepare a numerical for loop (opcode OP_FORPREP).
    /// Return true to skip the loop. Otherwise,
    /// after preparation, stack will be as follows:
    ///   ra : internal index (safe copy of the control variable)
    ///   ra + 1 : loop counter (integer loops) or limit (float loops)
    ///   ra + 2 : step
    ///   ra + 3 : control variable
    pub(crate) fn for_prep(&mut self, ra: usize) -> Result<bool, LuaError> {
        if self.stack[ra].is_integer() && self.stack[ra + 2].is_integer() {
            let init = self.stack[ra].get_integer_value();
            let step = self.stack[ra + 2].get_integer_value();
            let mut limit = 0;
            if step == 0 {
                self.run_error("'for' step is zero")?;
            }
            self.set_stack_from_value(ra + 3, TValue::Integer(init)); // control variable
            if self.for_limit(init, ra + 1, &mut limit, step)? {
                return Ok(true); // skip the loop
            } else {
                let mut count;
                // prepare loop counter
                if step > 0 {
                    // ascending loop
                    count = limit - init;
                    if step != 1 {
                        // avoid division in the too common case
                        count /= step;
                    }
                } else {
                    // step < 0; descending loop
                    count = init - limit;
                    // step+1 avoids negating mininteger
                    count /= (-(step + 1)) + 1;
                }
                // store the counter in place of the limit (which won't be
                // needed anymore)
                self.set_stack_from_value(ra + 1, TValue::Integer(count));
            }
        } else {
            // try making all values floats
            let limit = self.stack[ra + 1]
                .into_float()
                .ok_or(for_error(self, ra + 1, "limit"))?;
            let step = self.stack[ra + 2]
                .into_float()
                .ok_or(for_error(self, ra + 2, "step"))?;
            let init = self.stack[ra]
                .into_float()
                .ok_or(for_error(self, ra, "initial value"))?;
            if step == 0.0 {
                self.run_error("'for' step is zero")?;
            }
            let skip_loop = if step > 0.0 {
                limit < init
            } else {
                init < limit
            };
            if skip_loop {
                return Ok(true); // skip the loop
            }
            // make sure all internal values are floats
            self.set_stack_from_value(ra + 1, TValue::Float(limit));
            self.set_stack_from_value(ra + 2, TValue::Float(step));
            self.set_stack_from_value(ra, TValue::Float(init)); // internal index
            self.set_stack_from_value(ra + 3, TValue::Float(init)); // control variable
        }
        Ok(false)
    }

    /// Try to convert a 'for' limit to an integer, preserving the semantics
    /// of the loop. Return true if the loop must not run; otherwise, 'limit'
    /// gets the integer limit.
    /// (The following explanation assumes a positive step; it is valid for
    /// negative steps mutatis mutandis.)
    /// If the limit is an integer or can be converted to an integer,
    /// rounding down, that is the limit.
    /// Otherwise, check whether the limit can be converted to a float. If
    /// the float is too large, clip it to LUA_MAXINTEGER.  If the float
    /// is too negative, the loop should not run, because any initial
    /// integer value is greater than such limit; so, the function returns
    /// true to signal that. (For this latter case, no integer limit would be
    /// correct; even a limit of LUA_MININTEGER would run the loop once for
    /// an initial value equal to LUA_MININTEGER.)
    pub(crate) fn for_limit(
        &mut self,
        init: LuaInteger,
        ra: usize,
        limit: &mut LuaInteger,
        step: LuaInteger,
    ) -> Result<bool, LuaError> {
        if let Some(val) = self.stack[ra].into_integer() {
            *limit = val;
        } else {
            // not coercible to integer
            if let Some(fval) = self.stack[ra].into_float() {
                // 'fval' is a float out of integer bounds
                if fval > 0.0 {
                    // if it is positive, it is too large
                    if step < 0 {
                        //  initial value must be less than it
                        return Ok(true);
                    }
                    *limit = LuaInteger::MAX; // truncate
                } else {
                    // it is less than min integer
                    if step > 0 {
                        // initial value must be greater than it
                        return Ok(true);
                    }
                    *limit = LuaInteger::MIN; // truncate
                }
            } else {
                return Err(for_error(self, ra, "limit"));
            }
        }
        if step > 0 {
            return Ok(init > *limit);
        }
        Ok(init < *limit)
    }

    /// Insert a variable in the list of to-be-closed variables.
    pub(crate) fn new_tbc_value(&self, level: usize) {
        debug_assert!(self.tbc_list.is_empty() || level > *self.tbc_list.last().unwrap());
        if self.stack[level].is_false() {
            return; // false doesn't need to be closed
        }
        // TODO close metamethods
        todo!()
    }

    pub(crate) fn tforcall(
        &mut self,
        pc: &mut usize,
        protoid: usize,
        base: u32,
        i: u32,
    ) -> Result<(), LuaError> {
        let ra = get_ra(base, i);
        // 'ra' has the iterator function, 'ra + 1' has the state,
        // 'ra + 2' has the control variable, and 'ra + 3' has the
        // to-be-closed variable. The call will use the stack after
        // these values (starting at 'ra + 4')

        //  push function, state, and control variable
        self.set_stack_from_idx(ra + 4, ra);
        self.set_stack_from_idx(ra + 5, ra + 1);
        self.set_stack_from_idx(ra + 6, ra + 2);
        self.base_ci[self.ci].saved_pc = *pc;
        self.dcall(ra + 4, get_arg_c(i) as i32)?; // do the call
        let i = self.get_instruction(protoid, *pc);
        *pc += 1;
        debug_assert!(get_opcode(i) == OpCode::TForLoop && ra == get_ra(base, i));
        self.tforloop(pc, base, i);
        Ok(())
    }

    pub(crate) fn tforloop(&mut self, pc: &mut usize, base: u32, i: u32) {
        let ra = get_ra(base, i);
        if !self.stack[ra + 4].is_nil() {
            // continue loop?
            self.set_stack_from_idx(ra + 2, ra + 4); // save control variable
            *pc = (*pc as isize - get_arg_bx(i) as isize) as usize; // jump back
        }
    }
}

/// Close all upvalues and to-be-closed variables up to the given stack
/// level. Return restored 'level'.
pub(crate) fn f_close(
    state: &mut LuaState,
    level: usize,
    status: i32,
    yy: bool,
) -> Result<(), LuaError> {
    close_upval(state, level); // first, close the upvalues
    if state.tbc_list.is_empty() {
        return Ok(());
    }
    while *state.tbc_list.last().unwrap() >= level {
        // traverse tbc's down to that level
        let tbc = state.tbc_list.pop().unwrap(); // remove variable from the list
        prep_call_close_method(state, tbc, status, yy)?; // close variable
    }
    Ok(())
}

/// Prepare and call a closing method.
/// If status is CLOSEKTOP, the call to the closing method will be pushed
/// at the top of the stack. Otherwise, values can be pushed right after
/// the 'level' of the upvalue being closed, as everything after that
/// won't be used again.
fn prep_call_close_method(
    _state: &mut LuaState,
    _level: usize,
    _status: i32,
    _yy: bool,
) -> Result<(), LuaError> {
    // TODO
    todo!()
}

/// Close all upvalues up to the given stack level.
fn close_upval(state: &mut LuaState, level: usize) {
    while !state.open_upval.is_empty() && state.open_upval.last().unwrap().v >= level {
        let uv = state.open_upval.pop().unwrap();
        state.stack[uv.v] = uv.value.clone();
    }
}

#[cfg(feature = "debug_logs")]
fn dump_function_header(state: &mut LuaState, func: usize) {
    let cl = state.get_closure_ref(func);
    let cl = cl.borrow();
    let cl = cl.borrow_lua_closure();
    let nup = cl.upvalues.len();
    let proto = &state.protos[cl.proto];
    let nk = proto.k.len();
    if proto.line_defined == proto.lastlinedefined {
        _ = writeln!(state.stdout, "; function [{}] ", proto.line_defined);
    } else {
        _ = writeln!(
            state.stdout,
            "; function [{}-{}] ",
            proto.line_defined, proto.lastlinedefined
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
    let o = get_opcode(i);
    let a = get_arg_a(i);
    let b = get_arg_b(i) as i8;
    let c = get_arg_c(i) as i8;
    let sc = get_arg_sc(i);
    let ax = get_arg_ax(i);
    let sbx = get_arg_sbx(i);
    let bx = get_arg_bx(i);
    let k = get_arg_k(i);
    let cl = state.get_closure_ref(func);
    let cl = cl.borrow();
    let cl = cl.borrow_lua_closure();
    let proto = &state.protos[cl.proto];
    let mut res = if o.is_asbx() {
        format!("{:10} {:>5} {:>5}", OPCODE_NAME[o as usize], a, sbx)
    } else if o.is_a() {
        format!("{:10} {:>5}", OPCODE_NAME[o as usize], a)
    } else if o.is_ax() {
        format!("{:10} {:>5}", OPCODE_NAME[o as usize], ax)
    } else if o.is_abx() {
        format!("{:10} {:>5} {:>5}", OPCODE_NAME[o as usize], a, bx)
    } else if o.is_ak() {
        format!("{:10} {:>5} {:>5}", OPCODE_NAME[o as usize], a, k)
    } else if o.is_ab() {
        format!("{:10} {:>5} {:>5}", OPCODE_NAME[o as usize], a, b)
    } else if o.is_ac() {
        format!("{:10} {:>5} {:>5}", OPCODE_NAME[o as usize], a, c)
    } else if o.is_absc() {
        format!("{:10} {:>5} {:>5} {:>5}", OPCODE_NAME[o as usize], a, b, sc)
    } else {
        // abc
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
    if !(state.stack[top - 2].is_string() || state.stack[top - 2].is_number()) {
        // TODO metamethods
        luaG::concat_error(state, top as isize - 2, top as isize - 1)
    } else {
        let mut res = String::new();
        let first = top - total;
        for i in first..top {
            res.push_str(&state.stack[i].to_string());
        }
        state.stack[first] = TValue::from(res);
        state.stack.resize(first + 1, TValue::Nil);
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

#[inline]
fn get_rc(base: u32, i: u32) -> usize {
    (base + get_arg_c(i)) as usize
}
