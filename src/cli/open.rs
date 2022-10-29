use std::path::PathBuf;

pub fn open_src_file(path: PathBuf) -> Option<String> {
    match std::fs::read_to_string(path) {
        Ok(_src) => Some(_src),
        Err(_) => None,
    }
}
