#![allow(unused)]

pub mod expression;

use crate::ParseError::{InvalidValueIdentifier, UnexpectedEOF};
use zea_ast::zea;
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
    InvalidFloatLiteral(String, ParseState<'a>),
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

    fn peek_n(self, n: usize) -> Result<&'a str, ParseError<'a>> {
        self.input.get(self.index + n..).ok_or(UnexpectedEOF)
    }
    fn peek_remaining(self) -> &'a str {
        &self.input[self.index..]
    }

    fn eat_ignore(self) -> Result<ParseState<'a>, ParseError<'a>> {
        let (line, column) = if self.peek().ok_or(UnexpectedEOF)? == '\n' {
            (self.line + 1, 1)
        } else {
            (self.line, self.column + 1)
        };

        Ok(ParseState {
            line,
            column,
            index: self.index + 1,
            ..self
        })
    }
    fn eat(self) -> ParseResult<'a, char> {
        if self.index >= self.input.len() {
            return Err(UnexpectedEOF);
        }

        let c = self.peek().unwrap();

        let (line, column) = if c == '\n' {
            (self.line + 1, 1)
        } else {
            (self.line, self.column + 1)
        };

        Ok((
            c,
            ParseState {
                line,
                column,
                index: self.index + 1,
                ..self
            },
        ))
    }

    fn eat_bigly(self, n: usize) -> ParseResult<'a, &'a str> {
        let mut state = self;
        let mut s = String::with_capacity(n);

        for _ in 0..n {
            // let try_eat = state.eat().unwrap();
            let (ch, p_char) = state.eat()?;
            s.push(ch);
            state = p_char;
        }

        Ok((self.peek_n(n)?, state))
    }

    fn whitespace(self) -> ParseState<'a> {
        let mut state = self;

        loop {
            match state.peek() {
                Some(c) if c.is_whitespace() => state = state.eat_ignore().unwrap(),
                _ => break,
            }
        }

        state
    }

    pub fn peek_diff(self, other: ParseState<'a>) -> &'a str {
        let start = self.index.min(other.index);
        let end = self.index.max(other.index);
        &self.input[start..end]
    }

    pub fn succeed_with<T>(self, value: T) -> ParseResult<'a, T> {
        Ok((value, self))
    }

    pub fn no_eof(self) -> Result<ParseState<'a>, ParseError<'a>> {
        match self.peek() {
            Some(_) => Ok(self),
            None => Err(UnexpectedEOF),
        }
    }

    fn eat_while(
        self,
        predicate: impl Fn(char) -> bool,
        can_be_eof: bool,
    ) -> ParseResult<'a, &'a str> {
        let mut state = self;
        loop {
            match state.eat() {
                Ok((ch, p_char)) if predicate(ch) => {
                    state = p_char;
                }
                Ok((_ch, p_char)) => return Ok((self.peek_diff(state), state)),
                Err(e) => {
                    return if can_be_eof {
                        Ok((self.peek_diff(state), state))
                    } else {
                        Err(e)
                    };
                }
            }
        }
    }

    fn digit(self, radix: u32) -> ParseResult<'a, u64> {
        if radix < 2 || radix > 32 {
            panic!("invalid radix {radix}")
        }
        let d = self.peek().ok_or(UnexpectedEOF)?;
        let d = d
            .to_digit(radix)
            .ok_or(ParseError::UnexpectedInput(d.to_string(), self))?;
        Ok((d as u64, self.eat_ignore().unwrap()))
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
        fn is_valid_char(ch: char) -> bool {
            ch.is_lowercase() || ch.is_ascii_digit() || "-?!".contains(ch)
        }

        let mut state = self;

        loop {
            match state.eat() {
                Ok((c, p_char)) if is_valid_char(c) => {
                    state = p_char;
                }
                _ => break,
            }
        }

        let identifier = self.peek_diff(state).to_string();
        if invalid {
            Err(InvalidValueIdentifier(identifier, self))
        } else {
            Ok((identifier, state))
        }
    }

    pub fn parse_type_identifier(self) -> ParseResult<'a, String> {
        let invalid = match self.peek() {
            None => return Err(UnexpectedEOF),
            Some(c) => !c.is_uppercase(),
        };
        fn is_valid_char(ch: char) -> bool {
            ch.is_alphabetic() || ch.is_ascii_digit() || "_?!".contains(ch)
        }

        let mut state = self;

        loop {
            match state.eat() {
                Ok((c, p_char)) if is_valid_char(c) => {
                    state = p_char;
                }
                _ => break,
            }
        }

        let identifier = self.peek_diff(state).to_string();
        if invalid {
            Err(InvalidValueIdentifier(identifier, self))
        } else {
            Ok((identifier, state))
        }
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

    pub fn parse_import_statement(self) -> ParseResult<'a, Vec<String>> {
        let (_, state) = self.toklit("import")?;
        let state = state.whitespace();
        let (identifier, state) = state.parse_expr_identifier()?;

        Ok((vec![identifier], state))
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
        self.parse_array_type().or(self.parse_pointer_type())
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
}

#[cfg(test)]
mod tests {
    use super::*;
    mod types {
        use zea_ast::zea::Type;

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
        let state = state.eat_ignore().unwrap();
        assert_eq!(Some('b'), state.peek());
        let state = state.eat_ignore().unwrap();
        assert_eq!(Some('c'), state.peek());
        let state = state.eat_ignore().unwrap();
        assert_eq!(None, state.peek());
    }

    #[test]
    fn test_parse_state_eat_bigly() {
        let state = ParseState::new("abc");
        let (_, bigly) = state.eat_bigly(0).unwrap();
        assert!(bigly.index == state.index);
        let (_, bigly) = state.eat_bigly(1).unwrap();
        assert!(bigly.index == state.index + 1);
        let (_, bigly) = state.eat_bigly(2).unwrap();
        assert!(bigly.index == state.index + 2);

        let err = state.eat_bigly(4).unwrap_err();
        assert_eq!(UnexpectedEOF, err);
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

        // let state = ParseState::new("[Int");
        // let (_, expstate) = state.eat_bigly(4).unwrap();
        // let err = state.parse_type().unwrap_err();
        // assert_eq!(
        //     ParseError::LiteralNotMatched("]".to_string(), expstate),
        //     err
        // );

        // let state = ParseState::new("int");
        // let err = state.parse_type().unwrap_err();
        // assert_eq!(
        //     ParseError::InvalidTypeIdentifier("int".to_string(), state),
        //     err
        // );
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

        // let state = ParseState::new("Inv : [Int]");
        // let err = state.parse_func_param().unwrap_err();
        // assert_eq!(InvalidValueIdentifier("Inv".to_string(), state), err);
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
    fn test_parse_digit() {
        let state = ParseState::new("123");
        let (d, advanced) = state.digit(10).unwrap();
        assert_eq!(1, d);
        assert_eq!("23", advanced.peek_remaining());
        let (d, advanced) = advanced.digit(10).unwrap();
        assert_eq!(2, d);
        assert_eq!("3", advanced.peek_remaining());
        let (d, advanced) = advanced.digit(10).unwrap();
        assert_eq!(3, d);
        assert_eq!("", advanced.peek_remaining());

        let (d, _) = state.digit(2).unwrap();
        assert_eq!(1, d);
        let (d, _) = state.digit(16).unwrap();
        assert_eq!(1, d);

        let nine = ParseState::new("9");
        let (d, _) = nine.digit(10).unwrap();
        assert_eq!(9, d);
        let (d, _) = nine.digit(16).unwrap();
        assert_eq!(9, d);
        let err = nine.digit(2).unwrap_err();
        assert_eq!(ParseError::UnexpectedInput("9".to_string(), nine), err);

        let f = ParseState::new("f");
        let err = f.digit(2).unwrap_err();
        assert_eq!(ParseError::UnexpectedInput("f".to_string(), f), err);
        let f = ParseState::new("f");
        let err = f.digit(10).unwrap_err();
        assert_eq!(ParseError::UnexpectedInput("f".to_string(), f), err);
        let (d, _) = f.digit(16).unwrap();
        assert_eq!(15, d);
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
