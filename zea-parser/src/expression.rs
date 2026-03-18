use crate::ParseError::InvalidFloatLiteral;
use crate::{NodeIdGenerator, ParseResult, ParseState};
use zea_ast::zea::{Expression, ExpressionKind};

macro_rules! wrap_expr {
    (Unit with $generator:ident, $state:ident) => {{
        let e = Expression {
            id: $generator.get(),
            kind: ExpressionKind::Unit,
        };
        Ok((e, $state))
    }};
    ($kind:ident($e:expr) with $generator:expr, $state:ident) => {{
        let e = Expression {
            id: $generator.get(),
            kind: ExpressionKind::$kind($e),
        };
        Ok((e, $state))
    }};
    ($kind:ident($op:expr, $e1:expr) with $generator:expr, $state:ident) => {{
        let e = Expression {
            id: $generator.get(),
            kind: ExpressionKind::UnOpExpr($op, Box::new($e1)),
        };
        Ok((e, $state))
    }};

    ($kind:ident($op:expr, $e1:expr, $e2:expr) with $generator:expr, $state:ident) => {{
        let e = Expression {
            id: $generator.get(),
            kind: ExpressionKind::BinOpExpr($op, Box::new($e1), Box::new($e2)),
        };
        Ok((e, $state))
    }};
}
//
// impl<'state> ParseState<'state> {
//     pub fn parse_expr(
//         self,
//         node_id_generator: &mut NodeIdGenerator,
//     ) -> ParseResult<'state, Expression> {
//         let state = self.whitespace();
//         state
//             .parse_expr_lit_int(node_id_generator)
//             .or(state.parse_expr_lit_float(node_id_generator))
//     }
//
//     pub fn parse_expr_ident(
//         self,
//         node_id_generator: &mut NodeIdGenerator,
//     ) -> ParseResult<'state, Expression> {
//         let (ident, state) = self.p_identifier()?;
//         wrap_expr!(Ident(ident) with node_id_generator, state)
//     }
//
//     pub fn parse_expr_lit_int(
//         self,
//         node_id_generator: &mut NodeIdGenerator,
//     ) -> ParseResult<'state, Expression> {
//         let (n, state) = self.parse_lit_int()?;
//         wrap_expr!(IntegerLiteral(n) with node_id_generator, state)
//     }
//
//     pub fn parse_expr_lit_float(
//         self,
//         node_id_generator: &mut NodeIdGenerator,
//     ) -> ParseResult<'state, Expression> {
//         let (f, state) = self.parse_lit_float()?;
//         wrap_expr!(FloatLiteral(f) with node_id_generator, state)
//     }
//
//     pub fn parse_expr_unit(
//         self,
//         node_id_generator: &mut NodeIdGenerator,
//     ) -> ParseResult<'state, Expression> {
//         let state = self.kw_unit()?;
//         wrap_expr!(Unit with node_id_generator, state)
//     }
//
//     pub fn parse_lit_int_hex(self) -> ParseResult<'state, u64> {
//         let state = self.whitespace();
//
//         let mut state = state.literal("0x").or(state.literal("0X"))?;
//         let mut total: u64 = 0;
//
//         loop {
//             if let Ok((digit, p_digit)) = state.digit(16) {
//                 // debug_assert!(p_digit.index > state_digits.index, "{digit}");
//                 total *= 16;
//                 total += digit;
//                 state = p_digit;
//                 continue;
//             }
//
//             if let Ok(p_underscore) = state.literal("_") {
//                 state = p_underscore;
//                 continue;
//             }
//             break Ok((total, state));
//         }
//     }
//
//     pub fn parse_lit_int_bin(self) -> ParseResult<'state, u64> {
//         let state = self.whitespace();
//
//         let mut state = state.literal("0b").or(state.literal("0B"))?;
//         let mut total: u64 = 0;
//
//         loop {
//             if let Ok((digit, p_digit)) = state.digit(2) {
//                 // debug_assert!(p_digit.index > state_digits.index, "{digit}");
//                 total *= 2;
//                 total += digit;
//                 state = p_digit;
//                 continue;
//             }
//
//             if let Ok(p_underscore) = state.literal("_") {
//                 state = p_underscore;
//                 continue;
//             }
//             break Ok((total, state));
//         }
//     }
//
//     pub fn parse_lit_int_dec(self) -> ParseResult<'state, u64> {
//         let mut state = self.whitespace();
//         let mut total: u64 = 0;
//
//         loop {
//             if let Ok((digit, p_digit)) = state.digit(10) {
//                 // debug_assert!(p_digit.index > state_digits.index, "{digit}");
//                 total *= 10;
//                 total += digit;
//                 state = p_digit;
//                 continue;
//             }
//
//             if let Ok(p_underscore) = state.literal("_") {
//                 state = p_underscore;
//                 continue;
//             }
//             break Ok((total, state));
//         }
//     }
//
//     pub fn parse_lit_int(self) -> ParseResult<'state, u64> {
//         let state = self.whitespace();
//         let mut total: u64 = 0;
//
//         state
//             .parse_lit_int_hex()
//             .or(state.parse_lit_int_bin())
//             .or(state.parse_lit_int_dec())
//     }
//
//     fn parse_lit_float_nan(self) -> ParseResult<'state, f64> {
//         self.literal(".nan").map(|p_nan| (f64::NAN, p_nan))
//     }
//
//     fn parse_lit_float_inf(self, negative: bool) -> ParseResult<'state, f64> {
//         self.literal(".inf").map(|p_nan| {
//             (
//                 if negative {
//                     f64::NEG_INFINITY
//                 } else {
//                     f64::INFINITY
//                 },
//                 p_nan,
//             )
//         })
//     }
//     pub fn parse_lit_float(self) -> ParseResult<'state, f64> {
//         let state = self.whitespace();
//         state.parse_lit_float_nan().or({
//             let (negative, state) = state.parse_lit_float_sign()?;
//             state
//                 .parse_lit_float_inf(negative)
//                 .or(state.parse_lit_float_numeric(negative))
//         })
//     }
//
//     pub fn parse_lit_float_numeric(self, negative: bool) -> ParseResult<'state, f64> {
//         let (_int, state) = self.no_eof()?.parse_lit_int_dec()?;
//
//         let state = match state.literal(".") {
//             Ok(p_dot) => {
//                 let (_frac, p_frac) = p_dot.parse_lit_int_dec()?;
//                 let try_exp = match p_frac.literal("e").or(p_frac.literal("E")) {
//                     Ok(p_e) => {
//                         let (_sign, p_sign) = p_e.parse_lit_float_sign()?;
//                         let (_exp, p_expr) = p_sign.parse_lit_int_dec()?;
//                         p_expr
//                     }
//                     _ => p_frac,
//                 };
//                 try_exp
//             }
//             _ => state,
//         };
//         let s = self.peek_diff(state);
//
//         Ok((
//             self.peek_diff(state)
//                 .parse::<f64>()
//                 .map(|f| if negative { -1.0 * f } else { f })
//                 .map_err(|_| InvalidFloatLiteral(s.to_string(), self))?,
//             state,
//         ))
//     }
//
//     pub fn parse_lit_float_sign(self) -> ParseResult<'state, bool> {
//         self.literal("-")
//             .map(|state| (true, state))
//             .or(self.succeed_with(false))
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use crate::{ParseError, ParseState};
//
//     #[test]
//     fn test_parse_lit_int_dec() {
//         let (i, _) = ParseState::new("123").parse_lit_int_dec().unwrap();
//         assert_eq!(123, i);
//
//         let (i, _) = ParseState::new("0").parse_lit_int_dec().unwrap();
//         assert_eq!(0, i);
//
//         // let (i, _) = ParseState::new("0x10").parse_expr_lit_int_dec().unwrap();
//         // assert_eq!(16, i);
//         //
//         // let (i, _) = ParseState::new("0x11").parse_expr_lit_int_dec().unwrap();
//         // assert_eq!(17, i);
//
//         let (i, _) = ParseState::new("11").parse_lit_int_dec().unwrap();
//         assert_eq!(11, i);
//
//         // let (i, _) = ParseState::new("0b1111").parse_expr_lit_int_dec().unwrap();
//         // assert_eq!(15, i);
//
//         let (i, _) = ParseState::new("9999").parse_lit_int_dec().unwrap();
//         assert_eq!(9999, i);
//
//         let (i, _) = ParseState::new("1_000_000").parse_lit_int_dec().unwrap();
//         assert_eq!(1000000, i);
//
//         let (i, _) = ParseState::new("1_000_").parse_lit_int_dec().unwrap();
//         assert_eq!(1000, i);
//
//         let (i, _) = ParseState::new("________________________1_000_")
//             .parse_lit_int_dec()
//             .unwrap();
//         assert_eq!(1000, i);
//
//         let (i, _) = ParseState::new("0001").parse_lit_int_dec().unwrap();
//         assert_eq!(1, i);
//     }
//
//     #[test]
//     fn test_parse_lit_int_hex() {
//         let (i, _) = ParseState::new("0x10").parse_lit_int_hex().unwrap();
//         assert_eq!(16, i);
//
//         let (i, _) = ParseState::new("0x11").parse_lit_int_hex().unwrap();
//         assert_eq!(17, i);
//
//         let (i, _) = ParseState::new("0x________________________________________________11")
//             .parse_lit_int_hex()
//             .unwrap();
//         assert_eq!(17, i);
//
//         let (i, _) = ParseState::new("0x011").parse_lit_int_hex().unwrap();
//         assert_eq!(17, i);
//
//         let (i, _) = ParseState::new("0x_0_000_00_____00_00000____00_011")
//             .parse_lit_int_hex()
//             .unwrap();
//         assert_eq!(17, i);
//
//         let (i, _) = ParseState::new("0xFF").parse_lit_int_hex().unwrap();
//         assert_eq!(255, i);
//
//         let (i, _) = ParseState::new("0xff").parse_lit_int_hex().unwrap();
//         assert_eq!(255, i);
//         let (i, _) = ParseState::new("0xfF").parse_lit_int_hex().unwrap();
//         assert_eq!(255, i);
//     }
//
//     #[test]
//     fn test_parse_lit_int_bin() {
//         let (i, _) = ParseState::new("0b_1111_1111").parse_lit_int_bin().unwrap();
//         assert_eq!(255, i);
//
//         let (i, _) = ParseState::new("0b0000_1111_0000_1111")
//             .parse_lit_int_bin()
//             .unwrap();
//         assert_eq!(3855, i);
//
//         let (i, _) = ParseState::new("0b________________________________________________11")
//             .parse_lit_int_bin()
//             .unwrap();
//         assert_eq!(3, i);
//
//         let (i, _) = ParseState::new("0b011").parse_lit_int_bin().unwrap();
//         assert_eq!(3, i);
//
//         let (i, _) = ParseState::new("0b_0_000_00_____00_00000____00_011")
//             .parse_lit_int_bin()
//             .unwrap();
//         assert_eq!(3, i);
//     }
//     #[test]
//     fn test_parse_lit_int() {}
//
//     #[test]
//     fn test_parse_expr_lit_float_numeric() {
//         // --- Integer part only ---
//         let (val, after) = ParseState::new("42")
//             .parse_lit_float_numeric(false)
//             .unwrap();
//         assert_eq!(42., val);
//         assert_eq!(after.peek(), None);
//
//         let (val, after) = ParseState::new("0").parse_lit_float_numeric(false).unwrap();
//         assert_eq!(0.0, val);
//         assert_eq!(after.peek_remaining(), "");
//
//         // --- Integer + decimal ---
//         let (val, _) = ParseState::new("3.14")
//             .parse_lit_float_numeric(false)
//             .unwrap();
//         assert_eq!(3.14, val);
//
//         let (val, _) = ParseState::new("0.0")
//             .parse_lit_float_numeric(false)
//             .unwrap();
//         assert_eq!(0.0, val);
//
//         let (val, _) = ParseState::new("123.456")
//             .parse_lit_float_numeric(false)
//             .unwrap();
//         assert_eq!(123.456, val);
//
//         let (val, _) = ParseState::new("1.5e3")
//             .parse_lit_float_numeric(false)
//             .unwrap();
//         assert_eq!(1.5e3, val);
//
//         let (val, _) = ParseState::new("1.5E3")
//             .parse_lit_float_numeric(false)
//             .unwrap();
//         assert_eq!(1.5E3, val);
//
//         let (val, _) = ParseState::new("2.5e-3")
//             .parse_lit_float_numeric(false)
//             .unwrap();
//         assert_eq!(2.5e-3, val);
//
//         let (val, after) = ParseState::new("1.5 rest")
//             .parse_lit_float_numeric(false)
//             .unwrap();
//         assert_eq!(1.5, val);
//         assert_eq!(after.peek_remaining(), " rest");
//
//         let (val, after) = ParseState::new("99abc")
//             .parse_lit_float_numeric(false)
//             .unwrap();
//         assert_eq!(99.0, val);
//         assert_eq!(after.peek_remaining(), "abc");
//
//         let state = ParseState::new("");
//         assert!(matches!(
//             state.parse_lit_float_numeric(false),
//             Err(ParseError::UnexpectedEOF(state))
//         ));
//
//         assert!(
//             ParseState::new("-1.0")
//                 .parse_lit_float_numeric(false)
//                 .is_err()
//         );
//     }
//
//     #[test]
//     fn test_parse_expr_lit_float() {
//         // --- Positive numeric ---
//         let (val, _) = ParseState::new("3.14").parse_lit_float().unwrap();
//         assert_eq!(3.14, val);
//
//         let (val, _) = ParseState::new("0.0").parse_lit_float().unwrap();
//         assert_eq!(0.0, val);
//
//         let (val, _) = ParseState::new("1.5e3").parse_lit_float().unwrap();
//         assert_eq!(1.5e3, val);
//
//         let (val, _) = ParseState::new("2.5e-3").parse_lit_float().unwrap();
//         assert_eq!(2.5e-3, val);
//
//         // --- Negative numeric ---
//         let (val, _) = ParseState::new("-3.14").parse_lit_float().unwrap();
//         assert_eq!(-3.14, val);
//
//         let (val, _) = ParseState::new("-1.5e3").parse_lit_float().unwrap();
//         assert_eq!(-1.5e3, val);
//
//         let (val, _) = ParseState::new("-0.0").parse_lit_float().unwrap();
//         assert_eq!(val, -0.0);
//
//         // --- .inf ---
//         let (val, after) = ParseState::new(".inf").parse_lit_float().unwrap();
//         assert!(val.is_infinite() && val.is_sign_positive());
//         assert_eq!(after.peek_remaining(), "");
//
//         let (val, after) = ParseState::new("-.inf").parse_lit_float().unwrap();
//         assert!(val.is_infinite() && val.is_sign_negative());
//         assert_eq!(after.peek(), None);
//
//         // --- .nan ---
//         let (val, after) = ParseState::new(".nan").parse_lit_float().unwrap();
//         assert!(val.is_nan());
//         assert_eq!(after.peek(), None);
//         assert!(ParseState::new("-.nan").parse_lit_float().is_err());
//
//         // --- State advances correctly ---
//         let (val, after) = ParseState::new("1.5 rest").parse_lit_float().unwrap();
//         assert!((val - 1.5).abs() < f64::EPSILON);
//         assert_eq!(after.peek(), Some(' '));
//
//         let (val, after) = ParseState::new(".inf next").parse_lit_float().unwrap();
//         assert!(val.is_infinite());
//         assert_eq!(after.peek(), Some(' '));
//
//         let state = ParseState::new("");
//         assert_eq!(
//             state.parse_lit_float(),
//             Err(ParseError::UnexpectedEOF(state))
//         );
//     }
// }
