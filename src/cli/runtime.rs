use crate::core::Complier;

pub struct Runtime {
    complier: Complier,
}
impl Runtime {
    pub fn new() -> Self {
        Self {
            complier: Complier::new(),
        }
    }
    pub fn input(&mut self, src: &str) -> &mut Self {
        self.complier.input(src);
        self
    }
    pub fn run(&mut self) -> &mut Self {
        self.lexer();
        self.complier();
        self
    }
    fn lexer(&mut self) {}
    fn complier(&mut self) {}
}
