use super::basic::*;
use super::op::Op;
#[derive(Debug, Clone)]
pub enum Expr {
    P2(Box<Expr>, Op, Box<Expr>),
    P1(Op, Box<Expr>),
    Chain(Op, Vec<Expr>),
    Value(usize),
    KEy(Name),
}
impl Expr {
    pub fn build(fts: Vec<FullToken>) {
        let mut tokens = Vec::new();
        let mut poss = Vec::new();
        for ft in &fts {
            tokens.push(ft.get_token());
            poss.push(ft.get_pos());
        }
        // 优先级列表
        let mut p_s: Vec<usize> = Vec::new();
        for t in &tokens {
            let p = match t {
                Token::Keyword(_) => 0,
                Token::Number(_) => 0,
                Token::Symbol(_s) => _s.priority(),
            };
            p_s.push(p);
        }
        for iter_p in 2..10 {
            for p in &p_s {
                if *p == iter_p {}
            }
        }
    }
}
