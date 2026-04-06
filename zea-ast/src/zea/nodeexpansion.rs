// #![allow(unused)]

use crate::zea::{
    AssignmentPattern, ExpandedBlockExpr, Expression, ExpressionKind, Function, FunctionCall,
    HoistedFunctionSignature, IfThenElse, Initialisation, InitialisationKind, Module,
    PackedInitialisation, PartiallyUnpackedInitialisation, Statement, StatementBlock,
    StatementKind, StructDefinition, TypedIdentifier, UnpackedInitialisation,
};
use std::collections::{HashMap, HashSet};

/// This visitor will be called after each of the expansion-visitors
/// to ensure a correct AST before moving on to static analysis.
pub struct ASTValidator {
    ids: HashSet<usize>,
}
pub trait AcceptsASTValidator {
    /// Returns true if this node is considered valid
    fn ast_validate(&self, validator: &mut ASTValidator) -> bool;
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
    fn accept_block_expander(&mut self, block_expander: &mut NodeExpander) -> bool;
    /// Does this node have only block-expanded descendants?
    ///
    /// Returns false if any descendant is not yet expanded.
    fn has_blocks_expanded(&self, block_expander: &mut NodeExpander) -> bool;
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
    /// let mut expander = NodeExpander::new()
    /// while !ast.accept(&mut expander) {} // will always terminate
    /// ```
    fn accept(&mut self, tuple_namer: &mut NodeExpander) -> bool;
    fn is_expanded(&self, tuple_namer: &mut NodeExpander) -> bool;
}

impl AcceptsBlockExpander for Statement {
    fn accept_block_expander(&mut self, block_expander: &mut NodeExpander) -> bool {
        if self.has_blocks_expanded(block_expander) {
            return false;
        }
        match &mut self.kind {
            StatementKind::Block(b) => {
                self.kind = StatementKind::ExpandedBlock(block_expander.expand_expr_block(b));
                true
            }
            StatementKind::Initialisation(i) => i.accept_block_expander(block_expander),
            StatementKind::Reassignment(r) => r.value.accept_block_expander(block_expander),
            StatementKind::FunctionCall(call) => call.accept_block_expander(block_expander),
            StatementKind::Return(expr) => expr.accept_block_expander(block_expander),
            StatementKind::BlockTail(expr) => expr.accept_block_expander(block_expander),
            StatementKind::ExpandedBlock(b) => b.accept_block_expander(block_expander),
            StatementKind::CondBranch(b) => b.accept_block_expander(block_expander),
        };
        !self.has_blocks_expanded(block_expander)
    }
    fn has_blocks_expanded(&self, block_expander: &mut NodeExpander) -> bool {
        match &self.kind {
            StatementKind::Block(_) => false,

            StatementKind::Initialisation(i) => i.has_blocks_expanded(block_expander),
            StatementKind::Reassignment(r) => r.value.has_blocks_expanded(block_expander),
            StatementKind::FunctionCall(call) => call.has_blocks_expanded(block_expander),
            StatementKind::Return(expr) => expr.has_blocks_expanded(block_expander),
            StatementKind::BlockTail(expr) => expr.has_blocks_expanded(block_expander),
            StatementKind::ExpandedBlock(b) => b.has_blocks_expanded(block_expander),
            StatementKind::CondBranch(b) => b.has_blocks_expanded(block_expander),
        }
    }
}

impl AcceptsBlockExpander for IfThenElse {
    fn accept_block_expander(&mut self, block_expander: &mut NodeExpander) -> bool {
        self.condition.accept_block_expander(block_expander);
        self.true_case.accept_block_expander(block_expander);
        if let Some(e) = &mut self.false_case {
            e.accept_block_expander(block_expander);
        }
        !self.has_blocks_expanded(block_expander)
    }

    fn has_blocks_expanded(&self, block_expander: &mut NodeExpander) -> bool {
        self.condition.has_blocks_expanded(block_expander)
            && self.true_case.has_blocks_expanded(block_expander)
            && self
                .false_case
                .as_ref()
                .is_none_or(|e| e.has_blocks_expanded(block_expander))
    }
}

impl AcceptsBlockExpander for Initialisation {
    fn accept_block_expander(&mut self, block_expander: &mut NodeExpander) -> bool {
        if self.has_blocks_expanded(block_expander) {
            return false;
        }

        match &mut self.kind {
            InitialisationKind::Packed(p) => {
                p.value.accept_block_expander(block_expander);
            }
            InitialisationKind::PartiallyUnpacked(p) => {
                p.temporary.value.accept_block_expander(block_expander);
                for s in p.unpacked_assignments.iter_mut() {
                    s.accept_block_expander(block_expander);
                }
            }
        }

        !self.has_blocks_expanded(block_expander)
    }
    fn has_blocks_expanded(&self, block_expander: &mut NodeExpander) -> bool {
        match &self.kind {
            InitialisationKind::Packed(p) => p.value.has_blocks_expanded(block_expander),
            InitialisationKind::PartiallyUnpacked(p) => {
                p.temporary.value.has_blocks_expanded(block_expander)
                    && p.unpacked_assignments
                        .iter()
                        .all(|s| s.has_blocks_expanded(block_expander))
            }
        }
    }
}

impl AcceptsBlockExpander for Expression {
    fn accept_block_expander(&mut self, block_expander: &mut NodeExpander) -> bool {
        if self.has_blocks_expanded(block_expander) {
            return false;
        }

        match &mut self.kind {
            ExpressionKind::Block(block) => {
                self.kind = ExpressionKind::ExpandedBlock(Box::new(
                    block_expander.expand_expr_block(block),
                ));
                true
            }
            ExpressionKind::FuncCall(call) => call.accept_block_expander(block_expander),
            ExpressionKind::BinOpExpr(_, lhs, rhs) => {
                lhs.accept_block_expander(block_expander)
                    || rhs.accept_block_expander(block_expander)
            }
            ExpressionKind::UnOpExpr(_, arg) => arg.accept_block_expander(block_expander),
            ExpressionKind::ExpandedBlock(block) => block.accept_block_expander(block_expander),
            ExpressionKind::Unit => false,
            ExpressionKind::IntegerLiteral(_) => false,
            ExpressionKind::BoolLiteral(_) => false,
            ExpressionKind::FloatLiteral(_) => false,
            ExpressionKind::StringLiteral(_) => false,
            ExpressionKind::Ident(_) => false,
            ExpressionKind::MemberAccess(_, _) => false,
            ExpressionKind::CondBranch(b) => b.accept_block_expander(block_expander),
        };

        !self.has_blocks_expanded(block_expander)
    }
    fn has_blocks_expanded(&self, block_expander: &mut NodeExpander) -> bool {
        match &self.kind {
            ExpressionKind::Block(_block) => false,
            ExpressionKind::FuncCall(call) => call.has_blocks_expanded(block_expander),
            ExpressionKind::BinOpExpr(_, lhs, rhs) => {
                lhs.has_blocks_expanded(block_expander) && rhs.has_blocks_expanded(block_expander)
            }
            ExpressionKind::UnOpExpr(_, arg) => arg.has_blocks_expanded(block_expander),
            ExpressionKind::ExpandedBlock(block) => block.has_blocks_expanded(block_expander),
            ExpressionKind::CondBranch(b) => b.has_blocks_expanded(block_expander),
            _ => true,
        }
    }
}

impl AcceptsBlockExpander for FunctionCall {
    fn accept_block_expander(&mut self, block_expander: &mut NodeExpander) -> bool {
        if self.has_blocks_expanded(block_expander) {
            return false;
        }

        for arg in self.args.iter_mut() {
            arg.accept_block_expander(block_expander);
        }

        !self.has_blocks_expanded(block_expander)
    }
    fn has_blocks_expanded(&self, block_expander: &mut NodeExpander) -> bool {
        self.args
            .iter()
            .all(|e| e.has_blocks_expanded(block_expander))
    }
}

impl AcceptsBlockExpander for Function {
    fn accept_block_expander(&mut self, block_expander: &mut NodeExpander) -> bool {
        if self.has_blocks_expanded(block_expander) {
            return false;
        }
        self.body.accept_block_expander(block_expander);
        !self.has_blocks_expanded(block_expander)
    }

    fn has_blocks_expanded(&self, block_expander: &mut NodeExpander) -> bool {
        self.body.has_blocks_expanded(block_expander)
    }
}

impl AcceptsBlockExpander for StatementBlock {
    fn accept_block_expander(&mut self, block_expander: &mut NodeExpander) -> bool {
        if self.has_blocks_expanded(block_expander) {
            return false;
        }

        for stmt in self.statements.iter_mut() {
            stmt.accept_block_expander(block_expander);
        }
        !self.has_blocks_expanded(block_expander)
    }
    fn has_blocks_expanded(&self, block_expander: &mut NodeExpander) -> bool {
        self.statements
            .iter()
            .all(|s| s.has_blocks_expanded(block_expander))
    }
}

impl AcceptsBlockExpander for ExpandedBlockExpr {
    fn accept_block_expander(&mut self, block_expander: &mut NodeExpander) -> bool {
        if self.has_blocks_expanded(block_expander) {
            return false;
        }
        self.last.accept_block_expander(block_expander);
        for stmt in self.statements.iter_mut() {
            eprintln!("expanding stmt with id {}", stmt.id);
            stmt.accept_block_expander(block_expander);
        }
        !self.has_blocks_expanded(block_expander)
    }
    fn has_blocks_expanded(&self, block_expander: &mut NodeExpander) -> bool {
        self.last.has_blocks_expanded(block_expander)
            && self
                .statements
                .iter()
                .all(|s| s.has_blocks_expanded(block_expander))
    }
}

impl AcceptsBlockExpander for Module {
    fn accept_block_expander(&mut self, block_expander: &mut NodeExpander) -> bool {
        if self.has_blocks_expanded(block_expander) {
            return false;
        }

        for func in self.functions.iter_mut() {
            eprintln!("expanding function with name {}", func.name);
            func.accept_block_expander(block_expander);
        }
        !self.has_blocks_expanded(block_expander)
    }

    fn has_blocks_expanded(&self, block_expander: &mut NodeExpander) -> bool {
        self.functions
            .iter()
            .all(|f| f.has_blocks_expanded(block_expander))
    }
}

impl AcceptsAssignmentSimplifier for Function {
    fn accept_assignment_simplifier(&mut self, simplifier: &mut NodeExpander) -> bool {
        self.body.accept_assignment_simplifier(simplifier)
    }

    fn has_assignments_unpacked(&self, simplifier: &mut NodeExpander) -> bool {
        self.body.has_assignments_unpacked(simplifier)
    }
}

impl AcceptsAssignmentSimplifier for Module {
    fn accept_assignment_simplifier(&mut self, simplifier: &mut NodeExpander) -> bool {
        for f in self.functions.iter_mut() {
            f.accept_assignment_simplifier(simplifier);
        }
        // we do not simplify globs, as globs may only be simple assignments.
        !self.has_assignments_unpacked(simplifier)
    }
    fn has_assignments_unpacked(&self, simplifier: &mut NodeExpander) -> bool {
        self.functions
            .iter()
            .all(|f| f.has_assignments_unpacked(simplifier))
    }
}

impl Module {
    pub fn expand_blocks(mut self, block_expander: &mut NodeExpander) -> Module {
        while self.accept_block_expander(block_expander) {
            eprintln!("expanding blocks still...")
        }
        self
    }

    pub fn simplify_assignments(mut self, assignment_simplifier: &mut NodeExpander) -> Module {
        while self.accept_assignment_simplifier(assignment_simplifier) {
            eprintln!("simplifying assignments still...")
        }
        self
    }
}

#[derive(Default)]
pub struct NodeExpander {
    labeler: usize,
    hoisted_global_decls: HashSet<HoistedFunctionSignature>,
    /// All the types needed for a
    hoisted_global_types: HashSet<StructDefinition>,
    /// All the hoisted variable declarations within a function (blocks as expressions)
    hoisted_local_function_decls: HashMap<HoistedFunctionSignature, Vec<TypedIdentifier>>,
}

/// Tranform some node into a given variant, and label it.

impl NodeExpander {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn label(&mut self) -> usize {
        let label = self.labeler;
        self.labeler += 1;
        label
    }

    fn label_unpack(&mut self) -> (usize, String) {
        let label = self.label();
        (label, format!("__unpack{label}"))
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

    /// Given a tuple-value to unpack,
    /// generate a sequence of initializations that assign each member of the tuple-value:
    /// ```ignore
    /// tuple@(a,b) := label;
    ///
    /// becomes
    ///
    /// a := label._0;
    /// b := label._1;
    ///
    /// likewise:
    ///
    /// tup@((a,b),c)) = label;
    ///
    /// becomes:
    ///
    /// (a,b) := label._0;
    /// c := label._1;
    ///
    ///
    ///
    /// ```
    ///
    /// That is, for each member of the assignment-pattern, generate a new initialization,
    /// That gets assigned one field of the value
    fn simplify_tuple(
        &mut self,
        tuple: &Vec<AssignmentPattern>,
        value: Expression,
    ) -> Vec<Initialisation> {
        let mut assignees = vec![];
        for (field, assignee) in tuple.iter().enumerate() {
            let id = self.label();
            let kind = PackedInitialisation {
                typ: None,
                assignee: assignee.clone(),
                value: Expression::label_member_access(self, value.clone(), field),
            };
            let init = Initialisation {
                id,
                kind: InitialisationKind::Packed(kind),
            };
            assignees.push(init);
        }

        assignees
    }
}

pub trait AcceptsAssignmentSimplifier {
    /// Let the expander perform some transformation on `self`. Return false if no changes have been made.
    /// Repeatedly calling this method is guaranteed to eventually return false:
    ///
    /// ```ignore
    /// let ast = StatementBlock {
    ///     id: 0,
    ///     statements: vec![...]
    /// };
    /// let mut expander = NodeExpander::new()
    /// while !ast.accept(&mut expander) {} // will always terminate
    /// ```
    fn accept_assignment_simplifier(&mut self, simplifier: &mut NodeExpander) -> bool;
    fn has_assignments_unpacked(&self, simplifier: &mut NodeExpander) -> bool;
}

impl AcceptsAssignmentSimplifier for Initialisation {
    fn accept_assignment_simplifier(&mut self, simplifier: &mut NodeExpander) -> bool {
        match &mut self.kind {
            InitialisationKind::Packed(p) => match &p.assignee {
                AssignmentPattern::Identifier(_) => {}
                AssignmentPattern::Tuple(tup) => {
                    let (_id, label) = simplifier.label_unpack();
                    let temporary = UnpackedInitialisation {
                        typ: p.typ.clone(),
                        assignee: label.clone(),
                        value: p.value.clone(),
                    };
                    let label = Expression::ident(simplifier, label);
                    let unpacked_assignments = simplifier.simplify_tuple(&tup, label);
                    let partially_unpacked = PartiallyUnpackedInitialisation {
                        temporary,
                        unpacked_assignments,
                    };
                    self.kind = InitialisationKind::PartiallyUnpacked(partially_unpacked);
                }
            },

            InitialisationKind::PartiallyUnpacked(p) => {
                let inits = &mut p.unpacked_assignments;
                for init in inits.iter_mut() {
                    init.accept_assignment_simplifier(simplifier);
                }
            }
        };
        !self.has_assignments_unpacked(simplifier)
    }

    fn has_assignments_unpacked(&self, simplifier: &mut NodeExpander) -> bool {
        match &self.kind {
            InitialisationKind::PartiallyUnpacked(p) => p
                .unpacked_assignments
                .iter()
                .all(|init| init.has_assignments_unpacked(simplifier)),
            InitialisationKind::Packed(p) => matches!(p.assignee, AssignmentPattern::Identifier(_)),
        }
    }
}

impl AcceptsAssignmentSimplifier for StatementBlock {
    fn accept_assignment_simplifier(&mut self, simplifier: &mut NodeExpander) -> bool {
        for s in self.statements.iter_mut() {
            s.accept_assignment_simplifier(simplifier);
        }
        !self.has_assignments_unpacked(simplifier)
    }
    fn has_assignments_unpacked(&self, simplifier: &mut NodeExpander) -> bool {
        self.statements
            .iter()
            .all(|s| s.has_assignments_unpacked(simplifier))
    }
}

impl AcceptsAssignmentSimplifier for ExpandedBlockExpr {
    fn accept_assignment_simplifier(&mut self, simplifier: &mut NodeExpander) -> bool {
        for s in self.statements.iter_mut() {
            s.accept_assignment_simplifier(simplifier);
        }
        !self.has_assignments_unpacked(simplifier)
    }
    fn has_assignments_unpacked(&self, simplifier: &mut NodeExpander) -> bool {
        self.statements
            .iter()
            .all(|s| s.has_assignments_unpacked(simplifier))
    }
}

impl AcceptsAssignmentSimplifier for Statement {
    fn accept_assignment_simplifier(&mut self, simplifier: &mut NodeExpander) -> bool {
        match &mut self.kind {
            StatementKind::Initialisation(i) => i.accept_assignment_simplifier(simplifier),
            StatementKind::Block(b) => b.accept_assignment_simplifier(simplifier),
            StatementKind::ExpandedBlock(b) => b.accept_assignment_simplifier(simplifier),
            _ => false,
        }
    }

    fn has_assignments_unpacked(&self, simplifier: &mut NodeExpander) -> bool {
        match &self.kind {
            StatementKind::Initialisation(i) => i.has_assignments_unpacked(simplifier),
            StatementKind::Block(b) => b.has_assignments_unpacked(simplifier),
            StatementKind::ExpandedBlock(b) => b.has_assignments_unpacked(simplifier),
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::visualisation::IndentPrint;
use crate::zea::nodeexpansion::{
        AcceptsAssignmentSimplifier, AcceptsBlockExpander, NodeExpander,
    };
    use crate::zea::{
        AssignmentPattern, Expression, ExpressionKind, Function, Initialisation,
        InitialisationKind, Module, PackedInitialisation, Statement, StatementBlock, StatementKind,
        Type,
    };

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

        (init $p:expr ;= $val:expr) => {
            {
                use crate::zea::{AssignmentPattern,Initialisation,Statement,StatementKind};
            Statement {
                id: 0,
                kind: StatementKind::Initialisation(Initialisation {
                    id: 0,
                    kind: InitialisationKind::Packed(
                        PackedInitialisation {
                    assignee: $p,
                    typ: None,
                    value: $val,
                        }
                    )
                })
            }
        }};
    }

    // generated by claude code, prompt:
    // "can you make the pat macro a tt muncher
    // that accepts things like (a,(b,c)) and converts it to a nested assignment pattern"
    // + pat macro
    macro_rules! pat {
    // Single identifier — base case
    ($i:ident) => {
        AssignmentPattern::Identifier(String::from(stringify!($i)))
    };
    // Outer tuple — kick off the muncher with an empty accumulator
    (($($t:tt)*)) => {
        pat!(@munch [] $($t)*)
    };
    // Muncher: accumulator is complete, nothing left to consume
    (@munch [$($acc:expr),*]) => {
        AssignmentPattern::Tuple(vec![$($acc),*])
    };
    // Muncher: next item is a nested tuple, more items follow
    (@munch [$($acc:expr),*] ($($inner:tt)*), $($rest:tt)*) => {
        pat!(@munch [$($acc,)* pat!(($($inner)*))] $($rest)*)
    };
    // Muncher: next item is a nested tuple, nothing follows
    (@munch [$($acc:expr),*] ($($inner:tt)*)) => {
        pat!(@munch [$($acc,)* pat!(($($inner)*))])
    };
    // Muncher: next item is an identifier, more items follow
    (@munch [$($acc:expr),*] $i:ident, $($rest:tt)*) => {
        pat!(@munch [$($acc,)* pat!($i)] $($rest)*)
    };
    // Muncher: next item is an identifier, nothing follows
    (@munch [$($acc:expr),*] $i:ident) => {
        pat!(@munch [$($acc,)* pat!($i)])
    };
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
        let ast = zea_module! {
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
        assert!(ast.has_blocks_expanded(&mut block_expander));
        let mut ast = expr!(block block! {
            stmt!(init pat!(a) ;= expr!(litint 3));
            stmt!(tail expr!(ident a))
        });
        ast.accept_block_expander(&mut block_expander);
        let after = ast;
        let ExpressionKind::ExpandedBlock(expanded) = after.kind else {
            unreachable!()
        };
        assert_eq!(
            expanded.statements,
            vec![stmt!(init pat!(a) ;= expr!(litint 3))]
        );
        assert_eq!(expanded.last, expr!(ident a));
    }

    fn wrap_in_module(init: Initialisation) -> Module {
        Module {
            id: 0,
            imports: vec![],
            exports: vec![],
            globs: vec![],
            functions: vec![Function {
                id: 0,
                name: "test".to_string(),
                args: vec![],
                returns: Type::Basic("Unit".to_string()),
                body: StatementBlock {
                    id: 0,
                    statements: vec![Statement {
                        id: 0,
                        kind: StatementKind::Initialisation(init),
                    }],
                },
            }],
        }
    }

    #[test]
    fn test_simple_ident_init_is_already_done() {
        let mut simplifier = NodeExpander::new();

        let stmt = stmt!(init pat!(a) ;= expr!(litint 1));
        let StatementKind::Initialisation(ref init) = stmt.kind else {
            unreachable!()
        };

        assert!(
            init.has_assignments_unpacked(&mut simplifier),
            "Packed(Identifier) should already be considered unpacked"
        );
    }

    #[test]
    fn test_single_level_tuple_unpack() {
        let mut simplifier = NodeExpander::new();

        let mut stmt = stmt!(init pat!((a, b)) ;= expr!(ident some_tuple));
        let StatementKind::Initialisation(ref mut init) = stmt.kind else {
            unreachable!()
        };

        assert!(
            !init.has_assignments_unpacked(&mut simplifier),
            "Tuple init should not be considered done before simplification"
        );

        init.accept_assignment_simplifier(&mut simplifier);

        let InitialisationKind::PartiallyUnpacked(ref p) = init.kind else {
            panic!(
                "Expected PartiallyUnpacked after one pass, got {:?}",
                init.kind
            );
        };

        assert_eq!(p.temporary.assignee, "__unpack0");
        assert_eq!(p.unpacked_assignments.len(), 2);

        for sub in &p.unpacked_assignments {
            let InitialisationKind::Packed(ref packed) = sub.kind else {
                panic!("Expected Packed sub-assignment");
            };
            assert!(matches!(packed.assignee, AssignmentPattern::Identifier(_)));
            assert!(matches!(
                packed.value.kind,
                ExpressionKind::MemberAccess(_, _)
            ));
        }

        assert!(init.has_assignments_unpacked(&mut simplifier));
    }

    #[test]
    fn test_nested_tuple_requires_two_passes() {
        let mut simplifier = NodeExpander::new();

        // ((a, b), c) := nested_tuple
        let mut stmt = stmt!(init pat!(((a, b), c)) ;= expr!(ident nested_tuple));
        let StatementKind::Initialisation(ref mut init) = stmt.kind else {
            unreachable!()
        };

        let changed = init.accept_assignment_simplifier(&mut simplifier);
        eprintln!("\nafter p1:\n{}", init.indent_print(0));
        assert!(changed, "First pass should report a change");
        assert!(
            !init.has_assignments_unpacked(&mut simplifier),
            "Inner tuple should still need unpacking after first pass"
        );

        let changed = init.accept_assignment_simplifier(&mut simplifier);
        eprintln!("\nafter p2:\n{}", init.indent_print(0));
        assert!(changed, "Second pass should report a change");
        assert!(
            init.has_assignments_unpacked(&mut simplifier),
            "Should be fully done after second pass"
        );
    }

    #[test]
    fn test_module_simplify_assignments_end_to_end() {
        let mut simplifier = NodeExpander::new();

        let stmt = stmt!(init pat!((a, b, c)) ;= expr!(ident v));
        let StatementKind::Initialisation(init) = stmt.kind else {
            unreachable!()
        };

        let module = wrap_in_module(init).simplify_assignments(&mut simplifier);

        assert!(module.has_assignments_unpacked(&mut simplifier));

        for func in &module.functions {
            for stmt in &func.body.statements {
                let StatementKind::Initialisation(ref i) = stmt.kind else {
                    continue;
                };
                let InitialisationKind::PartiallyUnpacked(ref p) = i.kind else {
                    panic!("Expected PartiallyUnpacked at top level");
                };
                for sub in &p.unpacked_assignments {
                    assert!(
                        sub.has_assignments_unpacked(&mut simplifier),
                        "All sub-assignments should be fully simplified"
                    );
                }
            }
        }
    }
}
