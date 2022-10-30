use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub enum Op {
    Add, // +
    Min, // -
    Mul, // *
    Dev, // /
    Mod, // %
    Inv, // ^

    Sl, // <<
    Sr, // >>

    Eq,       //==
    Greater,  //<
    Ngreater, //<=
    Smaller,  //>
    Nsmaller, //>=

    And, // &&
    Or,  // ||
    Not, // !

    Assign, // =
    // Return, // return
    // If,     // if
    // Else,   // else
    // Try,    // try
    // Catch,  // catch
    // Match,  // match
    // Def,    // fn
    B1l, // (
    B2l, // [
    B3l, // {
    // B4l,       // <
    B1r, // )
    B2r, // ]
    B3r, // }
    // B4r,       // >
    B5,        // "
    UnderLine, // _

    None,
    Dot,     // .
    Tied,    // ,
    Cite,    // &
    Nothing, // \

    Exit,  // ?
    Lexer, // ;

    Type,
    Incomplete,
}
impl Op {
    pub fn from_char(c: char) -> Option<Self> {
        let o = match c {
            '+' => Self::Add,
            '-' => Self::Min,
            '*' => Self::Mul,
            '/' => Self::Dev,
            '%' => Self::Mod,
            '<' => Self::Smaller,
            '>' => Self::Greater,
            '=' => Self::Assign,
            '!' => Self::Not,
            '^' => Self::Inv,
            '(' => Self::B1l,
            ')' => Self::B1r,
            '[' => Self::B2l,
            ']' => Self::B2r,
            '{' => Self::B3l,
            '}' => Self::B3r,
            ';' => Self::Lexer,
            '&' => Self::Cite,
            '|' => Self::Incomplete,
            ':' => Self::Type,
            '.' => Self::Dot,
            ',' => Self::Tied,
            '\"' => Self::B5,
            '\\' => Self::Nothing,
            '_' => Self::UnderLine,
            '?' => Self::Exit,
            _ => Self::None,
        };
        if let Self::None = o {
            None
        } else {
            Some(o)
        }
    }
    pub fn from_char2(c1: char, c2: char) -> Option<Self> {
        let o = match (c1, c2) {
            ('&', '&') => Self::And,
            ('|', '|') => Self::Or,
            ('<', '<') => Self::Sl,
            ('>', '>') => Self::Sr,
            ('>', '=') => Self::Nsmaller,
            ('<', '=') => Self::Ngreater,
            ('=', '=') => Self::Eq,
            _ => Self::None,
        };
        if let Self::None = o {
            None
        } else {
            Some(o)
        }
    }
    pub fn priority(&self) -> usize {
        match self {
            // = < &&|| < <><= >= < ! <  +- < */ < ^ < () < other < .
            Self::B1r => 10, // )
            // Self::Assign => 10,  // =
            Self::Tied => 9, // ,
            // Self::Type => 8,     // :
            Self::Smaller => 8,  // <
            Self::Greater => 8,  // >
            Self::Nsmaller => 8, // >=
            Self::Ngreater => 8, // <=
            Self::And => 7,      // &&
            Self::Or => 7,       // ||
            Self::Add => 6,      // +
            Self::Min => 6,      // -
            Self::Mul => 5,      // *
            Self::Dev => 5,      // /
            Self::Mod => 5,      // %
            Self::Sl => 4,       // <<
            Self::Sr => 4,       // >>
            Self::Inv => 3,      // ^
            Self::Not => 2,      // !
            Self::B1l => 1,      // (
            _ => 0,              // others
        }
    }
}
impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::B1r => ")",       // )
                Self::Tied => ",",      // ,
                Self::Smaller => "<",   // <
                Self::Greater => ">",   // >
                Self::Nsmaller => ">=", // >=
                Self::Ngreater => ">=", // <=
                Self::And => "&&",      // &&
                Self::Or => "||",       // ||
                Self::Add => "+",       // +
                Self::Min => "-",       // -
                Self::Mul => "*",       // *
                Self::Dev => "/",       // /
                Self::Mod => "%",       // %
                Self::Inv => "^",       // ^
                Self::Not => "!",       // !
                Self::B1l => "(",       // (
                Self::Assign => "=",
                Self::Sl => "<<",
                Self::Sr => ">>",
                _ => " ", // others
            }
        )
    }
}
