use super::basic::*;
use super::context::Context;
use super::expr::{self, Expr};
use super::op::Op;
use super::prompt::TerlError;
use super::tree::{BeTree, Tree};
use super::{context::Statement, tokens::*};
/*
name = expr       |by op
name = (args):type|assi =
name   (args):type|br1l (
name : es         |type :
name : type = expr|

if condition statement
else statement
return expr
import expr
*/
#[derive(Debug, Clone)]
pub struct FunArg {
    arg_name: Name,
    arg_type: Type,
}
impl FunArg {
    fn new(arg_name: Name, arg_type: Type) -> Self {
        Self { arg_name, arg_type }
    }

    pub fn get_arg_name(&self) -> Name {
        self.arg_name.clone()
    }

    pub fn get_arg_type(&self) -> Type {
        self.arg_type.clone()
    }
}
fn is_konwn_statement(str: &Vec<char>) -> Option<usize> {
    for i in 0..TOKENS.len() {
        if str == &TOKENS[i].chars().collect::<Vec<char>>() {
            return Some(i);
        }
    }
    None
}
fn collect_build_defv(mut fulltokens: Vec<FullToken>, name: Name, mut sta: Statement) -> Statement {
    // 判空
    if fulltokens.len() == 1 {
        fulltokens[0].error(&TerlError::ExpectExpr(0));
    } else {
        fulltokens.remove(0);
    }
    // 转换
    let mut tokens = Vec::new();
    for fulltoken in &fulltokens {
        tokens.push(fulltoken.get_token());
    }
    match expr::build_expr(&tokens) {
        // 正常 查看盈余情况
        Ok((_expr, _endex)) => {
            // 正常
            if _endex == fulltokens.len() {
                *sta.context() = Context::DefVul(name, _expr, "auto".into());
                return sta;
            } else {
                fulltokens[_endex - 1].error(&TerlError::ExpectNothing(0));
            }
        }
        // 错误 错误处理
        Err(_err) => match _err {
            TerlError::ExpectVul(_index)
            | TerlError::ExpectSymbol(_index)
            | TerlError::MissBeacket(_index) => fulltokens[_index - 1].error(&_err),
            // TerlError::ExpectName(_) => todo!(),
            // TerlError::ExpectTypeName(_) => todo!(),
            // TerlError::ExpectOneOf(_) => todo!(),
            // TerlError::ExpectNothing(_) => todo!(),
            _ => todo!(),
        },
    }
    todo!()
}
fn collect_fun_args_type(fulltokens: &Vec<FullToken>) -> (Expr, Type) {
    // (args) + :type
    // 独立规则
    // 确定括号范围

    // 参数表
    let mut args: Vec<FunArg> = Vec::new();
    //step:  (name:type)
    //  0 want (
    //  0 want ,
    //  1 want name
    //  2 want :
    //  3 want type (keyword / fun_type)

    let mut step = 0;

    let mut had_begin = false;
    let mut had_closed = false;
    let mut arg_name = Name::new(Vec::new());
    let mut arg_type = Type::new(Name::new(Vec::new()));
    let mut endex = 0;
    for index in 0..fulltokens.len() {
        // 对函数类型的支持放缓
        let token: Token = fulltokens[index].get_token();
        let expr: Expr = (&token).to_owned().into();
        match step {
            1 => {
                if let Some(_name) = (&token).get_in_keyword() {
                    had_begin = true;
                    arg_name = _name.clone().into();
                    step += 1;
                    continue;
                } else if had_begin == false {
                    had_closed = true;
                    endex = index;
                    break;
                }
                fulltokens[index].error(&TerlError::ExpectName(0));
            }
            2 => {
                if let Some(_op) = (&token).get_in_symbol() {
                    if let Op::Type = _op {
                        had_begin = true;
                        step += 1;
                    } else {
                        fulltokens[index].error(&TerlError::ExpectOneOf(vec![':']));
                    }
                } else {
                    fulltokens[index - 1].error(&TerlError::ExpectSymbol(0));
                }
            }
            3 => {
                if let Some(_name) = (&token).get_in_keyword() {
                    had_begin = true;
                    let _name: Name = _name.into();
                    arg_type = _name.into();
                    step = 0;
                } else {
                    fulltokens[index - 1].error(&TerlError::ExpectTypeName(0));
                }
            }
            4 => {}
            _ => {
                if expr.is_left_part() && !had_begin {
                    step = 1;
                    // had_begin = true;
                } else if expr.is_right_part() {
                    had_closed = true;
                    endex = index;
                    break;
                } else if expr.is_tied() {
                    if had_begin {
                        args.push(FunArg::new(arg_name.clone(), arg_type.clone()));
                        step = 1;
                    } else {
                        fulltokens[index - 1].error(&TerlError::ExpectName(0));
                    }
                } else {
                    fulltokens[index - 1].error(&TerlError::MissBeacket(0));
                }
            }
        }
    }
    // 检查括号完整性
    if !had_closed {
        fulltokens[endex].error(&TerlError::MissBeacket(0))
    }
    // 返回值类型
    let mut ret_type: Type = "void".into();
    let mut ret_exprs = Vec::new();
    // 凑活下
    for fun_arg in args {
        ret_exprs.push(Expr::from(fun_arg));
    }
    // 不附加类型
    if fulltokens.len() == endex + 1 {
        ret_type = "void".into();
        return (
            Expr::Chain(Box::new(Expr::Op(Op::Tied)), ret_exprs),
            ret_type,
        );
    } else {
        // 取op
        if let Some(_fulltoken) = fulltokens.get(endex + 1) {
            let __op = _fulltoken.get_token().get_in_symbol();
            if let Some(_op) = __op {
                if let Op::Type = _op {
                    // 取typename
                    if let Some(_fulltoken) = fulltokens.get(endex + 2) {
                        if let Some(_typename) = _fulltoken.get_token().get_in_keyword() {
                            ret_type = Name::from(_typename).into();
                        }
                    }
                    return (
                        Expr::Chain(Box::new(Expr::Op(Op::Tied)), ret_exprs),
                        ret_type,
                    );
                }
            }
        }
        fulltokens[endex].error(&TerlError::ExpectOneOf(vec![':']))
    }
    // 应仅剩余两个部分：op_type&type

    todo!()
}
fn create_new(mut fulltokens: Vec<FullToken>, name: Name, is_pub: bool, deep: usize) -> Statement {
    if fulltokens.len() == 1 {
        fulltokens[0].error(&TerlError::ExpectSymbol(0));
    }
    let mut sta = Statement::new(deep);
    sta.set_pos(fulltokens[0].get_pos());
    sta.set_is_pub(is_pub);
    if let Some(_op) = fulltokens[1].get_token().get_in_symbol() {
        match _op {
            // name   (args):type
            Op::B1l => {
                fulltokens.remove(0); // name
                let (args, typename) = collect_fun_args_type(&fulltokens);
                *sta.context() = Context::DefFun(name, args, Tree::Node(Vec::new()), typename);
                return sta;
            }
            // name = expr
            Op::Assign => {
                //移除等号
                //收集表达式
                fulltokens.remove(0); // name
                                      // fulltokens.remove(0); // = 方便报错
                return collect_build_defv(fulltokens, name, sta);
            }
            // name : type = expr
            // name : es
            Op::Type => match fulltokens.len() {
                // len >= 2

                // name:
                2 => {
                    *sta.context() = Context::DefStruct(name, Tree::Node(Vec::new()), Vec::new());
                    return sta;
                }
                // name : ef1
                _ => {
                    if let Some(_typename) = fulltokens[2].get_token().get_in_keyword() {
                        if fulltokens.len() == 3 {
                            *sta.context() = Context::DefStruct(
                                name,
                                Tree::Node(Vec::new()),
                                vec![_typename.into()],
                            );
                            return sta;
                        }
                        // len >= 4
                        // name : type_name = expr
                        // name : ef1 ef2 ef...
                        if let Some(_op) = fulltokens[3].get_token().get_in_symbol() {
                            if let Op::Assign = _op {
                                fulltokens.remove(0);
                                fulltokens.remove(0);
                                fulltokens.remove(0);
                                // fulltokens.remove(0); 方便报错
                                return collect_build_defv(fulltokens, name, sta);
                            }
                        }
                        let mut efs = Vec::new();
                        for index in 3..fulltokens.len() {
                            if let Some(_typename) = fulltokens[index].get_token().get_in_keyword()
                            {
                                efs.push(Type::from(_typename))
                            } else {
                                fulltokens[index - 1].error(&TerlError::ExpectTypeName(0));
                            }
                        }
                        *sta.context() = Context::DefStruct(name, Tree::Node(Vec::new()), efs);
                        return sta;
                        // fulltokens[2].error(&TerlError::ExpectOneOf(vec!['=', ' ']))
                    } else {
                        fulltokens[1].error(&TerlError::ExpectTypeName(0))
                    }
                }
            },
            _ => fulltokens[1].error(&TerlError::ExpectOneOf(vec!['=', '(', ':'])),
        }
    } else {
        fulltokens[0].error(&TerlError::ExpectSymbol(0));
    }
    todo!()
}
pub fn build_statement(fulltokens: Vec<FullToken>, deep: usize) -> Statement {
    if fulltokens.is_empty() {
        let mut ret = Statement::new(deep);
        *ret.context() = Context::Empty;
        return ret;
    }
    if let Some(_name) = fulltokens[0].get_token().get_in_keyword() {
        let is_pub = false;
        if let Some(_index) = is_konwn_statement(&_name) {
            todo!("Match index")
        } // else
        let name = Name::new(_name);
        // fulltokens.remove(0);
        return create_new(fulltokens, name, is_pub, deep);
    } else {
        fulltokens[0].error(&TerlError::ExpectName(0));
    }
    todo!()
}
