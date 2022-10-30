use super::basic::*;
use super::builder::FunArg;
use super::op::Op;
use super::prompt::TerlError;
use super::tokens::*;
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
    pub fn is_tied(&self) -> bool {
        matches!(self, Expr::Op(Op::Tied))
    }
}
impl BeTree for Expr {
    fn get_deep(&self) -> usize {
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
impl From<FunArg> for Expr {
    fn from(farg: FunArg) -> Self {
        Self::P2(
            Box::new(Token::Keyword(farg.get_arg_name().get_name()).into()),
            Box::new(Self::Op(Op::Type)),
            Box::new(Token::Keyword(farg.get_arg_type().get_typename().get_name()).into()),
        )
    }
}
/// unsafe!
///
/// 完成一个复杂转变:从一系列的tokens中提取一个有效的expr
///
/// 支持括号
///
/// 主要用来提取表达式/参数列表
pub fn build_expr(tokens: &Vec<Token>) -> Result<(Expr, usize), TerlError> {
    // 准备
    let tokens = tokens.clone();
    let mut exprs: Vec<Expr> = Vec::new();
    // let mut priority_s: Vec<usize> = Vec::new();

    // 初始化优先级列表&&表达式列表
    for token in tokens {
        let expr: Expr = token.into();
        exprs.push(expr)
    }
    // 截取合适长度
    let len = get_expr_len(&exprs)?;
    let exprs: Vec<Expr> = exprs[..len].to_vec();
    let ret = unsafe { collect_expr(&exprs) };
    return Ok((ret, len));
}
fn get_expr_len(exprs: &Vec<Expr>) -> Result<usize, TerlError> {
    let mut len: usize = 0;
    let mut expect_vul: bool = true;
    let mut last_is_vul = false;
    let mut deep = 0;
    for expr in exprs {
        match expr {
            Expr::Op(_op) => {
                if expect_vul && !(matches!(_op, Op::B1l) || matches!(_op, Op::B1r)) {
                    return Err(TerlError::ExpectVul(len));
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
                            return Err(TerlError::ExpectVul(len));
                        }
                    }
                    Op::B1l => {
                        if last_is_vul || !expect_vul {
                            return Err(TerlError::ExpectSymbol(len));
                        } else {
                            len += 1;
                            deep += 1;
                            last_is_vul = false;
                            expect_vul = true;
                            continue;
                        }
                    }
                    Op::B1r => {
                        // if !last_is_vul || expect_vul {
                        //     return Err(TerlError::ExpectAVul(len));
                        // } else {
                        len += 1;
                        deep -= 1;
                        last_is_vul = true;
                        expect_vul = false;
                        continue;
                        // }
                    }
                    // 双目
                    _ => {
                        // 正常(?)
                        if last_is_vul || !expect_vul {
                            // 非运算符
                            if _op.priority() != 0 {
                                len += 1;
                                last_is_vul = false;
                                expect_vul = true;
                                continue;
                            } else {
                                break;
                            }
                        }
                        // 不正常
                        else {
                            return Err(TerlError::ExpectVul(len));
                        }
                    }
                };
            }
            _ => {
                if last_is_vul {
                    return Err(TerlError::ExpectSymbol(len));
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
        return Err(TerlError::ExpectVul(len));
    }
    if deep != 0 {
        return Err(TerlError::MissBeacket(len));
    }
    return Ok(len);
}
///这是一个不稳定的函数,没有考虑错误
///
///这是build_expr()的子函数
///
/// 不应该在除此之外任何地方调用这个函数
///
/// 否则可能会有很多错误
unsafe fn collect_expr(exprs: &Vec<Expr>) -> Expr {
    let mut priority_s = Vec::new();
    for expr in exprs {
        priority_s.push(match expr {
            Expr::Op(_op) => _op.priority(),
            _ => 0,
        });
    }
    // 解决该死的所有权
    let mut exprs = exprs.clone();
    let prio_ptr = &mut priority_s as *mut Vec<usize>;
    let expr_ptr = &mut exprs as *mut Vec<Expr>;
    // 闭包:提取表达式
    let build_op2 = |atsp: usize| {
        (*expr_ptr)[atsp - 1] = Expr::P2(
            Box::new((*expr_ptr)[atsp - 1].clone()),
            Box::new((*expr_ptr)[atsp].clone()),
            Box::new((*expr_ptr)[atsp + 1].clone()),
        );
        (*expr_ptr).remove(atsp);
        (*prio_ptr).remove(atsp);
        (*expr_ptr).remove(atsp);
        (*prio_ptr).remove(atsp);
    };
    let build_op1 = |atsp: usize| {
        (*expr_ptr)[atsp - 1] = Expr::P1(
            Box::new((*expr_ptr)[atsp - 1].clone()),
            Box::new((*expr_ptr)[atsp].clone()),
        );
        (*expr_ptr).remove(atsp);
        (*prio_ptr).remove(atsp);
    };
    let build_opc = |atsp: usize| {
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
    let mut bra_begin = 0;
    // let mut bra_end = 0;
    // 从低到高优先级遍历
    for priority_index in 1..11 {
        let mut index = 0;
        let mut bra_deep = 0;
        if crate::DEBUD_CORE_EXPR_BULDER {
            println!("Begin looping {}", priority_index);
        }
        while index < (*priority_s).len() {
            let priority = priority_s[index];
            if crate::DEBUD_CORE_EXPR_BULDER {
                // 调试函数
                println!("\t{:?}", exprs);
                print!("\t< {}", priority);
                print!(" {} > in ", priority_index);
                println!("\t{:?}\n", priority_s);
            }

            if priority == priority_index {
                match priority {
                    // (
                    1 => {
                        if bra_deep == 0 {
                            if crate::DEBUD_CORE_EXPR_BULDER {
                                println!("\nIN\n");
                            }
                            // DANGERIOUS
                            let more = exprs[index + 1..].to_vec();
                            if more[0].is_right_part() {
                                exprs[index] = Expr::Value(0)
                            } else {
                                exprs[index] = collect_expr(&more);
                            }
                            priority_s[index] = 0;
                            bra_begin = index;
                        }
                        bra_deep += 1;
                    }
                    // )
                    10 => {
                        if bra_deep == 0 {
                            if crate::DEBUD_CORE_EXPR_BULDER {
                                println!("\nOUT\n");
                            }

                            return exprs[0].clone();
                        }
                        //括号刚结束
                        else if bra_deep == 1 {
                            // 大回退
                            while index != bra_begin {
                                index -= 1;
                                (*expr_ptr).remove(bra_begin + 1);
                                (*prio_ptr).remove(bra_begin + 1);
                            }
                        }
                        bra_deep -= 1;
                    }
                    // Tied ,
                    9 => {
                        if bra_deep == 0 {
                            build_opc(index);
                            index -= 1;
                        }
                    }
                    // Not  !
                    2 => {
                        if bra_deep == 0 {
                            build_op1(index);
                            index -= 1;
                        }
                    }
                    // Other +-*/% > < >= <= && || ^
                    _ => {
                        if bra_deep == 0 {
                            build_op2(index);
                            index -= 1;
                        }
                    }
                }
            }
            index += 1;
        }
    }
    return exprs[0].clone();
}
