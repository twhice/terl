use std::fmt::Debug;

pub trait CompileUnit: Debug {
    fn generate(&self) -> Vec<String>;
}

impl CompileUnit for () {
    fn generate(&self) -> Vec<String> {
        vec![]
    }
}
