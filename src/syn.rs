use std::fmt::{Debug, Display};

use crate::{
    abi::{Statement, Variable, VariableName, VariableValue},
    ast,
    error::Error,
    meta::GlobalSpace,
};

#[derive(Debug)]
pub struct EmptyStmt;

impl Display for EmptyStmt {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
impl CompileUnit for EmptyStmt {
    fn generate(&mut self, _: &mut GlobalSpace, _: &mut Statements) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Statements {
    stmts: Vec<Statement>,
}

impl std::ops::Index<usize> for Statements {
    type Output = Statement;

    fn index(&self, index: usize) -> &Self::Output {
        &self.stmts[index]
    }
}

impl std::ops::IndexMut<usize> for Statements {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.stmts[index]
    }
}

impl Statements {
    pub fn new() -> Self {
        Self { stmts: vec![] }
    }

    pub fn push_stmt(&mut self, stmt: Statement) {
        self.stmts.push(stmt);
    }

    pub fn link<F>(&mut self, f: F) -> Result<(), Error>
    where
        F: FnOnce(&mut Self) -> Result<(), Error>,
    {
        let mut temp = Self::new();
        f(&mut temp)?;
        for mut stmt in temp.stmts {
            if let Some(old_target) = stmt.jump_target() {
                stmt.reset(self.stmts.len() + old_target);
            }

            self.push_stmt(stmt);
        }
        Ok(())
    }

    pub fn generate<C: CompileUnit>(
        &mut self,
        global: &mut GlobalSpace,
        cu: &mut C,
    ) -> Result<&mut Self, Error> {
        cu.generate(global, self).map(|_| self)
    }

    /// 取得上一次运算的结果,用于各种获取表达式的地方
    ///
    /// * 如果上一次运算是`op  symbol result v1 v2`,会返回result
    /// * 如果上一个运算是`set result = value     `,会直接移除上一行，然后返回value
    pub fn get_last_value(&mut self) -> Option<Variable> {
        match self.stmts.pop().unwrap() {
            Statement::Set { result, value } => {
                // 对于用分配的名字的变量，可以直接折叠
                // 有名字的，也可以直接给值
                return if result.name.is_index() || result.name.is_named() {
                    Some(value)
                // 否则，复制其值
                } else {
                    self.push_stmt(Statement::set(result, value.clone()));
                    Some(value)
                };
            }
            // 对于二元表达式，只能复制result
            Statement::Operation { result, op, v1, v2 } => {
                let value = result.clone();
                self.push_stmt(Statement::Operation { result, op, v1, v2 });
                return Some(value);
            }
            stmt => self.push_stmt(stmt),
        }
        None
    }

    /// 重命名上一次运算的结果,用于Bind
    ///
    /// * 如果上一次运算是`op  symbol result v1 v2`,会重命名result
    /// * 如果上一次运算是`set result = value     `,会重命名result
    pub fn set_last_value<'a>(&mut self, new_name: &'a str) -> Result<&Variable, &'a str> {
        match self.stmts.last_mut().ok_or(new_name)? {
            Statement::Set { result, value } if !result.name.is_named() => {
                result.name = VariableName::named(new_name);
                Ok(value)
            }
            Statement::Operation { result, .. } if !result.name.is_named() => {
                result.name = VariableName::named(new_name);
                Ok(result)
            }
            _ => Err(new_name),
        }
    }

    pub fn generate_jump(&mut self, target: usize, global: &mut GlobalSpace) -> Option<usize> {
        use crate::abi::JumpCondition::{self, *};
        match self.stmts.pop()? {
            Statement::Set { result, value } => {
                // 左值是分配的时，可以直接拿来用
                if result.name.is_index() {
                    self.push_stmt(Statement::jump(value, Neq, Variable::zero(), target));
                } else {
                    self.push_stmt(Statement::set(result.clone(), value));
                    self.push_stmt(Statement::jump(result, Neq, Variable::zero(), target));
                }
                Some(self.stmts.len() - 1)
            }
            Statement::Operation { result, op, v1, v2 } => {
                if result.name.is_index() && JumpCondition::try_from(op).is_ok() {
                    self.push_stmt(Statement::jump(v1, op.try_into().unwrap(), v2, target))
                } else {
                    self.push_stmt(Statement::operation(result.clone(), op, v1, v2));
                    self.push_stmt(Statement::jump(result, Neq, Variable::zero(), target));
                }
                Some(self.stmts.len() - 1)
            }
            stmt => {
                self.push_stmt(stmt);
                None
            }
        }
    }

    pub fn jump_always(&mut self, target: usize) -> usize {
        use crate::abi::JumpCondition::Always;
        self.push_stmt(Statement::jump(
            Variable::zero(),
            Always,
            Variable::zero(),
            target,
        ));
        self.stmts.len() - 1
    }
}

pub trait CompileUnit: Debug + Display {
    fn generate(&mut self, global: &mut GlobalSpace, stmts: &mut Statements) -> Result<(), Error>;
}

impl CompileUnit for ast::Expr<'_> {
    fn generate(&mut self, global: &mut GlobalSpace, stmts: &mut Statements) -> Result<(), Error> {
        match self {
            // 对于直接的值，直接set给分配的名字
            ast::Expr::Var { token, name } => {
                let value = global.global_lookup_var(name, token.location)?;
                stmts.push_stmt(Statement::set(
                    Variable::alloc(),
                    Variable::new(VariableName::named(name), value.clone()),
                ));
            }
            // 对于直接的数字，直接set给分配的名字
            ast::Expr::Num { vul, .. } => stmts.push_stmt(Statement::set(
                Variable::alloc(),
                Variable::new(VariableName::index(), VariableValue::Number(*vul)),
            )),
            // 对于直接的字符串，直接set给分配的名字
            ast::Expr::Str { vul, .. } => stmts.push_stmt(Statement::set(
                Variable::alloc(),
                Variable::new(VariableName::None, VariableValue::String(vul.to_string())),
            )),
            // 对于unary expr
            ast::Expr::Op1 { op, rv } => match op.symbol() {
                // 如果op为 ! 尝试给rv取反
                crate::lexer::Symbol::Not => match !&mut **rv {
                    Ok(o) => {
                        // 如果成功，直接generate rv
                        // 会在generate时进行常量折叠，等
                        stmts.generate(global, o)?;
                    }
                    Err(..) => {
                        // 如果没有成功
                        // 先生成rv
                        stmts.generate(global, &mut **rv)?;
                        // 然后获取结果
                        // 如果是数字，可以直接计算出结果，然后给出set
                        let e = stmts.get_last_value().unwrap();
                        let stmt = if let Some(&number) = e.value.as_number() {
                            let value = Variable::new(
                                VariableName::None,
                                VariableValue::Number((number != 0.0) as u8 as f64),
                            );
                            Statement::set(Variable::alloc(), value)
                        } else {
                            Statement::operation(
                                Variable::alloc(),
                                op.symbol(),
                                e,
                                Variable::zero(),
                            )
                        };
                        stmts.push_stmt(stmt);
                    }
                },
                // 同理
                crate::lexer::Symbol::Sub => match -&mut **rv {
                    Ok(o) => {
                        o.generate(global, stmts)?;
                    }
                    Err(..) => {
                        stmts.generate(global, &mut **rv)?;
                        let e = stmts.get_last_value().unwrap();
                        let stmt = if let Some(&number) = e.value.as_number() {
                            let value =
                                Variable::new(VariableName::None, VariableValue::Number(-number));
                            Statement::set(Variable::alloc(), value)
                        } else {
                            Statement::operation(
                                Variable::alloc(),
                                op.symbol(),
                                Variable::zero(),
                                e,
                            )
                        };
                        stmts.push_stmt(stmt);
                    }
                },
                // 按位取反行为暂时未知，搁置
                crate::lexer::Symbol::Flip => todo!(),
                // 其他的不可能是
                _ => panic!("unreachable"),
            },
            // 如果是二元表达式
            ast::Expr::Op2 { lv, op, rv } => {
                // 分别生成左右，进行取值
                stmts.generate(global, &mut **lv)?;
                let lv = stmts.get_last_value().unwrap();
                stmts.generate(global, &mut **rv)?;
                let rv = stmts.get_last_value().unwrap();
                #[allow(unused)]
                // 如果都是数字，可以直接折叠成set
                let stmt = if let (Some(l), Some(r)) = (lv.value.as_number(), rv.value.as_number())
                {
                    // 常量折叠
                    use std::ops::*;
                    macro_rules! cast_bool {
                        ($e : expr) => {
                            $e as u8 as f64
                        };
                    }

                    let number = match op.symbol() {
                        crate::lexer::Symbol::Add => l.add(r),
                        crate::lexer::Symbol::Sub => l.sub(r),
                        crate::lexer::Symbol::Mul => l.mul(r),
                        crate::lexer::Symbol::Div => l.div(r),
                        crate::lexer::Symbol::IDiv => l.div_euclid(*r),
                        crate::lexer::Symbol::Rem => l.rem(r),
                        crate::lexer::Symbol::Pow => l.powf(*r),
                        // 布尔运算 非零为true 零为false\
                        // true 默认转化为 1.0
                        crate::lexer::Symbol::Eq => cast_bool!(l.eq(r)),
                        crate::lexer::Symbol::Neq => cast_bool!(l.ne(r)),
                        crate::lexer::Symbol::And => cast_bool!(l.ne(&0.0) && l.ne(&0.0)),
                        crate::lexer::Symbol::Or => cast_bool!(l.ne(&0.0) || l.ne(&0.0)),
                        crate::lexer::Symbol::Lr => cast_bool!(l < r),
                        crate::lexer::Symbol::Gr => cast_bool!(l > r),
                        crate::lexer::Symbol::LrE => cast_bool!(l <= r),
                        crate::lexer::Symbol::GrE => cast_bool!(l >= r),
                        // Seq无法保证准确性：因为类型系统不完善
                        crate::lexer::Symbol::Seq => cast_bool!(l.eq(r)),
                        // Shl Shr Band Xor不确定实际行为
                        crate::lexer::Symbol::Shl => ((*l as usize) << (*r as usize)) as f64,
                        crate::lexer::Symbol::Shr => ((*l as usize) << (*r as usize)) as f64,
                        crate::lexer::Symbol::Band => ((*l as usize) & (*r as usize)) as f64,
                        crate::lexer::Symbol::Xor => ((*l as usize) ^ (*r as usize)) as f64,
                        _ => todo!(),
                    };
                    let value = Variable::new(VariableName::None, VariableValue::Number(number));
                    Statement::set(Variable::alloc(), value)
                } else {
                    // 不然只能老老实实（
                    // 这个情况下，是无法进行常量折叠的，也就是说result会保留下来
                    Statement::operation(Variable::alloc(), op.symbol(), lv, rv)
                };
                stmts.push_stmt(stmt);
            }
            ast::Expr::FnCall {
                fn_name_token,
                fn_name,
                args,
            } => {
                global.global_use_fn(fn_name, fn_name_token.location, args.len())?;
                todo!("函数的代码合并，参数传递")
            }
        }
        Ok(())
    }
}

impl CompileUnit for ast::Bind<'_> {
    fn generate(&mut self, global: &mut GlobalSpace, stmts: &mut Statements) -> Result<(), Error> {
        match self {
            ast::Bind::Define {
                var_tokens, vars, ..
            } => {
                for i in 0..vars.len() {
                    global.define_var(vars[i], var_tokens[i].location);
                }
            }
            ast::Bind::Init {
                var_tokens,
                vars,
                ass_op,
                vuls,
                ..
            } => match ass_op.symbol().remove_ass() {
                Some(ass_symbol) => {
                    for _ in 0..vars.len() {
                        // 啊？？？
                        let name = vars.pop().unwrap();
                        let token = var_tokens.pop().unwrap();

                        let mut expr = ast::Expr::Op2 {
                            lv: Box::new(ast::Expr::Var { token, name }),
                            op: ast::Op::Op {
                                token: ass_op.token(),
                                symbol: ass_symbol,
                            },
                            rv: Box::new(vuls.pop().unwrap()),
                        };

                        expr.generate(global, stmts)?;
                        let val = stmts.set_last_value(name).unwrap();
                        global.define_var(name, token.location);
                        global
                            .global_ass_var(name, token.location, val.value.clone())
                            .unwrap();
                    }
                }
                None => {
                    for _ in 0..vars.len() {
                        // 啊？？？
                        let name = vars.pop().unwrap();
                        let location = var_tokens.pop().unwrap().location;

                        let mut expr = vuls.pop().unwrap();

                        expr.generate(global, stmts)?;
                        let val = stmts.set_last_value(name).unwrap();
                        global.define_var(name, location);
                        global
                            .global_ass_var(name, location, val.value.clone())
                            .unwrap();
                    }
                }
            },
            ast::Bind::Ass {
                var_tokens,
                vars,
                ass_op,
                vuls,
            } => match ass_op.symbol().remove_ass() {
                Some(ass_symbol) => {
                    for _ in 0..vars.len() {
                        // 啊？？？
                        let name = vars.pop().unwrap();
                        let token = var_tokens.pop().unwrap();

                        let mut expr = ast::Expr::Op2 {
                            lv: Box::new(ast::Expr::Var { token, name }),
                            op: ast::Op::Op {
                                token: ass_op.token(),
                                symbol: ass_symbol,
                            },
                            rv: Box::new(vuls.pop().unwrap()),
                        };

                        expr.generate(global, stmts)?;
                        let val = stmts.set_last_value(name).unwrap();
                        // global.define_var(name, location);
                        global
                            .global_ass_var(name, token.location, val.value.clone())
                            .unwrap();
                    }
                }
                None => {
                    for _ in 0..vars.len() {
                        // 啊？？？
                        let name = vars.pop().unwrap();
                        let location = var_tokens.pop().unwrap().location;

                        let mut expr = vuls.pop().unwrap();

                        expr.generate(global, stmts)?;
                        let val = stmts.set_last_value(name).unwrap();
                        // global.define_var(name, location);
                        global
                            .global_ass_var(name, location, val.value.clone())
                            .unwrap();
                    }
                }
            },
        }
        Ok(())
    }
}

impl CompileUnit for ast::Block<'_> {
    fn generate(&mut self, global: &mut GlobalSpace, stmts: &mut Statements) -> Result<(), Error> {
        global.new_space();
        for stmt in &mut self.stmts {
            // 防止跳转出现问题
            stmts.link(|stmts| stmt.generate(global, stmts))?;
        }
        global.close_space();
        Ok(())
    }
}

impl CompileUnit for ast::ControlFlow<'_> {
    fn generate(&mut self, global: &mut GlobalSpace, stmts: &mut Statements) -> Result<(), Error> {
        match self {
            /*
                if 跳转设计：
                calc c1
                if c1 -> 'b1
                calc c2
                if c2 -> 'b2
                ...
                calc cn
                if cn -> 'bn
                always -> 'else / end

                'b1: ...
                always -> end
                'b2: ...
                always -> end
                ...
                'bn: ...
                [
                    always -> end
                    'else : ...
                ] or [
                ]
                'end : ...
            */
            ast::ControlFlow::If {
                conditions,
                blocks,
                else_block,
                ..
            } => {
                let mut jumps = Vec::with_capacity(conditions.len());
                for condition in conditions {
                    // calc cn
                    stmts.generate(global, condition)?;
                    // if cn -> '..
                    let jump = stmts.generate_jump(0, global).unwrap();
                    jumps.push(jump);
                }

                let to_else_or_end = stmts.jump_always(0);
                let mut to_ends = Vec::with_capacity(blocks.len());
                for i in 0..blocks.len() {
                    // 进行jump
                    let jump = jumps[i];
                    let stmts_len = stmts.stmts.len();
                    stmts[jump].reset(stmts_len);

                    // 链接，不然跳转会坏
                    let block = &mut blocks[i];
                    // stmts.generate(global, block)?;
                    stmts.link(|stmts| stmts.generate(global, block).map(|_| ()))?;

                    // 最后一个跳转块
                    let to_end = if i == blocks.len() - 1 {
                        if else_block.is_some() {
                            // 有else块
                            let to_end = stmts.jump_always(0);
                            // 是to_else
                            let stmts_len = stmts.stmts.len();
                            stmts[to_else_or_end].reset(stmts_len);
                            // 链接else块
                            stmts.link(|stmts| {
                                stmts
                                    .generate(global, else_block.as_mut().unwrap())
                                    .map(|_| ())
                            })?;

                            to_end
                            //
                        } else {
                            // 没有else块 to_else_or_end -> to_end
                            to_else_or_end
                        }
                    } else {
                        stmts.jump_always(0)
                    };
                    to_ends.push(to_end);
                }

                for to_end in to_ends {
                    let stmts_len = stmts.stmts.len();
                    stmts[to_end].reset(stmts_len);
                }
            }

            /*
                while跳转设计
                'calc: calc cond
                if !cond -> 'end
                'block: ..
                jump always -> 'clac
                'end ..
            */
            ast::ControlFlow::While {
                r#while,
                condition,
                block,
            } => {
                // 'calc
                let calc = stmts.stmts.len();
                // 居然有Clone！！！好大的性能问题！！！
                let mut condition = ast::Expr::Op2 {
                    lv: Box::new(condition.clone()),
                    op: ast::Op::Op {
                        token: r#while,
                        symbol: crate::lexer::Symbol::Eq,
                    },
                    rv: Box::new(ast::Expr::Num {
                        token: r#while,
                        vul: 0.0,
                    }),
                };
                // calc cond
                stmts.generate(global, &mut condition)?;
                // if !cond -> 'end
                let jump_to_end = stmts.generate_jump(0, global).unwrap();
                // 'block
                stmts.link(|stmts| stmts.generate(global, block).map(|_| ()))?;
                // jump always -> 'calc
                stmts.jump_always(calc);
                // 'end
                let len = stmts.stmts.len();
                stmts[jump_to_end].reset(len);
            }
        }
        Ok(())
    }
}

impl CompileUnit for ast::FnDef<'_> {
    fn generate(&mut self, global: &mut GlobalSpace, stmts: &mut Statements) -> Result<(), Error> {
        Ok(())
    }
}
