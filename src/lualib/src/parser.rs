//! Lua Parser

use std::{cell::RefCell, rc::Rc};

use crate::{
    api::LuaError,
    ldo::SParser,
    lex::{LabelDesc, LexState, Reserved, SemInfo},
    limits::{MAX_INT, MAX_UPVAL},
    luaH::{Table, TableRef},
    luaK::{
        self, code_abc, code_abx, code_asbx, code_k, exp2anyreg, exp2nextreg, exp2rk, fix_line,
        indexed, patch_list, patch_to_here, reserve_regs, ret, set_list, set_mult_ret, store_var,
    },
    luaconf::{LUAI_MAXRCALLS, LUAI_MAXVARS},
    object::{int2fb, LClosure, LocVar, ProtoId, TValue},
    opcodes::{
        get_arg_a, set_arg_b, set_arg_c, set_opcode, OpCode, LFIELDS_PER_FLUSH, MAXARG_BX, NO_JUMP,
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

/// Description of an upvalue for function prototypes
#[derive(Default, Clone)]
pub struct UpValDesc {
    ///  upvalue name (for debug information)
    name: String,
    /// whether it is in stack
    pub in_stack: bool,
    /// index of upvalue (in stack or in outer function's list)
    pub idx: usize,
}

/// nodes for block list (list of active blocks)
pub(crate) struct BlockCnt {
    /// index of first label in this block
    pub first_label: usize,
    /// index of first pending goto in this block
    first_goto: usize,
    /// # active locals outside the breakable structure
    nactvar: usize,
    /// true if some variable in the block is an upvalue
    upval: bool,
    /// true if `block' is a loop
    is_loop: bool,
}

impl BlockCnt {
    fn new(is_loop: bool, nactvar: usize, first_label: usize, first_goto: usize) -> Self {
        Self {
            first_label: first_label,
            first_goto: first_goto,
            nactvar,
            upval: false,
            is_loop,
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
    pub f: ProtoId,
    /// table to find (and reuse) constants in `f.k'
    pub h: TableRef,
    /// chain of current blocks
    pub(crate) bl: Vec<BlockCnt>,
    /// `pc' of last `jump target'
    pub last_target: i32,
    /// list of pending jumps to `pc'
    pub jpc: i32,
    /// index of first local var (in Dyndata array)
    pub first_local: usize,
    /// number of active local variables
    pub nactvar: usize,
    /// first free register
    pub freereg: usize,
    // declared-variable stack
    //pub actvar: [usize; LUAI_MAXVARS],
}

impl FuncState {
    pub(crate) fn new() -> Self {
        // TODO proto.source = lex_state.source
        Self {
            f: 0,
            h: Rc::new(RefCell::new(Table::new())),
            //prev: None,
            bl: Vec::new(),
            last_target: NO_JUMP,
            jpc: NO_JUMP,
            freereg: 0,
            first_local: 0,
            nactvar: 0,
        }
    }

    fn mark_upval(&mut self, level: usize) {
        for bl in self.bl.iter_mut().rev() {
            if bl.nactvar <= level {
                bl.upval = true;
                break;
            }
        }
    }

    pub(crate) fn borrow_block(&self) -> &BlockCnt {
        self.bl.last().unwrap()
    }

    pub(crate) fn add_constant(
        &mut self,
        state: &mut LuaState,
        key: TValue,
        value: TValue,
    ) -> usize {
        let val = self.h.borrow_mut().get(&key).cloned();
        match val {
            Some(TValue::Number(n)) => n as usize,
            _ => {
                let kid = state.protos[self.f].k.len();
                self.h
                    .borrow_mut()
                    .set(key, TValue::Number(kid as LuaNumber));
                state.protos[self.f].k.push(value);
                kid
            }
        }
    }

    pub fn string_constant(&mut self, state: &mut LuaState, value: &str) -> usize {
        let tvalue = TValue::from(value);
        self.add_constant(state, tvalue.clone(), tvalue)
    }

    pub fn number_constant(&mut self, state: &mut LuaState, value: LuaNumber) -> usize {
        let tvalue = TValue::Number(value);
        self.add_constant(state, tvalue.clone(), tvalue)
    }
}

pub fn parser<T>(state: &mut LuaState, parser: &mut SParser<T>) -> Result<LClosure, LuaError> {
    let mut lex = LexState::new(parser.z.take().unwrap(), &parser.name);
    let mut new_fs = FuncState::new();
    new_fs.f = state.add_prototype(&lex, &parser.name, 1);
    lex.vfs.push(new_fs);
    // read the first character in the stream
    lex.next_char(state);
    main_func(&mut lex, state)?;
    let cl = LClosure::new(0, 1); //create main closure
    Ok(cl)
}

fn main_func<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let mut v = ExpressionDesc::default();
    open_func(lex, state);
    lex.borrow_mut_proto(state, None).is_vararg = true; // main function is always vararg
    v.init(ExpressionKind::LocalRegister, 0);
    let envn = lex.envn.clone();
    new_upvalue(lex, state, None, &envn, &mut v)?;
    lex.next_token(state)?; // read first token
    stat_list(lex, state)?; // parse main body
    lex.check_eos(state)?;
    close_func(lex, state)?;
    Ok(())
}

fn open_func<T>(lex: &mut LexState<T>, state: &mut LuaState) {
    lex.borrow_mut_fs(None).first_local = lex.dyd.actvar.len();
    // put table of constants on stack
    state
        .stack
        .push(TValue::Table(Rc::clone(&lex.borrow_fs(None).h)));
    enter_block(lex, false);
}

fn close_func<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    luaK::ret(lex, state, 0, 0)?; // final return
    leave_block(lex, state)?;
    lex.vfs.pop();
    state.stack.pop(); // pop table of constants
    Ok(())
}

fn remove_vars<T>(lex: &mut LexState<T>, state: &mut LuaState, to_level: usize) {
    let mut nactvar = lex.borrow_fs(None).nactvar;
    let vars_to_remove = nactvar - to_level;
    while nactvar > to_level {
        nactvar -= 1;
        lex.borrow_mut_local_var(state, nactvar).end_pc = lex.next_pc(state) as usize;
    }
    lex.borrow_mut_fs(None).nactvar = nactvar;
    lex.dyd
        .actvar
        .truncate(lex.dyd.actvar.len() - vars_to_remove);
}

/// check whether current token is in the follow set of a block.
///  'until' closes syntactical blocks, but do not close scope,
///  so it handled in separate.
fn block_follow<T>(lex: &LexState<T>, with_until: bool) -> bool {
    match &lex.t {
        Some(tok) => match Reserved::try_from(tok.token) {
            Ok(Reserved::Else) => true,
            Ok(Reserved::ElseIf) => true,
            Ok(Reserved::End) => true,
            Ok(Reserved::Eos) => true,
            Ok(Reserved::Until) => with_until,
            _ => false,
        },
        _ => true,
    }
}

fn leave_level<T>(_lex: &mut LexState<T>, state: &mut LuaState) {
    state.n_rcalls -= 1;
}

fn test_next<T>(lex: &mut LexState<T>, state: &mut LuaState, arg: u32) -> Result<bool, LuaError> {
    if matches!(&lex.t, Some(t) if t.token == arg) {
        lex.next_token(state)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn enter_level<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    state.n_rcalls += 1;
    if state.n_rcalls >= crate::luaconf::LUAI_MAXRCALLS {
        return lex.lex_error(state, "chunk has too many syntax levels", None);
    }
    Ok(())
}

fn statement<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let line = lex.linenumber;
    enter_level(lex, state)?;
    if let Some(ref t) = lex.t {
        if t.token == ';' as u32 {
            // stat -> ';' (empty statement)
            lex.next_token(state)?; // skip ';'
        } else {
            match Reserved::try_from(t.token) {
                Ok(Reserved::If) => {
                    // stat -> ifstat
                    if_stat(lex, state, line)?;
                }
                Ok(Reserved::While) => {
                    // stat -> whilestat
                    while_stat(lex, state, line)?;
                }
                Ok(Reserved::Do) => {
                    // stat -> DO block END
                    lex.next_token(state)?;
                    block(lex, state)?;
                    check_match(lex, state, Reserved::End as u32, Reserved::Do as u32, line)?;
                }
                Ok(Reserved::For) => {
                    // stat -> forstat
                    for_stat(lex, state, line)?;
                }
                Ok(Reserved::Repeat) => {
                    // stat -> repeatstat
                    repeat_stat(lex, state, line)?;
                }
                Ok(Reserved::Function) => {
                    // stat -> funcstat
                    func_stat(lex, state, line)?;
                }
                Ok(Reserved::Local) => {
                    // stat -> localstat
                    lex.next_token(state)?;
                    if test_next(lex, state, Reserved::Function as u32)? {
                        // local function
                        local_func(lex, state)?;
                    } else {
                        local_stat(lex, state)?;
                    }
                }
                Ok(Reserved::DbColon) => {
                    // stat -> label
                    lex.next_token(state)?; // skip double colon
                    let name = str_checkname(lex, state)?;
                    label_stat(lex, state, name, line)?;
                }
                //  stat -> retstat
                Ok(Reserved::Return) => {
                    lex.next_token(state)?; // skip RETURN
                    return_stat(lex, state, line)?;
                }
                // stat -> breakstat
                // stat -> 'goto' NAME
                Ok(Reserved::Break) | Ok(Reserved::Goto) => {
                    let l = luaK::jump(lex, state)?;
                    goto_stat(lex, state, l)?;
                }
                _ => {
                    // stat -> func | assignment
                    expr_stat(lex, state)?;
                }
            }
        }
    } else {
        expr_stat(lex, state)?;
    }
    let nactvar = lex.borrow_fs(None).nactvar;
    lex.borrow_mut_fs(None).freereg = nactvar;
    leave_level(lex, state);
    Ok(())
}

/// label -> '::' NAME '::'
fn label_stat<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    label: String,
    line: usize,
) -> Result<(), LuaError> {
    lex.check_repeated(state, &label)?; // check for repeated labels
    check_next(lex, state, Reserved::DbColon as u32)?; // skip double colon
    let pc = lex.next_pc(state) as i32;
    let l = lex.dyd.label.len();
    lex.dyd.label.push(lex.new_label_entry(label, line, pc));
    skip_noop_stat(lex, state)?; // skip other no-op statements
    if block_follow(lex, false) {
        // label is last no-op statement in the block?
        // assume that locals are already out of scope
        lex.dyd.label[l].nactvar = lex.borrow_fs(None).nactvar;
    }
    find_gotos(lex, state, l)?;
    Ok(())
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
    /// info = result register
    NonRelocable,
    /// info = local register
    LocalRegister,
    /// info = index of upvalue in `upvalues'
    UpValue,
    /// info = table register; aux = index register (or `k')
    Indexed,
    /// info = instruction pc
    Jump,
    /// info = instruction pc
    Relocable,
    /// info = instruction pc
    Call,
    /// info = instruction pc
    VarArg,
}

#[derive(Default, Clone)]
pub struct IndexedDesc {
    /// index (R/K)
    pub idx: u32,
    /// table (register or upvalue)
    pub t: u32,
    /// whether 't' is register (VLOCAL) or upvalue (VUPVAL)
    pub is_t_upval: bool,
}

#[derive(Default, Clone)]
pub struct ExpressionDesc {
    pub k: ExpressionKind,
    // for ExpressionKind::Indexed
    pub ind: IndexedDesc,
    /// for generic use
    pub info: i32,
    /// for ExpressionKind::NumberConstant
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

    pub(crate) fn is_numeral(&self) -> bool {
        self.k == ExpressionKind::NumberConstant && self.t == NO_JUMP && self.f == NO_JUMP
    }
}

#[derive(Default)]
struct LHSAssignment {
    /// variable (global, local, upvalue, or indexed)
    v: ExpressionDesc,
}

/// stat -> func | assignment
fn expr_stat<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let mut lhs = LHSAssignment::default();
    suffixed_expr(lex, state, &mut lhs.v)?;
    if lex.is_token('=' as u32) || lex.is_token(',' as u32) {
        // stat -> assignment ?
        let mut vlhs = Vec::new();
        vlhs.push(lhs);
        assignment(lex, state, &mut vlhs, 1)?;
    } else {
        if lhs.v.k != ExpressionKind::Call {
            return lex.syntax_error(state, "syntax error");
        }
        // call statement uses no results
        set_arg_c(lex.borrow_mut_code(state, lhs.v.info as usize), 1);
    }
    Ok(())
}

/// suffixedexp -> primaryexp { '.' NAME | '[' exp ']' | ':' NAME funcargs | funcargs }
fn suffixed_expr<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    v: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    let line = lex.linenumber;
    primary_expr(lex, state, v)?;
    loop {
        match lex.t.clone() {
            Some(t) => match t.token as u8 as char {
                '.' => {
                    // fieldsel
                    field_sel(lex, state, v)?;
                }
                '[' => {
                    // [ 'exp1' ]
                    let mut key = ExpressionDesc::default();
                    luaK::exp2anyregup(lex, state, v)?;
                    yindex(lex, state, &mut key)?;
                    luaK::indexed(lex, state, v, &mut key)?;
                }
                ':' => {
                    // `:' NAME funcargs
                    let mut key = ExpressionDesc::default();
                    lex.next_token(state)?;
                    check_name(lex, state, &mut key)?;
                    luaK::op_self(lex, state, v, &mut key)?;
                    func_args(lex, state, v, line)?;
                }
                '(' => {
                    // funcargs
                    luaK::exp2nextreg(lex, state, v)?;
                    func_args(lex, state, v, line)?;
                }
                _ => break,
            },
            _ => break,
        }
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
        if nvk == ExpressionKind::LocalRegister {
            check_conflict(lex, state, &exp, lhs)?;
        }
        if nvars > LUAI_MAXRCALLS - state.n_rcalls {
            return lex.error_limit(state, LUAI_MAXRCALLS, "variables in assignment");
        }
        assignment(lex, state, lhs, nvars + 1)?;
        lhs.pop();
    } else {
        // assignment -> `=' explist1
        check_next(lex, state, '=' as u32)?;
        let nexps = exp_list(lex, state, &mut exp)?;
        if nexps != nvars {
            adjust_assign(lex, state, nvars, nexps, &mut exp)?;
            if nexps > nvars {
                lex.borrow_mut_fs(None).freereg -= nexps - nvars; // remove extra values
            }
        } else {
            luaK::set_one_ret(lex, state, &mut exp); // close last expression
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

/// check whether, in an assignment to a local variable, the local variable
/// is needed in a previous assignment (to a table). If so, save original
/// local value in a safe place and use this safe copy in the previous
/// assignment.
fn check_conflict<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    v: &ExpressionDesc,
    lhs: &mut [LHSAssignment],
) -> Result<(), LuaError> {
    let extra = lex.borrow_fs(None).freereg as u32; // eventual position to save local variable
    let mut conflict = false;
    for lh in lhs.iter_mut().rev() {
        // check all previous assignments
        if lh.v.k == ExpressionKind::Indexed {
            // assigning to a table?
            // table is the upvalue/local being assigned now?
            if lh.v.ind.is_t_upval == (v.k == ExpressionKind::UpValue)
                && lh.v.ind.t as i32 == v.info
            {
                // conflict?
                conflict = true;
                lh.v.ind.is_t_upval = false;
                lh.v.ind.t = extra; // previous assignment will use safe copy
            }
            // index is the local being assigned? (index cannot be upvalue)
            if v.k == ExpressionKind::LocalRegister && lh.v.ind.idx as i32 == v.info {
                // conflict ?
                conflict = true;
                lh.v.ind.idx = extra; // previous assignment will use safe copy
            }
        }
    }
    if conflict {
        // copy upvalue/local value to a temporary (in position 'extra')
        let opcode = if v.k == ExpressionKind::LocalRegister {
            OpCode::Move
        } else {
            OpCode::GetUpVal
        };
        luaK::code_abc(lex, state, opcode as u32, extra as i32, v.info, 0)?; // make copy
        luaK::reserve_regs(lex, state, 1)?;
    }
    Ok(())
}

/// primaryexp -> NAME | '(' expr ')'
fn primary_expr<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    if lex.is_token('(' as u32) {
        let line = lex.linenumber;
        lex.next_token(state)?;
        expr(lex, state, exp)?;
        check_match(lex, state, ')' as u32, '(' as u32, line)?;
        luaK::discharge_vars(lex, state, exp)?;
        Ok(())
    } else if lex.is_token(Reserved::Name as u32) {
        single_var(lex, state, exp)?;
        Ok(())
    } else {
        lex.syntax_error(state, "unexpected symbol")
    }
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
    line: usize,
) -> Result<(), LuaError> {
    let mut args = ExpressionDesc::default();
    match &lex.t.clone() {
        Some(t) if t.token == '(' as u32 => {
            // funcargs -> `(' [ explist1 ] `)'
            lex.next_token(state)?;
            if lex.is_token(')' as u32) {
                // arg list is empty
                args.k = ExpressionKind::Void;
            } else {
                exp_list(lex, state, &mut args)?;
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
                code_string(lex, state, &mut args, s);
            } else {
                unreachable!()
            }
            lex.next_token(state)?; // must use `seminfo' before `next'
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
    luaK::fix_line(lex, state, line);
    lex.borrow_mut_fs(None).freereg = base as usize + 1; // call remove function and arguments and leaves
                                                         // (unless changed) one result
    Ok(())
}

#[inline]
fn has_mult_ret(k: ExpressionKind) -> bool {
    k == ExpressionKind::Call || k == ExpressionKind::VarArg
}

/// set expression as a string constant
fn code_string<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
    val: &str,
) {
    exp.init(
        ExpressionKind::Constant,
        lex.borrow_mut_fs(None).string_constant(state, val) as i32,
    );
}

/// constructor -> '{' [ field { sep field } [sep] ] '}'
/// sep -> ',' | ';'
fn constructor<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    let line = lex.linenumber;
    let pc = code_abc(lex, state, OpCode::NewTable as u32, 0, 0, 0)?;
    let mut cc = ConstructorControl::default();
    exp.init(ExpressionKind::Relocable, pc as i32);
    exp2nextreg(lex, state, exp)?;
    check_next(lex, state, '{' as u32)?;
    loop {
        debug_assert!(cc.v.k == ExpressionKind::Void || cc.to_store > 0);
        if lex.is_token('}' as u32) {
            break;
        }
        close_list_field(lex, state, &mut cc, exp)?;
        field(lex, state, &mut cc, exp)?;
        if !test_next(lex, state, ',' as u32)? && !test_next(lex, state, ';' as u32)? {
            break;
        }
    }
    check_match(lex, state, '}' as u32, '{' as u32, line)?;
    last_list_field(lex, state, &mut cc, exp)?;
    set_arg_b(
        lex.borrow_mut_code(state, pc as usize),
        int2fb(cc.na as u32),
    ); // set initial array size
    set_arg_c(
        lex.borrow_mut_code(state, pc as usize),
        int2fb(cc.nh as u32),
    ); // set initial table size
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
fn rec_field<T>(
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
    code_string(lex, state, key, &name);
    Ok(())
}

fn list_field<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    cc: &mut ConstructorControl,
) -> Result<(), LuaError> {
    expr(lex, state, &mut cc.v)?;
    if cc.na > MAXARG_BX {
        return lex.error_limit(state, MAX_INT, "items in a constructor");
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

/// explist -> expr { `,' expr }
fn exp_list<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<usize, LuaError> {
    let mut n = 1; // at least one expression
    expr(lex, state, exp)?;
    while test_next(lex, state, ',' as u32)? {
        exp2nextreg(lex, state, exp)?;
        expr(lex, state, exp)?;
        n += 1;
    }
    Ok(n)
}

/// field -> listfield | recfield
fn field<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    cc: &mut ConstructorControl,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    match &lex.t {
        Some(t) if t.token == Reserved::Name as u32 => {
            //  may be listfields or recfields
            lex.look_ahead(state)?;
            if !lex.is_lookahead_token('=' as u32) {
                // expression ?
                list_field(lex, state, cc)?;
            } else {
                rec_field(lex, state, cc, exp)?;
            }
        }
        Some(t) if t.token == '[' as u32 => {
            // constructor_item -> recfield
            rec_field(lex, state, cc, exp)?;
        }
        _ => {
            // constructor_part -> listfield
            list_field(lex, state, cc)?;
        }
    }
    Ok(())
}

fn single_var<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    let name = str_checkname(lex, state)?;
    if single_var_aux(lex, state, lex.vfs.len() - 1, &name, exp, true)? == ExpressionKind::Void {
        // global name?
        let mut key = ExpressionDesc::default();
        // get environment variable
        let envn = lex.envn.clone();
        single_var_aux(lex, state, lex.vfs.len() - 1, &envn, exp, true)?;
        debug_assert!(exp.k == ExpressionKind::LocalRegister || exp.k == ExpressionKind::UpValue);
        code_string(lex, state, &mut key, &name); // key is variable name
        luaK::indexed(lex, state, exp, &mut key)?; // env[varname]
    }
    Ok(())
}

fn single_var_aux<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    fsid: usize,
    name: &str,
    exp: &mut ExpressionDesc,
    base: bool,
) -> Result<ExpressionKind, LuaError> {
    // look up at current level
    if let Some(v) = lex.search_var(state, Some(fsid), name) {
        exp.init(ExpressionKind::LocalRegister, v as i32);
        if !base {
            // local will be used as an upval
            lex.borrow_mut_fs(Some(fsid)).mark_upval(v);
        }
        Ok(ExpressionKind::LocalRegister)
    } else {
        // not found at current level; try upvalues
        let mut idx = search_upvalues(lex, state, fsid, name);
        if idx.is_none() {
            if fsid == 0 {
                // no more levels. var is global
                return Ok(ExpressionKind::Void);
            }
            let prev_fsid = fsid - 1;
            // not found ?
            // try upper levels
            if let Ok(ExpressionKind::Void) = single_var_aux(lex, state, prev_fsid, name, exp, base)
            {
                // not found; is a global
                return Ok(ExpressionKind::Void);
            } else {
                // else was LOCAL or UPVAL
                // will be a new upvalue
                idx = Some(new_upvalue(lex, state, Some(fsid), name, exp)?);
            }
        }
        exp.init(ExpressionKind::UpValue, idx.unwrap() as i32);
        Ok(ExpressionKind::UpValue)
    }
}

fn new_upvalue<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    fsid: Option<usize>,
    name: &str,
    exp: &mut ExpressionDesc,
) -> Result<usize, LuaError> {
    let nups = lex.borrow_proto(state, fsid).upvalues.len();
    if nups + 1 > MAX_UPVAL {
        lex.error_limit(state, MAX_UPVAL, "upvalues")?
    }
    let proto = lex.borrow_mut_proto(state, fsid);
    proto.upvalues.push(UpValDesc {
        name: name.to_owned(),
        in_stack: exp.k == ExpressionKind::LocalRegister,
        idx: exp.info as usize,
    });
    Ok(proto.upvalues.len() - 1)
}

fn search_upvalues<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    fsid: usize,
    name: &str,
) -> Option<usize> {
    let proto = lex.borrow_proto(state, Some(fsid));
    for (i, up) in proto.upvalues.iter().enumerate() {
        if up.name == name {
            return Some(i);
        }
    }
    None
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
        let line = lex.linenumber;
        lex.next_token(state)?;
        subexpr(lex, state, exp, UNARY_PRIORITY)?;
        luaK::prefix(lex, state, uop, exp, line)?;
    } else {
        simple_exp(lex, state, exp)?;
    }
    // expand while operators have priorities higher than `limit'
    let mut oop = binary_op(&lex.t);
    while let Some(op) = oop {
        if BINARY_OP_PRIO[op as usize].left <= limit {
            break;
        }
        let line = lex.linenumber;
        lex.next_token(state)?;
        luaK::infix(lex, state, op, exp)?;
        let mut exp2 = ExpressionDesc::default();
        // read sub-expression with higher priority
        let nextop = subexpr(lex, state, &mut exp2, BINARY_OP_PRIO[op as usize].right)?;
        luaK::postfix(lex, state, op, exp, &mut exp2, line)?;
        oop = nextop;
    }
    leave_level(lex, state);
    Ok(oop) // return first untreated operator
}

/// simpleexp -> NUMBER | STRING | NIL | TRUE | FALSE | ... |
/// constructor | FUNCTION body | suffixedexp
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
                code_string(lex, state, exp, s);
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
            if !lex.borrow_proto(state, None).is_vararg {
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
            suffixed_expr(lex, state, exp)?;
            return Ok(());
        }
    }
    lex.next_token(state)?;
    Ok(())
}

/// body ->  `(' parlist `)' block END
fn body<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
    is_method: bool,
    line: usize,
) -> Result<(), LuaError> {
    let mut new_fs = FuncState::new();
    new_fs.f = state.add_prototype(&lex, &lex.source, line);
    lex.vfs.push(new_fs);
    open_func(lex, state);
    check_next(lex, state, '(' as u32)?;
    if is_method {
        new_localvar(lex, state, "self".to_owned())?;
        adjust_local_vars(lex, state, 1);
    }
    parameter_list(lex, state)?;
    check_next(lex, state, ')' as u32)?;
    stat_list(lex, state)?;
    lex.borrow_mut_proto(state, None).lastlinedefined = lex.linenumber;
    check_match(
        lex,
        state,
        Reserved::End as u32,
        Reserved::Function as u32,
        line,
    )?;
    code_closure(lex, state, exp)?;
    close_func(lex, state)?;
    Ok(())
}

/// codes instruction to create new closure in parent function.
fn code_closure<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    let fs = lex.vfs.pop().unwrap();
    let funcnum = lex.borrow_proto(state, None).p.len() as u32 - 1;
    exp.init(
        ExpressionKind::Relocable,
        code_abx(lex, state, OpCode::Closure as u32, 0, funcnum)? as i32,
    );
    luaK::exp2nextreg(lex, state, exp)?; // fix it at the last register
    lex.vfs.push(fs);
    Ok(())
}

/// parlist -> [ param { `,' param } ]
fn parameter_list<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let mut nparams = 0;
    lex.borrow_mut_proto(state, None).is_vararg = false;
    if !lex.is_token(')' as u32) {
        // is `parlist' not empty?
        loop {
            match &lex.t {
                Some(t) if t.token == Reserved::Name as u32 => {
                    // param -> NAME
                    let var_name = str_checkname(lex, state)?;
                    new_localvar(lex, state, var_name)?;
                    nparams += 1;
                }
                Some(t) if t.token == Reserved::Dots as u32 => {
                    // param -> `...`
                    lex.next_token(state)?;
                    lex.borrow_mut_proto(state, None).is_vararg = true;
                }
                _ => {
                    return lex.syntax_error(state, "<name> or '... expected");
                }
            }
            if lex.borrow_proto(state, None).is_vararg || !test_next(lex, state, ',' as u32)? {
                break;
            }
        }
    }
    adjust_local_vars(lex, state, nparams);
    let nactvar = {
        let nactvar = lex.borrow_mut_fs(None).nactvar;
        lex.borrow_mut_proto(state, None).numparams = nactvar;
        nactvar
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

/// stat -> RETURN [explist] [';']
fn return_stat<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    _line: usize,
) -> Result<(), LuaError> {
    let first;
    let mut nret; // registers with returned values
    let mut e = ExpressionDesc::default();
    if block_follow(&lex, true) || lex.is_token(';' as u32) {
        first = 0; // return no values
        nret = 0;
    } else {
        nret = exp_list(lex, state, &mut e)? as i32; // optional return values
        if has_mult_ret(e.k) {
            set_mult_ret(lex, state, &mut e)?;
            if e.k == ExpressionKind::Call && nret == 1 {
                // tail call ?
                set_opcode(
                    lex.borrow_mut_code(state, e.info as usize),
                    OpCode::TailCall as u32,
                );
                debug_assert!(
                    get_arg_a(lex.get_code(state, e.info as usize))
                        == lex.borrow_fs(None).nactvar as u32
                );
            }
            first = lex.borrow_fs(None).nactvar;
            nret = LUA_MULTRET; // return all values
        } else if nret == 1 {
            // only one single value?
            first = exp2anyreg(lex, state, &mut e)? as usize;
        } else {
            exp2nextreg(lex, state, &mut e)?; // values must go to the `stack'
            first = lex.borrow_fs(None).nactvar; // return all `active' values
            debug_assert!(nret as usize == lex.borrow_fs(None).freereg - first);
        }
    }
    ret(lex, state, first as u32, nret as u32)?;
    test_next(lex, state, ';' as u32)?; // skip optional semicolon
    Ok(())
}

/// stat -> LOCAL NAME {`,' NAME} [`=' explist1]
fn local_stat<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let mut nvars = 0;
    let nexps;
    let mut exp = ExpressionDesc::default();
    loop {
        let var_name = str_checkname(lex, state)?;
        new_localvar(lex, state, var_name)?;
        nvars += 1;
        if !test_next(lex, state, ',' as u32)? {
            break;
        }
    }
    if test_next(lex, state, '=' as u32)? {
        nexps = exp_list(lex, state, &mut exp)?;
    } else {
        exp.k = ExpressionKind::Void;
        nexps = 0;
    }
    adjust_assign(lex, state, nvars, nexps, &mut exp)?;
    adjust_local_vars(lex, state, nvars);
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
    new_localvar(lex, state, var_name)?; // new local variable
    adjust_local_vars(lex, state, 1); // enter its scope
    let mut b = ExpressionDesc::default();
    body(lex, state, &mut b, false, lex.linenumber)?; // function created in next register
                                                      // debug information will only see the variable after this point!
    let pc = lex.next_pc(state) as usize;
    lex.borrow_mut_local_var(state, b.info as usize).start_pc = pc;
    Ok(())
}

fn adjust_local_vars<T>(lex: &mut LexState<T>, state: &mut LuaState, nvars: usize) {
    let (nactvar, pc) = {
        let fs = lex.borrow_mut_fs(None);
        fs.nactvar += nvars;
        (fs.nactvar, lex.next_pc(state) as usize)
    };
    for i in 1..=nvars {
        lex.borrow_mut_local_var(state, nactvar - i).start_pc = pc;
    }
}

fn new_localvar<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    var_name: String,
) -> Result<(), LuaError> {
    let reg = register_local_var(lex, state, var_name);
    let first_local = lex.borrow_fs(None).first_local;
    if lex.dyd.actvar.len() + 1 - first_local > LUAI_MAXVARS {
        return lex.error_limit(state, LUAI_MAXVARS, "local variables");
    }
    lex.dyd.actvar.push(reg);
    Ok(())
}

fn register_local_var<T>(lex: &mut LexState<T>, state: &mut LuaState, name: String) -> usize {
    let proto = lex.borrow_mut_proto(state, None);
    proto.locvars.push(LocVar {
        name,
        start_pc: 0,
        end_pc: 0,
    });
    proto.locvars.len() - 1
}

/// funcstat -> FUNCTION funcname body
fn func_stat<T>(lex: &mut LexState<T>, state: &mut LuaState, line: usize) -> Result<(), LuaError> {
    lex.next_token(state)?; // skip `function`
    let mut v = ExpressionDesc::default();
    let mut b = ExpressionDesc::default();
    let is_method = func_name(lex, state, &mut v)?;
    body(lex, state, &mut b, is_method, line)?;
    store_var(lex, state, &v, &mut b)?;
    fix_line(lex, state, line); // definition `happens' in the first line
    Ok(())
}

/// funcname -> NAME {fieldsel} [`:' NAME]
fn func_name<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    v: &mut ExpressionDesc,
) -> Result<bool, LuaError> {
    let mut is_method = false;
    single_var(lex, state, v)?;
    while lex.is_token('.' as u32) {
        field_sel(lex, state, v)?;
    }
    if lex.is_token(':' as u32) {
        is_method = true;
        field_sel(lex, state, v)?;
    }
    Ok(is_method)
}

/// fieldsel -> ['.' | ':'] NAME
fn field_sel<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    v: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    let mut key = ExpressionDesc::default();
    exp2anyreg(lex, state, v)?;
    lex.next_token(state)?; // skip the dot or colon
    check_name(lex, state, &mut key)?;
    indexed(lex, state, v, &mut key)
}

/// repeatstat -> REPEAT block UNTIL cond
fn repeat_stat<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    line: usize,
) -> Result<(), LuaError> {
    let repeat_init = luaK::get_label(lex, state);
    enter_block(lex, true); // loop block
    enter_block(lex, false); // scope block
    lex.next_token(state)?; // skip REPEAT
    stat_list(lex, state)?;
    check_match(
        lex,
        state,
        Reserved::Until as u32,
        Reserved::Repeat as u32,
        line,
    )?;
    let cond_exit = cond(lex, state)?; // read condition (inside scope block)
    let (upvals, nactvar) = {
        let bl = lex.borrow_fs(None).bl.last().unwrap();
        (bl.upval, bl.nactvar)
    };
    if upvals {
        // upvalues ?
        luaK::patch_close(lex, state, cond_exit, nactvar)?;
    }
    leave_block(lex, state)?; // finish scope
    luaK::patch_list(lex, state, cond_exit, repeat_init)?; // close the loop
    leave_block(lex, state)?; // finish loop
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
    let bl_previous = !lex.borrow_fs(None).bl.is_empty();
    if bl_previous && bl.upval {
        // create a 'jump to here' to close upvalues
        let j = luaK::jump(lex, state)?;
        luaK::patch_close(lex, state, j, bl.nactvar)?;
        luaK::patch_to_here(lex, state, j)?;
    }
    if bl.is_loop {
        break_label(lex, state)?; // close pending breaks
    }
    remove_vars(lex, state, bl.nactvar);
    debug_assert!(bl.nactvar == lex.borrow_fs(None).nactvar);
    lex.borrow_mut_fs(None).freereg = bl.nactvar; // free registers
    lex.dyd.label.truncate(bl.first_label); // remove local labels
    if bl_previous {
        // inner block ?
        move_gotos_out(lex, state, &bl)?; // update pending gotos to outer block
    } else if bl.first_goto > lex.dyd.gt.len() {
        undef_goto(lex, state, bl.first_goto)?;
    }
    Ok(())
}

fn undef_goto<T>(
    _lex: &mut LexState<T>,
    _state: &mut LuaState,
    _goto_idx: usize,
) -> Result<(), LuaError> {
    todo!()
}

/// "export" pending gotos to outer level, to check them against
/// outer labels; if the block being exited has upvalues, and
/// the goto exits the scope of any variable (which can be the
/// upvalue), close those variables being exited.
fn move_gotos_out<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    bl: &BlockCnt,
) -> Result<(), LuaError> {
    // correct pending gotos to current block and try to close it
    // with visible labels
    let mut i = bl.first_goto as usize;
    while i < lex.dyd.gt.len() {
        if lex.dyd.gt[i].nactvar > bl.nactvar {
            if bl.upval {
                luaK::patch_close(lex, state, lex.dyd.gt[i].pc as i32, bl.nactvar)?;
            }
            lex.dyd.gt[i].nactvar = bl.nactvar;
        }
        if !find_label(lex, state, i)? {
            i += 1; // move to next one
        }
    }
    Ok(())
}

/// try to close a goto with existing labels; this solves backward jumps
fn find_label<T>(lex: &mut LexState<T>, state: &mut LuaState, g: usize) -> Result<bool, LuaError> {
    let (first_label, upval) = {
        let bl = lex.borrow_fs(None).bl.last().unwrap();
        (bl.first_label, bl.upval)
    };
    // check labels in current block for a match
    for i in first_label as usize..lex.dyd.label.len() {
        if lex.dyd.label[i].name == lex.dyd.gt[g].name {
            // correct label?
            if lex.dyd.gt[g].nactvar > lex.dyd.label[i].nactvar
                && (upval || lex.dyd.label.len() > first_label as usize)
            {
                luaK::patch_close(
                    lex,
                    state,
                    lex.dyd.gt[g].pc as i32,
                    lex.dyd.label[i].nactvar,
                )?;
            }
            close_goto(lex, state, g, i)?; // close it
            return Ok(true);
        }
    }
    Ok(false) // label not found; cannot close goto
}

fn close_goto<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    g: usize,
    lb: usize,
) -> Result<(), LuaError> {
    debug_assert!(lex.dyd.gt[g].name == lex.dyd.label[lb].name);
    if lex.dyd.gt[g].nactvar < lex.dyd.label[lb].nactvar {
        let vname = &lex.borrow_loc_var(state, None, lex.dyd.gt[g].nactvar).name;
        let msg = format!(
            "<goto {}> at line {} jumps into the scope of local '{}'",
            &lex.dyd.gt[g].name, lex.dyd.gt[g].line, vname
        );
        lex.semantic_error(state, &msg)?;
    }
    luaK::patch_list(
        lex,
        state,
        lex.dyd.gt[g].pc as i32,
        lex.dyd.label[lb].pc as i32,
    )?;
    // remove goto from pending list
    lex.dyd.gt.remove(g);
    Ok(())
}

/// create a label named "break" to resolve break statements
fn break_label<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let (pc, nactvar) = {
        let f = lex.borrow_fs(None);
        (lex.next_pc(state), f.nactvar)
    };
    let l = lex.dyd.label.len();
    lex.dyd
        .label
        .push(LabelDesc::new("break", 0, pc as usize, nactvar));
    find_gotos(lex, state, l)?;
    Ok(())
}

/// check whether new label 'lb' matches any pending gotos in current
/// block; solves forward jumps
fn find_gotos<T>(lex: &mut LexState<T>, state: &mut LuaState, lb: usize) -> Result<(), LuaError> {
    let mut i = lex.borrow_fs(None).bl.last().unwrap().first_goto;
    while i < lex.dyd.gt.len() {
        if lex.dyd.gt[i].name == lex.dyd.label[lb].name {
            close_goto(lex, state, i, lb)?;
        } else {
            i += 1;
        }
    }
    Ok(())
}

/// forlist -> NAME {,NAME} IN explist1 forbody
fn for_list<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    var_name: String,
) -> Result<(), LuaError> {
    let base = lex.borrow_fs(None).freereg;
    // create control variables
    new_localvar(lex, state, "(for generator)".to_owned())?;
    new_localvar(lex, state, "(for state)".to_owned())?;
    new_localvar(lex, state, "(for control)".to_owned())?;
    // create declared variable
    new_localvar(lex, state, var_name)?;
    let mut nvars = 4;
    while test_next(lex, state, ',' as u32)? {
        let next_var_name = str_checkname(lex, state)?;
        new_localvar(lex, state, next_var_name)?;
        nvars += 1;
    }
    check_next(lex, state, Reserved::In as u32)?;
    let line = lex.linenumber;
    let mut e = ExpressionDesc::default();
    let nexps = exp_list(lex, state, &mut e)?;
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
    new_localvar(lex, state, "(for index)".to_owned())?;
    new_localvar(lex, state, "(for limit)".to_owned())?;
    new_localvar(lex, state, "(for step)".to_owned())?;
    new_localvar(lex, state, var_name)?;
    check_next(lex, state, '=' as u32)?;
    exp1(lex, state)?; // initial value
    check_next(lex, state, ',' as u32)?;
    exp1(lex, state)?; // limit
    if test_next(lex, state, ',' as u32)? {
        exp1(lex, state)?; // optional step
    } else {
        // default step = 1
        let k = luaK::number_constant(lex, state, 1.0) as u32;
        code_k(lex, state, lex.borrow_fs(None).freereg as i32, k)?;
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
    adjust_local_vars(lex, state, 3); // control variables
    check_next(lex, state, Reserved::Do as u32)?;
    let prep = if is_num {
        code_asbx(lex, state, OpCode::ForPrep as u32, base as i32, NO_JUMP)? as i32
    } else {
        luaK::jump(lex, state)?
    };
    enter_block(lex, false); // scope for declared variables
    adjust_local_vars(lex, state, nvars);
    reserve_regs(lex, state, nvars)?;
    block(lex, state)?;
    leave_block(lex, state)?; // end of scope for declared variables
    patch_to_here(lex, state, prep)?; // fix the forprep instruction jump
    let endfor = if is_num {
        // numeric for
        code_asbx(lex, state, OpCode::ForLoop as u32, base as i32, NO_JUMP)?
    } else {
        // generic for
        code_abc(
            lex,
            state,
            OpCode::TForCall as u32,
            base as i32,
            0,
            nvars as i32,
        )?;
        fix_line(lex, state, line); // pretend that `OP_FOR' starts the loop
        code_asbx(
            lex,
            state,
            OpCode::TForLoop as u32,
            base as i32 + 2,
            NO_JUMP,
        )?
    };
    patch_list(lex, state, endfor as i32, prep + 1)?;
    fix_line(lex, state, line);
    Ok(())
}

fn exp1<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<ExpressionKind, LuaError> {
    let mut e = ExpressionDesc::default();
    expr(lex, state, &mut e)?;
    let k = e.k;
    exp2nextreg(lex, state, &mut e)?;
    Ok(k)
}

fn enter_block<T>(lex: &mut LexState<T>, is_loop: bool) {
    let (first_label, first_goto) = (lex.dyd.label.len(), lex.dyd.gt.len());
    let fs = lex.borrow_mut_fs(None);
    fs.bl
        .push(BlockCnt::new(is_loop, fs.nactvar, first_label, first_goto));
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

///  block -> statlist
fn block<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    enter_block(lex, false);
    stat_list(lex, state)?;
    leave_block(lex, state)
}

/// whilestat -> WHILE cond DO block END
fn while_stat<T>(lex: &mut LexState<T>, state: &mut LuaState, line: usize) -> Result<(), LuaError> {
    lex.next_token(state)?; // skip WHILE
    let while_init = luaK::get_label(lex, state);
    let cond_exit = cond(lex, state)?;
    enter_block(lex, true);
    check_next(lex, state, Reserved::Do as u32)?;
    block(lex, state)?;
    let list = luaK::jump(lex, state)?;
    luaK::patch_list(lex, state, list, while_init)?;
    check_match(
        lex,
        state,
        Reserved::End as u32,
        Reserved::While as u32,
        line,
    )?;
    leave_block(lex, state)?;
    luaK::patch_to_here(lex, state, cond_exit)?;
    Ok(())
}

/// ifstat -> IF cond THEN block {ELSEIF cond THEN block} [ELSE block] END
fn if_stat<T>(lex: &mut LexState<T>, state: &mut LuaState, line: usize) -> Result<(), LuaError> {
    let mut escape_list = NO_JUMP; // exit list for finished parts
    test_then_block(lex, state, &mut escape_list)?; // IF cond THEN block
    while lex.is_token(Reserved::ElseIf as u32) {
        test_then_block(lex, state, &mut escape_list)?; // ELSEIF cond THEN block
    }
    if test_next(lex, state, Reserved::Else as u32)? {
        block(lex, state)?; // `else' part
    }
    check_match(lex, state, Reserved::End as u32, Reserved::If as u32, line)?;
    luaK::patch_to_here(lex, state, escape_list)?;
    Ok(())
}

/// test_then_block -> [IF | ELSEIF] cond THEN block
fn test_then_block<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    escape_list: &mut i32,
) -> Result<(), LuaError> {
    lex.next_token(state)?; // skip IF or ELSEIF
    let mut v = ExpressionDesc::default();
    expr(lex, state, &mut v)?; // read condition
    check_next(lex, state, Reserved::Then as u32)?;
    let jf = if lex.is_token(Reserved::Goto as u32) || lex.is_token(Reserved::Break as u32) {
        luaK::go_if_false(lex, state, &mut v)?; // will jump to label if condition is true
        enter_block(lex, false); // must enter block before 'goto'
        goto_stat(lex, state, v.t)?; // handle goto/break
        skip_noop_stat(lex, state)?; // skip other no-op statements
        if block_follow(&lex, false) {
            // 'goto' is the entire block?
            leave_block(lex, state)?;
            return Ok(()); // and that is it
        } else {
            // must skip over 'then' part if condition is false
            luaK::jump(lex, state)?
        }
    } else {
        //  regular case (not goto/break)
        luaK::go_if_true(lex, state, &mut v)?; // skip over block if condition is false
        enter_block(lex, false);
        v.f
    };
    stat_list(lex, state)?; // 'then' part
    leave_block(lex, state)?;
    if lex.is_token(Reserved::Else as u32) || lex.is_token(Reserved::Else as u32) {
        // followed by 'else'/'elseif' ?
        let l2 = luaK::jump(lex, state)?;
        luaK::concat(lex, state, escape_list, l2)?; // must jump over it
    }
    luaK::patch_to_here(lex, state, jf)?;
    Ok(())
}

/// statlist -> { stat [`;'] }
fn stat_list<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    while !block_follow(lex, true) {
        if lex.is_token(Reserved::Return as u32) {
            statement(lex, state)?;
            return Ok(()); // 'return' must be last statement
        }
        statement(lex, state)?;
    }
    Ok(())
}

/// skip no-op statements
fn skip_noop_stat<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    while lex.is_token(';' as u32) || lex.is_token(Reserved::DbColon as u32) {
        statement(lex, state)?;
    }
    Ok(())
}

fn goto_stat<T>(lex: &mut LexState<T>, state: &mut LuaState, pc: i32) -> Result<(), LuaError> {
    let line = lex.linenumber;
    let label = if test_next(lex, state, Reserved::Goto as u32)? {
        str_checkname(lex, state)?
    } else {
        lex.next_token(state)?; // skip break
        "break".to_owned()
    };
    let g = lex.dyd.gt.len();
    lex.dyd.gt.push(lex.new_label_entry(label, line, pc));
    find_label(lex, state, g)?; // close it if label already defined
    Ok(())
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
