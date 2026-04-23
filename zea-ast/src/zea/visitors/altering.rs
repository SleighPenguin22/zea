use crate::visualisation::IndentPrint;
use crate::zea::{
    AssignmentPattern, ExpandedBlockExpr, Expression, ExpressionKind, Function, FunctionCall,
    IfThenElse, Initialization, InitializationKind, Module, PackedInitialization,
    PartiallyUnpackedInitialization, Reassignment, SimpleInitialization, Statement, StatementBlock,
    StatementKind,
};

pub trait NodeLabeler {
    /// Start `Self`'s id-generator with the last id that `other_generator` used,
    /// such that [`Self::next_label`] calls will never produce an ID
    /// equal to any of `other_generator`'s ID's.
    fn continue_from_last_id_of(other_generator: impl NodeLabeler) -> Self;
    /// All implementors must ensure that any ID generated is not equal to 0,
    /// as this is a sentinel ID used to signify the need for a fresh ID
    fn next_label(&mut self) -> usize;
    /// Generate the next label, along with a valid unique identifier
    fn next_label_with_ident_string(&mut self) -> (usize, String) {
        let next = self.next_label();
        (next, format!("__label{}", next))
    }
    /// assign a fresh ID only if the current ID is equal to 0.
    fn update_label_if_sentinel(&mut self, current_id: &mut usize) {
        if *current_id == 0 {
            *current_id = self.next_label();
        }
    }
}
pub struct BareNodeLabeler {
    label: usize,
}

impl BareNodeLabeler {
    pub fn new() -> Self {
        Self { label: 1 }
    }
}
impl NodeLabeler for BareNodeLabeler {
    fn continue_from_last_id_of(mut other_generator: impl NodeLabeler) -> Self {
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

/// This trait gives node with a sentinel ID a unique, fresh ID.
pub trait LabelSentinelIDs {
    /// Give each id=0 in the Parse Tree a new, unique ID.
    ///
    fn accept_sentinel_labeler(&mut self, labeler: &mut impl NodeLabeler);

    fn label_sentinel_ids(&mut self) -> BareNodeLabeler {
        let mut labeler = BareNodeLabeler::new();
        self.accept_sentinel_labeler(&mut labeler);
        labeler
    }
}

impl LabelSentinelIDs for Module {
    fn accept_sentinel_labeler(&mut self, labeler: &mut impl NodeLabeler) {
        labeler.update_label_if_sentinel(&mut self.id);
        for glob_var in self.global_vars.iter_mut() {
            glob_var.accept_sentinel_labeler(labeler);
        }

        for func in self.functions.iter_mut() {
            func.accept_sentinel_labeler(labeler);
        }
    }
}

impl LabelSentinelIDs for Initialization {
    fn accept_sentinel_labeler(&mut self, labeler: &mut impl NodeLabeler) {
        labeler.update_label_if_sentinel(&mut self.id);
        match &mut self.kind {
            InitializationKind::PartiallyUnpacked(p) => p.accept_sentinel_labeler(labeler),
            InitializationKind::Packed(p) => p.accept_sentinel_labeler(labeler),
            InitializationKind::Unpacked(u) => {
                for init in u.iter_mut() {
                    init.accept_sentinel_labeler(labeler);
                }
            }
        }
    }
}

impl LabelSentinelIDs for PartiallyUnpackedInitialization {
    fn accept_sentinel_labeler(&mut self, labeler: &mut impl NodeLabeler) {
        self.temporary.accept_sentinel_labeler(labeler);
        for unpack in self.unpacked_assignments.iter_mut() {
            unpack.accept_sentinel_labeler(labeler);
        }
    }
}

impl LabelSentinelIDs for PackedInitialization {
    fn accept_sentinel_labeler(&mut self, labeler: &mut impl NodeLabeler) {
        self.value.accept_sentinel_labeler(labeler);
    }
}
impl LabelSentinelIDs for SimpleInitialization {
    fn accept_sentinel_labeler(&mut self, labeler: &mut impl NodeLabeler) {
        self.value.accept_sentinel_labeler(labeler);
    }
}
impl LabelSentinelIDs for Function {
    fn accept_sentinel_labeler(&mut self, labeler: &mut impl NodeLabeler) {
        labeler.update_label_if_sentinel(&mut self.id);
        self.body.accept_sentinel_labeler(labeler);
    }
}
impl LabelSentinelIDs for StatementBlock {
    fn accept_sentinel_labeler(&mut self, labeler: &mut impl NodeLabeler) {
        labeler.update_label_if_sentinel(&mut self.id);
        for stmt in self.statements.iter_mut() {
            stmt.accept_sentinel_labeler(labeler);
        }
    }
}

impl LabelSentinelIDs for Statement {
    fn accept_sentinel_labeler(&mut self, labeler: &mut impl NodeLabeler) {
        labeler.update_label_if_sentinel(&mut self.id);
        match &mut self.kind {
            StatementKind::Initialization(i) => i.accept_sentinel_labeler(labeler),
            StatementKind::Reassignment(r) => r.accept_sentinel_labeler(labeler),
            StatementKind::FunctionCall(call) => call.accept_sentinel_labeler(labeler),
            StatementKind::Return(e) => e.accept_sentinel_labeler(labeler),
            StatementKind::BlockTail(e) => e.accept_sentinel_labeler(labeler),
            StatementKind::Block(b) => b.accept_sentinel_labeler(labeler),
            StatementKind::ExpandedBlock(eb) => eb.accept_sentinel_labeler(labeler),
            StatementKind::IfThenElse(b) => b.accept_sentinel_labeler(labeler),
        }
    }
}
impl LabelSentinelIDs for Expression {
    fn accept_sentinel_labeler(&mut self, labeler: &mut impl NodeLabeler) {
        labeler.update_label_if_sentinel(&mut self.id);
        match &mut self.kind {
            ExpressionKind::Unit => {}
            ExpressionKind::IntegerLiteral(_) => {}
            ExpressionKind::BoolLiteral(_) => {}
            ExpressionKind::FloatLiteral(_) => {}
            ExpressionKind::StringLiteral(_) => {}
            ExpressionKind::Ident(_) => {}
            ExpressionKind::FunctionCall(call) => call.accept_sentinel_labeler(labeler),
            ExpressionKind::BinOpExpr(_, lhs, rhs) => {
                lhs.accept_sentinel_labeler(labeler);
                rhs.accept_sentinel_labeler(labeler);
            }
            ExpressionKind::UnOpExpr(_, arg) => arg.accept_sentinel_labeler(labeler),
            ExpressionKind::MemberAccess(data, _) => data.accept_sentinel_labeler(labeler),
            ExpressionKind::IfThenElse(b) => b.accept_sentinel_labeler(labeler),
            ExpressionKind::Block(b) => b.accept_sentinel_labeler(labeler),
            ExpressionKind::ExpandedBlock(eb) => eb.accept_sentinel_labeler(labeler),
        }
    }
}

impl LabelSentinelIDs for IfThenElse {
    fn accept_sentinel_labeler(&mut self, labeler: &mut impl NodeLabeler) {
        labeler.update_label_if_sentinel(&mut self.id);
        self.true_case.accept_sentinel_labeler(labeler);
        self.condition.accept_sentinel_labeler(labeler);
        if let Some(false_case) = &mut self.false_case {
            false_case.accept_sentinel_labeler(labeler);
        }
    }
}

impl LabelSentinelIDs for ExpandedBlockExpr {
    fn accept_sentinel_labeler(&mut self, labeler: &mut impl NodeLabeler) {
        labeler.update_label_if_sentinel(&mut self.id);
        for stmt in self.statements.iter_mut() {
            stmt.accept_sentinel_labeler(labeler);
        }
        self.last.accept_sentinel_labeler(labeler);
    }
}

impl LabelSentinelIDs for FunctionCall {
    fn accept_sentinel_labeler(&mut self, labeler: &mut impl NodeLabeler) {
        labeler.update_label_if_sentinel(&mut self.id);
        for arg in self.args.iter_mut() {
            arg.accept_sentinel_labeler(labeler);
        }
    }
}

impl LabelSentinelIDs for Reassignment {
    fn accept_sentinel_labeler(&mut self, labeler: &mut impl NodeLabeler) {
        labeler.update_label_if_sentinel(&mut self.id);
        self.value.accept_sentinel_labeler(labeler);
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
    fn continue_from_last_id_of(mut other_generator: impl NodeLabeler) -> Self {
        Self {
            label: other_generator.next_label(),
        }
    }
    fn next_label(&mut self) -> usize {
        let label = self.label;
        self.label += 1;
        label
    }
    fn next_label_with_ident_string(&mut self) -> (usize, String) {
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
    fn simplify_tuple_assignment(
        tuple: &Vec<AssignmentPattern>,
        value: Expression,
    ) -> Vec<Initialization> {
        let mut assignees = vec![];
        for (field, assignee) in tuple.iter().enumerate() {
            let id = 0;
            let kind = PackedInitialization {
                typ: None,
                assignee: assignee.clone(),
                value: Expression::tuple_member_access(value.clone(), field),
            };
            let init = Initialization {
                id,
                kind: InitializationKind::Packed(kind),
            };
            assignees.push(init);
        }

        assignees
    }

    pub fn flatten_partially_unpacked_initialization(init: &mut Initialization) {
        match &mut init.kind {
            InitializationKind::Packed(p) => {
                unreachable!("unexpected packed init:\n{}", p.indent_print(0))
            }
            InitializationKind::PartiallyUnpacked(pu) => {
                let temps = pu.get_cloned_temps();
                init.kind = InitializationKind::Unpacked(temps);
            }
            InitializationKind::Unpacked(_) => {}
        }
    }
    pub fn flatten_simple_packed_initialization(init: &mut Initialization) {
        match &mut init.kind {
            InitializationKind::Packed(p) => {
                let AssignmentPattern::Identifier(assignee) = &p.assignee else {
                    unreachable!(
                        "unexpected tuple assignee when flattening packed:\n{}",
                        init.indent_print(0)
                    )
                };
                let simple = SimpleInitialization {
                    id: 0,
                    typ: p.typ.clone(),
                    assignee: assignee.clone(),
                    value: p.value.clone(),
                };
                init.kind = InitializationKind::Unpacked(vec![simple]);
            }
            InitializationKind::PartiallyUnpacked(_) => unreachable!(
                "expected simple packed assignment, got:\n{}",
                init.indent_print(0)
            ),
            InitializationKind::Unpacked(_) => {}
        }
    }
}

impl PartiallyUnpackedInitialization {
    pub fn get_cloned_temps(&self) -> Vec<SimpleInitialization> {
        let mut temps = vec![self.temporary.clone()];
        for init in self.unpacked_assignments.iter() {
            match &init.kind {
                InitializationKind::Packed(_) => {
                    panic!("cannot get temps before assignments are simplified")
                }
                InitializationKind::PartiallyUnpacked(pu) => {
                    temps.extend(pu.get_cloned_temps());
                }
                InitializationKind::Unpacked(u) => {
                    temps.extend(u.clone());
                }
            }
        }
        temps
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
    fn simplify_assignments(&mut self) -> AssignmentSimplifier {
        let mut simplifier = AssignmentSimplifier::new();
        self.accept_assignment_simplifier(&mut simplifier);
        simplifier
    }
}

impl AcceptsAssignmentSimplifier for Initialization {
    fn accept_assignment_simplifier(&mut self, simplifier: &mut AssignmentSimplifier) -> bool {
        match &mut self.kind {
            InitializationKind::Packed(p) => match &p.assignee {
                AssignmentPattern::Identifier(_) => {
                    AssignmentSimplifier::flatten_simple_packed_initialization(self)
                }
                AssignmentPattern::Tuple(tup) => {
                    let (id, label) = simplifier.next_label_with_ident_string();
                    let temporary = SimpleInitialization::untyped(&label, p.value.clone());
                    let label = Expression::ident(&label);
                    let unpacked_assignments =
                        AssignmentSimplifier::simplify_tuple_assignment(&tup, label);
                    let partially_unpacked = PartiallyUnpackedInitialization {
                        temporary,
                        unpacked_assignments,
                    };
                    self.kind = InitializationKind::PartiallyUnpacked(partially_unpacked);
                }
            },

            InitializationKind::PartiallyUnpacked(p) => {
                let inits = &mut p.unpacked_assignments;
                for init in inits.iter_mut() {
                    init.accept_assignment_simplifier(simplifier);
                }
                AssignmentSimplifier::flatten_partially_unpacked_initialization(self);
            }
            &mut InitializationKind::Unpacked(_) => {}
        };

        !self.has_assignments_unpacked()
    }

    fn has_assignments_unpacked(&self) -> bool {
        match &self.kind {
            InitializationKind::Unpacked(_) => true,
            _ => {
                println!("non-unpacked init:\n{}", self.indent_print(1));
                false
            }
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
        self.statements
            .iter()
            .all(Statement::has_assignments_unpacked)
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
        self.statements
            .iter()
            .all(Statement::has_assignments_unpacked)
    }
}

impl AcceptsAssignmentSimplifier for Statement {
    fn accept_assignment_simplifier(&mut self, simplifier: &mut AssignmentSimplifier) -> bool {
        match &mut self.kind {
            StatementKind::Initialization(i) => i.accept_assignment_simplifier(simplifier),
            StatementKind::Block(b) => b.accept_assignment_simplifier(simplifier),
            StatementKind::ExpandedBlock(b) => b.accept_assignment_simplifier(simplifier),
            _ => false,
        }
    }

    fn has_assignments_unpacked(&self) -> bool {
        match &self.kind {
            StatementKind::Initialization(i) => i.has_assignments_unpacked(),
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
    fn continue_from_last_id_of(mut other_generator: impl NodeLabeler) -> Self {
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
            StatementKind::Initialization(i) => i.accept_block_expander(block_expander),
            StatementKind::Reassignment(r) => r.value.accept_block_expander(block_expander),
            StatementKind::FunctionCall(call) => call.accept_block_expander(block_expander),
            StatementKind::Return(expr) => expr.accept_block_expander(block_expander),
            StatementKind::BlockTail(expr) => expr.accept_block_expander(block_expander),
            StatementKind::ExpandedBlock(b) => b.accept_block_expander(block_expander),
            StatementKind::IfThenElse(b) => b.accept_block_expander(block_expander),
        };
        !self.has_blocks_expanded()
    }
    fn has_blocks_expanded(&self) -> bool {
        match &self.kind {
            StatementKind::Block(_) => false,

            StatementKind::Initialization(i) => i.has_blocks_expanded(),
            StatementKind::Reassignment(r) => r.value.has_blocks_expanded(),
            StatementKind::FunctionCall(call) => call.has_blocks_expanded(),
            StatementKind::Return(expr) => expr.has_blocks_expanded(),
            StatementKind::BlockTail(expr) => expr.has_blocks_expanded(),
            StatementKind::ExpandedBlock(b) => b.has_blocks_expanded(),
            StatementKind::IfThenElse(b) => b.has_blocks_expanded(),
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

impl AcceptsBlockExpander for Initialization {
    fn accept_block_expander(&mut self, block_expander: &mut BlockExpander) -> bool {
        if self.has_blocks_expanded() {
            return false;
        }

        match &mut self.kind {
            InitializationKind::Packed(p) => {
                p.value.accept_block_expander(block_expander);
            }
            InitializationKind::PartiallyUnpacked(p) => {
                p.temporary.value.accept_block_expander(block_expander);
                for s in p.unpacked_assignments.iter_mut() {
                    s.accept_block_expander(block_expander);
                }
            }
            InitializationKind::Unpacked(_) => {
                panic!("block expansion should happen before assignment unpacking")
            }
        }

        !self.has_blocks_expanded()
    }
    fn has_blocks_expanded(&self) -> bool {
        match &self.kind {
            InitializationKind::Packed(p) => p.value.has_blocks_expanded(),
            InitializationKind::PartiallyUnpacked(p) => {
                p.temporary.value.has_blocks_expanded()
                    && p.unpacked_assignments
                        .iter()
                        .all(|s| s.has_blocks_expanded())
            }
            InitializationKind::Unpacked(_) => {
                panic!("block expansion should happens before assignemnt unpacking")
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
            ExpressionKind::FunctionCall(call) => call.accept_block_expander(block_expander),
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
            ExpressionKind::IfThenElse(b) => b.accept_block_expander(block_expander),
        };

        !self.has_blocks_expanded()
    }
    fn has_blocks_expanded(&self) -> bool {
        match &self.kind {
            ExpressionKind::Block(_block) => false,
            ExpressionKind::FunctionCall(call) => call.has_blocks_expanded(),
            ExpressionKind::BinOpExpr(_, lhs, rhs) => {
                lhs.has_blocks_expanded() && rhs.has_blocks_expanded()
            }
            ExpressionKind::UnOpExpr(_, arg) => arg.has_blocks_expanded(),
            ExpressionKind::ExpandedBlock(block) => block.has_blocks_expanded(),
            ExpressionKind::IfThenElse(b) => b.has_blocks_expanded(),
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
    use crate::helper_impls::assert_structural_eq;
    use crate::helper_impls::StructuralEq;
    use crate::visualisation::IndentPrint;
    // use crate::visualisation::IndentPrint;
    use crate::zea::test_ast_macros::*;
    use crate::zea::visitors::altering::{AssignmentSimplifier, LabelSentinelIDs};
    use crate::zea::visitors::{AcceptsAssignmentSimplifier, AcceptsBlockExpander, BlockExpander};
    use crate::zea::{
        AssignmentPattern, Expression, ExpressionKind, Function, Initialization,
        InitializationKind, Module, NodeLabeler, PackedInitialization, Statement, StatementBlock,
        StatementKind, TypeSpecifier,
    };

    #[test]
    fn test_expand_block() {
        let block_expander = BlockExpander::new();
        let (ast, generator) = label_ast!(using block_expander ; zea_module! {
            imports {}
            exports {}
            globs {}
            funcs {
                func!(main() -> ztyp!(Int); {block!{
                    stmt!(tail expr!(litint 3))
                }})
            }
            structs {}
        });

        let (ast, generator) = ast.expand_blocks(generator);
        // eprintln!("{:?}", ast.functions[0]);
        assert!(ast.has_blocks_expanded());

        let (mut ast, generator) = label_ast!(using generator;  expr!(block block! {
            stmt!(init pat!(a) ;= expr!(litint 3));
            stmt!(tail expr!(ident a))
        }));
        ast.accept_block_expander(&mut BlockExpander::continue_from_last_id_of(generator));
        let after = ast;
        let ExpressionKind::ExpandedBlock(expanded) = after.kind else {
            unreachable!()
        };
        assert_structural_eq!(
            expanded.statements[0],
            stmt!(init pat!(a) ;= expr!(litint 3))
        );

        assert_structural_eq!(expanded.last, expr!(ident a));
    }

    fn wrap_in_module(init: Initialization) -> Module {
        Module {
            id: 0,
            imports: vec![],
            exports: vec![],
            global_vars: vec![],
            functions: vec![Function {
                id: 0,
                name: "test".to_string(),
                params: vec![],
                returns: TypeSpecifier::Basic("Unit".to_string()),
                body: StatementBlock {
                    id: 0,
                    statements: vec![Statement {
                        id: 0,
                        kind: StatementKind::Initialization(init),
                    }],
                },
            }],
            struct_definitions: vec![],
        }
    }

    #[test]
    fn test_simple_ident_init_is_already_done() {
        let _simplifier = AssignmentSimplifier::new();

        let mut stmt = stmt!(init pat!(a) ;= expr!(litint 1));
        let g = stmt.label_sentinel_ids();
        stmt.accept_assignment_simplifier(&mut AssignmentSimplifier::continue_from_last_id_of(g));
        let StatementKind::Initialization(ref init) = stmt.kind else {
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
        let StatementKind::Initialization(mut init) = stmt.kind else {
            unreachable!()
        };

        assert!(
            !init.has_assignments_unpacked(),
            "Tuple init should not be considered done before simplification"
        );

        init.accept_assignment_simplifier(&mut simplifier);

        let InitializationKind::PartiallyUnpacked(ref p) = init.kind else {
            panic!(
                "Expected PartiallyUnpacked after one pass, got {:?}",
                init.kind
            );
        };

        assert_eq!(p.temporary.assignee, "__unpack1");
        assert_eq!(p.unpacked_assignments.len(), 2);

        for sub in &p.unpacked_assignments {
            let InitializationKind::Packed(ref packed) = sub.kind else {
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
        stmt.accept_sentinel_labeler(&mut simplifier);
        let StatementKind::Initialization(ref mut init) = stmt.kind else {
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

        let mut stmt = stmt!(init pat!((a, b, c)) ;= expr!(ident v));

        let g = stmt.label_sentinel_ids();
        let mut g = AssignmentSimplifier::continue_from_last_id_of(g);
        while !stmt.accept_assignment_simplifier(&mut g) {}

        println!("_________________--\nMODULE END TO END\n\n");
        println!("{}", stmt.indent_print(0));

        println!("BOB");
        assert!(stmt.has_assignments_unpacked());
    }
}
