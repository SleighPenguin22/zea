use crate::zea::visitors::annotating::{ScopeAnnotations, ScopedIdentifier};
use crate::zea::visitors::{
    walk_mut_block, walk_mut_branch, walk_mut_call, walk_mut_expr, walk_mut_funcdef, walk_mut_initblock,
    walk_mut_module, walk_mut_reassignment, walk_mut_stmt, walk_mut_structdef, walk_mut_unpacked_init,
    Transfomer,
};
use crate::zea::{
    AssignmentPattern, BlockExpression, Expression, ExpressionKind, Function, FunctionCall,
    IfThenElse, InitializationBlock, InitializationKind, Module, NodeId, PackedInitialization,
    Reassignment, SimpleInitialization, Statement, StatementKind, StructDataTypeDefinition,
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
    type TransformerOk = ();
    type TransformerError = ();
    fn visit_block(
        &mut self,
        block: &mut BlockExpression,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.update_label(&mut block.id);
        walk_mut_block(self, block)
    }
    fn visit_branch(
        &mut self,
        branch: &mut IfThenElse,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.update_label(&mut branch.id);
        walk_mut_branch(self, branch)
    }
    fn visit_call(
        &mut self,
        call: &mut FunctionCall,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.update_label(&mut call.id);
        walk_mut_call(self, call)
    }
    fn visit_expr(
        &mut self,
        expr: &mut Expression,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.update_label(&mut expr.id);
        walk_mut_expr(self, expr)
    }
    fn visit_funcdef(
        &mut self,
        funcdef: &mut Function,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.update_label(&mut funcdef.id);
        walk_mut_funcdef(self, funcdef)
    }
    fn visit_init(
        &mut self,
        init: &mut SimpleInitialization,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.update_label(&mut init.id);
        walk_mut_unpacked_init(self, init)
    }
    fn visit_initblock(
        &mut self,
        init: &mut InitializationBlock,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.update_label(&mut init.id);
        walk_mut_initblock(self, init)
    }
    fn visit_module(
        &mut self,
        module: &mut Module,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.update_label(&mut module.id);
        walk_mut_module(self, module)
    }
    fn visit_reassignment(
        &mut self,
        reinit: &mut Reassignment,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.update_label(&mut reinit.id);
        walk_mut_reassignment(self, reinit)
    }
    fn visit_stmt(
        &mut self,
        stmt: &mut Statement,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.update_label(&mut stmt.id);
        walk_mut_stmt(self, stmt)
    }
    fn visit_structdef(
        &mut self,
        structdef: &mut StructDataTypeDefinition,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.update_label(&mut structdef.id);
        walk_mut_structdef(self, structdef)
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
    type TransformerError = ();
    type TransformerOk = ();
    fn visit_initblock(
        &mut self,
        init: &mut InitializationBlock,
    ) -> Result<Self::TransformerOk, Self::TransformerOk> {
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
    /// Also generates an expression referencing that initialization.
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

pub struct IdentifierScoper {
    /// map an Ident-expression to a scoped identifier and the nearest enclosing block.
    scope_stack: Vec<NodeId>,
    scope_annotations: ScopeAnnotations,
}

pub struct NotInScopeError {
    ident: String,
    scope_id: usize,
}

impl Transfomer for IdentifierScoper {
    type TransformerOk = ();
    type TransformerError = NotInScopeError;
    fn visit_module(
        &mut self,
        module: &mut Module,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        for glob_var in module.global_vars.iter_mut() {
            self.visit_initblock(glob_var)?;
        }

        for func in module.functions.iter_mut() {
            self.enter_scope(func.body.id);
            for stmt in func.body.statements.iter_mut() {
                self.visit_stmt(stmt)?;
            }
        }
        Ok(())
    }
    fn visit_initblock(
        &mut self,
        init: &mut InitializationBlock,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        let InitializationKind::Unpacked(u) = &mut init.kind else {
            unreachable!("assignments should be expanded")
        };
        for init in u.iter_mut() {
            self.visit_expr(&mut init.value)?;
        }
        Ok(())
    }
    fn visit_expr(
        &mut self,
        expr: &mut Expression,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
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
            ExpressionKind::Block(eb) => self.visit_block(eb)?,
        }
        Ok(())
    }
    fn visit_stmt(
        &mut self,
        stmt: &mut Statement,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        match &mut stmt.kind {
            StatementKind::Initialization(init) => self.visit_initblock(init),
            StatementKind::Reassignment(reinit) => self.visit_reassignment(reinit),
            StatementKind::FunctionCall(call) => self.visit_call(call),
            StatementKind::Return(e) => self.visit_expr(e),
            StatementKind::BlockTail(e) => self.visit_expr(e),
            StatementKind::Block(eb) => self.visit_block(eb),
            StatementKind::IfThenElse(ite) => self.visit_branch(ite),
        }
    }
    fn visit_block(
        &mut self,
        block: &mut BlockExpression,
    ) -> Result<Self::TransformerOk, Self::TransformerError> {
        self.enter_scope(block.id);
        for stmt in block.statements.iter_mut() {
            self.visit_stmt(stmt)?;
        }
        self.exit_scope();
        Ok(())
    }
}

impl IdentifierScoper {
    pub fn new(ast: &Module) -> Self {
        Self {
            scope_stack: vec![ast.id],
            scope_annotations: ScopeAnnotations::new(),
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
    fn search_for(&mut self, _ident: &str) -> Result<ScopedIdentifier, NotInScopeError> {
        todo!()
    }
}
