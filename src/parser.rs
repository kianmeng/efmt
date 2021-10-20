use crate::ast::Ast;
use crate::expect::{ExpectAtom, ExpectSymbol, ExpectVariable};
use crate::{Lexer, Result};
use erl_tokenize::tokens::{AtomToken, VariableToken};
use erl_tokenize::values::Symbol;

pub trait Parse: Sized {
    fn parse(lexer: &mut Lexer) -> Result<Self>;

    fn try_parse(lexer: &mut Lexer) -> Option<Self> {
        lexer.with_transaction(|lexer| Self::parse(lexer)).ok()
    }

    fn expect(&self, lexer: &mut Lexer) -> Result<()>
    where
        Self: PartialEq + std::fmt::Debug,
    {
        lexer.with_transaction(|lexer| {
            let value = Self::parse(lexer)?;
            if value == *self {
                Ok(())
            } else {
                Err(anyhow::anyhow!("expected {:?}, but got {:?}", self, value).into())
            }
        })
    }
}

#[derive(Debug)]
pub struct Parser {
    lexer: Lexer,
}

impl Parser {
    pub fn new(source_code: impl AsRef<str>) -> Self {
        Self {
            lexer: Lexer::new(source_code),
        }
    }
}

impl Iterator for Parser {
    type Item = Result<Ast>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lexer.eof() {
            Err(e) => return Some(Err(e)),
            Ok(true) => return None,
            Ok(false) => {}
        }
        match Ast::parse(&mut self.lexer) {
            Err(e) => Some(Err(e)),
            Ok(a) => Some(Ok(a)),
        }
    }
}

impl Parse for AtomToken {
    fn parse(lexer: &mut Lexer) -> Result<Self> {
        lexer.read_expect(ExpectAtom)
    }
}

impl Parse for VariableToken {
    fn parse(lexer: &mut Lexer) -> Result<Self> {
        lexer.read_expect(ExpectVariable)
    }
}

impl Parse for Symbol {
    fn parse(lexer: &mut Lexer) -> Result<Self> {
        lexer.read_expect(ExpectSymbol).map(|token| token.value())
    }
}
