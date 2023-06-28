//! Global State

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    api::LuaError,
    ldo::CallId,
    limits::InstId,
    luaH::{Table, TableRef},
    object::{Closure, Proto, RClosure, StkId, TValue, UpVal},
    opcodes::{get_arg_b, get_arg_c, rk_is_k, BIT_RK},
    LuaNumber, LuaRustFunction, LUA_ENVIRONINDEX, LUA_GLOBALSINDEX, LUA_MINSTACK, LUA_MULTRET,
    LUA_REGISTRYINDEX, lex::str2d,
};

pub type PanicFunction = fn(&mut LuaState) -> i32;

pub const EXTRA_STACK: usize = 5;

/// informations about a call
#[derive(Default)]
pub struct CallInfo {
    /// base for this function
    pub base: StkId,
    /// function index in the stack
    pub func: StkId,
    /// top for this function
    pub top: StkId,
    /// program counter
    pub saved_pc: InstId,
    /// expected number of results from this function
    pub nresults: i32,
    /// number of tail calls lost under this entry
    pub tailcalls: usize,
}

impl CallInfo {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

pub struct GlobalState {
    /// to be called in unprotected errors
    pub panic: Option<PanicFunction>,
    /// metatables for basic types
    pub mt: HashMap<String, Option<TableRef>>,
    pub registry: TValue,
}

impl Default for GlobalState {
    fn default() -> Self {
        Self {
            panic: None,
            mt: HashMap::new(),
            registry: TValue::new_table(),
        }
    }
}

#[derive(Default)]
pub struct LuaState {
    pub g: GlobalState,
    /// base of current function
    pub base: StkId,
    /// `savedpc' of current function
    pub saved_pc: InstId,
    /// stack base
    pub stack: Vec<TValue>,
    /// current error handling function (stack index)
    pub errfunc: StkId,
    /// number of nested Rust calls
    pub n_rcalls: usize,
    /// call info for current function
    pub ci: CallId,
    /// list of nested CallInfo
    pub base_ci: Vec<CallInfo>,
    pub allowhook: bool,
    pub hookmask: usize,
    /// table of globals
    pub l_gt: TableRef,
    pub gtvalue: TValue,
    /// temporary place for environments
    pub env: TableRef,
    pub envvalue: TValue,
    /// list of open upvalues
    pub open_upval: Vec<UpVal>,
    /// all closures prototypes
    pub protos: Vec<Proto>,
}

impl LuaState {
    pub(crate) fn init_stack(&mut self) {
        // initialize first ci
        let mut ci = CallInfo::new();
        // `function' entry for this `ci'
        //self.stack.push(TValue::Nil);
        self.base = 0;
        ci.base = 0;
        ci.top = 1 + LUA_MINSTACK;
        self.base_ci.push(ci);
    }
    #[inline]
    pub fn get_lua_constant(&self, protoid: usize, kid: usize) -> TValue {
        self.protos[protoid].k[kid].clone()
    }
    pub(crate) fn push_rust_function(&mut self, func: LuaRustFunction) {
        self.push_rust_closure(func, 0);
    }
    pub(crate) fn push_string(&mut self, value: &str) {
        self.stack.push(TValue::String(Rc::new(value.to_owned())));
    }
    pub(crate) fn push_number(&mut self, value: LuaNumber) {
        self.stack.push(TValue::Number(value));
    }
    pub(crate) fn push_boolean(&mut self, value: bool) {
        self.stack.push(TValue::Boolean(value));
    }
    pub(crate) fn push_nil(&mut self) {
        self.stack.push(TValue::Nil);
    }
    pub(crate) fn call(&mut self, nargs: usize, nresults: i32) -> Result<(), LuaError> {
        self.api_check_nelems(nargs + 1);
        self.check_results(nargs, nresults);
        let len = self.stack.len();
        self.dcall(len - nargs - 1, nresults)?;
        self.adjust_results(nresults);
        Ok(())
    }

    #[inline]
    fn api_check_nelems(&self, n: usize) {
        debug_assert!(n as i32 <= self.stack.len() as i32 - self.base as i32);
    }
    #[inline]
    pub(crate) fn check_results(&self, nargs: usize, nresults: i32) {
        debug_assert!(
            nresults == LUA_MULTRET
                || self.base_ci.last().unwrap().top as isize - self.stack.len() as isize
                    >= nresults as isize - nargs as isize
        );
    }

    pub(crate) fn push_rust_closure(&mut self, func: LuaRustFunction, nup_values: usize) {
        self.api_check_nelems(nup_values);
        let env = self.get_current_env();
        let mut cl = RClosure::new(func, Rc::clone(&env));
        for _ in 0..nup_values {
            cl.upvalues.push(self.stack.pop().unwrap());
        }
        self.stack.push(TValue::from(cl));
    }

    fn get_current_env(&self) -> TableRef {
        if self.base_ci.len() == 1 {
            // no enclosing function
            // use global table as environment
            return Rc::clone(&self.l_gt);
        } else {
            let ci_stkid = self.base_ci.last().unwrap().func;
            if let TValue::Function(cl) = &self.stack[ci_stkid] {
                return cl.borrow().get_env();
            }
        }
        unreachable!()
    }

    pub(crate) fn run_error(&mut self, msg: &str) -> Result<(), LuaError> {
        let fullmsg = {
            let ci = &self.base_ci[self.ci];
            let luacl = &self.stack[ci.func];
            if let TValue::Function(rcl) = luacl {
                let pc = self.saved_pc;
                let proto = &self.protos[rcl.borrow().get_lua_protoid()];
                let line = proto.lineinfo[pc];
                let chunk_id = &proto.source;
                format!("{}:{} {}", chunk_id, line, msg)
            } else {
                unreachable!()
            }
        };
        self.stack.push(TValue::from(&fullmsg[..]));
        Err(LuaError::RuntimeError)
    }

    pub(crate) fn adjust_results(&mut self, nresults: i32) {
        if nresults == LUA_MULTRET && self.stack.len() >= self.base_ci[self.ci].top {
            self.base_ci[self.ci].top = self.stack.len();
        }
    }

    pub(crate) fn push_value(&mut self, index: isize) {
        self.stack.push(self.index2adr(index).clone());
    }

    /// create a global variable `key` with last value on stack
    pub(crate) fn set_global(&mut self, key: &str) {
        self.set_field(LUA_GLOBALSINDEX, key);
    }

    pub(crate) fn push_literal(&mut self, value: &str) {
        self.stack.push(TValue::from(value));
    }

    pub(crate) fn create_table(&mut self) {
        self.stack.push(TValue::new_table());
    }

    pub(crate) fn set_metatable(&mut self, objindex: isize) {
        debug_assert!(!self.stack.is_empty());
        let mt = self.stack.pop().unwrap();
        let mt = if mt.is_nil() { None } else { Some(mt) };
        let objtype = {
            let objindex = if objindex < 0 && objindex > LUA_REGISTRYINDEX {
                objindex + 1
            } else {
                objindex
            };
            let obj = self.index2adr(objindex);
            match obj {
                TValue::Table(rcobj) => {
                    if let Some(TValue::Table(rcmt)) = mt {
                        rcobj.borrow_mut().metatable = Some(rcmt);
                        return;
                    } else {
                        rcobj.borrow_mut().metatable = None;
                        return;
                    }
                }
                TValue::UserData(rcobj) => {
                    if let Some(TValue::Table(rcmt)) = mt {
                        rcobj.borrow_mut().metatable = Some(rcmt);
                        return;
                    } else {
                        rcobj.borrow_mut().metatable = None;
                        return;
                    }
                }
                _ => obj.get_type_name().to_owned(),
            }
        };
        if let Some(TValue::Table(rcmt)) = mt {
            self.g.mt.insert(objtype, Some(rcmt));
        } else {
            self.g.mt.remove(&objtype);
        }
    }

    pub(crate) fn set_tablev(&self, tvalue: &TValue, key: TValue, value: TValue) {
        // TODO NEWINDEX metamethods
        if let TValue::Table(rt) = tvalue {
            rt.borrow_mut().set(key, value);
        } else {
            unreachable!()
        }
    }

    pub(crate) fn get_tablev2(
        stack: &mut Vec<TValue>,
        t: &TValue,
        key: &TValue,
        val: Option<StkId>,
    ) {
        // TODO INDEX metamethods
        if let TValue::Table(rt) = t {
            let mut rt = rt.clone();
            loop {
                let newrt;
                {
                    let mut rtmut = rt.borrow_mut();
                    match rtmut.get(key) {
                        Some(value) => {
                            // found a value, put it on stack
                            match val {
                                Some(idx) => if idx == stack.len() {
                                    stack.push(value.clone());
                                } else {
                                    stack[idx] = value.clone();
                                },
                                None => return stack.push(value.clone()),
                            }
                            return;
                        }
                        None => {
                            if let Some(ref mt) = rtmut.metatable {
                                // not found. try with the metatable
                                newrt = mt.clone();
                            } else {
                                // no metatable, put Nil on stack
                                match val {
                                    Some(idx) => if idx == stack.len() {
                                        stack.push(TValue::Nil);
                                    } else {
                                        stack[idx] = TValue::Nil;
                                    },
                                    None => stack.push(TValue::Nil),
                                }
                                return;
                            }
                        }
                    }
                }
                rt = newrt;
            }
        }
    }
    /// put field value `key` from table `t` on stack
    pub(crate) fn get_tablev(
        stack: &mut Vec<TValue>,
        tableid: usize,
        key: &TValue,
        val: Option<StkId>,
    ) {
        // TODO INDEX metamethods
        if let TValue::Table(rt) = &stack[tableid] {
            let mut rt = rt.clone();
            loop {
                let newrt;
                {
                    let mut rtmut = rt.borrow_mut();
                    match rtmut.get(key) {
                        Some(value) => {
                            // found a value, put it on stack
                            match val {
                                Some(idx) => {
                                    if idx == stack.len() {
                                        stack.push(value.clone());
                                    } else {
                                        stack[idx] = value.clone();
                                    }
                                }
                                None => return stack.push(value.clone()),
                            }
                            return;
                        }
                        None => {
                            if let Some(ref mt) = rtmut.metatable {
                                // not found. try with the metatable
                                newrt = mt.clone();
                            } else {
                                // no metatable, put Nil on stack
                                match val {
                                    Some(idx) => if idx == stack.len() {
                                        stack.push(TValue::Nil);
                                    } else {
                                        stack[idx] = TValue::Nil;
                                    },
                                    None => stack.push(TValue::Nil),
                                }
                                return;
                            }
                        }
                    }
                }
                rt = newrt;
            }
        }
    }

    /// set a field `k` on table at position `idx` with the last stack value as value
    pub(crate) fn set_field(&mut self, idx: isize, k: &str) {
        debug_assert!(!self.stack.is_empty());
        let value = self.stack.pop().unwrap();
        let idx = if idx < 0 && idx > LUA_REGISTRYINDEX {
            idx + 1
        } else {
            idx
        };
        let tvalue = self.index2adr(idx);
        debug_assert!(tvalue != TValue::Nil);
        let key = TValue::from(k);
        self.set_tablev(&tvalue, key, value);
    }

    pub(crate) fn index2adr(&self, index: isize) -> TValue {
        if index > 0 {
            // positive index in the stack
            let index = index as usize + self.base;
            debug_assert!(index <= self.base_ci[self.ci].top);
            if index > self.stack.len() {
                return TValue::Nil;
            }
            self.stack[index - 1].clone()
        } else if index > LUA_REGISTRYINDEX {
            // negative index in the stack (count from top)
            let index = (-index) as usize;
            debug_assert!(index != 0 && index <= self.stack.len());
            self.stack[self.stack.len() - index].clone()
        } else {
            match index {
                LUA_REGISTRYINDEX => self.g.registry.clone(),
                LUA_ENVIRONINDEX => {
                    let stkid = self.base_ci[self.ci].func;
                    if let TValue::Function(rcl) = &self.stack[stkid] {
                        let cl=rcl.borrow();
                        cl.get_envvalue().clone()
                    } else {
                        unreachable!()
                    }
                }
                LUA_GLOBALSINDEX => self.gtvalue.clone(),
                _ => {
                    // global index - n => return nth upvalue of current Rust closure
                    let index = (LUA_GLOBALSINDEX - index) as usize;
                    let stkid = self.base_ci[self.ci].func;
                    if index <= self.get_closure_nupvalues(stkid) {
                        return self.get_rust_closure_upvalue(stkid, index-1).clone();
                    }
                    TValue::Nil
                }
            }
        }
    }

    /// put field value `key` from table at `index` on stack
    pub(crate) fn get_field(&mut self, index: isize, key: &str) {
        let t = self.index2adr(index).clone();
        Self::get_tablev2(&mut self.stack, &t, &TValue::from(key), None);
    }

    pub(crate) fn is_table(&self, arg: isize) -> bool {
        self.index2adr(arg).is_table()
    }
    pub(crate) fn is_nil(&self, arg: isize) -> bool {
        self.index2adr(arg).is_nil()
    }

    pub(crate) fn pop_stack(&mut self, count: usize) {
        let newlen = self.stack.len() - count;
        self.stack.truncate(newlen);
    }

    #[inline]
    /// convert an index into an absolute index (-1 => stack.len()-1)
    fn index2abs(&self, index: isize) -> usize {
        if index < 0 {
            self.stack.len() - (-index) as usize
        } else {
            index as usize
        }
    }

    pub(crate) fn remove(&mut self, index: isize) {
        let index = self.index2abs(index);
        self.stack.remove(index);
    }

    /// move the stack top element to position `index`
    pub(crate) fn insert(&mut self, index: isize) {
        let index = self.index2abs(index);
        let value = self.stack.pop().unwrap();
        self.stack.insert(index, value);
    }

    /// get a field value from table at `index`. field name is last value on stack
    /// result : field value is last value on stack
    pub(crate) fn rawget(&mut self, index: isize) {
        let value = {
            let key = self.stack.pop().unwrap();
            let index = if index < 0 && index > LUA_REGISTRYINDEX {
                index + 1
            } else {
                index
            };
            let t = self.index2adr(index);
            if let TValue::Table(rct) = t {
                let mut t = rct.borrow_mut();
                t.get(&key).unwrap_or(&TValue::Nil).clone()
            } else {
                unreachable!()
            }
        };
        self.stack.push(value);
    }

    /// set a field on table at `index`. key and value are the last two objects on stack
    pub(crate) fn set_table(&mut self, index: isize) {
        debug_assert!(self.stack.len() >= 2);
        let value = self.stack.pop().unwrap();
        let key = self.stack.pop().unwrap();
        let index = if index < 0 && index > LUA_REGISTRYINDEX {
            index + 2
        } else {
            index
        };
        let t = self.index2adr(index);
        self.set_tablev(&t, key, value);
    }

    pub(crate) fn poscall(&mut self, first_result: u32) -> bool {
        // TODO hooks
        let ci = &self.base_ci[self.ci];
        // res == final position of 1st result
        let mut res = ci.func;
        let wanted = ci.nresults;

        self.base_ci.pop();
        self.ci -= 1;
        let ci = &self.base_ci[self.ci];
        self.base = ci.base;
        self.saved_pc = ci.saved_pc;
        let mut i = wanted;
        // move results to correct place
        let mut first_result = first_result as usize;
        while i != 0 && first_result < self.stack.len() {
            self.stack[res] = self.stack[first_result].clone();
            res += 1;
            first_result += 1;
            i -= 1;
        }
        while i > 0 {
            i = -1;
            self.stack[res] = TValue::Nil;
            res += 1;
        }
        self.stack.resize(res, TValue::Nil);
        wanted != LUA_MULTRET
    }

    pub(crate) fn find_upval(upvals: &mut Vec<UpVal>, stack: &mut [TValue], level: u32) -> UpVal {
        let mut index = 0;
        for (i, val) in upvals.iter().enumerate().rev() {
            if val.v < level as StkId {
                index = i + 1;
                break;
            }
            if val.v == level as StkId {
                // found a corresponding value
                return val.clone();
            }
        }
        let uv = UpVal {
            v: level as StkId,
            value: stack[level as usize].clone(),
        };
        upvals.insert(index, uv.clone());
        uv
    }

    /// convert stack[obj] to a number into stack[dst], return the number value
    pub(crate) fn to_number(
        stack: &mut [TValue],
        obj: StkId,
        dst: Option<StkId>,
    ) -> Option<LuaNumber> {
        match &stack[obj] {
            TValue::Number(n) => Some(*n),
            TValue::String(s) => match str2d(s) {
                Some(n) => {
                    if let Some(dst) = dst {
                        stack[dst] = TValue::Number(n);
                    }
                    Some(n)
                }
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn close_func(&mut self, level: StkId) {
        while let Some(uv) = self.open_upval.last() {
            if uv.v < level {
                break;
            }
            // if uv.v < self.stack.len() {
            //     //uv.value = self.stack[uv.v].clone();
            //     //self.stack[uv.v] = uv.value.clone();
            // }
            // TODO save upvals somewhere
            self.open_upval.pop();
        }
    }

    pub(crate) fn get_rkb(&self, i: u32, base: u32, protoid: usize) -> TValue {
        let b = get_arg_b(i);
        let rbi = (base + b) as usize;
        if rk_is_k(b) {
            self.get_lua_constant(protoid, (b & !BIT_RK) as usize)
        } else {
            self.stack[rbi].clone()
        }
    }
    pub(crate) fn get_rkc(&self, i: u32, base: u32, protoid: usize) -> TValue {
        let c = get_arg_c(i);
        let rci = (base + c) as usize;
        if rk_is_k(c) {
            self.get_lua_constant(protoid, (c & !BIT_RK) as usize)
        } else {
            self.stack[rci].clone()
        }
    }

    pub(crate) fn get_lua_closure_env(&self, func: usize) -> TableRef {
        let cl = if let TValue::Function(cl) = &self.stack[func] {
            cl.borrow()
        } else {
            unreachable!()
        };
        cl.get_env().clone()
    }

    pub(crate) fn get_lua_closure_env_value(&self, func: usize) -> TValue {
        let cl = if let TValue::Function(cl) = &self.stack[func] {
            cl.borrow()
        } else {
            unreachable!()
        };
        cl.get_envvalue().clone()
    }
    pub(crate) fn get_lua_closure_upval_desc(&self, func: usize, upval_id: usize) -> UpVal {
        let cl = if let TValue::Function(cl) = &self.stack[func] {
            cl.borrow()
        } else {
            unreachable!()
        };
        cl.get_lua_upval_desc(upval_id)
    }
    pub(crate) fn get_lua_closure_protoid(&self, func: usize) -> usize {
        let cl = if let TValue::Function(cl) = &self.stack[func] {
            cl.borrow()
        } else {
            unreachable!()
        };
        cl.get_lua_protoid()
    }
    pub(crate) fn get_lua_closure_upvalue(&self, func: usize, upval_id: usize) -> TValue {
        let cl = if let TValue::Function(cl) = &self.stack[func] {
            cl.borrow()
        } else {
            unreachable!()
        };
        cl.get_lua_upvalue(upval_id)
    }
    pub(crate) fn set_lua_closure_upvalue(&mut self, func: usize, upval_id: usize, value: TValue) {
        let mut cl = if let TValue::Function(cl) = &mut self.stack[func] {
            cl.borrow_mut()
        } else {
            unreachable!()
        };
        cl.set_lua_upvalue(upval_id, value);
    }

    fn get_closure_nupvalues(&self, func: usize) -> usize {
        let cl = if let TValue::Function(cl) = &self.stack[func] {
            cl.borrow()
        } else {
            unreachable!()
        };
        cl.get_nupvalues()
    }

    fn get_rust_closure_upvalue(&self, func: usize, upval_id: usize) -> TValue {
        let cl = if let TValue::Function(cl) = &self.stack[func] {
            cl.borrow()
        } else {
            unreachable!()
        };
        cl.get_rust_upvalue(upval_id)
    }

    pub(crate) fn set_or_push(&mut self, index: usize, val: TValue) {
        if index == self.stack.len() {
            self.stack.push(val);
        } else {
            self.stack[index] = val;
        }
    }
}

fn f_luaopen(state: &mut LuaState, _: ()) -> Result<i32, LuaError> {
    let gt = Table::new();
    state.init_stack();
    // table of globals
    state.l_gt = Rc::new(RefCell::new(gt));
    state.gtvalue = TValue::from(&state.l_gt);
    Ok(0)
}

pub(crate) fn newstate() -> LuaState {
    let mut state = LuaState {
        allowhook: true,
        ..Default::default()
    };
    f_luaopen(&mut state, ()).ok();
    state
}
