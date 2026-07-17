use crate::zea::immediate_parsed_representation::*;
use crate::zea::IPRScopedIdentifier;
use std::fmt::Debug;

pub trait IndentPrint: Debug {
    fn indent_print(&self, depth: usize) -> String {
        format!("{self:?}").indent_print(depth)
    }
}

macro_rules! indent {
    ($d:expr) => {{
        let d: usize = $d;
        " ".repeat(d * 2)
    }};
}

fn add_prefix(s: &str, list_depth: usize) -> String {
    let list_indent = indent!(list_depth);
    let mut result = String::new();
    for (i, line) in s.lines().enumerate() {
        if i == 0 {
            let trimmed = line.trim_start();
            result += &format!("{}- {}\n", list_indent, trimmed);
        } else {
            let shifted = if line.len() >= 2 { &line[2..] } else { line };
            result += &format!("{}\n", shifted);
        }
    }
    result
}

fn fmt_list<T: IndentPrint>(items: &[T], depth: usize) -> String {
    let mut result = String::new();
    for item in items {
        let item_str = item.indent_print(depth + 1);
        result += &add_prefix(&item_str, depth);
    }
    result
}

impl IndentPrint for IPRFuncParam {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = ".param".indent_print(depth);
        buffer += &self.name.indent_print(depth + 1);
        buffer += &self.typ.indent_print(depth + 1);
        buffer
    }
}

fn module_imports(imports: &[String], depth: usize) -> String {
    let mut result = ".imports".indent_print(depth);
    for e in imports.iter() {
        result += &format!("{}- {}\n", indent!(depth + 1), e);
    }
    result
}
fn module_exports(imports: &[String], depth: usize) -> String {
    let mut result = ".exports".indent_print(depth);
    for e in imports.iter() {
        result += &format!("{}- {}\n", indent!(depth + 1), e);
    }
    result
}

fn module_globs(globs: &[IPRInitializationBlock], depth: usize) -> String {
    let mut result = ".globs".indent_print(depth);
    result += &fmt_list(globs, depth + 1);
    result
}

fn module_funcs(funcs: &[IPRFunction], depth: usize) -> String {
    let mut result = ".funcs".indent_print(depth);
    result += &fmt_list(funcs, depth + 1);
    result
}

impl IndentPrint for IPRModule {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = "module".indent_print(depth);
        buffer += &module_imports(&self.imports, depth + 1);
        buffer += &module_exports(&self.exports, depth + 1);
        buffer += &module_globs(&self.global_vars, depth + 1);
        buffer += &module_funcs(&self.functions, depth + 1);
        buffer
    }
}

fn func_params(params: &[IPRFuncParam], depth: usize) -> String {
    let mut buffer = ".params".indent_print(depth);
    buffer += &fmt_list(params, depth + 1);
    buffer
}

impl IndentPrint for IPRFunction {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = format!("func `{}`", self.name).indent_print(depth);
        buffer += &".returns".indent_print(depth + 1);
        buffer += &self.returns.indent_print(depth + 2);

        buffer += &func_params(&self.params, depth + 1);
        buffer += &".body".indent_print(depth + 1);
        buffer += &self.body.indent_print(depth + 2);
        buffer
    }
}

impl IndentPrint for IPRTypedIdentifier {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = self.name.indent_print(depth);
        buffer += &".type".indent_print(depth + 1);
        buffer += &self.typ.indent_print(depth + 2);
        buffer
    }
}

impl IndentPrint for IPRBlockExpression {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = "block".indent_print(depth);

        for s in self.statements.iter() {
            let stmt_str = s.indent_print(depth + 2);
            buffer += &add_prefix(&stmt_str, depth + 1);
        }
        buffer += &self.last.indent_print(depth + 1);

        buffer
    }
}

impl IndentPrint for IPRStatement {
    fn indent_print(&self, depth: usize) -> String {
        use IPRStatementKind;
        match &self.kind {
            IPRStatementKind::Return(e) => {
                "return".indent_print(depth) + &e.indent_print(depth + 1)
            }
            IPRStatementKind::Initialization(i) => i.indent_print(depth),
            IPRStatementKind::BlockTail(e) => {
                "tail".indent_print(depth) + &e.indent_print(depth + 1)
            }
            IPRStatementKind::IfThenElse(b) => b.indent_print(depth),
            IPRStatementKind::Block(eb) => eb.indent_print(depth),
            IPRStatementKind::FunctionCall(c) => c.indent_print(depth),
            _ => todo!("pretty print statement with kind {:?}", self.kind),
        }
    }
}
impl IndentPrint for IPRInitializationBlock {
    fn indent_print(&self, depth: usize) -> String {
        use IPRInitializationKind;
        match &self.kind {
            IPRInitializationKind::Packed(p) => p.indent_print(depth),

            IPRInitializationKind::Unpacked(p) => {
                let mut buffer = "init_unpacked_block".indent_print(depth);
                for init in p.iter() {
                    let init_str = init.indent_print(depth + 2);
                    buffer += &add_prefix(&init_str, depth + 1);
                }
                buffer
            }
        }
    }
}

impl IndentPrint for IPRPackedInitialization {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = "init_packed".indent_print(depth);
        buffer += &".pattern".indent_print(depth + 1);
        buffer += &self.assignee.indent_print(depth + 2);
        buffer += &".type".indent_print(depth + 1);
        buffer += &self.typ.indent_print(depth + 2);
        buffer += &".value".indent_print(depth + 1);
        buffer += &self.value.indent_print(depth + 2);
        buffer
    }
}

impl IndentPrint for String {
    fn indent_print(&self, depth: usize) -> String {
        indent!(depth) + self + "\n"
    }
}

impl IndentPrint for &str {
    fn indent_print(&self, depth: usize) -> String {
        indent!(depth) + self + "\n"
    }
}
impl IndentPrint for bool {
    fn indent_print(&self, depth: usize) -> String {
        format!("{self}").indent_print(depth)
    }
}

impl IndentPrint for IPRTypeSpecifier {
    fn indent_print(&self, depth: usize) -> String {
        format!("{:?}", self).indent_print(depth)
    }
}

impl IndentPrint for Option<IPRTypeSpecifier> {
    fn indent_print(&self, depth: usize) -> String {
        match self {
            None => String::from("@unknown").indent_print(depth),
            Some(t) => t.indent_print(depth),
        }
    }
}

impl IndentPrint for IPRSimpleInitialization {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = "init".indent_print(depth);
        buffer += &".assignee".indent_print(depth + 1);
        buffer += &self.assignee.indent_print(depth + 2);
        buffer += &".type".indent_print(depth + 1);
        buffer += &self.typ.indent_print(depth + 2);
        buffer += &".value".indent_print(depth + 1);
        buffer += &self.value.indent_print(depth + 2);
        buffer
    }
}

impl IndentPrint for IPRAssignmentPattern {
    fn indent_print(&self, depth: usize) -> String {
        match self {
            IPRAssignmentPattern::Identifier(i) => i.indent_print(depth),
            IPRAssignmentPattern::Tuple(tup) => {
                let mut buffer = "(".indent_print(depth);
                for pat in tup {
                    buffer += &pat.indent_print(depth + 1);
                }
                buffer += &")".indent_print(depth);
                buffer
            }
        }
    }
}

impl IndentPrint for IPRBranch {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = "branch".indent_print(depth);

        buffer += &".cond".indent_print(depth + 1);
        buffer += &self.condition.indent_print(depth + 2);

        buffer += &".then".indent_print(depth + 1);
        buffer += &self.true_case.indent_print(depth + 2);

        if let Some(e) = self.false_case.as_ref() {
            buffer += &".otherwise".indent_print(depth + 1);
            buffer += &e.indent_print(depth + 2);
        };

        buffer
    }
}

impl IndentPrint for IPRExpression {
    fn indent_print(&self, depth: usize) -> String {
        use IPRExpressionKind;
        match &self.kind {
            IPRExpressionKind::UnScopedIdent(i) => format!("ident({i})").indent_print(depth),
            IPRExpressionKind::IntegerLiteral(i) => format!("lit_int({i})").indent_print(depth),
            IPRExpressionKind::FloatLiteral(i) => format!("lit_float({i})").indent_print(depth),
            IPRExpressionKind::BinOpExpr(op, l, r) => {
                let mut buffer = format!("operator`{op:?}`").indent_print(depth);
                buffer += &l.indent_print(depth + 1);
                buffer += &r.indent_print(depth + 1);
                buffer
            }
            IPRExpressionKind::UnOpExpr(op, arg) => {
                let mut buffer = format!("operator`{op:?}`").indent_print(depth);
                buffer += &arg.indent_print(depth + 1);
                buffer
            }
            IPRExpressionKind::MemberAccess(e, m) => {
                let mut buffer = "expr_member".indent_print(depth);
                buffer += &e.indent_print(depth + 1);
                buffer += &".member".indent_print(depth + 1);
                buffer += &m.indent_print(depth + 2);
                buffer
            }
            IPRExpressionKind::IfThenElse(b) => b.indent_print(depth),

            IPRExpressionKind::Block(eb) => eb.indent_print(depth),
            IPRExpressionKind::FunctionCall(c) => c.indent_print(depth),
            IPRExpressionKind::Unit => "@unit".indent_print(depth),
            IPRExpressionKind::ScopedIdent(si) => si.indent_print(depth),
            IPRExpressionKind::BoolLiteral(b) => format!("bool_lit({b})").indent_print(depth),
            _ => todo!("pretty print expression of kind {:?}", self.kind),
        }
    }
}
impl IndentPrint for IPRScopedIdentifier {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = format!("{:?}", self.kind).indent_print(depth);
        buffer += &format!(".ident {}", self.ident).indent_print(depth + 1);
        buffer += &format!(".origin {}", self.origin).indent_print(depth + 1);

        buffer
    }
}

impl IndentPrint for IPRFunctionCall {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = "call".indent_print(depth);

        buffer += &".subject".indent_print(depth + 1);
        buffer += &self.subject.indent_print(depth + 2);

        if !self.args.is_empty() {
            buffer += &".args".indent_print(depth + 1);
            for arg in self.args.iter() {
                let arg_str = arg.indent_print(depth + 3);
                buffer += &add_prefix(&arg_str, depth + 2);
            }
        };
        buffer
    }
}
