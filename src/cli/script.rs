use std::path::PathBuf;

use super::open;

pub fn start_script(path: PathBuf, args: Vec<String>) {
    let _path;
    if false {
        println!("{:?}", args)
    }
    if let Some(str) = path.clone().to_str() {
        _path = str;
        if let Some(src) = open::open_src_file(path) {
            let mut runtime = super::runtime::Runtime::new();
            runtime.input(&src, _path).run();
            println!("{:?}", runtime);
        }
    }
}
