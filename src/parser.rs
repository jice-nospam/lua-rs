//! Lua Parser

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    api::LuaError,
    ldo::SParser,
    lex::{LexState, Reserved, SemInfo, Token},
    luaK::{
        self, code_abc, code_abx, code_asbx, exp2anyreg, exp2nextreg, exp2rk, fix_line, indexed,
        patch_list, patch_to_here, reserve_regs, ret, set_list, set_mult_ret, store_var, op_self,
    },
    luaconf::LUAI_MAXVARS,
    object::{int2fb, LocVar, Proto, TValue},
    opcodes::{
        get_arg_a, set_arg_b, set_arg_c, set_opcode, OpCode, LFIELDS_PER_FLUSH, MAXARG_BX, NO_JUMP,
        NO_REG,
    },
    state::LuaState,
    LuaNumber, LUA_MULTRET,
};

#[derive(Clone, Copy)]
pub(crate) enum BinaryOp {
    Add = 0,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Concat,
    Ne,
    Eq,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

struct BinaryPriority {
    left: usize,
    right: usize,
}
const BINARY_OP_PRIO: [BinaryPriority; 15] = [
    BinaryPriority { left: 6, right: 6 },  // Add
    BinaryPriority { left: 6, right: 6 },  // Sub
    BinaryPriority { left: 7, right: 7 },  // Mul
    BinaryPriority { left: 7, right: 7 },  // Div
    BinaryPriority { left: 7, right: 7 },  // Mod
    BinaryPriority { left: 10, right: 9 }, // Pow
    BinaryPriority { left: 5, right: 4 },  // Concat
    BinaryPriority { left: 3, right: 3 },  // Ne
    BinaryPriority { left: 3, right: 3 },  // Eq
    BinaryPriority { left: 3, right: 3 },  // Lt
    BinaryPriority { left: 3, right: 3 },  // Le
    BinaryPriority { left: 3, right: 3 },  // Gt
    BinaryPriority { left: 3, right: 3 },  // Ge
    BinaryPriority { left: 2, right: 2 },  // And
    BinaryPriority { left: 1, right: 1 },  // Or
];

pub(crate) enum UnaryOp {
    Minus,
    Not,
    Len,
}

/// priority for unary operators
const UNARY_PRIORITY: usize = 8;

#[derive(Default)]
pub struct UpValDesc {
    k: ExpressionKind,
    info: i32,
    name: String,
}

/// nodes for block list (list of active blocks)
pub(crate) struct BlockCnt {
    /// list of jumps out of this loop
    breaklist: i32,
    /// # active locals outside the breakable structure
    nactvar: usize,
    /// true if some variable in the block is an upvalue
    upval: bool,
    /// true if `block' is a loop
    is_breakable: bool,
}

impl BlockCnt {
    fn new(is_breakable: bool, nactvar: usize) -> Self {
        Self {
            breaklist: NO_JUMP,
            nactvar,
            upval: false,
            is_breakable,
        }
    }
}

#[derive(Default)]
struct ConstructorControl {
    /// last list item read
    v: ExpressionDesc,
    /// table descriptor
    //t: Option<&'a mut ExpressionDesc>,
    ///  total number of `record' elements
    nh: usize,
    /// total number of array elements
    na: usize,
    ///  number of array elements pending to be stored
    to_store: usize,
}

/// state needed to generate code for a given function
pub struct FuncState {
    /// current function header
    pub f: Proto,
    /// table to find (and reuse) constants in `f.k'
    pub h: HashMap<TValue, usize>,
    /// enclosing function
    prev: Option<usize>,
    /// chain of current blocks
    pub(crate) bl: Vec<BlockCnt>,
    /// `pc' of last `jump target'
    pub last_target: i32,
    /// list of pending jumps to `pc'
    pub jpc: i32,
    /// first free register
    pub freereg: usize,
    /// number of active local variables
    pub nactvar: usize,
    /// upvalues
    pub upvalues: Vec<UpValDesc>,
    /// declared-variable stack
    pub actvar: [usize; LUAI_MAXVARS],
}

impl FuncState {
    pub(crate) fn new(source: &str) -> Self {
        let mut proto = Proto::new(source);
        // registers 0/1 are always valid
        proto.maxstacksize = 2;
        // TODO proto.source = lex_state.source
        Self {
            f: proto,
            h: HashMap::new(),
            prev: None,
            bl: Vec::new(),
            last_target: NO_JUMP,
            jpc: NO_JUMP,
            freereg: 0,
            nactvar: 0,
            upvalues: Vec::new(),
            actvar: [0; LUAI_MAXVARS],
        }
    }

    pub(crate) fn get_break_upval(&self) -> Option<(bool, usize)> {
        let mut upval=false;
        for (i,bl) in self.bl.iter().enumerate().rev() {
            if ! bl.is_breakable {
                upval = upval || bl.upval;
            } else {
                return Some((upval,i));
            }
        }
        None
    }

    fn search_var(&self, name: &str) -> Option<usize> {
        (0..self.nactvar)
            .rev()
            .find(|&i| name == self.get_loc_var(i).name)
    }

    fn mark_upval(&self, _v: usize) {
        todo!()
    }

    fn get_loc_var(&self, i: usize) -> &LocVar {
        &self.f.locvars[self.actvar[i]]
    }

    pub(crate) fn add_constant(&mut self, key: TValue, value: TValue) -> usize {
        match self.h.get(&key) {
            Some(i) => *i,
            None => {
                let kid = self.f.k.len();
                self.h.insert(key, self.f.k.len());
                self.f.k.push(value);
                kid
            }
        }
    }

    pub fn string_constant(&mut self, value: &str) -> usize {
        let tvalue = TValue::from(value);
        self.add_constant(tvalue.clone(), tvalue)
    }

    pub fn number_constant(&mut self, value: LuaNumber) -> usize {
        let tvalue = TValue::Number(value);
        self.add_constant(tvalue.clone(), tvalue)
    }

    pub(crate) fn next_pc(&self) -> i32 {
        self.f.code.len() as i32
    }
}

pub fn parser<T>(state: &mut LuaState, parser: &mut SParser<T>) -> Result<Proto, LuaError> {
    let lex_state = Rc::new(RefCell::new(LexState::new(
        parser.z.take().unwrap(),
        &parser.name,
    )));
    // read the first character in the stream
    lex_state.borrow_mut().next_char(state);
    let mut lstate = lex_state.borrow_mut();
    // main func. is always vararg
    lstate.borrow_mut_fs(None).f.is_vararg = true;
    lstate.next_token(state)?;
    chunk(&mut lstate, state)?;
    lstate.check_eos(state)?;
    close_func(&mut *lstate, state)?;
    Ok(lstate.borrow_mut_fs(None).f.clone())
}

fn close_func<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    remove_vars(lex, 0);
    luaK::ret(lex, state, 0, 0)?; // final return
    let fs = lex.borrow_mut_fs(None);
    fs.f.nups = fs.upvalues.len();
    Ok(())
}

fn remove_vars<T>(lex: &mut LexState<T>, to_level: usize) {
    let fs = lex.borrow_mut_fs(None);
    while fs.nactvar > to_level {
        fs.nactvar -= 1;
        borrow_mut_locvar(fs, fs.nactvar).end_pc = fs.next_pc() as usize;
    }
}

fn borrow_mut_locvar(fs: &mut FuncState, nactvar: usize) -> &mut LocVar {
    &mut fs.f.locvars[fs.actvar[nactvar]]
}

fn block_follow(t: Option<u32>) -> bool {
    if t.is_none() {
        return true;
    }
    matches!(
        Reserved::try_from(t.unwrap()),
        Ok(Reserved::Else) | Ok(Reserved::ElseIf) | Ok(Reserved::End) | Ok(Reserved::Until)
    )
}

/// parse next chunk
fn chunk<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let mut is_last = false;
    enter_level(lex, state)?;
    while !is_last && !block_follow(lex.t.as_ref().map(|t| t.token)) {
        is_last = statement(lex, state)?;
        test_next(lex, state, ';' as u32)?;
        {
            let fs = lex.borrow_mut_fs(None);
            debug_assert!(fs.f.maxstacksize >= fs.freereg && fs.freereg >= fs.nactvar);
            // free registers
            fs.freereg = fs.nactvar;
        }
    }
    leave_level(lex, state);
    Ok(())
}

fn leave_level<T>(_lex: &mut LexState<T>, state: &mut LuaState) {
    state.n_rcalls -= 1;
}

fn test_next<T>(lex: &mut LexState<T>, state: &mut LuaState, arg: u32) -> Result<bool, LuaError> {
    match &lex.t {
        Some(t) if t.token == arg => {
            lex.next_token(state)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn enter_level<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    state.n_rcalls += 1;
    if state.n_rcalls >= crate::luaconf::LUAI_MAXRCALLS {
        return lex.lex_error(state, "chunk has too many syntax levels", None);
    }
    Ok(())
}

fn statement<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<bool, LuaError> {
    let line = lex.linenumber;
    if let Some(ref t) = lex.t {
        match Reserved::try_from(t.token) {
            Ok(Reserved::If) => {
                if_stat(lex, state, line)?;
                return Ok(false);
            }
            Ok(Reserved::While) => {
                while_stat(lex, line)?;
                return Ok(false);
            }
            Ok(Reserved::Do) => {
                lex.next_token(state)?;
                block(lex, state)?;
                check_match(lex, state, Reserved::End as u32, Reserved::Do as u32, line)?;
                return Ok(false);
            }
            Ok(Reserved::For) => {
                for_stat(lex, state, line)?;
                return Ok(false);
            }
            Ok(Reserved::Repeat) => {
                repeat_stat(lex, state, line)?;
                return Ok(false);
            }
            Ok(Reserved::Function) => {
                func_stat(lex, state, line)?;
                return Ok(false);
            }
            Ok(Reserved::Local) => {
                lex.next_token(state)?;
                if test_next(lex, state, Reserved::Function as u32)? {
                    local_func(lex, state)?;
                } else {
                    local_stat(lex, state)?;
                }
                return Ok(false);
            }
            //  stat -> retstat
            Ok(Reserved::Return) => {
                return_stat(lex, state, line)?;
                return Ok(true); // must be last statement
            }
            // stat -> breakstat
            Ok(Reserved::Break) => {
                lex.next_token(state)?; // skip BREAK
                break_stat(lex, state)?;
                return Ok(true); // must be last statement
            }
            _ => {
                expr_stat(lex, state)?;
                return Ok(false);
            }
        }
    }
    expr_stat(lex, state)?;
    Ok(false)
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum ExpressionKind {
    /// no value
    #[default]
    Void = 0,
    Nil,
    True,
    False,
    /// info = index of constant in `k'
    Constant,
    /// nval = numerical value
    NumberConstant,
    /// info = local register
    LocalRegister,
    /// info = index of upvalue in `upvalues'
    UpValue,
    /// info = index of table; aux = index of global name in `k'
    GlobalVar,
    /// info = table register; aux = index register (or `k')
    Indexed,
    /// info = instruction pc
    Jump,
    /// info = instruction pc
    Relocable,
    /// info = result register
    NonRelocable,
    /// info = instruction pc
    Call,
    /// info = instruction pc
    VarArg,
}

#[derive(Default, Clone)]
pub struct ExpressionDesc {
    pub k: ExpressionKind,
    pub info: i32,
    pub aux: i32,
    pub nval: LuaNumber,
    /// patch list of `exit when true'
    pub t: i32,
    /// patch list of `exit when false'
    pub f: i32,
}
impl ExpressionDesc {
    pub(crate) fn init(&mut self, kind: ExpressionKind, info: i32) {
        self.k = kind;
        self.info = info;
        self.t = NO_JUMP;
        self.f = NO_JUMP;
    }

    fn index_upvalue(&mut self, fs: &mut FuncState, name: &str) -> i32 {
        for (i, uv) in fs.upvalues.iter().enumerate() {
            if uv.k == self.k && uv.info == self.info {
                return i as i32;
            }
        }
        fs.upvalues.push(UpValDesc {
            k: self.k,
            info: self.info,
            name: name.to_owned(),
        });
        fs.f.nups += 1;
        debug_assert!(fs.f.nups == fs.upvalues.len());
        fs.f.nups as i32 - 1
    }

    pub(crate) fn is_numeral(&self) -> bool {
        self.k == ExpressionKind::NumberConstant && self.t == NO_JUMP && self.f == NO_JUMP
    }
}

#[derive(Default)]
struct LHSAssignment {
    prev: Option<Box<LHSAssignment>>,
    /// variable (global, local, upvalue, or indexed)
    v: ExpressionDesc,
}

/// stat -> func | assignment
fn expr_stat<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let mut lhs = LHSAssignment::default();
    primary_expr(lex, state, &mut lhs.v)?;
    if lhs.v.k == ExpressionKind::Call {
        // statement = func
        // call statement uses no results
        set_arg_c(lex.borrow_mut_code(lhs.v.info as usize), 1);
    } else {
        // statement = assignment
        let mut vlhs = Vec::new();
        vlhs.push(lhs);
        assignment(lex, state, &mut vlhs, 1)?;
    }
    Ok(())
}

fn assignment<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    lhs: &mut Vec<LHSAssignment>,
    nvars: usize,
) -> Result<(), LuaError> {
    let mut exp = ExpressionDesc::default();
    if (lhs.last().unwrap().v.k as u32) < (ExpressionKind::LocalRegister as u32)
        || (lhs.last().unwrap().v.k as u32) > (ExpressionKind::Indexed as u32)
    {
        return lex.syntax_error(state, "syntax error");
    }
    if test_next(lex, state, ',' as u32)? {
        // assignment -> `,' primaryexp assignment
        let mut nv = LHSAssignment::default();
        primary_expr(lex, state, &mut nv.v)?;
        let nvk = nv.v.k;
        lhs.push(nv);
        if let ExpressionKind::LocalRegister = nvk {
            check_conflict(lex, &exp, lhs)?;
        }
        assignment(lex, state, lhs, nvars + 1)?;
        lhs.pop();
    } else {
        // assignment -> `=' explist1
        check_next(lex, state, '=' as u32)?;
        let nexps = exp_list1(lex, state, &mut exp)?;
        if nexps != nvars {
            adjust_assign(lex, state, nvars, nexps, &mut exp)?;
            if nexps > nvars {
                lex.borrow_mut_fs(None).freereg -= nexps - nvars; // remove extra values
            }
        } else {
            luaK::set_one_ret(lex, &mut exp); // close last expression
            luaK::store_var(lex, state, &lhs.last().unwrap().v, &mut exp)?;
            return Ok(());
        }
    }
    exp.init(
        ExpressionKind::NonRelocable,
        lex.borrow_fs(None).freereg as i32 - 1,
    ); // default assignment
    luaK::store_var(lex, state, &lhs.last().unwrap().v, &mut exp)?;
    Ok(())
}

fn check_next<T>(lex: &mut LexState<T>, state: &mut LuaState, token: u32) -> Result<(), LuaError> {
    check(lex, state, token)?;
    lex.next_token(state)
}

fn check_conflict<T>(
    _lex: &mut LexState<T>,
    _e: &ExpressionDesc,
    _lhs: &mut [LHSAssignment],
) -> Result<(), LuaError> {
    todo!()
}

/// primaryexp -> prefixexp { `.' NAME | `[' exp `]' | `:' NAME funcargs | funcargs }
fn primary_expr<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    prefix_expr(lex, state, exp)?;
    if lex.t.is_none() {
        return Ok(());
    }
    while let Some(t) = lex.t.clone() {
        match t.token {
            c if c == u32::from('.') => {
                // field
                field(lex, state, exp)?;
            }
            c if c == u32::from('[') => {
                // `[' exp1 `]'
                exp2anyreg(lex, state, exp)?;
                let mut key = ExpressionDesc::default();
                yindex(lex, state, &mut key)?;
                luaK::indexed(lex, state, exp, &mut key)?;
            }
            c if c == u32::from(':') => {
                // `:' NAME funcargs
                lex.next_token(state)?;
                let mut key=ExpressionDesc::default();
                check_name(lex, state, &mut key)?;
                op_self(lex, state, exp, &mut key)?;
                func_args(lex, state, exp)?;
            }
            c if c == u32::from('(') || c == u32::from('{') || c == Reserved::String as u32 => {
                luaK::exp2nextreg(lex, state, exp)?;
                func_args(lex, state, exp)?;
            }
            _ => return Ok(()),
        }
    }
    Ok(())
}

/// index -> '[' expr ']'
fn yindex<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    v: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    lex.next_token(state)?; // skip the '['
    expr(lex, state, v)?;
    luaK::exp2val(lex, state, v)?;
    check_next(lex, state, ']' as u32)
}

fn func_args<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    let line = lex.linenumber;
    let mut args = ExpressionDesc::default();
    match &lex.t.clone() {
        Some(t) if t.token == '(' as u32 => {
            // funcargs -> `(' [ explist1 ] `)'
            if line != lex.lastline {
                return lex.syntax_error(state, "ambiguous syntax (function call x new statement)");
            }
            lex.next_token(state)?;
            if lex.is_token(')' as u32) {
                // arg list is empty
                args.k = ExpressionKind::Void;
            } else {
                exp_list1(lex, state, &mut args)?;
                luaK::set_mult_ret(lex, state, &mut args)?;
            }
            check_match(lex, state, ')' as u32, '(' as u32, line)?;
        }
        Some(t) if t.token == '{' as u32 => {
            // funcargs -> constructor
            constructor(lex, state, &mut args)?;
        }
        Some(t) if t.token == Reserved::String as u32 => {
            // funcargs -> STRING
            if let SemInfo::String(s) = &t.seminfo {
                code_string(lex, &mut args, s);
            } else {
                unreachable!()
            }
            lex.next_token(state)?;
        }
        _ => {
            return lex.syntax_error(state, "function arguments expected");
        }
    }
    debug_assert!(exp.k == ExpressionKind::NonRelocable);
    let base = exp.info as u32; // base register for call
    let nparams = if has_mult_ret(args.k) {
        LUA_MULTRET as u32 // open call
    } else {
        if args.k != ExpressionKind::Void {
            exp2nextreg(lex, state, &mut args)?;
        }
        lex.borrow_fs(None).freereg as u32 - (base + 1)
    };
    exp.init(
        ExpressionKind::Call,
        code_abc(
            lex,
            state,
            OpCode::Call as u32,
            base as i32,
            nparams as i32 + 1,
            2,
        )? as i32,
    );
    luaK::fix_line(lex, line);
    lex.borrow_mut_fs(None).freereg = base as usize + 1; // call remove function and arguments and leaves
                                                         // (unless changed) one result
    Ok(())
}

#[inline]
fn has_mult_ret(k: ExpressionKind) -> bool {
    k == ExpressionKind::Call || k == ExpressionKind::VarArg
}

/// set expression as a astring constant
fn code_string<T>(lex: &mut LexState<T>, exp: &mut ExpressionDesc, val: &str) {
    exp.init(
        ExpressionKind::Constant,
        lex.borrow_mut_fs(None).string_constant(val) as i32,
    );
}

fn constructor<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    let line = lex.linenumber;
    let pc = code_abc(lex, state, OpCode::NewTable as u32, 0, 0, 0)?;
    let mut cc = ConstructorControl::default();
    //cc.t = Some(exp);
    exp.init(ExpressionKind::Relocable, pc as i32);
    exp2nextreg(lex, state, exp)?;
    check_next(lex, state, '{' as u32)?;
    loop {
        debug_assert!(cc.v.k == ExpressionKind::Void || cc.to_store > 0);
        if lex.is_token('}' as u32) {
            break;
        }
        close_list_field(lex, state, &mut cc, exp)?;
        match &lex.t {
            Some(t) if t.token == Reserved::Name as u32 => {
                //  may be listfields or recfields
                lex.look_ahead(state)?;
                if !lex.is_lookahead_token('=' as u32) {
                    // expression ?
                    list_field(lex, state, &mut cc)?;
                } else {
                    rect_field(lex, state, &mut cc, exp)?;
                }
            }
            Some(t) if t.token == '[' as u32 => {
                // constructor_item -> recfield
                rect_field(lex, state, &mut cc, exp)?;
            }
            _ => {
                // constructor_part -> listfield
                list_field(lex, state, &mut cc)?;
            }
        }
        if !test_next(lex, state, ',' as u32)? && !test_next(lex, state, ';' as u32)? {
            break;
        }
    }
    check_match(lex, state, '}' as u32, '{' as u32, line)?;
    last_list_field(lex, state, &mut cc, exp)?;
    set_arg_b(lex.borrow_mut_code(pc as usize), int2fb(cc.na as u32)); // set initial array size
    set_arg_c(lex.borrow_mut_code(pc as usize), int2fb(cc.nh as u32)); // set initial table size
    Ok(())
}

fn close_list_field<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    cc: &mut ConstructorControl,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    if cc.v.k == ExpressionKind::Void {
        return Ok(()); // there is no list item
    }
    exp2nextreg(lex, state, &mut cc.v)?;
    cc.v.k = ExpressionKind::Void;
    if cc.to_store == LFIELDS_PER_FLUSH as usize {
        set_list(lex, state, exp.info, cc.na as i32, cc.to_store as i32)?; // flush
        cc.to_store = 0; // no more items pending
    }
    Ok(())
}

/// recfield -> (NAME | `['exp1`]') = exp1
fn rect_field<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    cc: &mut ConstructorControl,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    let reg = lex.borrow_fs(None).freereg;
    let mut key = ExpressionDesc::default();
    let mut val = ExpressionDesc::default();
    if lex.is_token(Reserved::Name as u32) {
        if cc.nh > (i32::MAX - 2) as usize {
            return lex.error_limit(state, i32::MAX as usize - 2, "items in a constructor");
        }
        check_name(lex, state, &mut key)?;
    } else {
        // lex.t.token == '['
        yindex(lex, state, &mut key)?;
    }
    cc.nh += 1;
    check_next(lex, state, '=' as u32)?;
    let rkkey = exp2rk(lex, state, &mut key)? as i32;
    expr(lex, state, &mut val)?;
    let c = exp2rk(lex, state, &mut val)? as i32;
    code_abc(lex, state, OpCode::SetTable as u32, exp.info, rkkey, c)?;
    lex.borrow_mut_fs(None).freereg = reg; // free registers
    Ok(())
}

fn check_name<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    key: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    let name = str_checkname(lex, state)?;
    code_string(lex, key, &name);
    Ok(())
}

fn list_field<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    cc: &mut ConstructorControl,
) -> Result<(), LuaError> {
    expr(lex, state, &mut cc.v)?;
    if cc.na > MAXARG_BX {
        return lex.error_limit(state, MAXARG_BX, "items in a constructor");
    }
    cc.na += 1;
    cc.to_store += 1;
    Ok(())
}

fn last_list_field<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    cc: &mut ConstructorControl,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    if cc.to_store == 0 {
        return Ok(());
    }
    if has_mult_ret(cc.v.k) {
        set_mult_ret(lex, state, &mut cc.v)?;
        luaK::set_list(lex, state, exp.info, cc.na as i32, LUA_MULTRET)?;
        cc.na -= 1; // do not count last expression (unknown number of elements)
    } else {
        if cc.v.k != ExpressionKind::Void {
            exp2nextreg(lex, state, &mut cc.v)?;
        }
        luaK::set_list(lex, state, exp.info, cc.na as i32, cc.to_store as i32)?;
    }
    Ok(())
}

/// explist1 -> expr { `,' expr }
fn exp_list1<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<usize, LuaError> {
    let mut n = 1;
    expr(lex, state, exp)?;
    while test_next(lex, state, ',' as u32)? {
        exp2nextreg(lex, state, exp)?;
        expr(lex, state, exp)?;
        n += 1;
    }
    Ok(n)
}

/// field -> ['.' | ':'] NAME
fn field<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    let mut key = ExpressionDesc::default();
    exp2anyreg(lex, state, exp)?;
    lex.next_token(state)?; // skip the dot or colon
    check_name(lex, state, &mut key)?;
    indexed(lex, state, exp, &mut key)
}

/// prefixexp -> NAME | '(' expr ')'
fn prefix_expr<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    match &lex.t {
        Some(t) if t.token == u32::from('(') => {
            let line = lex.linenumber;
            lex.next_token(state)?;
            expr(lex, state, exp)?;
            check_match(lex, state, u32::from(')'), u32::from('('), line)?;
            luaK::discharge_vars(lex, state, exp)?;
            Ok(())
        }
        Some(t) if t.token == Reserved::Name as u32 => {
            single_var(lex, state, exp)?;
            Ok(())
        }
        _ => lex.syntax_error(state, "unexpected symbol"),
    }
}

fn single_var<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    let name = str_checkname(lex, state)?;
    if single_var_aux(lex, lex.fs, &name, exp, true)? == ExpressionKind::GlobalVar {
        // info points to global name
        exp.info = lex.borrow_mut_fs(None).string_constant(&name) as i32;
    }
    Ok(())
}

fn single_var_aux<T>(
    lex: &mut LexState<T>,
    fsid: usize,
    name: &str,
    exp: &mut ExpressionDesc,
    base: bool,
) -> Result<ExpressionKind, LuaError> {
    // look up at current level
    if let Some(v) = lex.borrow_fs(Some(fsid)).search_var(name) {
        exp.init(ExpressionKind::LocalRegister, v as i32);
        if !base {
            // local will be used as an upval
            lex.borrow_mut_fs(Some(fsid)).mark_upval(v);
        }
        Ok(ExpressionKind::LocalRegister)
    } else {
        // not found at current level; try upper one
        match lex.borrow_fs(Some(fsid)).prev {
            None => {
                // no more levels. var is global
                exp.init(ExpressionKind::GlobalVar, NO_REG as i32);
                Ok(ExpressionKind::GlobalVar)
            }
            Some(prev_fsid) => {
                if let Ok(ExpressionKind::GlobalVar) =
                    single_var_aux(lex, prev_fsid, name, exp, base)
                {
                    return Ok(ExpressionKind::GlobalVar);
                }
                exp.info = exp.index_upvalue(lex.borrow_mut_fs(Some(fsid)), name);
                exp.k = ExpressionKind::UpValue;
                Ok(ExpressionKind::UpValue)
            }
        }
    }
}

fn str_checkname<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<String, LuaError> {
    check(lex, state, Reserved::Name as u32)?;
    let name = if let Some(ref t) = lex.t {
        if let SemInfo::String(s) = &t.seminfo {
            s.to_owned()
        } else {
            unreachable!()
        }
    } else {
        unreachable!()
    };
    lex.next_token(state)?;
    Ok(name)
}

fn check<T>(lex: &mut LexState<T>, state: &mut LuaState, token: u32) -> Result<(), LuaError> {
    match &lex.t {
        Some(t) if t.token == token => Ok(()),
        _ => lex.syntax_error(state, &format!("'{}' expected", lex.token_2_txt(token))),
    }
}

fn expr<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    subexpr(lex, state, exp, 0)?;
    Ok(())
}

/// subexpr -> (simpleexp | unop subexpr) { binop subexpr }
/// where `binop' is any binary operator with a priority higher than `limit'
fn subexpr<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
    limit: usize,
) -> Result<Option<BinaryOp>, LuaError> {
    enter_level(lex, state)?;
    if let Some(uop) = unary_op(&lex.t) {
        lex.next_token(state)?;
        subexpr(lex, state, exp, UNARY_PRIORITY)?;
        luaK::prefix(lex, state, uop, exp)?;
    } else {
        simple_exp(lex, state, exp)?;
    }
    // expand while operators have priorities higher than `limit'
    let mut oop = binary_op(&lex.t);
    while let Some(op) = oop {
        if BINARY_OP_PRIO[op as usize].left <= limit {
            break;
        }
        lex.next_token(state)?;
        luaK::infix(lex, state, op, exp)?;
        let mut exp2 = ExpressionDesc::default();
        let nextop = subexpr(lex, state, &mut exp2, BINARY_OP_PRIO[op as usize].right)?;
        luaK::postfix(lex, state, op, exp, &mut exp2)?;
        oop = nextop;
    }
    leave_level(lex, state);
    Ok(oop)
}

/// simple_exp -> NUMBER | STRING | NIL | true | false | ... |
/// constructor | FUNCTION body | primaryexp
fn simple_exp<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    match &lex.t.clone() {
        Some(t) if t.token == Reserved::Number as u32 => {
            if let SemInfo::Number(val) = t.seminfo {
                exp.init(ExpressionKind::NumberConstant, 0);
                exp.nval = val;
            } else {
                unreachable!()
            }
        }
        Some(t) if t.token == Reserved::String as u32 => {
            if let SemInfo::String(s) = &t.seminfo {
                code_string(lex, exp, s);
            } else {
                unreachable!();
            }
        }
        Some(t) if t.token == Reserved::Nil as u32 => {
            exp.init(ExpressionKind::Nil, 0);
        }
        Some(t) if t.token == Reserved::True as u32 => {
            exp.init(ExpressionKind::True, 0);
        }
        Some(t) if t.token == Reserved::False as u32 => {
            exp.init(ExpressionKind::False, 0);
        }
        Some(t) if t.token == Reserved::Dots as u32 => {
            // vararg
            if !lex.borrow_fs(None).f.is_vararg {
                return lex.syntax_error(state, "cannot use '...' outside a vararg function");
            }
            exp.init(
                ExpressionKind::VarArg,
                code_abc(lex, state, OpCode::VarArg as u32, 0, 1, 0)? as i32,
            );
        }
        Some(t) if t.token == '{' as u32 => {
            // constructor
            constructor(lex, state, exp)?;
            return Ok(());
        }
        Some(t) if t.token == Reserved::Function as u32 => {
            lex.next_token(state)?;
            body(lex, state, exp, false, lex.linenumber)?;
            return Ok(());
        }
        _ => {
            primary_expr(lex, state, exp)?;
            return Ok(());
        }
    }
    lex.next_token(state)?;
    Ok(())
}

/// body ->  `(' parlist `)' chunk END
fn body<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
    need_self: bool,
    line: usize,
) -> Result<(), LuaError> {
    let mut new_fs = FuncState::new(&lex.source);
    new_fs.f.linedefined = line;
    new_fs.prev = Some(lex.fs);
    let old_fs = lex.fs;
    lex.fs = lex.vfs.len();
    lex.vfs.push(new_fs);
    let new_fs = lex.fs;
    check_next(lex, state, '(' as u32)?;
    if need_self {
        new_localvar(lex, state, "self".to_owned(), 0)?;
        adjust_local_vars(lex, 1);
    }
    parameter_list(lex, state)?;
    check_next(lex, state, ')' as u32)?;
    chunk(lex, state)?;
    lex.borrow_mut_fs(None).f.lastlinedefined = lex.linenumber;
    check_match(
        lex,
        state,
        Reserved::End as u32,
        Reserved::Function as u32,
        line,
    )?;
    close_func(lex, state)?;
    push_closure(lex, state, old_fs, new_fs, exp)?;
    lex.fs = old_fs;
    Ok(())
}

fn push_closure<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    old_fs: usize,
    new_fs: usize,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    let proto = lex.vfs[new_fs].f.clone();
    let protoid = state.protos.len();
    state.protos.push(proto);
    lex.vfs[old_fs].f.p.push(protoid);
    let funcnum = lex.vfs[old_fs].f.p.len() as u32 - 1;
    let backup = lex.fs;
    lex.fs = old_fs;
    exp.init(
        ExpressionKind::Relocable,
        code_abx(lex, state, OpCode::Closure as u32, 0, funcnum)? as i32,
    );
    for i in 0..lex.vfs[backup].f.nups {
        let o = if lex.vfs[backup].upvalues[i].k == ExpressionKind::LocalRegister {
            OpCode::Move as u32
        } else {
            OpCode::GetUpVal as u32
        };
        code_abc(lex, state, o, 0, lex.vfs[backup].upvalues[i].info, 0)?;
    }
    lex.fs = backup;
    Ok(())
}

/// parlist -> [ param { `,' param } ]
fn parameter_list<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let mut nparams = 0;
    lex.borrow_mut_fs(None).f.is_vararg = false;
    if !lex.is_token(')' as u32) {
        // is `parlist' not empty?
        loop {
            match &lex.t {
                Some(t) if t.token == Reserved::Name as u32 => {
                    // param -> NAME
                    let var_name = str_checkname(lex, state)?;
                    new_localvar(lex, state, var_name, nparams)?;
                    nparams += 1;
                }
                Some(t) if t.token == Reserved::Dots as u32 => {
                    // param -> `...`
                    lex.next_token(state)?;
                    lex.borrow_mut_fs(None).f.is_vararg = true;
                }
                _ => {
                    return lex.syntax_error(state, "<name> or '... expected");
                }
            }
            if lex.borrow_fs(None).f.is_vararg || !test_next(lex, state, ',' as u32)? {
                break;
            }
        }
    }
    adjust_local_vars(lex, nparams);
    let nactvar = {
        let fs = lex.borrow_mut_fs(None);
        fs.f.numparams = fs.nactvar;
        fs.nactvar
    };
    // reserve register for parameters
    reserve_regs(lex, state, nactvar)
}

fn unary_op(t: &Option<crate::lex::Token>) -> Option<UnaryOp> {
    match t {
        Some(t) if t.token == Reserved::Not as u32 => Some(UnaryOp::Not),
        Some(t) if t.token == '-' as u32 => Some(UnaryOp::Minus),
        Some(t) if t.token == '#' as u32 => Some(UnaryOp::Len),
        _ => None,
    }
}

fn binary_op(t: &Option<crate::lex::Token>) -> Option<BinaryOp> {
    match t {
        Some(t) if t.token == '+' as u32 => Some(BinaryOp::Add),
        Some(t) if t.token == '-' as u32 => Some(BinaryOp::Sub),
        Some(t) if t.token == '*' as u32 => Some(BinaryOp::Mul),
        Some(t) if t.token == '/' as u32 => Some(BinaryOp::Div),
        Some(t) if t.token == '%' as u32 => Some(BinaryOp::Mod),
        Some(t) if t.token == '^' as u32 => Some(BinaryOp::Pow),
        Some(t) if t.token == Reserved::Concat as u32 => Some(BinaryOp::Concat),
        Some(t) if t.token == Reserved::Ne as u32 => Some(BinaryOp::Ne),
        Some(t) if t.token == Reserved::Eq as u32 => Some(BinaryOp::Eq),
        Some(t) if t.token == '<' as u32 => Some(BinaryOp::Lt),
        Some(t) if t.token == Reserved::Le as u32 => Some(BinaryOp::Le),
        Some(t) if t.token == '>' as u32 => Some(BinaryOp::Gt),
        Some(t) if t.token == Reserved::Ge as u32 => Some(BinaryOp::Ge),
        Some(t) if t.token == Reserved::And as u32 => Some(BinaryOp::And),
        Some(t) if t.token == Reserved::Or as u32 => Some(BinaryOp::Or),
        _ => None,
    }
}

fn break_stat<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let bl = match lex.borrow_fs(None).get_break_upval() {
        None => return lex.syntax_error(state, "no loop to break"),
        Some((upval,bl)) if upval => {
            let nactvar = lex.borrow_fs(None).bl[bl].nactvar;
            luaK::code_abc(lex, state, OpCode::Close as u32, nactvar as i32, 0,0)?;
            bl
        },
        Some((_,bl))=> bl,
    };
    let l2=luaK::jump(lex, state)?;
    let mut break_list=lex.borrow_fs(None).bl[bl].breaklist;
    luaK::concat(lex, state, &mut break_list, l2)?;
    lex.borrow_mut_fs(None).bl[bl].breaklist = break_list;
    Ok(())
}

/// stat -> RETURN explist
fn return_stat<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    _line: usize,
) -> Result<(), LuaError> {
    let first;
    let mut nret; // registers with returned values
    let mut e = ExpressionDesc::default();
    lex.next_token(state)?; // skip RETURN
    if block_follow(lex.t.as_ref().map(|t| t.token)) || lex.is_token(';' as u32) {
        first = 0; // return no values
        nret = 0;
    } else {
        nret = exp_list1(lex, state, &mut e)? as i32; // optional return values
        if has_mult_ret(e.k) {
            set_mult_ret(lex, state, &mut e)?;
            if e.k == ExpressionKind::Call && nret == 1 {
                // tail call ?
                set_opcode(
                    lex.borrow_mut_code(e.info as usize),
                    OpCode::TailCall as u32,
                );
                debug_assert!(
                    get_arg_a(lex.get_code(e.info as usize)) == lex.borrow_fs(None).nactvar as u32
                );
            }
            first = lex.borrow_fs(None).nactvar;
            nret = LUA_MULTRET;
        } else if nret == 1 {
            // only one single value?
            first = exp2anyreg(lex, state, &mut e)? as usize;
        } else {
            exp2nextreg(lex, state, &mut e)?; // values must go to the `stack'
            first = lex.borrow_fs(None).nactvar;
            debug_assert!(nret as usize == lex.borrow_fs(None).freereg - first);
        }
    }
    ret(lex, state, first as u32, nret as u32)
}

/// stat -> LOCAL NAME {`,' NAME} [`=' explist1]
fn local_stat<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let mut nvars = 0;
    let nexps;
    let mut exp = ExpressionDesc::default();
    loop {
        let var_name = str_checkname(lex, state)?;
        new_localvar(lex, state, var_name, nvars)?;
        nvars += 1;
        if !test_next(lex, state, ',' as u32)? {
            break;
        }
    }
    if test_next(lex, state, '=' as u32)? {
        nexps = exp_list1(lex, state, &mut exp)?;
    } else {
        exp.k = ExpressionKind::Void;
        nexps = 0;
    }
    adjust_assign(lex, state, nvars, nexps, &mut exp)?;
    adjust_local_vars(lex, nvars);
    Ok(())
}

fn adjust_assign<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    nvars: usize,
    nexps: usize,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    let mut extra = nvars as i32 - nexps as i32;
    if has_mult_ret(exp.k) {
        extra = (extra + 1).max(0); // includes call itself
        luaK::set_returns(lex, state, exp, extra)?; // last exp. provides the difference
        if extra > 1 {
            reserve_regs(lex, state, extra as usize - 1)?;
        }
    } else {
        if exp.k != ExpressionKind::Void {
            exp2nextreg(lex, state, exp)?; // close last expression
        }
        if extra > 0 {
            let reg = lex.borrow_fs(None).freereg;
            reserve_regs(lex, state, extra as usize)?;
            luaK::nil(lex, state, reg as u32, extra)?;
        }
    }
    Ok(())
}

fn local_func<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let var_name = str_checkname(lex, state)?;
    new_localvar(lex, state, var_name, 0)?;
    let mut v = ExpressionDesc::default();
    v.init(
        ExpressionKind::LocalRegister,
        lex.borrow_fs(None).freereg as i32,
    );
    luaK::reserve_regs(lex, state, 1)?;
    adjust_local_vars(lex, 1);
    let mut b = ExpressionDesc::default();
    body(lex, state, &mut b, false, lex.linenumber)?;
    luaK::store_var(lex, state, &v, &mut b)?;
    // debug information will only see the variable after this point!
    let (nactvar, pc) = {
        let fs = lex.borrow_fs(None);
        (fs.nactvar, fs.next_pc() as usize)
    };
    lex.borrow_mut_local_var(nactvar - 1).start_pc = pc;
    Ok(())
}

fn adjust_local_vars<T>(lex: &mut LexState<T>, nvars: usize) {
    let (nactvar, pc) = {
        let fs = lex.borrow_mut_fs(None);
        fs.nactvar += nvars;
        (fs.nactvar, fs.next_pc() as usize)
    };
    for i in 1..=nvars {
        lex.borrow_mut_local_var(nactvar - i).start_pc = pc;
    }
}

fn new_localvar<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    var_name: String,
    n: usize,
) -> Result<(), LuaError> {
    if lex.borrow_mut_fs(None).nactvar + n + 1 > LUAI_MAXVARS {
        return lex.error_limit(state, LUAI_MAXVARS, "local variables");
    }
    let fs = lex.borrow_mut_fs(None);
    fs.actvar[fs.nactvar + n] = register_local_var(fs, var_name);
    Ok(())
}

fn register_local_var(fs: &mut FuncState, name: String) -> usize {
    fs.f.locvars.push(LocVar {
        name,
        start_pc: 0,
        end_pc: 0,
    });
    fs.f.locvars.len() - 1
}

/// funcstat -> FUNCTION funcname body
fn func_stat<T>(lex: &mut LexState<T>, state: &mut LuaState, line: usize) -> Result<(), LuaError> {
    lex.next_token(state)?; // skip `function`
    let mut v = ExpressionDesc::default();
    let mut b = ExpressionDesc::default();
    let need_self = func_name(lex, state, &mut v)?;
    body(lex, state, &mut b, need_self, line)?;
    store_var(lex, state, &v, &mut b)?;
    fix_line(lex, line); // definition `happens' in the first line
    Ok(())
}

/// funcname -> NAME {field} [`:' NAME]
fn func_name<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    v: &mut ExpressionDesc,
) -> Result<bool, LuaError> {
    let mut need_self = false;
    single_var(lex, state, v)?;
    while lex.is_token('.' as u32) {
        field(lex, state, v)?;
    }
    if lex.is_token(':' as u32) {
        need_self = true;
        field(lex, state, v)?;
    }
    Ok(need_self)
}

/// repeatstat -> REPEAT block UNTIL cond
fn repeat_stat<T>(lex: &mut LexState<T>, state: &mut LuaState, line: usize) -> Result<(), LuaError> {
    let repeat_init = luaK::get_label(lex);
    enter_block(lex, true); // loop block
    enter_block(lex, false); // scope block
    lex.next_token(state)?; // skip REPEAT
    chunk(lex, state)?;
    check_match(lex, state, Reserved::Until as u32, Reserved::Repeat as u32, line)?;
    let cond_exit = cond(lex, state)?; // read condition (inside scope block)
    let upvals = lex.borrow_fs(None).bl.last().unwrap().upval;
    if ! upvals { // no upvalues ?
        leave_block(lex, state)?; // finish scope
        luaK::patch_list(lex, state, cond_exit, repeat_init)?; // close the loop
    } else {
        // complete semantics when there are upvalues
        break_stat(lex, state)?; // if condition then break
        luaK::patch_to_here(lex, state, cond_exit)?; // else...
        leave_block(lex, state)?; // finish scope
        let list=luaK::jump(lex, state)?;
        luaK::patch_list(lex, state, list, repeat_init)?; // and repeat
    }
    leave_block(lex, state)?;
    Ok(())
}

/// forstat -> FOR (fornum | forlist) END
fn for_stat<T>(lex: &mut LexState<T>, state: &mut LuaState, line: usize) -> Result<(), LuaError> {
    enter_block(lex, true); //scope for loop and control variables
    lex.next_token(state)?; // skip `for`
    let var_name = str_checkname(lex, state)?; // first variable name
    match &lex.t {
        Some(t) if t.token == '=' as u32 => {
            for_num(lex, state, var_name, line)?;
        }
        Some(t) if t.token == ',' as u32 || t.token == Reserved::In as u32 => {
            for_list(lex, state, var_name)?;
        }
        _ => {
            return lex.syntax_error(state, "'=' or 'in' expected");
        }
    }
    check_match(lex, state, Reserved::End as u32, Reserved::For as u32, line)?;
    leave_block(lex, state) // loop scope (`break' jumps to this point)
}

fn leave_block<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let bl = lex.borrow_mut_fs(None).bl.pop().unwrap();
    remove_vars(lex, bl.nactvar);
    if bl.upval {
        code_abc(lex, state, OpCode::Close as u32, bl.nactvar as i32, 0, 0)?;
    }
    // a block either controls scope or breaks (never both)
    {
        let fs = lex.borrow_mut_fs(None);
        debug_assert!(!bl.is_breakable || !bl.upval);
        debug_assert!(bl.nactvar == fs.nactvar);
        fs.freereg = fs.nactvar; // free registers
    }
    patch_to_here(lex, state, bl.breaklist)
}

/// forlist -> NAME {,NAME} IN explist1 forbody
fn for_list<T>(lex: &mut LexState<T>, state: &mut LuaState, var_name: String) -> Result<(), LuaError> {
    let base = lex.borrow_fs(None).freereg;
    // create control variables
    new_localvar(lex, state, "(for generator)".to_owned(), 0)?;
    new_localvar(lex, state, "(for state)".to_owned(), 1)?;
    new_localvar(lex, state, "(for control)".to_owned(), 2)?;
    // create declared variable
    new_localvar(lex, state, var_name, 3)?;
    let mut nvars=4;
    while test_next(lex, state, ',' as u32)? {
        let next_var_name = str_checkname(lex, state)?;
        new_localvar(lex, state, next_var_name, nvars)?;
        nvars+=1;
    }
    check_next(lex, state, Reserved::In as u32)?;
    let line = lex.linenumber;
    let mut e=ExpressionDesc::default();
    let nexps = exp_list1(lex, state, &mut e)?;
    adjust_assign(lex, state, 3, nexps, &mut e)?;
    luaK::check_stack(lex, state, 3)?; // extra space to call generator
    for_body(lex, state, base, line, nvars - 3, false)
}

/// fornum -> NAME = exp1,exp1[,exp1] forbody
fn for_num<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    var_name: String,
    line: usize,
) -> Result<(), LuaError> {
    let base = lex.borrow_fs(None).freereg;
    new_localvar(lex, state, "(for index)".to_owned(), 0)?;
    new_localvar(lex, state, "(for limit)".to_owned(), 1)?;
    new_localvar(lex, state, "(for step)".to_owned(), 2)?;
    new_localvar(lex, state, var_name, 3)?;
    check_next(lex, state, '=' as u32)?;
    exp1(lex, state)?; // initial value
    check_next(lex, state, ',' as u32)?;
    exp1(lex, state)?; // limit
    if test_next(lex, state, ',' as u32)? {
        exp1(lex, state)?; // optional step
    } else {
        // default step = 1
        let k = luaK::number_constant(lex, 1.0) as u32;
        code_abx(
            lex,
            state,
            OpCode::LoadK as u32,
            lex.borrow_fs(None).freereg as i32,
            k,
        )?;
        reserve_regs(lex, state, 1)?;
    }
    for_body(lex, state, base, line, 1, true)
}

/// forbody -> DO block
fn for_body<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    base: usize,
    line: usize,
    nvars: usize,
    is_num: bool,
) -> Result<(), LuaError> {
    adjust_local_vars(lex, 3); // control variables
    check_next(lex, state, Reserved::Do as u32)?;
    let prep = if is_num {
        code_asbx(lex, state, OpCode::ForPrep as u32, base as i32, NO_JUMP)? as i32
    } else {
        luaK::jump(lex, state)?
    };
    enter_block(lex, false); // scope for declared variables
    adjust_local_vars(lex, nvars);
    reserve_regs(lex, state, nvars)?;
    block(lex, state)?;
    leave_block(lex, state)?; // end of scope for declared variables
    patch_to_here(lex, state, prep)?; // fix the forprep instruction jump
    let endfor = if is_num {
        code_asbx(lex, state, OpCode::ForLoop as u32, base as i32, NO_JUMP)?
    } else {
        code_abc(
            lex,
            state,
            OpCode::TForLoop as u32,
            base as i32,
            0,
            nvars as i32,
        )?
    };
    fix_line(lex, line); // pretend that `OP_FOR' starts the loop
    if is_num {
        patch_list(lex, state, endfor as i32, prep + 1)?;
    } else {
        let endfor = luaK::jump(lex, state)?;
        patch_list(lex, state, endfor, prep + 1)?;
    }
    Ok(())
}

fn exp1<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<ExpressionKind, LuaError> {
    let mut e = ExpressionDesc::default();
    expr(lex, state, &mut e)?;
    let k = e.k;
    exp2nextreg(lex, state, &mut e)?;
    Ok(k)
}

fn enter_block<T>(lex: &mut LexState<T>, is_breakable: bool) {
    let fs = lex.borrow_mut_fs(None);
    fs.bl.push(BlockCnt::new(is_breakable, fs.nactvar));
    debug_assert!(fs.freereg == fs.nactvar);
}

fn check_match<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    what: u32,
    who: u32,
    line: usize,
) -> Result<(), LuaError> {
    if !test_next(lex, state, what)? {
        if lex.linenumber == line {
            return lex.syntax_error(state, &format!("'{}' expected", lex.token_2_txt(what)));
        }
        let msg = format!(
            "'{}' expected (to close '{}' at {})",
            lex.token_2_txt(what),
            lex.token_2_txt(who),
            line
        );
        state.push_string(&msg);
        return lex.syntax_error(state, &msg);
    }
    Ok(())
}

///  block -> chunk
fn block<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    enter_block(lex, false);
    chunk(lex, state)?;
    debug_assert!(lex.borrow_fs(None).bl.last().unwrap().breaklist == NO_JUMP);
    leave_block(lex, state)
}

fn while_stat<T>(_lex: &mut LexState<T>, _line: usize) -> Result<(), LuaError> {
    todo!()
}

/// ifstat -> IF cond THEN block {ELSEIF cond THEN block} [ELSE block] END
fn if_stat<T>(lex: &mut LexState<T>, state: &mut LuaState, line: usize) -> Result<(), LuaError> {
    let mut escape_list = NO_JUMP;
    let mut flist = test_then_block(lex, state)?; // IF cond THEN block
    while lex.is_token(Reserved::ElseIf as u32) {
        let l2=luaK::jump(lex, state)?;
        luaK::concat(lex, state, &mut escape_list, l2)?;
        luaK::patch_to_here(lex, state, flist)?;
        flist = test_then_block(lex, state)?; // ELSEIF cond THEN block
    }
    if lex.is_token(Reserved::Else as u32) {
        let l2=luaK::jump(lex, state)?;
        luaK::concat(lex, state, &mut escape_list, l2)?;
        luaK::patch_to_here(lex, state, flist)?;
        lex.next_token(state)?; // skip ELSE (after patch, for correct line info)
        block(lex, state)?; // `else' part
    } else {
        luaK::concat(lex, state, &mut escape_list, flist)?;
    }
    luaK::patch_to_here(lex, state, escape_list)?;
    check_match(lex, state, Reserved::End as u32, Reserved::If as u32, line)
}

/// test_then_block -> [IF | ELSEIF] cond THEN block
fn test_then_block<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<i32, LuaError> {
    lex.next_token(state)?; // skip IF or ELSEIF
    let cond_exit = cond(lex,state)?;
    check_next(lex, state, Reserved::Then as u32)?;
    block(lex, state)?;
    Ok(cond_exit)
}

/// cond -> exp
fn cond<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<i32, LuaError> {
    let mut v = ExpressionDesc::default();
    expr(lex, state, &mut v)?; // read condition
    if v.k == ExpressionKind::Nil {
        v.k = ExpressionKind::False; // `falses' are all equal here
    }
    luaK::go_if_true(lex, state, &mut v)?;
    Ok(v.f)
}
