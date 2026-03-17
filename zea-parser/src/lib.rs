#![allow(unused)]

pub mod expression;
pub mod statement;

use crate::ParseError::{InvalidValueIdentifier, UnexpectedEOF};
use zea_ast::zea;
use zea_ast::zea::{Function, StatementBlock, Type, TypedIdentifier};

const KW_FUNC: &str = "fn";
const KW_STRUCT: &str = "struct";
const KW_TAGGED_UNION: &str = "enum";
const KW_LOOP: &str = "for";
const KW_IF: &str = "if";
const KW_UNLESS: &str = "unless";
const KW_WHILE: &str = "while";
const KW_UNTIL: &str = "until";
const KW_IMPORTS: &str = "imports";
const KW_EXPORTS: &str = "exports";
const KW_MODULE: &str = "module";
const KW_RETURN: &str = "return";
const OP_ASSIGN: &str = "=";
const OP_DEREF: &str = "@";

const OP_REF: &str = "&";

const OP_CAST: &str = "as";
const OP_LOG_OR: &str = "||";
const OP_LOG_AND: &str = "&&";
const OP_LOG_XOR: &str = "^^";
const OP_LOG_NOT: &str = "~";

const OP_BIT_OR: &str = "|";
const OP_BIT_AND: &str = "&";
const OP_BIT_XOR: &str = "^";
const OP_BIT_NOT: &str = "~";

const OP_PIPE: &str = "|>";
const PIPE_HOLE: &str = "$";

const KW_UNIT: &str = "void";

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

impl<'state> ParseState<'state> {
    pub fn new(input: &'state str) -> ParseState<'state> {
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

    /// Peek n characters forward,
    /// or return UnexpectedEOF if there is less than n characters of input left
    fn peek_n(self, n: usize) -> Result<&'state str, ParseError<'state>> {
        self.input.get(self.index + n..).ok_or(UnexpectedEOF)
    }
    /// Peek the remaining input (i.e. The after starting from the state's index)
    fn peek_remaining(self) -> &'state str {
        &self.input[self.index..]
    }

    /// Advance the state by one character, but discards that character.
    fn eat_ignore(self) -> Result<ParseState<'state>, ParseError<'state>> {
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

    /// Advance the state by one character and return it
    fn eat(self) -> ParseResult<'state, char> {
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

    /// Advance the state by n characters,
    /// returning the consumed characters.
    fn eat_bigly(self, n: usize) -> ParseResult<'state, &'state str> {
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

    /// Skip any whitespace at the current input onwards.
    /// Guarantees the state to end up at a non-whitespace character.
    fn whitespace(self) -> ParseState<'state> {
        let mut state = self;

        loop {
            match state.peek() {
                Some(c) if c.is_whitespace() => state = state.eat_ignore().unwrap(),
                _ => break,
            }
        }

        state
    }

    /// Skip any whitespace at the current input onwards.
    /// Guarantees the state to end up at a non-whitespace character.
    /// Returns an error if there is not at least one whitespace character consumed.
    fn require_whitespace(self) -> Result<ParseState<'state>, ParseError<'state>> {
        let mut state = self;
        if state.peek().is_some_and(|ch| !ch.is_whitespace()) {
            return Err(ParseError::UnexpectedInput(
                state.peek().unwrap().to_string(),
                state,
            ));
        }
        loop {
            match state.peek() {
                Some(c) if c.is_whitespace() => state = state.eat_ignore().unwrap(),
                _ => break,
            }
        }

        Ok(state)
    }

    /// Get the input span between two states
    pub fn peek_diff(self, other: ParseState<'state>) -> &'state str {
        let start = self.index.min(other.index);
        let end = self.index.max(other.index);
        &self.input[start..end]
    }

    /// Wrap some value with a state into an Ok-ParseResult
    pub fn succeed_with<T>(self, value: T) -> ParseResult<'state, T> {
        Ok((value, self))
    }

    pub fn no_eof(self) -> Result<ParseState<'state>, ParseError<'state>> {
        match self.peek() {
            Some(_) => Ok(self),
            None => Err(UnexpectedEOF),
        }
    }

    /// Keep consuming characters while some predicate holds.
    /// Will return an error if `can_be_eof` is `false` and the state encounters EOF.
    /// returns a result containing the consumed characters otherwise.
    fn eat_while(
        self,
        predicate: impl Fn(char) -> bool,
        can_be_eof: bool,
    ) -> ParseResult<'state, &'state str> {
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

    /// Parse some digit in base `radix`
    /// A valid radix is one between 2 and 32: `2 <= radix <= 32`.
    fn digit(self, radix: u32) -> ParseResult<'state, u64> {
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
    /// Parse some literal string.
    /// Does not check that the literal ends with a whitespace.
    pub fn token<'token: 'state>(self, keyword: &'token str) -> ParseResult<'state, &'state str> {
        if !self.starts_with(keyword) {
            return Err(ParseError::LiteralNotMatched(keyword.to_string(), self));
        }

        Ok(self.eat_bigly(keyword.len()).unwrap())
    }

    pub fn parse_non_type_identifier(self) -> ParseResult<'state, String> {
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

    pub fn parse_type_identifier(self) -> ParseResult<'state, String> {
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

    pub fn open_paren(self) -> Result<ParseState<'state>, ParseError<'state>> {
        let (_, state) = self.token("(")?;
        Ok(state)
    }

    pub fn close_paren(self) -> Result<ParseState<'state>, ParseError<'state>> {
        let (_, state) = self.token(")")?;
        Ok(state)
    }

    pub fn open_brace(self) -> Result<ParseState<'state>, ParseError<'state>> {
        let (_, state) = self.token("{")?;
        Ok(state)
    }

    pub fn close_brace(self) -> Result<ParseState<'state>, ParseError<'state>> {
        let (_, state) = self.token("}")?;
        Ok(state)
    }

    pub fn comma(self) -> Result<ParseState<'state>, ParseError<'state>> {
        let (_, state) = self.token(",")?;
        Ok(state)
    }

    pub fn colon(self) -> Result<ParseState<'state>, ParseError<'state>> {
        let (_, state) = self.token(":")?;
        Ok(state)
    }

    pub fn semicolon(self) -> Result<ParseState<'state>, ParseError<'state>> {
        let (_, state) = self.token(";")?;
        Ok(state)
    }

    pub fn fn_arrow(self) -> Result<ParseState<'state>, ParseError<'state>> {
        let (_, state) = self.token("->")?;
        Ok(state)
    }

    pub fn kw_func(self) -> Result<ParseState<'state>, ParseError<'state>> {
        self.keyword(KW_FUNC)
    }
    pub fn kw_return(self) -> Result<ParseState<'state>, ParseError<'state>> {
        self.keyword(KW_RETURN)
    }

    pub fn kw_unit(self) -> Result<ParseState<'state>, ParseError<'state>> {
        self.keyword(KW_UNIT)
    }

    pub fn op_assign(self) -> Result<ParseState<'state>, ParseError<'state>> {
        let (_, state) = self.token(OP_ASSIGN)?;
        Ok(state)
    }

    /// Consume some literal token, asserting that the token is followed by whitespace
    pub fn keyword<'token: 'state>(
        self,
        keyword: &'token str,
    ) -> Result<ParseState<'state>, ParseError<'state>> {
        let (_, state) = self.token(keyword)?;
        let state = state.require_whitespace()?;
        Ok(state)
    }

    pub fn parse_import_statement(self) -> ParseResult<'state, Vec<String>> {
        let (_, state) = self.token("import")?;
        let state = state.whitespace();
        let (identifier, state) = state.parse_non_type_identifier()?;

        Ok((vec![identifier], state))
    }
    fn parse_pointer_type(self) -> ParseResult<'state, Type> {
        let state = self;
        let (ident, mut state) = self.parse_type_identifier()?;
        let mut res = Type::Basic(ident);

        loop {
            if let Ok((_, parsed_pointer)) = state.token("*") {
                res = Type::Pointer(Box::new(res));
                state = parsed_pointer;
                continue;
            } else {
                break;
            }
        }
        Ok((res, state))
    }

    fn parse_array_type(self) -> ParseResult<'state, Type> {
        let (_, state) = self.whitespace().token("[")?;

        let (typ, state): (Type, ParseState<'state>) = state.parse_type()?;
        let state: ParseState<'state> = state.whitespace();

        let (_, state) = state.token("]")?;
        let state = state.whitespace();
        let typ = Type::ArrayOf(Box::new(typ));
        Ok((typ, state))
    }

    fn parse_type(self) -> ParseResult<'state, zea::Type> {
        self.parse_array_type().or(self.parse_pointer_type())
    }

    pub fn parse_func_param(self) -> ParseResult<'state, TypedIdentifier> {
        let (ident, state) = self.whitespace().parse_non_type_identifier()?;
        let state = state.whitespace();
        let state = state.colon()?;
        let state = state.whitespace();
        let (typ, state) = state.parse_type()?;
        let state = state.whitespace();
        Ok((TypedIdentifier::new(typ, ident), state))
    }

    pub fn parse_func_param_list(self) -> ParseResult<'state, Vec<TypedIdentifier>> {
        let mut state = self.whitespace();
        let mut res = vec![];

        loop {
            if let Ok((param, parsed_param)) = state.whitespace().parse_func_param() {
                res.push(param);
                state = parsed_param.whitespace();

                if let Ok(p_comma) = state.whitespace().comma() {
                    state = p_comma.whitespace();
                    continue;
                }
            }
            break;
        }

        Ok((res, state))
    }

    pub fn parse_func_head(self) -> ParseResult<'state, (String, Vec<TypedIdentifier>, Type)> {
        let state = self.whitespace().keyword(KW_FUNC)?;
        let (name, state) = state.parse_non_type_identifier()?;
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

    pub fn parse_func_body(
        self,
        node_id_generator: &mut NodeIdGenerator,
    ) -> ParseResult<'state, StatementBlock> {
        let mut state = self.open_brace()?;
        let mut block = StatementBlock {
            id: node_id_generator.get(),
            statements: Vec::new(),
        };
        loop {
            match state.whitespace().parse_stmt(node_id_generator) {
                Ok((stmt, p_stmt)) => {
                    block.statements.push(stmt);
                    state = p_stmt;
                }
                Err(_) => break,
            }
        }

        let state = state.close_brace()?;
        Ok((block, state))
    }

    pub fn parse_func(
        self,
        node_id_generator: &mut NodeIdGenerator,
    ) -> ParseResult<'state, Function> {
        let ((name, args, returns), state) = self.parse_func_head()?;
        let (body, state) = state.parse_func_body(node_id_generator)?;
        let function = Function {
            id: node_id_generator.get(),
            name,
            args,
            returns,
            body,
        };
        Ok((function, state))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::types::typ;
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
        let (identifier, state) = state.parse_non_type_identifier().unwrap();

        assert_eq!("xyz", identifier);

        let state = ParseState::new("xyz? bob");
        let (identifier, state) = state.parse_non_type_identifier().unwrap();

        assert_eq!("xyz?", identifier);

        let state = ParseState::new("xyz! bob");
        let (identifier, state) = state.parse_non_type_identifier().unwrap();

        assert_eq!("xyz!", identifier);

        let state = ParseState::new("xyz!?? bob");
        let (identifier, state) = state.parse_non_type_identifier().unwrap();

        assert_eq!("xyz!??", identifier);

        let state = ParseState::new("bob-is-cool");
        let (identifier, state) = state.parse_non_type_identifier().unwrap();

        assert_eq!("bob-is-cool", identifier);
        let state = ParseState::new("is-even? no its not bro");
        let (identifier, state) = state.parse_non_type_identifier().unwrap();

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

    #[test]
    fn test_parse_function() {
        let state = ParseState::new("fn main() -> Int {}");
        let mut ids = NodeIdGenerator::new();
        let (func, _) = state.parse_func(&mut ids).unwrap();
        assert_eq!(func.returns, typ("Int"));
        assert_eq!(func.name, "main");
        assert_eq!(func.args, vec![]);
        assert_eq!(func.body.statements, vec![]);

        let state = ParseState::new("fn main()->Int{}");
        let mut ids = NodeIdGenerator::new();
        let (func, _) = state.parse_func(&mut ids).unwrap();
        assert_eq!(func.returns, typ("Int"));
        assert_eq!(func.name, "main");
        assert_eq!(func.args, vec![]);
        assert_eq!(func.body.statements, vec![]);

        let state = ParseState::new("fnmain () ->Int{}");
        let mut ids = NodeIdGenerator::new();
        assert!(state.parse_func(&mut ids).is_err());
    }
}
