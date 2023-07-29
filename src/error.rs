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
}

impl ErrorKind {
    pub fn generate_error(self, token: &Token) -> Error {
        Error {
            location: token.location.into(),
            kind: self,
            note: String::new(),
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
