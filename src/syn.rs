use std::fmt::{Debug, Display};

use crate::ast;

#[derive(Debug)]
pub struct EmptyStmt;

impl Display for EmptyStmt {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
impl CompileUnit for EmptyStmt {
    fn generate(&self) -> Vec<String> {
        vec![]
    }
}

pub trait CompileUnit: Debug + Display {
    fn generate(&self) -> Vec<String>;
}

impl CompileUnit for ast::Expr<'_> {
    fn generate(&self) -> Vec<String> {
        todo!()
    }
}

impl CompileUnit for ast::Bind<'_> {
    fn generate(&self) -> Vec<String> {
        todo!()
    }
}

impl CompileUnit for ast::Block<'_> {
    fn generate(&self) -> Vec<String> {
        todo!()
    }
}

impl CompileUnit for ast::ControlFlow<'_> {
    fn generate(&self) -> Vec<String> {
        todo!()
    }
}

impl CompileUnit for ast::FnDef<'_> {
    fn generate(&self) -> Vec<String> {
        todo!()
    }
}
