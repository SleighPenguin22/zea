use crate::zea;
use crate::zea::{FuncParam, Function, InitializationBlock};
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

impl IndentPrint for FuncParam {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = ".param".indent_print(depth);
        buffer += &self.name.indent_print(depth + 1);
        buffer += &self.typ.indent_print(depth + 1);
        buffer
    }
}

fn module_imports(imports: &[String], depth: usize) -> String {
    if imports.is_empty() {
        ".imports nothing".indent_print(depth + 1)
    } else {
        let mut imp_buffer = ".imports".indent_print(depth + 1);
        for e in imports.iter() {
            imp_buffer += &e.indent_print(depth + 2);
        }
        imp_buffer
    }
}
fn module_exports(imports: &[String], depth: usize) -> String {
    if imports.is_empty() {
        ".exports nothing".indent_print(depth + 1)
    } else {
        let mut imp_buffer = ".exports".indent_print(depth + 1);
        for e in imports.iter() {
            imp_buffer += &e.indent_print(depth + 2);
        }
        imp_buffer
    }
}

fn module_globs(globs: &[InitializationBlock], depth: usize) -> String {
    if globs.is_empty() {
        ".globs none".indent_print(depth + 1)
    } else {
        let mut buffer = String::new();
        buffer += &".globs".indent_print(depth + 1);
        for glob in globs {
            buffer += &glob.indent_print(depth + 2);
        }
        buffer += &"/.globs".indent_print(depth + 1);
        buffer
    }
}

fn module_funcs(funcs: &[Function], depth: usize) -> String {
    if funcs.is_empty() {
        ".funcs none".indent_print(depth + 1)
    } else {
        let mut buffer = String::new();
        buffer += &".funcs".indent_print(depth + 1);
        for glob in funcs {
            buffer += &glob.indent_print(depth + 2);
        }
        buffer += &"/.funcs".indent_print(depth + 1);
        buffer
    }
}

impl IndentPrint for zea::Module {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = "module".indent_print(depth);
        let imports = module_imports(&self.imports, depth);
        buffer += &imports;

        let exports = module_exports(&self.exports, depth);
        buffer += &exports;

        buffer += &module_globs(&self.global_vars, depth);

        buffer += &module_funcs(&self.functions, depth);

        buffer += &"/module".indent_print(depth);
        buffer
    }
}

fn func_params(params: &[FuncParam], depth: usize) -> String {
    if params.is_empty() {
        ".params none".indent_print(depth)
    } else {
        let mut buffer = ".params".indent_print(depth + 1);
        for arg in params {
            buffer += &arg.indent_print(depth + 2);
        }
        buffer
    }
}

impl IndentPrint for zea::Function {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = format!("func `{}`", self.name).indent_print(depth);
        buffer += &".returns".indent_print(depth + 1);
        buffer += &self.returns.indent_print(depth + 2);

        buffer += &func_params(&self.params, depth + 1);
        buffer += &".body".indent_print(depth + 1);
        buffer += &self.body.indent_print(depth + 2);
        buffer += &format!("/func {}", self.name).indent_print(depth);
        buffer
    }
}

impl IndentPrint for zea::TypedIdentifier {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = self.name.indent_print(depth);
        buffer += &".type".indent_print(depth);
        buffer += &self.typ.indent_print(depth + 1);
        buffer
    }
}

impl IndentPrint for zea::StatementBlock {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = "block_sugared".indent_print(depth);

        for s in self.statements.iter() {
            buffer += &"-block_stmt".indent_print(depth + 1);
            buffer += &s.indent_print(depth + 1);
            buffer += &"/-block_stmt".indent_print(depth + 1);
        }
        buffer += &"/block_sugared".indent_print(depth);

        buffer
    }
}

impl IndentPrint for zea::ExpandedBlockExpr {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = "block".indent_print(depth);

        for s in self.statements.iter() {
            buffer += &"-block_stmt".indent_print(depth + 1);
            buffer += &s.indent_print(depth + 2);
            buffer += &"/-block_stmt".indent_print(depth + 1);
        }
        buffer += &self.last.indent_print(depth + 1);

        buffer += &"/block".indent_print(depth);

        buffer
    }
}

impl IndentPrint for zea::Statement {
    fn indent_print(&self, depth: usize) -> String {
        use zea::StatementKind;
        match &self.kind {
            StatementKind::Return(e) => "return".indent_print(depth) + &e.indent_print(depth + 1),
            StatementKind::Initialization(i) => i.indent_print(depth),
            StatementKind::BlockTail(e) => "tail".indent_print(depth) + &e.indent_print(depth + 1),
            StatementKind::IfThenElse(b) => b.indent_print(depth),
            StatementKind::Block(eb) => eb.indent_print(depth),
            StatementKind::FunctionCall(c) => c.indent_print(depth),
            _ => todo!("pretty print statement with kind {:?}", self.kind),
        }
    }
}
impl IndentPrint for zea::InitializationBlock {
    fn indent_print(&self, depth: usize) -> String {
        use zea::InitializationKind;
        match &self.kind {
            InitializationKind::Packed(p) => p.indent_print(depth),

            InitializationKind::Unpacked(p) => {
                let mut buffer = "init_unpacked_block".indent_print(depth);
                for init in p.iter() {
                    buffer += &init.indent_print(depth + 1);
                }
                buffer += &"/init_unpacked_block".indent_print(depth);
                buffer
            }
        }
    }
}

impl IndentPrint for zea::PackedInitialization {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = "init_packed".indent_print(depth);
        buffer += &".pattern".indent_print(depth + 1);
        buffer += &self.assignee.indent_print(depth + 2);
        buffer += &".type".indent_print(depth + 1);
        buffer += &self.typ.indent_print(depth + 2);
        buffer += &".value".indent_print(depth + 1);
        buffer += &self.value.indent_print(depth + 1);
        buffer += &"/init_packed".indent_print(depth);
        buffer
    }
}

impl IndentPrint for String {
    fn indent_print(&self, depth: usize) -> String {
        indent!(depth) + self + "\n"
    }
}

impl<'a> IndentPrint for &'a str {
    fn indent_print(&self, depth: usize) -> String {
        indent!(depth) + self + "\n"
    }
}

impl IndentPrint for zea::TypeSpecifier {
    fn indent_print(&self, depth: usize) -> String {
        format!("{:?}", self).indent_print(depth)
    }
}

impl IndentPrint for Option<zea::TypeSpecifier> {
    fn indent_print(&self, depth: usize) -> String {
        match self {
            None => String::from("to be inferred").indent_print(depth),
            Some(t) => t.indent_print(depth),
        }
    }
}

impl IndentPrint for zea::SimpleInitialization {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = "init".indent_print(depth);
        buffer += &".assignee".indent_print(depth + 1);
        buffer += &self.assignee.indent_print(depth + 2);
        buffer += &".type".indent_print(depth + 1);
        buffer += &self.typ.indent_print(depth + 2);
        buffer += &".value".indent_print(depth + 1);
        buffer += &self.value.indent_print(depth + 2);
        buffer += &"/init".indent_print(depth);
        buffer
    }
}

impl IndentPrint for zea::AssignmentPattern {
    fn indent_print(&self, depth: usize) -> String {
        // let mut buffer = "assign_pattern".indent_print(depth);
        match self {
            zea::AssignmentPattern::Identifier(i) => i.indent_print(depth),
            zea::AssignmentPattern::Tuple(tup) => {
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

impl IndentPrint for zea::IfThenElse {
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

        buffer += &"/branch".indent_print(depth);
        buffer
    }
}

impl IndentPrint for zea::Expression {
    fn indent_print(&self, depth: usize) -> String {
        use zea::ExpressionKind;
        let kind_str = self.kind.variant_as_str();
        match &self.kind {
            ExpressionKind::UnScopedIdent(i) => format!("{kind_str}({i})").indent_print(depth),
            ExpressionKind::IntegerLiteral(i) => format!("lit_int({i})").indent_print(depth),
            ExpressionKind::FloatLiteral(i) => format!("lit_float({i})").indent_print(depth),
            ExpressionKind::BinOpExpr(op, l, r) => {
                let mut buffer = format!("operator`{op:?}`").indent_print(depth);
                buffer += &l.indent_print(depth + 1);
                buffer += &r.indent_print(depth + 1);
                buffer += &format!("/operator`{op:?}`").indent_print(depth);
                buffer
            }
            ExpressionKind::UnOpExpr(op, arg) => {
                let mut buffer = format!("operator`{op:?}`").indent_print(depth);
                buffer += &arg.indent_print(depth + 1);
                buffer += &format!("/operator`{op:?}`").indent_print(depth);
                buffer
            }
            ExpressionKind::MemberAccess(e, m) => {
                let mut buffer = "expr_member".indent_print(depth);
                buffer += &e.indent_print(depth + 1);
                buffer += &m.indent_print(depth + 1);
                buffer += &format!("/expr_member").indent_print(depth);
                buffer
            }
            ExpressionKind::IfThenElse(b) => b.indent_print(depth),
            ExpressionKind::Block(b) => b.indent_print(depth),
            ExpressionKind::ExpandedBlock(eb) => eb.indent_print(depth),
            ExpressionKind::FunctionCall(c) => c.indent_print(depth),
            ExpressionKind::Unit => "unit_value".indent_print(depth),
            ExpressionKind::ScopedIdent(si) => si.indent_print(depth),
            _ => todo!("pretty print expression of kind {:?}", self.kind),
        }
    }
}
impl IndentPrint for zea::ScopedIdentifier {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = format!("{:?}", self.kind).indent_print(depth);
        buffer += &format!(".ident {}", self.ident).indent_print(depth + 1);
        buffer += &format!(".origin {}", self.origin).indent_print(depth + 1);

        buffer
    }
}

impl IndentPrint for zea::FunctionCall {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = "call".indent_print(depth);

        buffer += &".subject".indent_print(depth + 1);
        buffer += &self.subject.indent_print(depth + 2);

        if !self.args.is_empty() {
            for arg in self.args.iter() {
                buffer += &"-arg".indent_print(depth + 1);
                buffer += &arg.indent_print(depth + 2);
                buffer += &"/-arg".indent_print(depth + 1);
            }
        };
        buffer
    }
}
