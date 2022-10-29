mod open;
mod prel;
mod runtime;
mod script;
mod terlarg;

use std::env::args;
use terlarg::*;
/*
    terl const(help...)
    terl input_filename
    terl dir_name
*/
pub fn program_begin() {
    let args = args().collect::<Vec<String>>();
    match args.len() {
        1 => {
            prel::start_prel();
            return;
        }
        2 => {
            let real_arg = args[1].clone();
            let arg_cmd = TerlArg::new(&real_arg);
            match arg_cmd {
                TerlArg::None => {
                    // 不ret的分支,执行后面的语句
                    // (为了美观)
                }
                _ => {
                    println!("{}", arg_cmd);
                    return;
                }
            }
        }
        _ => {
            // 同上
        }
    }
    match open::open_src_file(args[1].clone().into()) {
        Some(_src) => {
            // 参数列表暂时不考虑
            script::start_script(&_src, Vec::new());
        }
        None => {
            // 不考虑
        }
    }
    todo!("打开文件夹,其他情况")
}
