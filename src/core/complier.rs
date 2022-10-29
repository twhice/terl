use super::{
    context::Statement,
    lexer::Lexer,
    tree::{BeTree, Tree},
};
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
    main: Tree<Statement>,
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
                self.main
                    .push(self.lexer.lex(line.to_owned().clone()).get_statement());
                self.src.remove(0);
            } else {
                break;
            }
        }
        self.fill_main();
    }
    fn fill_main(&mut self) {
        self.main = Statement::build_deep_tree(self.main.open_to_vec(), 0);
        let mut fill_one = false;
        let main_vec = &mut self.main.node_to_vec();
        for index in 0..main_vec.len() {
            let statement = main_vec[index].clone();
            match &statement {
                Tree::Node(_vec) => {
                    if fill_one {
                        main_vec[index - 1].open_to_vec()[0]
                            .context()
                            .fill_block(statement);
                        fill_one = false;
                        continue;
                    }
                }
                Tree::Dot(_statement) => match _statement.clone().context() {
                    super::context::Context::If(_, _)
                    | super::context::Context::Else(_)
                    | super::context::Context::DefFun(_, _, _, _)
                    | super::context::Context::DefStruct(_, _, _) => {
                        if !fill_one {
                            fill_one = true
                        } else {
                            fill_one = false
                        }
                    }
                    _ => {}
                },
            }
        }
    }
}
