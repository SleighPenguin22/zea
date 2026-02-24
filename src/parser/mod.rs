use crate::ast::ZeaExpression;
use crate::ast::Literal;

struct ParsingState<'a> {
    input: &'a str,
    line: usize,
    column: usize,
    index: usize
}

struct ParseError {
    line: usize,
    column: usize,
    message: String,
}

/// We would like the state to NOT be mutable, i.e. we won't be consuming
/// tokens and putting them back. This is cumbersome if we want to put a token
/// back and get the last column and line state.
///
/// All our recursive-descent parsing function return this struct. It is either:
/// 
///    1. A parsing error, together with the line, column and a custom message.
///    2. A tuple consisting of the desired result (integer, AST node etc.)
///       and the new parsing state.
type ParseResult<'a, T> =  Result<(T, ParsingState<'a>), ParseError>;

enum Token {
    Import,
    Export,
    Identifier(String),
}

impl<'a> ParsingState<'a> {
    pub fn parse(&self) -> ParseResult<'a, u8> {
        let state = ParsingState { line: 42, column: 32, input: self.input, index: 10000 };
        Ok((42, state))
    }

    pub fn peek(&self) -> char {
        self.input[self.index];
    }

    pub fn advance(&self) -> Option<ParsingState<'a>> {
        assert!(self.index < self.input.len());

        if self.index == self.input.len() {
            return None;
        }

        let (line, column) = if self.peek().is_newline() {
            (self.line + 1, 1)
        } else {
            (self.line, self.column + 1)
        }

        ParsingState { self.input, line, column, self.index + 1 }
    }
}

pub fn parse<'a>(input: &'a str) -> ParseResult<'a, u8> {
    let state = ParsingState {
        input: &input,
        line: 1,
        column: 1,
        index: 0,
    };

    state.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn something_runs() {
        for test in [
            "Hello",
            "World",
        ] {
            assert_eq!(test, "Hello");
        }
    }
}
