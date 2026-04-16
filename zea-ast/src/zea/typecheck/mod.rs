use crate::zea;
use crate::zea::visitors::annotating::ScopeAnnotations;
use crate::zea::{BinOp, ExpressionKind, Module, UnOp};
use indexmap::{IndexMap, IndexSet};

#[allow(non_snake_case)]
pub fn BUILTIN_TYPES() -> [zea::Type; 5] {
    [
        zea::Type::I64(),
        zea::Type::Bool(),
        zea::Type::F64(),
        zea::Type::Unit(),
        zea::Type::Never(),
    ]
}

pub enum TypeCheckError {
    UnifyError(TypeConcreteId, TypeConcreteId),
    ExpectedResolvedType(InferenceId),
    ExpectedIntroducedType(zea::Expression),
}
/// The id that a concrete type gets during type-checking
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct TypeConcreteId {
    id: usize,
}

/// The id that a type-variable gets during type-checking
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct TypeVarId {
    id: usize,
}

/// A table holding all unique types within a module.
pub struct TypeInterningTable {
    type_ids: IndexSet<zea::Type>,
}

impl TypeInterningTable {
    pub fn with_zea_types() -> Self {
        Self {
            type_ids: IndexSet::from_iter(BUILTIN_TYPES()),
        }
    }
    pub fn introduce(&mut self, typ: &zea::Type) -> TypeConcreteId {
        let (index, _existed) = self.type_ids.insert_full(typ.clone());
        TypeConcreteId { id: index }
    }
    pub fn get_interned_type(&self, id: TypeConcreteId) -> Option<&zea::Type> {
        self.type_ids.get_index(id.id)
    }

    pub fn interned_unit(&self) -> TypeConcreteId {
        TypeConcreteId {
            id: self.type_ids.get_index_of(&zea::Type::Unit()).unwrap(),
        }
    }
    pub fn interned_i64(&self) -> TypeConcreteId {
        TypeConcreteId {
            id: self.type_ids.get_index_of(&zea::Type::I64()).unwrap(),
        }
    }

    pub fn interned_f64(&self) -> TypeConcreteId {
        TypeConcreteId {
            id: self.type_ids.get_index_of(&zea::Type::F64()).unwrap(),
        }
    }
    pub fn interned_bool(&self) -> TypeConcreteId {
        TypeConcreteId {
            id: self.type_ids.get_index_of(&zea::Type::Bool()).unwrap(),
        }
    }

    pub fn contains_id(&self, id: TypeConcreteId) -> bool {
        self.type_ids.get_index(id.id).is_some()
    }

    pub fn contains_type(&self, typ: &zea::Type) -> bool {
        self.type_ids.get(typ).is_some()
    }
}
pub struct TypeVarSubstitutionTable {
    arena: Vec<TypeVarId>,
    known_types: IndexMap<TypeVarId, TypeConcreteId>,
}

impl TypeVarSubstitutionTable {
    pub fn new() -> Self {
        Self {
            arena: Vec::new(),
            known_types: IndexMap::new(),
        }
    }
    pub fn fresh(&mut self) -> TypeVarId {
        let len = self.arena.len();
        self.arena.push(TypeVarId { id: len });
        self.arena[len]
    }

    pub fn union(&mut self, var: TypeVarId, with: TypeVarId) {
        self.arena[var.id] = with;
    }

    pub fn find(&mut self, var: TypeVarId) -> TypeVarId {
        if self.arena[var.id] == var {
            var
        } else {
            // path compression
            let root = self.find(self.arena[var.id]);
            self.arena[var.id] = root;
            root
        }
    }

    pub fn find_uncompressed(&self, var: TypeVarId) -> TypeVarId {
        if self.arena[var.id] == var {
            var
        } else {
            // path compression
            self.find_uncompressed(self.arena[var.id])
        }
    }

    pub fn add_known_type(&mut self, var: TypeVarId, actual_type: TypeConcreteId) {
        self.known_types.insert(var, actual_type);
    }

    /// if the representative element of a var has a known type, return it,
    /// otherwise return the representative
    pub fn get_resolved_type(&mut self, var: TypeVarId) -> InferenceId {
        let var = self.find(var);
        match self.known_types.get(&var) {
            Some(concrete_id) => (*concrete_id).into(),
            None => var.into(),
        }
    }
}

impl From<TypeConcreteId> for InferenceId {
    fn from(value: TypeConcreteId) -> Self {
        Self::TypeConcrete(value)
    }
}

impl From<TypeVarId> for InferenceId {
    fn from(value: TypeVarId) -> Self {
        Self::TypeVar(value)
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub enum InferenceId {
    TypeConcrete(TypeConcreteId), // map some type-id to an actual type within an interning-table
    TypeVar(TypeVarId),
}

impl InferenceId {
    pub fn is_concrete(&self) -> bool {
        matches!(self, InferenceId::TypeConcrete(_))
    }
}
struct ModuleInferenceContext<'ast> {
    ast: &'ast Module,
    /// unique types within a program, use a [`TypeConcreteId`] to lookup
    intering_table: TypeInterningTable,
    /// type variables, use a [`TypeVarId`] to lookup,
    /// call [`ModuleInferenceContext::follow_inference_id`] to get the possibly known type
    subst_table: TypeVarSubstitutionTable,
    /// map a node id to its (possibly not-yet-known) type.)
    node_types: IndexMap<usize, InferenceId>,
    scopes: ScopeAnnotations,
}

impl<'ast> ModuleInferenceContext<'ast> {
    pub fn new(ast: &'ast Module) -> Self {
        Self {
            ast,
            intering_table: TypeInterningTable::with_zea_types(),
            subst_table: TypeVarSubstitutionTable::new(),
            node_types: IndexMap::new(),
            scopes: ast.annotate_scopes(),
        }
    }

    pub fn type_bool_inference_id(&self) -> InferenceId {
        self.intering_table.interned_bool().into()
    }

    pub fn introduce(&mut self, expr: &zea::Expression) {
        match expr.kind {
            ExpressionKind::Unit
            | ExpressionKind::IntegerLiteral(_)
            | ExpressionKind::BoolLiteral(_)
            | ExpressionKind::FloatLiteral(_) => self.introduce_trivial_type(expr),
            ExpressionKind::FuncCall(_)
            | ExpressionKind::BinOpExpr(_, _, _)
            | ExpressionKind::UnOpExpr(_, _)
            | ExpressionKind::MemberAccess(_, _)
            | ExpressionKind::CondBranch(_)
            | ExpressionKind::ExpandedBlock(_)
            | ExpressionKind::Ident(_) => self.introduce_non_trivial_type(expr),
            ExpressionKind::StringLiteral(_) => todo!("string types for Zea"),
            ExpressionKind::Block(_) => unreachable!("ast should have unexpanded blocks removed"),
        }
    }

    fn introduce_trivial_type(&mut self, expr: &zea::Expression) {
        let inference_id: InferenceId = match expr.kind {
            ExpressionKind::Unit => self.intering_table.interned_unit().into(),
            ExpressionKind::IntegerLiteral(_) => self.intering_table.interned_i64().into(),
            ExpressionKind::BoolLiteral(_) => self.intering_table.interned_bool().into(),
            ExpressionKind::FloatLiteral(_) => self.intering_table.interned_f64().into(),
            _ => unreachable!(
                "AST node with id {} is not a trivial expression: {:?}",
                expr.id, expr.kind
            ),
        };
        self.node_types.insert(expr.id, inference_id);
    }
    fn introduce_non_trivial_type(&mut self, expr: &zea::Expression) {
        match &expr.kind {
            ExpressionKind::Ident(_) => {
                let var = self.subst_table.fresh();
                self.node_types.insert(expr.id, var.into());
            }
            ExpressionKind::FuncCall(_) => {
                let var = self.subst_table.fresh();
                self.node_types.insert(expr.id, var.into());
            }
            ExpressionKind::BinOpExpr(_op, l, r) => {
                let var = self.subst_table.fresh();
                self.node_types.insert(expr.id, var.into());
                self.introduce(l.as_ref());
                self.introduce(r.as_ref());
            }
            ExpressionKind::UnOpExpr(_op, arg) => {
                let var = self.subst_table.fresh();
                self.node_types.insert(expr.id, var.into());
                self.introduce(arg.as_ref());
            }
            ExpressionKind::MemberAccess(datatype, _member) => {
                let var = self.subst_table.fresh();
                self.node_types.insert(expr.id, var.into());
                self.introduce(datatype.as_ref());
            }
            ExpressionKind::CondBranch(branch) => {
                let var = self.subst_table.fresh();
                self.node_types.insert(expr.id, var.into());

                self.introduce(branch.condition.as_ref());
                self.introduce(branch.true_case.as_ref());
                if let Some(ref false_case) = branch.false_case {
                    self.introduce(false_case.as_ref());
                }
            }
            ExpressionKind::ExpandedBlock(_b) => {
                let var = self.subst_table.fresh();
                self.node_types.insert(expr.id, var.into());
            }
            ExpressionKind::Block(_) => {
                unreachable!("AST should not contain any un-expanded blocks")
            }
            _ => unreachable!(
                "AST node with id {} is not a non-trivial expression: {:?}",
                expr.id, expr.kind
            ),
        };
    }

    pub fn get_inference_id(
        &self,
        node: &'ast zea::Expression,
    ) -> Result<InferenceId, TypeCheckError> {
        self.node_types
            .get(&node.id)
            .cloned()
            .ok_or(TypeCheckError::ExpectedIntroducedType(node.to_owned()))
    }

    pub fn follow_inference_id(&mut self, var: InferenceId) -> InferenceId {
        match var {
            InferenceId::TypeConcrete(_) => var,
            InferenceId::TypeVar(v) => self.subst_table.get_resolved_type(v),
        }
    }

    pub fn resolve_id_to_concrete(&mut self, inference_id: InferenceId) -> Option<TypeConcreteId> {
        let maybe_con_id = self.follow_inference_id(inference_id);
        match maybe_con_id {
            InferenceId::TypeConcrete(c) => Some(c),
            InferenceId::TypeVar(_) => None,
        }
    }

    pub fn try_unify(
        &mut self,
        expr: &zea::Expression,
        with: &zea::Expression,
    ) -> Result<(), TypeCheckError> {
        let expr = self.get_inference_id(expr)?;
        let with = self.get_inference_id(with)?;
        self.unify_ids(expr, with)
    }

    pub fn unify_ids(
        &mut self,
        expr: InferenceId,
        with: InferenceId,
    ) -> Result<(), TypeCheckError> {
        let expr = self.follow_inference_id(expr);
        let with = self.follow_inference_id(with);
        match (expr, with) {
            (InferenceId::TypeConcrete(a), InferenceId::TypeConcrete(b)) => {
                let ta = self.intering_table.get_interned_type(a).unwrap();
                let tb = self.intering_table.get_interned_type(b).unwrap();
                if self.equal_type(ta, tb) {
                    Ok(())
                } else {
                    Err(TypeCheckError::UnifyError(a, b))
                }
            }
            (InferenceId::TypeConcrete(_), InferenceId::TypeVar(_)) => self.unify_ids(with, expr),
            (InferenceId::TypeVar(v), InferenceId::TypeConcrete(c)) => {
                self.subst_table.known_types.insert(v, c);
                Ok(())
            }
            (InferenceId::TypeVar(v1), InferenceId::TypeVar(v2)) => {
                self.subst_table.union(v1, v2);
                Ok(())
            }
        }
    }

    pub fn equal_type(&self, t1: &zea::Type, t2: &zea::Type) -> bool {
        match (t1, t2) {
            (zea::Type::Basic(s1), zea::Type::Basic(s2)) => s1 == s2,
            (zea::Type::ArrayOf(t1), zea::Type::ArrayOf(t2)) => self.equal_type(t1, t2),
            (zea::Type::Pointer(t1), zea::Type::Pointer(t2)) => self.equal_type(t1, t2),
            _ => false,
        }
    }
    pub fn valid_subtype(&self, t1: &zea::Type, t2: &zea::Type) -> bool {
        match (t1, t2) {
            (zea::Type::Basic(s1), zea::Type::Basic(s2)) => self.valid_basic_subtype(s1, s2),
            (zea::Type::ArrayOf(t1), zea::Type::ArrayOf(t2)) => self.equal_type(t1, t2),
            (zea::Type::Pointer(t1), zea::Type::Pointer(t2)) => self.equal_type(t1, t2),
            _ => false,
        }
    }

    pub fn valid_basic_subtype(&self, t1: &str, t2: &str) -> bool {
        match (t1, t2) {
            (t1, t2) if t1 == t2 => true,
            ("I32", "I64") => true,
            ("I16", "I64") => true,
            ("I8", "I64") => true,
            ("Bool", "I64") => true,
            ("I16", "I32") => true,
            ("I8", "I32") => true,
            ("Bool", "I32") => true,
            ("I8", "I16") => true,
            ("Bool", "I16") => true,
            ("Bool", "I8") => true,
            ("F32", "F64") => true,
            _ => false,
        }
    }

    pub fn infer_block(&mut self, block: &zea::Expression) -> Result<InferenceId, TypeCheckError> {
        let ExpressionKind::ExpandedBlock(_block) = &block.kind else {
            panic!("infer_block expects block, got {:?}", block.kind)
        };
        todo!()
    }

    pub fn infer_branch(&mut self, expr: &zea::Expression) -> Result<InferenceId, TypeCheckError> {
        let ExpressionKind::CondBranch(branch) = &expr.kind else {
            panic!("infer_branch expects branch, got {:?}", expr.kind)
        };

        let cond_id = self.get_inference_id(branch.condition.as_ref())?;
        self.unify_ids(cond_id, self.type_bool_inference_id())?;

        let true_id = self.get_inference_id(branch.true_case.as_ref())?;

        if let Some(false_case) = &branch.false_case {
            let false_id = self.get_inference_id(false_case.as_ref())?;
            self.unify_ids(false_id, true_id)?;
        };

        let expr_id = self.get_inference_id(expr)?;
        self.unify_ids(expr_id, true_id)?;
        self.infer_expr(branch.true_case.as_ref())
    }

    pub fn infer_binop(&mut self, expr: &zea::Expression) -> Result<InferenceId, TypeCheckError> {
        let ExpressionKind::BinOpExpr(op, _lhs, _rhs) = &expr.kind else {
            panic!("infer_binop expects binop, got {:?}", expr.kind)
        };
        match op {
            BinOp::Add
            | BinOp::Sub
            | BinOp::Mul
            | BinOp::Mod
            | BinOp::Div
            | BinOp::BitAnd
            | BinOp::BitOr
            | BinOp::BitXor
            | BinOp::Lsh
            | BinOp::Rsh => todo!(),
            _ => unreachable!(),
        }
    }

    pub fn infer_unop(&mut self, expr: &zea::Expression) -> Result<InferenceId, TypeCheckError> {
        let ExpressionKind::UnOpExpr(op, _arg) = &expr.kind else {
            panic!("infer_unop expects unop, got {:?}", expr.kind)
        };
        match op {
            UnOp::Neg => todo!(),
            UnOp::LogNot => todo!(),
            UnOp::BitNot => todo!(),
        }
    }

    pub fn infer_func_call(
        &mut self,
        expr: &zea::Expression,
    ) -> Result<InferenceId, TypeCheckError> {
        let ExpressionKind::FuncCall(call) = &expr.kind else {
            panic!("infer_branch expects branch, got {:?}", expr.kind)
        };

        let func = self
            .ast
            .iter_symbols()
            .find(|func| func.name == call.name)
            .expect("AST should have ");
        let returns_id: InferenceId = self.intering_table.introduce(&func.returns).into();
        let expr_inference_id = self.get_inference_id(expr)?;
        self.unify_ids(returns_id, expr_inference_id)?;

        let param_types: Vec<&zea::Type> = func.args.iter().map(|ti| &ti.typ).collect();

        for (arg, typ) in call.args.iter().zip(param_types) {
            let param_con_id: InferenceId = self.intering_table.introduce(typ).into();
            let arg_infer_id = self.infer_expr(arg)?;
            self.unify_ids(param_con_id, arg_infer_id)?;
        }
        Ok(returns_id)
    }

    pub fn infer_ident(&mut self, _ident: &zea::Expression) -> Result<InferenceId, TypeCheckError> {
        todo!()
    }

    pub fn infer_member_access(
        &mut self,
        member_access: &zea::Expression,
    ) -> Result<InferenceId, TypeCheckError> {
        let ExpressionKind::MemberAccess(datatype, _member) = &member_access.kind else {
            panic!(
                "infer_member_access expects member_access, got {:?}",
                member_access.kind
            )
        };
        let datatype = datatype.as_ref();
        let datatype_infer_id = self.infer_expr(datatype)?;
        let _datatype_con_id = self
            .resolve_id_to_concrete(datatype_infer_id)
            .ok_or(TypeCheckError::ExpectedResolvedType(datatype_infer_id))?;

        todo!("user defined datatypes in Zea")
    }

    pub fn infer_expr(&mut self, expr: &zea::Expression) -> Result<InferenceId, TypeCheckError> {
        let expr_inference_id = self.get_inference_id(expr)?;
        if let Some(con_id) = self.resolve_id_to_concrete(expr_inference_id) {
            let con_id: InferenceId = con_id.into();
            self.node_types.insert(expr.id, con_id);
            Ok(con_id)
        } else {
            match &expr.kind {
                ExpressionKind::CondBranch(_) => self.infer_branch(expr),
                ExpressionKind::ExpandedBlock(_) => self.infer_block(expr),
                ExpressionKind::Ident(_) => self.infer_ident(expr),
                ExpressionKind::FuncCall(_) => self.infer_func_call(expr),
                ExpressionKind::BinOpExpr(_, _, _) => self.infer_binop(expr),
                ExpressionKind::UnOpExpr(_, _) => self.infer_unop(expr),
                ExpressionKind::MemberAccess(_, _) => self.infer_member_access(expr),

                ExpressionKind::Block(_) => {
                    unreachable!("AST should not contain un-expanded blocks")
                }
                ExpressionKind::StringLiteral(_) => todo!("string types in Zea"),
                // ExpressionKind::Unit => {}
                // ExpressionKind::IntegerLiteral(_) => {}
                // ExpressionKind::BoolLiteral(_) => {}
                // ExpressionKind::FloatLiteral(_) => {}
                _ => unreachable!(),
            }
        }
    }
}
