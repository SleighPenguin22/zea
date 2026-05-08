use crate::helper_impls::StructuralEq;
use crate::visualisation::IndentPrint;
use crate::zea;
use crate::zea::typecheck::TypeCheckError::{MissingInternedTypeId, UnifyError};
use crate::zea::visitors::annotating::ScopeAnnotations;
use crate::zea::{
    BinOp, ExpressionKind, Function, IfThenElse, Initialization, InitializationKind, Module,
    SimpleInitialization, StructDataTypeDefinition, TypeSpecifier, UnOp,
};
use crate::zea::{FunctionCall, Hasher};
use crate::zea::{Hash, StatementKind};
use indexmap::{IndexMap, IndexSet};
use zea_macros::{ASTStructuralEq, HashEqById};

pub struct TupleSignature {
    members: Vec<zea::TypeSpecifier>,
}

#[derive(Debug, Clone, HashEqById, ASTStructuralEq)]
pub struct HoistedDeclaration {
    pub id: usize,
    pub typ: zea::TypeSpecifier,
    pub assignee: String,
}

#[allow(non_snake_case)]
pub fn BUILTIN_SCALAR_TYPES() -> [zea::TypeSpecifier; 13] {
    [
        zea::TypeSpecifier::t_Bool(),
        zea::TypeSpecifier::t_I8(),
        zea::TypeSpecifier::t_I16(),
        zea::TypeSpecifier::t_I32(),
        zea::TypeSpecifier::t_I64(),
        zea::TypeSpecifier::t_U8(),
        zea::TypeSpecifier::t_U16(),
        zea::TypeSpecifier::t_U32(),
        zea::TypeSpecifier::t_U64(),
        zea::TypeSpecifier::t_F32(),
        zea::TypeSpecifier::t_F64(),
        zea::TypeSpecifier::t_Unit(),
        zea::TypeSpecifier::t_Never(),
    ]
}

pub enum TypeCheckError {
    UnifyError(TypeConcreteId, TypeConcreteId),
    ExpectedResolvedType(InferenceId),
    ExpectedIntroducedType(zea::Expression),
    MissingInternedTypeId(TypeConcreteId),
    MissingInternedTypeSpec(TypeSpecifier),
    ExpectedArrayType(TypeConcreteId),
    ExpectedIntegerType(TypeConcreteId),
    ExpectedStructType(TypeConcreteId),
    ExpectedBoolType(TypeConcreteId),
    MissingStructDefinition(String),
    MissingStructField(TypeConcreteId, String),
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
    type_ids: IndexSet<zea::TypeSpecifier>,
}

impl TypeInterningTable {
    pub fn new_builtin_zea_types() -> Self {
        Self {
            type_ids: IndexSet::from_iter(BUILTIN_SCALAR_TYPES()),
        }
    }
    pub fn introduce(&mut self, typ: &zea::TypeSpecifier) -> TypeConcreteId {
        let (index, _existed) = self.type_ids.insert_full(typ.clone());
        TypeConcreteId { id: index }
    }

    pub fn lookup_type_specifier(
        &self,
        typ: &zea::TypeSpecifier,
    ) -> Result<TypeConcreteId, TypeCheckError> {
        self.type_ids
            .get_index_of(typ)
            .map(|index| TypeConcreteId { id: index })
            .ok_or(TypeCheckError::MissingInternedTypeSpec(typ.clone()))
    }

    pub fn lookup_by_id(&self, id: TypeConcreteId) -> Result<&TypeSpecifier, TypeCheckError> {
        self.type_ids
            .get_index(id.id)
            .ok_or(MissingInternedTypeId(id))
    }
    pub fn get_interned_int_literal_type(&self, literal: usize) -> TypeConcreteId {
        let t = zea::TypeSpecifier::t_ILit_from(literal);
        TypeConcreteId {
            id: self.type_ids.get_index_of(&t).unwrap(),
        }
    }

    pub fn interned_unit(&self) -> TypeConcreteId {
        TypeConcreteId {
            id: self
                .type_ids
                .get_index_of(&TypeSpecifier::t_Unit())
                .unwrap(),
        }
    }
    pub fn interned_i8(&self) -> TypeConcreteId {
        TypeConcreteId {
            id: self
                .type_ids
                .get_index_of(&zea::TypeSpecifier::t_I8())
                .unwrap(),
        }
    }
    pub fn interned_i16(&self) -> TypeConcreteId {
        TypeConcreteId {
            id: self
                .type_ids
                .get_index_of(&zea::TypeSpecifier::t_I16())
                .unwrap(),
        }
    }
    pub fn interned_i32(&self) -> TypeConcreteId {
        TypeConcreteId {
            id: self
                .type_ids
                .get_index_of(&zea::TypeSpecifier::t_I32())
                .unwrap(),
        }
    }
    pub fn interned_i64(&self) -> TypeConcreteId {
        TypeConcreteId {
            id: self
                .type_ids
                .get_index_of(&zea::TypeSpecifier::t_I64())
                .unwrap(),
        }
    }
    pub fn interned_u8(&self) -> TypeConcreteId {
        TypeConcreteId {
            id: self
                .type_ids
                .get_index_of(&zea::TypeSpecifier::t_U8())
                .unwrap(),
        }
    }
    pub fn interned_u16(&self) -> TypeConcreteId {
        TypeConcreteId {
            id: self
                .type_ids
                .get_index_of(&zea::TypeSpecifier::t_U16())
                .unwrap(),
        }
    }
    pub fn interned_u32(&self) -> TypeConcreteId {
        TypeConcreteId {
            id: self
                .type_ids
                .get_index_of(&zea::TypeSpecifier::t_U32())
                .unwrap(),
        }
    }
    pub fn interned_u64(&self) -> TypeConcreteId {
        TypeConcreteId {
            id: self
                .type_ids
                .get_index_of(&zea::TypeSpecifier::t_U64())
                .unwrap(),
        }
    }

    pub fn interned_f32(&self) -> TypeConcreteId {
        TypeConcreteId {
            id: self
                .type_ids
                .get_index_of(&zea::TypeSpecifier::t_F32())
                .unwrap(),
        }
    }
    pub fn interned_f64(&self) -> TypeConcreteId {
        TypeConcreteId {
            id: self
                .type_ids
                .get_index_of(&zea::TypeSpecifier::t_F64())
                .unwrap(),
        }
    }
    pub fn interned_bool(&self) -> TypeConcreteId {
        TypeConcreteId {
            id: self
                .type_ids
                .get_index_of(&zea::TypeSpecifier::t_Bool())
                .unwrap(),
        }
    }

    pub fn contains_id(&self, id: TypeConcreteId) -> bool {
        self.type_ids.get_index(id.id).is_some()
    }

    pub fn contains_type(&self, typ: &zea::TypeSpecifier) -> bool {
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
pub struct ModuleInferenceContext<'ast> {
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
            intering_table: TypeInterningTable::new_builtin_zea_types(),
            subst_table: TypeVarSubstitutionTable::new(),
            node_types: IndexMap::new(),
            scopes: ast.annotate_scopes(),
        }
    }

    pub fn typespecifier_behind_inference_id(
        &mut self,
        id: InferenceId,
    ) -> Result<(TypeSpecifier, TypeConcreteId), TypeCheckError> {
        let id = self.follow_inference_id(id);
        let concrete_id = self
            .resolve_id_to_concrete(id)
            .ok_or(TypeCheckError::ExpectedResolvedType(id))?;
        let concrete = self.intering_table.lookup_by_id(concrete_id).cloned()?;
        Ok((concrete, concrete_id))
    }

    pub fn type_bool_inference_id(&self) -> InferenceId {
        self.intering_table.interned_bool().into()
    }
    pub fn type_unit_inference_id(&self) -> InferenceId {
        self.intering_table.interned_unit().into()
    }
    pub fn type_u64_inference_id(&self) -> InferenceId {
        self.intering_table.interned_u64().into()
    }
    pub fn type_int_inference_id(
        &self,
        width: usize,
        signed: bool,
    ) -> Result<InferenceId, TypeCheckError> {
        self.intering_table
            .lookup_type_specifier(&TypeSpecifier::Integer { width, signed })
            .map(|conc_id| InferenceId::TypeConcrete(conc_id))
    }
    pub fn type_float_inference_id(&self, width: usize) -> Result<InferenceId, TypeCheckError> {
        self.intering_table
            .lookup_type_specifier(&TypeSpecifier::Float { width })
            .map(|conc_id| InferenceId::TypeConcrete(conc_id))
    }

    pub fn introduce_visit_module(&mut self, module: &Module) {
        for func in module.functions.iter() {
            self.introduce_visit_function(func);
        }
        for global_var in module.global_vars.iter() {
            self.introduce_visit_initialisation(global_var);
        }
    }
    pub fn introduce_visit_function(&mut self, func: &Function) {
        for stmt in func.body.statements.iter() {
            self.introduce_visit_statement(stmt);
        }
    }
    pub fn introduce_visit_funccall(&mut self, call: &FunctionCall) {
        for arg in call.args.iter() {
            self.introduce_visit_expression(arg);
        }
    }
    pub fn introduce_visit_branch(&mut self, branch: &IfThenElse) {
        self.introduce_visit_expression(branch.condition.as_ref());
        self.introduce_visit_expression(branch.true_case.as_ref());
        if let Some(ref false_case) = branch.false_case {
            self.introduce_visit_expression(false_case.as_ref());
        }
    }
    pub fn introduce_visit_expression(&mut self, expr: &zea::Expression) {
        match expr.kind {
            ExpressionKind::Unit
            | ExpressionKind::IntegerLiteral(_)
            | ExpressionKind::BoolLiteral(_)
            | ExpressionKind::FloatLiteral(_) => self.introduce_visit_expression_trivial_type(expr),
            ExpressionKind::FunctionCall(_)
            | ExpressionKind::BinOpExpr(_, _, _)
            | ExpressionKind::UnOpExpr(_, _)
            | ExpressionKind::MemberAccess(_, _)
            | ExpressionKind::ScopedIdent(_)
            | ExpressionKind::IfThenElse(_)
            | ExpressionKind::ExpandedBlock(_)
            | ExpressionKind::UnScopedIdent(_) => {
                self.introduce_visit_expression_non_trivial_type(expr)
            }
            ExpressionKind::StringLiteral(_) => todo!("string types for Zea"),
            ExpressionKind::Block(_) => unreachable!("ast should have unexpanded blocks removed"),
        }
    }

    fn introduce_visit_initialisation(&mut self, init: &zea::Initialization) {
        let zea::InitializationKind::Unpacked(unpacked) = &init.kind else {
            unreachable!()
        };
        for init in unpacked.iter() {
            self.introduce_visit_expression(&init.value);
        }
    }
    fn introduce_visit_block(&mut self, block: &zea::ExpandedBlockExpr) {
        for stmt in block.statements.iter() {
            self.introduce_visit_statement(stmt);
        }
        self.introduce_visit_expression(&block.last);
    }

    fn introduce_visit_statement(&mut self, stmt: &zea::Statement) {
        match &stmt.kind {
            StatementKind::Return(e) => self.introduce_visit_expression(e),
            StatementKind::Initialization(init) => self.introduce_visit_initialisation(init),
            StatementKind::Reassignment(r) => self.introduce_visit_expression(&r.value),
            StatementKind::FunctionCall(call) => self.introduce_visit_funccall(call),
            StatementKind::BlockTail(e) => self.introduce_visit_expression(e),
            StatementKind::ExpandedBlock(b) => {
                self.introduce_visit_block(b);
            }
            StatementKind::IfThenElse(branch) => {
                self.introduce_visit_branch(branch);
            }
            StatementKind::Block(_) => unreachable!(),
        }
    }

    fn introduce_visit_expression_trivial_type(&mut self, expr: &zea::Expression) {
        let inference_id: InferenceId = match expr.kind {
            ExpressionKind::Unit => self.intering_table.interned_unit().into(),
            ExpressionKind::IntegerLiteral(i) => {
                self.intering_table.get_interned_int_literal_type(i).into()
            }
            ExpressionKind::BoolLiteral(_) => self.intering_table.interned_bool().into(),
            ExpressionKind::FloatLiteral(_) => self.intering_table.interned_f64().into(),
            _ => unreachable!(
                "AST node with id {} is not a trivial expression: {:?}",
                expr.id, expr.kind
            ),
        };
        self.node_types.insert(expr.id, inference_id);
    }
    fn introduce_visit_expression_non_trivial_type(&mut self, expr: &zea::Expression) {
        match &expr.kind {
            ExpressionKind::ScopedIdent(_) => {
                let var = self.subst_table.fresh();
                self.node_types.insert(expr.id, var.into());
            }
            ExpressionKind::FunctionCall(call) => {
                let var = self.subst_table.fresh();
                self.node_types.insert(expr.id, var.into());
                self.introduce_visit_funccall(call);
            }
            ExpressionKind::BinOpExpr(_op, l, r) => {
                let var = self.subst_table.fresh();
                self.node_types.insert(expr.id, var.into());
                self.introduce_visit_expression(l.as_ref());
                self.introduce_visit_expression(r.as_ref());
            }
            ExpressionKind::UnOpExpr(_op, arg) => {
                let var = self.subst_table.fresh();
                self.node_types.insert(expr.id, var.into());
                self.introduce_visit_expression(arg.as_ref());
            }
            ExpressionKind::MemberAccess(datatype, _member) => {
                let var = self.subst_table.fresh();
                self.node_types.insert(expr.id, var.into());
                self.introduce_visit_expression(datatype.as_ref());
            }
            ExpressionKind::IfThenElse(branch) => {
                let var = self.subst_table.fresh();
                self.node_types.insert(expr.id, var.into());

                self.introduce_visit_branch(branch);
            }
            ExpressionKind::ExpandedBlock(block) => {
                let var = self.subst_table.fresh();
                self.node_types.insert(expr.id, var.into());
                self.introduce_visit_block(block);
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

    pub fn get_resolved_type(
        &mut self,
        inference_id: InferenceId,
    ) -> Result<&TypeSpecifier, TypeCheckError> {
        let maybe_con_id = self.follow_inference_id(inference_id);
        match maybe_con_id {
            InferenceId::TypeConcrete(c) => self.intering_table.lookup_by_id(c),

            InferenceId::TypeVar(_) => Err(TypeCheckError::ExpectedResolvedType(maybe_con_id)),
        }
    }

    pub fn try_unify_coerce(
        &mut self,
        expr: &zea::Expression,
        with: &zea::Expression,
    ) -> Result<(), TypeCheckError> {
        let expr: InferenceId = self.get_inference_id(expr)?;
        let with = self.get_inference_id(with)?;
        self.unify_coerce_ids(expr, with)
    }

    /// Tell the inference context that `expr` should coerce to the type that `with` has.
    ///
    /// Implementation is loosely based on Hindley Milner.
    pub fn unify_coerce_ids(
        &mut self,
        expr: impl Into<InferenceId>,
        with: impl Into<InferenceId>,
    ) -> Result<(), TypeCheckError> {
        let expr = self.follow_inference_id(expr.into());
        let with = self.follow_inference_id(with.into());
        match (expr, with) {
            (InferenceId::TypeConcrete(a), InferenceId::TypeConcrete(b)) => {
                if self.try_coerce_concrete_types(a, b)?.is_some() {
                    Ok(())
                } else {
                    Err(UnifyError(a, b))
                }
            }
            (InferenceId::TypeConcrete(c), InferenceId::TypeVar(v))
            | (InferenceId::TypeVar(v), InferenceId::TypeConcrete(c)) => {
                self.subst_table.add_known_type(v, c);
                Ok(())
            }
            (InferenceId::TypeVar(v1), InferenceId::TypeVar(v2)) => {
                self.subst_table.union(v1, v2);
                Ok(())
            }
        }
    }
    /// Tell the inference context that `expr` has the same type that `with` has.
    ///
    /// Implementation is loosely based on Hindley Milner.
    pub fn unify_equify_ids(
        &mut self,
        expr: impl Into<InferenceId>,
        with: impl Into<InferenceId>,
    ) -> Result<(), TypeCheckError> {
        let expr = self.follow_inference_id(expr.into());
        let with = self.follow_inference_id(with.into());
        match (expr, with) {
            (InferenceId::TypeConcrete(a), InferenceId::TypeConcrete(b)) => {
                if a == b {
                    Ok(())
                } else {
                    Err(UnifyError(a, b))
                }
            }
            (InferenceId::TypeConcrete(c), InferenceId::TypeVar(v))
            | (InferenceId::TypeVar(v), InferenceId::TypeConcrete(c)) => {
                self.subst_table.add_known_type(v, c);
                Ok(())
            }
            (InferenceId::TypeVar(v1), InferenceId::TypeVar(v2)) => {
                self.subst_table.union(v1, v2);
                Ok(())
            }
        }
    }

    fn require_integer_type(
        &mut self,
        id: InferenceId,
        with_minimum_width: Option<usize>,
        with_sign: Option<bool>,
    ) -> Result<TypeSpecifier, TypeCheckError> {
        let (typ, id) = self.typespecifier_behind_inference_id(id)?;
        match typ {
            TypeSpecifier::Integer {
                width: w,
                signed: s,
            } if with_minimum_width.is_none_or(|width| width > w)
                && with_sign.is_none_or(|signed| signed == s) =>
            {
                Ok(typ)
            }
            _ => Err(TypeCheckError::ExpectedArrayType(id)),
        }
    }
    fn require_array_type(&mut self, id: InferenceId) -> Result<TypeSpecifier, TypeCheckError> {
        let (typ, id) = self.typespecifier_behind_inference_id(id)?;
        match typ {
            TypeSpecifier::ArrayOf(t) => Ok(*t),
            _ => Err(TypeCheckError::ExpectedArrayType(id)),
        }
    }

    fn require_struct_type(
        &mut self,
        id: InferenceId,
    ) -> Result<StructDataTypeDefinition, TypeCheckError> {
        let (s, bob) = self.typespecifier_behind_inference_id(id)?;
        match s {
            TypeSpecifier::Basic(t) => self.find_struct_def(&t).cloned(),
            _ => Err(TypeCheckError::ExpectedStructType(bob)),
        }
    }
    fn require_bool_type(&mut self, id: InferenceId) -> Result<TypeSpecifier, TypeCheckError> {
        let (s, bob) = self.typespecifier_behind_inference_id(id)?;
        match s {
            t @ TypeSpecifier::Bool => Ok(t),
            _ => Err(TypeCheckError::ExpectedBoolType(bob)),
        }
    }

    /// (used for implicit casts) try to coerce one type into another.
    ///
    /// Returns Err(..) when some internal operation could not succeed.
    /// Returns Ok(None) if the types cannot be coerced,
    /// returns Ok(Some(t)) if the types can be coerced, where t is the resulting type-id.
    ///
    /// This operation is not communitative, i.e.
    /// ```ignore
    /// let t1 = ...;
    /// let t2 = ...;
    /// if try_coerce_concrete_types(t1, t2)?.is_some() { // if this is true,
    ///     try_coerce_concrete_types(t1, t2); // there is no guarantee that this will be true;
    /// }
    /// ```
    pub fn try_coerce_concrete_types(
        &self,
        typ: TypeConcreteId,
        to: TypeConcreteId,
    ) -> Result<Option<TypeConcreteId>, TypeCheckError> {
        let t_typ = self.intering_table.lookup_by_id(typ)?;
        let t_to = self.intering_table.lookup_by_id(to)?;
        if t_typ == t_to {
            return Ok(Some(to));
        }
        match (t_typ, t_to) {
            (
                zea::TypeSpecifier::Integer {
                    width: width1,
                    signed: signed1,
                },
                zea::TypeSpecifier::Integer {
                    width: width2,
                    signed: signed2,
                },
            ) => {
                if *signed1 && !signed2 {
                    // signed types may not be coerced to unsigned types
                    Ok(None)
                } else if !signed1 && *signed2 && width1 < width2 {
                    // unsigned types may only be cast if they are guaranteed to fit
                    Ok(Some(to))
                } else {
                    if width1 <= width2 {
                        Ok(Some(to))
                    } else {
                        Ok(None)
                    }
                }
            }
            (
                zea::TypeSpecifier::Float { width: width1 },
                zea::TypeSpecifier::Float { width: width2 },
            ) => {
                if width1 <= width2 {
                    Ok(Some(to))
                } else {
                    Ok(None)
                }
            }
            (zea::TypeSpecifier::Bool, zea::TypeSpecifier::Bool) => Ok(Some(to)),
            (zea::TypeSpecifier::Bool, zea::TypeSpecifier::Integer { .. }) => Ok(Some(to)),
            (zea::TypeSpecifier::Bool, zea::TypeSpecifier::Float { .. }) => Ok(Some(to)),
            _ => Ok(None),
        }
    }

    pub fn infer_block(&mut self, block: &zea::Expression) -> Result<InferenceId, TypeCheckError> {
        let id = self.get_inference_id(block)?;
        let ExpressionKind::ExpandedBlock(block) = &block.kind else {
            panic!("infer_block expects block, got {:?}", block.kind)
        };
        let t = self.infer_expr(&block.last)?;
        self.unify_coerce_ids(id, t)?;

        Ok(self.follow_inference_id(id))
    }

    pub fn infer_branch(&mut self, expr: &zea::Expression) -> Result<InferenceId, TypeCheckError> {
        let ExpressionKind::IfThenElse(branch) = &expr.kind else {
            panic!("infer_branch expects branch, got {:?}", expr.kind)
        };

        let cond_id = self.infer_expr(branch.condition.as_ref())?;
        self.unify_coerce_ids(cond_id, self.type_bool_inference_id())?;

        let true_id = self.infer_expr(branch.true_case.as_ref())?;

        if let Some(false_case) = &branch.false_case {
            let false_id = self.infer_expr(false_case.as_ref())?;
            self.unify_coerce_ids(true_id, false_id)?;
        } else {
            self.unify_coerce_ids(true_id, self.type_unit_inference_id())?;
        }

        let expr_id = self.get_inference_id(expr)?;
        self.unify_coerce_ids(expr_id, true_id)?;
        Ok(self.follow_inference_id(expr_id))
    }

    pub fn infer_binop(&mut self, expr: &zea::Expression) -> Result<InferenceId, TypeCheckError> {
        let expr_id = self.get_inference_id(expr)?;
        let ExpressionKind::BinOpExpr(op, lhs, rhs) = &expr.kind else {
            panic!("infer_binop expects binop, got {:?}", expr.kind)
        };
        let t_lhs = self.infer_expr(lhs)?;
        let t_rhs = self.infer_expr(rhs)?;
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
            | BinOp::Rsh
            | BinOp::Eq
            | BinOp::Neq
            | BinOp::Geq
            | BinOp::GT
            | BinOp::Leq
            | BinOp::LT => {
                self.unify_coerce_ids(t_lhs, t_rhs)?;

                self.unify_coerce_ids(expr_id, t_lhs)?;
            }

            BinOp::LogAnd | BinOp::LogOr | BinOp::LogXor => {
                self.unify_coerce_ids(t_lhs, self.type_bool_inference_id())?;
                self.unify_coerce_ids(t_rhs, self.type_bool_inference_id())?;
                self.unify_coerce_ids(expr_id, self.type_bool_inference_id())?;
            }
            BinOp::Subscript => {
                self.unify_coerce_ids(t_rhs, self.type_u64_inference_id())?;
                let t_inner = self.require_array_type(t_lhs)?;
                let t_inner_id = self.intering_table.introduce(&t_inner);
                self.unify_equify_ids(expr_id, t_inner_id)?;
            }
        }
        Ok(expr_id)
    }

    pub fn infer_unop(&mut self, expr: &zea::Expression) -> Result<InferenceId, TypeCheckError> {
        let ExpressionKind::UnOpExpr(op, arg) = &expr.kind else {
            panic!("infer_unop expects unop, got {:?}", expr.kind)
        };
        let expr_id = self.get_inference_id(expr)?;
        let arg_id = self.infer_expr(arg)?;

        match op {
            UnOp::Neg => self.infer_unop_neg(arg_id, expr_id),
            UnOp::LogNot => self.infer_unop_log_not(arg_id),
            UnOp::BitNot => todo!(),
        }
    }
    pub fn infer_unop_log_not(
        &mut self,
        arg_id: InferenceId,
    ) -> Result<InferenceId, TypeCheckError> {
        let bool_type = self.type_bool_inference_id();
        self.require_bool_type(arg_id)?;

        self.unify_coerce_ids(arg_id, bool_type)?;
        Ok(bool_type)
    }
    pub fn infer_unop_neg(
        &mut self,
        arg_id: InferenceId,
        expr_id: InferenceId,
    ) -> Result<InferenceId, TypeCheckError> {
        let typ = self.get_resolved_type(arg_id)?;
        if let TypeSpecifier::Integer {
            width: _w,
            signed: true,
        } = typ
        {
            self.unify_coerce_ids(expr_id, arg_id)?;
        } else if let TypeSpecifier::Integer {
            width: w,
            signed: false,
        } = typ
        {
            let resulting_width = (2 * w).min(64);
            let t_res = self.type_int_inference_id(resulting_width, false)?;
            self.unify_coerce_ids(expr_id, t_res)?;
        } else if let TypeSpecifier::Float { width: _w } = typ {
            self.unify_coerce_ids(expr_id, arg_id)?;
        }
        Ok(expr_id)
    }

    pub fn infer_unop_bit_not(
        &mut self,
        arg_id: InferenceId,
        expr_id: InferenceId,
    ) -> Result<InferenceId, TypeCheckError> {
        // bitwise operations are legal on any integer type
        self.require_integer_type(arg_id, None, None)?;
        self.unify_coerce_ids(arg_id, expr_id)?;
        Ok(expr_id)
    }

    pub fn infer_func_call(
        &mut self,
        expr: &zea::Expression,
    ) -> Result<InferenceId, TypeCheckError> {
        let ExpressionKind::FunctionCall(_call) = &expr.kind else {
            panic!("infer_func_call expects branch, got {:?}", expr.kind)
        };
        todo!()
    }
    pub fn infer_func_call_named(
        &mut self,
        _expr_id: InferenceId,
        call: &FunctionCall,
    ) -> Result<InferenceId, TypeCheckError> {
        let ExpressionKind::ScopedIdent(_s) = &call.subject.kind else {
            unreachable!()
        };
        todo!()
    }

    pub fn infer_ident(&mut self, ident: &zea::Expression) -> Result<InferenceId, TypeCheckError> {
        let ExpressionKind::ScopedIdent(_s) = &ident.kind else {
            unreachable!("infer_ident expects a scoped identifier expression")
        };
        todo!()
    }

    pub fn infer_member_access(
        &mut self,
        member_access: &zea::Expression,
    ) -> Result<InferenceId, TypeCheckError> {
        let ExpressionKind::MemberAccess(datatype, member) = &member_access.kind else {
            panic!(
                "infer_member_access expects member_access, got {:?}",
                member_access.kind
            )
        };
        let expr_id = self.get_inference_id(member_access)?;
        let datatype_infer_id = self.infer_expr(datatype)?;
        let s = self.require_struct_type(datatype_infer_id)?;

        let field = self.get_struct_member_type(&s, member)?;
        let field_id = self.intering_table.introduce(&field).into();
        self.unify_coerce_ids(expr_id, field_id)?;
        Ok(field_id)
    }
    fn find_struct_def(&self, name: &str) -> Result<&StructDataTypeDefinition, TypeCheckError> {
        self.ast
            .struct_definitions
            .iter()
            .find(|s| s.name == name)
            .ok_or(TypeCheckError::MissingStructDefinition(name.to_string()))
    }
    fn get_struct_member_type(
        &mut self,
        struct_def: &StructDataTypeDefinition,
        field: &str,
    ) -> Result<TypeSpecifier, TypeCheckError> {
        let struct_spec = TypeSpecifier::from(struct_def.name.as_str());
        let struct_id = self.intering_table.lookup_type_specifier(&struct_spec)?;
        struct_def
            .members
            .iter()
            .find_map(|member| (member.name == field).then_some(member.typ.clone()))
            .ok_or(TypeCheckError::MissingStructField(
                struct_id,
                field.to_string(),
            ))
    }

    pub fn infer_expr(&mut self, expr: &zea::Expression) -> Result<InferenceId, TypeCheckError> {
        match &expr.kind {
            ExpressionKind::IfThenElse(_) => self.infer_branch(expr),
            ExpressionKind::ExpandedBlock(_) => self.infer_block(expr),
            ExpressionKind::UnScopedIdent(_) => self.infer_ident(expr),
            ExpressionKind::FunctionCall(_) => self.infer_func_call(expr),
            ExpressionKind::BinOpExpr(_, _, _) => self.infer_binop(expr),
            ExpressionKind::UnOpExpr(_, _) => self.infer_unop(expr),
            ExpressionKind::MemberAccess(_, _) => self.infer_member_access(expr),
            // ExpressionKind::Unit => {}
            // ExpressionKind::IntegerLiteral(_) => {}
            // ExpressionKind::BoolLiteral(_) => {}
            // ExpressionKind::FloatLiteral(_) => {}
            _ => unreachable!("ILLEGAL NODE WHILE INFERRING:\n{}", expr.indent_print(1)),
        }
    }

    pub fn typecheck_assignment(
        &mut self,
        init: &mut Initialization,
    ) -> Result<(), TypeCheckError> {
        let InitializationKind::Unpacked(inits) = &mut init.kind else {
            unreachable!("AST should have assignments unpacked")
        };

        for init in inits.iter_mut() {
            self.typecheck_simple_assignment(init)?;
        }
        Ok(())
    }
    pub fn typecheck_simple_assignment(
        &mut self,
        init: &mut SimpleInitialization,
    ) -> Result<(), TypeCheckError> {
        let t_actual = self.infer_expr(&init.value)?;
        if let Some(t) = &mut init.typ {
            let t_expected = self.intering_table.introduce(&t);
            self.unify_equify_ids(t_actual, t_expected)?;
        } else {
            let t_actual_spec = self.get_resolved_type(t_actual).cloned()?;
            init.typ = Some(t_actual_spec);
        }
        Ok(())
    }
}
