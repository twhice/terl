use super::{
    basic::{FullToken, Name, Token},
    context::{Context, Word},
    op::Op,
    pos::Pos,
};

#[derive(Clone, Debug)]
pub struct Lexer {
    line: Vec<char>, //一行
    temp: Vec<char>, //缓存
    tokens_temp: Vec<FullToken>,
    word: Word,
    // 状态
    finish: bool,
    getting_num: bool,
    begin: bool,
    number: usize,
    deep: usize,
    index: usize,
    pos: Pos,
}
impl Lexer {
    pub fn new() -> Self {
        Self {
            line: Vec::new(),
            temp: Vec::new(),
            // tokens: Vec::new(),
            tokens_temp: Vec::new(),
            word: Word::new(0),
            finish: false,
            getting_num: false,
            begin: true,
            index: 0,
            number: 0,
            deep: 0,
            pos: Pos::new(),
        }
    }
    pub fn lex(&mut self, src: Vec<char>) {
        self.line = src;
        self.basic_init_line();
        while !self.finish {
            self.basic_lex();
        }
        self.basic_finish_line();
        self.lex2();
    }
    // basic
    fn get(&self) -> char {
        if !self.line.is_empty() && self.index < self.line.len() {
            return self.line[self.index].clone();
        } else {
            return '\0';
        }
    }
    fn next(&mut self) -> char {
        self.index += 1;
        self.pos.pass();
        self.get()
    }
    fn basic_init_line(&mut self) {
        self.pos.new_line();
        // self.basic_endcurrt();
        // self.index = 0;
    }
    fn basic_finish_line(&mut self) {
        if self.finish {
            // 重置状态
            self.finish = false;
            // 收尾截取
            self.basic_endcurrt();
            // deep载入 清理
            self.word.set_deep(self.deep);
            self.deep = 0;
            self.index = 0;
        }
    }
    fn basic_endcurrt(&mut self) {
        if !self.temp.is_empty() {
            self.tokens_temp.push(FullToken::new(
                self.pos.clone(),
                Token::Keyword(self.temp.clone()),
            ));
            self.temp.clear();
        } else if self.getting_num {
            self.tokens_temp
                .push(FullToken::new(self.pos.clone(), Token::Number(self.number)));
            self.number = 0;
            self.getting_num = false;
        }
    }
    fn basic_lex(&mut self) {
        let currten = self.get();
        if currten == '\0' {
            self.finish = true;
            return;
        } else if currten == '/' {
            let next = self.next();
            if next == currten {
                self.finish = true;
                return;
            } else {
                let l = crate::BRACKET_L;
                let r = crate::BRACKET_R;
                self.error(&format!(
                    "Expect {l}/{r},find {l}{}{r} {}",
                    next,
                    self.pos.to_string()
                ))
            }
        }
        // 字符串
        else if currten.is_ascii_alphabetic() || currten == '_' {
            self.begin = false;
            self.temp.push(currten);
            let awa = |char: char| -> bool { char.is_ascii_alphanumeric() || char == '_' };
            while awa(self.next()) {
                self.temp.push(self.get())
            }
            return;
        } else if currten.is_ascii_digit() {
            self.begin = false;
            self.getting_num = true;
            self.number = self.get() as usize - '0' as usize;
            while self.next().is_ascii_digit() {
                self.number *= 10;
                self.number += self.get() as usize - '0' as usize;
            }
            return;
        } else if currten.is_ascii_whitespace() {
            self.basic_endcurrt();
            while self.next().is_ascii_whitespace() {
                if currten == '\t' && self.begin {
                    self.deep += 1;
                }
            }
            return;
        } else if currten.is_ascii_punctuation() {
            self.basic_endcurrt();
            if self.next().is_ascii_punctuation() {
                if let Some(_op) = Op::from_char2(currten, self.get()) {
                    self.tokens_temp
                        .push(FullToken::new(self.pos.clone(), Token::Symbol(_op)));
                    return;
                }
            }
            if let Some(_op) = Op::from_char(currten) {
                self.tokens_temp
                    .push(FullToken::new(self.pos.clone(), Token::Symbol(_op)));
                return;
            } else {
                self.pos.back();
                let l = crate::BRACKET_L;
                let r = crate::BRACKET_R;
                self.error(&format!("Unknow symbol {l}{currten}{r} {}", self.pos));
            }
        }
    }
    // more
    fn lex2(&mut self) {
        let mut tokens = Vec::new();
        let mut poss = Vec::new();
        for ft in &self.tokens_temp {
            tokens.push(ft.get_token());
            poss.push(ft.get_pos());
        }
        for i in 0..tokens.len() {
            let token = tokens[i].clone();
            // 分清1类(完整句) 2类(残缺句)
            if i == 0 {
                // 完整
                if let Token::Keyword(_str) = token {
                    self.word.context().up(_str);
                // 残缺
                } else {
                    self.word.context().up_to_exprs();
                    self.fill_expr();
                    return;
                }
            // 残缺句已退出:补全立即句,区分定义句
            } else if i == 1 {
                match self.word.context() {
                    super::context::Context::If(_, _) => {
                        self.fill_if();
                        return;
                    }
                    super::context::Context::Else(_) => {
                        self.fill_else();
                        return;
                    }
                    super::context::Context::Return(_) => {
                        self.fill_return();
                        return;
                    }
                    _ => {
                        if let Token::Symbol(_op) = token {
                            match _op {
                                Op::B1l => {
                                    *self.word.context() = Context::DefFun(
                                        Name::new(Vec::new()),
                                        super::expr::Expr::Value(0),
                                        super::tree::Tree::Node(Vec::new()),
                                    );
                                    self.fill_deff();
                                    return;
                                }
                                Op::Assign => {
                                    *self.word.context() = Context::DefVul(
                                        Name::new(Vec::new()),
                                        super::expr::Expr::Value(0),
                                    );
                                    self.fill_vul();
                                    return;
                                }
                                Op::DefStruct => {
                                    *self.word.context() = Context::DefStruct(
                                        Name::new(Vec::new()),
                                        super::tree::Tree::Node(Vec::new()),
                                    );
                                    self.fill_struct();
                                    return;
                                }
                                _ => todo!("排除在外"),
                            }
                        }
                    }
                }
            } else {
                return;
            }
        }
    }
    // 好日子还在后头呢
    // 看看这一堆可癌的方法
    fn fill_if(&mut self) {}
    fn fill_else(&mut self) {}
    fn fill_return(&mut self) {}
    fn fill_vul(&mut self) {}
    fn fill_deff(&mut self) {}
    fn fill_struct(&mut self) {}
    fn fill_expr(&mut self) {}
    fn error(&self, meg: &str) {
        panic!("{}", meg)
    }
}
