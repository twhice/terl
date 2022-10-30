use crate::core::Complier;
#[derive(Debug)]
pub struct Runtime {
    complier: Box<Complier>,
}
impl Runtime {
    pub fn new() -> Self {
        Self {
            complier: Box::new(Complier::new()),
        }
    }
    pub fn input(&mut self, src: &str, filename: &str) -> &mut Self {
        self.complier.input(src, filename);
        self
    }
    pub fn run(&mut self) -> &mut Self {
        self.lexer();
        self.complier();
        self
    }
    fn lexer(&mut self) {
        self.complier.lex()
    }
    fn complier(&mut self) {}
}
