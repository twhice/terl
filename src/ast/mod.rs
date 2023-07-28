use crate::{
    error::{Error, ErrorKind},
    lexer::{Location, Symbol, Token, TokenVul},
    parser::{CompileMeta, CompileUnit, Parser},
};

const EMPTY_TOKEN: Token = Token {
    location: Location::new(0, 0, 0),
    vul: crate::lexer::TokenVul::Comment,
};

#[derive(Debug, Clone, Copy)]
pub struct Op<'a> {
    token: &'a Token,
    symbol: Symbol,
}

impl CompileUnit for Op<'_> {
    fn build(parser: &mut crate::parser::Parser) -> Result<Self, crate::error::Error> {
        let op = parser
            .next_token()
            .ok_or(ErrorKind::Not("运算符".to_owned()).error())?;
        let TokenVul::Symbol(symbol) = op.vul else {
            return Err(ErrorKind::Not("运算符".to_owned()).to_error(op));
        };
        if symbol.is_ass_op() {
            return Err(ErrorKind::ShouldBe {
                should_be: "普通运算符".to_owned(),
                should_not_be: "赋值运算符".to_owned(),
            }
            .to_error(op));
        }
        Ok(Self { token: op, symbol })
    }

    fn compile(&self) -> Vec<String> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct AssOp<'a> {
    token: &'a Token,
    symbol: Symbol,
}

impl CompileUnit for AssOp<'_> {
    fn build(parser: &mut crate::parser::Parser) -> Result<Self, crate::error::Error> {
        let op = parser
            .next_token()
            .ok_or(ErrorKind::Not("赋值运算符".to_owned()).error())?;
        let TokenVul::Symbol(symbol) = op.vul else {
            return Err(ErrorKind::Not("赋值运算符".to_owned()).to_error(op));
        };
        if symbol.is_op() {
            return Err(ErrorKind::Not("赋值运算符".to_owned()).to_error(op));
        }
        Ok(Self { token: op, symbol })
    }

    fn compile(&self) -> Vec<String> {
        todo!()
    }
}

pub struct VarUse {
    location: Location,
    var_name: String,
}

impl CompileMeta for VarUse {
    fn effect(&self, state: &mut crate::parser::CompileState) {
        todo!()
    }

    fn test(&self, state: &mut crate::parser::CompileState) -> Result<(), Error> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum Expr<'a> {
    Op1 {
        op: Op<'a>,
        rv: Box<Expr<'a>>,
    },
    Op2 {
        op: Op<'a>,
        lv: Box<Expr<'a>>,
        rv: Box<Expr<'a>>,
    },
    Num {
        token: &'a Token,
        vul: f64,
    },
    Var {
        token: &'a Token,
        name: &'a str,
    },
}

impl CompileUnit for Expr<'_> {
    fn build(parser: &mut crate::parser::Parser) -> Result<Self, crate::error::Error> {
        fn atmoic_number(p: &mut Parser) -> Result<Expr<'static>, Error> {
            let number = p
                .next_token()
                .ok_or(ErrorKind::Not("数字".to_owned()).error())?;
            let TokenVul::Number(num) =  number.vul else {
                    return  Err(ErrorKind::Not("数字".to_owned()).to_error(number));
                };
            Ok(Expr::Num {
                token: number,
                vul: num,
            })
        }
        fn atomic_var(p: &mut Parser) -> Result<Expr<'static>, Error> {
            let ident = p
                .next_token()
                .ok_or(ErrorKind::Not("标识符".to_owned()).error())?;
            let TokenVul::Ident(name) =  &ident.vul else {
                    return  Err(ErrorKind::Not("标识符".to_owned()).to_error(ident));
                };
            p.push_meta(VarUse {
                location: ident.location,
                var_name: name.clone(),
            });
            Ok(Expr::Var { token: ident, name })
        }
        fn bracket_expr(p: &mut Parser) -> Result<Expr<'static>, Error> {
            p.match_next_token_vul(ErrorKind::NotOneOf(vec!["(".to_owned()]), |v| {
                matches!(v, TokenVul::Symbol(Symbol::BarcketL))
            })?;

            let expr = Expr::build(p)?;
            p.match_next_token_vul(ErrorKind::NotOneOf(vec![")".to_owned()]), |v| {
                matches!(v, TokenVul::Symbol(Symbol::BarcketR))
            })?;

            Ok(expr)
        }
        fn unary_expr(p: &mut Parser) -> Result<Expr<'static>, Error> {
            let op = Op::build(p)?;
            if !op.symbol.is_unary() {
                return Err(ErrorKind::ShouldBe {
                    should_be: "一元运算符".to_owned(),
                    should_not_be: op.symbol.to_string(),
                }
                .to_error(op.token));
            }

            let atomic_expr = atomic_expr(p)?;
            Ok(Expr::Op1 {
                op,
                rv: Box::new(atomic_expr),
            })
        }
        fn atomic_expr(p: &mut Parser) -> Result<Expr<'static>, Error> {
            p.try_parse(atmoic_number)
                .or_try_parse(atomic_var)
                .or_try_parse(unary_expr)
                .or_try_parse(bracket_expr)
                .finish()
        }

        let mut ops = vec![];
        let mut exprs = vec![atomic_expr(parser)?];

        while let Ok((op, expr)) = parser
            .try_parse(|p| Ok((Op::build(p)?, atomic_expr(p)?)))
            .finish()
        {
            // flod

            ops.push(op);
            exprs.push(expr);
        }

        while !ops.is_empty() {
            // 折叠在location的op
            // 使得exprs[location] op[location] exprs[location+1]组成一个expr
            let location = 'location: {
                let mut location = 0;
                if ops.len() == 1 {
                    break 'location 0;
                }

                while location < ops.len() {
                    let priority = ops[location].symbol.priority();
                    macro_rules! get_priority {
                        ($index : expr) => {
                            ops.get($index)
                                .map(|op| op.symbol.priority())
                                .unwrap_or_default()
                        };
                    }

                    if location < ops.len() - 1
                        && priority >= get_priority!(location + 1)
                        && priority > get_priority!(location + 2)
                    {
                        break 'location location + 1;
                    }

                    if location > 0
                        && priority >= get_priority!(location - 1)
                        && priority > get_priority!(location - 2)
                    {
                        break 'location location - 1;
                    }
                    location += 1;
                }
                location
            };

            exprs[location] = Expr::Op2 {
                op: ops[location],
                lv: Box::new(exprs[location].clone()),
                rv: Box::new(exprs[location + 1].clone()),
            };
            exprs.remove(location + 1);
            ops.remove(location);
        }

        assert!(exprs.len() == 1, "是bug");
        Ok(exprs.pop().unwrap())
    }

    fn compile(&self) -> Vec<String> {
        todo!()
    }
}

pub struct VarDef {
    location: Location,
    var_name: String,
}

impl CompileMeta for VarDef {
    fn effect(&self, state: &mut crate::parser::CompileState) {
        todo!()
    }

    fn test(&self, state: &mut crate::parser::CompileState) -> Result<(), Error> {
        todo!()
    }
}

pub struct VarAss {
    location: Location,
    var_name: String,
}

impl CompileMeta for VarAss {
    fn effect(&self, state: &mut crate::parser::CompileState) {
        todo!()
    }

    fn test(&self, state: &mut crate::parser::CompileState) -> Result<(), Error> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum Bind<'a> {
    NewWithoutInit {
        r#let: &'a Token,
        vars: Vec<String>,
    },
    NewInit {
        r#let: &'a Token,
        vars: Vec<String>,
        ass_op: AssOp<'a>,
        vuls: Vec<Expr<'a>>,
    },
    Ass {
        vars: Vec<String>,
        ass_op: AssOp<'a>,
        vuls: Vec<Expr<'a>>,
    },
}

impl CompileUnit for Bind<'_> {
    fn build(parser: &mut Parser) -> Result<Self, Error> {
        fn get_ident(p: &mut Parser, error: ErrorKind) -> Result<(String, Location), Error> {
            match p.next_token() {
                Some(Token {
                    vul: TokenVul::Ident(name),
                    location,
                }) => Ok((name.to_owned(), *location)),
                Some(token) => Err(error.to_error(token)),

                None => Err(error.error()),
            }
        }

        parser
            .try_parse(|p| {
                let r#let = p.match_next_token_vul(ErrorKind::Not("let".to_string()), |v| {
                    if let TokenVul::Ident(ident) = v {
                        ident == "let"
                    } else {
                        false
                    }
                })?;
                let (var_name1, location1) = get_ident(p, ErrorKind::Not("标识符".to_owned()))?;
                p.push_meta(VarDef {
                    location: location1,
                    var_name: var_name1.clone(),
                });
                let mut vars = vec![var_name1];
                vars.extend(
                    (0..)
                        .map(|_| {
                            p.try_parse(|p| {
                                p.match_next_token_vul(ErrorKind::None, |v| {
                                    matches!(v, &TokenVul::Symbol(Symbol::Split))
                                })?;
                                let (name, location) = get_ident(p, ErrorKind::None)?;

                                p.push_meta(VarDef {
                                    location,
                                    var_name: name.clone(),
                                });
                                Ok(name)
                            })
                            .finish()
                        })
                        .take_while(|r| r.is_ok())
                        .filter_map(|re| re.ok()),
                );

                let mut vars = Some(vars);

                p.try_parse(|p| {
                    let ass_op = AssOp::build(p)?;
                    let mut vuls = vec![];
                    {
                        let vars = vars.as_ref().unwrap();
                        for expr_idx in 0..vars.len() {
                            vuls.push(Expr::build(p)?);
                            if expr_idx != vars.len() - 1 {
                                p.match_next_token_vul(
                                    ErrorKind::NotOneOf(vec![",".to_owned()]),
                                    |v| matches!(v, TokenVul::Symbol(Symbol::Split)),
                                )?;
                            }
                        }
                    }
                    Ok(Self::Ass {
                        vars: vars.take().unwrap(),
                        ass_op,
                        vuls,
                    })
                })
                .or_try_parse(|_p| {
                    Ok(Self::NewWithoutInit {
                        r#let,
                        vars: vars.take().unwrap(),
                    })
                })
                .finish()
            })
            .or_try_parse(|p| {
                let (var_name1, location1) = get_ident(p, ErrorKind::Not("标识符".to_owned()))?;
                p.push_meta(VarAss {
                    location: location1,
                    var_name: var_name1.clone(),
                });
                let mut vars = vec![var_name1];
                vars.extend(
                    (0..)
                        .map(|_| {
                            p.try_parse(|p| {
                                p.match_next_token_vul(ErrorKind::None, |v| {
                                    matches!(v, &TokenVul::Symbol(Symbol::Split))
                                })?;
                                let (name, location) = get_ident(p, ErrorKind::None)?;

                                p.push_meta(VarAss {
                                    location,
                                    var_name: name.clone(),
                                });
                                Ok(name)
                            })
                            .finish()
                        })
                        .take_while(|r| r.is_ok())
                        .filter_map(|re| re.ok()),
                );

                let ass_op = AssOp::build(p)?;

                let mut vuls = vec![];
                for expr_idx in 0..vars.len() {
                    vuls.push(Expr::build(p)?);
                    if expr_idx != vars.len() - 1 {
                        p.match_next_token_vul(ErrorKind::NotOneOf(vec![",".to_owned()]), |v| {
                            matches!(v, TokenVul::Symbol(Symbol::Split))
                        })?;
                    }
                }

                Ok(Self::Ass { vars, ass_op, vuls })
            })
            .finish()
    }

    fn compile(&self) -> Vec<String> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;

    use super::*;

    #[test]
    fn test_name() -> std::result::Result<(), Error> {
        // 再不写test debug会累死
        let src = "let x = 1 + 2 + 3";
        let tokens = Lexer::new(src).toekns();
        let let_stmt = Parser::new(&tokens).try_parse(Bind::build).finish();
        dbg!(let_stmt)?;
        Ok(())
    }
}
