use std::fmt::Debug;
/// 常量,是区分出if/else/return的参考标准
/// 废除impl
pub static TOKENS: [&str; 5] = ["if", "else", "return", "pub", "import"];
/// terl的变量名
///
#[derive(Clone)]
pub struct Name {
    name: Vec<char>,
}

impl Name {
    pub fn new(name: Vec<char>) -> Self {
        Self { name }
    }
    pub fn get_name(&self) -> Vec<char> {
        self.name.clone()
    }

    pub fn set_name(&mut self, name: Vec<char>) {
        self.name = name;
    }
}
impl Debug for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str = String::new();
        for char in &self.name {
            let c = String::from(char.to_owned());
            str += c.as_str();
        }

        write!(f, "{}", str)
    }
}
impl From<&str> for Name {
    fn from(str: &str) -> Self {
        Self::new(str.chars().collect())
    }
}
impl From<Vec<char>> for Name {
    fn from(str: Vec<char>) -> Self {
        Self::new(str)
    }
}
#[derive(Clone)]
pub struct Type {
    name: Name,
}
impl Type {
    pub fn new(name: Name) -> Self {
        Self { name }
    }
    pub fn get_typename(&self) -> Name {
        self.name.clone()
    }
}
impl From<&str> for Type {
    fn from(str: &str) -> Self {
        Self::new(Name::from(str))
    }
}
impl From<Name> for Type {
    fn from(name: Name) -> Self {
        Type::new(name)
    }
}
impl From<Vec<char>> for Type {
    fn from(vec: Vec<char>) -> Self {
        Type::new(vec.into())
    }
}
impl Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.name)
    }
}
