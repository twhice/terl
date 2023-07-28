use std::fmt::Debug;

use crate::ast;

pub trait CompileUnit: Debug {
    fn generate(&self) -> Vec<String>;
}

impl CompileUnit for () {
    fn generate(&self) -> Vec<String> {
        vec![]
    }
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
