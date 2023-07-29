use std::fmt::Debug;

use crate::{
    error::{Error, ErrorKind},
    lexer::{Symbol, Token},
    meta::CompileRecord,
};

#[derive(Debug)]
pub struct Parser<'a> {
    index: usize,
    tokens: &'a [Token],
    metas: Vec<Box<dyn CompileRecord>>,
}

impl Parser<'_> {
    pub fn new(tokens: &[Token]) -> Parser {
        Parser {
            tokens,
            index: 0,

            metas: vec![],
        }
    }

    pub fn get_next_token(&mut self) -> Option<&'static Token> {
        let token = self
            .tokens
            .get(self.index)
            .map(|tk| unsafe { std::mem::transmute(tk) });
        self.index += 1;
        token
    }

    pub fn ul_err<E>(&mut self, e: E) -> Error
    where
        E: FnOnce() -> ErrorKind,
    {
        let e = e();
        if e.is_none() {
            return Error::empty();
        }
        let token = if self.index >= self.tokens.len() {
            self.tokens.last().unwrap()
        } else {
            &self.tokens[self.index]
        };

        e.generate_error(token)
    }

    pub fn l_err<E>(&mut self, e: E, token: &Token) -> Error
    where
        E: FnOnce() -> ErrorKind,
    {
        let e = e();
        if e.is_none() {
            return Error::empty();
        }
        e.generate_error(token)
    }
}

macro_rules! generate_getter {
    ($getter : ident,$matcher : ident,$v : ident,$ret : ty) => {
        pub fn $getter<E>(&mut self, or: E) -> Result<(&'static Token, &'static $ret), Error>
        where
            E: FnOnce() -> ErrorKind,
        {
            match self.get_next_token() {
                Some(token) => match &token.vul {
                    crate::lexer::TokenVul::$v(ver) => Ok((token, ver)),
                    _ => Err(self.l_err(or, token)),
                },
                None => Err(self.ul_err(or)),
            }
        }

        pub fn $matcher<E>(&mut self, matcher: &$ret, or: E) -> Result<&'static Token, Error>
        where
            E: FnOnce() -> ErrorKind + Copy,
        {
            let (token, item) = self.$getter(or)?;

            match item == matcher {
                true => Ok(token),
                false => Err(self.l_err(or, token)),
            }
        }
    };
}

impl Parser<'_> {
    generate_getter! {get_ident,match_ident, Ident, String}
    generate_getter! {get_string, match_string,String, String}
    generate_getter! {get_number, match_number,Number, f64}
    generate_getter! {get_symbol, match_symbol,Symbol, Symbol}
    pub fn match_endlines(&mut self) -> Result<(), Error> {
        fn e() -> ErrorKind {
            ErrorKind::not("换行")
        }
        fn match_endline(p: &mut Parser) -> Result<(), Error> {
            match p.get_next_token() {
                Some(token) => match &token.vul {
                    crate::lexer::TokenVul::EndLine => Ok(()),
                    _ => Err(p.l_err(e, token)),
                },
                None => Err(p.ul_err(e)),
            }
        }

        match_endline(self)?;
        while self.try_parse(match_endline).finish(e).is_ok() {}
        Ok(())
    }
}

impl Parser<'_> {}

#[derive(Debug)]
pub struct Try<'t, 'p, T> {
    parser: &'p mut Parser<'t>,
    state: Option<Result<T, Error>>,
}

impl<T> Try<'_, '_, T> {
    pub fn or_try_parse<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut Parser) -> Result<T, Error> + 'static,
    {
        if self.state.is_some() {
            return self;
        }
        // 创造一个temp 进行收集
        let mut temp = Parser::new(self.parser.tokens);
        temp.index = self.parser.index;

        // 如果成功
        match f(&mut temp) {
            Ok(t) => {
                self.state = Some(Ok(t));
                self.parser.metas.extend(temp.metas);
                self.parser.index = temp.index;
            }

            Err(e) => {
                if !e.kind.is_none() {
                    self.state = Some(Err(e))
                }
            }
        }

        self
    }

    pub fn with_note<N>(mut self, note: N) -> Self
    where
        N: FnOnce() -> String,
    {
        if let Some(Err(e)) = self.state.as_mut() {
            if e.note.is_empty() {
                e.note = note();
            }
        }
        self
    }

    pub fn finish<E>(self, default_error: E) -> Result<T, Error>
    where
        E: FnOnce() -> ErrorKind,
    {
        match self.state {
            Some(r) => r,
            None => Err(self.parser.ul_err(default_error)),
        }
    }
}

impl<'t> Parser<'t> {
    pub fn try_parse<'p, T, F>(&'p mut self, f: F) -> Try<'t, 'p, T>
    where
        F: FnOnce(&mut Parser) -> Result<T, Error> + 'static,
    {
        let r#try = Try {
            parser: self,
            state: None,
        };
        r#try.or_try_parse(f)
    }
}
