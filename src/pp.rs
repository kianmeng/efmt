use crate::expect::{Either, Expect, ExpectAtom, ExpectVariable, Or};
use erl_tokenize::tokens::{AtomToken, CommentToken, VariableToken};
use erl_tokenize::values::{Keyword, Symbol};
use erl_tokenize::{LexicalToken, Position, PositionRange, Token, Tokenizer};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unexpected EOF")]
    UnexpectedEof,

    #[error("macro {name:?} is not defined ({position:?})")]
    UndefinedMacro { name: String, position: Position },

    #[error("badly formed argument ({position:?})")]
    MalformedMacroArg { position: Position },

    #[error(transparent)]
    TokenizeError(#[from] erl_tokenize::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug)]
pub struct Preprocessor {
    tokenizer: Tokenizer<String>,
    macro_defines: HashMap<String, MacroDefine>,
    preprocessed: Preprocessed,
}

impl Preprocessor {
    // TODO: user defined predefined macros
    pub fn new(tokenizer: Tokenizer<String>) -> Self {
        let preprocessed = Preprocessed {
            file: tokenizer.next_position().filepath().cloned(),
            text: tokenizer.text().to_owned(),
            tokens: Vec::new(),
            comments: BTreeMap::new(),
            macro_calls: BTreeMap::new(),
        };
        Self {
            tokenizer,
            macro_defines: MacroDefine::predefined(),
            preprocessed,
        }
    }

    fn try_handle_directives(&mut self) -> Result<()> {
        if let Some(LexicalToken::Atom(token)) = self.next_lexical_token()? {
            match token.value() {
                "define" => {
                    let define = self.parse_define()?;
                    self.macro_defines.insert(define.name().to_owned(), define);
                }
                "include" => todo!(),
                "include_lib" => todo!(),
                _ => {}
            }
        }
        Ok(())
    }

    fn parse_define(&mut self) -> Result<MacroDefine> {
        let _ = self.read_expect(Symbol::OpenParen)?;
        let name = self.read_expect(Or(ExpectAtom, ExpectVariable))?;
        let params = if self
            .read_expect(Or(Symbol::Comma, Symbol::OpenParen))?
            .is_a()
        {
            None
        } else {
            let mut params = Vec::new();
            while let Some(param) = self.try_read_expect(ExpectVariable) {
                params.push(param);
                if self.try_read_expect(Symbol::Comma).is_none() {
                    break;
                }
            }
            let _ = self.read_expect(Symbol::CloseParen)?;
            let _ = self.read_expect(Symbol::Comma)?;
            Some(params)
        };
        let mut replacement = Vec::new();
        let mut level = 0;
        while let Some(token) = self.next_lexical_token()? {
            match &token {
                LexicalToken::Symbol(x) if x.value() == Symbol::OpenParen => {
                    level += 1;
                }
                LexicalToken::Symbol(x) if x.value() == Symbol::CloseParen => {
                    if level == 0 {
                        break;
                    }
                    level -= 1;
                }
                _ => {}
            }
            replacement.push(token);
        }
        let _ = self.read_expect(Symbol::Dot)?;
        Ok(MacroDefine {
            name,
            params,
            replacement,
        })
    }

    fn try_read_expect<T: Expect>(&mut self, expected: T) -> Option<T::Token> {
        let position = self.tokenizer.next_position();
        if let Some(token) = self.read_expect(expected).ok() {
            Some(token)
        } else {
            self.tokenizer.set_position(position);
            None
        }
    }

    fn read_expect<T: Expect>(&mut self, expected: T) -> Result<T::Token> {
        if let Some(token) = self.next_lexical_token()? {
            expected.expect(token).map_err(|token| {
                anyhow::anyhow!("expected {:?}, but got {:?}", expected, token).into()
            })
        } else {
            Err(Error::UnexpectedEof)
        }
    }

    fn next_lexical_token(&mut self) -> Result<Option<LexicalToken>> {
        while let Some(token) = self.tokenizer.next().transpose()? {
            match token {
                Token::Whitespace(_) => {}
                Token::Comment(x) => {
                    if self.preprocessed.file.as_ref() == x.start_position().filepath() {
                        self.preprocessed.comments.insert(x.start_position(), x);
                    }
                }
                Token::Symbol(x) => return Ok(Some(x.into())),
                Token::Atom(x) => return Ok(Some(x.into())),
                Token::Char(x) => return Ok(Some(x.into())),
                Token::Float(x) => return Ok(Some(x.into())),
                Token::Integer(x) => return Ok(Some(x.into())),
                Token::Keyword(x) => return Ok(Some(x.into())),
                Token::String(x) => return Ok(Some(x.into())),
                Token::Variable(x) => return Ok(Some(x.into())),
            }
        }
        Ok(None)
    }

    pub fn preprocess(mut self) -> Result<Preprocessed> {
        while let Some(token) = self.next_lexical_token()? {
            if let LexicalToken::Symbol(x) = &token {
                if x.value() == Symbol::Question {
                    self.expand_macro()?;
                    continue;
                } else if x.value() == Symbol::Hyphen {
                    self.try_handle_directives()?;
                    self.tokenizer.set_position(x.end_position());
                }
            }
            self.preprocessed.tokens.push(token);
        }
        Ok(self.preprocessed)
    }

    fn expand_macro(&mut self) -> Result<()> {
        let position = self.tokenizer.next_position();
        let name = match self.read_expect(Or(ExpectAtom, ExpectVariable))? {
            Either::A(x) => x.value().to_owned(),
            Either::B(x) => x.value().to_owned(),
        };
        let define =
            self.macro_defines
                .get(&name)
                .cloned()
                .ok_or_else(|| Error::UndefinedMacro {
                    name: name.clone(),
                    position: position.clone(),
                })?;

        if let Some(params) = &define.params {
            let _ = self.read_expect(Symbol::OpenParen)?;
            let mut args = HashMap::new();
            for (i, param) in params.iter().enumerate() {
                let arg = self.parse_macro_arg()?;
                args.insert(param.value(), arg);
                if i + 1 < params.len() {
                    self.read_expect(Symbol::Comma)?;
                }
            }
            let _ = self.read_expect(Symbol::CloseParen)?;
            self.preprocessed
                .tokens
                .extend(define.replace_variables(&args));
        } else {
            self.preprocessed
                .tokens
                .extend(define.replacement.iter().cloned());
        }
        Ok(())
    }

    fn parse_macro_arg(&mut self) -> Result<Vec<LexicalToken>> {
        #[derive(Debug, Default)]
        struct Level {
            paren: usize,
            brace: usize,
            square: usize,
            block: usize,
            // TODO: '<<''>>'
        }

        impl Level {
            fn is_top_level(&self) -> bool {
                (self.paren + self.brace + self.square + self.block) == 0
            }
        }

        let mut level = Level::default();
        let mut tokens = Vec::new();
        while let Some(token) = self.next_lexical_token()? {
            let position = token.start_position();
            match &token {
                LexicalToken::Symbol(x) if x.value() == Symbol::Comma && level.is_top_level() => {
                    self.tokenizer.set_position(token.start_position());
                    break;
                }
                LexicalToken::Symbol(x)
                    if x.value() == Symbol::CloseParen && level.is_top_level() =>
                {
                    self.tokenizer.set_position(token.start_position());
                    break;
                }
                LexicalToken::Symbol(x) if x.value() == Symbol::Dot => {
                    return Err(Error::MalformedMacroArg { position });
                }
                LexicalToken::Symbol(x) if x.value() == Symbol::OpenParen => {
                    level.paren += 1;
                }
                LexicalToken::Symbol(x) if x.value() == Symbol::CloseParen => {
                    if level.paren == 0 {
                        return Err(Error::MalformedMacroArg { position });
                    }
                    level.paren -= 1;
                }
                LexicalToken::Symbol(x) if x.value() == Symbol::OpenBrace => {
                    level.brace += 1;
                }
                LexicalToken::Symbol(x) if x.value() == Symbol::CloseBrace => {
                    if level.brace == 0 {
                        return Err(Error::MalformedMacroArg { position });
                    }
                    level.brace -= 1;
                }
                LexicalToken::Symbol(x) if x.value() == Symbol::OpenSquare => {
                    level.square += 1;
                }
                LexicalToken::Symbol(x) if x.value() == Symbol::CloseSquare => {
                    if level.square == 0 {
                        return Err(Error::MalformedMacroArg { position });
                    }
                    level.square -= 1;
                }
                LexicalToken::Keyword(x)
                    if x.value() == Keyword::Begin
                        && x.value() == Keyword::Try
                        && x.value() == Keyword::Fun
                        && x.value() == Keyword::Case
                        && x.value() == Keyword::If =>
                {
                    level.block += 1;
                }
                LexicalToken::Keyword(x) if x.value() == Keyword::End => {
                    if level.block == 0 {
                        return Err(Error::MalformedMacroArg { position });
                    }
                    level.block -= 1;
                }
                _ => {}
            }
            tokens.push(token);
        }

        Ok(tokens)
    }
}

#[derive(Debug)]
pub struct Preprocessed {
    pub file: Option<PathBuf>,
    pub text: String,
    pub tokens: Vec<LexicalToken>,
    pub comments: BTreeMap<Position, CommentToken>,
    pub macro_calls: BTreeMap<Position, MacroCall>,
}

#[derive(Debug, Clone)]
pub struct MacroDefine {
    name: Either<AtomToken, VariableToken>,
    params: Option<Vec<VariableToken>>,
    replacement: Vec<LexicalToken>,
}

impl MacroDefine {
    pub fn predefined() -> HashMap<String, Self> {
        [
            "MODULE",
            "MODULE_STRING",
            "FILE",
            "LINE",
            "MACHINE",
            "FUNCTION_NAME",
            "FUNCTION_ARITY",
            "OTP_RELEASE",
        ]
        .into_iter()
        .map(|name| {
            (
                name.to_owned(),
                MacroDefine {
                    name: Either::B(VariableToken::from_value(name, Position::new()).expect("bug")),
                    params: None,
                    replacement: vec![AtomToken::from_value("dummy", Position::new()).into()],
                },
            )
        })
        .collect()
    }

    pub fn name(&self) -> &str {
        match &self.name {
            Either::A(x) => x.value(),
            Either::B(x) => x.value(),
        }
    }

    pub fn replace_variables(&self, args: &HashMap<&str, Vec<LexicalToken>>) -> Vec<LexicalToken> {
        let mut tokens = Vec::new();
        for token in self.replacement.iter().cloned() {
            match &token {
                LexicalToken::Variable(x) if args.contains_key(x.value()) => {
                    tokens.extend(args[x.value()].iter().cloned());
                    continue;
                }
                _ => {}
            }
            tokens.push(token);
        }
        tokens
    }
}

#[derive(Debug)]
pub struct MacroCall {}
