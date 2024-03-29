//! Global State

use std::{collections::HashMap, rc::Rc};

use crate::{
    api::LuaError,
    ldo::CallId,
    lex::{str2d, LexState},
    limits::{InstId, MAX_UPVAL},
    luaH::TableRef,
    object::{Closure, ClosureRef, Proto, ProtoId, RClosure, StkId, TValue, UpVal},
    opcodes::{get_arg_b, get_arg_c, rk_is_k, BIT_RK},
    LuaFloat, LuaInteger, LuaRustFunction, LUA_MINSTACK, LUA_MULTRET, LUA_REGISTRYINDEX,
    LUA_RIDX_GLOBALS,
};

#[cfg(target_arch = "wasm32")]
use crate::wasm::js_console;

pub type PanicFunction = fn(&mut LuaState) -> i32;

pub const EXTRA_STACK: usize = 5;

// Bits in CallInfo call_status
/// original value of 'allowhook'
pub const CIST_OAH: u32 = 1 << 0;
/// call is running a Lua function
pub const CIST_LUA: u32 = 1 << 1;
/// call is running a debug hook
pub const CIST_HOOKED: u32 = 1 << 2;
/// call is running on a fresh invocation of luaV_execute
pub const CIST_FRESH: u32 = 1 << 3;
/// call is a yieldable protected call
pub const CIST_YPCALL: u32 = 1 << 4;
/// call was tail called
pub const CIST_TAIL: u32 = 1 << 5;
/// last hook called yielded
pub const CIST_HOOKYIELD: u32 = 1 << 6;
/// using __lt for __le
pub const CIST_LEQ: u32 = 1 << 7;
/// call is running a finalizer
pub const CIST_FIN: u32 = 1 << 8;

/// informations about a call
/// When a thread yields, 'func' is adjusted to pretend that the
/// top function has only the yielded values in its stack; in that
/// case, the actual 'func' value is saved in field 'extra'.
/// When a function calls another with a continuation, 'extra' keeps
/// the function index so that, in case of errors, the continuation
/// function can be called with the correct top.
#[derive(Default)]
pub struct CallInfo {
    /// function index in the stack
    pub func: StkId,
    /// top for this function
    pub top: StkId,
    /// expected number of results from this function
    pub nresults: i32,
    /// bitfield. see CIST_*
    pub call_status: u32,
    // for Lua functions
    /// base for this function
    pub base: StkId,
    /// program counter
    pub saved_pc: InstId,
    // for Rust functions
    /// context info. in case of yields
    pub ctx: u32,
    /// continuation in case of yields
    pub k: Option<LuaRustFunction>,
}

impl CallInfo {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

/// 'global state', shared by all threads of this state
pub struct GlobalState {
    /// to be called in unprotected errors
    pub panic: Option<PanicFunction>,
    /// metatables for basic types
    pub mt: HashMap<String, Option<TableRef>>,
    registry: TValue,
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

/// 'per thread' state
pub struct LuaState {
    pub g: GlobalState,
    /// stack
    pub stack: Vec<TValue>,
    /// current error handling function (stack index)
    pub errfunc: StkId,
    /// number of non-yieldable calls in stack
    pub nny: usize,
    /// number of nested Rust calls
    pub n_rcalls: usize,
    /// call info for current function
    pub ci: CallId,
    /// list of nested CallInfo
    pub base_ci: Vec<CallInfo>,
    pub allowhook: bool,
    pub hookmask: usize,
    /// temporary place for environments
    pub env: TableRef,
    pub envvalue: TValue,
    /// list of open upvalues
    pub open_upval: Vec<UpVal>,
    /// all closures prototypes
    pub protos: Vec<Proto>,
    /// io default output
    pub stdout: Box<dyn std::io::Write>,
    /// io default error output
    pub stderr: Box<dyn std::io::Write>,
}

#[cfg(target_arch = "wasm32")]
impl Default for LuaState {
    fn default() -> Self {
        Self {
            stdout: js_console(),
            stderr: js_console(),
            g: Default::default(),
            stack: Default::default(),
            errfunc: Default::default(),
            nny: Default::default(),
            n_rcalls: Default::default(),
            ci: Default::default(),
            base_ci: Default::default(),
            allowhook: Default::default(),
            hookmask: Default::default(),
            env: Default::default(),
            envvalue: Default::default(),
            open_upval: Default::default(),
            protos: Default::default(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for LuaState {
    fn default() -> Self {
        Self {
            stdout: Box::new(std::io::stdout()),
            stderr: Box::new(std::io::stderr()),
            g: Default::default(),
            stack: Default::default(),
            errfunc: Default::default(),
            nny: Default::default(),
            n_rcalls: Default::default(),
            ci: Default::default(),
            base_ci: Default::default(),
            allowhook: Default::default(),
            hookmask: Default::default(),
            env: Default::default(),
            envvalue: Default::default(),
            open_upval: Default::default(),
            protos: Default::default(),
        }
    }
}

impl LuaState {
    pub(crate) fn init_stack(&mut self) {
        // initialize first ci
        let mut ci = CallInfo::new();
        // `function' entry for this `ci'
        //self.stack.push(TValue::Nil);
        ci.top = 1 + LUA_MINSTACK;
        ci.call_status = CIST_LUA;
        self.base_ci.push(ci);
    }
    #[inline]
    pub(crate) fn get_instruction(&self, protoid: usize, pc: usize) -> u32 {
        self.protos[protoid].code[pc]
    }
    pub(crate) fn borrow_mut_instruction(&mut self, protoid: usize, pc: usize) -> &mut u32 {
        &mut self.protos[protoid].code[pc]
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
    pub(crate) fn push_number(&mut self, value: LuaFloat) {
        self.stack.push(TValue::Float(value));
    }
    pub(crate) fn push_integer(&mut self, value: LuaInteger) {
        self.stack.push(TValue::Integer(value));
    }
    pub(crate) fn push_boolean(&mut self, value: bool) {
        self.stack.push(TValue::Boolean(value));
    }
    pub(crate) fn push_nil(&mut self) {
        self.stack.push(TValue::Nil);
    }
    pub(crate) fn call(
        &mut self,
        nargs: usize,
        nresults: i32,
        ctx: u32,
        k: Option<LuaRustFunction>,
    ) -> Result<(), LuaError> {
        self.api_check_nelems(nargs + 1);
        self.check_results(nargs, nresults);
        let len = self.stack.len();
        let func = len - nargs - 1;
        if k.is_some() && self.nny == 0 {
            // need to prepare continuation
            self.base_ci[self.ci].k = k; // save continuation
            self.base_ci[self.ci].ctx = ctx; // save context
            self.dcall(func, nresults)?; // do the call
        } else {
            // no continuation or no yieldable
            self.dcall_no_yield(func, nresults)?;
        }
        self.adjust_results(nresults);
        Ok(())
    }

    #[inline]
    fn api_check_nelems(&self, n: usize) {
        debug_assert!(n as i32 <= self.stack.len() as i32 - self.base_ci[self.ci].base as i32);
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
        let mut cl = RClosure::new(func);
        for _ in 0..nup_values {
            cl.upvalues.push(self.stack.pop().unwrap());
        }
        self.stack.push(TValue::from(cl));
    }
    pub(crate) fn get_closure_ref(&self, func: usize) -> ClosureRef {
        if let TValue::Function(cl) = &self.stack[func] {
            Rc::clone(&cl)
        } else {
            unreachable!()
        }
    }
    pub(crate) fn run_error(&mut self, msg: &str) -> Result<(), LuaError> {
        let fullmsg = {
            let ci = &self.base_ci[self.ci];
            let rcl = self.get_closure_ref(ci.func);
            let pc = self.base_ci[self.ci].saved_pc;
            let proto = &self.protos[rcl.borrow().get_proto_id()];
            let line = proto.lineinfo[pc];
            let chunk_id = &proto.source;
            format!("{}:{} {}", chunk_id, line, msg)
        };
        self.stack.push(TValue::from(&fullmsg[..]));
        Err(LuaError::RuntimeError)
    }

    pub(crate) fn adjust_results(&mut self, nresults: i32) {
        if nresults == LUA_MULTRET && self.stack.len() > self.base_ci[self.ci].top {
            self.base_ci[self.ci].top = self.stack.len();
        }
    }

    pub(crate) fn push_value(&mut self, index: isize) {
        self.stack.push(self.index2adr(index).clone());
    }

    pub(crate) fn push_literal(&mut self, value: &str) {
        self.stack.push(TValue::from(value));
    }

    pub(crate) fn create_table(&mut self) {
        self.stack.push(TValue::new_table());
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
                                    Some(idx) => {
                                        if idx == stack.len() {
                                            stack.push(TValue::Nil);
                                        } else {
                                            stack[idx] = TValue::Nil;
                                        }
                                    }
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
                                    Some(idx) => {
                                        if idx == stack.len() {
                                            stack.push(TValue::Nil);
                                        } else {
                                            stack[idx] = TValue::Nil;
                                        }
                                    }
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

    pub(crate) fn is_index_valid(&self, index: isize) -> bool {
        let len = self.stack.len() as isize;
        (index >= 0 && index < len) || (index < 0 && index >= -len) || index <= LUA_REGISTRYINDEX
    }
    pub(crate) fn index2adr(&self, index: isize) -> TValue {
        let func = self.base_ci[self.ci].func;
        if index > 0 {
            // positive index in the stack
            let index = index as usize + func;
            debug_assert!(index < self.base_ci[self.ci].top);
            if index >= self.stack.len() {
                return TValue::Nil;
            }
            self.stack[index].clone()
        } else if index > LUA_REGISTRYINDEX {
            // negative index in the stack (count from top)
            let index = (-index) as usize;
            debug_assert!(index != 0 && index + func <= self.stack.len());
            self.stack[self.stack.len() - index].clone()
        } else {
            match index {
                LUA_REGISTRYINDEX => self.g.registry.clone(),
                _ => {
                    // upvalues
                    let index = (LUA_REGISTRYINDEX - index) as usize;
                    debug_assert!(index <= MAX_UPVAL + 1);
                    // TODO light rust function
                    let stkid = self.base_ci[self.ci].func;
                    if index <= self.get_closure_nupvalues(stkid) {
                        return self.get_rust_closure_upvalue(stkid, index - 1).clone();
                    }
                    TValue::Nil
                }
            }
        }
    }

    pub(crate) fn pop_stack(&mut self, count: usize) {
        let newlen = self.stack.len() - count;
        self.stack.truncate(newlen);
    }

    /// Finishes a function call: calls hook if necessary, removes CallInfo,
    /// moves current number of results to proper place; returns 0 iff call
    /// wanted multiple (variable number of) results.
    pub(crate) fn poscall(&mut self, first_result: StkId, nres: usize) -> bool {
        // TODO hooks
        let ci = &self.base_ci[self.ci];
        // res == final position of 1st result
        let res = ci.func;
        let wanted = ci.nresults;

        self.base_ci.pop(); // back to caller
        self.ci -= 1;
        // move results to proper place
        self.move_results(first_result, res, nres, wanted)
    }

    /// Given 'nres' results at 'firstResult', move 'wanted' of them to 'res'.
    /// Handle most typical cases (zero results for commands, one result for
    /// expressions, multiple results for tail calls/single parameters)
    /// separated.
    fn move_results(&mut self, first_result: StkId, res: StkId, nres: usize, wanted: i32) -> bool {
        match wanted {
            0 => (), // nothing to move
            1 => {
                if nres == 0 {
                    // no results?
                    self.set_stack_from_value(res, TValue::Nil); // adjust with nil
                } else {
                    self.set_stack_from_idx(res, first_result); // move it to proper place
                }
            }
            LUA_MULTRET => {
                for i in 0..nres {
                    // move all results to correct place
                    self.set_stack_from_idx(res + i, first_result + i);
                }
                self.stack.resize(res + nres, TValue::Nil);
                return false; //  wanted == LUA_MULTRET
            }
            _ => {
                if wanted <= nres as i32 {
                    // enough results?
                    for i in 0..wanted as usize {
                        // move wanted results to correct place
                        self.set_stack_from_idx(res + i, first_result + i);
                    }
                } else {
                    // not enough results; use all of them plus nils
                    for i in 0..nres {
                        // move all results to correct place
                        self.set_stack_from_idx(res + i, first_result + i);
                    }
                    for i in nres..wanted as usize {
                        // complete wanted number of results
                        self.set_stack_from_value(res + i, TValue::Nil);
                    }
                }
            }
        }
        self.stack.resize(res + wanted as usize, TValue::Nil); // top points after the last result

        true
    }

    pub(crate) fn find_upval(&mut self, func: usize, level: usize) -> UpVal {
        let cl = self.get_closure_ref(func);
        let mut rcl = cl.borrow_mut();
        if let Closure::Lua(cl) = &mut *rcl {
            let mut index = 0;
            for (i, val) in cl.upvalues.iter().enumerate().rev() {
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
                value: self.stack[level as usize].clone(),
            };
            cl.upvalues.insert(index, uv.clone());
            return uv;
        }
        unreachable!()
    }

    /// convert stack[obj] to a number into stack[dst], return the number value
    pub(crate) fn to_number(
        stack: &mut [TValue],
        obj: StkId,
        dst: Option<StkId>,
    ) -> Option<LuaFloat> {
        match &stack[obj] {
            TValue::Integer(n) => Some(*n as LuaFloat),
            TValue::Float(n) => Some(*n),
            TValue::String(s) => match str2d(s) {
                Some(n) => {
                    if let Some(dst) = dst {
                        stack[dst] = TValue::Float(n);
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

    pub(crate) fn _get_lua_closure_upval_desc(&self, func: usize, upval_id: usize) -> UpVal {
        let cl = self.get_closure_ref(func);
        let cl = cl.borrow();
        cl.get_lua_upval_desc(upval_id)
    }
    pub(crate) fn get_lua_closure_protoid(&self, func: usize) -> usize {
        let cl = self.get_closure_ref(func);
        let cl = cl.borrow();
        cl.get_proto_id()
    }
    pub(crate) fn get_lua_closure_upvalue(&self, func: usize, upval_id: usize) -> TValue {
        let cl = self.get_closure_ref(func);
        let cl = cl.borrow();
        cl.get_lua_upvalue(upval_id)
    }
    pub(crate) fn get_lua_closure_upval(&self, func: usize, upval_id: usize) -> UpVal {
        let cl = self.get_closure_ref(func);
        let cl = cl.borrow();
        cl.borrow_lua_upval(upval_id).clone()
    }
    pub(crate) fn set_lua_closure_upvalue(&mut self, func: usize, upval_id: usize, value: TValue) {
        let cl = self.get_closure_ref(func);
        let mut cl = cl.borrow_mut();
        cl.set_lua_upval_value(upval_id, value);
    }

    fn get_closure_nupvalues(&self, func: usize) -> usize {
        let cl = self.get_closure_ref(func);
        let cl = cl.borrow();
        cl.get_nupvalues()
    }

    fn get_rust_closure_upvalue(&self, func: usize, upval_id: usize) -> TValue {
        let cl = self.get_closure_ref(func);
        let cl = cl.borrow();
        cl.get_rust_upvalue(upval_id)
    }

    #[inline]
    pub(crate) fn set_or_push(&mut self, index: usize, val: TValue) {
        if index == self.stack.len() {
            self.stack.push(val);
        } else {
            self.stack[index] = val;
        }
    }

    fn init_registry(&mut self) {
        if let TValue::Table(tref) = &self.g.registry {
            let mut t = tref.borrow_mut();
            // registry[LUA_RIDX_GLOBALS] = table of globals
            t.set_num(LUA_RIDX_GLOBALS, TValue::new_table());
        }
    }

    pub(crate) fn get_global_table(&self) -> TValue {
        if let TValue::Table(tref) = &self.g.registry {
            let mut t = tref.borrow_mut();
            t.get_num(LUA_RIDX_GLOBALS).clone()
        } else {
            unreachable!()
        }
    }

    #[inline]
    pub(crate) fn set_stack_from_idx(&mut self, dest: usize, source: usize) {
        let value = self.stack[source].clone();
        self.set_stack_from_value(dest, value);
    }

    #[inline]
    pub(crate) fn set_stack_from_value(&mut self, dest: usize, value: TValue) {
        if dest == self.stack.len() {
            self.stack.push(value);
        } else {
            self.stack[dest] = value;
        }
    }

    pub(crate) fn add_prototype<T>(
        &mut self,
        lex: &LexState<T>,
        source: &str,
        line: usize,
    ) -> ProtoId {
        let mut proto = Proto::new(source);
        let cur_proto = lex.borrow_fs(None).f;
        proto.linedefined = line;
        let id = self.protos.len();
        self.protos.push(proto);
        self.protos[cur_proto].p.push(id);
        id
    }
}

fn f_luaopen(state: &mut LuaState, _: ()) -> Result<i32, LuaError> {
    state.init_stack();
    state.init_registry();
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
