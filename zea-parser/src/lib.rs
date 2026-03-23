#![allow(unused)]

mod grammar;
pub use grammar::ExprParser as ExpressionParser;
pub use grammar::ModParser as ModuleParser;
pub use grammar::StmtParser as StatementParser;
use zea_ast::zea::{BinOp, Expression, ExpressionKind, Function, Initialisation, UnOp};

#[derive(Default, Clone, Copy)]
pub struct NodeIdGenerator {
    cur: usize,
}

pub(crate) enum ModuleItem {
    Init(Initialisation),
    Func(Function),
}

pub(crate) fn separate(items: Vec<ModuleItem>) -> (Vec<Initialisation>, Vec<Function>) {
    let mut globs = vec![];
    let mut funcs = vec![];
    for item in items {
        match item {
            ModuleItem::Init(i) => globs.push(i),
            ModuleItem::Func(f) => funcs.push(f),
        }
    }
    (globs, funcs)
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

fn binop(g: &mut NodeIdGenerator, op: BinOp, l: Expression, r: Expression) -> Expression {
    Expression {
        id: g.get(),
        kind: ExpressionKind::BinOpExpr(op, Box::new(l), Box::new(r)),
    }
}
fn unop(g: &mut NodeIdGenerator, op: UnOp, e: Expression) -> Expression {
    Expression {
        id: g.get(),
        kind: ExpressionKind::UnOpExpr(op, Box::new(e)),
    }
}

#[cfg(test)]
mod tests {
    use crate::NodeIdGenerator;
    use crate::grammar::{
        AssignPatParser, ExprParser, FuncParser, InitParser, ModParser, StmtParser,
    };
    use zea_ast::zea::*;

    // ── helpers ───────────────────────────────────────────────────────────────

    fn idgen() -> NodeIdGenerator {
        NodeIdGenerator::new()
    }

    fn parse_expr(src: &str) -> Expression {
        ExprParser::new()
            .parse(&mut idgen(), src)
            .unwrap_or_else(|e| panic!("expr parse failed for {src:?}:\n  {e}"))
    }

    fn parse_stmt(src: &str) -> Statement {
        StmtParser::new()
            .parse(&mut idgen(), src)
            .unwrap_or_else(|e| panic!("stmt parse failed for {src:?}:\n  {e}"))
    }

    fn parse_func(src: &str) -> Function {
        FuncParser::new()
            .parse(&mut idgen(), src)
            .unwrap_or_else(|e| panic!("func parse failed for {src:?}:\n  {e}"))
    }

    fn parse_init(src: &str) -> Initialisation {
        InitParser::new()
            .parse(&mut idgen(), src)
            .unwrap_or_else(|e| panic!("init parse failed for {src:?}:\n  {e}"))
    }

    fn parse_mod(src: &str) -> Module {
        ModParser::new()
            .parse(&mut idgen(), src)
            .unwrap_or_else(|e| panic!("module parse failed for {src:?}:\n  {e}"))
    }

    fn parse_pat(src: &str) -> AssignmentPattern {
        AssignPatParser::new()
            .parse(&mut idgen(), src)
            .unwrap_or_else(|e| panic!("pattern parse failed for {src:?}:\n  {e}"))
    }

    fn kind(e: &Expression) -> &ExpressionKind {
        &e.kind
    }

    // ── atoms ─────────────────────────────────────────────────────────────────

    #[test]
    fn integer_decimal() {
        assert!(matches!(
            kind(&parse_expr("42")),
            ExpressionKind::IntegerLiteral(42)
        ));
    }

    #[test]
    fn integer_hex() {
        assert!(matches!(
            kind(&parse_expr("0xFF")),
            ExpressionKind::IntegerLiteral(255)
        ));
    }

    #[test]
    fn integer_hex_lowercase() {
        assert!(matches!(
            kind(&parse_expr("0xff")),
            ExpressionKind::IntegerLiteral(255)
        ));
    }

    #[test]
    fn integer_binary() {
        assert!(matches!(
            kind(&parse_expr("0b1010")),
            ExpressionKind::IntegerLiteral(10)
        ));
    }

    #[test]
    fn integer_decimal_underscores() {
        assert!(matches!(
            kind(&parse_expr("1_000_000")),
            ExpressionKind::IntegerLiteral(1_000_000)
        ));
    }

    #[test]
    fn integer_hex_underscores() {
        assert!(matches!(
            kind(&parse_expr("0xFF_FF")),
            ExpressionKind::IntegerLiteral(0xFFFF)
        ));
    }

    #[test]
    fn integer_binary_underscores() {
        assert!(matches!(
            kind(&parse_expr("0b1111_0000")),
            ExpressionKind::IntegerLiteral(0b1111_0000)
        ));
    }

    #[test]
    fn identifier_simple() {
        assert!(matches!(kind(&parse_expr("foo")), ExpressionKind::Ident(s) if s == "foo"));
    }

    #[test]
    fn identifier_with_digits() {
        assert!(matches!(kind(&parse_expr("foo123")), ExpressionKind::Ident(s) if s == "foo123"));
    }

    #[test]
    fn identifier_with_hyphens() {
        assert!(matches!(kind(&parse_expr("my-var")), ExpressionKind::Ident(s) if s == "my-var"));
    }

    #[test]
    fn identifier_question_mark() {
        assert!(matches!(kind(&parse_expr("empty?")), ExpressionKind::Ident(s) if s == "empty?"));
    }

    #[test]
    fn identifier_bang() {
        assert!(matches!(kind(&parse_expr("reset!")), ExpressionKind::Ident(s) if s == "reset!"));
    }

    // ── unary ops ─────────────────────────────────────────────────────────────

    #[test]
    fn unary_negate() {
        assert!(matches!(
            kind(&parse_expr("-1")),
            ExpressionKind::UnOpExpr(UnOp::Neg, _)
        ));
    }

    #[test]
    fn unary_logical_not() {
        assert!(matches!(
            kind(&parse_expr("!x")),
            ExpressionKind::UnOpExpr(UnOp::LogNot, _)
        ));
    }

    #[test]
    fn unary_bitwise_not() {
        assert!(matches!(
            kind(&parse_expr("~x")),
            ExpressionKind::UnOpExpr(UnOp::BitNot, _)
        ));
    }

    #[test]
    fn unary_chained() {
        // !!x — outer LogNot wrapping inner LogNot
        let e = parse_expr("!!x");
        assert!(matches!(kind(&e),
            ExpressionKind::UnOpExpr(UnOp::LogNot, inner)
            if matches!(kind(inner), ExpressionKind::UnOpExpr(UnOp::LogNot, _))
        ));
    }

    // ── binary ops — one test per tier ───────────────────────────────────────

    macro_rules! binop_test {
        ($name:ident, $src:expr, $op:pat) => {
            #[test]
            fn $name() {
                assert!(matches!(
                    kind(&parse_expr($src)),
                    ExpressionKind::BinOpExpr($op, _, _)
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
            ExpressionKind::BinOpExpr(BinOp::Add, _, rhs) => assert!(matches!(
                kind(rhs),
                ExpressionKind::BinOpExpr(BinOp::Mul, _, _)
            )),
            other => panic!("expected Add at root, got {other:?}"),
        }
    }

    #[test]
    fn prec_add_over_shift() {
        // a << b + c  =>  Lsh(a, Add(b, c))
        match kind(&parse_expr("a << b + c")) {
            ExpressionKind::BinOpExpr(BinOp::Lsh, _, rhs) => assert!(matches!(
                kind(rhs),
                ExpressionKind::BinOpExpr(BinOp::Add, _, _)
            )),
            other => panic!("expected Lsh at root, got {other:?}"),
        }
    }

    #[test]
    fn prec_shift_over_cmp() {
        // a < b << c  =>  LT(a, Lsh(b, c))
        match kind(&parse_expr("a < b << c")) {
            ExpressionKind::BinOpExpr(BinOp::LT, _, rhs) => assert!(matches!(
                kind(rhs),
                ExpressionKind::BinOpExpr(BinOp::Lsh, _, _)
            )),
            other => panic!("expected LT at root, got {other:?}"),
        }
    }

    #[test]
    fn prec_cmp_over_eq() {
        // a == b < c  =>  Eq(a, LT(b, c))
        match kind(&parse_expr("a == b < c")) {
            ExpressionKind::BinOpExpr(BinOp::Eq, _, rhs) => assert!(matches!(
                kind(rhs),
                ExpressionKind::BinOpExpr(BinOp::LT, _, _)
            )),
            other => panic!("expected Eq at root, got {other:?}"),
        }
    }

    #[test]
    fn prec_eq_over_bitand() {
        // a & b == c  =>  BitAnd(a, Eq(b, c))
        match kind(&parse_expr("a & b == c")) {
            ExpressionKind::BinOpExpr(BinOp::BitAnd, _, rhs) => assert!(matches!(
                kind(rhs),
                ExpressionKind::BinOpExpr(BinOp::Eq, _, _)
            )),
            other => panic!("expected BitAnd at root, got {other:?}"),
        }
    }

    #[test]
    fn prec_bitand_over_bitxor() {
        // a ^ b & c  =>  BitXor(a, BitAnd(b, c))
        match kind(&parse_expr("a ^ b & c")) {
            ExpressionKind::BinOpExpr(BinOp::BitXor, _, rhs) => assert!(matches!(
                kind(rhs),
                ExpressionKind::BinOpExpr(BinOp::BitAnd, _, _)
            )),
            other => panic!("expected BitXor at root, got {other:?}"),
        }
    }

    #[test]
    fn prec_bitxor_over_bitor() {
        // a | b ^ c  =>  BitOr(a, BitXor(b, c))
        match kind(&parse_expr("a | b ^ c")) {
            ExpressionKind::BinOpExpr(BinOp::BitOr, _, rhs) => assert!(matches!(
                kind(rhs),
                ExpressionKind::BinOpExpr(BinOp::BitXor, _, _)
            )),
            other => panic!("expected BitOr at root, got {other:?}"),
        }
    }

    #[test]
    fn prec_bitor_over_logand() {
        // a && b | c  =>  LogAnd(a, BitOr(b, c))
        match kind(&parse_expr("a && b | c")) {
            ExpressionKind::BinOpExpr(BinOp::LogAnd, _, rhs) => assert!(matches!(
                kind(rhs),
                ExpressionKind::BinOpExpr(BinOp::BitOr, _, _)
            )),
            other => panic!("expected LogAnd at root, got {other:?}"),
        }
    }

    #[test]
    fn prec_logand_over_logxor() {
        // a ^^ b && c  =>  LogXor(a, LogAnd(b, c))
        match kind(&parse_expr("a ^^ b && c")) {
            ExpressionKind::BinOpExpr(BinOp::LogXor, _, rhs) => assert!(matches!(
                kind(rhs),
                ExpressionKind::BinOpExpr(BinOp::LogAnd, _, _)
            )),
            other => panic!("expected LogXor at root, got {other:?}"),
        }
    }

    #[test]
    fn prec_logxor_over_logor() {
        // a || b ^^ c  =>  LogOr(a, LogXor(b, c))
        match kind(&parse_expr("a || b ^^ c")) {
            ExpressionKind::BinOpExpr(BinOp::LogOr, _, rhs) => assert!(matches!(
                kind(rhs),
                ExpressionKind::BinOpExpr(BinOp::LogXor, _, _)
            )),
            other => panic!("expected LogOr at root, got {other:?}"),
        }
    }

    #[test]
    fn prec_unary_over_mul() {
        // -a * b  =>  Mul(Neg(a), b)
        match kind(&parse_expr("-a * b")) {
            ExpressionKind::BinOpExpr(BinOp::Mul, lhs, _) => {
                assert!(matches!(kind(lhs), ExpressionKind::UnOpExpr(UnOp::Neg, _)))
            }
            other => panic!("expected Mul at root, got {other:?}"),
        }
    }

    #[test]
    fn prec_postfix_over_unary() {
        // -a.b  =>  Neg(MemberAccess(a, b)),  NOT  MemberAccess(Neg(a), b)
        match kind(&parse_expr("-a.b")) {
            ExpressionKind::UnOpExpr(UnOp::Neg, inner) => {
                assert!(matches!(kind(inner), ExpressionKind::MemberAccess(_, _)))
            }
            other => panic!("expected Neg at root, got {other:?}"),
        }
    }

    // ── associativity ─────────────────────────────────────────────────────────

    #[test]
    fn left_assoc_add() {
        // a + b + c  =>  Add(Add(a, b), c)
        match kind(&parse_expr("a + b + c")) {
            ExpressionKind::BinOpExpr(BinOp::Add, lhs, _) => assert!(matches!(
                kind(lhs),
                ExpressionKind::BinOpExpr(BinOp::Add, _, _)
            )),
            other => panic!("expected Add(Add, _), got {other:?}"),
        }
    }

    #[test]
    fn left_assoc_sub() {
        // a - b - c  =>  Sub(Sub(a, b), c), i.e. NOT a-(b-c)
        match kind(&parse_expr("a - b - c")) {
            ExpressionKind::BinOpExpr(BinOp::Sub, lhs, _) => assert!(matches!(
                kind(lhs),
                ExpressionKind::BinOpExpr(BinOp::Sub, _, _)
            )),
            other => panic!("expected Sub(Sub, _), got {other:?}"),
        }
    }

    #[test]
    fn left_assoc_mul() {
        match kind(&parse_expr("a * b * c")) {
            ExpressionKind::BinOpExpr(BinOp::Mul, lhs, _) => assert!(matches!(
                kind(lhs),
                ExpressionKind::BinOpExpr(BinOp::Mul, _, _)
            )),
            other => panic!("expected Mul(Mul, _), got {other:?}"),
        }
    }

    // ── postfix ───────────────────────────────────────────────────────────────

    #[test]
    fn member_access() {
        assert!(matches!(kind(&parse_expr("foo.bar")),
            ExpressionKind::MemberAccess(_, m) if m == "bar"));
    }

    #[test]
    fn chained_member_access() {
        // a.b.c  =>  MemberAccess(MemberAccess(a, b), c)
        match kind(&parse_expr("a.b.c")) {
            ExpressionKind::MemberAccess(inner, c) => {
                assert_eq!(c, "c");
                assert!(matches!(kind(inner), ExpressionKind::MemberAccess(_, b) if b == "b"));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn subscript() {
        assert!(matches!(
            kind(&parse_expr("arr[0]")),
            ExpressionKind::BinOpExpr(BinOp::Subscript, _, _)
        ));
    }

    #[test]
    fn chained_subscript() {
        // a[0][1]  =>  Subscript(Subscript(a, 0), 1)
        match kind(&parse_expr("a[0][1]")) {
            ExpressionKind::BinOpExpr(BinOp::Subscript, lhs, _) => assert!(matches!(
                kind(lhs),
                ExpressionKind::BinOpExpr(BinOp::Subscript, _, _)
            )),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn member_then_subscript() {
        // foo.bar[0]  =>  Subscript(MemberAccess(foo, bar), 0)
        match kind(&parse_expr("foo.bar[0]")) {
            ExpressionKind::BinOpExpr(BinOp::Subscript, lhs, _) => {
                assert!(matches!(kind(lhs), ExpressionKind::MemberAccess(_, _)))
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    // ── function calls ────────────────────────────────────────────────────────

    #[test]
    fn call_no_args() {
        assert!(matches!(kind(&parse_expr("foo()")),
            ExpressionKind::FuncCall(FunctionCall { args, .. }) if args.is_empty()));
    }

    // Comma0 — all four trailing-comma variants

    #[test]
    fn call_one_arg_no_trailing() {
        match kind(&parse_expr("foo(1)")) {
            ExpressionKind::FuncCall(fc) => assert_eq!(fc.args.len(), 1),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn call_one_arg_trailing_comma() {
        match kind(&parse_expr("foo(1,)")) {
            ExpressionKind::FuncCall(fc) => assert_eq!(fc.args.len(), 1),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn call_many_args_no_trailing() {
        match kind(&parse_expr("add(a, b, c)")) {
            ExpressionKind::FuncCall(fc) => assert_eq!(fc.args.len(), 3),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn call_many_args_trailing_comma() {
        match kind(&parse_expr("add(a, b, c,)")) {
            ExpressionKind::FuncCall(fc) => assert_eq!(fc.args.len(), 3),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn call_expr_arg() {
        match kind(&parse_expr("f(a + b)")) {
            ExpressionKind::FuncCall(fc) => assert!(matches!(
                kind(&fc.args[0]),
                ExpressionKind::BinOpExpr(BinOp::Add, _, _)
            )),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn call_name_recorded() {
        match kind(&parse_expr("my-func(x)")) {
            ExpressionKind::FuncCall(fc) => assert_eq!(fc.name, "my-func"),
            other => panic!("unexpected {other:?}"),
        }
    }

    // ── assignment patterns ───────────────────────────────────────────────────

    #[test]
    fn pat_identifier() {
        assert!(matches!(parse_pat("x"), AssignmentPattern::Identifier(s) if s == "x"));
    }

    #[test]
    fn pat_tuple_no_trailing() {
        assert!(matches!(parse_pat("(a, b)"), AssignmentPattern::Tuple(v) if v.len() == 2));
    }

    #[test]
    fn pat_tuple_trailing_comma() {
        assert!(matches!(parse_pat("(a, b,)"), AssignmentPattern::Tuple(v) if v.len() == 2));
    }

    #[test]
    fn pat_nested_tuple() {
        match parse_pat("((a, b), c)") {
            AssignmentPattern::Tuple(outer) => {
                assert_eq!(outer.len(), 2);
                assert!(matches!(&outer[0], AssignmentPattern::Tuple(_)));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    // ── initialisations ───────────────────────────────────────────────────────

    #[test]
    fn init_inferred() {
        let i = parse_init("x := 42;");
        assert!(i.typ.is_none());
        assert!(matches!(&i.assignee, AssignmentPattern::Identifier(s) if s == "x"));
    }

    #[test]
    fn init_explicit_type() {
        let i = parse_init("x : U64 = 42;");
        assert!(matches!(&i.typ, Some(Type::Basic(t)) if t == "U64"));
    }

    #[test]
    fn init_pointer_type() {
        let i = parse_init("p : U8* = ptr;");
        assert!(matches!(&i.typ, Some(Type::Pointer(inner))
            if matches!(inner.as_ref(), Type::Basic(t) if t == "U8")));
    }

    #[test]
    fn init_array_type() {
        let i = parse_init("xs : [U32] = arr;");
        assert!(matches!(&i.typ, Some(Type::ArrayOf(_))));
    }

    #[test]
    fn init_tuple_destructure() {
        let i = parse_init("(a, b) := pair;");
        assert!(matches!(&i.assignee, AssignmentPattern::Tuple(v) if v.len() == 2));
    }

    // ── statements ────────────────────────────────────────────────────────────

    #[test]
    fn stmt_return_literal() {
        assert!(matches!(
            parse_stmt("return 0;").kind,
            StatementKind::Return(_)
        ));
    }

    #[test]
    fn stmt_return_expr() {
        match parse_stmt("return a + b;").kind {
            StatementKind::Return(e) => assert!(matches!(
                kind(&e),
                ExpressionKind::BinOpExpr(BinOp::Add, _, _)
            )),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn stmt_init() {
        assert!(matches!(
            parse_stmt("x := 1;").kind,
            StatementKind::Initialisation(_)
        ));
    }

    #[test]
    fn stmt_reassign() {
        assert!(matches!(
            parse_stmt("x = 2;").kind,
            StatementKind::Reassignment(_)
        ));
    }

    #[test]
    fn stmt_nested_block() {
        assert!(matches!(
            parse_stmt("{ x := 1; }").kind,
            StatementKind::Block(_)
        ));
    }

    // ── functions ─────────────────────────────────────────────────────────────

    #[test]
    fn func_no_args_void_return() {
        let f = parse_func("fn greet() {}");
        assert_eq!(f.name, "greet");
        assert!(f.args.is_empty());
        assert!(matches!(&f.returns, Type::Basic(s) if s == "Void"));
    }

    #[test]
    fn func_args_no_trailing() {
        let f = parse_func("fn add(a: U64, b: U64) -> U64 { return a + b; }");
        assert_eq!(f.args.len(), 2);
        assert_eq!(f.args[0].name, "a");
        assert_eq!(f.args[1].name, "b");
    }

    #[test]
    fn func_args_trailing_comma() {
        let f = parse_func("fn add(a: U64, b: U64,) -> U64 { return a + b; }");
        assert_eq!(f.args.len(), 2);
    }

    #[test]
    fn func_return_pointer() {
        let f = parse_func("fn foo() -> U64* {}");
        assert!(matches!(&f.returns, Type::Pointer(inner)
            if matches!(inner.as_ref(), Type::Basic(s) if s == "U64")));
    }

    #[test]
    fn func_return_array() {
        let f = parse_func("fn foo() -> [U8] {}");
        assert!(matches!(&f.returns, Type::ArrayOf(_)));
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
            StatementKind::BlockTail(_)
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
        assert!(m.globs.is_empty());
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
        assert_eq!(m.globs.len(), 1);
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
        assert_eq!(m.globs.len(), 1);
        assert_eq!(m.functions.len(), 1);
    }

    // ── reject cases ──────────────────────────────────────────────────────────

    macro_rules! reject {
        ($name:ident, $parser:ident, $src:expr) => {
            #[test]
            fn $name() {
                assert!(
                    $parser::new().parse(&mut idgen(), $src).is_err(),
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
