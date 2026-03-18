use crate::{ParseError, ParseState};
use std::fmt::{Display, Formatter};

impl<'a> ParseError<'a> {
    fn input_up_to_index(&self) -> &'a str {
        match self {
            ParseError::InvalidFloatLiteral(_, state) => state.peek_parsed(),
            ParseError::UnexpectedInput(_, state) => state.peek_parsed(),
            ParseError::InvalidTypeIdentifier(_, state) => state.peek_parsed(),
            ParseError::UnexpectedEOF(state) => state.peek_parsed(),
            ParseError::LiteralNotMatched(_, state) => state.peek_parsed(),
            ParseError::InvalidValueIdentifier(_, state) => state.peek_parsed(),
        }
    }

    fn state(&self) -> &ParseState<'a> {
        match self {
            ParseError::InvalidFloatLiteral(_, state) => state,
            ParseError::UnexpectedInput(_, state) => state,
            ParseError::InvalidTypeIdentifier(_, state) => state,
            ParseError::UnexpectedEOF(state) => state,
            ParseError::LiteralNotMatched(_, state) => state,
            ParseError::InvalidValueIdentifier(_, state) => state,
        }
    }

    fn decorate_nth_line(&self, n: usize) -> (String, usize) {
        let state = self.state();
        let lines: Vec<_> = state.input.lines().collect();
        // Source - https://stackoverflow.com/a/69298721
        // Posted by Daniel, modified by community. See post 'Timeline' for change history
        // Retrieved 2026-03-17, License - CC BY-SA 4.0
        let n_digits = n.checked_ilog10().unwrap_or(0) as usize + 1;
        let decoration_length = n_digits + 2;

        (format!("{}: {}", state.line, lines[n]), decoration_length)
    }

    fn decorate_input(&self) -> String {
        let state = self.state();
        let column = state.column;
        let (line, line_decoration_length) = self.decorate_nth_line(state.line - 1);
        let decoration = "_".repeat(column - 1 + line_decoration_length) + "^";
        format!(
            "{line}\n{decoration}\n{}\nat {}:{}",
            self.variant_as_str(),
            state.line,
            state.column
        )
    }
}

impl<'a> Display for ParseError<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.decorate_input())
    }
}
