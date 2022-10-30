use std::fmt::{Debug, Display};

#[derive(Clone)]
pub struct Pos {
    filename: String,
    line: usize,
    row: usize,
}
impl Pos {
    pub fn new() -> Self {
        Self {
            filename: "".to_owned(),
            line: 0,
            row: 1,
        }
    }
    pub fn new_line(&mut self) {
        self.line += 1;
        self.row = 1;
    }
    pub fn pass(&mut self) {
        self.row += 1;
    }
    pub fn back(&mut self) {
        if self.row > 0 {
            self.row -= 1;
        }
    }
    pub fn set_filename(&mut self, new_name: &str) {
        self.filename = new_name.to_owned();
    }
}

impl Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let l = crate::BRACKET_L;
        let r = crate::BRACKET_R;
        // write!(
        //     f,
        //     "in file {l}{}{r},at \n\tline:{}\n\trow :{}\n",
        //     self.filename, self.line, self.row
        // )
        write!(f, "at {l} {}:{}:{} {r}", self.filename, self.line, self.row)
    }
}
impl Debug for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
