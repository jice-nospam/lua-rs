//! Lexical Analyzer

use crate::{
    api::LuaError,
    limits::Instruction,
    object::{chunk_id, LocVar, Proto},
    parser::FuncState,
    state::LuaState,
    zio::Zio,
    LuaNumber,
};

const FIRST_RESERVED: isize = 257;
/// maximum char value as \ddd in lua strings
const CHAR_MAX: u32 = 255;

// must match TOKEN_NAMES
#[derive(Clone, Copy)]
pub enum Reserved {
    // terminal symbols denoted by reserved words
    And = FIRST_RESERVED,
    Break,
    Do,
    Else,
    ElseIf,
    End,
    False,
    For,
    Function,
    Goto,
    If,
    In,
    Local,
    Nil,
    Not,
    Or,
    Repeat,
    Return,
    Then,
    True,
    Until,
    While,
    // other terminal symbols
    Concat,  // '..'
    Dots,    // '...'
    Eq,      // '=='
    Ge,      // '>='
    Le,      // '<='
    Ne,      // '~='
    DbColon, // '::'
    Eos,
    Number,
    Name,
    String,
}

impl TryFrom<u32> for Reserved {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            x if Reserved::And as u32 == x => Ok(Reserved::And),
            x if Reserved::Break as u32 == x => Ok(Reserved::Break),
            x if Reserved::Do as u32 == x => Ok(Reserved::Do),
            x if Reserved::Else as u32 == x => Ok(Reserved::Else),
            x if Reserved::ElseIf as u32 == x => Ok(Reserved::ElseIf),
            x if Reserved::End as u32 == x => Ok(Reserved::End),
            x if Reserved::False as u32 == x => Ok(Reserved::False),
            x if Reserved::For as u32 == x => Ok(Reserved::For),
            x if Reserved::Function as u32 == x => Ok(Reserved::Function),
            x if Reserved::Goto as u32 == x => Ok(Reserved::Goto),
            x if Reserved::If as u32 == x => Ok(Reserved::If),
            x if Reserved::In as u32 == x => Ok(Reserved::In),
            x if Reserved::Local as u32 == x => Ok(Reserved::Local),
            x if Reserved::Nil as u32 == x => Ok(Reserved::Nil),
            x if Reserved::Not as u32 == x => Ok(Reserved::Not),
            x if Reserved::Or as u32 == x => Ok(Reserved::Or),
            x if Reserved::Repeat as u32 == x => Ok(Reserved::Repeat),
            x if Reserved::Return as u32 == x => Ok(Reserved::Return),
            x if Reserved::Then as u32 == x => Ok(Reserved::Then),
            x if Reserved::True as u32 == x => Ok(Reserved::True),
            x if Reserved::Until as u32 == x => Ok(Reserved::Until),
            x if Reserved::While as u32 == x => Ok(Reserved::While),
            x if Reserved::Concat as u32 == x => Ok(Reserved::Concat),
            x if Reserved::Dots as u32 == x => Ok(Reserved::Dots),
            x if Reserved::Eq as u32 == x => Ok(Reserved::Eq),
            x if Reserved::Ge as u32 == x => Ok(Reserved::Ge),
            x if Reserved::Le as u32 == x => Ok(Reserved::Le),
            x if Reserved::Ne as u32 == x => Ok(Reserved::Ne),
            x if Reserved::DbColon as u32 == x => Ok(Reserved::DbColon),
            x if Reserved::Eos as u32 == x => Ok(Reserved::Eos),
            x if Reserved::Number as u32 == x => Ok(Reserved::Number),
            x if Reserved::Name as u32 == x => Ok(Reserved::Name),
            x if Reserved::String as u32 == x => Ok(Reserved::String),

            _ => Err(()),
        }
    }
}

const TOKEN_NAMES: [&str; 33] = [
    "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "goto", "if", "in",
    "local", "nil", "not", "or", "repeat", "return", "then", "true", "until", "while", "..", "...",
    "==", ">=", "<=", "~=", "::", "<eof>", "<number>", "<name>", "<string>",
];

const NUM_RESERVED: isize = Reserved::While as isize - FIRST_RESERVED + 1;

#[derive(Clone)]
pub enum SemInfo {
    Number(LuaNumber),
    String(String),
}

#[derive(Clone)]
pub struct Token {
    pub token: u32,
    pub seminfo: SemInfo,
}

impl From<Reserved> for Token {
    fn from(value: Reserved) -> Self {
        Token::new(value as u32)
    }
}

impl Default for Token {
    fn default() -> Self {
        Self {
            token: Reserved::Eos as u32,
            seminfo: SemInfo::Number(0.0),
        }
    }
}

impl Token {
    pub fn new<T: Into<u32>>(c: T) -> Self {
        Self {
            token: c.into(),
            seminfo: SemInfo::Number(0.0),
        }
    }
    pub fn new_string(value: &str) -> Self {
        Self {
            token: Reserved::String as u32,
            seminfo: SemInfo::String(value.to_owned()),
        }
    }
    pub fn new_name(value: &str) -> Self {
        Self {
            token: Reserved::Name as u32,
            seminfo: SemInfo::String(value.to_owned()),
        }
    }
    pub fn new_number(value: LuaNumber) -> Self {
        Self {
            token: Reserved::Number as u32,
            seminfo: SemInfo::Number(value),
        }
    }
}

/// description of pending goto statements and label statements
pub struct LabelDesc {
    /// label identifier
    pub name: String,
    /// position in code
    pub pc: usize,
    /// line where it appeared
    pub line: usize,
    /// local level where it appears in current block
    pub nactvar: usize,
}
impl LabelDesc {
    pub(crate) fn new(name: &str, line: usize, pc: usize, nactvar: usize) -> Self {
        Self {
            name: name.to_owned(),
            pc,
            line,
            nactvar,
        }
    }
}

#[derive(Default)]
/// dynamic structures used by the parser
pub struct DynData {
    /// list of active local variables
    pub actvar: Vec<usize>,
    /// list of pending gotos
    pub gt: Vec<LabelDesc>,
    /// list of active labels
    pub label: Vec<LabelDesc>,
}

pub struct LexState<T> {
    /// current character
    current: Option<char>,
    ///  input line counter
    pub linenumber: usize,
    /// line of last token `consumed'
    pub lastline: usize,
    /// current token
    pub t: Option<Token>,
    /// look ahead token
    lookahead: Option<Token>,
    /// input stream
    z: Zio<T>,
    /// buffer for tokens
    buff: Vec<char>,
    /// dynamic structures used by the parser
    pub dyd: DynData,
    /// current source name
    pub source: String,
    // environment variable name
    pub envn: String,
    /// locale decimal point
    pub decpoint: String,
    /// func states
    pub vfs: Vec<FuncState>,
}

impl<T> LexState<T> {
    pub fn new(z: Zio<T>, source: &str) -> Self {
        Self {
            current: None,
            linenumber: 1,
            lastline: 1,
            t: None,
            lookahead: None,
            z,
            buff: Vec::new(),
            dyd: DynData::default(),
            source: source.to_owned(),
            envn: "_ENV".to_owned(),
            decpoint: ".".to_owned(),
            vfs: vec![FuncState::new()],
        }
    }
    pub fn borrow_mut_fs(&mut self, idx: Option<usize>) -> &mut FuncState {
        match idx {
            Some(idx) => &mut self.vfs[idx],
            None => self.vfs.last_mut().unwrap(),
        }
    }
    pub fn borrow_fs(&self, idx: Option<usize>) -> &FuncState {
        match idx {
            Some(idx) => &self.vfs[idx],
            None => self.vfs.last().unwrap(),
        }
    }
    pub fn next_pc(&self, state: &mut LuaState) -> usize {
        self.borrow_proto(state, None).next_pc() as usize
    }
    pub(crate) fn borrow_mut_proto<'a>(
        &self,
        state: &'a mut LuaState,
        fsid: Option<usize>,
    ) -> &'a mut Proto {
        let id = self.borrow_fs(fsid).f;
        &mut state.protos[id]
    }
    pub(crate) fn borrow_proto<'a>(
        &self,
        state: &'a mut LuaState,
        fsid: Option<usize>,
    ) -> &'a Proto {
        let id = self.borrow_fs(fsid).f;
        &state.protos[id]
    }
    pub fn borrow_mut_code<'a>(
        &mut self,
        state: &'a mut LuaState,
        pc: usize,
    ) -> &'a mut Instruction {
        let protoid = self.borrow_fs(None).f;
        state.borrow_mut_instruction(protoid, pc)
    }
    pub fn get_code(&self, state: &LuaState, pc: usize) -> Instruction {
        let protoid = self.borrow_fs(None).f;
        state.get_instruction(protoid, pc)
    }
    pub(crate) fn search_var(
        &self,
        state: &mut LuaState,
        fs_id: Option<usize>,
        name: &str,
    ) -> Option<usize> {
        let nactvar = self.borrow_fs(fs_id).nactvar;
        if nactvar > 0 {
            (0..nactvar)
                .rev()
                .find(|&i| name == self.borrow_loc_var(state, fs_id, i).name)
        } else {
            None
        }
    }

    pub(crate) fn borrow_loc_var<'a>(
        &self,
        state: &'a mut LuaState,
        fs_id: Option<usize>,
        i: usize,
    ) -> &'a LocVar {
        let first_local = self.borrow_fs(fs_id).first_local;
        let idx = self.dyd.actvar[first_local + i];
        let proto = self.borrow_proto(state, fs_id);
        debug_assert!(idx < proto.locvars.len());
        &proto.locvars[idx]
    }
    pub(crate) fn borrow_mut_local_var<'a>(
        &mut self,
        state: &'a mut LuaState,
        id: usize,
    ) -> &'a mut LocVar {
        let first_local = self.borrow_fs(None).first_local;
        let idx = self.dyd.actvar[first_local + id];
        let proto = self.borrow_mut_proto(state, None);
        debug_assert!(idx < proto.locvars.len());
        &mut proto.locvars[idx]
    }

    /// read next character in the stream
    pub fn next_char(&mut self, state: &mut LuaState) {
        self.current = self.z.getc(state);
    }
    pub fn next_token(&mut self, state: &mut LuaState) -> Result<(), LuaError> {
        self.lastline = self.linenumber;
        // take lookahead token if it exists, else read next token
        if self.lookahead.is_none() {
            self.t = self.lex(state)?;
        } else {
            self.t = self.lookahead.take();
        }
        Ok(())
    }

    /// parse next token
    fn lex(&mut self, state: &mut LuaState) -> Result<Option<Token>, LuaError> {
        self.buff.clear();
        loop {
            match self.current {
                None => {
                    return Ok(None);
                }
                Some('\n') | Some('\r') => {
                    self.inc_line_number(state)?;
                    continue;
                }
                Some('-') => {
                    self.next_char(state);
                    match self.current {
                        Some('-') => (),
                        _ => return Ok(Some(Token::new('-'))),
                    }
                    // else is a comment
                    self.next_char(state);
                    if let Some('[') = self.current {
                        // long comment
                        let sep = self.skip_sep(state);
                        self.buff.clear();
                        if sep >= 0 {
                            self.read_long_string(state, sep, true)?;
                            self.buff.clear();
                            continue;
                        }
                    }
                    // short comment. skip to end of line
                    while !self.is_current_newline() && self.current.is_some() {
                        self.next_char(state);
                    }
                    continue;
                }
                Some('[') => {
                    let sep = self.skip_sep(state);
                    if sep >= 0 {
                        // long string
                        let string_value = (self.read_long_string(state, sep, false)?).unwrap();
                        return Ok(Some(Token::new_string(&string_value)));
                    } else if sep == -1 {
                        return Ok(Some(Token::new('[')));
                    } else {
                        // invalid delimiter, for example [==]
                        return self.lex_error(
                            state,
                            "invalid long string delimiter",
                            Some(Reserved::String as u32),
                        );
                    }
                }
                Some('=') => {
                    self.next_char(state);
                    match self.current {
                        Some('=') => {
                            self.next_char(state);
                            return Ok(Some(Reserved::Eq.into()));
                        }
                        _ => {
                            return Ok(Some(Token::new('=')));
                        }
                    }
                }
                Some('<') => {
                    self.next_char(state);
                    match self.current {
                        Some('=') => {
                            self.next_char(state);
                            return Ok(Some(Reserved::Le.into()));
                        }
                        _ => {
                            return Ok(Some(Token::new('<')));
                        }
                    }
                }
                Some('>') => {
                    self.next_char(state);
                    match self.current {
                        Some('=') => {
                            self.next_char(state);
                            return Ok(Some(Reserved::Ge.into()));
                        }
                        _ => {
                            return Ok(Some(Token::new('>')));
                        }
                    }
                }
                Some('~') => {
                    self.next_char(state);
                    match self.current {
                        Some('=') => {
                            self.next_char(state);
                            return Ok(Some(Reserved::Ne.into()));
                        }
                        _ => {
                            return Ok(Some(Token::new('~')));
                        }
                    }
                }
                Some(':') => {
                    self.next_char(state);
                    match self.current {
                        Some(':') => {
                            self.next_char(state);
                            return Ok(Some(Reserved::DbColon.into()));
                        }
                        _ => {
                            return Ok(Some(Token::new(':')));
                        }
                    }
                }
                Some('\"') | Some('\'') => {
                    let string_value = self.read_string(state, self.current.unwrap())?;
                    return Ok(Some(Token::new_string(&string_value)));
                }
                Some('.') => {
                    self.save_and_next(state);
                    if self.check_next(state, ".") {
                        if self.check_next(state, ".") {
                            // ...
                            return Ok(Some(Reserved::Dots.into()));
                        }
                        // ..
                        return Ok(Some(Reserved::Concat.into()));
                    } else if !self.is_current_digit() {
                        return Ok(Some(Token::new('.')));
                    } else {
                        let value = self.read_numeral(state)?;
                        return Ok(Some(Token::new_number(value)));
                    }
                }
                Some(c) => {
                    if self.is_current_space() {
                        self.next_char(state);
                        continue;
                    } else if self.is_current_digit() {
                        let value = self.read_numeral(state)?;
                        return Ok(Some(Token::new_number(value)));
                    } else if self.is_current_alphabetic() || self.is_current('_') {
                        // identifier or reserved word
                        self.save_and_next(state);
                        while self.is_current_alphanumeric() || self.is_current('_') {
                            self.save_and_next(state);
                        }
                        let iden = self.buff.iter().cloned().collect::<String>();
                        for (i, item) in TOKEN_NAMES.iter().enumerate().take(NUM_RESERVED as usize)
                        {
                            if *item == iden {
                                // reserved word
                                return Ok(Some(
                                    Reserved::try_from(i as u32 + FIRST_RESERVED as u32)
                                        .unwrap()
                                        .into(),
                                ));
                            }
                        }
                        return Ok(Some(Token::new_name(&iden)));
                    } else {
                        self.next_char(state);
                        return Ok(Some(Token::new(c)));
                    }
                }
            }
        }
    }

    fn inc_line_number(&mut self, state: &mut LuaState) -> Result<(), LuaError> {
        let old = self.current;
        debug_assert!(self.is_current_newline());
        // skip `\n' or `\r'
        self.next_char(state);
        if self.is_current_newline() && self.current != old {
            // skip `\n\r' or `\r\n'
            self.next_char(state);
        }
        self.linenumber += 1;
        if self.linenumber >= std::usize::MAX - 2 {
            return self.syntax_error(state, "chunk has too many lines");
        }
        Ok(())
    }

    fn is_current_newline(&self) -> bool {
        matches!(self.current, Some('\n') | Some('\r'))
    }
    fn is_current_digit(&self) -> bool {
        matches!(self.current,Some(c) if c.is_ascii_digit())
    }
    fn is_current_xdigit(&self) -> bool {
        matches!(self.current,Some(c) if c.is_ascii_hexdigit())
    }
    fn is_current_alphanumeric(&self) -> bool {
        matches!(self.current,Some(c) if c.is_alphanumeric())
    }
    fn is_current_alphabetic(&self) -> bool {
        matches!(self.current,Some(c) if c.is_alphabetic())
    }

    fn is_current(&self, arg: char) -> bool {
        matches!(self.current ,Some(c) if c == arg)
    }

    fn is_current_space(&self) -> bool {
        matches!(self.current,Some(c) if c.is_whitespace())
    }

    pub fn syntax_error(&self, state: &mut LuaState, msg: &str) -> Result<(), LuaError> {
        let token = self.t.as_ref().map(|t| t.token);
        self.lex_error(state, msg, token)
    }

    pub fn lex_error<D>(
        &self,
        state: &mut LuaState,
        msg: &str,
        t: Option<u32>,
    ) -> Result<D, LuaError> {
        let chunk_id = chunk_id(&self.source);
        state.push_string(&format!("{}:{}: {}", &chunk_id, self.linenumber, msg));
        if let Some(t) = t {
            state.push_string(&format!("{} near '{}'", msg, self.token_2_txt(t)));
        }
        if let Some(panic) = state.g.panic {
            panic(state);
        }
        Err(LuaError::SyntaxError)
    }

    pub fn token_2_txt(&self, t: u32) -> String {
        match t.try_into() {
            Ok(Reserved::Name) | Ok(Reserved::String) | Ok(Reserved::Number) => {
                self.buff.iter().collect::<String>()
            }
            Ok(_) => TOKEN_NAMES[t as usize - FIRST_RESERVED as usize].to_owned(),
            Err(()) => {
                let c = char::from_u32(t).unwrap();
                if c.is_ascii_control() {
                    format!("char({})", t)
                } else {
                    format!("{}", c)
                }
            }
        }
    }

    /// skip a long comment/string separator [===[ or ]===]
    /// return the number of '=' characters in the separator
    fn skip_sep(&mut self, state: &mut LuaState) -> isize {
        let mut count = 0;
        let s = self.current.unwrap();
        debug_assert!(s == '[' || s == ']');
        self.save_and_next(state);
        while let Some('=') = self.current {
            self.save_and_next(state);
            count += 1;
        }
        match self.current {
            Some(x) if x == s => count,
            _ => (-count) - 1,
        }
    }

    fn read_long_string(
        &mut self,
        state: &mut LuaState,
        sep: isize,
        is_comment: bool,
    ) -> Result<Option<String>, LuaError> {
        // skip 2nd `['
        self.save_and_next(state);
        // string starts with a newline?
        if self.is_current_newline() {
            // skip it
            self.inc_line_number(state)?;
        }
        loop {
            match self.current {
                None => {
                    return self.lex_error(
                        state,
                        if is_comment {
                            "unfinished long comment"
                        } else {
                            "unfinished long string"
                        },
                        Some(Reserved::Eos as u32),
                    )
                }
                Some('[') => {
                    if self.skip_sep(state) == sep {
                        // skip 2nd `['
                        self.save_and_next(state);
                        if sep == 0 {
                            return self.lex_error(
                                state,
                                "nesting of [[...]] is deprecated",
                                Some('[' as u32),
                            );
                        }
                    }
                }
                Some(']') => {
                    if self.skip_sep(state) == sep {
                        // skip 2nd `]'
                        self.save_and_next(state);
                        break;
                    }
                }
                Some('\n') | Some('\r') => {
                    self.save('\n');
                    self.inc_line_number(state)?;
                    if is_comment {
                        self.buff.clear();
                    }
                }
                _ => {
                    if is_comment {
                        self.next_char(state);
                    } else {
                        self.save_and_next(state);
                    }
                }
            }
        }

        if is_comment {
            Ok(None)
        } else {
            // return the string without the [==[ ]==] delimiters
            Ok(Some(
                self.buff[2 + sep as usize..self.buff.len() - 2 * (sep as usize + 2)]
                    .iter()
                    .cloned()
                    .collect::<String>(),
            ))
        }
    }

    fn save_and_next(&mut self, state: &mut LuaState) {
        self.save(self.current.unwrap());
        self.next_char(state);
    }

    fn save(&mut self, c: char) {
        self.buff.push(c);
    }

    fn read_string(&mut self, state: &mut LuaState, delimiter: char) -> Result<String, LuaError> {
        self.save_and_next(state);
        let mut c: char;
        loop {
            match self.current {
                Some(c) if c == delimiter => {
                    break;
                }
                None => {
                    return self.lex_error(state, "unfinished string", Some(Reserved::Eos as u32));
                }
                Some('\r') | Some('\n') => {
                    return self.lex_error(
                        state,
                        "unfinished string",
                        Some(Reserved::String as u32),
                    );
                }
                Some('\\') => {
                    // do not save the \
                    self.next_char(state);
                    match self.current {
                        Some('a') => c = '\x07', // bell
                        Some('b') => c = '\x08', // backspace
                        Some('f') => c = '\x0C', // form feed
                        Some('n') => c = '\n',
                        Some('r') => c = '\r',
                        Some('t') => c = '\t',
                        Some('v') => c = '\x0B', // vertical tab
                        Some('\r') | Some('\n') => {
                            self.save('\n');
                            self.inc_line_number(state)?;
                            continue;
                        }
                        None => {
                            continue; // will raise an error next loop
                        }
                        Some(c) => {
                            if !c.is_ascii_digit() {
                                // handles \\, \", \', and \?
                                self.save_and_next(state);
                            } else {
                                // character numerical value \ddd
                                let mut i = 1;
                                let mut value = c as u32 - '0' as u32;
                                self.next_char(state);
                                while i < 3 && self.is_current_digit() {
                                    value =
                                        10 * value + (self.current.unwrap() as u32 - '0' as u32);
                                    self.next_char(state);
                                    i += 1;
                                }
                                if value > CHAR_MAX {
                                    return self.lex_error(
                                        state,
                                        "escape sequence too large",
                                        Some(Reserved::String as u32),
                                    );
                                }
                                self.save(char::from_u32(value).unwrap());
                            }
                            continue;
                        }
                    }
                    self.save(c);
                    self.next_char(state);
                    continue;
                }
                _ => {
                    self.save_and_next(state);
                }
            }
        }
        // skip ending delimiter
        self.save_and_next(state);
        // return the string without the ' or " delimiters
        Ok(self.buff[1..self.buff.len() - 1]
            .iter()
            .cloned()
            .collect::<String>())
    }

    /// save and consume current token if it is inside arg
    fn check_next(&mut self, state: &mut LuaState, arg: &str) -> bool {
        if let Some(c) = self.current {
            if arg.contains(c) {
                self.save_and_next(state);
                return true;
            }
        }
        false
    }

    /// returns an error if we did not reach end of stream
    pub fn check_eos(&mut self, state: &mut LuaState) -> Result<(), LuaError> {
        if self.current.is_some() {
            return self.syntax_error(
                state,
                &format!("'{}' expected", self.token_2_txt(Reserved::Eos as u32)),
            );
        }
        Ok(())
    }

    fn read_numeral(&mut self, state: &mut LuaState) -> Result<f64, LuaError> {
        debug_assert!(self.is_current_digit());
        let first = self.current.unwrap();
        self.save_and_next(state);
        let mut expo = "Ee";
        if first == '0' && self.check_next(state, "Xx") {
            // hexadecimal ?
            expo = "Pp";
        }
        loop {
            if self.check_next(state, expo) {
                // exponent part ?
                self.check_next(state, "+-"); // exponent sign
            }
            if self.is_current_xdigit() || self.is_current('.') {
                self.save_and_next(state);
            } else {
                break;
            }
        }
        let svalue = self.buff.iter().cloned().collect::<String>();
        // follow locale for decimal point
        let svalue = svalue.replace('.', &self.decpoint);
        str2d(&svalue).ok_or_else(|| {
            self.lex_error::<()>(state, "malformed number", Some(Reserved::Number as u32))
                .err()
                .unwrap()
        })
    }

    pub(crate) fn is_token(&self, arg: u32) -> bool {
        match &self.t {
            Some(t) => t.token == arg,
            _ => false,
        }
    }

    pub(crate) fn is_lookahead_token(&self, arg: u32) -> bool {
        match &self.lookahead {
            Some(t) => t.token == arg,
            _ => false,
        }
    }

    pub(crate) fn error_limit(
        &self,
        state: &mut LuaState,
        limit: usize,
        what: &str,
    ) -> Result<(), LuaError> {
        let msg = {
            let proto = self.borrow_proto(state, None);
            if proto.linedefined == 0 {
                format!("main function has more than {} {}", limit, what)
            } else {
                format!(
                    "function at line {} has more than {} {}",
                    proto.linedefined, limit, what
                )
            }
        };
        self.lex_error(state, &msg, None)
    }

    pub(crate) fn look_ahead(&mut self, state: &mut LuaState) -> Result<(), LuaError> {
        debug_assert!(self.lookahead.is_none());
        self.lookahead = self.lex(state)?;
        Ok(())
    }

    pub(crate) fn new_label_entry(&self, label: String, line: usize, pc: i32) -> LabelDesc {
        let nactvar = self.borrow_fs(None).nactvar;
        LabelDesc {
            name: label,
            pc: pc as usize,
            line,
            nactvar,
        }
    }

    /// semantic error
    pub(crate) fn semantic_error(
        &mut self,
        state: &mut LuaState,
        msg: &str,
    ) -> Result<(), LuaError> {
        self.t = None; // remove 'near to' from final message
        self.syntax_error(state, msg)
    }

    /// check for repeated labels on the same block
    pub(crate) fn check_repeated(
        &mut self,
        state: &mut LuaState,
        label: &str,
    ) -> Result<(), LuaError> {
        let first_label = self.borrow_fs(None).borrow_block().first_label;
        for i in first_label..self.dyd.label.len() {
            if label == self.dyd.label[i].name {
                let msg = format!(
                    "label '{}' already defined on line {}",
                    label, self.dyd.label[i].line
                );
                return self.semantic_error(state, &msg);
            }
        }
        Ok(())
    }
}

pub(crate) fn str2d(svalue: &str) -> Option<f64> {
    if strpbrk(svalue, "nN") {
        // reject 'inf' and 'nan'
        None
    } else if strpbrk(svalue, "xX") {
        // hexa?
        strx2number(svalue)
    } else {
        svalue.parse::<f64>().ok()
    }
}

/// convert an hexadecimal numeric string to a number, following
/// C99 specification for 'strtod'
fn strx2number(svalue: &str) -> Option<f64> {
    let mut r = 0.0;
    let mut e = 0.0;
    let mut i = 0;
    let mut it = 0;
    let chars: Vec<char> = svalue.chars().collect();
    let len = chars.len();
    let neg = chars[0] == '-';
    if neg || chars[0] == '+' {
        it += 1;
    }
    if !(chars[it] == '0' && (chars[it + 1] == 'x' || chars[it + 1] == 'X')) {
        // invalid format. should start with 0x
        return None;
    }
    it += 2;
    while it < len && chars[it].is_ascii_hexdigit() {
        // read integer part
        r = r * 16.0 + (u8::from_str_radix(&chars[it].to_string(), 16).unwrap() as f64);
        it += 1;
        i += 1;
    }
    if it < len && chars[it] == '.' {
        it += 1; // skip dot
        while it < len && chars[it].is_ascii_hexdigit() {
            // read fractional part
            r = r * 16.0 + (u8::from_str_radix(&chars[it].to_string(), 16).unwrap() as f64);
            it += 1;
            e += 1.0;
        }
    }
    if i == 0 && e == 0.0 {
        // invalid format (no digit)
        return None;
    }
    e *= -4.0; // each fractional digit divides value by 2^-4
    if it < len && (chars[it] == 'p' || chars[it] == 'P') {
        // exponent part?
        let mut exp1 = 0.0;
        it += 1; // skip 'p'
        let neg1 = if it < len { chars[it] == '-' } else { false };
        if neg1 || (it < len && chars[it] == '+') {
            it += 1;
        }
        while it < len && chars[it].is_ascii_digit() {
            // read exponent
            exp1 = exp1 * 10.0 + (chars[it] as u8 - '0' as u8) as f64;
            it += 1;
        }
        if neg1 {
            exp1 = -exp1;
        }
        e += exp1;
    }
    if neg {
        r = -r;
    }
    Some(r + e)
}

fn strpbrk(haystack: &str, needle: &str) -> bool {
    for c in haystack.chars() {
        if needle.contains(c) {
            return true;
        }
    }
    false
}
