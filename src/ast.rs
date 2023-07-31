use std::fmt::Display;

use crate::{
    error::{Error, ErrorKind},
    lexer::{Symbol, Token},
    parser::Parser,
    syn::CompileUnit,
};

pub trait ParserUnit: Sized {
    fn parse(p: &mut Parser) -> Result<Self, Error>;
}

#[derive(Debug, Clone, Copy)]
pub enum Op<'a> {
    AssOp { token: &'a Token, symbol: Symbol },
    Op { token: &'a Token, symbol: Symbol },
}

impl std::ops::Not for Op<'_> {
    type Output = Result<Self, Self>;

    fn not(self) -> Self::Output {
        match &self {
            Op::AssOp { .. } => Err(self),
            Op::Op { symbol, token } => match !*symbol {
                Some(symbol) => Ok(Self::Op { token, symbol }),
                None => Err(self),
            },
        }
    }
}

impl Op<'_> {
    pub fn priority(&self) -> Option<usize> {
        match self {
            Op::AssOp { .. } => None,
            Op::Op { symbol, .. } => Some(symbol.priority()),
        }
    }

    pub fn token(&self) -> &Token {
        match self {
            Op::AssOp { token, .. } | Op::Op { token, .. } => token,
        }
    }

    pub fn symbol(&self) -> Symbol {
        match self {
            Op::AssOp { symbol, .. } | Op::Op { symbol, .. } => *symbol,
        }
    }
}

impl Display for Op<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::AssOp { symbol, .. } | Op::Op { symbol, .. } => {
                write!(f, "{symbol}")
            }
        }
    }
}

impl ParserUnit for Op<'_> {
    fn parse(p: &mut Parser) -> Result<Self, Error> {
        let e = || ErrorKind::not_one_of(&["运算符", "赋值运算符"]);

        let (token, &symbol) = p.get_symbol(e)?;
        if symbol.is_ass_op() {
            Ok(Self::AssOp { token, symbol })
        } else if symbol.is_op() {
            Ok(Self::Op { token, symbol })
        } else {
            Err(p.l_err(e, token))
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr<'a> {
    Var {
        token: &'a Token,
        name: &'a str,
    },
    Num {
        token: &'a Token,
        vul: f64,
    },
    Str {
        token: &'a Token,
        vul: &'a str,
    },
    Op1 {
        op: Op<'a>,
        rv: Box<Expr<'a>>,
    },
    Op2 {
        lv: Box<Expr<'a>>,
        op: Op<'a>,
        rv: Box<Expr<'a>>,
    },
    FnCall {
        fn_name_token: &'a Token,
        fn_name: &'a str,
        args: Vec<Expr<'a>>,
    },
}

impl std::ops::Neg for &mut Expr<'_> {
    type Output = Result<Self, ()>;

    fn neg(self) -> Self::Output {
        match self {
            Expr::Num { vul, .. } => {
                *vul = -*vul;
                Ok(self)
            }
            Expr::Op1 { op, rv } if matches!(op.symbol(), Symbol::Not) => Ok(rv),
            _ => Err(()),
        }
    }
}

impl std::ops::Not for &mut Expr<'_> {
    type Output = Result<Self, ()>;

    fn not(self) -> Self::Output {
        match self {
            Expr::Num { vul, .. } => {
                *vul = (*vul == 0.0) as u8 as f64;
                Ok(self)
            }
            Expr::Op1 { op, rv } => {
                if matches!(op.symbol(), Symbol::Not) {
                    Ok(rv)
                } else {
                    Err(())
                }
            }
            Expr::Op2 { op, .. } if (!*op).is_ok() => {
                *op = (!*op).unwrap();
                Ok(self)
            }
            _ => Err(()),
        }
    }
}

fn get_splitd_units_in_bracket<F, R>(p: &mut Parser, parser: F) -> Result<Vec<R>, Error>
where
    F: FnOnce(&mut Parser) -> Result<R, Error> + Copy + 'static,
{
    p.match_symbol(&Symbol::BarcketL, ErrorKind::none)?;
    let mut units = vec![];
    loop {
        // 先尝试获取一个单元
        if let Ok(unit) = p.try_parse(parser).finish(ErrorKind::none) {
            units.push(unit);
            // 如果紧接着是一个',' 表示之后可能还有单元
            if p.try_parse(|p| p.match_symbol(&Symbol::BarcketL, ErrorKind::none))
                .finish(ErrorKind::none)
                .is_ok()
            {
                continue;
            }
        }

        // 并没有成功捕获到单元
        // 有可能是)
        // 开始匹配 或者退出
        p.match_symbol(&Symbol::BarcketR, || ErrorKind::not(")"))?;

        break;
    }
    Ok(units)
}

fn parse_fn_call(p: &mut Parser) -> Result<Expr<'static>, Error> {
    let (fn_name_token, fn_name) = p.get_ident(ErrorKind::none)?;

    let args = get_splitd_units_in_bracket(p, Expr::parse)?;

    // 部分重写

    Ok(Expr::FnCall {
        fn_name_token,
        fn_name,
        args,
    })
}

fn parse_fn_call_stmt(p: &mut Parser) -> Result<Expr<'static>, Error> {
    let r = parse_fn_call(p)?;
    p.match_endlines()?;
    Ok(r)
}

impl ParserUnit for Expr<'_> {
    fn parse(p: &mut Parser) -> Result<Self, Error> {
        fn atomic_var(p: &mut Parser) -> Result<Expr<'static>, Error> {
            let (token, name) = p.get_ident(ErrorKind::none)?;

            Ok(Expr::Var { token, name })
        }
        fn atomic_num(p: &mut Parser) -> Result<Expr<'static>, Error> {
            let (token, &vul) = p.get_number(ErrorKind::none)?;
            Ok(Expr::Num { token, vul })
        }
        fn atomic_string(p: &mut Parser) -> Result<Expr<'static>, Error> {
            let (token, vul) = p.get_string(ErrorKind::none)?;
            Ok(Expr::Str { token, vul })
        }

        fn bracket(p: &mut Parser) -> Result<Expr<'static>, Error> {
            p.match_symbol(&Symbol::BarcketL, ErrorKind::none)?;
            let expr = Expr::parse(p)?;
            p.match_symbol(&Symbol::BarcketR, || ErrorKind::not(")"))?;
            Ok(expr)
        }
        fn unary_expr(p: &mut Parser) -> Result<Expr<'static>, Error> {
            let op = p.try_parse(Op::parse).finish(ErrorKind::none)?;
            match &op {
                Op::Op { symbol, .. } if symbol.is_unary() => {
                    let rv = Box::new(atomic_expr(p)?);
                    Ok(Expr::Op1 { op, rv })
                }
                Op::AssOp { token, .. } | Op::Op { token, .. } => {
                    Err(p.l_err(ErrorKind::none, token))
                }
            }
        }
        fn generate_error() -> ErrorKind {
            ErrorKind::not_one_of(&["标识符", "数字", "字符串", "函数调用", "括号", "一元表达式"])
        }
        fn note() -> String {
            format!(
                "{}\n\t{}\n\t{}\n\t{}\n\t{}\n\t{}\n",
                "普通表达式：以下语法之一",
                "数字",
                "变量",
                "一元运算符 表达式",
                "函数名(参数,...)",
                "(表达式)",
            )
        }

        fn atomic_expr(p: &mut Parser) -> Result<Expr<'static>, Error> {
            p.try_parse(parse_fn_call)
                .or_try_parse(atomic_num)
                .or_try_parse(atomic_var)
                .or_try_parse(atomic_string)
                .or_try_parse(unary_expr)
                .or_try_parse(bracket)
                .with_note(note)
                .finish(generate_error)
        }

        // return fn_call(p);

        // 第一个表达式不计错误
        let mut exprs = vec![atomic_expr(p)?];
        let mut ops = vec![];
        while let Ok(op) = p.try_parse(Op::parse).finish(ErrorKind::none) {
            if let Op::AssOp { token, .. } = &op {
                return Err(p.l_err(|| ErrorKind::not("普通运算符"), token));
            }
            ops.push(op);
            exprs.push(atomic_expr(p)?);
        }

        macro_rules! priority {
            ($idx : expr) => {
                ops.get($idx)
                    .and_then(|op| op.priority())
                    .unwrap_or_default()
            };
        }

        while !ops.is_empty() {
            let index = 'calc: {
                let mut index = 0;
                if ops.len() == 1 {
                    break 'calc 0;
                }
                while index < ops.len() {
                    let priority = priority!(index);
                    if priority >= priority!(index + 1)
                        && (index >= ops.len() - 1 || priority >= priority!(index + 2))
                    {
                        break 'calc index;
                    }
                    if index >= 1
                        && priority >= priority!(index - 1)
                        && (index < 2 || priority >= priority!(index - 2))
                    {
                        break 'calc index - 1;
                    }
                    index += 1;
                }
                index
            };
            exprs[index] = Expr::Op2 {
                op: ops[index],
                lv: Box::new(exprs[index].clone()),
                rv: Box::new(exprs[index + 1].clone()),
            };
            exprs.remove(index + 1);
            ops.remove(index);
        }
        assert!(exprs.len() == 1);
        Ok(exprs.pop().unwrap())
    }
}

impl Display for Expr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Var { name, .. } => write!(f, "{name}"),
            Expr::Num { vul, .. } => write!(f, "{vul}"),
            Expr::Str { vul, .. } => write!(f, "\"{vul}\""),
            Expr::Op1 { op, rv } => write!(f, "{op}{rv}"),
            Expr::Op2 { lv, op, rv } => write!(f, "({lv}{op}{rv})"),
            Expr::FnCall { fn_name, args, .. } => {
                write!(f, "{fn_name}(")?;
                for arg in args {
                    write!(f, "{arg},")?;
                }
                write!(f, ")")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Bind<'a> {
    Define {
        r#let: &'a Token,
        var_tokens: Vec<&'a Token>,
        vars: Vec<&'a str>,
    },
    Init {
        r#let: &'a Token,
        var_tokens: Vec<&'a Token>,
        vars: Vec<&'a str>,
        ass_op: Op<'a>,
        vuls: Vec<Expr<'a>>,
    },
    Ass {
        var_tokens: Vec<&'a Token>,
        vars: Vec<&'a str>,
        ass_op: Op<'a>,
        vuls: Vec<Expr<'a>>,
    },
}

fn get_splitd_idents_tokens(
    p: &mut Parser,
) -> Result<(Vec<&'static Token>, Vec<&'static str>), Error> {
    let (var_token, var) = p.get_ident(|| ErrorKind::not("标识符"))?;
    let var_tokens = vec![var_token];
    let vars = vec![var.as_str()];
    get_splitted_idents(p, var_tokens, vars)
}

fn get_splitted_idents(
    p: &mut Parser,
    mut var_tokens: Vec<&'static Token>,
    mut vars: Vec<&'static str>,
) -> Result<(Vec<&'static Token>, Vec<&'static str>), Error> {
    while p
        .try_parse(|p| p.match_symbol(&Symbol::Split, ErrorKind::none))
        .finish(ErrorKind::none)
        .is_ok()
    {
        let (var_token, var) = p.get_ident(|| ErrorKind::not("标识符"))?;
        var_tokens.push(var_token);
        vars.push(var.as_str());
    }
    Ok((var_tokens, vars))
}

impl ParserUnit for Bind<'_> {
    fn parse(p: &mut Parser) -> Result<Self, Error> {
        fn get_splitted_exprs(p: &mut Parser) -> Result<Vec<Expr<'static>>, Error> {
            let mut exprs = vec![Expr::parse(p)?];
            while p
                .try_parse(|p| p.match_symbol(&Symbol::Split, ErrorKind::none))
                .finish(ErrorKind::none)
                .is_ok()
            {
                exprs.push(Expr::parse(p)?);
            }
            Ok(exprs)
        }

        fn with_let(p: &mut Parser) -> Result<Bind<'static>, Error> {
            let r#let = p.match_ident(&"let".to_string(), ErrorKind::none)?;
            let (var_tokens, vars) = get_splitd_idents_tokens(p)?;
            // 先匹配换行
            match p
                .try_parse(|p| p.match_endlines())
                .finish(|| ErrorKind::not_one_of(&["赋值运算符", "换行"]))
            {
                Ok(..) => Ok(Bind::Define {
                    r#let,
                    var_tokens,
                    vars,
                }),
                Err(..) => {
                    let ass_op = Op::parse(p)?;
                    match &ass_op {
                        Op::AssOp { .. } => {
                            let vuls = get_splitted_exprs(p)?;
                            // 赋值运算符两边的元素的数量不一致
                            if vuls.len() != vars.len() {
                                return Err(ErrorKind::CantAss.generate_error(ass_op.token()));
                            }
                            p.match_endlines()?;

                            Ok(Bind::Init {
                                r#let,
                                var_tokens,
                                vars,
                                ass_op,
                                vuls,
                            })
                        }
                        Op::Op { token, .. } => {
                            Err(ErrorKind::not("赋值运算符").generate_error(token))
                        }
                    }
                }
            }
        }

        fn without_let(p: &mut Parser) -> Result<Bind<'static>, Error> {
            // 这个情况有些复杂
            // x,y,z = ...
            // x = ...
            // x()
            // 都可能判定上 很可能会产生错误的报错

            // 通过检查 / 交换Bind和FnCallStmt的位置 解决
            let (var_token, var) = p.get_ident(ErrorKind::none)?;
            let var_tokens = vec![var_token];
            let vars = vec![var.as_str()];
            let (var_tokens, vars) = get_splitted_idents(p, var_tokens, vars)?;

            // let ass_op = match Op::parse(p) {
            //     Ok(ass_op) => ass_op,
            //     Err(e) => {
            //         if vars.len() == 1 {
            //             // 可能是 x() 这样的情况
            //             return Err(Error::empty());
            //         } else {
            //             return Err(e);
            //         }
            //     }
            // };
            let ass_op = Op::parse(p)?;
            match &ass_op {
                Op::AssOp { .. } => {
                    let vuls = get_splitted_exprs(p)?;
                    // 赋值运算符两边的元素的数量不一致
                    if vuls.len() != vars.len() {
                        return Err(ErrorKind::CantAss.generate_error(ass_op.token()));
                    }
                    // 先匹配换行
                    p.match_endlines()?;

                    Ok(Bind::Ass {
                        var_tokens,
                        vars,
                        ass_op,
                        vuls,
                    })
                }
                Op::Op { token, .. } => Err(ErrorKind::not("赋值运算符").generate_error(token)),
            }
        }

        fn note() -> String {
            format!(
                "{}\n\t{}\n\t{}\n\t{}\n",
                "Bind语句：以下语法之一",
                "var1, ..varn = vul1, ..vuln",
                "let var1, ..varn",
                "let var1, ..varn = vul1, ..vuln",
            )
        }

        p.try_parse(with_let)
            .or_try_parse(without_let)
            .with_note(note)
            .finish(ErrorKind::none)
    }
}

fn vector_to_string<D: ToString>(slice: &[D]) -> String {
    let mut buffer = String::new();
    for i in 0..slice.len() {
        buffer += &slice[i].to_string();
        if i < slice.len() - 1 {
            buffer += ","
        }
    }
    buffer
}

impl Display for Bind<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Bind::Define { vars, .. } => write!(f, "let {}", vector_to_string(vars)),
            Bind::Init {
                vars, ass_op, vuls, ..
            } => write!(
                f,
                "let {} {} {}",
                vector_to_string(vars),
                ass_op,
                vector_to_string(vuls)
            ),
            Bind::Ass {
                vars, ass_op, vuls, ..
            } => write!(
                f,
                "{} {} {}",
                vector_to_string(vars),
                ass_op,
                vector_to_string(vuls)
            ),
        }
    }
}

#[derive(Debug)]
pub struct Block<'a> {
    // pub space: usize,
    pub start: &'a Token,
    pub stmts: Vec<Box<dyn CompileUnit>>,
    pub end: &'a Token,
}

impl ParserUnit for Block<'_> {
    fn parse(p: &mut Parser) -> Result<Self, Error> {
        // static SPACE_INDEX: atomic::AtomicUsize = atomic::AtomicUsize::new(1);
        let start = p.match_symbol(&Symbol::SpaceL, ErrorKind::none)?;
        // let index = SPACE_INDEX.fetch_add(1, atomic::Ordering::Relaxed);

        let stmts = parse_compile_units(p)?;
        let end = p.match_symbol(&Symbol::SpaceR, ErrorKind::unexpect)?;

        Ok(Self {
            // space: index,
            start,
            stmts,
            end,
        })
    }
}

impl Display for Block<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for stmt in &self.stmts {
            writeln!(f, "{stmt}")?;
        }
        write!(f, "}}")
    }
}

#[derive(Debug)]
pub enum ControlFlow<'a> {
    If {
        r#if: &'a Token,
        conditions: Vec<Expr<'a>>,
        blocks: Vec<Block<'a>>,
        r#else: Option<&'a Token>,
        else_block: Option<Block<'a>>,
    },
    While {
        r#while: &'a Token,
        condition: Expr<'a>,
        block: Block<'a>,
    },
}

impl ParserUnit for ControlFlow<'_> {
    fn parse(p: &mut Parser) -> Result<Self, Error> {
        p.try_parse(|p| {
            let r#if = p.match_ident(&"if".to_string(), || ErrorKind::None)?;
            // 为了显示note
            p.try_parse(|p| {
                let mut conditions = vec![Expr::parse(p)?];
                let mut blocks = vec![Block::parse(p)?];

                let else_if = |p: &mut Parser| {
                    p.match_ident(&"else".to_owned(), ErrorKind::none)?;
                    p.match_ident(&"if".to_owned(), ErrorKind::none)
                };

                // let elif_error = || ErrorKind::not_one_of(&["else if", "elif"]);

                while p
                    .try_parse(else_if)
                    .or_try_parse(|p| p.match_ident(&"elif".to_owned(), ErrorKind::none))
                    .finish(ErrorKind::none)
                    .is_ok()
                {
                    let condition = Expr::parse(p)?;
                    let block = Block::parse(p)?;
                    conditions.push(condition);
                    blocks.push(block);
                }

                if let Ok(r#else) = p
                    .try_parse(|p| p.match_ident(&"else".to_owned(), ErrorKind::none))
                    .finish(ErrorKind::none)
                {
                    let else_block = Block::parse(p)?;
                    p.match_endlines()?;
                    Ok(Self::If {
                        r#if,
                        conditions,
                        blocks,
                        r#else: Some(r#else),
                        else_block: Some(else_block),
                    })
                } else {
                    p.match_endlines()?;
                    Ok(Self::If {
                        r#if,
                        conditions,
                        blocks,
                        r#else: None,
                        else_block: None,
                    })
                }
            })
            .with_note(|| {
                format!(
                    "{}\n\t\t{}\n\t{}\n\t{}\n",
                    "If用法:",
                    " if 条件 代码块",
                    "零或多条 else if/elif 条件 代码块",
                    "零或一条 else 代码块"
                )
            })
            .finish(ErrorKind::none)
        })
        .or_try_parse(|p| {
            let r#while = p.match_ident(&"while".to_string(), || ErrorKind::None)?;
            p.try_parse(|p| {
                let condition = Expr::parse(p)?;
                let block = Block::parse(p)?;
                p.match_endlines()?;
                Ok(Self::While {
                    r#while,
                    condition,
                    block,
                })
            })
            .with_note(|| format!("{}\n\t{}\n", "while用法：", "while 条件 代码块"))
            .finish(ErrorKind::none)
        })
        .finish(ErrorKind::none)
    }
}

impl Display for ControlFlow<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlFlow::If {
                conditions,
                blocks,
                r#else,
                else_block,
                ..
            } => {
                for i in 0..conditions.len() {
                    if i != 0 {
                        write!(f, "elif")?;
                    } else {
                        write!(f, "if")?;
                    }
                    write!(f, " {} {}", conditions[i], blocks[i])?;
                }
                if r#else.is_some() {
                    write!(f, "else {}", else_block.as_ref().unwrap())?;
                }
                Ok(())
            }
            ControlFlow::While {
                condition, block, ..
            } => write!(f, "while {} {}", condition, block),
        }
    }
}

#[derive(Debug)]
pub struct FnDef<'a> {
    r#fn: &'a Token,
    fn_name_token: &'a Token,
    fn_name: &'a str,
    parm_tokens: Vec<&'a Token>,
    parms: Vec<&'a str>,
    block: Block<'a>,
}

impl ParserUnit for FnDef<'_> {
    fn parse(p: &mut Parser) -> Result<Self, Error> {
        p.try_parse(|p| {
            let r#fn = p.match_ident(&"fn".to_string(), ErrorKind::none)?;
            let (fn_name_token, fn_name) = p.get_ident(|| ErrorKind::not("标识符"))?;

            let get_parm_token = |p: &mut Parser| {
                let (token, parm) = p.get_ident(ErrorKind::none)?;
                Ok((token, parm.as_str()))
            };

            let (parm_tokens, parms): (Vec<_>, Vec<_>) =
                get_splitd_units_in_bracket(p, get_parm_token)?
                    .into_iter()
                    .unzip();

            let block = Block::parse(p)?;
            p.match_endlines()?;

            // p.record(records::FnRecord::define(
            //     fn_name,
            //     fn_name_token.location,
            //     parms.len(),
            // ));

            Ok(Self {
                r#fn,
                fn_name_token,
                fn_name,
                parm_tokens,
                parms,
                block,
            })
        })
        .with_note(|| format!("{}\n\t\t{}", "Fn用法：", "fn 标识符(标识符,...) 代码块"))
        .finish(ErrorKind::none)
    }
}

impl Display for FnDef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn {}({})", self.fn_name, vector_to_string(&self.parms))?;
        write!(f, "{}", self.block)
    }
}

fn trim_endlines(p: &mut Parser) -> Result<crate::syn::EmptyStmt, Error> {
    p.try_parse(|p| p.match_endlines())
        .finish(ErrorKind::none)
        // 不阻塞
        .map_err(|mut e| {
            e.kind = ErrorKind::None;
            e
        })?;
    Ok(crate::syn::EmptyStmt)
}

pub fn parse_compile_unit(p: &mut Parser) -> Result<Box<dyn CompileUnit>, Error> {
    macro_rules! cu_box {
        ($e : expr) => {
            |p| Ok(Box::new(($e)(p)?) as Box<dyn CompileUnit>)
        };
    }
    p.try_parse(cu_box!(parse_fn_call_stmt))
        .or_try_parse(cu_box!(trim_endlines))
        .or_try_parse(cu_box!(ControlFlow::parse))
        .or_try_parse(cu_box!(Block::parse))
        .or_try_parse(cu_box!(FnDef::parse))
        .or_try_parse(cu_box!(Bind::parse))
        .with_note(|| "?".to_string())
        .finish(|| ErrorKind::not_one_of(&["Bind", "If", "While", "FnCall"]))
}

pub fn parse_compile_units(p: &mut Parser) -> Result<Vec<Box<dyn CompileUnit>>, Error> {
    let mut compile_units = vec![];
    // 因为Block::parse会处理 “}”
    // 对于任何错误 直接break 交给外层处理
    while let Ok(compile_unit) = parse_compile_unit(p) {
        compile_units.push(compile_unit)
    }

    // let _ = p.try_parse(|p| p.match_endlines()).finish(ErrorKind::none);
    // assert!(p.get_next_token().is_none()); 作为语句块的结束时可能有剩余
    Ok(compile_units)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parser_test<T, F>(src: &str, f: F) -> Result<T, Error>
    where
        F: FnOnce(&mut Parser) -> Result<T, Error> + 'static,
    {
        let tokens = crate::lexer::Lexer::new(src).toekns();
        let mut parser = crate::parser::Parser::new(&tokens);
        let result = parser.try_parse(f).finish(ErrorKind::none);

        result
    }

    #[test]
    fn expr() {
        let src = r"1 + 2 * 3 + !main()";

        let r = parser_test(src, Expr::parse);
        assert!(r.is_ok());
        assert!(r.unwrap().to_string() == "((1+(2*3))+!main())")
    }

    #[test]
    fn bind() {
        let src = "let a = 12345";
        assert!(parser_test(src, Bind::parse).is_ok());
        let src = "b = a";
        assert!(parser_test(src, Bind::parse).is_ok());
        let src = "let c,d";
        assert!(parser_test(src, Bind::parse).is_ok());
        let src = "let e,f = g,h";
        assert!(parser_test(src, Bind::parse).is_ok());
        let src = "let i,j = k";
        assert!(parser_test(src, Bind::parse).is_err());
    }

    #[test]
    fn control_flow() {
        let src = "if x {}";
        assert!(parser_test(src, ControlFlow::parse).is_ok());

        let src = "while y {}";
        assert!(parser_test(src, ControlFlow::parse).is_ok());
    }

    #[test]
    fn block() {
        let src = r"
            let x
            {  
                x = 12
                let y = 13
                main()
            }
            y = 1
        ";
        let r = parser_test(src, parse_compile_units);
        dbg!(&r);
        assert!(r.is_ok());
    }
}
