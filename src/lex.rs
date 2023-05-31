//! Lexical Analyzer

use crate::{
    api::LuaError,
    limits::Instruction,
    object::{chunk_id, LocVar},
    parser::FuncState,
    state::LuaState,
    zio::Zio,
    LuaNumber,
};

const FIRST_RESERVED: isize = 257;
/// maximum char value as \ddd in lua strings
const CHAR_MAX: u32 = 255;

#[derive(Clone, Copy)]
pub enum Reserved {
    // terminal symbols denoted by reserved words
    AND = FIRST_RESERVED,
    BREAK,
    DO,
    ELSE,
    ELSEIF,
    END,
    FALSE,
    FOR,
    FUNCTION,
    IF,
    IN,
    LOCAL,
    NIL,
    NOT,
    OR,
    REPEAT,
    RETURN,
    THEN,
    TRUE,
    UNTIL,
    WHILE,
    // other terminal symbols
    CONCAT,
    DOTS,
    EQ,
    GE,
    LE,
    NE,
    NUMBER,
    NAME,
    STRING,
    EOS,
}

impl TryFrom<u32> for Reserved {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            x if Reserved::AND as u32 == x => Ok(Reserved::AND),
            x if Reserved::BREAK as u32 == x => Ok(Reserved::BREAK),
            x if Reserved::CONCAT as u32 == x => Ok(Reserved::CONCAT),
            x if Reserved::DO as u32 == x => Ok(Reserved::DO),
            x if Reserved::DOTS as u32 == x => Ok(Reserved::DOTS),
            x if Reserved::ELSE as u32 == x => Ok(Reserved::ELSE),
            x if Reserved::ELSEIF as u32 == x => Ok(Reserved::ELSEIF),
            x if Reserved::END as u32 == x => Ok(Reserved::END),
            x if Reserved::EOS as u32 == x => Ok(Reserved::EOS),
            x if Reserved::EQ as u32 == x => Ok(Reserved::EQ),
            x if Reserved::FALSE as u32 == x => Ok(Reserved::FALSE),
            x if Reserved::FOR as u32 == x => Ok(Reserved::FOR),
            x if Reserved::FUNCTION as u32 == x => Ok(Reserved::FUNCTION),
            x if Reserved::GE as u32 == x => Ok(Reserved::GE),
            x if Reserved::IF as u32 == x => Ok(Reserved::IF),
            x if Reserved::IN as u32 == x => Ok(Reserved::IN),
            x if Reserved::LE as u32 == x => Ok(Reserved::LE),
            x if Reserved::LOCAL as u32 == x => Ok(Reserved::LOCAL),
            x if Reserved::NAME as u32 == x => Ok(Reserved::NAME),
            x if Reserved::NE as u32 == x => Ok(Reserved::NE),
            x if Reserved::NIL as u32 == x => Ok(Reserved::NIL),
            x if Reserved::NOT as u32 == x => Ok(Reserved::NOT),
            x if Reserved::NUMBER as u32 == x => Ok(Reserved::NUMBER),
            x if Reserved::OR as u32 == x => Ok(Reserved::OR),
            x if Reserved::REPEAT as u32 == x => Ok(Reserved::REPEAT),
            x if Reserved::RETURN as u32 == x => Ok(Reserved::RETURN),
            x if Reserved::STRING as u32 == x => Ok(Reserved::STRING),
            x if Reserved::THEN as u32 == x => Ok(Reserved::THEN),
            x if Reserved::TRUE as u32 == x => Ok(Reserved::TRUE),
            x if Reserved::UNTIL as u32 == x => Ok(Reserved::UNTIL),
            x if Reserved::WHILE as u32 == x => Ok(Reserved::WHILE),
            _ => Err(()),
        }
    }
}

const TOKEN_NAMES: [&str; 31] = [
    "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "if", "in", "local",
    "nil", "not", "or", "repeat", "return", "then", "true", "until", "while", "..", "...", "==",
    ">=", "<=", "~=", "<number>", "<name>", "<string>", "<eof>",
];

const NUM_RESERVED: isize = Reserved::WHILE as isize - FIRST_RESERVED + 1;

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
            token: Reserved::EOS as u32,
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
            token: Reserved::STRING as u32,
            seminfo: SemInfo::String(value.to_owned()),
        }
    }
    pub fn new_name(value: &str) -> Self {
        Self {
            token: Reserved::NAME as u32,
            seminfo: SemInfo::String(value.to_owned()),
        }
    }
    pub fn new_number(value: LuaNumber) -> Self {
        Self {
            token: Reserved::NUMBER as u32,
            seminfo: SemInfo::Number(value),
        }
    }
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
    //struct FuncState *fs;  /* `FuncState' is private to the parser */
    //pub state: &mut LuaState,
    /// input stream
    z: Zio<T>,
    /// buffer for tokens
    buff: Vec<char>,
    /// current source name
    pub source: String,
    /// locale decimal point
    pub decpoint: String,
    /// func states
    pub vfs: Vec<FuncState>,
    /// current func state
    pub fs: usize,
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
            source: source.to_owned(),
            decpoint: ".".to_owned(),
            vfs: vec![FuncState::new(source)],
            fs: 0,
        }
    }
    pub fn borrow_mut_fs(&mut self, idx: Option<usize>) -> &mut FuncState {
        &mut self.vfs[idx.unwrap_or(self.fs)]
    }
    pub fn borrow_fs(&self, idx: Option<usize>) -> &FuncState {
        &self.vfs[idx.unwrap_or(self.fs)]
    }
    pub fn borrow_mut_code(&mut self, pc: usize) -> &mut Instruction {
        &mut self.vfs[self.fs].f.code[pc]
    }
    pub fn borrow_mut_local_var(&mut self, id: usize) -> &mut LocVar {
        let fs = self.borrow_mut_fs(None);
        &mut fs.f.locvars[fs.actvar[id]]
    }
    pub fn get_code(&self, pc: usize) -> Instruction {
        self.vfs[self.fs].f.code[pc]
    }
    /// read next character in the stream
    pub fn next_char(&mut self,state: &mut LuaState) {
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
                    while !self.is_current_newline() && !self.current.is_none() {
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
                        return self.lex_error(state,
                            "invalid long string delimiter",
                            Some(Reserved::STRING as u32),
                        );
                    }
                }
                Some('=') => {
                    self.next_char(state);
                    match self.current {
                        Some('=') => {
                            self.next_char(state);
                            return Ok(Some(Reserved::EQ.into()));
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
                            return Ok(Some(Reserved::LE.into()));
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
                            return Ok(Some(Reserved::GE.into()));
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
                            return Ok(Some(Reserved::NE.into()));
                        }
                        _ => {
                            return Ok(Some(Token::new('~')));
                        }
                    }
                }
                Some('\"') | Some('\'') => {
                    let string_value = self.read_string(state,self.current.unwrap())?;
                    return Ok(Some(Token::new_string(&string_value)));
                }
                Some('.') => {
                    self.save_and_next(state);
                    if self.check_next(state,".") {
                        if self.check_next(state,".") {
                            // ...
                            return Ok(Some(Reserved::DOTS.into()));
                        }
                        // ..
                        return Ok(Some(Reserved::CONCAT.into()));
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
                        for i in 0..NUM_RESERVED as usize {
                            if TOKEN_NAMES[i] == iden {
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
            return self.syntax_error(state,"chunk has too many lines");
        }
        Ok(())
    }

    fn is_current_newline(&self) -> bool {
        match self.current {
            Some('\n') | Some('\r') => true,
            _ => false,
        }
    }
    fn is_current_digit(&self) -> bool {
        match self.current {
            Some(c) if c.is_digit(10) => true,
            _ => false,
        }
    }
    fn is_current_alphanumeric(&self) -> bool {
        match self.current {
            Some(c) if c.is_alphanumeric() => true,
            _ => false,
        }
    }
    fn is_current_alphabetic(&self) -> bool {
        match self.current {
            Some(c) if c.is_alphabetic() => true,
            _ => false,
        }
    }

    fn is_current(&self, arg: char) -> bool {
        match self.current {
            Some(c) if c == arg => true,
            _ => false,
        }
    }

    fn is_current_space(&self) -> bool {
        match self.current {
            Some(c) if c.is_whitespace() => true,
            _ => false,
        }
    }

    pub fn syntax_error(&self, state: &mut LuaState, msg: &str) -> Result<(), LuaError> {
        let token = if let Some(ref t) = self.t {
            Some(t.token)
        } else {
            None
        };
        self.lex_error(state, msg, token)
    }

    pub fn lex_error<D>(&self, state: &mut LuaState, msg: &str, t: Option<u32>) -> Result<D, LuaError> {
        let chunk_id = chunk_id(&self.source);
        state.push_string(
            &format!("{}:{}: {}", &chunk_id, self.linenumber, msg),
        );
        if let Some(t) = t {
            state.push_string(
                &format!("{} near '{}'", msg, self.token_2_txt(t)),
            );
        }
        if let Some(panic) = state.g.panic {
            panic(state);
        }
        Err(LuaError::SyntaxError)
    }

    pub fn token_2_txt(&self, t: u32) -> String {
        match t.try_into() {
            Ok(Reserved::NAME) | Ok(Reserved::STRING) | Ok(Reserved::NUMBER) => {
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
    fn skip_sep(&mut self,state: &mut LuaState) -> isize {
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
        &mut self, state: &mut LuaState,
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
                        Some(Reserved::EOS as u32),
                    )
                }
                Some('[') => {
                    if self.skip_sep(state) == sep {
                        // skip 2nd `['
                        self.save_and_next(state);
                        if sep == 0 {
                            return self
                                .lex_error(state,"nesting of [[...]] is deprecated", Some('[' as u32));
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

    fn save_and_next(&mut self,state: &mut LuaState) {
        self.save(self.current.unwrap());
        self.next_char(state);
    }

    fn save(&mut self, c: char) {
        self.buff.push(c);
    }

    fn read_string(&mut self, state: &mut LuaState,delimiter: char) -> Result<String, LuaError> {
        self.save_and_next(state);
        let mut c: char;
        loop {
            match self.current {
                Some(c) if c == delimiter => {
                    break;
                }
                None => {
                    return self.lex_error(state,"unfinished string", Some(Reserved::EOS as u32));
                }
                Some('\r') | Some('\n') => {
                    return self.lex_error(state,"unfinished string", Some(Reserved::STRING as u32));
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
                            if !c.is_digit(10) {
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
                                    i = i + 1;
                                }
                                if value > CHAR_MAX {
                                    return self.lex_error(state,
                                        "escape sequence too large",
                                        Some(Reserved::STRING as u32),
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
    fn check_next(&mut self,state: &mut LuaState, arg: &str) -> bool {
        if let Some(c) = self.current {
            if arg.contains(c) {
                self.save_and_next(state);
                return true;
            }
        }
        false
    }

    /// returns an error if we did not reach end of stream
    pub fn check_eos(&mut self,state: &mut LuaState) -> Result<(), LuaError> {
        if self.current.is_some() {
            return self.syntax_error(state,&format!(
                "'{}' expected",
                self.token_2_txt(Reserved::EOS as u32)
            ));
        }
        Ok(())
    }

    fn read_numeral(&mut self,state: &mut LuaState) -> Result<f64, LuaError> {
        debug_assert!(self.is_current_digit());
        self.save_and_next(state);
        while self.is_current_digit() || self.is_current('.') {
            self.save_and_next(state);
        }
        if self.check_next(state,"Ee") {
            // optional exponent sign
            self.check_next(state,"+-");
        }
        while self.is_current_alphanumeric() || self.is_current('_') {
            self.save_and_next(state);
        }
        let svalue = self.buff.iter().cloned().collect::<String>();
        // follow locale for decimal point
        let svalue = svalue.replace('.', &self.decpoint);
        svalue.parse::<f64>().map_err(|_| {
            self.lex_error::<()>(state,"malformed number", Some(Reserved::NUMBER as u32))
                .ok();
            LuaError::SyntaxError
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

    pub(crate) fn error_limit(&self,state: &mut LuaState, limit: usize, what: &str) -> Result<(), LuaError> {
        let msg = {
            let fs = self.borrow_fs(None);
            if fs.f.linedefined == 0 {
                format!("main function has more than {} {}", limit, what)
            } else {
                format!(
                    "function at line {} has more than {} {}",
                    fs.f.linedefined, limit, what
                )
            }
        };
        self.lex_error(state,&msg, None)
    }

    pub(crate) fn look_ahead(&mut self, state: &mut LuaState) -> Result<(), LuaError> {
        debug_assert!(self.is_lookahead_token(Reserved::EOS as u32));
        self.lookahead = self.lex(state)?;
        Ok(())
    }
}
