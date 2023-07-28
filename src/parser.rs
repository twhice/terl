use crate::{
    error::{Error, ErrorKind},
    lexer::{Token, TokenVul},
};

pub trait CompileUnit: Sized {
    fn build(parser: &mut Parser) -> Result<Self, Error>;
    fn compile(&self) -> Vec<String>;
}

pub struct Parser<'a> {
    tokens: &'a [Token],
    index: usize,
    metas: Vec<Box<dyn CompileMeta>>,
}

impl<'t> Parser<'t> {
    pub fn new(tokens: &[Token]) -> Parser {
        Parser {
            tokens,
            index: 0,

            metas: vec![],
        }
    }

    pub fn next_token(&mut self) -> Option<&'static Token> {
        let opt = self
            .tokens
            .get(self.index)
            .map(|token| unsafe { std::mem::transmute(token) });
        self.index += 1;
        opt
    }

    pub fn match_next_token_vul<F>(
        &mut self,
        error: ErrorKind,
        matcher: F,
    ) -> Result<&'static Token, Error>
    where
        F: FnOnce(&TokenVul) -> bool,
    {
        match self.next_token() {
            Some(token) => {
                if !matcher(&token.vul) {
                    return Err(error.to_error(token));
                }
                Ok(token)
            }
            None => Err(error.error()),
        }
    }

    pub fn push_meta<M: CompileMeta>(&mut self, m: M) {
        self.metas.push(Box::new(m));
    }

    pub fn try_parse<T, F>(&mut self, f: F) -> ParserState<'t, '_, T>
    where
        F: FnOnce(&mut Parser) -> Result<T, Error>,
    {
        let state = ParserState {
            parser: self,
            state: Err(Error::empty()),
        };
        state.or_try_parse(f)
    }

    fn append(&mut self, ano: Self) {
        self.index = ano.index;
        self.metas.extend(ano.metas);
    }
}

impl Clone for Parser<'_> {
    fn clone(&self) -> Self {
        Self {
            tokens: self.tokens,
            index: self.index,
            metas: vec![],
        }
    }
}

pub struct ParserState<'t, 'a, T> {
    parser: &'a mut Parser<'t>,
    state: Result<T, Error>,
}

impl<T> ParserState<'_, '_, T> {
    pub fn or_try_parse<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut Parser) -> Result<T, Error>,
    {
        if self.state.is_ok() {
            return self;
        }

        let mut temp = self.parser.clone();
        self.state = (f)(&mut temp);
        if self.state.is_ok() {
            self.parser.append(temp)
        }
        self
    }

    pub fn finish(self) -> Result<T, Error> {
        self.state
    }
}

pub struct CompileState {}

pub trait CompileMeta: 'static {
    fn effect(&self, state: &mut CompileState);
    fn test(&self, state: &mut CompileState) -> Result<(), Error>;
}
