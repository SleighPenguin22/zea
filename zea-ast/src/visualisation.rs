use crate::zea;
pub trait IndentPrint {
    fn indent_print(&self, depth: usize) -> String;
    fn depth_str(depth: usize) -> String {
        " ".repeat(depth * 2)
    }
}

macro_rules! indent {
    ($d:expr) => {{
        let d: usize = $d;
        " ".repeat(d * 2)
    }};
}

impl IndentPrint for zea::Module {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = "Module(\n".indent_print(depth);
        let imports = if self.imports.is_empty() {
            "#IMPORTS NOTHING\n".indent_print(depth + 1)
        } else {
            let mut imp_buffer = "#IMPORTS".indent_print(depth + 1);
            for e in self.imports.iter() {
                imp_buffer += &e.indent_print(depth + 2);
            }
            imp_buffer
        };
        buffer += &imports;

        let exports = if self.exports.is_empty() {
            "#EXPORTS NOTHING\n".indent_print(depth + 1)
        } else {
            let mut exp_buffer = "#EXPORTS".indent_print(depth + 1);
            for e in self.exports.iter() {
                exp_buffer += &e.indent_print(depth + 1);
            }
            exp_buffer
        };
        buffer += &exports;

        buffer += &"#GLOBS".indent_print(depth + 1);
        for glob in self.globs.iter() {
            buffer += &glob.indent_print(depth + 2);
        }
        buffer += &"/#GLOBS".indent_print(depth + 1);

        buffer += &"#FUNCS".indent_print(depth + 1);
        for func in self.functions.iter() {
            buffer += &func.indent_print(depth + 2);
        }
        buffer += &"/#FUNCS\n".indent_print(depth + 1);

        buffer += &"/MODULE".indent_print(depth);
        buffer
    }
}

impl IndentPrint for zea::Function {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = format!("FUNC `{}`", self.name).indent_print(depth);
        buffer += &"#RETURNS".indent_print(depth + 1);
        buffer += &self.returns.indent_print(depth + 2);
        buffer += &"#ARGS".indent_print(depth + 1);
        for arg in self.args.iter() {
            buffer += &arg.indent_print(depth + 2);
        }
        buffer += &"#BODY".indent_print(depth + 1);
        buffer += &self.body.indent_print(depth + 1);
        buffer += &format!("/#FUNC {}", self.name).indent_print(depth);
        buffer
    }
}

impl IndentPrint for zea::TypedIdentifier {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = self.name.indent_print(depth);
        buffer += &"#TYPE".indent_print(depth);
        buffer += &self.typ.indent_print(depth + 1);
        buffer
    }
}

impl IndentPrint for zea::StatementBlock {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = format!("BLOCK {}", self.id).indent_print(depth);

        for s in self.statements.iter() {
            buffer += &format!("BLOCKSTMT {}", s.id).indent_print(depth + 1);
            buffer += &s.indent_print(depth + 1);
            buffer += &format!("/BLOCKSTMT {}", s.id).indent_print(depth + 1);
        }
        buffer += &format!("/BLOCK {}", self.id).indent_print(depth);

        buffer
    }
}

impl IndentPrint for zea::ExpandedBlockExpr {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = format!("BLOCK {}", self.id).indent_print(depth);

        for s in self.statements.iter() {
            buffer += &format!("BLOCKSTMT {}", s.id).indent_print(depth + 1);
            buffer += &s.indent_print(depth + 2);
            buffer += &format!("/BLOCKSTMT {}", s.id).indent_print(depth + 1);
        }
        buffer += &self.last.indent_print(depth + 1);

        buffer += &format!("/BLOCK {}", self.id).indent_print(depth);

        buffer
    }
}

impl IndentPrint for zea::Statement {
    fn indent_print(&self, depth: usize) -> String {
        use zea::StatementKind;
        match &self.kind {
            StatementKind::Return(e) => {
                format!("{}RETURN\n{}", indent!(depth), e.indent_print(depth + 1))
            }
            StatementKind::Initialisation(i) => i.indent_print(depth),
            StatementKind::BlockTail(e) => {
                format!("{}TAIL\n{}", indent!(depth), e.indent_print(depth + 1))
            }
            StatementKind::CondBranch(b) => b.indent_print(depth),
            StatementKind::ExpandedBlock(eb) => eb.indent_print(depth),
            StatementKind::FunctionCall(c) => c.indent_print(depth),
            _ => todo!("pretty print statement with kind {:?}", self.kind),
        }
    }
}
impl IndentPrint for zea::Initialisation {
    fn indent_print(&self, depth: usize) -> String {
        use zea::InitialisationKind;
        match &self.kind {
            InitialisationKind::Packed(p) => p.indent_print(depth),
            InitialisationKind::PartiallyUnpacked(p) => p.indent_print(depth),
        }
    }
}

impl IndentPrint for zea::PackedInitialisation {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = "P_INIT".indent_print(depth);
        buffer += &"#PATTERN".indent_print(depth + 1);
        buffer += &self.assignee.indent_print(depth + 2);
        buffer += &"#TYPE".indent_print(depth + 1);
        buffer += &self.typ.indent_print(depth + 2);
        buffer += &"#VALUE".indent_print(depth + 1);
        buffer += &self.value.indent_print(depth + 1);
        buffer += &"/P_INIT".indent_print(depth);
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

impl IndentPrint for zea::Type {
    fn indent_print(&self, depth: usize) -> String {
        format!("{:?}", self).indent_print(depth)
    }
}

impl IndentPrint for Option<zea::Type> {
    fn indent_print(&self, depth: usize) -> String {
        match self {
            None => String::from("TO BE INFERRED").indent_print(depth),
            Some(t) => t.indent_print(depth),
        }
    }
}

impl IndentPrint for zea::UnpackedInitialisation {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = "UNP_INIT".indent_print(depth);
        buffer += &"#PATTERN".indent_print(depth + 1);
        buffer += &self.assignee.indent_print(depth + 2);
        buffer += &"#TYPE".indent_print(depth + 1);
        buffer += &self.typ.indent_print(depth + 2);
        buffer += &"#VALUE".indent_print(depth + 1);
        buffer += &self.value.indent_print(depth + 1);
        buffer += &"/UNP_INIT".indent_print(depth);
        buffer
    }
}

impl IndentPrint for zea::PartiallyUnpackedInitialisation {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = self.temporary.indent_print(depth);
        for u in self.unpacked_assignments.iter() {
            buffer += &u.indent_print(depth + 1);
        }
        buffer
    }
}

impl IndentPrint for zea::AssignmentPattern {
    fn indent_print(&self, depth: usize) -> String {
        match self {
            zea::AssignmentPattern::Identifier(i) => indent!(depth) + i,
            zea::AssignmentPattern::Tuple(tup) => {
                indent!(depth)
                    + &tup
                        .iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
            }
        }
    }
}

impl IndentPrint for zea::IfThenElse {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = "IFELSE".indent_print(depth);

        buffer += &"#COND".indent_print(depth + 1);
        buffer += &self.condition.indent_print(depth + 2);

        buffer += &"#IFTRUE".indent_print(depth + 1);
        buffer += &self.true_case.indent_print(depth + 2);

        if let Some(e) = self.false_case.as_ref() {
            buffer += &"#FALSECASE".indent_print(depth + 1);
            buffer += &e.indent_print(depth + 2);
        };
        buffer
    }
}

impl IndentPrint for zea::Expression {
    fn indent_print(&self, depth: usize) -> String {
        use zea::ExpressionKind;
        let kind_str = self.kind.variant_as_str();
        match &self.kind {
            ExpressionKind::Ident(i) => format!("{}{kind_str}({i})\n", indent!(depth)),
            ExpressionKind::IntegerLiteral(i) => format!("{}Int({i})\n", indent!(depth)),
            ExpressionKind::FloatLiteral(i) => format!("{}Float({i})\n", indent!(depth)),
            ExpressionKind::BinOpExpr(op, l, r) => {
                format!(
                    "{0}{op:?}\n{1}{2}{0}/`{op:?}`\n",
                    indent!(depth),
                    l.indent_print(depth + 1),
                    r.indent_print(depth + 1),
                )
            }
            ExpressionKind::UnOpExpr(op, arg) => {
                format!(
                    "{0}{op:?}\n{1}{0}/`{op:?}`\n",
                    indent!(depth),
                    arg.indent_print(depth + 1),
                )
            }
            ExpressionKind::MemberAccess(e, m) => {
                format!(
                    "{0}MEMBER\n{1}{2}",
                    indent!(depth),
                    e.indent_print(depth + 1),
                    m.indent_print(depth + 1),
                )
            }
            ExpressionKind::CondBranch(b) => b.indent_print(depth),
            ExpressionKind::Block(b) => b.indent_print(depth),
            ExpressionKind::ExpandedBlock(eb) => eb.indent_print(depth),
            ExpressionKind::FuncCall(c) => c.indent_print(depth),
            ExpressionKind::Unit => "UNITVALUE".indent_print(depth),
            _ => todo!("pretty print expression of kind {:?}", self.kind),
        }
    }
}
impl IndentPrint for zea::FunctionCall {
    fn indent_print(&self, depth: usize) -> String {
        let mut buffer = "CALL".indent_print(depth);

        buffer += &"#FUNC".indent_print(depth + 1);
        buffer += &self.name.indent_print(depth + 2);

        if !self.args.is_empty() {
            for arg in self.args.iter() {
                buffer += &"#ARG".indent_print(depth + 1);
                buffer += &arg.indent_print(depth + 2);
            }
        };
        buffer
    }
}
