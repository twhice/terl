use std::fmt::Display;

use crate::lexer::{Location, Token};

#[derive(Debug, Clone)]
pub struct Error {
    pub location: Option<Location>,
    pub note: String,
    pub kind: ErrorKind,
}

impl Error {
    pub fn empty() -> Self {
        Self {
            location: None,
            kind: ErrorKind::None,
            note: String::new(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    None,
    /// 赋值运算符两边的元素数量不一致
    CantAss,
    NotOneOf(Vec<String>),
    Not(String),
    UnExpect,
    UnDefinedVar(String),
    /// 先前定义的位置
    DoubleFnDefine(Location),
    CallUnDefinedFn(String),
    /// 定义的位置 函数的参数的个数 调用时传入的个数
    CallFnWithIncorrectArgs(Location, usize, usize),
}

impl ErrorKind {
    pub fn generate_error(self, token: &Token) -> Error {
        self.make_error(token.location)
    }

    pub fn make_error(self, location: Location) -> Error {
        Error {
            location: location.into(),
            note: String::new(),
            kind: self,
        }
    }

    pub fn not_one_of(of: &[&str]) -> Self {
        Self::NotOneOf(of.iter().copied().map(|s| s.to_owned()).collect())
    }

    pub fn not(be: &str) -> Self {
        Self::Not(be.to_owned())
    }

    pub fn unexpect() -> Self {
        Self::UnExpect
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn none() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone)]
pub struct Warn {
    pub location: Option<Location>,
    pub note: String,
    pub kind: WarnKind,
}

impl Warn {
    pub fn empty() -> Self {
        Self {
            location: None,
            kind: WarnKind::None,
            note: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum WarnKind {
    None,
}

impl WarnKind {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn none() -> Self {
        Self::None
    }
}
