//! Global State

use std::{cell::RefCell, rc::Rc};

use crate::{
    api::LuaError,
    ldo::CallId,
    limits::InstId,
    luaD::rawrunprotected,
    luaH::{Table, TableRef},
    object::{Closure, RClosure, StkId, TValue, TVALUE_TYPE_COUNT},
    LuaRustFunction, LUA_ENVIRONINDEX, LUA_GLOBALSINDEX, LUA_MINSTACK, LUA_MULTRET,
    LUA_REGISTRYINDEX,
};

pub type PanicFunction = fn(LuaStateRef) -> i32;

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
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct GlobalState {
    /// to be called in unprotected errors
    pub panic: Option<PanicFunction>,
    /// metatables for basic types
    pub mt: Vec<Option<TableRef>>,
    pub registry: TValue,
}

impl Default for GlobalState {
    fn default() -> Self {
        let mut mt = Vec::new();
        for _ in 0..TVALUE_TYPE_COUNT {
            mt.push(None)
        }
        Self {
            panic: None,
            mt,
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
}

impl LuaState {
    pub fn init_stack(&mut self) {
        // initialize first ci
        let mut ci = CallInfo::new();
        // `function' entry for this `ci'
        //self.stack.push(TValue::Nil);
        self.base = 0;
        ci.base = 0;
        ci.top = 1 + LUA_MINSTACK;
        self.base_ci.push(ci);
    }
    pub fn push_rust_function(&mut self, func: LuaRustFunction) {
        self.push_rust_closure(func, 0);
    }
    pub fn push_string(&mut self, value: &str) {
        self.stack.push(TValue::String(Rc::new(value.to_owned())));
    }
    pub fn call(&mut self, nargs: usize, nresults: i32) -> Result<(), LuaError> {
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
        self.stack
            .push(TValue::Function(Rc::new(Closure::Rust(cl))));
    }

    fn get_current_env(&self) -> TableRef {
        if self.base_ci.len() == 1 {
            // no enclosing function
            // use global table as environment
            return Rc::clone(&self.l_gt);
        } else {
            let ci_stkid = self.base_ci.last().unwrap().func;
            if let TValue::Function(cl) = &self.stack[ci_stkid] {
                return cl.get_env();
            }
        }
        unreachable!()
    }

    pub fn run_error(&self, _arg: &str) -> Result<(), LuaError> {
        todo!()
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
        self.stack.push(TValue::new_string(value));
    }

    pub(crate) fn create_table(&mut self) {
        self.stack.push(TValue::new_table());
    }

    pub(crate) fn set_metatable(&mut self, objindex: isize) {
        debug_assert!(self.stack.len() >= 1);
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
                        rcobj.borrow_mut().metatable = Some(rcmt.clone());
                        return;
                    } else {
                        rcobj.borrow_mut().metatable = None;
                        return;
                    }
                }
                TValue::UserData(rcobj) => {
                    if let Some(TValue::Table(rcmt)) = mt {
                        rcobj.borrow_mut().metatable = Some(rcmt.clone());
                        return;
                    } else {
                        rcobj.borrow_mut().metatable = None;
                        return;
                    }
                }
                _ => obj.type_as_usize(),
            }
        };
        if let Some(TValue::Table(rcmt)) = mt {
            self.g.mt[objtype] = Some(rcmt.clone());
        } else {
            self.g.mt[objtype] = None;
        }
    }

    pub(crate) fn set_tablev(&self, tvalue: &TValue, key: TValue, value: TValue) {
        // TODO NEWINDEX metamethods
        if let TValue::Table(rt) = tvalue {
            rt.borrow_mut().set(key, value);
            return;
        } else {
            unreachable!()
        }
    }

    /// put field value `key` from table `t` on stack
    pub(crate) fn get_tablev(&mut self, t: &TValue, key: &TValue, val: Option<StkId>) {
        // TODO INDEX metamethods
        if let TValue::Table(rt) = t {
            match rt.borrow_mut().get(key) {
                Some(value) => {
                    match val {
                        Some(idx) => self.stack[idx]=value.clone(),
                        None => self.stack.push(value.clone()),
                    }
                    return;
                }
                None => {
                    match val {
                        Some(idx) => self.stack[idx] = TValue::Nil,
                        None => self.stack.push(TValue::Nil),
                    }
                }
            }
        }
    }

    /// set a field `k` on table at position `idx` with the last stack value as value
    pub(crate) fn set_field(&mut self, idx: isize, k: &str) {
        debug_assert!(self.stack.len() >= 1);
        let value = self.stack.pop().unwrap();
        let idx = if idx < 0 && idx > LUA_REGISTRYINDEX {
            idx + 1
        } else {
            idx
        };
        let tvalue = self.index2adr(idx);
        debug_assert!(*tvalue != TValue::Nil);
        let key = TValue::String(Rc::new(k.to_owned()));
        self.set_tablev(tvalue, key, value);
    }

    pub(crate) fn index2adr(&self, index: isize) -> &TValue {
        if index > 0 {
            // positive index in the stack
            let index = index as usize + self.base;
            debug_assert!(index <= self.base_ci[self.ci].top);
            if index - 1 >= self.stack.len() {
                return &TValue::Nil;
            }
            &self.stack[index - 1]
        } else if index > LUA_REGISTRYINDEX {
            // negative index in the stack (count from top)
            let index = (-index) as usize;
            debug_assert!(index != 0 && index <= self.stack.len());
            &self.stack[self.stack.len() - index]
        } else {
            match index {
                LUA_REGISTRYINDEX => &self.g.registry,
                LUA_ENVIRONINDEX => {
                    let stkid = self.base_ci[self.ci].func;
                    if let TValue::Function(cl) = &self.stack[stkid] {
                        cl.get_envvalue()
                    } else {
                        unreachable!()
                    }
                }
                LUA_GLOBALSINDEX => &self.gtvalue,
                _ => {
                    // global index - n => return nth upvalue of current Rust closure
                    let index = (LUA_GLOBALSINDEX - index) as usize;
                    let stkid = self.base_ci[self.ci].func;
                    if let TValue::Function(cl) = &self.stack[stkid] {
                        if index <= cl.get_nupvalues() {
                            if let Closure::Rust(cl) = cl.as_ref() {
                                return cl.borrow_upvalue(index - 1);
                            }
                        }
                        &TValue::Nil
                    } else {
                        unreachable!()
                    }
                }
            }
        }
    }

    /// put field value `key` from table at `index` on stack
    pub(crate) fn get_field(&mut self, index: isize, key: &str) {
        let t = self.index2adr(index);
        self.get_tablev(&t.clone(), &TValue::new_string(key), None);
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
        self.set_tablev(t, key, value);
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
        let mut i=wanted;
        // move results to correct place
        while i != 0 && (first_result as usize) < self.stack.len() {
            let result = self.stack.remove(first_result as usize);
            self.stack[res] = result;
            res+=1;
            i-=1;
        }
        wanted != LUA_MULTRET
    }
}

fn f_luaopen(state: LuaStateRef, _: ()) -> Result<i32, LuaError> {
    let gt = Table::new();
    let mut state = state.borrow_mut();
    state.init_stack();
    // table of globals
    state.l_gt = Rc::new(RefCell::new(gt));
    state.gtvalue = TValue::Table(state.l_gt.clone());
    Ok(0)
}

pub fn newstate() -> LuaStateRef {
    let mut state = LuaState::default();
    state.allowhook = true;
    let stateref = Rc::new(RefCell::new(state));
    if rawrunprotected(Rc::clone(&stateref), f_luaopen, ()).is_err() {
    } else {
    }
    stateref
}

pub type LuaStateRef = Rc<RefCell<LuaState>>;

pub fn push_string(state: LuaStateRef, s: &str) {
    state.borrow_mut().push_string(s);
}
