#![allow(unused)]

use crate::ParseError::{InvalidValueIdentifier, UnexpectedEOF};
use zea_ast::zea;
use zea_ast::zea::Type::Pointer;
use zea_ast::zea::{Type, TypedIdentifier};

const KW_FUNC: &str = "fn";
const KW_STRUCT: &str = "struct";
const KW_TAGGED_UNION: &str = "enum";

#[derive(Default, Clone, Copy)]
struct NodeIdGenerator {
    cur: usize,
}
impl NodeIdGenerator {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get(&mut self) -> usize {
        let cur = self.cur;
        self.cur += 1;
        cur
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct ParseState<'a> {
    input: &'a str,
    line: usize,
    column: usize,
    index: usize,
}

#[derive(Debug, PartialEq)]
enum ParseError<'a> {
    UnexpectedEOF,
    LiteralNotMatched(String, ParseState<'a>),
    InvalidValueIdentifier(String, ParseState<'a>),
    InvalidTypeIdentifier(String, ParseState<'a>),
    UnexpectedInput(String, ParseState<'a>),
}

type ParseResult<'a, T> = Result<(T, ParseState<'a>), ParseError<'a>>;

impl<'a> ParseState<'a> {
    pub fn new(input: &'a str) -> ParseState<'a> {
        ParseState {
            input,
            line: 1,
            column: 1,
            index: 0,
        }
    }

    fn peek(self) -> Option<char> {
        self.input.chars().nth(self.index)
    }

    fn eat(self) -> Option<ParseState<'a>> {
        if self.index >= self.input.len() {
            return None;
        }

        let (line, column) = if self.peek().unwrap() == '\n' {
            (self.line + 1, 1)
        } else {
            (self.line, self.column + 1)
        };

        Some(ParseState {
            line,
            column,
            index: self.index + 1,
            ..self
        })
    }

    fn eat_bigly(self, n: usize) -> Option<(&'a str, ParseState<'a>)> {
        let mut state = self;
        for _ in 0..n {
            state = state.eat()?
        }

        let (eaten, _rest) = self.input.split_at(n);

        Some((eaten, state))
    }

    fn whitespace(self) -> ParseState<'a> {
        let mut state = self;

        loop {
            match state.peek() {
                Some(c) if c.is_whitespace() => state = state.eat().unwrap(),
                _ => break,
            }
        }

        state
    }

    fn digit(self, radix: u8) -> ParseResult<'a, u64> {
        if radix < 2 || radix > 32 {
            panic!("invalid radix {radix}")
        }
        let state = self.eat_bigly(1);
        match state {
            None => Err(UnexpectedEOF),
            Some((d, state)) => {
                let d = u64::from_str_radix(d, radix as u32)
                    .map_err(|_| ParseError::UnexpectedInput(d.to_string(), self))?;
                Ok((d, state))
            }
        }
    }

    pub fn starts_with(self, s: &str) -> bool {
        match self.input.get(self.index..) {
            None => false,
            Some(slice) => slice.starts_with(s),
        }
    }
    pub fn toklit(self, keyword: &'a str) -> ParseResult<'a, &'a str> {
        if !self.starts_with(keyword) {
            return Err(ParseError::LiteralNotMatched(keyword.to_string(), self));
        }

        Ok(self.eat_bigly(keyword.len()).unwrap())
    }

    pub fn parse_expr_identifier(self) -> ParseResult<'a, String> {
        let invalid = match self.peek() {
            None => return Err(UnexpectedEOF),
            Some(c) => !c.is_lowercase(),
        };

        let mut state = self;
        let start_index = state.index;

        while let Some(c) = state.peek() {
            if c.is_alphanumeric() || c == '?' || c == '!' || c == '-' {
                state = state.eat().unwrap()
            } else {
                break;
            }
        }

        let identifier = self.input[start_index..state.index].to_string();
        if invalid {
            Err(InvalidValueIdentifier(identifier, self))
        } else {
            Ok((identifier, state))
        }
    }

    pub fn parse_type_identifier(self) -> ParseResult<'a, String> {
        let invalid = match self.peek() {
            None => {
                return Err(UnexpectedEOF);
            }
            Some(c) => !c.is_uppercase(),
        };

        let mut state = self;
        let start_index = state.index;

        while let Some(c) = state.peek() {
            if c.is_alphanumeric() {
                state = state.eat().unwrap();
            } else {
                break;
            }
        }

        let identifier = self.input[start_index..state.index].to_string();
        if invalid {
            Err(ParseError::InvalidTypeIdentifier(identifier, self))
        } else {
            Ok((identifier, state))
        }
    }

    pub fn parse_import_statement(self) -> ParseResult<'a, Vec<String>> {
        let (_, state) = self.toklit("import")?;
        let state = state.whitespace();
        let (identifier, state) = state.parse_expr_identifier()?;

        Ok((vec![identifier], state))
    }

    pub fn open_paren(self) -> Result<ParseState<'a>, ParseError<'a>> {
        let (_, state) = self.toklit("(")?;
        Ok(state)
    }

    pub fn close_paren(self) -> Result<ParseState<'a>, ParseError<'a>> {
        let (_, state) = self.toklit(")")?;
        Ok(state)
    }

    pub fn comma(self) -> Result<ParseState<'a>, ParseError<'a>> {
        let (_, state) = self.toklit(",")?;
        Ok(state)
    }

    pub fn colon(self) -> Result<ParseState<'a>, ParseError<'a>> {
        let (_, state) = self.toklit(":")?;
        Ok(state)
    }

    pub fn fn_arrow(self) -> Result<ParseState<'a>, ParseError<'a>> {
        let (_, state) = self.toklit("->")?;
        Ok(state)
    }

    pub fn kw_func(self) -> Result<ParseState<'a>, ParseError<'a>> {
        let (_, state) = self.toklit(KW_FUNC)?;
        Ok(state)
    }

    fn parse_pointer_type(self) -> ParseResult<'a, Type> {
        let state = self;
        let (ident, mut state) = self.parse_type_identifier()?;
        let mut res = Type::Basic(ident);

        loop {
            if let Ok((_, parsed_pointer)) = state.toklit("*") {
                res = Type::Pointer(Box::new(res));
                state = parsed_pointer;
                continue;
            } else {
                break;
            }
        }
        Ok((res, state))
    }

    fn parse_array_type(self) -> ParseResult<'a, Type> {
        let (_, state) = self.whitespace().toklit("[")?;

        let (typ, state): (Type, ParseState<'a>) = state.parse_type()?;
        let state: ParseState<'a> = state.whitespace();

        let (_, state) = state.toklit("]")?;
        let state = state.whitespace();
        let typ = Type::ArrayOf(Box::new(typ));
        Ok((typ, state))
    }

    fn parse_type(self) -> ParseResult<'a, zea::Type> {
        if let Ok(_) = self.whitespace().toklit("[") {
            self.parse_array_type()
        } else {
            self.parse_pointer_type()
        }
    }

    pub fn parse_func_param(self) -> ParseResult<'a, TypedIdentifier> {
        let (ident, state) = self.whitespace().parse_expr_identifier()?;
        let state = state.whitespace();
        let state = state.colon()?;
        let state = state.whitespace();
        let (typ, state) = state.parse_type()?;
        let state = state.whitespace();
        Ok((TypedIdentifier::new(typ, ident), state))
    }

    pub fn parse_func_param_list(self) -> ParseResult<'a, Vec<TypedIdentifier>> {
        let mut state = self.whitespace();
        let mut res = vec![];

        loop {
            if let Ok((param, parsed_param)) = state.parse_func_param() {
                res.push(param);
                state = parsed_param.whitespace();

                if let Ok(p_comma) = state.comma() {
                    state = p_comma.whitespace();
                    continue;
                }
            }
            break;
        }

        Ok((res, state))
    }

    pub fn parse_func_head(self) -> ParseResult<'a, (String, Vec<TypedIdentifier>, Type)> {
        let state = self.whitespace().kw_func()?;
        let (name, state) = state.whitespace().parse_expr_identifier()?;
        let state = state.whitespace().open_paren()?;
        let (params, state) = state.whitespace().parse_func_param_list()?;
        let mut state = state.whitespace().close_paren()?;

        let returns = match state.whitespace().fn_arrow() {
            Ok(p_arrow) => {
                let (returns, p_type) = p_arrow.whitespace().parse_type()?;
                state = p_type.whitespace();
                returns
            }
            _ => Type::Basic("Void".to_string()),
        };
        let res = (name, params, returns);
        Ok((res, state))
    }

    pub fn parse_expr_lit_int(self) -> ParseResult<'a, u64> {
        let state = self.whitespace();
        let mut total: u64 = 0;
        let (radix, mut state) = match self.eat_bigly(2) {
            Some((c, p_prefix)) if c.eq_ignore_ascii_case("0x") => (16, p_prefix),
            Some((c, p_prefix)) if c.eq_ignore_ascii_case("0b") => (2, p_prefix),
            _ => (10, state),
        };

        loop {
            if let Ok((digit, p_digit)) = state.digit(radix) {
                total *= radix as u64;
                total += digit;
                state = p_digit;
                continue;
            }

            if let Ok((_, p_underscore)) = self.toklit("_") {
                state = p_underscore;
                continue;
            }
            break;
        }
        Ok((total, state))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    mod types {
        use zea_ast::zea::{Type, TypedIdentifier};

        macro_rules! typ {
            ($t:ident) => {
                typ(stringify!($t).to_string())
            };

            ($t:ident*) => {
                ptr(typ!(t))
            };
            ([$t:ident]) => {
                arr(typ!(t))
            };
        }

        macro_rules! typed_ident {
            ($t:ident: $i:ident) => {
                TypedIdentifier::new()
            };
        }

        pub fn typ(typ: &str) -> Type {
            Type::Basic(typ.to_string())
        }

        pub fn ptr(typ: Type) -> Type {
            Type::Pointer(Box::new(typ))
        }

        pub fn arr(typ: Type) -> Type {
            Type::ArrayOf(Box::new(typ))
        }
    }

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
    fn test_parse_literal() {
        let state = ParseState::new("import abc");
        let (_, state) = state.toklit("import").unwrap();

        assert_eq!(6, state.index);
        assert_eq!(7, state.column);
        assert_eq!(1, state.line);
    }

    #[test]
    fn test_parse_identifier() {
        let state = ParseState::new("xyz abracadabra");
        let (identifier, state) = state.parse_expr_identifier().unwrap();

        assert_eq!("xyz", identifier);

        let state = ParseState::new("xyz? bob");
        let (identifier, state) = state.parse_expr_identifier().unwrap();

        assert_eq!("xyz?", identifier);

        let state = ParseState::new("xyz! bob");
        let (identifier, state) = state.parse_expr_identifier().unwrap();

        assert_eq!("xyz!", identifier);

        let state = ParseState::new("xyz!?? bob");
        let (identifier, state) = state.parse_expr_identifier().unwrap();

        assert_eq!("xyz!??", identifier);

        let state = ParseState::new("bob-is-cool");
        let (identifier, state) = state.parse_expr_identifier().unwrap();

        assert_eq!("bob-is-cool", identifier);
        let state = ParseState::new("is-even? no its not bro");
        let (identifier, state) = state.parse_expr_identifier().unwrap();

        assert_eq!("is-even?", identifier);
    }

    #[test]
    fn test_parse_type() {
        use types::*;

        let (typ_, _) = ParseState::new("Int").parse_type().unwrap();
        assert_eq!(typ_, typ("Int"));

        let (typ_, _) = ParseState::new("Int*").parse_type().unwrap();
        assert_eq!(typ_, ptr(typ("Int")));

        let (typ_, _) = (ParseState::new("[Int]").parse_type()).unwrap();
        assert_eq!(typ_, arr(typ("Int")));

        let (typ_, _) = (ParseState::new("[[Int]]").parse_type()).unwrap();
        assert_eq!(typ_, arr(arr(typ("Int"))));

        let (typ_, _) = (ParseState::new("[Int*]").parse_type()).unwrap();
        assert_eq!(typ_, arr(ptr(typ("Int"))));

        let (typ_, _) = (ParseState::new("[Int**]").parse_type()).unwrap();
        assert_eq!(typ_, arr(ptr(ptr(typ("Int")))));

        let state = ParseState::new("[Int");
        let (_, expstate) = state.eat_bigly(4).unwrap();
        let err = state.parse_type().unwrap_err();
        assert_eq!(
            ParseError::LiteralNotMatched("]".to_string(), expstate),
            err
        );

        let state = ParseState::new("int");
        let err = state.parse_type().unwrap_err();
        assert_eq!(
            ParseError::InvalidTypeIdentifier("int".to_string(), state),
            err
        );
    }

    #[test]
    fn test_parse_func_param() {
        use types::*;
        let (param, _) = ParseState::new("a : Int").parse_func_param().unwrap();
        assert_eq!(TypedIdentifier::new(typ("Int"), "a"), param);

        let (param, _) = ParseState::new("a : Int*").parse_func_param().unwrap();
        assert_eq!(TypedIdentifier::new(ptr(typ("Int")), "a"), param);

        let (param, _) = ParseState::new("a? : Int*").parse_func_param().unwrap();
        assert_eq!(TypedIdentifier::new(ptr(typ("Int")), "a?"), param);

        let state = ParseState::new("Inv : [Int]");
        let err = state.parse_func_param().unwrap_err();
        assert_eq!(InvalidValueIdentifier("Inv".to_string(), state), err);
    }
    #[test]
    fn test_parse_func_params() {
        use types::*;

        let (params, _) = ParseState::new("").parse_func_param_list().unwrap();
        let exp: Vec<TypedIdentifier> = vec![];
        assert_eq!(exp, params);

        let (params, _) = ParseState::new("a: Int").parse_func_param_list().unwrap();
        let exp: Vec<TypedIdentifier> = vec![TypedIdentifier::new(typ("Int"), "a")];
        assert_eq!(exp, params);

        let (params, _) = ParseState::new("a: Int,").parse_func_param_list().unwrap();
        let exp: Vec<TypedIdentifier> = vec![TypedIdentifier::new(typ("Int"), "a")];
        assert_eq!(exp, params);

        let (params, _) = ParseState::new("a: Int, b: Bool")
            .parse_func_param_list()
            .unwrap();
        let exp: Vec<TypedIdentifier> = vec![
            TypedIdentifier::new(typ("Int"), "a"),
            TypedIdentifier::new(typ("Bool"), "b"),
        ];
        assert_eq!(exp, params);

        let (params, _) = ParseState::new("a: Int, b: Bool,")
            .parse_func_param_list()
            .unwrap();
        let exp: Vec<TypedIdentifier> = vec![
            TypedIdentifier::new(typ("Int"), "a"),
            TypedIdentifier::new(typ("Bool"), "b"),
        ];
        assert_eq!(exp, params);

        let (params, _) = ParseState::new("a:Int,b:Bool")
            .parse_func_param_list()
            .unwrap();
        let exp: Vec<TypedIdentifier> = vec![
            TypedIdentifier::new(typ("Int"), "a"),
            TypedIdentifier::new(typ("Bool"), "b"),
        ];
        assert_eq!(exp, params);

        let (params, _) = ParseState::new("a : Int , b : Bool")
            .parse_func_param_list()
            .unwrap();
        let exp: Vec<TypedIdentifier> = vec![
            TypedIdentifier::new(typ("Int"), "a"),
            TypedIdentifier::new(typ("Bool"), "b"),
        ];
        assert_eq!(exp, params);
    }

    #[test]
    fn test_parse_func_head() {
        let (head, _) = ParseState::new("fn f() -> Int").parse_func_head().unwrap();

        let (head, _) = ParseState::new("fn f(a:Int) -> Int")
            .parse_func_head()
            .unwrap();

        let (head, _) = ParseState::new("fn exit()").parse_func_head().unwrap();

        let (head, _) = ParseState::new("fn print(s: String)")
            .parse_func_head()
            .unwrap();

        let (head, _) = ParseState::new("fn print(s: String,)")
            .parse_func_head()
            .unwrap();

        let (head, _) = ParseState::new("fn print(s: String,) -> Int")
            .parse_func_head()
            .unwrap();
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
