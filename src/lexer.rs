use std::{fmt::Display, ops::Deref};

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
        if this_char.is_alphabetic() || this_char == '_' || this_char == '@' {
            self.collect(|s| {
                let mut ident: String = String::new();
                while let Some(this) = s.this_char() {
                    if !(this.is_alphanumeric() || this == '_' || this_char == '@') {
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
            fn string_collector(s: &mut Lexer) -> TokenVul {
                let mut collect = String::new();
                loop {
                    match s.this_char() {
                        Some('\"') => {
                            s.next_char();
                            return TokenVul::String(collect);
                        }
                        Some(this) => {
                            let real = if this == '\\' {
                                let Some(real) = s.next_char() else {
                                    return TokenVul::Unknow(collect);
                                };
                                match real {
                                    'n' => '\n',
                                    't' => '\t',
                                    'r' => '\r',
                                    '\"' => '\"',
                                    _ => {
                                        // 忽略 不是转义字符
                                        collect.push('\\');
                                        real
                                    }
                                }
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
            }
            self.collect(string_collector)
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

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let location = self.location.display_location();
        write!(f, "{} at {}:{}", self.vul, location.0, location.1)
    }
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

impl Display for TokenVul {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenVul::Ident(ident) => write!(f, "{ident}"),
            TokenVul::String(string) => write!(f, "\"{string}\""),
            TokenVul::Number(number) => write!(f, "{number}"),
            TokenVul::Symbol(symbol) => write!(f, "{symbol}"),
            TokenVul::EndLine => write!(f, "ENDLINE"),
            TokenVul::Unknow(unknow) => write!(f, "UNKNOW({unknow})"),
        }
    }
}

macro_rules! symbols {
    ($($name : ident ,$src : literal ,$is_ass : expr , $is_op : expr, $priority : expr);*) => {
        /// 是词法上的符号，也是abi的符号
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

symbols! {//     is_ass_op  is_op
    // 算术
    Add      ,"+"   ,false ,true  ,5;
    Sub      ,"-"   ,false ,true  ,5;
    Mul      ,"*"   ,false ,true  ,6;
    Div      ,"/"   ,false ,true  ,6;
    IDiv     ,"//"  ,false ,true  ,6;
    Rem      ,"%"   ,false ,true  ,6;
    Pow      ,"**"  ,false ,true  ,7;

    // 逻辑
    Eq       ,"=="  ,false ,true  ,3;
    Neq      ,"!="  ,false ,true  ,3;
    And      ,"&&"  ,false ,true  ,3;
    Not      ,"!"   ,false ,true  ,9;
    Or       ,"||"  ,false ,true  ,3;
    Lr       ,"<"   ,false ,true  ,3;
    Gr       ,">"   ,false ,true  ,3;
    Seq      ,"===" ,false ,true  ,3;
    LrE      ,"<="  ,false ,true  ,3;
    GrE      ,">="  ,false ,true  ,3;

    // 位运算
    Shl      ,"<<"  ,false ,true  ,4;
    Shr      ,">>"  ,false ,true  ,4;
    Band     ,"&"   ,false ,true  ,4;
    Xor      ,"^"   ,false ,true  ,4;
    Flip     ,"~"   ,false ,true  ,4;

    // 词法符号
    Split    ,","   ,false ,false ,0;
    BarcketL ,"("   ,false ,false ,0;
    BarcketR ,")"   ,false ,false ,0;
    SpaceL   ,"{"   ,false ,false ,0;
    SpaceR   ,"}"   ,false ,false ,0;

    // 赋值运算符（纯粹语法糖）
    Ass         ,"="    ,true  ,false ,0;
    AddAss      ,"+="   ,true  ,false ,0;
    SubAss      ,"-="   ,true  ,false ,0;
    MulAss      ,"*="   ,true  ,false ,0;
    DivAss      ,"/="   ,true  ,false ,0;
    IDivAss     ,"//="  ,true  ,false ,0;
    ModAss      ,"%="   ,true  ,false ,0;
    PowAss      ,"**="  ,true  ,false ,0;
    NeqAss      ,"!=="  ,true  ,false ,0;
    AndAss      ,"&&="  ,true  ,false ,0;
    NotAss      ,"!="   ,true  ,false ,0;
    OrAss       ,"||="  ,true  ,false ,0;
    LrAss       ,"<="   ,true  ,false ,0;
    GrAss       ,">="   ,true  ,false ,0;
    LrEAss      ,"<=="  ,true  ,false ,0;
    GrEAss      ,">=="  ,true  ,false ,0;
    ShlAss      ,"<<="  ,true  ,false ,0;
    ShrAss      ,">>="  ,true  ,false ,0;
    BandAss     ,"&="   ,true  ,false ,0;
    XorAss      ,"^="   ,true  ,false ,0;
    FlipAss     ,"~="   ,true  ,false ,0

    // EqAss       ,"==="  ,true  ,false ,0; 歧义Seq
    // SeqAss      ,"====" ,true  ,false ,0; 什么鬼

}

impl Symbol {
    pub fn is_unary(&self) -> bool {
        matches!(self, Self::Not | Self::Sub | Self::Flip)
    }

    pub fn remove_ass(&self) -> Option<Self> {
        let op = match self {
            Self::AddAss => Self::Add,
            Self::SubAss => Self::Sub,
            Self::MulAss => Self::Mul,
            Self::DivAss => Self::Div,
            Self::IDivAss => Self::IDiv,
            Self::ModAss => Self::Rem,
            Self::PowAss => Self::Pow,
            Self::NeqAss => Self::Neq,
            Self::AndAss => Self::And,
            Self::NotAss => Self::Not,
            Self::OrAss => Self::Or,
            Self::LrAss => Self::Lr,
            Self::GrAss => Self::Gr,
            Self::LrEAss => Self::LrE,
            Self::GrEAss => Self::GrE,
            Self::ShlAss => Self::Shl,
            Self::ShrAss => Self::Shr,
            Self::BandAss => Self::Band,
            Self::XorAss => Self::Xor,
            Self::FlipAss => Self::Flip,
            _ => return None,
        };
        Some(op)
    }
}

impl std::ops::Not for Symbol {
    type Output = Option<Symbol>;

    fn not(self) -> Self::Output {
        let result = match self {
            Self::Gr => Self::LrE,
            Self::LrE => Self::Gr,
            Self::Lr => Self::GrE,
            Self::GrE => Self::Lr,
            Self::Eq => Self::Neq,
            Self::Neq => Self::Eq,
            Self::Not => Self::None,
            _ => return None,
        };
        Some(result)
    }
}
