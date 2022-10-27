use super::op::Op;
use super::pos::Pos;
use std::fmt::Debug;
/// 常量,是区分出if/else/return的参考标准
/// 废除impl
pub static TOKENS: [&str; 4] = ["if", "else", "return", "impl"];
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
}
#[derive(Debug, Clone)]
pub struct Type {}
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
