#[derive(Copy, Clone)]
struct ParsingState<'a> {
    input: &'a str,
    line: usize,
    column: usize,
    index: usize,
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
type ParseResult<'a, T> = Result<(T, ParsingState<'a>), ParseError>;

enum Token {
    Import,
    Export,
    Identifier(String),
}

pub fn parse_module<'a>(state: ParsingState<'a>) -> ParseResult<'a, u8> {
    let state = ParsingState {
        line: 42,
        column: 32,
        input: state.input,
        index: 10000,
    };
    Ok((42, state))
}

pub fn parse<'a>(input: &'a str) -> ParseResult<'a, u8> {
    let state = ParsingState {
        input: &input,
        line: 1,
        column: 1,
        index: 0,
    };

    parse_module(state)
}

#[cfg(test)]
mod tests {
    #[test]
    fn something_runs() {
        for test in ["Hello", "World"] {
            assert_eq!(test, "Hello");
        }
    }
}
