mod abi;
mod ast;
mod error;
mod lexer;
mod meta;
mod parser;
mod syn;
/*
    编译流程：
        源码 经过词法分析 被解析成大量Token
        Token经过语法分析 被解析成Ast
        Ast在代码生成进行一些逻辑性的检查：
            * 函数定义时的参数和调用时是否一致
            * 参数的作用域静态检查

*/
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

    let mut global = meta::GlobalSpace::new();
    let mut stmts = syn::Statements::new();

    for mut cu in cus {
        stmts.link(|stmts| cu.generate(&mut global, stmts))?;
    }

    dbg!(&stmts);

    Ok(())
}
