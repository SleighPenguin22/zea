// #![allow(unused)]

use crate::zea::{ExpandedBlockExpr, ExpandedInitialisation};
use crate::zea::{Expression, ExpressionKind};
use crate::zea::{Function, FunctionCall, Module};
use crate::zea::{Initialisation, StatementBlock};
use crate::zea::{Statement, StatementKind};
use std::collections::HashSet;

/// This visitor will be called after each of the expansion-visitors
/// to ensure a correct AST before moving on to static analysis.
pub struct ASTValidator {
    ids: HashSet<usize>,
}
pub trait AcceptsASTValidator {
    /// Returns true if this node is considered valid
    fn accept(&self, validator: &mut ASTValidator) -> bool;
}

pub trait AcceptsBlockExpander {
    /// Let the expander perform some transformation on `self`. Return false if no changes have been made.
    /// Repeatedly calling this method is guaranteed to eventually return false:
    ///
    /// ```ignore
    /// let ast = StatementBlock {
    ///     id: 0,
    ///     statements: vec![...]
    /// };
    /// let mut expander = NodeExpander
    /// while !ast.accept(&mut expander) {} // will always terminate
    /// ```
    fn accept(&mut self, block_expander: &mut NodeExpander) -> bool;
    fn is_expanded(&self, block_expander: &mut NodeExpander) -> bool;
}

pub trait AcceptsTupleNamer {
    /// Let the expander perform some transformation on `self`. Return false if no changes have been made.
    /// Repeatedly calling this method is guaranteed to eventually return false:
    ///
    /// ```ignore
    /// let ast = StatementBlock {
    ///     id: 0,
    ///     statements: vec![...]
    /// };
    /// let mut expander = NodeExpander
    /// while !ast.accept(&mut expander) {} // will always terminate
    /// ```
    fn accept(&mut self, tuple_namer: &mut NodeExpander) -> bool;
    fn is_expanded(&self, tuple_namer: &mut NodeExpander) -> bool;
}

impl AcceptsBlockExpander for Statement {
    fn accept(&mut self, block_expander: &mut NodeExpander) -> bool {
        if self.is_expanded(block_expander) {
            return false;
        }
        match &mut self.kind {
            StatementKind::Block(b) => {
                self.kind = StatementKind::ExpandedBlock(block_expander.expand_expr_block(b));
                true
            }
            StatementKind::Initialisation(i) => i.value.accept(block_expander),
            StatementKind::Reassignment(r) => r.value.accept(block_expander),
            StatementKind::FunctionCall(call) => call.accept(block_expander),
            StatementKind::Return(expr) => expr.accept(block_expander),
            StatementKind::BlockTail(expr) => expr.accept(block_expander),
            StatementKind::ExpandedInitialisation(init) => init.accept(block_expander),
            StatementKind::SimpleInitialisation(sinit) => sinit.value.accept(block_expander),
            StatementKind::ExpandedBlock(b) => b.accept(block_expander),
        };
        !self.is_expanded(block_expander)
    }
    fn is_expanded(&self, block_expander: &mut NodeExpander) -> bool {
        match &self.kind {
            StatementKind::Block(_) => false,

            StatementKind::Initialisation(i) => i.value.is_expanded(block_expander),
            StatementKind::Reassignment(r) => r.value.is_expanded(block_expander),
            StatementKind::FunctionCall(call) => call.is_expanded(block_expander),
            StatementKind::Return(expr) => expr.is_expanded(block_expander),
            StatementKind::BlockTail(expr) => expr.is_expanded(block_expander),
            StatementKind::ExpandedInitialisation(init) => init.is_expanded(block_expander),
            StatementKind::SimpleInitialisation(sinit) => sinit.value.is_expanded(block_expander),
            StatementKind::ExpandedBlock(b) => b.is_expanded(block_expander),
        }
    }
}
impl AcceptsBlockExpander for Initialisation {
    fn accept(&mut self, block_expander: &mut NodeExpander) -> bool {
        if self.is_expanded(block_expander) {
            return false;
        }

        self.value.accept(block_expander);
        !self.is_expanded(block_expander)
    }
    fn is_expanded(&self, block_expander: &mut NodeExpander) -> bool {
        self.value.is_expanded(block_expander)
    }
}

impl AcceptsBlockExpander for Expression {
    fn accept(&mut self, block_expander: &mut NodeExpander) -> bool {
        if self.is_expanded(block_expander) {
            return false;
        }

        match &mut self.kind {
            ExpressionKind::Block(block) => {
                self.kind = ExpressionKind::ExpandedBlock(Box::new(
                    block_expander.expand_expr_block(block),
                ));
                true
            }
            ExpressionKind::FuncCall(call) => call.accept(block_expander),
            ExpressionKind::BinOpExpr(_, lhs, rhs) => {
                lhs.accept(block_expander) || rhs.accept(block_expander)
            }
            ExpressionKind::UnOpExpr(_, arg) => arg.accept(block_expander),
            ExpressionKind::ExpandedBlock(block) => block.accept(block_expander),
            ExpressionKind::Unit => false,
            ExpressionKind::IntegerLiteral(_) => false,
            ExpressionKind::BoolLiteral(_) => false,
            ExpressionKind::FloatLiteral(_) => false,
            ExpressionKind::StringLiteral(_) => false,
            ExpressionKind::Ident(_) => false,
            ExpressionKind::MemberAccess(_, _) => false,
        };

        !self.is_expanded(block_expander)
    }
    fn is_expanded(&self, block_expander: &mut NodeExpander) -> bool {
        match &self.kind {
            ExpressionKind::Block(_block) => false,
            ExpressionKind::FuncCall(call) => call.is_expanded(block_expander),
            ExpressionKind::BinOpExpr(_, lhs, rhs) => {
                lhs.is_expanded(block_expander) && rhs.is_expanded(block_expander)
            }
            ExpressionKind::UnOpExpr(_, arg) => arg.is_expanded(block_expander),
            ExpressionKind::ExpandedBlock(block) => block.is_expanded(block_expander),
            _ => true,
        }
    }
}

impl AcceptsBlockExpander for ExpandedInitialisation {
    fn accept(&mut self, block_expander: &mut NodeExpander) -> bool {
        if self.is_expanded(block_expander) {
            return false;
        }

        self.temporary.value.accept(block_expander);
        for assign in &mut self.unpacked_assignments {
            assign.accept(block_expander);
        }
        !self.is_expanded(block_expander)
    }
    fn is_expanded(&self, block_expander: &mut NodeExpander) -> bool {
        self.unpacked_assignments
            .iter()
            .all(|a| a.is_expanded(block_expander))
    }
}

impl AcceptsBlockExpander for FunctionCall {
    fn accept(&mut self, block_expander: &mut NodeExpander) -> bool {
        if self.is_expanded(block_expander) {
            return false;
        }

        for arg in self.args.iter_mut() {
            arg.accept(block_expander);
        }

        !self.is_expanded(block_expander)
    }
    fn is_expanded(&self, block_expander: &mut NodeExpander) -> bool {
        self.args.iter().all(|e| e.is_expanded(block_expander))
    }
}

impl AcceptsBlockExpander for Function {
    fn accept(&mut self, block_expander: &mut NodeExpander) -> bool {
        if self.is_expanded(block_expander) {
            return false;
        }
        self.body.accept(block_expander);
        !self.is_expanded(block_expander)
    }

    fn is_expanded(&self, block_expander: &mut NodeExpander) -> bool {
        self.body.is_expanded(block_expander)
    }
}

impl AcceptsBlockExpander for StatementBlock {
    fn accept(&mut self, block_expander: &mut NodeExpander) -> bool {
        if self.is_expanded(block_expander) {
            return false;
        }

        for stmt in self.statements.iter_mut() {
            stmt.accept(block_expander);
        }
        !self.is_expanded(block_expander)
    }
    fn is_expanded(&self, block_expander: &mut NodeExpander) -> bool {
        self.statements
            .iter()
            .all(|s| s.is_expanded(block_expander))
    }
}

impl AcceptsBlockExpander for ExpandedBlockExpr {
    fn accept(&mut self, block_expander: &mut NodeExpander) -> bool {
        if self.is_expanded(block_expander) {
            return false;
        }
        self.last.accept(block_expander);
        for stmt in self.statements.iter_mut() {
            eprintln!("expanding stmt with id {}", stmt.id);
            stmt.accept(block_expander);
        }
        !self.is_expanded(block_expander)
    }
    fn is_expanded(&self, block_expander: &mut NodeExpander) -> bool {
        self.last.is_expanded(block_expander)
            && self
                .statements
                .iter()
                .all(|s| s.is_expanded(block_expander))
    }
}

impl AcceptsBlockExpander for Module {
    fn accept(&mut self, block_expander: &mut NodeExpander) -> bool {
        if self.is_expanded(block_expander) {
            return false;
        }

        for func in self.functions.iter_mut() {
            eprintln!("expanding function with name {}", func.name);
            func.accept(block_expander);
        }
        !self.is_expanded(block_expander)
    }

    fn is_expanded(&self, block_expander: &mut NodeExpander) -> bool {
        self.functions.iter().all(|f| f.is_expanded(block_expander))
    }
}

impl Module {
    pub fn expand_blocks(mut self, block_expander: &mut NodeExpander) -> Module {
        while self.accept(block_expander) {
            eprintln!("expanding still...")
        }
        self
    }
}
#[derive(Default)]
pub struct NodeExpander {
    labeler: usize,
    // expanded_nodes: HashMap<usize, bool>,
}

/// Tranform some node into a given variant, and label it.

impl NodeExpander {
    pub fn new() -> Self {
        Self::default()
    }

    fn label(&mut self) -> usize {
        let label = self.labeler;
        self.labeler += 1;
        label
    }

    /// Expand some expression block
    ///
    /// Inserts a unit-tail if the block does not end with a tail expression.
    pub fn expand_expr_block(&mut self, block: &StatementBlock) -> ExpandedBlockExpr {
        let (statements, last) = match block.statements.last() {
            Some(Statement {
                kind: StatementKind::BlockTail(_),
                ..
            }) => {
                let (last, init) = block.statements.split_last().unwrap();
                let init = init.to_vec();
                let StatementKind::BlockTail(last) = last.clone().kind else {
                    unreachable!()
                };
                (init, last)
            }
            _ => (block.statements.clone(), Expression::unit(self.label())),
        };

        ExpandedBlockExpr {
            id: self.label(),
            statements,
            last,
        }
    }

    // pub fn expand_func_body(&mut self, func: &mut Function) -> ExpandedBlockExpr {
    //     let body = &mut func.body;
    //     let (statements, last) = match body.statements.last() {
    //         Some(Statement {
    //             kind: StatementKind::Return(_),
    //             ..
    //         }) => {
    //             let tail = body.statements.last().cloned().unwrap();
    //             let StatementKind::Return(e) = tail.kind else {
    //                 unreachable!()
    //             };
    //             (body.statements.clone(), e)
    //         }
    //         _ => (body.statements.clone(), Expression::unit(self.label())),
    //     };
    //
    //     ExpandedBlockExpr {
    //         id: self.label(),
    //         statements,
    //         last,
    //     }
    // }

    // pub fn expand_statement(&mut self, statement: Statement) -> Statement {
    //     let kind = match statement.kind {
    //         StatementKind::Block(b) => StatementKind::ExpandedBlock(self.expand_expr_block(&b)),
    //         StatementKind::Initialisation(assignment) => {
    //             StatementKind::ExpandedInitialisation(self.expand_assignment(assignment))
    //         }
    //         _ => return statement,
    //     };
    //     Statement {
    //         id: self.label(),
    //         kind,
    //     }
    // }
    //
    // pub fn expand_expression(&mut self, expression: Expression) -> Expression {
    //     let kind = match expression.kind {
    //         ExpressionKind::Block(block) => {
    //             ExpressionKind::ExpandedBlock(Box::new(self.expand_expr_block(&block)))
    //         }
    //         ref _other => return expression,
    //     };
    //     Expression {
    //         id: self.label(),
    //         kind,
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use crate::zea::nodeexpansion::{AcceptsBlockExpander, NodeExpander};
    use crate::zea::{Expression, ExpressionKind, Function, Module, StatementBlock, Type};

    macro_rules! block {
        {} => {
           {
               StatementBlock {
                    id: 0,
                    statements: vec![]
               }
           }
        };
        {$($e:expr);+ $(;)?} => {
           {
               StatementBlock {
                    id: 0,
                    statements: vec![$($e),+]
               }
           }
        };
    }

    macro_rules! stmt {
        (ret $e:expr) => {
            {use crate::zea::{Statement,StatementKind};
            Statement {
                id: 0,
                kind: StatementKind::Return($e)
            }
        }};
        (block $e:expr) => {
            {
                use crate::zea::{Statement, StatementKind};
            Statement {
                id: 0,
                kind: StatementKind::Block($e)
            }
        }};
        (tail $e:expr) => {
            {use crate::zea::{Statement,StatementKind};
            Statement {
                id: 0,
                kind: StatementKind::BlockTail($e)
            }
        }};
        (call $name:ident ($($e:expr),*)) => {
            {
                use crate::zea::{Statement, StatementKind, FunctionCall}
            Statement {
                id: 0,
                kind: StatementKind::FunctionCall(FunctionCall {
                    id: 0,
                    name: $name,
                    args: vec![$($e),*]
                })
            }
        }};

        (init $name:ident := $val:expr) => {
            {
                use crate::zea::{AssignmentPattern,Initialisation,Statement,StatementKind};
            Statement {
                id: 0,
                kind: StatementKind::Initialisation(Initialisation {
                    id: 0,
                    assignee: AssignmentPattern::Identifier(String::from(stringify!($name))),
                    typ: None,
                    value: $val,
                })
            }
        }};
    }

    macro_rules! expr {
        (ident $($l:tt)+) => {{
            use crate::zea::{Expression, ExpressionKind};
            Expression {
                id: 0,
                kind: ExpressionKind::Ident(String::from(stringify!($($l)+))),
            }
        }};
        (litint $l:literal) => {{
            use crate::zea::{Expression, ExpressionKind};
            Expression {
                id: 0,
                kind: ExpressionKind::IntegerLiteral($l),
            }
        }};
        (litfloat $l:literal) => {{
            use crate::zea::{Expression, ExpressionKind};
            Expression {
                id: 0,
                kind: ExpressionKind::FloatLiteral($l),
            }
        }};
        (litbool $l:literal) => {{
            use crate::zea::{Expression, ExpressionKind};
            Expression {
                id: 0,
                kind: ExpressionKind::BoolLiteral($l),
            }
        }};
        (litstr $l:literal) => {
            Expression {
                id: 0,
                kind: ExpressionKind::StringLiteral(stringify!($l)),
            }
        };
        (unit) => {{
            use crate::zea::{Expression, ExpressionKind};
            Expression {
                id: 0,
                kind: ExpressionKind::Unit,
            }
        }};
        (block $block:expr) => {{
            use crate::zea::{Expression,ExpressionKind,StatementBlock};
            Expression {
                id: 0,
                kind: ExpressionKind::Block($block)
            }
            }
        }
    }

    macro_rules! zea_module {
        (imports {$($imp:ident),* $(,)?}
         exports {$($exp:ident),* $(,)?}
         globs   {$($glob:expr);* $(;)?}
         funcs   {$($func:expr);* $(;)?}
        ) => {
            {
                use crate::zea::Module;
                Module {
                    id: 0,
                    imports: vec![$(String::from(stringify!($imp))),*],
                    exports: vec![$(String::from(stringify!($exp))),*],
                    globs: vec![$($glob),*],
                    functions: vec![$($func),*],
                }
            }
        };
    }
    macro_rules! func {
        {$name:ident ( $($arg:ident: $typ:expr),* ) -> $ret:expr; { $body:expr }} => {
            {
                use crate::zea::{Function, TypedIdentifier};
                let args = vec![$(
                TypedIdentifier(String::from(stringify!($arg)), $typ)
                ),*];
                Function {
                    id: 0,
                    name: String::from(stringify!($name)),
                    args,
                    returns: $ret,
                    body: $body,
                }
            }
        };
    }
    macro_rules! ztyp {
        ($t:ident) => {
            {
            use crate::zea::Type;
                Type::Basic(String::from(stringify!($t)))
            }
        };
        (*$($t:tt)+) => {
            {
            use crate::zea::Type;
                Type::Pointer(Box::new(ztyp!($($t)+)))
            }
        };
        ([ ]$($t:tt)+) => {
            {
            use crate::zea::Type;
                Type::ArrayOf(Box::new(ztyp!($($t)+)))
            }
        };
    }
    #[test]
    fn test_expand_block() {
        let mut block_expander = NodeExpander::new();
        let mut ast = zea_module! {
            imports {}
            exports {}
            globs {}
            funcs {
                func!(main() -> ztyp!(Int); {block!{
                    stmt!(tail expr!(litint 3))
                }})
            }
        };

        let ast = ast.expand_blocks(&mut block_expander);
        // eprintln!("{:?}", ast.functions[0]);
        assert!(ast.is_expanded(&mut block_expander));
        let mut ast = expr!(block block! {
            stmt!(init a := expr!(litint 3));
            stmt!(tail expr!(ident a))
        });
        let before = ast.clone();
        ast.accept(&mut block_expander);
        let after = ast;
        let ExpressionKind::ExpandedBlock(expanded) = after.kind else {
            unreachable!()
        };
        assert_eq!(expanded.statements, vec![stmt!(init a := expr!(litint 3))]);
        assert_eq!(expanded.last, expr!(ident a));
    }
}
