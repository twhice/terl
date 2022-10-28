use super::{
    basic::{Name, Type},
    expr::Expr,
    pos::Pos,
    tree::{BeTree, Tree},
};

#[derive(Debug, Clone)]
pub struct Word {
    deep: usize,
    is_pub: bool,
    pos: Pos,
    contest: Context,
}
impl Word {
    pub fn new(deep: usize) -> Self {
        Self {
            deep,
            pos: Pos::new(),
            is_pub: false,
            contest: Context::Begin,
        }
    }

    pub fn set_deep(&mut self, deep: usize) {
        self.deep = deep;
    }

    pub fn set_is_pub(&mut self, is_pub: bool) {
        self.is_pub = is_pub;
    }
    pub fn context(&mut self) -> &mut Context {
        &mut self.contest
    }
    pub fn set_pos(&mut self, pos: Pos) {
        self.pos = pos
    }
}
impl BeTree for Word {
    fn deep(&self) -> usize {
        self.deep
    }
    // 不会使用
    fn is_left_part(&self) -> bool {
        todo!()
    }

    fn is_right_part(&self) -> bool {
        todo!()
    }
}
type Block = Tree<Word>;
#[derive(Debug, Clone)]
pub enum Context {
    Begin,
    New,
    If(Expr, Block),
    Else(Block),
    Return(Expr),
    DefFun(Name, Expr, Block, Type),
    DefVul(Name, Expr, Type),
    DefStruct(Name, Block, Vec<Type>),
    Import(Expr),
    Empty,
}
