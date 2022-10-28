use super::pos::Pos;
use super::{expr::Expr, op::Op};
use std::fmt::{Debug, Display};
/// 常量,是区分出if/else/return的参考标准
/// 废除impl
pub static TOKENS: [&str; 5] = ["if", "else", "return", "pub", "import"];
/// terl的变量名
///
#[derive(Debug, Clone)]
pub struct Name {
    name: Vec<char>,
}

impl Name {
    pub fn new(name: Vec<char>) -> Self {
        Self { name }
    }
    pub fn get_name(&self) -> Vec<char> {
        self.name.clone()
    }

    pub fn set_name(&mut self, name: Vec<char>) {
        self.name = name;
    }
}
#[derive(Debug, Clone)]
pub struct Type {
    name: Name,
}
impl Type {
    pub fn new(name: Name) -> Self {
        Self { name }
    }
}
/// 扩展的Token,带有位置信息
/// (方便报错)
#[derive(Clone)]
pub struct FullToken {
    pos: Pos,
    token: Token,
}
impl FullToken {
    pub fn new(pos: Pos, token: Token) -> Self {
        Self { pos, token }
    }
    pub fn get_token(&self) -> Token {
        self.token.clone()
    }
    pub fn get_pos(&self) -> Pos {
        self.pos.clone()
    }
}
impl Debug for FullToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {}\n", self.token, self.pos.to_string())
    }
}
/// 标准Token枚举
/// - 一个关键字
/// - 一个数字
/// - 一个符号
#[derive(Debug, Clone)]
pub enum Token {
    Keyword(Vec<char>),
    Number(usize),
    Symbol(Op),
}

impl Token {
    /// Returns `true` if the token is [`Keyword`].
    ///
    /// [`Keyword`]: Token::Keyword
    #[must_use]
    pub fn is_keyword(&self) -> bool {
        matches!(self, Self::Keyword(..))
    }

    /// Returns `true` if the token is [`Number`].
    ///
    /// [`Number`]: Token::Number
    // #[must_use]
    // pub fn is_number(&self) -> bool {
    //     matches!(self, Self::Number(..))
    // }

    /// Returns `true` if the token is [`Symbol`].
    ///
    /// [`Symbol`]: Token::Symbol
    #[must_use]
    pub fn is_symbol(&self) -> bool {
        matches!(self, Self::Symbol(..))
    }

    pub fn is_op_br1l(&self) -> bool {
        matches!(self, Self::Symbol(Op::B1l))
    }
    pub fn is_op_assi(&self) -> bool {
        matches!(self, Self::Symbol(Op::Assign))
    }
}
impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Keyword(_key) => {
                let mut str = String::new();
                for char in _key {
                    let c = String::from(char.to_owned());
                    str += c.as_str();
                }

                write!(f, "{}", str)
            }
            Token::Number(_number) => write!(f, "{}", _number),
            Token::Symbol(_op) => write!(f, "{:?}", _op),
        }
    }
}
impl Into<Expr> for Token {
    fn into(self) -> Expr {
        return match self {
            Token::Keyword(_k) => Expr::Key(Name::new(_k)),
            Token::Number(_n) => Expr::Value(_n),
            Token::Symbol(_o) => Expr::Op(_o),
        };
    }
}
