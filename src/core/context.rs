use super::{
    basic::{FullToken, Name, TOKENS},
    expr::Expr,
    tree::{BeTree, Tree},
};

#[derive(Debug, Clone)]
pub struct Word {
    deep: usize,
    contest: Context,
}
impl Word {
    pub fn new(deep: usize) -> Self {
        Self {
            deep,
            contest: Context::Begin,
        }
    }

    pub fn set_deep(&mut self, deep: usize) {
        self.deep = deep;
    }

    pub fn deep(&self) -> usize {
        self.deep
    }

    pub fn context(&mut self) -> &mut Context {
        &mut self.contest
    }
}
impl BeTree for Word {
    fn deep(&self) -> usize {
        self.deep
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
    DefFun(Name, Expr, Block),
    DefVul(Name, Expr),
    DefStruct(Name, Block),
    OnlyExpr(Expr),
}
impl Context {
    // pub fn new(fts: Vec<FullToken>) {
    //     let mut ret = Self::Begin;
    //     let mut tokens = Vec::new();
    //     let mut poss = Vec::new();
    //     for ft in &fts {
    //         tokens.push(ft.get_token());
    //         poss.push(ft.get_pos());
    //     }
    //     let have_index = |index: usize| -> bool { !fts.is_empty() && index < fts.len() };
    //     if have_index(0) {
    //         //分类
    //     }
    // }
    pub fn up(&mut self, str: Vec<char>) {
        let mut id: usize = 0;
        for con in 0..TOKENS.len() {
            if str == TOKENS[con].chars().collect::<Vec<char>>() {
                id = con + 1;
            }
        }
        match id {
            1 => *self = Self::If(Expr::Value(0), Block::Node(Vec::new())),
            2 => *self = Self::Else(Block::Node(Vec::new())),
            3 => *self = Self::Return(Expr::Value(0)),
            // 4 => {}
            _ => *self = Self::New,
        }
    }
    pub fn up_to_exprs(&mut self) {
        *self = Self::OnlyExpr(Expr::Value(0));
    }
    pub fn expect_all(&self) -> &str {
        match self {
            Context::Begin => "",
            Context::If(_, _) => "neb",            //立刻得出(i0)
            Context::Else(_) => "ne",              //立刻得出(i0)
            Context::Return(_) => "ne",            //立刻得出(i0)
            Context::DefFun(_, _, _) => "k(e):tb", //其次得出(i2)
            Context::DefVul(_, _) => "k=e",        //其次得出(i2)
            Context::DefStruct(_, _) => "k:b",     //其次得出(i2)
            Context::OnlyExpr(_) => "e",           //最后考虑(i3)
            Context::New => todo!(),
        }
    }
}
