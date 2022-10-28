use super::{context::Word, lexer::Lexer, tree::Tree};
/// "编译器"
///
/// input方法输入
///
/// lex方法解析
///
/// # Examples
/// ```
/// use terl::core::complier::Complier;
/// let mut complier = Complier::new();
/// complier.input("// nuthing\n114514\n").lex();
/// ```
#[derive(Debug, Clone)]
pub struct Complier {
    src: Vec<Vec<char>>,
    main: Tree<Word>,

    lexer: Lexer,
}

impl Complier {
    pub fn new() -> Self {
        Self {
            src: Vec::new(),
            main: Tree::Node(Vec::new()),
            lexer: Lexer::new(),
        }
    }
    pub fn input(&mut self, src: &str) -> &mut Self {
        for line in src.lines() {
            self.src.push(line.chars().collect())
        }
        self
    }
    pub fn lex(&mut self) {
        loop {
            if let Some(line) = &self.src.first() {
                self.lexer.lex(line.to_owned().clone());
                self.src.remove(0);
                self.main.push(self.lexer.get_word());
                // println!("{:?}", self);
            } else {
                break;
            }
        }
    }
}
