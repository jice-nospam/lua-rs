//! Lua Parser

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    api::LuaError,
    ldo::SParser,
    lex::{LexState, Reserved, SemInfo},
    luaK::{self, code_abc, exp2nextreg},
    luaconf::LUAI_MAXVARS,
    object::{LocVar, Proto, TValue},
    opcodes::{set_arg_c, OpCode, NO_JUMP, NO_REG},
    state::{LuaStateRef},
    LuaNumber, LUA_MULTRET,
};

#[derive(Clone,Copy)]
pub(crate) enum BinaryOp {
    Add=0,
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
    left:usize,right:usize
}
const BINARY_OP_PRIO: [BinaryPriority;15] = [
    BinaryPriority{left:6,right:6}, // Add
    BinaryPriority{left:6,right:6}, // Sub
    BinaryPriority{left:7,right:7}, // Mul
    BinaryPriority{left:7,right:7}, // Div
    BinaryPriority{left:7,right:7}, // Mod
    BinaryPriority{left:10,right:9}, // Pow
    BinaryPriority{left:5,right:4}, // Concat
    BinaryPriority{left:3,right:3}, // Ne
    BinaryPriority{left:3,right:3}, // Eq
    BinaryPriority{left:3,right:3}, // Lt
    BinaryPriority{left:3,right:3}, // Le
    BinaryPriority{left:3,right:3}, // Gt
    BinaryPriority{left:3,right:3}, // Ge
    BinaryPriority{left:2,right:2}, // And
    BinaryPriority{left:1,right:1}, // Or
];

pub(crate) enum UnaryOp {
    Minus,
    Not,
    Len,
}

/// priority for unary operators
const UNARY_PRIORITY: usize = 8;

#[derive(Default)]
struct UpValDesc {
    k: ExpressionKind,
    info: u32,
    name: String,
}

/// nodes for block list (list of active blocks)
struct BlockCnt {
    /// list of jumps out of this loop
    breaklist: usize,
    /// # active locals outside the breakable structure
    nactvar: usize,
    /// true if some variable in the block is an upvalue
    upval: bool,
    /// true if `block' is a loop
    isbreakable: bool,
}

/// state needed to generate code for a given function
pub struct FuncState<T> {
    /// current function header
    pub f: Proto,
    /// table to find (and reuse) constants in `f.k'
    pub h: HashMap<TValue, usize>,
    /// enclosing function
    prev: Option<Box<FuncState<T>>>,
    /// lexical state
    ls: Rc<RefCell<LexState<T>>>,
    /// copy of the Lua state
    state: LuaStateRef,
    /// chain of current blocks
    bl: Vec<BlockCnt>,
    /// next position to code
    pc: usize,
    /// `pc' of last `jump target'
    lasttarget: i32,
    /// list of pending jumps to `pc'
    pub jpc: i32,
    /// first free register
    pub freereg: usize,
    /// number of elements in `k'
    pub nk: usize,
    /// number of elements in `p'
    np: usize,
    /// number of elements in `locvars'
    nlocvars: usize,
    /// number of active local variables
    pub nactvar: usize,
    /// upvalues
    upvalues: Vec<UpValDesc>,
    /// declared-variable stack
    actvar: [usize; LUAI_MAXVARS],
}

impl<T> FuncState<T> {
    fn new(state: LuaStateRef, lex_state: Rc<RefCell<LexState<T>>>) -> Self {
        let mut proto = Proto::new();
        // registers 0/1 are always valid
        proto.maxstacksize = 2;
        // TODO proto.source = lex_state.source
        Self {
            f: proto,
            h: HashMap::new(),
            prev: None,
            ls: Rc::clone(&lex_state),
            state: Rc::clone(&state),
            bl: Vec::new(),
            pc: 0,
            lasttarget: NO_JUMP,
            jpc: NO_JUMP,
            freereg: 0,
            nk: 0,
            np: 0,
            nlocvars: 0,
            nactvar: 0,
            upvalues: Vec::new(),
            actvar: [0; LUAI_MAXVARS],
        }
    }

    fn search_var(&self, name: &str) -> Option<usize> {
        for i in (0..self.nactvar).rev() {
            if name == self.get_loc_var(i).name {
                return Some(i);
            }
        }
        None
    }

    fn mark_upval(&self, _v: usize) {
        todo!()
    }

    fn get_loc_var(&self, i: usize) -> &LocVar {
        &self.f.locvars[self.actvar[i]]
    }

    fn add_constant(&mut self, key: TValue, value: TValue) -> usize {
        match self.h.get(&key) {
            Some(i) => return *i,
            None => {
                self.h.insert(key, self.nk);
                self.f.k.push(value);
                self.nk += 1;
                return self.nk - 1;
            }
        }
    }

    pub fn string_constant(&mut self, value: &str) -> usize {
        let tvalue = TValue::new_string(value);
        self.add_constant(tvalue.clone(), tvalue)
    }

    pub fn number_constant(&mut self, value: LuaNumber) -> usize {
        let tvalue = TValue::Number(value);
        self.add_constant(tvalue.clone(), tvalue)
    }
}

pub fn parser<T>(state: LuaStateRef, parser: &mut SParser<T>) -> Result<Proto, LuaError> {
    let lex_state = Rc::new(RefCell::new(LexState::new(
        Rc::clone(&state),
        parser.z.take().unwrap(),
        &parser.name,
    )));
    // read the first character in the stream
    lex_state.borrow_mut().next_char();
    let mut fs = FuncState::new(Rc::clone(&state), Rc::clone(&lex_state));
    // main func. is always vararg
    fs.f.is_vararg = true;
    {
        let mut lstate = lex_state.borrow_mut();
        lstate.next_token()?;
        chunk(&mut lstate, &mut fs)?;
        lstate.check_eos()?;
    }
    close_func(&mut fs, lex_state);
    Ok(fs.f)
}

fn close_func<T>(fs: &mut FuncState<T>, lex_state: Rc<RefCell<LexState<T>>>) {
    remove_vars(fs,0);
    luaK::ret(&mut *lex_state.borrow_mut(), fs,0,0); // final return
    fs.f.sizecode = fs.pc;
    fs.f.sizelineinfo = fs.pc;
    fs.f.sizek = fs.nk;
    fs.f.sizep = fs.np;
    fs.f.sizelocvars = fs.nlocvars;
    fs.f.sizeupvalues = fs.upvalues.len();
}

fn remove_vars<T>(fs: &mut FuncState<T>, to_level: usize) {
    while fs.nactvar > to_level {
        fs.nactvar -= 1;
        borrow_mut_locvar(fs, fs.nactvar).end_pc = fs.pc;
    }
}

fn borrow_mut_locvar<T>(fs: &mut FuncState<T>, nactvar: usize) -> &mut LocVar {
    &mut fs.f.locvars[fs.actvar[nactvar]]
}

fn block_follow(t: Option<u32>) -> bool {
    if t.is_none() {
        return true;
    }
    match Reserved::try_from(t.unwrap()) {
        Ok(Reserved::ELSE) | Ok(Reserved::ELSEIF) | Ok(Reserved::END) | Ok(Reserved::UNTIL) => true,
        _ => false,
    }
}

/// parse next chunk
fn chunk<T>(lex: &mut LexState<T>, fs: &mut FuncState<T>) -> Result<(), LuaError> {
    let mut is_last = false;
    enter_level(lex)?;
    while !is_last && !block_follow(lex.t.as_ref().map(|t| t.token)) {
        is_last = statement(lex, fs)?;
        test_next(lex, ';' as u32)?;
        debug_assert!(fs.f.maxstacksize >= fs.freereg && fs.freereg >= fs.nactvar);
        // free registers
        fs.freereg = fs.nactvar;
    }
    leave_level(lex);
    Ok(())
}

fn leave_level<T>(lex: &mut LexState<T>) {
    lex.state.borrow_mut().n_rcalls -= 1;
}

fn test_next<T>(lex: &mut LexState<T>, arg: u32) -> Result<bool, LuaError> {
    match &lex.t {
        Some(t) if t.token == arg => {
            lex.next_token()?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn enter_level<T>(lex: &mut LexState<T>) -> Result<(), LuaError> {
    let mut state = lex.state.borrow_mut();
    state.n_rcalls += 1;
    if state.n_rcalls >= crate::luaconf::LUAI_MAXRCALLS {
        return lex.lex_error("chunk has too many syntax levels", None);
    }
    Ok(())
}

fn statement<T>(lex: &mut LexState<T>, fs: &mut FuncState<T>) -> Result<bool, LuaError> {
    let line = lex.linenumber;
    if let Some(ref t) = lex.t {
        match Reserved::try_from(t.token) {
            Ok(Reserved::IF) => {
                if_stat(lex, line)?;
                return Ok(false);
            }
            Ok(Reserved::WHILE) => {
                while_stat(lex, line)?;
                return Ok(false);
            }
            Ok(Reserved::DO) => {
                lex.next_token()?;
                block(lex)?;
                check_match(lex, Reserved::END as u32, Reserved::DO as u32, line)?;
                return Ok(false);
            }
            Ok(Reserved::FOR) => {
                for_stat(lex, line)?;
                return Ok(false);
            }
            Ok(Reserved::REPEAT) => {
                repeat_stat(lex, line)?;
                return Ok(false);
            }
            Ok(Reserved::FUNCTION) => {
                func_stat(lex, line)?;
                return Ok(false);
            }
            Ok(Reserved::LOCAL) => {
                lex.next_token()?;
                if test_next(lex, Reserved::FUNCTION as u32)? {
                    local_func(lex)?;
                } else {
                    local_stat(lex)?;
                }
                return Ok(false);
            }
            Ok(Reserved::RETURN) => {
                return_stat(lex, line)?;
                return Ok(true);
            }
            Ok(Reserved::BREAK) => {
                lex.next_token()?;
                break_stat(lex, line)?;
                return Ok(true);
            }
            _ => {
                expr_stat(lex, fs)?;
                return Ok(false);
            }
        }
    }
    expr_stat(lex, fs)?;
    return Ok(false);
}

#[derive(Default, PartialEq, Clone, Copy)]
pub enum ExpressionKind {
    /// no value
    #[default]
    VVOID = 0,
    VNIL,
    VTRUE,
    VFALSE,
    /// info = index of constant in `k'
    VK,
    /// nval = numerical value
    VKNUM,
    /// info = local register
    VLOCAL,
    /// info = index of upvalue in `upvalues'
    VUPVAL,
    /// info = index of table; aux = index of global name in `k'
    VGLOBAL,
    /// info = table register; aux = index register (or `k')
    VINDEXED,
    /// info = instruction pc
    VJMP,
    /// info = instruction pc
    VRELOCABLE,
    /// info = result register
    VNONRELOC,
    /// info = instruction pc
    VCALL,
    /// info = instruction pc
    VVARARG,
}

#[derive(Default)]
pub struct ExpressionDesc {
    pub k: ExpressionKind,
    pub info: u32,
    pub aux: u32,
    pub nval: LuaNumber,
    /// patch list of `exit when true'
    pub t: i32,
    /// patch list of `exit when false'
    pub f: i32,
}
impl ExpressionDesc {
    fn init(&mut self, kind: ExpressionKind, info: u32) {
        self.k = kind;
        self.info = info;
        self.t = NO_JUMP;
        self.f = NO_JUMP;
    }

    fn index_upvalue<T>(&mut self, fs: &mut FuncState<T>, name: &str) -> usize {
        for (i, uv) in fs.upvalues.iter().enumerate() {
            if uv.k == self.k && uv.info == self.info {
                return i;
            }
        }
        fs.upvalues.push(UpValDesc {
            k: self.k,
            info: self.info,
            name: name.to_owned(),
        });
        fs.f.nups += 1;
        debug_assert!(fs.f.nups == fs.upvalues.len());
        return fs.f.nups - 1;
    }
}

#[derive(Default)]
struct LHSAssignment {
    prev: Option<Box<LHSAssignment>>,
    /// variable (global, local, upvalue, or indexed)
    v: ExpressionDesc,
}

/// stat -> func | assignment
fn expr_stat<T>(lex: &mut LexState<T>, fs: &mut FuncState<T>) -> Result<(), LuaError> {
    let mut lhs = LHSAssignment::default();
    primary_expr(lex, fs, &mut lhs.v)?;
    if lhs.v.k == ExpressionKind::VCALL {
        // statement = func
        // call statement uses no results
        set_arg_c(&mut fs.f.code[lhs.v.info as usize], 1);
    } else {
        // statement = assignment
        assignment(lex, fs, lhs, 1)?;
    }
    Ok(())
}

fn assignment<T>(
    lex: &mut LexState<T>,
    fs: &mut FuncState<T>,
    lhs: LHSAssignment,
    nvars: usize,
) -> Result<(), LuaError> {
    let mut exp = ExpressionDesc::default();
    if (lhs.v.k as u32) < (ExpressionKind::VLOCAL as u32)
        || (lhs.v.k as u32) > (ExpressionKind::VINDEXED as u32)
    {
        return lex.syntax_error("syntax error");
    }
    if test_next(lex, ',' as u32)? {
        // assignment -> `,' primaryexp assignment
        let mut nv = LHSAssignment::default();
        nv.prev = Some(Box::new(lhs));
        primary_expr(lex, fs, &mut nv.v)?;
        if let ExpressionKind::VLOCAL = nv.v.k {
            check_conflict(lex, fs, &exp, &mut nv)?;
        }
        assignment(lex, fs, nv, nvars + 1)?;
    } else {
        // assignment -> `=' explist1
        check_next(lex, '=' as u32)?;
        let _nexps = exp_list1(lex, fs, &mut exp)?;
        todo!();
    }
    exp.init(ExpressionKind::VNONRELOC, fs.freereg as u32 - 1);
    Ok(())
}

fn check_next<T>(lex: &mut LexState<T>, token: u32) -> Result<(), LuaError> {
    check(lex, token)?;
    lex.next_token()
}

fn check_conflict<T>(
    _lex: &mut LexState<T>,
    _fs: &mut FuncState<T>,
    _e: &ExpressionDesc,
    lhs: &mut LHSAssignment,
) -> Result<(), LuaError> {
    if let Some(ref mut _nv) = lhs.prev {}
    todo!()
}

/// primaryexp -> prefixexp { `.' NAME | `[' exp `]' | `:' NAME funcargs | funcargs }
fn primary_expr<T>(
    lex: &mut LexState<T>,
    fs: &mut FuncState<T>,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    prefix_expr(lex, fs, exp)?;
    if lex.t.is_none() {
        return Ok(());
    }
    while let Some(t) = lex.t.clone() {
        match t.token {
            c if c == u32::from('.') => {
                // field
                field(lex, exp)?;
            }
            c if c == u32::from('[') => {
                // `[' exp1 `]'
                todo!()
            }
            c if c == u32::from(':') => {
                // `:' NAME funcargs
                todo!()
            }
            c if c == u32::from('(') || c == u32::from('{') || c == Reserved::STRING as u32 => {
                luaK::exp2nextreg(lex, fs, exp)?;
                func_args(lex, fs, exp)?;
            }
            _ => return Ok(()),
        }
    }
    Ok(())
}

fn func_args<T>(
    lex: &mut LexState<T>,
    fs: &mut FuncState<T>,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    let line = lex.linenumber;
    let mut args = ExpressionDesc::default();
    match &lex.t {
        Some(t) if t.token == '(' as u32 => {
            // funcargs -> `(' [ explist1 ] `)'
            if line != lex.lastline {
                return lex.syntax_error("ambiguous syntax (function call x new statement)");
            }
            lex.next_token()?;
            if lex.is_token(')' as u32) {
                // arg list is empty
                args.k = ExpressionKind::VVOID;
            } else {
                exp_list1(lex, fs, &mut args)?;
                luaK::set_mult_ret(lex, fs, &mut args)?;
            }
            check_match(lex, ')' as u32, '(' as u32, line)?;
        }
        Some(t) if t.token == '{' as u32 => {
            // funcargs -> constructor
            constructor(lex, fs, &mut args)?;
        }
        Some(t) if t.token == Reserved::STRING as u32 => {
            // funcargs -> STRING
            code_string(fs, &mut args, &t.seminfo);
            lex.next_token()?;
        }
        _ => {
            return lex.syntax_error("function arguments expected");
        }
    }
    debug_assert!(exp.k == ExpressionKind::VNONRELOC);
    let base = exp.info; // base register for call
    let nparams;
    if has_mult_ret(args.k) {
        nparams = LUA_MULTRET as u32; // open call
    } else {
        if args.k != ExpressionKind::VVOID {
            exp2nextreg(lex, fs, &mut args)?;
        }
        nparams = fs.freereg as u32 - (base + 1);
    }
    exp.init(
        ExpressionKind::VCALL,
        code_abc(lex, fs, OpCode::Call as u32, base, nparams + 1, 2),
    );
    luaK::fix_line(fs, line);
    fs.freereg = base as usize + 1; // call remove function and arguments and leaves
                                    // (unless changed) one result
    Ok(())
}

#[inline]
fn has_mult_ret(k: ExpressionKind) -> bool {
    k == ExpressionKind::VCALL || k == ExpressionKind::VVARARG
}

/// set expression as a astring constant
fn code_string<T>(fs: &mut FuncState<T>, exp: &mut ExpressionDesc, seminfo: &SemInfo) {
    if let SemInfo::String(s) = seminfo {
        exp.init(ExpressionKind::VK, fs.string_constant(s) as u32);
    } else {
        unreachable!()
    }
}

fn constructor<T>(
    _lex: &mut LexState<T>,
    _fs: &mut FuncState<T>,
    _exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    todo!()
}

/// explist1 -> expr { `,' expr }
fn exp_list1<T>(
    lex: &mut LexState<T>,
    fs: &mut FuncState<T>,
    exp: &mut ExpressionDesc,
) -> Result<usize, LuaError> {
    let mut n = 1;
    expr(lex, fs, exp)?;
    while test_next(lex, ',' as u32)? {
        exp2nextreg(lex, fs, exp)?;
        expr(lex, fs, exp)?;
        n += 1;
    }
    Ok(n)
}

fn field<T>(_lex: &mut LexState<T>, _exp: &mut ExpressionDesc) -> Result<(), LuaError> {
    todo!()
}

/// prefixexp -> NAME | '(' expr ')'
fn prefix_expr<T>(
    lex: &mut LexState<T>,
    fs: &mut FuncState<T>,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    match &lex.t {
        Some(t) if t.token == u32::from('(') => {
            let line = lex.linenumber;
            lex.next_token()?;
            expr(lex, fs, exp)?;
            check_match(lex, u32::from(')'), u32::from('('), line)?;
            luaK::discharge_vars(lex, fs, exp)?;
            return Ok(());
        }
        Some(t) if t.token == Reserved::NAME as u32 => {
            single_var(lex, fs, exp)?;
            return Ok(());
        }
        _ => {
            return lex.syntax_error("unexpected symbol");
        }
    }
}

fn single_var<T>(
    lex: &mut LexState<T>,
    fs: &mut FuncState<T>,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    let name = str_checkname(lex)?;
    if single_var_aux(fs, &name, exp, true)? == ExpressionKind::VGLOBAL {
        // info points to global name
        exp.info = fs.string_constant(&name) as u32;
    }
    Ok(())
}

fn single_var_aux<T>(
    fs: &mut FuncState<T>,
    name: &str,
    exp: &mut ExpressionDesc,
    base: bool,
) -> Result<ExpressionKind, LuaError> {
    // look up at current level
    if let Some(v) = fs.search_var(name) {
        exp.init(ExpressionKind::VLOCAL, v as u32);
        if !base {
            // local will be used as an upval
            fs.mark_upval(v);
        }
        return Ok(ExpressionKind::VLOCAL);
    } else {
        // not found at current level; try upper one
        match fs.prev {
            None => {
                // no more levels. var is global
                exp.init(ExpressionKind::VGLOBAL, NO_REG);
                return Ok(ExpressionKind::VGLOBAL);
            }
            Some(ref mut fs) => {
                if let Ok(ExpressionKind::VGLOBAL) = single_var_aux(fs, name, exp, base) {
                    return Ok(ExpressionKind::VGLOBAL);
                }
                exp.index_upvalue(fs, name);
                return Ok(ExpressionKind::VUPVAL);
            }
        }
    }
}

fn str_checkname<T>(lex: &mut LexState<T>) -> Result<String, LuaError> {
    check(lex, Reserved::NAME as u32)?;
    let name = if let Some(ref t) = lex.t {
        if let SemInfo::String(s) = &t.seminfo {
            s.to_owned()
        } else {
            unreachable!()
        }
    } else {
        unreachable!()
    };
    lex.next_token()?;
    Ok(name)
}

fn check<T>(lex: &mut LexState<T>, token: u32) -> Result<(), LuaError> {
    match &lex.t {
        Some(t) if t.token == token => Ok(()),
        _ => lex.syntax_error(&format!("'{}' expected", lex.token_2_txt(token))),
    }
}

fn expr<T>(lex: &mut LexState<T>, fs: &mut FuncState<T>,exp: &mut ExpressionDesc) -> Result<(), LuaError> {
    subexpr(lex, fs, exp, 0)?;
    Ok(())
}

/// subexpr -> (simpleexp | unop subexpr) { binop subexpr }
/// where `binop' is any binary operator with a priority higher than `limit'
fn subexpr<T>(
    lex: &mut LexState<T>,
    fs: &mut FuncState<T>,
    exp: &mut ExpressionDesc,
    limit: usize,
) -> Result<Option<BinaryOp>, LuaError> {
    enter_level(lex)?;
    if let Some(uop) = unary_op(&lex.t) {
        lex.next_token()?;
        subexpr(lex,fs, exp,UNARY_PRIORITY)?;
        luaK::prefix(fs, uop, exp);
    } else {
        simple_exp(lex,fs, exp)?;
    }
    // expand while operators have priorities higher than `limit'
    let mut oop=binary_op(&lex.t);
    while let Some(op) = oop {
        if BINARY_OP_PRIO[op as usize].left <= limit {
            break;
        }
        lex.next_token()?;
        luaK::infix(lex,fs,op,exp);
        let mut exp2=ExpressionDesc::default();
        let nextop = subexpr(lex, fs, &mut exp2, BINARY_OP_PRIO[op as usize].right)?;
        luaK::postfix(lex,fs,exp, &mut exp2);
        oop = nextop;
    }
    leave_level(lex);
    Ok(oop)
}

/// simple_exp -> NUMBER | STRING | NIL | true | false | ... |
/// constructor | FUNCTION body | primaryexp
fn simple_exp<T>(lex: &mut LexState<T>, fs: &mut FuncState<T>,exp: &mut ExpressionDesc) -> Result<(), LuaError> {
    match &lex.t {
        Some(t) if t.token == Reserved::NUMBER as u32 => {
            if let SemInfo::Number(val) = t.seminfo {
                exp.init(ExpressionKind::VKNUM,0);
                exp.nval = val;
            } else {
                unreachable!()
            }
        }
        Some(t) if t.token == Reserved::STRING as u32 => {
            code_string(fs, exp, &t.seminfo);
        }
        Some(t) if t.token == Reserved::NIL as u32 => {
            exp.init(ExpressionKind::VNIL, 0);
        }
        Some(t) if t.token == Reserved::TRUE as u32 => {
            exp.init(ExpressionKind::VTRUE, 0);
        }
        Some(t) if t.token == Reserved::FALSE as u32 => {
            exp.init(ExpressionKind::VFALSE, 0);
        }
        Some(t) if t.token == Reserved::DOTS as u32 => {
            // vararg
            if !fs.f.is_vararg {
                return lex.syntax_error("cannot use '...' outside a vararg function");
            }
            exp.init(ExpressionKind::VVARARG, code_abc(lex, fs, OpCode::VarArg as u32, 0, 1, 0));
        }
        Some(t) if t.token == '{' as u32 => {
            // constructor
            constructor(lex, fs, exp)?;
            return Ok(());
        }
        Some(t) if t.token == Reserved::FUNCTION as u32 => {
            lex.next_token()?;
            body(lex, fs, exp, false, lex.linenumber)?;
            return Ok(());
        }
        _ => {
            primary_expr(lex, fs, exp)?;
            return Ok(());
        }
    }
    lex.next_token()?;
    Ok(())
}

fn body<T>(_lex: &mut LexState<T>, _fs: &mut FuncState<T>,_exp: &mut ExpressionDesc, _need_self: bool, _line: usize) -> Result<(), LuaError>  {
    todo!()
    // TODO when calling close_func, fs should become fs.prev
}

fn unary_op(t: &Option<crate::lex::Token>) -> Option<UnaryOp> {
    match t {
        Some(t) if t.token == Reserved::NOT as u32 => Some(UnaryOp::Not),
        Some(t) if t.token == '-' as u32 => Some(UnaryOp::Minus),
        Some(t) if t.token == '#' as u32 => Some(UnaryOp::Len),
        _ => None
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
        Some(t) if t.token == Reserved::CONCAT as u32 => Some(BinaryOp::Concat),
        Some(t) if t.token == Reserved::NE as u32 => Some(BinaryOp::Ne),
        Some(t) if t.token == Reserved::EQ as u32 => Some(BinaryOp::Eq),
        Some(t) if t.token == '<' as u32 => Some(BinaryOp::Lt),
        Some(t) if t.token == Reserved::LE as u32 => Some(BinaryOp::Le),
        Some(t) if t.token == '>' as u32 => Some(BinaryOp::Gt),
        Some(t) if t.token == Reserved::GE as u32 => Some(BinaryOp::Ge),
        Some(t) if t.token == Reserved::AND as u32 => Some(BinaryOp::And),
        Some(t) if t.token == Reserved::OR as u32 => Some(BinaryOp::Or),
        _ => None
    }
}

fn break_stat<T>(_lex: &mut LexState<T>, _line: usize) -> Result<(), LuaError> {
    todo!()
}

fn return_stat<T>(_lex: &mut LexState<T>, _line: usize) -> Result<(), LuaError> {
    todo!()
}

fn local_stat<T>(_lex: &mut LexState<T>) -> Result<(), LuaError> {
    todo!()
}

fn local_func<T>(_lex: &mut LexState<T>) -> Result<(), LuaError> {
    todo!()
}

fn func_stat<T>(_lex: &mut LexState<T>, _line: usize) -> Result<(), LuaError> {
    todo!()
}

fn repeat_stat<T>(_lex: &mut LexState<T>, _line: usize) -> Result<(), LuaError> {
    todo!()
}

fn for_stat<T>(_lex: &mut LexState<T>, _line: usize) -> Result<(), LuaError> {
    todo!()
}

fn check_match<T>(
    lex: &mut LexState<T>,
    what: u32,
    who: u32,
    line: usize,
) -> Result<(), LuaError> {
    if ! test_next(lex, what)? {
        if lex.linenumber == line {
            return lex.syntax_error(&format!("'{}' expected", lex.token_2_txt(what)));
        }
        let msg=format!("'{}' expected (to close '{}' at {})",lex.token_2_txt(what),lex.token_2_txt(who),line);
        lex.state.borrow_mut().push_string(&msg);
        return lex.syntax_error(&msg);
    }
    Ok(())
}

fn block<T>(_lex: &mut LexState<T>) -> Result<(), LuaError> {
    todo!()
}

fn while_stat<T>(_lex: &mut LexState<T>, _line: usize) -> Result<(), LuaError> {
    todo!()
}

fn if_stat<T>(_lex: &mut LexState<T>, _line: usize) -> Result<(), LuaError> {
    todo!()
}
