use crate::token::reserved::NON_IDENTIFIER_CHARS;
use crate::token::TokenError::{UnexpectedEOF, UnexpectedInput};
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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Token<'source> {
    span: TokenSpan<'source>,
    kind: TokenKind<'source>,
}
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct TokenSpan<'source> {
    phantom_data: PhantomData<&'source str>,
    pub start: usize,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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
    Plus,
    Carot,
    OpenAngle,
    CloseAngle,
    Dash,
    Comma,
    Dot,
    Tilde,
    Star,
    At,
    Fence,
    Dollar,
    Pipe,
    LogXor,
    LogAnd,
    LogOr,
}

pub fn tokenise<'source>(src: &'source str) -> Vec<Token<'source>> {
    Tokeniser::start(src).collect()
}

mod reserved {
    use std::collections::HashSet;

    pub const KW_FUNC: &str = "fn";
    pub const KW_STRUCT: &str = "struct";
    pub const KW_TAGGED_UNION: &str = "enum";
    pub const KW_LOOP: &str = "for";
    pub const KW_IF: &str = "if";
    pub const KW_UNLESS: &str = "unless";
    pub const KW_WHILE: &str = "while";
    pub const KW_UNTIL: &str = "until";
    pub const KW_IMPORTS: &str = "imports";
    pub const KW_EXPORTS: &str = "exports";
    pub const KW_MODULE: &str = "module";
    pub const KW_RETURN: &str = "return";
    pub const KW_UNIT: &str = "void";
    pub const OP_ASSIGN: &str = "=";
    pub const OP_DEREF: &str = "@";

    pub const OP_REF: &str = "&";

    // pub const OP_CAST: &str = "as";
    pub const OP_LOG_OR: &str = "||";
    pub const OP_LOG_AND: &str = "&&";
    pub const OP_LOG_XOR: &str = "^^";
    pub const OP_LOG_NOT: &str = "~";

    pub const OP_BIT_OR: &str = "|";
    pub const OP_BIT_AND: &str = "&";
    pub const OP_BIT_XOR: &str = "^";
    pub const OP_BIT_NOT: &str = "~";

    pub const OP_PIPE: &str = "|>";
    pub const PIPE_HOLE: &str = "$";
    pub const FN_ARROW: &str = "->";

    const OPERATORS: [&str; 12] = [
        OP_LOG_AND, OP_LOG_XOR, OP_LOG_OR, OP_LOG_NOT, OP_BIT_AND, OP_BIT_NOT, OP_BIT_XOR,
        OP_BIT_OR, OP_DEREF, OP_PIPE, OP_REF, OP_ASSIGN,
    ];

    const MISC_SYMBOLS: &str = "(){}[]!";

    pub const NON_IDENTIFIER_CHARS: &str = "(){}[]<>!@#$^&-+,;:|~";
    pub fn operator_firsts() -> HashSet<char> {
        OPERATORS
            .iter()
            .map(|op| op.chars().nth(0).unwrap())
            .collect()
    }

    pub fn operator_lasts() -> HashSet<char> {
        OPERATORS
            .iter()
            .map(|op| op.chars().last().unwrap())
            .collect()
    }
    pub fn operator_chars() -> HashSet<char> {
        OPERATORS.iter().flat_map(|op| op.chars()).collect()
    }

    pub fn symbol_chars() -> HashSet<char> {
        NON_IDENTIFIER_CHARS.chars().collect()
    }
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
            match state.eat() {
                Ok((c, t_ws)) if c.is_whitespace() => state = t_ws,
                _ => break,
            }
        }
        state
    }
    fn try_single_char_token(self) -> TokenResult<'source, Token<'source>> {
        let state = self
            .whitespace()
            .require(|c| NON_IDENTIFIER_CHARS.contains(c))?;
        match state.eat() {
            Ok((c, t_char)) => {
                let (c, kind) = Token::validate_char(c)
                    .ok_or(TokenError::UnexpectedInput(c.to_string(), t_char))?;
                Ok((Token::new(TokenSpan::from_char(c, t_char), kind), t_char))
            }
            Err(e) => Err(e),
        }
    }
    fn is_valid_expr_identifier_char(ch: char) -> bool {
        ch.is_lowercase() || ch.is_numeric() || ch == '-' || ch == '?' || ch == '!'
    }
    fn try_expr_identifier(self) -> TokenResult<'source, Token<'source>> {
        let mut state = self.whitespace();
        let mut start = state.require(Self::is_valid_expr_identifier_char)?;
        loop {
            match state.eat() {
                Ok((ch, t_char)) if Self::is_valid_expr_identifier_char(ch) => {
                    state = t_char;
                }
                _ => break,
            }
        }
        let ident_str = start.peek_diff(state);
        let ident = Token::new(
            TokenSpan::from_str(ident_str, start),
            TokenKind::ExprIdent(ident_str),
        );
        Ok((ident, state))
    }

    fn is_valid_type_identifier_char(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_'
    }

    fn require(
        self,
        p: impl FnOnce(char) -> bool,
    ) -> Result<Tokeniser<'source>, TokenError<'source>> {
        match self.eat() {
            Ok((c, _)) if p(c) => Ok(self),
            Ok((c, invalid)) => Err(UnexpectedInput(c.to_string(), invalid)),
            Err(_) => Err(UnexpectedEOF(self)),
        }
    }
    fn try_type_identifier(self) -> TokenResult<'source, Token<'source>> {
        let mut start = self.whitespace();
        let mut state = start.require(|c| c.is_uppercase())?;
        loop {
            match state.eat() {
                Ok((ch, t_char)) if Self::is_valid_type_identifier_char(ch) => {
                    state = t_char;
                }
                _ => break,
            }
        }
        let ident_str = start.peek_diff(state);
        let ident = Token::new(
            TokenSpan::from_str(ident_str, start),
            TokenKind::TypeIdent(ident_str),
        );

        Ok((ident, state))
    }

    fn try_multi_char_token(self) -> TokenResult<'source, Token<'source>> {
        let start = self.whitespace();
        let mut state = start.require(|ch| NON_IDENTIFIER_CHARS.contains(ch))?;
        loop {
            match state.eat() {
                Ok((ch, p_non_ws)) if NON_IDENTIFIER_CHARS.contains(ch) => {
                    state = p_non_ws;
                }
                // Ok((ch, p_ws)) if ch.is_whitespace() => {
                //     state = p_ws;
                //     break;
                // }
                _ => break,
            }
        }
        assert!(state.index > start.index);

        let token_str = start.peek_diff(state);
        let kind = Token::validate_multi_char_token(token_str)
            .ok_or(UnexpectedInput(token_str.to_string(), start))?;

        let keyword = Token::new(TokenSpan::from_str(token_str, start), kind);
        Ok((keyword, state))
    }

    fn _next(self) -> TokenResult<'source, Token<'source>> {
        let state = self.require_input()?.whitespace();
        state
            .try_multi_char_token()
            .or_else(|_| state.try_single_char_token())
            .or_else(|_| state.try_expr_identifier())
            .or_else(|_| state.try_type_identifier())
    }
}

impl<'source> Iterator for Tokeniser<'source> {
    type Item = Token<'source>;
    fn next(&mut self) -> Option<Self::Item> {
        match self._next() {
            Ok((token, state)) => {
                self.index = state.index;
                self.line = state.line;
                self.column = state.column;
                Some(token)
            }
            Err(_) => None,
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
            line: state.line,
            column: state.column,
            length: token_value.len(),
        }
    }
    pub fn from_char<'value: 'source>(
        token_value: char,
        state: Tokeniser<'source>,
    ) -> TokenSpan<'source> {
        TokenSpan {
            phantom_data: PhantomData::default(),
            start: state.index,
            line: state.line,
            column: state.column,
            length: 1,
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
            '+' => Some((ch, T::Plus)),
            '-' => Some((ch, T::Dash)),
            '<' => Some((ch, T::OpenAngle)),
            '>' => Some((ch, T::CloseAngle)),
            '|' => Some((ch, T::Pipe)),
            '$' => Some((ch, T::Dollar)),
            '#' => Some((ch, T::Fence)),
            '@' => Some((ch, T::At)),
            '*' => Some((ch, T::Star)),
            '~' => Some((ch, T::Tilde)),
            '.' => Some((ch, T::Dot)),
            ',' => Some((ch, T::Comma)),
            '^' => Some((ch, T::Carot)),
            _ => None,
        }
    }

    pub fn validate_multi_char_token(keyword: &'source str) -> Option<TokenKind<'source>> {
        use TokenKind as T;
        match keyword {
            reserved::KW_IMPORTS => Some(T::Imports),
            reserved::KW_MODULE => Some(T::Module),
            reserved::KW_RETURN => Some(T::Return),
            reserved::KW_IF => Some(T::If),
            reserved::FN_ARROW => Some(T::FnArrow),
            reserved::KW_WHILE => Some(T::While),
            reserved::OP_LOG_XOR => Some(T::LogXor),
            reserved::OP_LOG_AND => Some(T::LogAnd),
            reserved::OP_LOG_OR => Some(T::LogOr),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::token::TokenKind::{Dash, FnArrow};
    use crate::token::{Token, TokenKind, Tokeniser};

    fn first_token<'a>(s: &'a str) -> Token<'a> {
        Tokeniser::start(s).next().unwrap()
    }

    fn kinds<'a>(s: &'a str) -> Vec<TokenKind> {
        Tokeniser::start(s).map(|tok| tok.kind).collect()
    }

    #[test]
    fn eat_advances() {
        let s = "bob";
        let start = Tokeniser::start(s);
        let mut ts = start;
        let bob = ts.next().unwrap();
        assert_eq!(start.index + 3, ts.index);
    }
    #[test]
    fn idents_cannot_be_empty() {
        let s = "";
        let mut ts = Tokeniser::start(s);
        match ts.next() {
            Some(t) => panic!(),
            None => {}
        }

        let s = "bob";
        let mut ts = Tokeniser::start(s);
        match ts.next() {
            Some(t) => assert_eq!(t.span.length, 3),
            None => panic!("expected token"),
        }

        let s = "bob @";
        let mut ts = Tokeniser::start(s);
        let _bob = ts.next();
        match ts.next() {
            Some(at) => assert_eq!(at.span.length, 1),
            None => panic!("expected token"),
        }
    }

    #[test]
    fn ambiguous_tokens() {
        use TokenKind::*;
        assert_eq!(kinds("->"), vec![FnArrow]);
        assert_eq!(kinds("- >"), vec![Dash, CloseAngle]);
        assert_eq!(kinds("-->"), vec![Dash, FnArrow]);
        assert_eq!(kinds("^^"), vec![LogXor]);
        assert_eq!(kinds("^ ^"), vec![Carot, Carot]);
        assert_eq!(kinds("^^^"), vec![Carot, LogXor]);
        assert_eq!(kinds("a-b"), vec![ExprIdent("a-b")]);
        assert_eq!(kinds("a - b"), vec![ExprIdent("a"), Dash, ExprIdent("b")]);
    }
    #[test]
    fn token_full() {}
}
