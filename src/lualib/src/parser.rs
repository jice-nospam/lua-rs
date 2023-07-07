//! Lua Parser

use std::{cell::RefCell, rc::Rc};

use crate::{
    api::LuaError,
    ldo::SParser,
    lex::{LabelDesc, LexState, Reserved, SemInfo, VarDesc, VarKind},
    limits::MAX_UPVAL,
    luaH::{Table, TableRef},
    luaK,
    luaconf::LUAI_MAXVARS,
    object::{LClosure, LocVar, ProtoId, TValue},
    opcodes::{
        get_arg_a, set_arg_bx, set_arg_c, set_opcode, OpCode, LFIELDS_PER_FLUSH, MAXARG_BX, NO_JUMP,
    },
    state::LuaState,
    LuaFloat, LuaInteger, LUA_MULTRET,
};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum BinaryOp {
    Add = 0,
    Sub,
    Mul,
    Mod,
    Pow,
    Div,
    IntDiv,
    BinaryAnd,
    BinaryOr,
    BinaryXor,
    Shl,
    Shr,
    Concat,
    Eq,
    Lt,
    Le,
    Ne,
    Gt,
    Ge,
    And,
    Or,
}

impl TryInto<OpCode> for BinaryOp {
    type Error = ();
    fn try_into(self) -> Result<OpCode, Self::Error> {
        match self {
            BinaryOp::Add => Ok(OpCode::Add),
            BinaryOp::Sub => Ok(OpCode::Sub),
            BinaryOp::Mod => Ok(OpCode::Mod),
            BinaryOp::Pow => Ok(OpCode::Pow),
            BinaryOp::Div => Ok(OpCode::Div),
            BinaryOp::IntDiv => Ok(OpCode::IntegerDiv),
            BinaryOp::BinaryAnd => Ok(OpCode::BinaryAnd),
            BinaryOp::Mul => Ok(OpCode::Mul),
            BinaryOp::BinaryOr => Ok(OpCode::BinaryOr),
            BinaryOp::BinaryXor => Ok(OpCode::BinaryXor),
            BinaryOp::Shl => Ok(OpCode::Shl),
            BinaryOp::Shr => Ok(OpCode::Shr),
            _ => Err(()),
        }
    }
}

struct BinaryPriority {
    left: usize,
    right: usize,
}
const BINARY_OP_PRIO: [BinaryPriority; 21] = [
    BinaryPriority {
        left: 10,
        right: 10,
    }, // Add
    BinaryPriority {
        left: 10,
        right: 10,
    }, // Sub
    BinaryPriority {
        left: 11,
        right: 11,
    }, // Mul
    BinaryPriority {
        left: 11,
        right: 11,
    }, // Mod
    BinaryPriority {
        left: 14,
        right: 13,
    }, // Pow (right associative)
    BinaryPriority {
        left: 11,
        right: 11,
    }, // Div
    BinaryPriority {
        left: 11,
        right: 11,
    }, // IntDiv
    BinaryPriority { left: 6, right: 6 }, // BinaryAnd
    BinaryPriority { left: 4, right: 4 }, // BinaryOr
    BinaryPriority { left: 5, right: 5 }, // BinaryXor
    BinaryPriority { left: 7, right: 7 }, // Shl
    BinaryPriority { left: 7, right: 7 }, // Shr
    BinaryPriority { left: 9, right: 8 }, // Concat (right associative)
    BinaryPriority { left: 3, right: 3 }, // Eq
    BinaryPriority { left: 3, right: 3 }, // Lt
    BinaryPriority { left: 3, right: 3 }, // Le
    BinaryPriority { left: 3, right: 3 }, // Ne
    BinaryPriority { left: 3, right: 3 }, // Gt
    BinaryPriority { left: 3, right: 3 }, // Ge
    BinaryPriority { left: 2, right: 2 }, // And
    BinaryPriority { left: 1, right: 1 }, // Or
];

pub(crate) enum UnaryOp {
    Minus,
    BinaryNot,
    Not,
    Len,
}

/// priority for unary operators
const UNARY_PRIORITY: usize = 12;

/// Description of an upvalue for function prototypes
#[derive(Default, Clone)]
pub struct UpValDesc {
    ///  upvalue name (for debug information)
    pub name: String,
    /// whether it is in stack
    pub in_stack: bool,
    /// index of upvalue (in stack or in outer function's list)
    pub idx: usize,
    /// kind of corresponding variable
    pub kind: VarKind,
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
    pub(crate) upval: bool,
    /// true if `block' is a loop
    is_loop: bool,
    /// true if inside the scope of a to-be-closed var.
    pub(crate) is_inside_tbc: bool,
}

impl BlockCnt {
    fn new(
        is_loop: bool,
        nactvar: usize,
        first_label: usize,
        first_goto: usize,
        is_inside_tbc: bool,
    ) -> Self {
        Self {
            first_label,
            first_goto,
            nactvar,
            upval: false,
            is_loop,
            is_inside_tbc,
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
    /// last line that was saved in 'lineinfo'
    pub previous_line: usize,
    /// list of pending jumps to `pc'
    pub jpc: i32,
    /// index of first local var (in Dyndata array)
    pub first_local: usize,
    /// index of first label var (in Dyndata array)
    pub first_label: usize,
    /// number of active local variables
    pub nactvar: usize,
    /// first free register
    pub freereg: usize,
    /// instructions issued since last absolute line info
    pub iwthabs: u32,
    /// function needs to close upvalues when returning
    pub need_close: bool,
}

impl FuncState {
    pub(crate) fn new(line_defined: usize) -> Self {
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
            first_label: 0,
            nactvar: 0,
            previous_line: line_defined,
            iwthabs: 0,
            need_close: false,
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
    pub(crate) fn borrow_mut_block(&mut self) -> &mut BlockCnt {
        self.bl.last_mut().unwrap()
    }

    /// Add constant 'value' to prototype's list of constants (field 'k').
    /// Use scanner's table to cache position of constants in constant list
    /// and try to reuse constants. Because some values should not be used
    /// as keys (nil cannot be a key, integer keys can collapse with float
    /// keys), the caller must provide a useful 'key' for indexing the cache.
    pub(crate) fn add_constant(
        &mut self,
        state: &mut LuaState,
        key: TValue,
        value: TValue,
    ) -> usize {
        let val = self.h.borrow_mut().get(&key).cloned();
        match val {
            Some(TValue::Integer(n)) => n as usize,
            _ => {
                let kid = state.protos[self.f].k.len();
                self.h
                    .borrow_mut()
                    .set(key, TValue::Integer(kid as LuaInteger));
                state.protos[self.f].k.push(value);
                kid
            }
        }
    }

    pub fn string_constant(&mut self, state: &mut LuaState, value: &str) -> usize {
        let tvalue = TValue::from(value);
        self.add_constant(state, tvalue.clone(), tvalue)
    }

    pub fn float_constant(&mut self, state: &mut LuaState, value: LuaFloat) -> usize {
        let tvalue = TValue::Float(value);
        self.add_constant(state, tvalue.clone(), tvalue)
    }

    pub fn integer_constant(&mut self, state: &mut LuaState, value: LuaInteger) -> usize {
        let tvalue = TValue::Integer(value);
        self.add_constant(state, tvalue.clone(), tvalue)
    }
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum ExpressionKind {
    #[default]
    /// when 'expdesc' describes the last expression of a list, this kind means an empty list (so, no expression)
    Void = 0,
    /// constant nil
    Nil,
    /// constant true
    True,
    /// constant false
    False,
    /// constant in 'k'; info = index of constant in 'k'
    Constant,
    /// floating constant; nval = numerical float value
    FloatConstant,
    /// integer constant; ival = numerical integer value
    IntegerConstant,
    /// string constant; strval = TString address;(string is fixed by the lexer)
    StringConstant,
    /// expression has its value in a fixed register;info = result register
    NonRelocable,
    /// local variable; var.ridx = register index;var.vidx = relative index in 'actvar.arr'
    LocalRegister,
    /// upvalue variable; info = index of upvalue in 'upvalues'
    UpValue,
    /// compile-time <const> variable; info = absolute index in 'actvar.arr'
    CompileTimeConst,
    /// info = table register; aux = index register (or `k')
    Indexed,
    /// indexed upvalue;ind.t = table upvalue;ind.idx = key's K index
    IndexedUpvalue,
    /// indexed variable with constant integer;ind.t = table register;ind.idx = key's value
    IndexedInteger,
    /// indexed variable with literal string;ind.t = table register;ind.idx = key's K index
    IndexedString,
    /// expression is a test/comparison;info = pc of corresponding jump instruction
    Jump,
    /// expression can put result in any register;info = instruction pc
    Relocable,
    /// expression is a function call; info = instruction pc
    Call,
    /// vararg expression; info = instruction pc
    VarArg,
}
impl ExpressionKind {
    fn is_var(&self) -> bool {
        matches!(
            self,
            Self::LocalRegister
                | Self::UpValue
                | Self::Constant
                | Self::Indexed
                | Self::IndexedUpvalue
                | Self::IndexedInteger
                | Self::IndexedString
        )
    }

    fn is_indexed(&self) -> bool {
        matches!(
            self,
            Self::Indexed | Self::IndexedUpvalue | Self::IndexedInteger | Self::IndexedString
        )
    }

    fn has_multret(&self) -> bool {
        matches!(self, Self::Call | Self::VarArg)
    }
}

#[derive(Default, Clone)]
pub struct ExpressionDesc {
    pub k: ExpressionKind,
    /// for local variables
    /// index (R or "long" K)
    pub ind_idx: i32,
    /// table (register or upvalue)
    pub ind_t: usize,
    /// register holding the variable
    pub var_ridx: i32,
    /// compiler index (in 'actvar')
    pub var_vidx: usize,
    /// for generic use
    pub info: i32,
    /// for ExpressionKind::FloatConstant
    pub nval: LuaFloat,
    /// for ExpressionKind::IntegerConstant
    pub ival: LuaInteger,
    /// for ExpressionKind::StringConstant
    pub strval: String,
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
    #[inline]
    pub(crate) fn has_jumps(&self) -> bool {
        self.t != self.f
    }
    pub(crate) fn is_numeral(&self) -> bool {
        if self.has_jumps() {
            false
        } else {
            matches!(
                self.k,
                ExpressionKind::FloatConstant | ExpressionKind::IntegerConstant
            )
        }
    }
    pub(crate) fn to_numeral(&self, v: Option<&mut TValue>) -> bool {
        if self.has_jumps() {
            false
        } else {
            match self.k {
                ExpressionKind::IntegerConstant => {
                    if let Some(v) = v {
                        *v = TValue::from(self.ival);
                    }
                    true
                }
                ExpressionKind::FloatConstant => {
                    if let Some(v) = v {
                        *v = TValue::from(self.nval);
                    }
                    true
                }
                _ => false,
            }
        }
    }

    /// set expression as a string constant
    fn code_string(&mut self, val: &str) {
        self.t = NO_JUMP;
        self.f = NO_JUMP;
        self.k = ExpressionKind::StringConstant;
        self.strval = val.to_owned();
    }

    /// Check whether expression 'e' is a literal integer.
    pub(crate) fn is_k_int(&self) -> bool {
        self.k == ExpressionKind::IntegerConstant && !self.has_jumps()
    }

    /// Create an expression representing variable 'vidx'
    pub(crate) fn init_var(&mut self, vidx: usize, ridx: i32) {
        self.f = NO_JUMP;
        self.t = NO_JUMP;
        self.k = ExpressionKind::LocalRegister;
        self.var_vidx = vidx;
        self.var_ridx = ridx;
    }

    fn is_indexed(&self) -> bool {
        matches!(
            self.k,
            ExpressionKind::Indexed
                | ExpressionKind::IndexedInteger
                | ExpressionKind::IndexedString
                | ExpressionKind::IndexedUpvalue
        )
    }
}

#[derive(Default)]
struct LHSAssignment {
    /// variable (global, local, upvalue, or indexed)
    v: ExpressionDesc,
}

pub fn parser<T>(state: &mut LuaState, parser: &mut SParser<T>) -> Result<LClosure, LuaError> {
    let mut lex = LexState::new(parser.z.take().unwrap(), &parser.name);
    let mut new_fs = FuncState::new(1);
    new_fs.f = state.add_prototype(&lex, &parser.name, 1);
    lex.vfs.push(new_fs);
    // read the first character in the stream
    lex.next_char(state);
    main_func(&mut lex, state)?;
    let cl = LClosure::new(0, 1); //create main closure
    Ok(cl)
}

/// compiles the main function, which is a regular vararg function with an
/// upvalue named _ENV
fn main_func<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let mut v = ExpressionDesc::default();
    open_func(lex, state);
    set_vararg(lex, state, 0)?; // main function is always declared vararg
    v.init(ExpressionKind::LocalRegister, 0);
    let envn = lex.envn.clone();
    let proto = lex.borrow_mut_proto(state, None);
    proto.upvalues.push(UpValDesc {
        name: envn,
        in_stack: true,
        idx: 0,
        kind: VarKind::Regular,
    });
    lex.next_token(state)?; // read first token
    stat_list(lex, state)?; // parse main body
    lex.check_eos(state)?;
    close_func(lex, state)?;
    Ok(())
}

fn set_vararg<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    nparams: i32,
) -> Result<(), LuaError> {
    lex.borrow_mut_proto(state, None).is_vararg = true;
    luaK::code_abc(lex, state, OpCode::VarArgPrep as u32, nparams, 0, 0)?;
    Ok(())
}

fn open_func<T>(lex: &mut LexState<T>, state: &mut LuaState) {
    let first_local = lex.dyd.actvar.len();
    let first_label = lex.dyd.label.len();
    let line_defined = lex.borrow_proto(state, None).line_defined;
    let fs = lex.borrow_mut_fs(None);
    fs.first_local = first_local;
    fs.first_label = first_label;
    fs.previous_line = line_defined;
    enter_block(lex, false);
}

fn close_func<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let nvar = lex.get_nvar_stack() as u32;
    luaK::ret(lex, state, nvar, 0)?; // final return
    leave_block(lex, state)?;
    debug_assert!(lex.borrow_fs(None).bl.is_empty());
    luaK::finish(lex, state)?;
    lex.vfs.pop();
    Ok(())
}

/// Close the scope for all variables up to level 'tolevel'.
/// (debug info.)
fn remove_vars<T>(lex: &mut LexState<T>, state: &mut LuaState, to_level: usize) {
    let mut nactvar = lex.borrow_fs(None).nactvar;
    let vars_to_remove = nactvar - to_level;
    let next_pc = lex.next_pc(state);
    while nactvar > to_level {
        nactvar -= 1;
        if let Some(var) = lex.borrow_mut_local_var(state, nactvar) {
            var.end_pc = next_pc;
        }
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
        _ => false,
    }
}

fn leave_level<T>(_lex: &mut LexState<T>, state: &mut LuaState) {
    state.n_rcalls -= 1;
}

/// Test whether next token is 'c'; if so, skip it.
fn test_next<T>(lex: &mut LexState<T>, state: &mut LuaState, c: u32) -> Result<bool, LuaError> {
    if matches!(&lex.t, Some(t) if t.token == c) {
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
                Ok(Reserved::Break) => {
                    break_stat(lex, state)?;
                }
                // stat -> 'goto' NAME
                Ok(Reserved::Goto) => {
                    lex.next_token(state)?; // skip 'goto'
                    goto_stat(lex, state)?;
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
    let nactvar = lex.get_nvar_stack();
    debug_assert!(
        lex.borrow_mut_proto(state, None).maxstacksize >= lex.borrow_fs(None).freereg
            && lex.borrow_fs(None).freereg >= nactvar
    );
    lex.borrow_mut_fs(None).freereg = nactvar;
    leave_level(lex, state);
    Ok(())
}

/// Break statement. Semantically equivalent to "goto break".
fn break_stat<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let line = lex.linenumber;
    lex.next_token(state)?; // skip break
    let pc = luaK::jump(lex, state)? as i32;
    lex.dyd
        .gt
        .push(lex.new_label_entry("break".to_owned(), line, pc));
    Ok(())
}

/// label -> '::' NAME '::'
fn label_stat<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    name: String,
    line: usize,
) -> Result<(), LuaError> {
    check_next(lex, state, Reserved::DbColon as u32)?; // skip double colon
    loop {
        match &lex.t {
            Some(tok) if tok.token == ';' as u32 || tok.token == Reserved::DbColon as u32 => {
                statement(lex, state)?; // skip other no-op statements
            }
            _ => break,
        }
    }
    lex.check_repeated(state, &name)?;
    let last = block_follow(lex, false);
    create_label(lex, state, &name, line, last)?;
    Ok(())
}

/// Create a new label with the given 'name' at the given 'line'.
/// 'last' tells whether label is the last non-op statement in its
/// block. Solves all pending gotos to this new label and adds
/// a close instruction if necessary.
/// Returns true iff it added a close instruction.
fn create_label<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    name: &str,
    line: usize,
    last: bool,
) -> Result<bool, LuaError> {
    let l = lex.dyd.label.len();
    let pc = luaK::get_label(lex, state);
    lex.dyd
        .label
        .push(lex.new_label_entry(name.to_owned(), line, pc));
    if last {
        // label is last no-op statement in the block?
        // assume that locals are already out of scope
        lex.dyd.label[l].nactvar = lex.borrow_fs(None).borrow_block().nactvar;
    }
    if solve_gotos(lex, state, l)? {
        luaK::code_abc(
            lex,
            state,
            OpCode::Close as u32,
            lex.get_nvar_stack() as i32,
            0,
            0,
        )?;
        return Ok(true);
    }
    Ok(false)
}

/// stat -> func | assignment
fn expr_stat<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let mut lhs = LHSAssignment::default();
    suffixed_expr(lex, state, &mut lhs.v)?;
    if lex.is_token('=' as u32) || lex.is_token(',' as u32) {
        // stat -> assignment ?
        let mut vlhs = Vec::new();
        vlhs.push(lhs);
        rest_assign(lex, state, &mut vlhs, 1)?;
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
    while let Some(t) = lex.t.clone() {
        match t.token as u8 as char {
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
                code_name(lex, state, &mut key)?;
                luaK::op_self(lex, state, v, &mut key)?;
                func_args(lex, state, v, line)?;
            }
            '(' => {
                // funcargs
                luaK::exp2nextreg(lex, state, v)?;
                func_args(lex, state, v, line)?;
            }
            _ => break,
        }
    }
    Ok(())
}

/// Parse and compile a multiple assignment. The first "variable"
/// (a 'suffixedexp') was already read by the caller.
/// assignment -> suffixedexp restassign
/// restassign -> ',' suffixedexp restassign | '=' explist
fn rest_assign<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    lhs: &mut Vec<LHSAssignment>,
    nvars: usize,
) -> Result<(), LuaError> {
    let mut exp = ExpressionDesc::default();
    if !lhs.last().unwrap().v.k.is_var() {
        return lex.syntax_error(state, "syntax error");
    }
    check_readonly(lex, state, &lhs.last().unwrap().v)?;
    if test_next(lex, state, ',' as u32)? {
        // restassign -> ',' suffixedexp restassign
        let mut nv = LHSAssignment::default();
        suffixed_expr(lex, state, &mut nv.v)?;
        if !nv.v.k.is_indexed() {
            check_conflict(lex, state, &nv.v, lhs)?;
        }
        lhs.push(nv);
        enter_level(lex, state)?;
        rest_assign(lex, state, lhs, nvars + 1)?;
        leave_level(lex, state);
        lhs.pop();
    } else {
        // restassign -> '=' explist
        check_next(lex, state, '=' as u32)?;
        let nexps = exp_list(lex, state, &mut exp)?;
        if nexps != nvars {
            adjust_assign(lex, state, nvars, nexps, &mut exp)?;
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

/// Raises an error if variable described by 'e' is read only
fn check_readonly<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    e: &ExpressionDesc,
) -> Result<(), LuaError> {
    if let Some(var_name) = match e.k {
        ExpressionKind::Constant => Some(lex.dyd.actvar[e.info as usize].name.to_owned()),
        ExpressionKind::LocalRegister => {
            let var_desc = lex.borrow_loc_var_desc(None, e.var_vidx);
            if var_desc.kind != VarKind::Regular {
                Some(var_desc.name.to_owned())
            } else {
                None
            }
        }
        ExpressionKind::UpValue => {
            let up = &lex.borrow_proto(state, None).upvalues[e.info as usize];
            if up.kind != VarKind::Regular {
                Some(up.name.to_owned())
            } else {
                None
            }
        }
        _ => return Ok(()), //other cases cannot be read-only
    } {
        let msg = format!("attempt to assign to const variable '{}'", &var_name);
        return lex.semantic_error(state, &msg);
    }
    Ok(())
}

/// Check that next token is 'c' and skip it.
fn check_next<T>(lex: &mut LexState<T>, state: &mut LuaState, c: u32) -> Result<(), LuaError> {
    check(lex, state, c)?;
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
        if lh.v.is_indexed() {
            // assignment to table field?
            // table is the upvalue/local being assigned now?
            if lh.v.k == ExpressionKind::IndexedUpvalue {
                // is table an upvalue?
                if v.k == ExpressionKind::UpValue && lh.v.ind_t as i32 == v.info {
                    conflict = true; // table is the upvalue being assigned now
                    lh.v.k = ExpressionKind::IndexedString;
                    lh.v.ind_t = extra as usize; // assignment will use safe copy
                }
            } else {
                // table is a register
                if v.k == ExpressionKind::LocalRegister && lh.v.ind_idx == v.var_ridx {
                    conflict = true; // table is the local being assigned now
                    lh.v.ind_t = extra as usize; //  assignment will use safe copy
                }
                //  is index the local being assigned?
                if lh.v.k == ExpressionKind::Indexed
                    && v.k == ExpressionKind::LocalRegister
                    && lh.v.ind_idx == v.var_ridx
                {
                    conflict = true;
                    lh.v.ind_idx = extra as i32; //previous assignment will use safe copy
                }
            }
        }
    }
    if conflict {
        // copy upvalue/local value to a temporary (in position 'extra')
        if v.k == ExpressionKind::LocalRegister {
            luaK::code_abc(lex, state, OpCode::Move as u32, extra as i32, v.var_ridx, 0)?;
        } else {
            luaK::code_abc(lex, state, OpCode::GetUpVal as u32, extra as i32, v.info, 0)?;
        }
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
                if args.k.has_multret() {
                    luaK::set_mult_ret(lex, state, &mut args)?;
                }
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
                args.code_string(s);
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
    let nparams = if args.k.has_multret() {
        LUA_MULTRET as u32 // open call
    } else {
        if args.k != ExpressionKind::Void {
            luaK::exp2nextreg(lex, state, &mut args)?;
        }
        lex.borrow_fs(None).freereg as u32 - (base + 1)
    };
    exp.init(
        ExpressionKind::Call,
        luaK::code_abc(
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

/// constructor -> '{' [ field { sep field } [sep] ] '}'
/// sep -> ',' | ';'
fn constructor<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    t: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    let line = lex.linenumber;
    let pc = luaK::code_abc(lex, state, OpCode::NewTable as u32, 0, 0, 0)?;
    let mut cc = ConstructorControl::default();
    luaK::code(lex, state, 0)?; // space for extra arg
    t.init(
        ExpressionKind::NonRelocable,
        lex.borrow_fs(None).freereg as i32, // table will be at stack top
    );
    luaK::reserve_regs(lex, state, 1)?;
    cc.v.init(ExpressionKind::Void, 0); // no value yet
    check_next(lex, state, '{' as u32)?;
    loop {
        debug_assert!(cc.v.k == ExpressionKind::Void || cc.to_store > 0);
        if lex.is_token('}' as u32) {
            break;
        }
        close_list_field(lex, state, &mut cc, t)?;
        field(lex, state, &mut cc, t)?;
        if !test_next(lex, state, ',' as u32)? && !test_next(lex, state, ';' as u32)? {
            break;
        }
    }
    check_match(lex, state, '}' as u32, '{' as u32, line)?;
    last_list_field(lex, state, &mut cc, t)?;
    luaK::set_table_size(lex, state, pc as usize, t.info)?;
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
    luaK::exp2nextreg(lex, state, &mut cc.v)?;
    cc.v.k = ExpressionKind::Void;
    if cc.to_store == LFIELDS_PER_FLUSH as usize {
        luaK::set_list(lex, state, exp.info, cc.na as i32, cc.to_store as i32)?; // flush
        cc.na += cc.to_store;
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
        code_name(lex, state, &mut key)?;
    } else {
        // lex.t.token == '['
        yindex(lex, state, &mut key)?;
    }
    cc.nh += 1;
    check_next(lex, state, '=' as u32)?;
    let mut tab = exp.clone();
    luaK::indexed(lex, state, &mut tab, &mut key)?;
    expr(lex, state, &mut val)?;
    luaK::store_var(lex, state, &mut tab, &mut val)?;
    lex.borrow_mut_fs(None).freereg = reg; // free registers
    Ok(())
}

fn code_name<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    key: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    let name = str_checkname(lex, state)?;
    key.code_string(&name);
    Ok(())
}

/// listfield -> exp
fn list_field<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    cc: &mut ConstructorControl,
) -> Result<(), LuaError> {
    expr(lex, state, &mut cc.v)?;
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
    let remove_last_expression = if cc.v.k.has_multret() {
        luaK::set_mult_ret(lex, state, &mut cc.v)?;
        luaK::set_list(lex, state, exp.info, cc.na as i32, LUA_MULTRET)?;
        true // do not count last expression (unknown number of elements)
    } else {
        if cc.v.k != ExpressionKind::Void {
            luaK::exp2nextreg(lex, state, &mut cc.v)?;
        }
        luaK::set_list(lex, state, exp.info, cc.na as i32, cc.to_store as i32)?;
        false
    };
    cc.na += cc.to_store;
    if remove_last_expression {
        cc.na -= 1;
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
        luaK::exp2nextreg(lex, state, exp)?;
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

/// Find a variable with the given name, handling global variables
/// too.
fn single_var<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    var: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    let name = str_checkname(lex, state)?;
    single_var_aux(lex, state, lex.vfs.len() - 1, &name, var, true)?;
    if var.k == ExpressionKind::Void {
        // global name?
        let mut key = ExpressionDesc::default();
        // get environment variable
        let envn = lex.envn.clone();
        single_var_aux(lex, state, lex.vfs.len() - 1, &envn, var, true)?; // get environment variable
        debug_assert!(var.k != ExpressionKind::Void); // this one must exist
        luaK::exp2anyregup(lex, state, var)?; // but could be a constant
        key.code_string(&name); // key is variable name
        luaK::indexed(lex, state, var, &mut key)?; // env[varname]
    }
    Ok(())
}

/// Find a variable with the given name. If it is an upvalue, add
/// this upvalue into all intermediate functions. If it is a global, set
/// 'var' as 'void' as a flag.
fn single_var_aux<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    fsid: usize,
    name: &str,
    var: &mut ExpressionDesc,
    base: bool,
) -> Result<(), LuaError> {
    // look up at current level
    if let Some(v) = lex.search_var(Some(fsid), name, var) {
        if v == ExpressionKind::LocalRegister && !base {
            // local will be used as an upval
            lex.borrow_mut_fs(Some(fsid)).mark_upval(var.var_vidx);
        }
    } else {
        // not found as local at current level; try upvalues
        let mut idx = search_upvalues(lex, state, fsid, name);
        if idx.is_none() {
            if fsid == 0 {
                // no more levels. var is global
                var.init(ExpressionKind::Void, 0);
                return Ok(());
            }
            let prev_fsid = fsid - 1;
            // not found ?
            // try upper levels
            single_var_aux(lex, state, prev_fsid, name, var, base)?;
            if var.k == ExpressionKind::LocalRegister || var.k == ExpressionKind::UpValue {
                // will be a new upvalue
                idx = Some(new_upvalue(lex, state, Some(fsid), name, var)?);
            } else {
                // not found; is a global or a constant
                // don't need to do anything at this level
                return Ok(());
            }
        }
        var.init(ExpressionKind::UpValue, idx.unwrap() as i32);
    }
    Ok(())
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
    let upval_desc = {
        let prev_fsid = fsid.map(|v| v - 1);
        let (in_stack, idx, kind) = if exp.k == ExpressionKind::LocalRegister {
            (
                true,
                exp.var_ridx as usize,
                &lex.borrow_loc_var_desc(prev_fsid, exp.var_vidx).kind,
            )
        } else {
            let protoid = lex.borrow_fs(prev_fsid).f;
            (
                false,
                exp.info as usize,
                &state.protos[protoid].upvalues[exp.info as usize].kind,
            )
        };
        UpValDesc {
            name: name.to_owned(),
            in_stack,
            idx,
            kind: *kind,
        }
    };
    let proto = lex.borrow_mut_proto(state, fsid);
    proto.upvalues.push(upval_desc);
    Ok(proto.upvalues.len() - 1)
}

/// Search the upvalues of the function 'fsid' for one
/// with the given 'name'.
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
        luaK::pos_fix(lex, state, op, exp, &mut exp2, line)?;
        oop = nextop;
    }
    leave_level(lex, state);
    Ok(oop) // return first untreated operator
}

/// simpleexp -> FLOAT | INTEGER | STRING | NIL | TRUE | FALSE | ... |
/// constructor | FUNCTION body | suffixedexp
fn simple_exp<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    match &lex.t.clone() {
        Some(t) if t.token == Reserved::Float as u32 => {
            if let SemInfo::Number(val) = t.seminfo {
                exp.init(ExpressionKind::FloatConstant, 0);
                exp.nval = val;
            } else {
                unreachable!()
            }
        }
        Some(t) if t.token == Reserved::Integer as u32 => {
            if let SemInfo::Integer(val) = t.seminfo {
                exp.init(ExpressionKind::IntegerConstant, 0);
                exp.ival = val;
            } else {
                unreachable!()
            }
        }
        Some(t) if t.token == Reserved::String as u32 => {
            if let SemInfo::String(s) = &t.seminfo {
                exp.code_string(s);
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
                luaK::code_abc(lex, state, OpCode::VarArg as u32, 0, 0, 1)? as i32,
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
    let mut new_fs = FuncState::new(line);
    new_fs.f = state.add_prototype(lex, &lex.source, line);
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
/// The OpCode::Closure instruction uses the last available register
fn code_closure<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    let fs = lex.vfs.pop().unwrap();
    let funcnum = lex.borrow_proto(state, None).p.len() as u32 - 1;
    exp.init(
        ExpressionKind::Relocable,
        luaK::code_abx(lex, state, OpCode::Closure as u32, 0, funcnum)? as i32,
    );
    luaK::exp2nextreg(lex, state, exp)?; // fix it at the last register
    lex.vfs.push(fs);
    Ok(())
}

/// parlist -> [ {NAME ','} (NAME | '...') ]
fn parameter_list<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let mut nparams = 0;
    let mut is_vararg = false;
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
                    is_vararg = true;
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
    if is_vararg {
        set_vararg(lex, state, nactvar as i32)?;
    }
    // reserve register for parameters
    luaK::reserve_regs(lex, state, nactvar)
}

fn unary_op(t: &Option<crate::lex::Token>) -> Option<UnaryOp> {
    match t {
        Some(t) if t.token == Reserved::Not as u32 => Some(UnaryOp::Not),
        Some(t) if t.token == '-' as u32 => Some(UnaryOp::Minus),
        Some(t) if t.token == '~' as u32 => Some(UnaryOp::BinaryNot),
        Some(t) if t.token == '#' as u32 => Some(UnaryOp::Len),
        _ => None,
    }
}

fn binary_op(t: &Option<crate::lex::Token>) -> Option<BinaryOp> {
    match t {
        Some(t) if t.token == '+' as u32 => Some(BinaryOp::Add),
        Some(t) if t.token == '-' as u32 => Some(BinaryOp::Sub),
        Some(t) if t.token == '*' as u32 => Some(BinaryOp::Mul),
        Some(t) if t.token == '%' as u32 => Some(BinaryOp::Mod),
        Some(t) if t.token == '^' as u32 => Some(BinaryOp::Pow),
        Some(t) if t.token == '/' as u32 => Some(BinaryOp::Div),
        Some(t) if t.token == Reserved::IntDiv as u32 => Some(BinaryOp::IntDiv),
        Some(t) if t.token == '&' as u32 => Some(BinaryOp::BinaryAnd),
        Some(t) if t.token == '|' as u32 => Some(BinaryOp::BinaryOr),
        Some(t) if t.token == '~' as u32 => Some(BinaryOp::BinaryXor),
        Some(t) if t.token == Reserved::Shl as u32 => Some(BinaryOp::Shl),
        Some(t) if t.token == Reserved::Shr as u32 => Some(BinaryOp::Shr),
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
    let mut first = lex.get_nvar_stack(); //  first slot to be returned
    let mut nret; // number of values being returned
    let mut e = ExpressionDesc::default();
    if block_follow(lex, true) || lex.is_token(';' as u32) {
        nret = 0; // return no values
    } else {
        nret = exp_list(lex, state, &mut e)? as i32; // optional return values
        if e.k.has_multret() {
            luaK::set_mult_ret(lex, state, &mut e)?;
            if e.k == ExpressionKind::Call
                && nret == 1
                && !lex.borrow_fs(None).borrow_block().is_inside_tbc
            {
                // tail call ?
                set_opcode(
                    lex.borrow_mut_code(state, e.info as usize),
                    OpCode::TailCall as u32,
                );
                debug_assert!(
                    get_arg_a(lex.get_code(state, e.info as usize)) == lex.get_nvar_stack() as u32
                );
            }
            nret = LUA_MULTRET; // return all values
        } else if nret == 1 {
            // only one single value?
            first = luaK::exp2anyreg(lex, state, &mut e)? as usize; // can use original slot
        } else {
            luaK::exp2nextreg(lex, state, &mut e)?; // values must go to the `stack'
            debug_assert!(nret as usize == lex.borrow_fs(None).freereg - first);
        }
    }
    luaK::ret(lex, state, first as u32, nret as u32)?;
    test_next(lex, state, ';' as u32)?; // skip optional semicolon
    Ok(())
}

/// stat -> LOCAL NAME ATTRIB { ',' NAME ATTRIB } ['=' explist]
fn local_stat<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let mut nvars = 0;
    let nexps;
    let mut vidx;
    let mut to_close = None;
    let mut exp = ExpressionDesc::default();
    loop {
        let var_name = str_checkname(lex, state)?;
        vidx = new_localvar(lex, state, var_name)?;
        let kind = get_local_attribute(lex, state)?;
        lex.borrow_mut_loc_var_desc(None, vidx).kind = kind;
        if kind == VarKind::ToBeClosed {
            // to-be-closed?
            if to_close.is_some() {
                // one already present?
                lex.semantic_error(state, "multiple to-be-closed variables in local list")?;
            }
            to_close = Some(lex.borrow_fs(None).nactvar + nvars);
        }
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
    let kind = lex.borrow_loc_var_desc(None, vidx).kind; // get last variable
    let mut value = lex.borrow_loc_var_desc(None, vidx).k.clone();
    if nvars == nexps // no adjustments?
        && kind == VarKind::Const // last variable is const?
        && luaK::exp2const(lex,  &exp, &mut value)?
    // compile-time constant?
    {
        lex.borrow_mut_loc_var_desc(None, vidx).kind = VarKind::CompileTimeConst; // variable is a compile-time constant
        lex.borrow_mut_loc_var_desc(None, vidx).k = value; // variable is a compile-time constant
        adjust_local_vars(lex, state, nvars - 1); // exclude last variable
        lex.borrow_mut_fs(None).nactvar += 1; // but count it
    } else {
        adjust_assign(lex, state, nvars, nexps, &mut exp)?;
        adjust_local_vars(lex, state, nvars);
    }
    check_to_close(lex, state, to_close)?;
    Ok(())
}

fn check_to_close<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    level: Option<usize>,
) -> Result<(), LuaError> {
    if let Some(level) = level {
        lex.mark_to_be_closed();
        luaK::code_abc(
            lex,
            state,
            OpCode::ToBeClosed as u32,
            lex.reg_level(level) as i32,
            0,
            0,
        )?;
    }
    Ok(())
}

/// ATTRIB -> ['<' Name '>']
fn get_local_attribute<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
) -> Result<VarKind, LuaError> {
    if test_next(lex, state, '<' as u32)? {
        let attr = str_checkname(lex, state)?;
        check_next(lex, state, '>' as u32)?;
        if attr == "const" {
            return Ok(VarKind::Const); // read-only variable
        } else if attr == "close" {
            return Ok(VarKind::ToBeClosed); // to-be-closed variable
        } else {
            lex.semantic_error(state, &format!("unknown attribute '{}'", attr))?;
        }
    }
    Ok(VarKind::Regular)
}

/// Adjust the number of results from an expression list 'e' with 'nexps'
/// expressions to 'nvars' values.
fn adjust_assign<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    nvars: usize,
    nexps: usize,
    e: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    let needed = nvars as i32 - nexps as i32; // extra values needed
    if e.k.has_multret() {
        // last expression has multiple returns?
        let extra = (needed + 1).max(0); // discount last expression itself
        luaK::set_returns(lex, state, e, extra)?; // last exp. provides the difference
    } else {
        if e.k != ExpressionKind::Void {
            // at least one expression?
            luaK::exp2nextreg(lex, state, e)?; // close last expression
        }
        if needed > 0 {
            // missing values?
            let reg = lex.borrow_fs(None).freereg;
            luaK::nil(lex, state, reg as u32, needed)?; // complete with nils
        }
    }
    if needed > 0 {
        luaK::reserve_regs(lex, state, needed as usize)?; // registers for extra values
    } else {
        let free_reg = lex.borrow_fs(None).freereg as i32;
        lex.borrow_mut_fs(None).freereg = (free_reg + needed) as usize; //  remove extra values
    }
    Ok(())
}

fn local_func<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let var_name = str_checkname(lex, state)?;
    new_localvar(lex, state, var_name)?; // new local variable
    let fvar = lex.borrow_fs(None).nactvar; // function's variable index
    adjust_local_vars(lex, state, 1); // enter its scope
    let mut b = ExpressionDesc::default();
    body(lex, state, &mut b, false, lex.linenumber)?; // function created in next register
                                                      // debug information will only see the variable after this point!
    let pc = lex.next_pc(state);
    lex.borrow_mut_local_var(state, fvar).unwrap().start_pc = pc;
    Ok(())
}

/// Start the scope for the last 'nvars' created variables.
fn adjust_local_vars<T>(lex: &mut LexState<T>, state: &mut LuaState, nvars: usize) {
    let mut reg_level = lex.get_nvar_stack();
    for _ in 0..nvars {
        let vidx = lex.borrow_fs(None).nactvar;
        lex.borrow_mut_fs(None).nactvar += 1;
        let name = lex.borrow_loc_var_desc(None, vidx).name.to_owned();
        let pidx = register_local_var(lex, state, name);
        let var = lex.borrow_mut_loc_var_desc(None, vidx);
        var.ridx = reg_level;
        reg_level += 1;
        var.pidx = pidx;
    }
}

/// Create a new local variable with the given 'name'. Return its index
/// in the function.
fn new_localvar<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    name: String,
) -> Result<usize, LuaError> {
    let first_local = lex.borrow_fs(None).first_local;
    let id = lex.dyd.actvar.len();
    if lex.dyd.actvar.len() + 1 - first_local > LUAI_MAXVARS {
        lex.error_limit(state, LUAI_MAXVARS, "local variables")?;
    }
    lex.dyd.actvar.push(VarDesc {
        value: TValue::Nil,
        kind: VarKind::Regular,
        ridx: 0,
        pidx: 0,
        name,
        k: TValue::Nil,
    });
    Ok(id - first_local)
}

/// Register a new local variable in the active 'Proto' (for debug
/// information).
fn register_local_var<T>(lex: &mut LexState<T>, state: &mut LuaState, name: String) -> usize {
    let pc = lex.next_pc(state);
    let proto = lex.borrow_mut_proto(state, None);
    proto.locvars.push(LocVar {
        name,
        start_pc: pc,
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
    check_readonly(lex, state, &v)?;
    luaK::store_var(lex, state, &v, &mut b)?;
    luaK::fix_line(lex, state, line); // definition `happens' in the first line
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
    luaK::exp2anyregup(lex, state, v)?;
    lex.next_token(state)?; // skip the dot or colon
    code_name(lex, state, &mut key)?;
    luaK::indexed(lex, state, v, &mut key)
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
    let mut cond_exit = cond(lex, state)?; // read condition (inside scope block)
    leave_block(lex, state)?; // finish scope
    let (upvals, nactvar) = {
        let bl = lex.borrow_fs(None).bl.last().unwrap();
        (bl.upval, bl.nactvar)
    };
    if upvals {
        // upvalues ?
        let exit = luaK::jump(lex, state)? as i32; // normal exit must jump over fix
        luaK::patch_to_here(lex, state, cond_exit)?; //repetition must close upvalues
        luaK::code_abc(
            lex,
            state,
            OpCode::Close as u32,
            lex.reg_level(nactvar) as i32,
            0,
            0,
        )?;
        cond_exit = luaK::jump(lex, state)? as i32; // repeat after closing upvalues
        luaK::patch_to_here(lex, state, exit)?; //normal exit comes to here
    }
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
    let mut has_close = false;
    let bl_previous = lex.borrow_fs(None).bl.len() > 1;
    let bl = lex.borrow_mut_fs(None).bl.pop().unwrap();
    let stk_level = lex.reg_level(bl.nactvar); // level outside the block
    remove_vars(lex, state, bl.nactvar); // remove block locals
    debug_assert!(bl.nactvar == lex.borrow_fs(None).nactvar);
    if bl.is_loop {
        // has to fix pending breaks
        has_close = create_label(lex, state, "break", 0, false)?;
    }
    if !has_close && bl_previous && bl.upval {
        // still need a 'close'?
        luaK::code_abc(lex, state, OpCode::Close as u32, stk_level as i32, 0, 0)?;
    }
    lex.borrow_mut_fs(None).freereg = stk_level; // free registers
    lex.dyd.label.truncate(bl.first_label); // remove local labels
    if bl_previous {
        // inner block ?
        move_gotos_out(lex, &bl)?; // update pending gotos to outer block
    } else if bl.first_goto < lex.dyd.gt.len() {
        undef_goto(lex, state, bl.first_goto)?;
    }
    Ok(())
}

/// generates an error for an undefined 'goto'.
fn undef_goto<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    goto_idx: usize,
) -> Result<(), LuaError> {
    let msg = if lex.dyd.gt[goto_idx].name == "break" {
        format!("break outside loop at line {}", lex.dyd.gt[goto_idx].line)
    } else {
        format!(
            "no visible label '{}' for <goto> at line {}",
            &lex.dyd.gt[goto_idx].name, lex.dyd.gt[goto_idx].line
        )
    };
    lex.semantic_error(state, &msg)
}

/// Adjust pending gotos to outer level of a block.
fn move_gotos_out<T>(lex: &mut LexState<T>, bl: &BlockCnt) -> Result<(), LuaError> {
    // correct pending gotos to current block
    let mut i = bl.first_goto;
    // for each pending goto
    while i < lex.dyd.gt.len() {
        // leaving a variable scope?
        if lex.reg_level(lex.dyd.gt[i].nactvar) > lex.reg_level(bl.nactvar) {
            lex.dyd.gt[i].close = lex.dyd.gt[i].close || bl.upval; // jump may need a close
        }
        lex.dyd.gt[i].nactvar = bl.nactvar; // update goto level
        i += 1;
    }
    Ok(())
}

/// Search for an active label with the given name.
fn find_label<'a, T>(lex: &'a LexState<T>, name: &str) -> Option<&'a LabelDesc> {
    let first_label = lex.borrow_fs(None).first_label;
    // check labels in current function for a match
    for i in first_label..lex.dyd.label.len() {
        if lex.dyd.label[i].name == name {
            return Some(&lex.dyd.label[i]);
        }
    }
    None
}

/// Solves forward jumps. Check whether new label 'lb' matches any
/// pending gotos in current block and solves them. Return true
/// if any of the gotos need to close upvalues.
fn solve_gotos<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    lb: usize,
) -> Result<bool, LuaError> {
    let mut i = lex.borrow_fs(None).borrow_block().first_goto;
    let mut needs_close = false;
    while i < lex.dyd.gt.len() {
        if lex.dyd.gt[i].name == lex.dyd.label[lb].name {
            needs_close = needs_close || lex.dyd.gt[i].close;
            solve_goto(lex, state, i, lb)?; // will remove 'i' from the list
        } else {
            i += 1;
        }
    }
    Ok(needs_close)
}

/// Solves the goto at index 'g' to given label 'lb' and removes it
/// from the list of pending gotos.
/// If it jumps into the scope of some variable, raises an error.
fn solve_goto<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    g: usize,
    lb: usize,
) -> Result<(), LuaError> {
    debug_assert!(lex.dyd.gt[g].name == lex.dyd.label[lb].name);
    if lex.dyd.gt[g].nactvar < lex.dyd.label[lb].nactvar {
        // enter some scope?
        jump_scope_error(lex, state, g)?;
    }
    luaK::patch_list(
        lex,
        state,
        lex.dyd.gt[g].pc as i32,
        lex.dyd.label[lb].pc as i32,
    )?;
    lex.dyd.gt.remove(g); // remove goto from pending list
    Ok(())
}

/// Generates an error that a goto jumps into the scope of some
/// local variable.
fn jump_scope_error<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    g: usize,
) -> Result<(), LuaError> {
    let var_name = &lex.borrow_loc_var_desc(None, lex.dyd.gt[g].nactvar).name;
    let msg = format!(
        "<goto {}> at line {} jumps into the scope of local '{}'",
        lex.dyd.gt[g].name, lex.dyd.gt[g].line, var_name
    );
    lex.semantic_error(state, &msg)
}

/// forlist -> NAME {,NAME} IN explist1 forbody
fn for_list<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    index_name: String,
) -> Result<(), LuaError> {
    let base = lex.borrow_fs(None).freereg;
    let mut nvars = 5; // gen, state, control, toclose, 'indexname'
                       // create control variables
    new_localvar(lex, state, "(for state)".to_owned())?;
    new_localvar(lex, state, "(for state)".to_owned())?;
    new_localvar(lex, state, "(for state)".to_owned())?;
    new_localvar(lex, state, "(for state)".to_owned())?;
    // create declared variable
    new_localvar(lex, state, index_name)?;
    while test_next(lex, state, ',' as u32)? {
        let next_var_name = str_checkname(lex, state)?;
        new_localvar(lex, state, next_var_name)?;
        nvars += 1;
    }
    check_next(lex, state, Reserved::In as u32)?;
    let line = lex.linenumber;
    let mut e = ExpressionDesc::default();
    let nexps = exp_list(lex, state, &mut e)?;
    adjust_assign(lex, state, 4, nexps, &mut e)?;
    adjust_local_vars(lex, state, 4);
    lex.mark_to_be_closed();
    luaK::check_stack(lex, state, 3)?; // extra space to call generator
    for_body(lex, state, base, line, nvars - 4, true)
}

/// fornum -> NAME = exp,exp[,exp] forbody
fn for_num<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    var_name: String,
    line: usize,
) -> Result<(), LuaError> {
    let base = lex.borrow_fs(None).freereg;
    new_localvar(lex, state, "(for state)".to_owned())?;
    new_localvar(lex, state, "(for state)".to_owned())?;
    new_localvar(lex, state, "(for state)".to_owned())?;
    new_localvar(lex, state, var_name)?;
    check_next(lex, state, '=' as u32)?;
    exp1(lex, state)?; // initial value
    check_next(lex, state, ',' as u32)?;
    exp1(lex, state)?; // limit
    if test_next(lex, state, ',' as u32)? {
        exp1(lex, state)?; // optional step
    } else {
        // default step = 1
        luaK::integer(lex, state, lex.borrow_fs(None).freereg as u32, 1)?;
        luaK::reserve_regs(lex, state, 1)?;
    }
    adjust_local_vars(lex, state, 3); // control variables
    for_body(lex, state, base, line, 1, false)
}

/// forbody -> DO block
fn for_body<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    base: usize,
    line: usize,
    nvars: usize,
    is_gen: bool,
) -> Result<(), LuaError> {
    let forprep_op = if is_gen {
        OpCode::TForPrep
    } else {
        OpCode::ForPrep
    };
    let forloop_op = if is_gen {
        OpCode::TForLoop
    } else {
        OpCode::ForLoop
    };
    check_next(lex, state, Reserved::Do as u32)?;
    let prep = luaK::code_abx(lex, state, forprep_op as u32, base as i32, 0)? as i32;
    enter_block(lex, false); // scope for declared variables
    adjust_local_vars(lex, state, nvars);
    luaK::reserve_regs(lex, state, nvars)?;
    block(lex, state)?;
    leave_block(lex, state)?; // end of scope for declared variables
    let dest = luaK::get_label(lex, state);
    fix_for_jump(lex, state, prep, dest, false)?;
    if is_gen {
        luaK::code_abc(
            lex,
            state,
            OpCode::TForCall as u32,
            base as i32,
            0,
            nvars as i32,
        )?;
        luaK::fix_line(lex, state, line);
    }
    let endfor = luaK::code_abx(lex, state, forloop_op as u32, base as i32, 0)? as i32;
    fix_for_jump(lex, state, endfor, prep + 1, true)?;
    luaK::fix_line(lex, state, line);
    Ok(())
}

/// Fix for instruction at position 'pc' to jump to 'dest'.
/// (Jump addresses are relative in Lua). 'back' true means
/// a back jump.
fn fix_for_jump<T>(
    lex: &mut LexState<T>,
    state: &mut LuaState,
    pc: i32,
    dest: i32,
    back: bool,
) -> Result<(), LuaError> {
    let mut offset = dest - (pc + 1);
    if back {
        offset = -offset;
    }
    if offset > MAXARG_BX as i32 {
        lex.syntax_error(state, "control structure too long")?;
    }
    let jmp = lex.borrow_mut_code(state, pc as usize);
    set_arg_bx(jmp, offset as u32);
    Ok(())
}

/// Read an expression and generate code to put its results in next
/// stack slot.
fn exp1<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let mut e = ExpressionDesc::default();
    expr(lex, state, &mut e)?;
    luaK::exp2nextreg(lex, state, &mut e)?;
    debug_assert!(e.k == ExpressionKind::NonRelocable);
    Ok(())
}

fn enter_block<T>(lex: &mut LexState<T>, is_loop: bool) {
    let (first_label, first_goto) = (lex.dyd.label.len(), lex.dyd.gt.len());
    let fs = lex.borrow_mut_fs(None);
    let is_inside_tbc = !fs.bl.is_empty() && fs.bl.last().unwrap().is_inside_tbc;
    fs.bl.push(BlockCnt::new(
        is_loop,
        fs.nactvar,
        first_label,
        first_goto,
        is_inside_tbc,
    ));
    debug_assert!(fs.freereg == lex.get_nvar_stack());
}

/// Check that next token is 'what' and skip it. In case of error,
/// raise an error that the expected 'what' should match a 'who'
/// in line 'line' (if that is not the current line).
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
            "{} expected (to close {} at line {})",
            lex.token_2_txt(what),
            lex.token_2_txt(who),
            line
        );
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
    let list = luaK::jump(lex, state)? as i32;
    luaK::patch_list(lex, state, list, while_init)?;
    check_match(
        lex,
        state,
        Reserved::End as u32,
        Reserved::While as u32,
        line,
    )?;
    leave_block(lex, state)?;
    luaK::patch_to_here(lex, state, cond_exit)?; // false conditions finish the loop
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
    let jf = if lex.is_token(Reserved::Break as u32) {
        // if x then break
        let line = lex.linenumber;
        luaK::go_if_false(lex, state, &mut v)?; // will jump to label if condition is true
        lex.next_token(state)?; // skip 'break'
        enter_block(lex, false); // must enter block before 'goto'
        lex.dyd
            .gt
            .push(lex.new_label_entry("break".to_owned(), line, v.t));
        while test_next(lex, state, ';' as u32)? {} // skip colons
        if block_follow(lex, false) {
            // jump the entire block?
            leave_block(lex, state)?;
            return Ok(()); // and that is it
        } else {
            // must skip over 'then' part if condition is false
            luaK::jump(lex, state)? as i32
        }
    } else {
        // regular case (not a break)
        luaK::go_if_true(lex, state, &mut v)?; // skip over block if condition is false
        enter_block(lex, false);
        v.f
    };
    stat_list(lex, state)?; // 'then' part
    leave_block(lex, state)?;
    if lex.is_token(Reserved::Else as u32) || lex.is_token(Reserved::ElseIf as u32) {
        // followed by 'else'/'elseif' ?
        let l2 = luaK::jump(lex, state)? as i32;
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

fn goto_stat<T>(lex: &mut LexState<T>, state: &mut LuaState) -> Result<(), LuaError> {
    let line = lex.linenumber;
    let name = str_checkname(lex, state)?; // label's name
    let (nactvar, pc) = match find_label(lex, &name) {
        Some(lb) => (Some(lb.nactvar), Some(lb.pc)),
        None => (None, None),
    };
    match (nactvar, pc) {
        (Some(nactvar), Some(pc)) => {
            // found a label
            // backward jump; will be resolved here
            let lb_level = lex.reg_level(nactvar);
            if lex.get_nvar_stack() > lb_level {
                // leaving the scope of a variable?
                luaK::code_abc(lex, state, OpCode::Close as u32, lb_level as i32, 0, 0)?;
            }
            let list = luaK::jump(lex, state)? as i32;
            // create jump and link it to the label
            luaK::patch_list(lex, state, list, pc as i32)?;
        }
        _ => {
            // no label?
            // forward jump; will be resolved when the label is declared
            let pc = luaK::jump(lex, state)? as i32;
            lex.dyd.gt.push(lex.new_label_entry(name, line, pc));
        }
    }
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
