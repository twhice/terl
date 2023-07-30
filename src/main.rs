mod ast;
mod error;
mod lexer;
mod meta;
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
    let mut parser = parser::Parser::new(&tokens);
    let cus = parser.get_compile_units()?;
    dbg!(&cus);
    let (_global_space, warns) = parser.resolve_records()?;
    dbg!(&warns);

    Ok(())
}
