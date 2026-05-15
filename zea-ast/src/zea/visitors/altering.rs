use crate::visualisation::IndentPrint;
use crate::zea::visitors::annotating::{ScopeAnnotations, ScopedIdentifier};
use crate::zea::visitors::{
    walk_mut_block, walk_mut_branch, walk_mut_call, walk_mut_expr, walk_mut_funcdef, walk_mut_initblock,
    walk_mut_module, walk_mut_reassignment, walk_mut_stmt, walk_mut_structdef, walk_mut_sugared_block,
    walk_mut_unpacked_init, Transfomer, Visitor,
};
use crate::zea::{
    AssignmentPattern, ExpandedBlockExpr, Expression, ExpressionKind, Function, FunctionCall,
    IfThenElse, InitializationBlock, InitializationKind, Module, NodeId, PackedInitialization,
    Reassignment, SimpleInitialization, Statement, StatementBlock, StatementKind,
    StructDataTypeDefinition,
};

pub trait NodeLabeler: Sized {
    /// Start `Self`'s id-generator with the last id that `other_generator` used,
    /// such that [`Self::next_id`] calls will never produce an ID
    /// equal to any of `other_generator`'s ID's.
    fn labeler_from(other_generator: impl NodeLabeler) -> Self;
    fn labeler_into<V: NodeLabeler>(self) -> V {
        NodeLabeler::labeler_from(self)
    }
    /// All implementors must ensure that any ID generated is not equal to 0,
    /// as this is a sentinel ID used to signify the need for a fresh ID
    fn next_id(&mut self) -> NodeId;
    /// Generate the next label, along with a valid unique identifier
    fn next_label_with_ident_string(&mut self) -> (NodeId, String) {
        let next = self.next_id();
        (next, format!("__synthetic{}", next))
    }
    /// assign a fresh ID only if the current ID is equal to 0.
    fn update_label(&mut self, current_id: &mut NodeId) {
        if *current_id == NodeId(0) {
            *current_id = self.next_id();
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
    fn next_id(&mut self) -> NodeId {
        let l = NodeId(self.label);
        self.label += 1;
        l
    }
    fn labeler_from(mut other_generator: impl NodeLabeler) -> Self {
        Self {
            label: other_generator.next_id().0,
        }
    }
}

impl Transfomer for BareNodeLabeler {
    fn visit_block(&mut self, block: &mut ExpandedBlockExpr) {
        self.update_label(&mut block.id);
        walk_mut_block(self, block);
    }
    fn visit_branch(&mut self, branch: &mut IfThenElse) {
        self.update_label(&mut branch.id);
        walk_mut_branch(self, branch);
    }
    fn visit_call(&mut self, call: &mut FunctionCall) {
        self.update_label(&mut call.id);
        walk_mut_call(self, call);
    }
    fn visit_expr(&mut self, expr: &mut Expression) {
        self.update_label(&mut expr.id);
        walk_mut_expr(self, expr);
    }
    fn visit_funcdef(&mut self, funcdef: &mut Function) {
        self.update_label(&mut funcdef.id);
        walk_mut_funcdef(self, funcdef)
    }
    fn visit_init(&mut self, init: &mut SimpleInitialization) {
        self.update_label(&mut init.id);
        walk_mut_unpacked_init(self, init);
    }
    fn visit_initblock(&mut self, init: &mut InitializationBlock) {
        self.update_label(&mut init.id);
    }
    fn visit_module(&mut self, module: &mut Module) {
        self.update_label(&mut module.id);
        walk_mut_module(self, module);
    }
    fn visit_reassignment(&mut self, reinit: &mut Reassignment) {
        self.update_label(&mut reinit.id);
        walk_mut_reassignment(self, reinit);
    }
    fn visit_stmt(&mut self, stmt: &mut Statement) {
        self.update_label(&mut stmt.id);
        walk_mut_stmt(self, stmt);
    }
    fn visit_structdef(&mut self, structdef: &mut StructDataTypeDefinition) {
        self.update_label(&mut structdef.id);
        walk_mut_structdef(self, structdef);
    }
    fn visit_sugared_block(&mut self, sugared_block: &mut StatementBlock) {
        self.update_label(&mut sugared_block.id);
        walk_mut_sugared_block(self, sugared_block);
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
    fn labeler_from(mut other_generator: impl NodeLabeler) -> Self {
        Self {
            label: other_generator.next_id().0,
        }
    }
    fn next_id(&mut self) -> NodeId {
        let label = self.label;
        self.label += 1;
        NodeId(label)
    }
    fn next_label_with_ident_string(&mut self) -> (NodeId, String) {
        let label = self.next_id();
        (label, format!("__unpack{}", label))
    }
}

impl Transfomer for AssignmentSimplifier {
    fn visit_initblock(&mut self, init: &mut InitializationBlock) {
        match init.kind {
            InitializationKind::Packed(_) => {
                init.kind = InitializationKind::Unpacked(self.expand_assignment(init.clone()));
            }
            InitializationKind::Unpacked(_) => {}
        }
        walk_mut_initblock(self, init)
    }
}

impl AssignmentSimplifier {
    /// synthesize a [`SimpleInitialization`] for use in assignment expansion.
    ///
    /// Also generates an expression referencing that
    fn synthesize_temporary(&mut self, value: Expression) -> (SimpleInitialization, Expression) {
        let (id, label) = self.next_label_with_ident_string();
        let mut init = SimpleInitialization::untyped(&label, value);
        init.id = id;
        let ident_expr =
            Expression::scoped_local(init.assignee.clone(), id).with_id(self.next_id());
        (init, ident_expr)
    }
    fn synthesize_unpacking_tuple_item(
        &mut self,
        assignee: AssignmentPattern,
        value: Expression,
        index: usize,
    ) -> PackedInitialization {
        let mut member_access = Expression::member_access(value, format!("_{index}"));
        member_access.id = self.next_id();
        PackedInitialization::untyped(assignee, member_access)
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
    /// __unpack0 := label._0;
    /// a := __unpack0._0;
    /// b := __unpack0._1;
    /// c := label._1;
    /// ```
    ///
    /// That is, for each member of the assignment-pattern, generate a new initialization,
    /// That gets assigned one field of the value
    fn expand_assignment(&mut self, init: InitializationBlock) -> Vec<SimpleInitialization> {
        match init.kind {
            InitializationKind::Packed(p) => self.expand_packed_init(p),
            InitializationKind::Unpacked(u) => u,
        }
    }
    fn expand_packed_init(&mut self, init: PackedInitialization) -> Vec<SimpleInitialization> {
        match init.assignee {
            AssignmentPattern::Identifier(i) => {
                let mut simple = SimpleInitialization::untyped(&i, init.value);
                simple.id = self.next_id();
                vec![simple]
            }
            AssignmentPattern::Tuple(t) => {
                let (temp, ident_expr) = self.synthesize_temporary(init.value.clone());

                let mut res = vec![temp];
                for (index, assignee) in t.into_iter().enumerate() {
                    let sub_init =
                        self.synthesize_unpacking_tuple_item(assignee, ident_expr.clone(), index);

                    let recursive_unpacked = self.expand_packed_init(sub_init);
                    res.extend(recursive_unpacked)
                }
                res
            }
        }
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
    fn labeler_from(mut other_generator: impl NodeLabeler) -> Self {
        Self {
            label: other_generator.next_id().0,
        }
    }
    fn next_id(&mut self) -> NodeId {
        let label = self.label;
        self.label += 1;
        NodeId(label)
    }
}

impl Transfomer for BlockExpander {
    fn visit_stmt(&mut self, stmt: &mut Statement) {
        match &mut stmt.kind {
            StatementKind::Initialization(_) => {}
            StatementKind::Reassignment(_) => {}
            StatementKind::FunctionCall(_) => {}
            StatementKind::Return(_) => {}
            StatementKind::BlockTail(_) => {}
            StatementKind::SugaredBlock(sb) => {
                let eb = self.expand_expr_block(sb.clone());
                stmt.kind = StatementKind::Block(eb);
            }
            StatementKind::Block(_) => {}
            StatementKind::IfThenElse(_) => {}
        }
        walk_mut_stmt(self, stmt);
    }
}

impl BlockExpander {
    pub fn new() -> Self {
        Self { label: 1 }
    }

    /// Expand some expression block
    ///
    /// Inserts a unit-tail if the block does not end with a tail expression.
    pub fn expand_expr_block(&mut self, mut block: StatementBlock) -> ExpandedBlockExpr {
        let (statements, last) = match block.statements.last().cloned() {
            Some(Statement {
                kind: StatementKind::BlockTail(tail),
                ..
            }) => {
                // we have already captured the tail expression in the pattern match,
                // and thus can truncate the vector to avoid a clone.
                block.statements.truncate(block.statements.len() - 1);
                (block.statements, tail)
            }
            _ => (block.statements, Expression::unit(self.next_id())),
        };

        ExpandedBlockExpr {
            id: self.next_id(),
            statements,
            last,
        }
    }
}

pub trait AcceptsTupleNamer {
    /// Let the expander perform some transformation on `self`. Return false if no changes have been made.
    /// Repeatedly calling this method is guaranteed to eventually return false:
    ///
    /// ```ignore
    /// let ast = StatementBlock {
    ///     id: NodeId::sentinel(,
    ///     statements: vec![...]
    /// };
    /// let mut expander = NodeExpander::new()
    /// while !ast.accept(&mut expander) {} // will always terminate
    /// ```
    fn accept(&mut self, tuple_namer: &mut BlockExpander) -> bool;
    fn is_expanded(&self, tuple_namer: &mut BlockExpander) -> bool;
}

pub struct IdentifierScope {
    /// map an Ident-expression to a scoped identifier and the nearest enclosing block.
    scope_stack: Vec<NodeId>,
    scope_annotations: ScopeAnnotations,
}

pub struct NotInScopeError {
    ident: String,
    scope_id: usize,
}

impl IdentifierScope {
    pub fn new(ast: &Module) -> Self {
        Self {
            scope_stack: vec![ast.id],
            scope_annotations: todo!(),
        }
    }
    fn enter_scope(&mut self, scope: NodeId) {
        self.scope_stack.push(scope)
    }
    fn exit_scope(&mut self) {
        self.scope_stack.pop();
    }
    fn current_scope(&self) -> NodeId {
        *self.scope_stack.last().unwrap()
    }

    pub(crate) fn visit_module(&mut self, module: &mut Module) -> Result<(), NotInScopeError> {
        for glob_var in module.global_vars.iter_mut() {
            self.visit_init(glob_var)?;
        }

        for func in module.functions.iter_mut() {
            self.enter_scope(func.body.id);
            for stmt in func.body.statements.iter_mut() {
                self.visit_stmt(stmt)?;
            }
        }
        Ok(())
    }
    fn visit_init(&mut self, init: &mut InitializationBlock) -> Result<(), NotInScopeError> {
        let InitializationKind::Unpacked(u) = &mut init.kind else {
            unreachable!("assignments should be expanded")
        };
        for init in u.iter_mut() {
            self.visit_expr(&mut init.value)?;
        }
        Ok(())
    }
    fn visit_expr(&mut self, expr: &mut Expression) -> Result<(), NotInScopeError> {
        match &mut expr.kind {
            ExpressionKind::Unit => {}
            ExpressionKind::IntegerLiteral(_) => {}
            ExpressionKind::BoolLiteral(_) => {}
            ExpressionKind::FloatLiteral(_) => {}
            ExpressionKind::StringLiteral(_) => {}
            ExpressionKind::UnScopedIdent(i) => {
                let scoped_ident = self.search_for(i)?;
                expr.kind = ExpressionKind::ScopedIdent(scoped_ident)
            }
            ExpressionKind::ScopedIdent(_) => {}
            ExpressionKind::FunctionCall(call) => self.visit_call(call)?,
            ExpressionKind::BinOpExpr(_, lhs, rhs) => {
                self.visit_expr(lhs)?;
                self.visit_expr(rhs)?;
            }
            ExpressionKind::UnOpExpr(_, arg) => self.visit_expr(arg)?,
            ExpressionKind::MemberAccess(data, _) => self.visit_expr(data)?,
            ExpressionKind::IfThenElse(ite) => self.visit_branch(ite)?,
            ExpressionKind::ExpandedBlock(eb) => self.visit_block(eb)?,
            ExpressionKind::Block(_) => unreachable!("blocks should be expanded"),
        }
        Ok(())
    }
    fn visit_block(&mut self, block: &mut ExpandedBlockExpr) -> Result<(), NotInScopeError> {
        self.enter_scope(block.id);
        for stmt in block.statements.iter_mut() {
            self.visit_stmt(stmt)?;
        }
        self.exit_scope();
        Ok(())
    }
    fn visit_stmt(&mut self, stmt: &mut Statement) -> Result<(), NotInScopeError> {
        match &mut stmt.kind {
            StatementKind::Initialization(init) => self.visit_init(init),
            StatementKind::Reassignment(reinit) => self.visit_reassignment(reinit),
            StatementKind::FunctionCall(call) => self.visit_call(call),
            StatementKind::Return(e) => self.visit_expr(e),
            StatementKind::BlockTail(e) => self.visit_expr(e),
            StatementKind::Block(eb) => self.visit_block(eb),
            StatementKind::IfThenElse(ite) => self.visit_branch(ite),
            StatementKind::SugaredBlock(_) => unreachable!("blocks should be expanded"),
        }
    }
    fn search_for(&mut self, _ident: &str) -> Result<ScopedIdentifier, NotInScopeError> {
        todo!()
    }

    fn visit_branch(&mut self, branch: &mut IfThenElse) -> Result<(), NotInScopeError> {
        self.visit_expr(&mut branch.condition)?;
        self.visit_expr(&mut branch.true_case)?;
        if let Some(false_case) = &mut branch.false_case {
            self.visit_expr(false_case)?;
        }
        Ok(())
    }

    fn visit_call(&mut self, call: &mut FunctionCall) -> Result<(), NotInScopeError> {
        for arg in call.args.iter_mut() {
            self.visit_expr(arg)?;
        }
        Ok(())
    }

    fn visit_reassignment(&mut self, reinit: &mut Reassignment) -> Result<(), NotInScopeError> {
        self.visit_expr(&mut reinit.value)
    }
}

#[cfg(test)]
mod block_expander_tests {
    use crate::helper_impls::assert_structural_eq;
    use crate::helper_impls::StructuralEq;
    use crate::visualisation::IndentPrint;
    use crate::zea::test_ast_macros::*;
    use crate::zea::visitors::altering::{AssignmentSimplifier, LabelSentinelIDs};
    use crate::zea::visitors::{AcceptsAssignmentSimplifier, AcceptsBlockExpander, BlockExpander};
    // use crate::visualisation::IndentPrint;
    use crate::zea::NodeId;
    use crate::zea::{
        AssignmentPattern, Expression, ExpressionKind, Function, InitializationBlock,
        InitializationKind, Module, NodeLabeler, PackedInitialization, Statement, StatementBlock,
        StatementKind, TypeSpecifier,
    };

    #[test]
    fn test_expand_block() {
        let block_expander = BlockExpander::new();
        let (mut ast, generator) = label_ast!(using block_expander ; zea_module! {
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

        let generator = ast.expand_blocks_with(generator);
        // eprintln!("{:?}", ast.functions[0]);
        assert!(ast.has_blocks_expanded());

        let (mut ast, generator) = label_ast!(using generator;  expr!(block block! {
            stmt!(init pat!(a) ;= expr!(litint 3));
            stmt!(tail expr!(ident a))
        }));
        ast.accept_block_expander(&mut BlockExpander::labeler_from(generator));
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

    fn wrap_in_module(init: InitializationBlock) -> Module {
        Module {
            id: NodeId::sentinel(),
            imports: vec![],
            exports: vec![],
            global_vars: vec![],
            functions: vec![Function {
                id: NodeId::sentinel(),
                name: "test".to_string(),
                params: vec![],
                returns: TypeSpecifier::Basic("Unit".to_string()),
                body: StatementBlock {
                    id: NodeId::sentinel(),
                    statements: vec![Statement {
                        id: NodeId::sentinel(),
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
        stmt.accept_assignment_unpacker(&mut AssignmentSimplifier::labeler_from(g));
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

        init.accept_assignment_unpacker(&mut simplifier);

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
    fn test_nested_tuple_structure() {
        // ((a, b), c) := nested_tuple
        let (mut stmt, g) =
            label_ast!(fresh stmt!(init pat!(((a, b), c)) ;= expr!(ident nested_tuple)));

        let _g = stmt.simplify_assignments_with(g);
        println!("{}", stmt.indent_print(1));
    }

    #[test]
    fn test_module_simplify_assignments_end_to_end() {
        let (mut stmt, g) = label_ast!(fresh zea_module!(
        imports {}
            exports {}
            globs {}
            funcs {
        func!(f() -> ztyp!(Int); {block!{
                stmt!(init pat!((a, b, c)) ;= expr!(ident v))
            }})
            }
            structs {}
        ));

        let g = AssignmentSimplifier::labeler_from(g);

        stmt.simplify_assignments_with(g);

        println!("_________________--\nMODULE END TO END\n\n");
        println!("{}", stmt.indent_print(0));

        println!("BOB");
        assert!(stmt.has_assignments_unpacked());
    }
}
