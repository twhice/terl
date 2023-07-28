use crate::lexer::{Location, Token};

#[derive(Debug, Clone)]
pub struct Error {
    pub location: Option<Location>,
    pub error: ErrorKind,
}

impl Error {
    pub fn empty() -> Self {
        Self {
            location: None,
            error: ErrorKind::None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    None,
    ShouldBe {
        should_be: String,
        should_not_be: String,
    },
    Not(String),
    NotOneOf(Vec<String>),
}

impl ErrorKind {
    pub fn to_error(self, token: &Token) -> Error {
        Error {
            location: token.location.into(),
            error: self,
        }
    }

    pub fn error(self) -> Error {
        Error {
            location: None,
            error: self,
        }
    }
}
