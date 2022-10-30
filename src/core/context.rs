use std::fmt::Debug;

use super::{
    basic::{Name, Type},
    expr::Expr,
    pos::Pos,
    tree::{BeTree, Tree},
};

#[derive(Clone)]
pub struct Statement {
    deep: usize,
    is_pub: bool,
    pos: Pos,
    context: Context,
}
impl Statement {
    pub fn new(deep: usize) -> Self {
        Self {
            deep,
            pos: Pos::new(),
            is_pub: false,
            context: Context::Begin,
        }
    }

    pub fn set_is_pub(&mut self, is_pub: bool) {
        self.is_pub = is_pub;
    }
    pub fn context(&mut self) -> &mut Context {
        &mut self.context
    }
    pub fn set_pos(&mut self, pos: Pos) {
        self.pos = pos
    }
}
impl Debug for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\nStatement:\n\tdeep: {}\n\tis_pub: {}\n\tpos: {:?}\n\tcontext: {:?}",
            self.deep, self.is_pub, self.pos, self.context
        )
    }
}
impl BeTree for Statement {
    fn get_deep(&self) -> usize {
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
type Block = Tree<Statement>;
#[derive(Clone)]
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
impl Context {
    pub fn fill_block(&mut self, block: Block) {
        match self.clone() {
            Context::If(_e, _b) => {
                *self = Self::If(_e, block);
                return;
            }
            Context::Else(_b) => {
                *self = Self::Else(block);
                return;
            }
            Context::DefFun(_n, _e, _b, _t) => {
                *self = Self::DefFun(_n, _e, block, _t);
                return;
            }
            Context::DefStruct(_n, _b, _ts) => {
                *self = Self::DefStruct(_n, block, _ts);
                return;
            }

            _ => {
                return;
            }
        }
    }
}
impl Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let l = crate::BRACKET_L;
        let r = crate::BRACKET_R;
        match self {
            Self::Begin => write!(f, "BEGIN"),
            Self::New => write!(f, "NEW"),
            Self::Empty => write!(f, "EMPTY"),

            Context::If(_cond, _block) => write!(f, "IF({_cond:?}):{_block:?}"),
            Context::Else(_block) => write!(f, "ELSE:{_block:?}"),
            Context::Return(_e) => write!(f, "RETURN({_e:?})"),
            Context::DefFun(_n, _a, _b, _t) => write!(f, "{_t:?}{l}{_n:?}{r}({_a:?}):{_b:?}"),
            Context::DefVul(_n, _e, _t) => write!(f, "{_t:?}{l}{_n:?}{r}={_e:?}"),
            Context::DefStruct(_n, _b, _e_s) => write!(f, "{l}{_n:?}{r}:{_e_s:?} {_b:?}"),
            Context::Import(_) => todo!(),
        }
    }
}
