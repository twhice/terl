use std::env::args;

mod ast;
mod error;
mod lexer;
mod parser;

fn main() -> std::io::Result<()> {
    let mut args = args();
    assert!(args.len() == 2, "用法: <输入文件>");
    let src_path = args.nth(1).unwrap();
    let src = std::fs::read_to_string(src_path)?;
    let tokens = lexer::Lexer::new(&src).toekns();
    println!("{:?}", tokens);
    Ok(())
}
