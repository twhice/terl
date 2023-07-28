mod ast;
mod error;
mod lexer;
mod parser;
mod syn;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // use std::env::args;
    // let mut args = args();
    // assert!(args.len() == 2, "用法: <输入文件>");
    // let src_path = args.nth(1).unwrap();
    let src_path = "test.tl";
    let src = std::fs::read_to_string(src_path)?;
    let tokens = lexer::Lexer::new(&src).toekns();
    dbg!(&tokens);
    let mut parser = parser::Parser::new(&tokens);
    let asts = parser.do_parse()?;
    // dbg!(&asts);
    let errors = parser.finish();
    Ok(())
}
