use crate::core::tree::Tree;

use super::{
    basic::{FullToken, Name, Token, Type, TOKENS},
    context::{Context, Word},
    expr::{self, Expr},
    op::Op,
    pos::Pos,
};

#[derive(Clone, Debug)]
/// 文法解析器
///
/// - new      构造
/// - lex      读取一行文本并解析
/// - get_word 读取解析结果
///

pub struct Lexer {
    line: Vec<char>, //一行
    temp: Vec<char>, //缓存
    fulltokens: Vec<FullToken>,
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
            fulltokens: Vec::new(),
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
    pub fn lex(&mut self, src: Vec<char>) -> &mut Self {
        self.line = src;
        self.basic_init_line();
        while !self.finish {
            self.basic_lex();
        }
        self.basic_finish_line();
        self.word.set_pos(self.pos.clone());
        self.build();
        self
    }
    pub fn get_word(&self) -> Word {
        self.word.clone()
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
            self.fulltokens.push(FullToken::new(
                self.pos.clone(),
                Token::Keyword(self.temp.clone()),
            ));
            self.temp.clear();
        } else if self.getting_num {
            self.fulltokens
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
                    self.fulltokens
                        .push(FullToken::new(self.pos.clone(), Token::Symbol(_op)));
                    return;
                }
            }
            if let Some(_op) = Op::from_char(currten) {
                self.fulltokens
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
    fn build(&mut self) {
        //  根据第一个字符就可以确定: 是定义/流程
        if !self.fulltokens.is_empty() {
            let ft = &self.fulltokens[0];
            if let Token::Keyword(_key) = ft.get_token() {
                let mut key = 0;
                for index in 0..TOKENS.len() {
                    if _key == TOKENS[index].chars().collect::<Vec<char>>() {
                        key = index + 1;
                        break;
                    } else {
                        key = 0;
                    }
                }
                type Block = Tree<Word>;
                match key {
                    1 => {
                        *self.word.context() = Context::If(Expr::Value(0), Block::Node(Vec::new()));
                        self.fill_if();
                        return;
                    }

                    2 => {
                        *self.word.context() = Context::Else(Block::Node(Vec::new()));
                        self.fill_else();
                        return;
                    }
                    3 => {
                        *self.word.context() = Context::Return(Expr::Value(0));
                        self.fill_return();
                        return;
                    }
                    5 => {
                        *self.word.context() = Context::Import(Expr::Value(0));
                        self.fill_import();
                        return;
                    }
                    4 => {
                        self.word.set_is_pub(true);
                        // 删除 "pub"
                        self.fulltokens.remove(0);
                        *self.word.context() = Context::New;
                    }
                    _ => {
                        self.word.set_is_pub(false);
                        *self.word.context() = Context::New;
                    }
                }
            } else {
                self.error(&format!(
                    "Expect a str,but found{}{}",
                    ft.get_token(),
                    ft.get_pos()
                ))
            }
        } else {
            *self.word.context() = Context::Empty;
            return;
        }
        // 剩下的,就是一个定义

        // 空的
        if self.fulltokens.len() == 0 {
            self.error(&format!("Expect a name,but found nothing{}", self.pos))
        }
        //注册(?) 一个vul
        else if self.fulltokens.len() == 1 {
            if let Token::Keyword(_name) = self.fulltokens[0].get_token() {
                *self.word.context() = Context::DefVul(
                    Name::new(_name),
                    Expr::Value(0),
                    Type::new(Name::new("auto".chars().collect())),
                );
                return;
            } else {
                let l = crate::BRACKET_L;
                let r = crate::BRACKET_R;
                self.error(&format!(
                    "Expect a name,but found {l}{}{r}{}",
                    self.fulltokens[0].get_token(),
                    self.fulltokens[0].get_pos()
                ))
            }
        } else if self.fulltokens[0].get_token().is_keyword()
            && self.fulltokens[1].get_token().is_symbol()
        {
            self.build_by_op()
        } else {
            let l = crate::BRACKET_L;
            let r = crate::BRACKET_R;
            self.error(&format!(
                "Expect a name,but found {l}{}{r}{}",
                self.fulltokens[1].get_token(),
                self.fulltokens[1].get_pos()
            ))
        }
    }
    // 好日子还在后头呢
    // 看看这一堆可癌的方法

    // const构造
    fn deafult_build_con(&mut self) -> Expr {
        // 获取位置信息(,方便报错)
        let pos = self.fulltokens[0].get_pos();
        self.fulltokens.remove(0);
        // 没有表达式
        if self.fulltokens.is_empty() {
            let l = crate::BRACKET_L;
            let r = crate::BRACKET_R;
            self.error(&format!(
                "Expect a expr after {l}if{r},but found nothing{}",
                pos
            ))
        }
        let (expr, endex) = self.try_collect_expr(0);
        // 表达式后有多余
        if endex != self.fulltokens.len() {
            let l = crate::BRACKET_L;
            let r = crate::BRACKET_R;
            self.error(&format!(
                "Expect nothing after expr ,but found  {l}{}{r}{}",
                self.fulltokens[endex].get_token(),
                pos
            ))
        }
        return expr;
    }
    fn fill_if(&mut self) {
        *self.word.context() = Context::If(self.deafult_build_con(), Tree::Node(Vec::new()))
    }
    fn fill_else(&mut self) {
        if self.fulltokens.len() != 1 {
            let l = crate::BRACKET_L;
            let r = crate::BRACKET_R;
            self.error(&format!(
                "Expect nothing after {l}else{r} ,but found  {l}{}{r}{}",
                self.fulltokens[1].get_token(),
                self.fulltokens[1].get_token(),
            ))
        }
        *self.word.context() = Context::Else(Tree::Node(Vec::new()));
    }
    fn fill_return(&mut self) {
        *self.word.context() = Context::Return(self.deafult_build_con());
    }
    fn fill_import(&mut self) {
        *self.word.context() = Context::Import(self.deafult_build_con());
    }
    // define构造
    unsafe fn deafult_build_def(&mut self, t: Type, name: Name, fft: FullToken) {
        let this = self as *mut Lexer;
        let err_not_expr = || {
            (*this).error(&format!(
                "Expect a expr after{},but found nothing{}",
                fft.get_token(),
                fft.get_pos()
            ))
        };
        let err_unknow_after = |index: usize| {
            (*this).error(&format!(
                "Expect nothing,but found{}{}",
                (*this).fulltokens[index].get_token(),
                (*this).fulltokens[index].get_pos()
            ))
        };
        let err_not_name = |index: usize| {
            let l = crate::BRACKET_L;
            let r = crate::BRACKET_R;
            (*this).error(&format!(
                "Expect a name,but found{l}{}{r}{}",
                (*this).fulltokens[index].get_token(),
                (*this).fulltokens[index].get_pos()
            ))
        };
        let len = (*this).fulltokens.len();
        if len == 0 {
            err_not_expr();
        }
        let (expr, endex) = self.try_collect_expr(0);
        // 有多余 edefv/edeff+t/deff+t
        if endex < len {
            if (*this).fulltokens[0].get_token().is_op_br1l() {
                if endex == len - 1 {
                    if let Token::Keyword(_type_name) = (*this).fulltokens[endex].get_token() {
                        let return_type = Type::new(Name::new(_type_name));
                        *(*this).word.context() = Context::DefFun(
                            name.clone(),
                            expr,
                            Tree::Node(Vec::new()),
                            return_type,
                        );
                        // ohhhhhhhhhhhhhhhhhhhhhhhhhhhhh
                        return;
                    } else {
                        err_not_name(endex);
                    }
                    // deff+t
                } else {
                    // edef+t
                    err_unknow_after(len - 1)
                }
            } else {
                // edefv
                err_unknow_after(endex)
            }
        // 没多余 defv/deff
        } else {
            if (*this).fulltokens[0].get_token().is_op_br1l() {
                *(*this).word.context() =
                    Context::DefFun(name.clone(), expr, Tree::Node(Vec::new()), t);
                return;
            } else {
                *(*this).word.context() = Context::DefVul(name.clone(), expr, t);
                return;
            }
        }
    }
    fn build_by_op(&mut self) {
        // token_temp:
        // 0 name
        // 1 op
        // 废话获取 1
        let mut name = Name::new(Vec::new());
        if let Token::Keyword(_name) = self.fulltokens[0].get_token() {
            name.set_name(_name);
            self.fulltokens.remove(0);
        }
        // 废话获取2
        let fft = self.fulltokens[0].clone();
        let mut op = Op::None;
        if let Token::Symbol(_op) = fft.get_token() {
            op = _op;
            if let Op::B1l = op {
            } else {
                self.fulltokens.remove(0);
            }
        }

        // 喜闻乐见裸指针
        let this = self as *mut Lexer;

        // 各种错误
        let err_not_name = |index: usize| unsafe {
            (*this).error(&format!(
                "Expect a name,but found{}{}",
                (*this).fulltokens[index].get_token(),
                (*this).fulltokens[index].get_pos()
            ))
        };
        let err_not_assi = |index: usize| unsafe {
            let l = crate::BRACKET_L;
            let r = crate::BRACKET_R;
            (*this).error(&format!(
                "Expect a {l}={r},but found{}{}",
                (*this).fulltokens[index].get_token(),
                (*this).fulltokens[index].get_pos()
            ))
        };

        //2个闭包

        let build_by_type = || unsafe {
            // 绝对是结构体
            if (*this).fulltokens.len() <= 1 {
                let mut externs = Vec::new();
                for fulltoken in &(*this).fulltokens {
                    if let Token::Keyword(_extern_typename) = fulltoken.get_token() {
                        externs.push(Type::new(Name::new(_extern_typename)));
                    } else {
                        let l = crate::BRACKET_L;
                        let r = crate::BRACKET_R;
                        (*this).error(&format!(
                            "Expect a type-name as extern ,but found {l}{}{r}{}",
                            fulltoken.get_token(),
                            fulltoken.get_pos()
                        ))
                    }
                }
                *(*this).word.context() =
                    Context::DefStruct(name.clone(), Tree::Node(Vec::new()), externs);
                return;
            }
            // len>=2
            else {
                if let Token::Keyword(_typename) = (*this).fulltokens[0].get_token() {
                    let t = Type::new(Name::new(_typename));
                    (*this).fulltokens.remove(0);
                    if (*this).fulltokens[0].get_token().is_op_assi() {
                        (*this).fulltokens.remove(0);
                        (*this).deafult_build_def(t, name.clone(), fft.clone());
                        return;
                    } else {
                        err_not_assi(0);
                    }
                } else {
                    err_not_name(0);
                }
            }
        };
        let build_by_br1l = || unsafe {
            (*this).deafult_build_def(Type::from("void"), name.clone(), fft.clone());
            return;
        };

        match op {
            // a=xxx
            // a=(args):$t
            Op::Assign => unsafe { self.deafult_build_def(Type::from("void"), name, fft) },
            // a:t=(args):$block
            // a:t=$expr
            // a:t  结构体,不允许这样定义变量/函数
            // a: +$extern_froms
            Op::Type => build_by_type(),
            // a()...
            Op::B1l => build_by_br1l(),
            _ => {
                panic!("看看你写的什么代码")
            }
        }
    }
    fn try_collect_expr(&mut self, start_index: usize) -> (Expr, usize) {
        let mut tokens = Vec::new();
        let mut poss = Vec::new();
        for fulltoken in &self.fulltokens {
            tokens.push(fulltoken.get_token());
            poss.push(fulltoken.get_pos());
        }
        let tokens = tokens[start_index..].to_vec();
        let l = crate::BRACKET_L;
        let r = crate::BRACKET_R;
        match expr::build_expr(&tokens) {
            Ok(_expr) => return _expr,
            Err(_err) => match _err {
                super::error::TerlError::ExpectAVul(_index) => self.error(&format!(
                    "Expect a vul ,bul found{l}{}{r}{}",
                    self.fulltokens[start_index + _index].get_token(),
                    self.fulltokens[start_index + _index].get_pos()
                )),
                super::error::TerlError::ExpectASymbol(_index) => self.error(&format!(
                    "Expect a symbol ,bul found{l}{}{r}{}",
                    self.fulltokens[start_index + _index].get_token(),
                    self.fulltokens[start_index + _index].get_pos()
                )),
                super::error::TerlError::MissBeacket(_index) => self.error(&format!(
                    "miss bracket{}",
                    // self.fulltokens[start_index + _index].get_token(),
                    self.fulltokens[start_index + _index].get_pos()
                )),
            },
        };
        todo!()
    }
    // 错误处理
    fn error(&self, meg: &str) {
        panic!("{}", meg)
    }
}
