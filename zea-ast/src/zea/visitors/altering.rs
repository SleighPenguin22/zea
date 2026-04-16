use crate::zea::{
    AssignmentPattern, ExpandedBlockExpr, Expression, ExpressionKind, Function, FunctionCall,
    IfThenElse, Initialisation, InitialisationKind, Module, PackedInitialisation,
    PartiallyUnpackedInitialisation, Reassignment, Statement, StatementBlock, StatementKind,
    UnpackedInitialisation,
};

pub struct BareNodeLabeler {
    label: usize,
}

pub trait NodeLabeler {
    /// Start `Self`'s id-generator with the last id that `other_generator` used,
    /// such that [`Self::next_label`] calls will never produce an ID
    /// equal to any of `other_generator`'s ID's
    fn transplant_generator(other_generator: impl NodeLabeler) -> Self;
    /// All implementors must ensure that any ID generated is not equal to 0,
    /// as this is a sentinel ID used to signify the need for a fresh ID
    fn next_label(&mut self) -> usize;
    /// Generate the next label, along with a valid unique identifier
    fn next_label_with_string(&mut self) -> (usize, String) {
        let next = self.next_label();
        (next, format!("__label{}", next))
    }
    /// assign a fresh ID only if the current ID is equal to 0.
    fn validify_sentinel_label(&mut self, current_id: &mut usize) {
        if *current_id == 0 {
            *current_id = self.next_label();
        }
    }
}
impl BareNodeLabeler {
    pub fn new() -> Self {
        Self { label: 1 }
    }
    pub fn next_label(&mut self) -> usize {
        let label = self.label;
        self.label += 1;
        label
    }
}
impl NodeLabeler for BareNodeLabeler {
    fn transplant_generator(mut other_generator: impl NodeLabeler) -> Self {
        Self {
            label: other_generator.next_label(),
        }
    }
    fn next_label(&mut self) -> usize {
        BareNodeLabeler::next_label(self)
    }
}

pub trait Relabel {
    fn give_unique_ids(&mut self, labeler: &mut impl NodeLabeler);
}

impl Relabel for Module {
    fn give_unique_ids(&mut self, labeler: &mut impl NodeLabeler) {
        labeler.validify_sentinel_label(&mut self.id);
        for glob_var in self.global_vars.iter_mut() {
            glob_var.give_unique_ids(labeler);
        }

        for func in self.functions.iter_mut() {
            func.give_unique_ids(labeler);
        }
    }
}

impl Relabel for Initialisation {
    fn give_unique_ids(&mut self, labeler: &mut impl NodeLabeler) {
        labeler.validify_sentinel_label(&mut self.id);
        match &mut self.kind {
            InitialisationKind::PartiallyUnpacked(p) => p.give_unique_ids(labeler),
            InitialisationKind::Packed(p) => p.give_unique_ids(labeler),
        }
    }
}

impl Relabel for PartiallyUnpackedInitialisation {
    fn give_unique_ids(&mut self, labeler: &mut impl NodeLabeler) {
        self.temporary.give_unique_ids(labeler);
        for unpack in self.unpacked_assignments.iter_mut() {
            unpack.give_unique_ids(labeler);
        }
    }
}

impl Relabel for PackedInitialisation {
    fn give_unique_ids(&mut self, labeler: &mut impl NodeLabeler) {
        self.value.give_unique_ids(labeler);
    }
}
impl Relabel for UnpackedInitialisation {
    fn give_unique_ids(&mut self, labeler: &mut impl NodeLabeler) {
        self.value.give_unique_ids(labeler);
    }
}
impl Relabel for Function {
    fn give_unique_ids(&mut self, labeler: &mut impl NodeLabeler) {
        labeler.validify_sentinel_label(&mut self.id);
        self.body.give_unique_ids(labeler);
    }
}
impl Relabel for StatementBlock {
    fn give_unique_ids(&mut self, labeler: &mut impl NodeLabeler) {
        labeler.validify_sentinel_label(&mut self.id);
        for stmt in self.statements.iter_mut() {
            stmt.give_unique_ids(labeler);
        }
    }
}

impl Relabel for Statement {
    fn give_unique_ids(&mut self, labeler: &mut impl NodeLabeler) {
        labeler.validify_sentinel_label(&mut self.id);
        match &mut self.kind {
            StatementKind::Initialisation(i) => i.give_unique_ids(labeler),
            StatementKind::Reassignment(r) => r.give_unique_ids(labeler),
            StatementKind::FunctionCall(call) => call.give_unique_ids(labeler),
            StatementKind::Return(e) => e.give_unique_ids(labeler),
            StatementKind::BlockTail(e) => e.give_unique_ids(labeler),
            StatementKind::Block(b) => b.give_unique_ids(labeler),
            StatementKind::ExpandedBlock(eb) => eb.give_unique_ids(labeler),
            StatementKind::CondBranch(b) => b.give_unique_ids(labeler),
        }
    }
}
impl Relabel for Expression {
    fn give_unique_ids(&mut self, labeler: &mut impl NodeLabeler) {
        labeler.validify_sentinel_label(&mut self.id);
        match &mut self.kind {
            ExpressionKind::Unit => {}
            ExpressionKind::IntegerLiteral(_) => {}
            ExpressionKind::BoolLiteral(_) => {}
            ExpressionKind::FloatLiteral(_) => {}
            ExpressionKind::StringLiteral(_) => {}
            ExpressionKind::Ident(_) => {}
            ExpressionKind::FuncCall(call) => call.give_unique_ids(labeler),
            ExpressionKind::BinOpExpr(_, lhs, rhs) => {
                lhs.give_unique_ids(labeler);
                rhs.give_unique_ids(labeler);
            }
            ExpressionKind::UnOpExpr(_, arg) => arg.give_unique_ids(labeler),
            ExpressionKind::MemberAccess(data, _) => data.give_unique_ids(labeler),
            ExpressionKind::CondBranch(b) => b.give_unique_ids(labeler),
            ExpressionKind::Block(b) => b.give_unique_ids(labeler),
            ExpressionKind::ExpandedBlock(eb) => eb.give_unique_ids(labeler),
        }
    }
}

impl Relabel for IfThenElse {
    fn give_unique_ids(&mut self, labeler: &mut impl NodeLabeler) {
        labeler.validify_sentinel_label(&mut self.id);
        self.true_case.give_unique_ids(labeler);
        self.condition.give_unique_ids(labeler);
        if let Some(false_case) = &mut self.false_case {
            false_case.give_unique_ids(labeler);
        }
    }
}

impl Relabel for ExpandedBlockExpr {
    fn give_unique_ids(&mut self, labeler: &mut impl NodeLabeler) {
        labeler.validify_sentinel_label(&mut self.id);
        for stmt in self.statements.iter_mut() {
            stmt.give_unique_ids(labeler);
        }
        self.last.give_unique_ids(labeler);
    }
}

impl Relabel for FunctionCall {
    fn give_unique_ids(&mut self, labeler: &mut impl NodeLabeler) {
        labeler.validify_sentinel_label(&mut self.id);
        for arg in self.args.iter_mut() {
            arg.give_unique_ids(labeler);
        }
    }
}

impl Relabel for Reassignment {
    fn give_unique_ids(&mut self, labeler: &mut impl NodeLabeler) {
        labeler.validify_sentinel_label(&mut self.id);
        self.value.give_unique_ids(labeler);
    }
}

pub struct AssignmentSimplifier {
    label: usize,
}
impl AssignmentSimplifier {
    pub fn new() -> Self {
        Self { label: 1 }
    }
}
impl NodeLabeler for AssignmentSimplifier {
    fn transplant_generator(mut other_generator: impl NodeLabeler) -> Self {
        Self {
            label: other_generator.next_label(),
        }
    }
    fn next_label(&mut self) -> usize {
        let label = self.label;
        self.label += 1;
        label
    }
    fn next_label_with_string(&mut self) -> (usize, String) {
        let label = self.next_label();
        (label, format!("__unpack{}", label))
    }
}
impl AssignmentSimplifier {
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
    /// ```
    ///
    /// That is, for each member of the assignment-pattern, generate a new initialization,
    /// That gets assigned one field of the value
    fn simplify_tuple(tuple: &Vec<AssignmentPattern>, value: Expression) -> Vec<Initialisation> {
        let mut assignees = vec![];
        for (field, assignee) in tuple.iter().enumerate() {
            let id = 0;
            let kind = PackedInitialisation {
                typ: None,
                assignee: assignee.clone(),
                value: Expression::label_member_access(value.clone(), field),
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
    /// Let the expand packed assignments in `self` and its descendants.
    ///
    /// Returns false if all the assignments under self are expanded.
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
    fn accept_assignment_simplifier(&mut self, simplifier: &mut AssignmentSimplifier) -> bool;
    fn has_assignments_unpacked(&self) -> bool;
}

impl AcceptsAssignmentSimplifier for Initialisation {
    fn accept_assignment_simplifier(&mut self, simplifier: &mut AssignmentSimplifier) -> bool {
        match &mut self.kind {
            InitialisationKind::Packed(p) => match &p.assignee {
                AssignmentPattern::Identifier(_) => {}
                AssignmentPattern::Tuple(tup) => {
                    let (_id, label) = simplifier.next_label_with_string();
                    let temporary = UnpackedInitialisation {
                        typ: p.typ.clone(),
                        assignee: label.clone(),
                        value: p.value.clone(),
                    };
                    let label = Expression::ident(label);
                    let unpacked_assignments = AssignmentSimplifier::simplify_tuple(&tup, label);
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
        !self.has_assignments_unpacked()
    }

    fn has_assignments_unpacked(&self) -> bool {
        match &self.kind {
            InitialisationKind::PartiallyUnpacked(p) => p
                .unpacked_assignments
                .iter()
                .all(|init| init.has_assignments_unpacked()),
            InitialisationKind::Packed(p) => matches!(p.assignee, AssignmentPattern::Identifier(_)),
        }
    }
}

impl AcceptsAssignmentSimplifier for StatementBlock {
    fn accept_assignment_simplifier(&mut self, simplifier: &mut AssignmentSimplifier) -> bool {
        for s in self.statements.iter_mut() {
            s.accept_assignment_simplifier(simplifier);
        }
        !self.has_assignments_unpacked()
    }
    fn has_assignments_unpacked(&self) -> bool {
        self.statements.iter().all(|s| s.has_assignments_unpacked())
    }
}

impl AcceptsAssignmentSimplifier for ExpandedBlockExpr {
    fn accept_assignment_simplifier(&mut self, simplifier: &mut AssignmentSimplifier) -> bool {
        for s in self.statements.iter_mut() {
            s.accept_assignment_simplifier(simplifier);
        }
        !self.has_assignments_unpacked()
    }
    fn has_assignments_unpacked(&self) -> bool {
        self.statements.iter().all(|s| s.has_assignments_unpacked())
    }
}

impl AcceptsAssignmentSimplifier for Statement {
    fn accept_assignment_simplifier(&mut self, simplifier: &mut AssignmentSimplifier) -> bool {
        match &mut self.kind {
            StatementKind::Initialisation(i) => i.accept_assignment_simplifier(simplifier),
            StatementKind::Block(b) => b.accept_assignment_simplifier(simplifier),
            StatementKind::ExpandedBlock(b) => b.accept_assignment_simplifier(simplifier),
            _ => false,
        }
    }

    fn has_assignments_unpacked(&self) -> bool {
        match &self.kind {
            StatementKind::Initialisation(i) => i.has_assignments_unpacked(),
            StatementKind::Block(b) => b.has_assignments_unpacked(),
            StatementKind::ExpandedBlock(b) => b.has_assignments_unpacked(),
            _ => true,
        }
    }
}

impl AcceptsAssignmentSimplifier for Function {
    fn accept_assignment_simplifier(&mut self, simplifier: &mut AssignmentSimplifier) -> bool {
        self.body.accept_assignment_simplifier(simplifier)
    }

    fn has_assignments_unpacked(&self) -> bool {
        self.body.has_assignments_unpacked()
    }
}

impl AcceptsAssignmentSimplifier for Module {
    fn accept_assignment_simplifier(&mut self, simplifier: &mut AssignmentSimplifier) -> bool {
        for f in self.functions.iter_mut() {
            f.accept_assignment_simplifier(simplifier);
        }
        // we do not simplify globs, as globs may only be simple assignments.
        !self.has_assignments_unpacked()
    }
    fn has_assignments_unpacked(&self) -> bool {
        self.functions.iter().all(|f| f.has_assignments_unpacked())
    }
}

#[derive(Default)]
pub struct BlockExpander {
    label: usize,
    // hoisted_global_decls: HashSet<HoistedFunctionSignature>,
    // /// All the types needed for a
    // hoisted_global_types: HashSet<StructDefinition>,
    // /// All the hoisted variable declarations within a function (blocks as expressions)
    // hoisted_local_function_decls: HashMap<HoistedFunctionSignature, Vec<TypedIdentifier>>,
}

/// Tranform some node into a given variant, and label it.

impl NodeLabeler for BlockExpander {
    fn transplant_generator(mut other_generator: impl NodeLabeler) -> Self {
        Self {
            label: other_generator.next_label(),
        }
    }
    fn next_label(&mut self) -> usize {
        let label = self.label;
        self.label += 1;
        label
    }
}

impl BlockExpander {
    pub fn new() -> Self {
        Self { label: 1 }
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
            _ => (
                block.statements.clone(),
                Expression::unit(self.next_label()),
            ),
        };

        ExpandedBlockExpr {
            id: self.next_label(),
            statements,
            last,
        }
    }
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
    fn accept_block_expander(&mut self, block_expander: &mut BlockExpander) -> bool;
    /// Does this node have only block-expanded descendants?
    ///
    /// Returns false if any descendant is not yet expanded.
    fn has_blocks_expanded(&self) -> bool;
}
impl AcceptsBlockExpander for Statement {
    fn accept_block_expander(&mut self, block_expander: &mut BlockExpander) -> bool {
        if self.has_blocks_expanded() {
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
        !self.has_blocks_expanded()
    }
    fn has_blocks_expanded(&self) -> bool {
        match &self.kind {
            StatementKind::Block(_) => false,

            StatementKind::Initialisation(i) => i.has_blocks_expanded(),
            StatementKind::Reassignment(r) => r.value.has_blocks_expanded(),
            StatementKind::FunctionCall(call) => call.has_blocks_expanded(),
            StatementKind::Return(expr) => expr.has_blocks_expanded(),
            StatementKind::BlockTail(expr) => expr.has_blocks_expanded(),
            StatementKind::ExpandedBlock(b) => b.has_blocks_expanded(),
            StatementKind::CondBranch(b) => b.has_blocks_expanded(),
        }
    }
}

impl AcceptsBlockExpander for IfThenElse {
    fn accept_block_expander(&mut self, block_expander: &mut BlockExpander) -> bool {
        self.condition.accept_block_expander(block_expander);
        self.true_case.accept_block_expander(block_expander);
        if let Some(e) = &mut self.false_case {
            e.accept_block_expander(block_expander);
        }
        !self.has_blocks_expanded()
    }

    fn has_blocks_expanded(&self) -> bool {
        self.condition.has_blocks_expanded()
            && self.true_case.has_blocks_expanded()
            && self
                .false_case
                .as_ref()
                .is_none_or(|e| e.has_blocks_expanded())
    }
}

impl AcceptsBlockExpander for Initialisation {
    fn accept_block_expander(&mut self, block_expander: &mut BlockExpander) -> bool {
        if self.has_blocks_expanded() {
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

        !self.has_blocks_expanded()
    }
    fn has_blocks_expanded(&self) -> bool {
        match &self.kind {
            InitialisationKind::Packed(p) => p.value.has_blocks_expanded(),
            InitialisationKind::PartiallyUnpacked(p) => {
                p.temporary.value.has_blocks_expanded()
                    && p.unpacked_assignments
                        .iter()
                        .all(|s| s.has_blocks_expanded())
            }
        }
    }
}

impl AcceptsBlockExpander for Expression {
    fn accept_block_expander(&mut self, block_expander: &mut BlockExpander) -> bool {
        if self.has_blocks_expanded() {
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

        !self.has_blocks_expanded()
    }
    fn has_blocks_expanded(&self) -> bool {
        match &self.kind {
            ExpressionKind::Block(_block) => false,
            ExpressionKind::FuncCall(call) => call.has_blocks_expanded(),
            ExpressionKind::BinOpExpr(_, lhs, rhs) => {
                lhs.has_blocks_expanded() && rhs.has_blocks_expanded()
            }
            ExpressionKind::UnOpExpr(_, arg) => arg.has_blocks_expanded(),
            ExpressionKind::ExpandedBlock(block) => block.has_blocks_expanded(),
            ExpressionKind::CondBranch(b) => b.has_blocks_expanded(),
            _ => true,
        }
    }
}

impl AcceptsBlockExpander for FunctionCall {
    fn accept_block_expander(&mut self, block_expander: &mut BlockExpander) -> bool {
        if self.has_blocks_expanded() {
            return false;
        }

        for arg in self.args.iter_mut() {
            arg.accept_block_expander(block_expander);
        }

        !self.has_blocks_expanded()
    }
    fn has_blocks_expanded(&self) -> bool {
        self.args.iter().all(|e| e.has_blocks_expanded())
    }
}

impl AcceptsBlockExpander for Function {
    fn accept_block_expander(&mut self, block_expander: &mut BlockExpander) -> bool {
        if self.has_blocks_expanded() {
            return false;
        }
        self.body.accept_block_expander(block_expander);
        !self.has_blocks_expanded()
    }

    fn has_blocks_expanded(&self) -> bool {
        self.body.has_blocks_expanded()
    }
}

impl AcceptsBlockExpander for StatementBlock {
    fn accept_block_expander(&mut self, block_expander: &mut BlockExpander) -> bool {
        if self.has_blocks_expanded() {
            return false;
        }

        for stmt in self.statements.iter_mut() {
            stmt.accept_block_expander(block_expander);
        }
        !self.has_blocks_expanded()
    }
    fn has_blocks_expanded(&self) -> bool {
        self.statements.iter().all(|s| s.has_blocks_expanded())
    }
}

impl AcceptsBlockExpander for ExpandedBlockExpr {
    fn accept_block_expander(&mut self, block_expander: &mut BlockExpander) -> bool {
        if self.has_blocks_expanded() {
            return false;
        }
        self.last.accept_block_expander(block_expander);
        for stmt in self.statements.iter_mut() {
            eprintln!("expanding stmt with id {}", stmt.id);
            stmt.accept_block_expander(block_expander);
        }
        !self.has_blocks_expanded()
    }
    fn has_blocks_expanded(&self) -> bool {
        self.last.has_blocks_expanded() && self.statements.iter().all(|s| s.has_blocks_expanded())
    }
}

impl AcceptsBlockExpander for Module {
    fn accept_block_expander(&mut self, block_expander: &mut BlockExpander) -> bool {
        if self.has_blocks_expanded() {
            return false;
        }

        for func in self.functions.iter_mut() {
            eprintln!("expanding function with name {}", func.name);
            func.accept_block_expander(block_expander);
        }
        !self.has_blocks_expanded()
    }

    fn has_blocks_expanded(&self) -> bool {
        self.functions.iter().all(|f| f.has_blocks_expanded())
    }
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
    fn accept(&mut self, tuple_namer: &mut BlockExpander) -> bool;
    fn is_expanded(&self, tuple_namer: &mut BlockExpander) -> bool;
}

#[cfg(test)]
mod block_expander_tests {
    use crate::zea::test_ast_macros::*;
    use crate::zea::visitors::altering::{AssignmentSimplifier, Relabel};
    use crate::zea::visitors::{AcceptsAssignmentSimplifier, AcceptsBlockExpander, BlockExpander};
    use crate::zea::{
        AssignmentPattern, Expression, ExpressionKind, Function, Initialisation,
        InitialisationKind, Module, PackedInitialisation, Statement, StatementBlock, StatementKind,
        Type,
    };

    #[test]
    fn test_expand_block() {
        let mut block_expander = BlockExpander::new();
        let (mut ast, generator) = label_ast!(using block_expander ; zea_module! {
            imports {}
            exports {}
            globs {}
            funcs {
                func!(main() -> ztyp!(Int); {block!{
                    stmt!(tail expr!(litint 3))
                }})
            }
        });

        let (ast, mut generator) = ast.expand_blocks(generator);
        // eprintln!("{:?}", ast.functions[0]);
        assert!(ast.has_blocks_expanded());

        let (mut ast, mut generator) = label_ast!(using generator;  expr!(block block! {
            stmt!(init pat!(a) ;= expr!(litint 3));
            stmt!(tail expr!(ident a))
        }));
        ast.accept_block_expander(&mut generator);
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
            global_vars: vec![],
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
        let simplifier = AssignmentSimplifier::new();

        let stmt = stmt!(init pat!(a) ;= expr!(litint 1));
        let StatementKind::Initialisation(ref init) = stmt.kind else {
            unreachable!()
        };

        assert!(
            init.has_assignments_unpacked(),
            "Packed(Identifier) should already be considered unpacked"
        );
    }

    #[test]
    fn test_single_level_tuple_unpack() {
        let mut simplifier = AssignmentSimplifier::new();

        let stmt = stmt!(init pat!((a, b)) ;= expr!(ident some_tuple));
        let StatementKind::Initialisation(mut init) = stmt.kind else {
            unreachable!()
        };

        assert!(
            !init.has_assignments_unpacked(),
            "Tuple init should not be considered done before simplification"
        );

        init.accept_assignment_simplifier(&mut simplifier);

        let InitialisationKind::PartiallyUnpacked(ref p) = init.kind else {
            panic!(
                "Expected PartiallyUnpacked after one pass, got {:?}",
                init.kind
            );
        };

        assert_eq!(p.temporary.assignee, "__unpack1");
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

        assert!(init.has_assignments_unpacked());
    }

    #[test]
    fn test_nested_tuple_requires_two_passes() {
        let mut simplifier = AssignmentSimplifier::new();

        // ((a, b), c) := nested_tuple
        let mut stmt = stmt!(init pat!(((a, b), c)) ;= expr!(ident nested_tuple));
        stmt.give_unique_ids(&mut simplifier);
        let StatementKind::Initialisation(ref mut init) = stmt.kind else {
            unreachable!()
        };

        let notdone = init.accept_assignment_simplifier(&mut simplifier);
        // eprintln!("\nafter p1:\n{}", init.indent_print(0));
        assert!(notdone, "First pass should report a change");
        assert!(
            !init.has_assignments_unpacked(),
            "Inner tuple should still need unpacking after first pass"
        );

        let notdone = init.accept_assignment_simplifier(&mut simplifier);
        // eprintln!("\nafter p2:\n{}", init.indent_print(0));
        assert!(!notdone, "Second pass should report a change");
        assert!(
            init.has_assignments_unpacked(),
            "Should be fully done after second pass"
        );
    }

    #[test]
    fn test_module_simplify_assignments_end_to_end() {
        let simplifier = BlockExpander::new();

        let stmt = stmt!(init pat!((a, b, c)) ;= expr!(ident v));
        let StatementKind::Initialisation(init) = stmt.kind else {
            unreachable!()
        };

        let (module, _generator) = wrap_in_module(init).simplify_assignments(simplifier);

        assert!(module.has_assignments_unpacked());

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
                        sub.has_assignments_unpacked(),
                        "All sub-assignments should be fully simplified"
                    );
                }
            }
        }
    }
}
