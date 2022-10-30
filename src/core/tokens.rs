use super::expr::Expr;
use super::op::Op;
use super::pos::Pos;
use super::{basic::*, prompt::TerlError};
use std::fmt::{Debug, Display};

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
    pub fn error(&self, err: &TerlError) {
        let l = crate::BRACKET_L;
        let r = crate::BRACKET_R;
        let meg: String = match err {
            TerlError::ExpectVul(_) => "expect vul after".to_owned(),
            TerlError::ExpectSymbol(_) => "expect symbol after".to_owned(),
            TerlError::MissBeacket(_) => "miss bracket after".to_owned(),
            TerlError::ExpectName(_) => "expect name, but found".to_owned(),
            TerlError::ExpectTypeName(_) => "expect typename after".to_owned(),
            TerlError::ExpectNothing(_) => "expect nothing after".to_owned(),
            TerlError::ExpectExpr(_) => "expect expr after".to_owned(),
            TerlError::ExpectOneOf(_arms) => {
                let mut ret = String::new();
                ret += "Exprct one of ";
                for arm in _arms {
                    ret += &format!("{l}{arm}{r}");
                }
                ret += "but found";
                ret
            }
        };
        panic!("\n{meg}{l}{}{r}{}\n", self.get_token(), self.get_pos());
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
    pub fn is_keyword(&self) -> bool {
        matches!(self, Self::Keyword(..))
    }
    pub fn is_symbol(&self) -> bool {
        matches!(self, Self::Symbol(..))
    }
    pub fn is_op_br1l(&self) -> bool {
        matches!(self, Self::Symbol(Op::B1l))
    }
    pub fn is_op_assi(&self) -> bool {
        matches!(self, Self::Symbol(Op::Assign))
    }
    pub fn get_in_keyword(&self) -> Option<Vec<char>> {
        if let Token::Keyword(_key) = self {
            Some(_key.clone())
        } else {
            None
        }
    }
    pub fn get_in_symbol(&self) -> Option<Op> {
        if let Token::Symbol(_op) = self {
            Some(_op.clone())
        } else {
            None
        }
    }
    // pub fn get_in_number(&self) -> Option<usize> {
    //     if let Token::Number(_num) = self {
    //         Some(_num.clone())
    //     } else {
    //         None
    //     }
    // }pub fn get_in_number(&self) -> Option<usize> {
    //     if let Token::Number(_num) = self {
    //         Some(_num.clone())
    //     } else {
    //         None
    //     }
    // }
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
            Token::Symbol(_op) => write!(f, "{}", _op),
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
