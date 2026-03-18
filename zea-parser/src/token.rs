use crate::token::TokenError::UnexpectedEOF;
use std::marker::PhantomData;
use zea_macros::VariantToStr;

#[derive(Debug, PartialEq, VariantToStr)]
pub enum TokenError<'a> {
    UnexpectedEOF(Tokeniser<'a>),
    LiteralNotMatched(String, Tokeniser<'a>),
    InvalidValueIdentifier(String, Tokeniser<'a>),
    InvalidTypeIdentifier(String, Tokeniser<'a>),
    UnexpectedInput(String, Tokeniser<'a>),
    InvalidFloatLiteral(String, Tokeniser<'a>),
}

type TokenResult<'a, T> = Result<(T, Tokeniser<'a>), TokenError<'a>>;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Tokeniser<'source> {
    input: &'source str,
    line: usize,
    column: usize,
    index: usize,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Token<'source> {
    span: TokenSpan<'source>,
    kind: TokenKind<'source>,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct TokenSpan<'source> {
    phantom_data: PhantomData<&'source str>,
    pub start: usize,
    pub len: usize,
}
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TokenKind<'source> {
    ExprIdent(&'source str),
    TypeIdent(&'source str),
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    FnArrow,
    Module,
    Imports,
    Exports,
    Assign,
    Colon,
    While,
    Until,
    If,
    Else,
    Unless,
    Return,
    SemiColon,
    // For,
    // Match
}

impl<'source> Tokeniser<'source> {
    pub fn start(input: &'source str) -> Self {
        Self {
            input,
            index: 0,
            line: 1,
            column: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.index)
    }

    fn peek_n(&self, n: usize) -> Option<&'source str> {
        self.input.get(self.index..(self.index + n))
    }

    fn peek_remaining(self) -> &'source str {
        &self.input[self.index..]
    }
    fn peek_eaten(self) -> &'source str {
        &self.input[..self.index]
    }

    pub fn peek_diff(self, other: Tokeniser<'source>) -> &'source str {
        let start = self.index.min(other.index);
        let end = self.index.max(other.index);
        &self.input[start..end]
    }
    fn eat(self) -> TokenResult<'source, char> {
        if self.index >= self.input.len() {
            return Err(UnexpectedEOF(self));
        }

        let c = self.peek().unwrap();

        let (line, column) = if c == '\n' {
            (self.line + 1, 1)
        } else {
            (self.line, self.column + 1)
        };

        Ok((
            c,
            Tokeniser {
                line,
                column,
                index: self.index + 1,
                ..self
            },
        ))
    }

    /// return Ok(self) if there is any input remaining
    fn require_input(self) -> Result<Tokeniser<'source>, TokenError<'source>> {
        match self.eat() {
            Ok(..) => Ok(self),
            Err(_) => Err(UnexpectedEOF(self)),
        }
    }

    fn eat_ignore(self) -> Result<Tokeniser<'source>, TokenError<'source>> {
        if self.index >= self.input.len() {
            return Err(UnexpectedEOF(self));
        }

        let c = self.peek().unwrap();

        let (line, column) = if c == '\n' {
            (self.line + 1, 1)
        } else {
            (self.line, self.column + 1)
        };

        Ok(Tokeniser {
            line,
            column,
            index: self.index + 1,
            ..self
        })
    }
    fn whitespace(self) -> Self {
        let mut state = self;

        loop {
            match state.peek() {
                Some(c) if c.is_whitespace() => state = state.eat_ignore().unwrap(),
                _ => break,
            }
        }
        state
    }
    fn try_single_char_token(self) -> TokenResult<'source, Token<'source>> {
        match self.whitespace().eat() {
            Ok((c, t_char)) => {
                let (c, kind) = Token::validate_char(c)
                    .ok_or(TokenError::UnexpectedInput(c.to_string(), t_char))?;
                Ok((Token::new(TokenSpan::from_char(c, t_char), kind), t_char))
            }
            Err(e) => Err(e),
        }
    }
    fn is_valid_expr_identifier_char(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '-' || ch == '?' || ch == '!'
    }
    fn try_expr_identifier(self) -> TokenResult<'source, Token<'source>> {
        let mut state = self.require_input()?;
        loop {
            match state.eat() {
                Ok((ch, t_char)) if Self::is_valid_expr_identifier_char(ch) => {
                    state = t_char;
                }
                _ => break,
            }
        }
        let ident_str = self.peek_diff(state);
        let ident = Token::new(
            TokenSpan::from_str(ident_str, self),
            TokenKind::ExprIdent(ident_str),
        );

        Ok((ident, state))
    }

    fn is_valid_type_identifier_char(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_'
    }
    fn try_type_identifier(self) -> TokenResult<'source, Token<'source>> {
        let mut state = self.require_input()?;
        let invalid = self.peek().is_some_and(|ch| !ch.is_uppercase());
        loop {
            match state.eat() {
                Ok((ch, t_char)) if Self::is_valid_type_identifier_char(ch) => {
                    state = t_char;
                }
                _ => break,
            }
        }
        let ident_str = self.peek_diff(state);
        let ident = Token::new(
            TokenSpan::from_str(ident_str, self),
            TokenKind::ExprIdent(ident_str),
        );
        if invalid {
            Err(TokenError::InvalidValueIdentifier(
                ident_str.to_string(),
                state,
            ))
        } else {
            Ok((ident, state))
        }
    }
}

impl<'source> TokenSpan<'source> {
    pub fn from_str<'value: 'source>(
        token_value: &'value str,
        state: Tokeniser<'source>,
    ) -> TokenSpan<'source> {
        TokenSpan {
            phantom_data: PhantomData::default(),
            start: state.index,
            len: token_value.len(),
        }
    }
    pub fn from_char<'value: 'source>(
        token_value: char,
        state: Tokeniser<'source>,
    ) -> TokenSpan<'source> {
        TokenSpan {
            phantom_data: PhantomData::default(),
            start: state.index,
            len: 1,
        }
    }
}

impl<'source> Token<'source> {
    pub fn new(span: TokenSpan<'source>, kind: TokenKind<'source>) -> Self {
        Self { span, kind }
    }

    pub fn validate_char(ch: char) -> Option<(char, TokenKind<'source>)> {
        use TokenKind as T;
        match ch {
            '{' => Some((ch, T::OpenBrace)),
            '}' => Some((ch, T::CloseBrace)),
            '(' => Some((ch, T::OpenParen)),
            ')' => Some((ch, T::CloseParen)),
            ':' => Some((ch, T::Colon)),
            '=' => Some((ch, T::Assign)),
            ';' => Some((ch, T::Assign)),
            _ => None,
        }
    }
}
