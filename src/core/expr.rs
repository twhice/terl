use super::basic::*;
use super::error::TerlError;
use super::op::Op;
use super::tree::BeTree;
use std::fmt::Debug;
#[derive(Debug, Clone)]
pub enum Expr {
    P2(Box<Expr>, Box<Expr>, Box<Expr>),
    P1(Box<Expr>, Box<Expr>),
    Chain(Box<Expr>, Vec<Expr>),
    Value(usize),
    Key(Name),
    Op(Op),
}
impl Expr {
    /// Returns `true` if the expr is [`Op`].
    ///
    /// [`Op`]: Expr::Op
    #[must_use]
    pub fn tryget_priority(&self) -> usize {
        if let Self::Op(_op) = self {
            _op.priority()
        } else {
            0
        }
    }
}
impl BeTree for Expr {
    fn deep(&self) -> usize {
        // 永远不用
        todo!()
    }

    fn is_left_part(&self) -> bool {
        if let Self::Op(_op) = self {
            matches!(_op, Op::B1l)
        } else {
            false
        }
    }

    fn is_right_part(&self) -> bool {
        if let Self::Op(_op) = self {
            matches!(_op, Op::B1r)
        } else {
            false
        }
    }
}
/// unsafe!
pub fn collext_expr(tokens: &Vec<Token>) -> Result<(Expr, usize), TerlError> {
    // 准备
    let tokens = tokens.clone();
    let mut exprs: Vec<Expr> = Vec::new();
    let mut priority_s: Vec<usize> = Vec::new();
    // 初始化优先级列表&&表达式列表
    for token in tokens {
        let expr: Expr = token.into();
        if let Expr::Op(_op) = &expr {
            priority_s.push(_op.priority());
        } else {
            priority_s.push(0);
        }
        exprs.push(expr)
    }
    // 截取合适长度
    let len = get_expr_len(&exprs)?;
    let mut exprs: Vec<Expr> = exprs[..len].to_vec();
    // 解决该死的所有权
    let prio_ptr = &mut priority_s as *mut Vec<usize>;
    let expr_ptr = &mut exprs as *mut Vec<Expr>;
    // 闭包:提取表达式
    let build_op2 = |atsp: usize| unsafe {
        (*expr_ptr)[atsp - 1] = Expr::P2(
            Box::new(exprs[atsp - 1].clone()),
            Box::new(exprs[atsp].clone()),
            Box::new(exprs[atsp + 1].clone()),
        );
        (*expr_ptr).remove(atsp);
        (*prio_ptr).remove(atsp);
        (*expr_ptr).remove(atsp);
        (*prio_ptr).remove(atsp);
    };
    let build_op1 = |atsp: usize| unsafe {
        (*expr_ptr)[atsp - 1] = Expr::P1(
            Box::new(exprs[atsp - 1].clone()),
            Box::new(exprs[atsp].clone()),
        );
        (*expr_ptr).remove(atsp);
        (*prio_ptr).remove(atsp);
    };
    let build_opc = |atsp: usize| unsafe {
        let mut tied_exprs: Vec<Expr> = Vec::new();
        tied_exprs.push((*expr_ptr)[atsp - 1].clone());
        while (*expr_ptr)[atsp].tryget_priority() == 9 {
            tied_exprs.push((*expr_ptr)[atsp + 1].clone());
            (*expr_ptr).remove(atsp);
            (*prio_ptr).remove(atsp);
            (*expr_ptr).remove(atsp);
            (*prio_ptr).remove(atsp);
        }
        (*expr_ptr)[atsp - 1] = Expr::Chain(Box::new(Expr::Op(Op::Tied)), tied_exprs);
    };
    // 从低到高优先级遍历
    for priority_index in 2..10 {
        let mut index = 0;
        loop {
            if index < (*priority_s).len() {
                let priority = priority_s[index];
                if priority == priority_index {
                    match priority {
                        // Tied ,
                        9 => {
                            build_opc(index);
                            index -= 1;
                        }
                        // Not  !
                        2 => {
                            build_op1(index);
                            index -= 1;
                        }
                        // Other +-*/% > < >= <= && || ^
                        _ => {
                            build_op2(index);
                            index -= 1;
                        }
                    }
                }
                index += 1;
            } else {
                break;
            }
        }
    }
    // 返回
    return Ok((exprs[0].clone(), len));
}
fn get_expr_len(exprs: &Vec<Expr>) -> Result<usize, TerlError> {
    let mut len: usize = 0;
    let mut expect_vul: bool = true;
    let mut last_is_vul = false;
    let mut deep = 0;
    for expr in exprs {
        match expr {
            Expr::Op(_op) => {
                if expect_vul && !matches!(_op, Op::B1l) {
                    return Err(TerlError::ExpectAVul(len));
                }
                match _op {
                    // 单目
                    Op::Not => {
                        expect_vul = true;
                        len += 1;
                        last_is_vul = false;
                        continue;
                    }
                    // 特殊:连续
                    Op::Tied => {
                        if last_is_vul {
                            len += 1;
                            last_is_vul = false;
                            expect_vul = true;
                            continue;
                        } else {
                            return Err(TerlError::ExpectAVul(len));
                        }
                    }
                    Op::B1l => {
                        if last_is_vul || !expect_vul {
                            return Err(TerlError::ExpectASymbol(len));
                        } else {
                            len += 1;
                            deep += 1;
                            last_is_vul = false;
                            expect_vul = true;
                            continue;
                        }
                    }
                    Op::B1r => {
                        if !last_is_vul || expect_vul {
                            return Err(TerlError::ExpectAVul(len));
                        } else {
                            len += 1;
                            deep -= 1;
                            last_is_vul = true;
                            expect_vul = false;
                            continue;
                        }
                    }
                    // 双目
                    _ => {
                        // 正常
                        if last_is_vul || !expect_vul {
                            len += 1;
                            last_is_vul = false;
                            expect_vul = true;
                            continue;
                        }
                        // 不正常
                        else {
                            return Err(TerlError::ExpectAVul(len));
                        }
                    }
                };
            }
            _ => {
                if last_is_vul {
                    return Err(TerlError::ExpectASymbol(len));
                } else {
                    last_is_vul = true;
                    len += 1;
                    expect_vul = false;
                    continue;
                }
            }
        }
    }
    if expect_vul || !last_is_vul {
        return Err(TerlError::ExpectAVul(len));
    }
    if deep != 0 {
        return Err(TerlError::MissBeacket(len));
    }
    return Ok(len);
}
