use std::ops::Deref;

pub struct Lexer {
    location: Location,
    lines: Vec<Vec<char>>,
}

impl Lexer {
    pub fn new(src: &str) -> Lexer {
        let lines = src
            .lines()
            .map(|l| l.chars().chain(std::iter::once('\n')).collect())
            .collect();
        Lexer {
            lines,
            location: Location::new(0, 0, 0),
        }
    }

    fn next_char(&mut self) -> Option<char> {
        self.location.row += 1;
        if self.location.row == self.lines.get(self.location.line)?.len() {
            self.location.line += 1;
            self.location.row = 0;
        }

        if self.location.line == self.lines.len() {
            return None;
        }
        Some(self.lines[self.location.line][self.location.row])
    }

    fn this_char(&self) -> Option<char> {
        self.lines
            .get(self.location.line)?
            .get(self.location.row)
            .copied()
    }

    fn next(&mut self) -> Option<Token> {
        let this_char = self.this_char()?;
        if this_char.is_alphabetic() || this_char == '_' {
            self.collect(|s| {
                let mut ident: String = String::new();
                while let Some(this) = s.this_char() {
                    if !(this.is_alphanumeric() || this == '_') {
                        break;
                    } else {
                        s.next_char();
                        ident.push(this);
                    }
                }
                TokenVul::Ident(ident)
            })
        } else if this_char.is_ascii_digit() {
            self.collect(|s| {
                let mut float = false;
                let mut number: String = String::new();
                while let Some(this) = s.this_char() {
                    if !(this.is_ascii_digit()) || (!float && this == '.') {
                        break;
                    }
                    if this == '.' {
                        float = !float
                    }
                    s.next_char();
                    number.push(this)
                }
                TokenVul::Number(number.parse().unwrap())
            })
        } else if this_char == '\n' {
            self.collect(|s| {
                s.next_char();
                TokenVul::EndLine
            })
        } else if this_char == ' ' {
            while self.this_char()? == ' ' {
                self.next_char();
            }
            self.next()
        } else if this_char == '#' {
            self.collect(|s| {
                while s.this_char().is_some_and(|this| this != '\n') {
                    s.next_char();
                }
                TokenVul::EndLine
            })
        } else if this_char == '\"' {
            self.next_char();
            self.collect(|s| {
                let mut collect = String::new();
                loop {
                    match s.this_char() {
                        Some('\"') => {
                            return TokenVul::String(collect);
                        }
                        Some(this) => {
                            let real = if this == '\\' {
                                let Some(real) = s.next_char() else {
                                    return TokenVul::Unknow(collect);
                                };
                                real
                            } else {
                                this
                            };
                            collect.push(real);
                            s.next_char();
                        }
                        None => {
                            return TokenVul::Unknow("\"".to_owned() + &collect);
                        }
                    }
                }
            })
        } else {
            self.collect(|s| {
                let mut string = String::from(this_char);
                let mut symbol = match string.parse::<Symbol>() {
                    Ok(symbol) => symbol,
                    Err(_) => return TokenVul::Unknow(string),
                };
                while let Some(char) = s.next_char() {
                    string.push(char);
                    match string.parse::<Symbol>() {
                        Ok(new_symbol) => symbol = new_symbol,
                        Err(_) => break,
                    }
                }
                TokenVul::Symbol(symbol)
            })
        }
    }

    fn collect<F>(&mut self, collector: F) -> Option<Token>
    where
        F: FnOnce(&mut Self) -> TokenVul,
    {
        let mut location = self.location;
        let vul = collector(self);
        if let TokenVul::Unknow(..) = vul {
            return None;
        }
        if location.line == self.location.line {
            location.len = self.location.row - location.row;
        }

        Some(Token { location, vul })
    }

    pub fn toekns(mut self) -> Vec<Token> {
        (0..)
            .map(|_| self.next())
            .take_while(|opt| opt.is_some())
            .flatten()
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub location: Location,
    pub vul: TokenVul,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Location {
    pub line: usize,
    pub row: usize,
    pub len: usize,
}

impl Location {
    pub const fn new(line: usize, row: usize, len: usize) -> Location {
        Location { line, row, len }
    }

    pub const fn display_location(&self) -> (usize, usize) {
        (self.line + 1, self.row + 1)
    }

    pub const fn read_location(&self) -> (usize, usize) {
        (self.line, self.row)
    }

    pub fn locate<'a, S>(&self, lines: &'a [S]) -> Option<&'a [char]>
    where
        S: Deref<Target = [char]>,
    {
        Some(&lines.get(self.len)?[self.row..self.row + self.len])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenVul {
    Ident(String),
    String(String),
    Number(f64),
    Symbol(Symbol),
    EndLine,
    Unknow(String),
}

impl TokenVul {
    pub fn r#type(&self) -> &'static str {
        match self {
            TokenVul::Ident(_) => "标识符",
            TokenVul::String(_) => "字符串",
            TokenVul::Number(_) => "数字",
            TokenVul::Symbol(_) => "符号",
            TokenVul::EndLine => "换行",
            TokenVul::Unknow(_) => "未知",
        }
    }
}

macro_rules! symbols {
    ($($name : ident ,$src : literal ,$is_ass : expr , $is_op : expr, $priority : expr);*) => {
        #[derive(Debug,Clone,Copy,PartialEq,Eq)]
        pub enum Symbol{
            #[allow(unused)]
            None,
            $($name,)*
        }

        impl std::str::FromStr for Symbol{
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let r = match s{
                    $(
                        $src => Self::$name,
                    )*
                    _=> return Err(())
                };
                Ok(r)
            }
        }

        impl Symbol{
            pub fn is_ass_op(&self) -> bool{
                match self{
                    $(
                        Self::$name => $is_ass,
                    )*
                    _=> false
                }
            }

            pub fn is_op(&self) -> bool{
                match self{
                    $(
                        Self::$name => $is_op,
                    )*
                    _=> false
                }
            }

            pub fn priority(&self) -> usize{
                match self{
                    $(
                        Self::$name => $priority,
                    )*
                    _=> 0
                }
            }
        }

        impl std::fmt::Display for Symbol{
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let w = match self{
                    $(
                        Self::$name => $src,
                    )*
                    _=>"None"
                };
                write!(f,"{w}")
            }
}
    };
}

symbols! {   // is_ass_op is_op
    Add      ,"+"   ,false ,true  ,4;
    Sub      ,"-"   ,false ,true  ,4;
    Mul      ,"*"   ,false ,true  ,4;
    Div      ,"/"   ,false ,true  ,4;
    Not      ,"!"   ,false ,true  ,9;
    Split    ,","   ,false ,false ,0;
    BarcketL ,"("   ,false ,false ,0;
    BarcketR ,")"   ,false ,false ,0;
    Ass      ,"="   ,true  ,false ,0

}

impl Symbol {
    pub fn is_unary(&self) -> bool {
        matches!(self, Self::Not | Self::Sub)
    }
}
