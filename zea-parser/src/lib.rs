#![allow(unused)]

#[cfg(feature = "lalrpop_parser")]
mod parser;
#[cfg(feature = "zepast")]
pub mod zepast;

use log::{error, info, log};

#[cfg(feature = "lalrpop_parser")]
pub use parser::ExprParser as ExpressionParser;
#[cfg(feature = "lalrpop_parser")]
pub use parser::ModParser as ModuleParser;
#[cfg(feature = "lalrpop_parser")]
pub use parser::StmtParser as StatementParser;
use std::process::exit;
use zea_ipr::zea::BareNodeLabeler;

use zea_ipr::zea::visitors::annotating::SemanticASTViolation;

use zea_ipr::zea::immediate_parsed_representation::*;
use zea_ipr::zea::visitors::IPRTransfomer;
pub fn parse_module(src: &'_ str) -> (IPRModule, BareNodeLabeler) {
    let p = ModuleParser::new();
    info!("parsing source file...");
    let mut module = match p.parse(src) {
        Ok(module) => module,
        Err(e) => {
            error!("{e}");
            exit(1);
        }
    };
    info!("\tparsed source file succesfully");
    let mut labeler = BareNodeLabeler::new();
    info!("starting node-labeling...");
    labeler.visit_module(&mut module);
    info!("\tnode-labeling successful");
    (module, labeler)
}

pub(crate) enum IPRModuleItem {
    Init(IPRInitializationBlock),
    Func(IPRFunction),
    StructDef(IPRStructDataTypeDefinition),
    EnumDef(IPRTaggedUnionDataTypeDefinition),
}

pub(crate) fn separate_module_items(
    items: Vec<IPRModuleItem>,
) -> (
    Vec<IPRInitializationBlock>,
    Vec<IPRFunction>,
    Vec<IPRStructDataTypeDefinition>,
    Vec<IPRTaggedUnionDataTypeDefinition>,
) {
    let mut globs = vec![];
    let mut funcs = vec![];
    let mut structs = vec![];
    let mut tagged_unions = vec![];
    for item in items {
        match item {
            IPRModuleItem::Init(i) => globs.push(i),
            IPRModuleItem::Func(f) => funcs.push(f),
            IPRModuleItem::StructDef(s) => structs.push(s),
            IPRModuleItem::EnumDef(tu) => tagged_unions.push(tu),
        }
    }
    (globs, funcs, structs, tagged_unions)
}

#[cfg(all(feature = "lalrpop_parser", test))]
mod tests {
    use crate::parser::{
        AssignPatParser, ExprParser, FuncParser, InitParser, ModParser, StmtParser,
    };

    use zea_ipr::zea::{immediate_parsed_representation::*, BinOp, UnOp};

    // ── helpers ───────────────────────────────────────────────────────────────

    fn parse_expr(src: &str) -> IPRExpression {
        ExprParser::new()
            .parse(src)
            .unwrap_or_else(|e| panic!("expr parse failed for {src:?}:\n  {e}"))
    }

    fn parse_stmt(src: &str) -> IPRStatement {
        StmtParser::new()
            .parse(src)
            .unwrap_or_else(|e| panic!("stmt parse failed for {src:?}:\n  {e}"))
    }

    fn parse_func(src: &str) -> IPRFunction {
        FuncParser::new()
            .parse(src)
            .unwrap_or_else(|e| panic!("func parse failed for {src:?}:\n  {e}"))
    }

    fn parse_init(src: &str) -> IPRInitializationBlock {
        InitParser::new()
            .parse(src)
            .unwrap_or_else(|e| panic!("init parse failed for {src:?}:\n  {e}"))
    }

    fn parse_mod(src: &str) -> IPRModule {
        ModParser::new()
            .parse(src)
            .unwrap_or_else(|e| panic!("module parse failed for {src:?}:\n  {e}"))
    }

    fn parse_pat(src: &str) -> IPRAssignmentPattern {
        AssignPatParser::new()
            .parse(src)
            .unwrap_or_else(|e| panic!("pattern parse failed for {src:?}:\n  {e}"))
    }

    fn kind(e: &IPRExpression) -> &IPRExpressionKind {
        &e.kind
    }

    // ── atoms ─────────────────────────────────────────────────────────────────

    #[test]
    fn integer_decimal() {
        assert!(matches!(
            kind(&parse_expr("42")),
            IPRExpressionKind::IntegerLiteral(42)
        ));
    }

    #[test]
    fn integer_hex() {
        assert!(matches!(
            kind(&parse_expr("0xFF")),
            IPRExpressionKind::IntegerLiteral(255)
        ));
    }

    #[test]
    fn integer_hex_lowercase() {
        assert!(matches!(
            kind(&parse_expr("0xff")),
            IPRExpressionKind::IntegerLiteral(255)
        ));
    }

    #[test]
    fn integer_binary() {
        assert!(matches!(
            kind(&parse_expr("0b1010")),
            IPRExpressionKind::IntegerLiteral(10)
        ));
    }

    #[test]
    fn integer_decimal_underscores() {
        assert!(matches!(
            kind(&parse_expr("1_000_000")),
            IPRExpressionKind::IntegerLiteral(1_000_000)
        ));
    }

    #[test]
    fn integer_hex_underscores() {
        assert!(matches!(
            kind(&parse_expr("0xFF_FF")),
            IPRExpressionKind::IntegerLiteral(0xFFFF)
        ));
    }

    #[test]
    fn integer_binary_underscores() {
        assert!(matches!(
            kind(&parse_expr("0b1111_0000")),
            IPRExpressionKind::IntegerLiteral(0b1111_0000)
        ));
    }

    #[test]
    fn identifier_simple() {
        assert!(
            matches!(kind(&parse_expr("foo")), IPRExpressionKind::UnScopedIdent(s) if s == "foo")
        );
    }

    #[test]
    fn identifier_with_digits() {
        assert!(
            matches!(kind(&parse_expr("foo123")), IPRExpressionKind::UnScopedIdent(s) if s == "foo123")
        );
    }

    #[test]
    fn identifier_with_hyphens() {
        assert!(
            matches!(kind(&parse_expr("my-var")), IPRExpressionKind::UnScopedIdent(s) if s == "my-var")
        );
    }

    #[test]
    fn identifier_question_mark() {
        assert!(
            matches!(kind(&parse_expr("empty?")), IPRExpressionKind::UnScopedIdent(s) if s == "empty?")
        );
    }

    #[test]
    fn identifier_bang() {
        assert!(
            matches!(kind(&parse_expr("reset!")), IPRExpressionKind::UnScopedIdent(s) if s == "reset!")
        );
    }

    // ── unary ops ─────────────────────────────────────────────────────────────

    #[test]
    fn unary_negate() {
        assert!(matches!(
            kind(&parse_expr("-1")),
            IPRExpressionKind::UnOpExpr(UnOp::Neg, _)
        ));
    }

    #[test]
    fn unary_logical_not() {
        assert!(matches!(
            kind(&parse_expr("!x")),
            IPRExpressionKind::UnOpExpr(UnOp::LogNot, _)
        ));
    }

    #[test]
    fn unary_bitwise_not() {
        assert!(matches!(
            kind(&parse_expr("~x")),
            IPRExpressionKind::UnOpExpr(UnOp::BitNot, _)
        ));
    }

    #[test]
    fn unary_chained() {
        // !!x — outer LogNot wrapping inner LogNot
        let e = parse_expr("!!x");
        assert!(matches!(kind(&e),
            IPRExpressionKind::UnOpExpr(UnOp::LogNot, inner)
            if matches!(kind(inner), IPRExpressionKind::UnOpExpr(UnOp::LogNot, _))
        ));
    }

    // ── binary ops — one test per tier ───────────────────────────────────────

    macro_rules! binop_test {
        ($name:ident, $src:expr, $op:pat) => {
            #[test]
            fn $name() {
                assert!(matches!(
                    kind(&parse_expr($src)),
                    IPRExpressionKind::BinOpExpr($op, _, _)
                ));
            }
        };
    }

    binop_test!(binop_mul, "a * b", BinOp::Mul);
    binop_test!(binop_div, "a / b", BinOp::Div);
    binop_test!(binop_mod, "a % b", BinOp::Mod);
    binop_test!(binop_add, "a + b", BinOp::Add);
    binop_test!(binop_sub, "a - b", BinOp::Sub);
    binop_test!(binop_lsh, "a << b", BinOp::Lsh);
    binop_test!(binop_rsh, "a >> b", BinOp::Rsh);
    binop_test!(binop_lt, "a < b", BinOp::LT);
    binop_test!(binop_gt, "a > b", BinOp::GT);
    binop_test!(binop_leq, "a <= b", BinOp::Leq);
    binop_test!(binop_geq, "a >= b", BinOp::Geq);
    binop_test!(binop_eq, "a == b", BinOp::Eq);
    binop_test!(binop_neq, "a != b", BinOp::Neq);
    binop_test!(binop_bitand, "a & b", BinOp::BitAnd);
    binop_test!(binop_bitxor, "a ^ b", BinOp::BitXor);
    binop_test!(binop_bitor, "a | b", BinOp::BitOr);
    binop_test!(binop_logand, "a && b", BinOp::LogAnd);
    binop_test!(binop_logxor, "a ^^ b", BinOp::LogXor);
    binop_test!(binop_logor, "a || b", BinOp::LogOr);

    // ── precedence ────────────────────────────────────────────────────────────

    #[test]
    fn prec_mul_over_add() {
        // a + b * c  =>  Add(a, Mul(b, c))
        match kind(&parse_expr("a + b * c")) {
            IPRExpressionKind::BinOpExpr(BinOp::Add, _, rhs) => assert!(matches!(
                kind(rhs),
                IPRExpressionKind::BinOpExpr(BinOp::Mul, _, _)
            )),
            other => panic!("expected Add at root, got {other:?}"),
        }
    }

    #[test]
    fn prec_add_over_shift() {
        // a << b + c  =>  Lsh(a, Add(b, c))
        match kind(&parse_expr("a << b + c")) {
            IPRExpressionKind::BinOpExpr(BinOp::Lsh, _, rhs) => assert!(matches!(
                kind(rhs),
                IPRExpressionKind::BinOpExpr(BinOp::Add, _, _)
            )),
            other => panic!("expected Lsh at root, got {other:?}"),
        }
    }

    #[test]
    fn prec_shift_over_cmp() {
        // a < b << c  =>  LT(a, Lsh(b, c))
        match kind(&parse_expr("a < b << c")) {
            IPRExpressionKind::BinOpExpr(BinOp::LT, _, rhs) => assert!(matches!(
                kind(rhs),
                IPRExpressionKind::BinOpExpr(BinOp::Lsh, _, _)
            )),
            other => panic!("expected LT at root, got {other:?}"),
        }
    }

    #[test]
    fn prec_cmp_over_eq() {
        // a == b < c  =>  Eq(a, LT(b, c))
        match kind(&parse_expr("a == b < c")) {
            IPRExpressionKind::BinOpExpr(BinOp::Eq, _, rhs) => assert!(matches!(
                kind(rhs),
                IPRExpressionKind::BinOpExpr(BinOp::LT, _, _)
            )),
            other => panic!("expected Eq at root, got {other:?}"),
        }
    }

    #[test]
    fn prec_eq_over_bitand() {
        // a & b == c  =>  BitAnd(a, Eq(b, c))
        match kind(&parse_expr("a & b == c")) {
            IPRExpressionKind::BinOpExpr(BinOp::BitAnd, _, rhs) => assert!(matches!(
                kind(rhs),
                IPRExpressionKind::BinOpExpr(BinOp::Eq, _, _)
            )),
            other => panic!("expected BitAnd at root, got {other:?}"),
        }
    }

    #[test]
    fn prec_bitand_over_bitxor() {
        // a ^ b & c  =>  BitXor(a, BitAnd(b, c))
        match kind(&parse_expr("a ^ b & c")) {
            IPRExpressionKind::BinOpExpr(BinOp::BitXor, _, rhs) => assert!(matches!(
                kind(rhs),
                IPRExpressionKind::BinOpExpr(BinOp::BitAnd, _, _)
            )),
            other => panic!("expected BitXor at root, got {other:?}"),
        }
    }

    #[test]
    fn prec_bitxor_over_bitor() {
        // a | b ^ c  =>  BitOr(a, BitXor(b, c))
        match kind(&parse_expr("a | b ^ c")) {
            IPRExpressionKind::BinOpExpr(BinOp::BitOr, _, rhs) => assert!(matches!(
                kind(rhs),
                IPRExpressionKind::BinOpExpr(BinOp::BitXor, _, _)
            )),
            other => panic!("expected BitOr at root, got {other:?}"),
        }
    }

    #[test]
    fn prec_bitor_over_logand() {
        // a && b | c  =>  LogAnd(a, BitOr(b, c))
        match kind(&parse_expr("a && b | c")) {
            IPRExpressionKind::BinOpExpr(BinOp::LogAnd, _, rhs) => assert!(matches!(
                kind(rhs),
                IPRExpressionKind::BinOpExpr(BinOp::BitOr, _, _)
            )),
            other => panic!("expected LogAnd at root, got {other:?}"),
        }
    }

    #[test]
    fn prec_logand_over_logxor() {
        // a ^^ b && c  =>  LogXor(a, LogAnd(b, c))
        match kind(&parse_expr("a ^^ b && c")) {
            IPRExpressionKind::BinOpExpr(BinOp::LogXor, _, rhs) => assert!(matches!(
                kind(rhs),
                IPRExpressionKind::BinOpExpr(BinOp::LogAnd, _, _)
            )),
            other => panic!("expected LogXor at root, got {other:?}"),
        }
    }

    #[test]
    fn prec_logxor_over_logor() {
        // a || b ^^ c  =>  LogOr(a, LogXor(b, c))
        match kind(&parse_expr("a || b ^^ c")) {
            IPRExpressionKind::BinOpExpr(BinOp::LogOr, _, rhs) => assert!(matches!(
                kind(rhs),
                IPRExpressionKind::BinOpExpr(BinOp::LogXor, _, _)
            )),
            other => panic!("expected LogOr at root, got {other:?}"),
        }
    }

    #[test]
    fn prec_unary_over_mul() {
        // -a * b  =>  Mul(Neg(a), b)
        match kind(&parse_expr("-a * b")) {
            IPRExpressionKind::BinOpExpr(BinOp::Mul, lhs, _) => {
                assert!(matches!(
                    kind(lhs),
                    IPRExpressionKind::UnOpExpr(UnOp::Neg, _)
                ))
            }
            other => panic!("expected Mul at root, got {other:?}"),
        }
    }

    #[test]
    fn prec_postfix_over_unary() {
        // -a.b  =>  Neg(MemberAccess(a, b)),  NOT  MemberAccess(Neg(a), b)
        match kind(&parse_expr("-a.b")) {
            IPRExpressionKind::UnOpExpr(UnOp::Neg, inner) => {
                assert!(matches!(kind(inner), IPRExpressionKind::MemberAccess(_, _)))
            }
            other => panic!("expected Neg at root, got {other:?}"),
        }
    }

    // ── associativity ─────────────────────────────────────────────────────────

    #[test]
    fn left_assoc_add() {
        // a + b + c  =>  Add(Add(a, b), c)
        match kind(&parse_expr("a + b + c")) {
            IPRExpressionKind::BinOpExpr(BinOp::Add, lhs, _) => assert!(matches!(
                kind(lhs),
                IPRExpressionKind::BinOpExpr(BinOp::Add, _, _)
            )),
            other => panic!("expected Add(Add, _), got {other:?}"),
        }
    }

    #[test]
    fn left_assoc_sub() {
        // a - b - c  =>  Sub(Sub(a, b), c), i.e. NOT a-(b-c)
        match kind(&parse_expr("a - b - c")) {
            IPRExpressionKind::BinOpExpr(BinOp::Sub, lhs, _) => assert!(matches!(
                kind(lhs),
                IPRExpressionKind::BinOpExpr(BinOp::Sub, _, _)
            )),
            other => panic!("expected Sub(Sub, _), got {other:?}"),
        }
    }

    #[test]
    fn left_assoc_mul() {
        match kind(&parse_expr("a * b * c")) {
            IPRExpressionKind::BinOpExpr(BinOp::Mul, lhs, _) => assert!(matches!(
                kind(lhs),
                IPRExpressionKind::BinOpExpr(BinOp::Mul, _, _)
            )),
            other => panic!("expected Mul(Mul, _), got {other:?}"),
        }
    }

    // ── postfix ───────────────────────────────────────────────────────────────

    #[test]
    fn member_access() {
        assert!(matches!(kind(&parse_expr("foo.bar")),
            IPRExpressionKind::MemberAccess(_, m) if m == "bar"));
    }

    #[test]
    fn chained_member_access() {
        // a.b.c  =>  MemberAccess(MemberAccess(a, b), c)
        match kind(&parse_expr("a.b.c")) {
            IPRExpressionKind::MemberAccess(inner, c) => {
                assert_eq!(c, "c");
                assert!(matches!(kind(inner), IPRExpressionKind::MemberAccess(_, b) if b == "b"));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn subscript() {
        assert!(matches!(
            kind(&parse_expr("arr[0]")),
            IPRExpressionKind::BinOpExpr(BinOp::Subscript, _, _)
        ));
    }

    #[test]
    fn chained_subscript() {
        // a[0][1]  =>  Subscript(Subscript(a, 0), 1)
        match kind(&parse_expr("a[0][1]")) {
            IPRExpressionKind::BinOpExpr(BinOp::Subscript, lhs, _) => assert!(matches!(
                kind(lhs),
                IPRExpressionKind::BinOpExpr(BinOp::Subscript, _, _)
            )),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn member_then_subscript() {
        // foo.bar[0]  =>  Subscript(MemberAccess(foo, bar), 0)
        match kind(&parse_expr("foo.bar[0]")) {
            IPRExpressionKind::BinOpExpr(BinOp::Subscript, lhs, _) => {
                assert!(matches!(kind(lhs), IPRExpressionKind::MemberAccess(_, _)))
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    // ── function calls ────────────────────────────────────────────────────────

    #[test]
    fn call_no_args() {
        assert!(matches!(kind(&parse_expr("foo()")),
            IPRExpressionKind::FunctionCall(IPRFunctionCall { args, .. }) if args.is_empty()));
    }

    // Comma0 — all four trailing-comma variants

    #[test]
    fn call_one_arg_no_trailing() {
        match kind(&parse_expr("foo(1)")) {
            IPRExpressionKind::FunctionCall(fc) => assert_eq!(fc.args.len(), 1),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn call_one_arg_trailing_comma() {
        match kind(&parse_expr("foo(1,)")) {
            IPRExpressionKind::FunctionCall(fc) => assert_eq!(fc.args.len(), 1),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn call_many_args_no_trailing() {
        match kind(&parse_expr("add(a, b, c)")) {
            IPRExpressionKind::FunctionCall(fc) => assert_eq!(fc.args.len(), 3),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn call_many_args_trailing_comma() {
        match kind(&parse_expr("add(a, b, c,)")) {
            IPRExpressionKind::FunctionCall(fc) => assert_eq!(fc.args.len(), 3),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn call_expr_arg() {
        match kind(&parse_expr("f(a + b)")) {
            IPRExpressionKind::FunctionCall(fc) => assert!(matches!(
                kind(&fc.args[0]),
                IPRExpressionKind::BinOpExpr(BinOp::Add, _, _)
            )),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn call_name_recorded() {
        match kind(&parse_expr("my-func(x)")) {
            IPRExpressionKind::FunctionCall(fc) => assert!(
                matches!(&fc.subject.kind, IPRExpressionKind::UnScopedIdent(s) if s == "my-func")
            ),
            other => panic!("unexpected {other:?}"),
        }
    }

    // ── assignment patterns ───────────────────────────────────────────────────

    #[test]
    fn pat_identifier() {
        assert!(matches!(parse_pat("x"), IPRAssignmentPattern::Identifier(s) if s == "x"));
    }

    #[test]
    fn pat_tuple_no_trailing() {
        assert!(matches!(parse_pat("@(a, b)"), IPRAssignmentPattern::Tuple(v) if v.len() == 2));
    }

    #[test]
    fn pat_tuple_trailing_comma() {
        assert!(matches!(parse_pat("@(a, b,)"), IPRAssignmentPattern::Tuple(v) if v.len() == 2));
    }

    #[test]
    fn pat_nested_tuple() {
        match parse_pat("@((a, b), c)") {
            IPRAssignmentPattern::Tuple(outer) => {
                assert_eq!(outer.len(), 2);
                assert!(matches!(&outer[0], IPRAssignmentPattern::Tuple(_)));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    // ── initialisations ───────────────────────────────────────────────────────

    #[test]
    fn init_inferred() {
        let i = parse_init("x := 42;");
        assert!(matches!(i.kind, IPRInitializationKind::Packed(_)));
        let (IPRInitializationKind::Packed(i)) = i.kind else {
            unreachable!()
        };
        assert!(matches!(&i.assignee, IPRAssignmentPattern::Identifier(s) if s == "x"));
    }

    #[test]
    fn init_explicit_type() {
        let i = parse_init("x : U64 = 42;");
        assert!(matches!(i.kind, IPRInitializationKind::Packed(_)));
        let (IPRInitializationKind::Packed(i)) = i.kind else {
            unreachable!()
        };
        assert!(matches!(&i.typ, Some(IPRTypeSpecifier::NonScalar(t)) if t == "U64"));
    }

    #[test]
    fn init_pointer_type() {
        let i = parse_init("p : U8* = ptr;");
        assert!(matches!(i.kind, IPRInitializationKind::Packed(_)));
        let (IPRInitializationKind::Packed(i)) = i.kind else {
            unreachable!()
        };
        assert!(matches!(&i.typ, Some(IPRTypeSpecifier::Pointer(inner))
            if matches!(inner.as_ref(), IPRTypeSpecifier::NonScalar(t) if t == "U8")));
    }

    #[test]
    fn init_array_type() {
        let i = parse_init("xs : [U32] = arr;");
        assert!(matches!(i.kind, IPRInitializationKind::Packed(_)));
        let (IPRInitializationKind::Packed(i)) = i.kind else {
            unreachable!()
        };
        assert!(matches!(&i.typ, Some(IPRTypeSpecifier::ArrayOf(_))));
    }

    #[test]
    fn init_tuple_destructure() {
        let i = parse_init("@(a, b) := pair;");
        assert!(matches!(i.kind, IPRInitializationKind::Packed(_)));
        let (IPRInitializationKind::Packed(i)) = i.kind else {
            unreachable!()
        };
        assert!(matches!(&i.assignee, IPRAssignmentPattern::Tuple(v) if v.len() == 2));
    }

    // ── statements ────────────────────────────────────────────────────────────

    #[test]
    fn stmt_return_literal() {
        assert!(matches!(
            parse_stmt("return 0;").kind,
            IPRStatementKind::Return(_)
        ));
    }

    #[test]
    fn stmt_return_expr() {
        match parse_stmt("return a + b;").kind {
            IPRStatementKind::Return(e) => assert!(matches!(
                kind(&e),
                IPRExpressionKind::BinOpExpr(BinOp::Add, _, _)
            )),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn stmt_init() {
        assert!(matches!(
            parse_stmt("x := 1;").kind,
            IPRStatementKind::Initialization(_)
        ));
    }

    #[test]
    fn stmt_reassign() {
        assert!(matches!(
            parse_stmt("x = 2;").kind,
            IPRStatementKind::Reassignment(_)
        ));
    }

    // ── functions ─────────────────────────────────────────────────────────────

    #[test]
    fn func_no_args_void_return() {
        let f = parse_func("fn greet() {}");
        assert_eq!(f.name, "greet");
        assert!(f.params.is_empty());
        assert!(matches!(&f.returns, IPRTypeSpecifier::NonScalar(s) if s == "Void"));
    }

    #[test]
    fn func_args_no_trailing() {
        let f = parse_func("fn add(a: U64, b: U64) -> U64 { return a + b; }");
        assert_eq!(f.params.len(), 2);
        assert_eq!(f.params[0].name, "a");
        assert_eq!(f.params[1].name, "b");
    }

    #[test]
    fn func_args_trailing_comma() {
        let f = parse_func("fn add(a: U64, b: U64,) -> U64 { return a + b; }");
        assert_eq!(f.params.len(), 2);
    }

    #[test]
    fn func_return_pointer() {
        let f = parse_func("fn foo() -> U64* {}");
        assert!(matches!(&f.returns, IPRTypeSpecifier::Pointer(inner)
            if matches!(inner.as_ref(), IPRTypeSpecifier::NonScalar(s) if s == "U64")));
    }

    #[test]
    fn func_return_array() {
        let f = parse_func("fn foo() -> [U8] {}");
        assert!(matches!(&f.returns, IPRTypeSpecifier::ArrayOf(_)));
    }

    #[test]
    fn func_body_statements() {
        let f = parse_func("fn foo() { x := 1; y := 2; }");
        assert_eq!(f.body.statements.len(), 2);
    }

    #[test]
    fn func_body_tail_expr() {
        let f = parse_func("fn foo() -> U64 { x := 1; x }");
        assert!(matches!(
            f.body.statements.last().unwrap().kind,
            IPRStatementKind::BlockTail(_)
        ));
    }

    #[test]
    fn func_empty_body() {
        assert!(parse_func("fn noop() {}").body.statements.is_empty());
    }

    // ── modules ───────────────────────────────────────────────────────────────

    #[test]
    fn module_minimal() {
        let m = parse_mod("module mymod");
        assert!(m.imports.is_empty());
        assert!(m.exports.is_empty());
        assert!(m.global_vars.is_empty());
        assert!(m.functions.is_empty());
    }

    #[test]
    fn module_imports_no_trailing() {
        let m = parse_mod("module mymod imports { foo, bar }");
        assert_eq!(m.imports, vec!["foo", "bar"]);
    }

    #[test]
    fn module_imports_trailing_comma() {
        let m = parse_mod("module mymod imports { foo, bar, }");
        assert_eq!(m.imports, vec!["foo", "bar"]);
    }

    #[test]
    fn module_exports_no_trailing() {
        let m = parse_mod("module mymod exports { baz }");
        assert_eq!(m.exports, vec!["baz"]);
    }

    #[test]
    fn module_exports_trailing_comma() {
        let m = parse_mod("module mymod exports { baz, }");
        assert_eq!(m.exports, vec!["baz"]);
    }

    #[test]
    fn module_with_global() {
        let m = parse_mod("module mymod x := 42;");
        assert_eq!(m.global_vars.len(), 1);
    }

    #[test]
    fn module_with_function() {
        let m = parse_mod("module mymod fn greet() {}");
        assert_eq!(m.functions.len(), 1);
        assert_eq!(m.functions[0].name, "greet");
    }

    #[test]
    fn module_multiple_functions() {
        let m = parse_mod("module mymod fn a() {} fn b() {}");
        assert_eq!(m.functions.len(), 2);
    }

    #[test]
    fn module_full() {
        let src = r#"
            module mymod
            imports { io, math }
            exports { main }
            max-val : U64 = 100;
            fn main() {
                x := max-val;
                io-print(x);
            }
        "#;
        let m = parse_mod(src);
        assert_eq!(m.imports, vec!["io", "math"]);
        assert_eq!(m.exports, vec!["main"]);
        assert_eq!(m.global_vars.len(), 1);
        assert_eq!(m.functions.len(), 1);
    }

    // ── reject cases ──────────────────────────────────────────────────────────

    macro_rules! reject {
        ($name:ident, $parser:ident, $src:expr) => {
            #[test]
            fn $name() {
                assert!(
                    $parser::new().parse($src).is_err(),
                    "expected parse failure for {:?}",
                    $src
                );
            }
        };
    }

    reject!(reject_uppercase_ident, ExprParser, "Foo");
    reject!(reject_bare_plus, ExprParser, "+ 1");
    reject!(reject_unclosed_call, ExprParser, "foo(1");
    reject!(reject_unclosed_subscript, ExprParser, "a[0");
    reject!(reject_missing_semicolon, StmtParser, "return 0");
    reject!(reject_reassign_no_semi, StmtParser, "x = 1");
    reject!(reject_init_no_semi, InitParser, "x := 1");
    reject!(reject_empty_tuple_pat, InitParser, "() := x;");
    reject!(reject_double_colon_eq, InitParser, "x ::= 1;");
    reject!(reject_func_missing_brace, FuncParser, "fn foo()");
    reject!(reject_module_no_name, ModParser, "module");
}
