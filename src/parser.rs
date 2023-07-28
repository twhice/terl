use std::collections::HashMap;

use crate::{
    error::{Error, ErrorKind},
    lexer::{Location, Token, TokenVul},
};

pub trait ParseUnit: Sized {
    fn build(parser: &mut Parser) -> Result<Self, Error>;
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

    pub fn next_endline(&mut self) -> Result<(), Error> {
        self.match_next_token_vul(ErrorKind::NotOneOf(vec!["\n".to_owned()]), |v| {
            matches!(v, TokenVul::EndLine)
        })?;
        // 消除多余换行
        // while self.try_parse(|p| p.next_endline()).finish().is_ok() {}
        Ok(())
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

    pub fn do_parse(&mut self) -> Result<Vec<Box<dyn crate::syn::CompileUnit>>, Error> {
        use crate::ast;
        macro_rules! ast_box {
            ($f : expr) => {
                |p| {
                    let ast = dbg!(($f)(p))?;
                    Ok(Box::new(ast) as Box<dyn crate::syn::CompileUnit>)
                }
            };
        }
        let mut asts = vec![];
        let error: Option<Error>;
        loop {
            let ast = self
                .try_parse(ast_box!(ast::Bind::build))
                .or_try_parse(ast_box!(ast::Expr::fn_call_stmt))
                .finish();
            match ast {
                Ok(ast) => asts.push(ast),
                Err(e) => {
                    error = Some(e);
                    break;
                }
            }
        }

        if let (Some(error), Some(..)) = (error, self.next_token()) {
            return Err(error);
        }

        Ok(asts)
    }

    pub fn finish(&mut self) -> Vec<Error> {
        let mut state = CompileState::default();
        for meta in self.metas.iter() {
            meta.effect(&mut state);
        }
        self.metas
            .iter()
            .filter_map(|meta| meta.test(&mut state).err())
            .collect()
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

#[derive(Debug, Clone, Default)]
pub struct CompileState {
    pub var_use: HashMap<String, Location>,
    pub var_def: HashMap<String, Location>,
    pub var_ass: HashMap<String, Location>,
}

pub trait CompileMeta: 'static {
    fn effect(&self, state: &mut CompileState);
    fn test(&self, state: &mut CompileState) -> Result<(), Error>;
}
