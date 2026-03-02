#[derive(Copy, Clone)]
struct ParseState<'a> {
    input: &'a str,
    line: usize,
    column: usize,
    index: usize,
}

#[derive(Debug)]
enum ParseError {
    KeywordNotMatched,
    IdentifierNotMatched,
}

type ParseResult<'a, T> = Result<(T, ParseState<'a>), ParseError>;

impl<'a> ParseState<'a> {
    pub fn new(input: &'a str) -> ParseState<'a> {
        ParseState {
            input,
            line: 1,
            column: 1,
            index: 0,
        }
    }

    pub fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.index)
    }

    pub fn eat(&self) -> Option<ParseState<'a>> {
        if self.index >= self.input.len() {
            return None;
        }

        let (line, column) = if self.peek().unwrap() == '\n' {
            (self.line + 1, 1)
        } else {
            (self.line, self.column + 1)
        };

        Some(ParseState {
            input: self.input,
            line,
            column,
            index: self.index + 1,
        })
    }

    pub fn eat_bigly(&self, n: usize) -> Option<ParseState<'a>> {
        let mut state = *self;
        for _ in 0..n {
            state = state.eat()?
        }
        Some(state)
    }

    pub fn skip_whitespace(&self) -> ParseState<'a> {
        let mut state = *self;

        while let Some(c) = state.peek() {
            if c.is_whitespace() {
                match state.eat() {
                    None => break,
                    Some(new) => state = new,
                }
            } else {
                break;
            }
        }

        state
    }

    pub fn starts_with(&self, s: &str) -> bool {
        match self.input.get(self.index..) {
            None => false,
            Some(slice) => slice.starts_with(s),
        }
    }

    pub fn parse_keyword(&self, keyword: &str) -> Result<ParseState<'a>, ParseError> {
        if !self.starts_with(keyword) {
            return Err(ParseError::KeywordNotMatched);
        }

        let state = self.eat_bigly(keyword.len()).unwrap();
        match state.peek() {
            None => Ok(state), // we reached the end
            Some(c) => {
                if c.is_alphanumeric() {
                    Err(ParseError::KeywordNotMatched)
                } else {
                    Ok(state)
                }
            }
        }
    }

    pub fn parse_identifier(&self) -> ParseResult<'a, String> {
        match self.peek() {
            None => return Err(ParseError::IdentifierNotMatched),
            Some(c) => {
                if !c.is_alphanumeric() {
                    return Err(ParseError::IdentifierNotMatched);
                }
            }
        }

        let mut state = *self;
        let start_index = state.index;

        while let Some(c) = state.peek() {
            if c.is_alphanumeric() {
                state = state.eat().unwrap()
            } else {
                break;
            }
        }

        let identifier = self.input[start_index..state.index].to_string();

        Ok((identifier, state))
    }

    pub fn parse_import_statement(&self) -> ParseResult<'a, Vec<String>> {
        let state = *self;
        let state = state.parse_keyword("import")?;
        let state = state.skip_whitespace();
        let (identifier, state) = state.parse_identifier()?;

        Ok((vec![identifier], state))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_state_new() {
        let state = ParseState::new("Hello");
        assert_eq!(0, state.index);
        assert_eq!(1, state.line);
        assert_eq!(1, state.column);
    }

    #[test]
    fn test_parse_state_peek() {
        let state = ParseState::new("abc");
        assert_eq!(Some('a'), state.peek());

        let state = ParseState::new("");
        assert_eq!(None, state.peek());
    }

    #[test]
    fn test_parse_state_eat() {
        let state = ParseState::new("abc");
        assert_eq!(Some('a'), state.peek());
        let state = state.eat().unwrap();
        assert_eq!(Some('b'), state.peek());
        let state = state.eat().unwrap();
        assert_eq!(Some('c'), state.peek());
        let state = state.eat().unwrap();
        assert_eq!(None, state.peek());
    }

    #[test]
    fn test_parse_keyword() {
        let state = ParseState::new("import abc");
        let state = state.parse_keyword("import").unwrap();

        assert_eq!(6, state.index);
        assert_eq!(7, state.column);
        assert_eq!(1, state.line);
    }

    #[test]
    fn test_parse_identifier() {
        let state = ParseState::new("xyz abracadabra");
        let (identifier, state) = state.parse_identifier().unwrap();

        assert_eq!("xyz", identifier);
        assert_eq!(3, state.index);
        assert_eq!(4, state.column);
        assert_eq!(1, state.line);
    }

    #[test]
    fn test_parse_import_statement() {
        let state = ParseState::new("import io");
        match state.parse_import_statement() {
            Ok((identifiers, _state)) => assert_eq!(vec![String::from("io")], identifiers),
            Err(error) => panic!("{:?}", error),
        }
    }
}
