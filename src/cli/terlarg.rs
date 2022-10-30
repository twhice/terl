use std::fmt::Display;

pub enum TerlArg {
    Help,
    Version,
    None,
}
impl TerlArg {
    pub fn new(arg: &str) -> TerlArg {
        let mut match_index = None;
        for index in 0..CONST_ARGS.len() {
            let const_arg = CONST_ARGS[index];
            if arg == const_arg {
                match_index = Some(index)
            }
        }
        if let Some(_index) = match_index {
            match _index {
                1 | 3 => TerlArg::Help,
                2 | 4 => TerlArg::Version,
                _ => TerlArg::None,
            }
        } else {
            TerlArg::None
        }
    }
    fn show(&self) -> String {
        match self {
            TerlArg::Help => {
                let mut str = String::new();
                str += "terl";
                str += "\n\th / help     : 帮助";
                str += "\n\tv / version  : 获取版本信息";
                str += "\n\tsrc_file_name: 运行terl源文件";
                str += "\n\tdir_name     : 运行某文件夹";
                format!("{}", str)
            }
            TerlArg::Version => {
                let mut str = String::new();
                str += "terl v0.0.1";
                str += "\nBy     : (twhice)异月";
                str += "\nLICENSE: Apache License 2.0";
                format!("{}", str)
            }
            TerlArg::None => {
                let mut str = String::new();
                str += "Unknow argument";
                str += "\ntry to use \"terl help\" for help";
                format!("{}", str)
            } // _ => todo!(),
        }
    }
}
impl Display for TerlArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.show())
    }
}
static CONST_ARGS: [&str; 5] = ["", "help", "version", "h", "v"];
