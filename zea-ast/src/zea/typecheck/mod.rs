use crate::zea;
use crate::zea::{BinOp, ExpressionKind, Module};
use indexmap::{IndexMap, IndexSet};
pub fn BUILTIN_TYPES() -> [zea::Type; 5] {
    [
        zea::Type::I64(),
        zea::Type::Bool(),
        zea::Type::F64(),
        zea::Type::Unit(),
        zea::Type::Exit(),
    ]
}

pub enum TypeCheckError {
    UnifyError(TypeConcreteId, TypeConcreteId),
}
/// The id that a conrete type gets during type-checking
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
    pub fn introduce(&mut self, typ: zea::Type) -> TypeConcreteId {
        let (index, _existed) = self.type_ids.insert_full(typ);
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

struct ModuleInferenceContext<'ast> {
    ast: &'ast Module,
    /// unique types within a program, use a [`TypeConcreteId`] to lookup
    intering_table: TypeInterningTable,
    subst_table: TypeVarSubstitutionTable,
    /// values in this map are references to entries in the arena
    node_types: IndexMap<usize, InferenceId>,
}

impl<'ast> ModuleInferenceContext<'ast> {
    pub fn new(ast: &'ast Module) -> Self {
        Self {
            ast,
            intering_table: TypeInterningTable::with_zea_types(),
            subst_table: TypeVarSubstitutionTable::new(),
            node_types: IndexMap::new(),
        }
    }

    pub fn type_bool_id(&self) -> InferenceId {
        self.intering_table.interned_bool().into()
    }

    pub fn introduce(&mut self, expr: &zea::Expression) -> Result<(), TypeCheckError> {
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

    fn introduce_trivial_type(&mut self, expr: &zea::Expression) -> Result<(), TypeCheckError> {
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
        Ok(())
    }
    fn introduce_non_trivial_type(&mut self, expr: &zea::Expression) -> Result<(), TypeCheckError> {
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
                self.introduce(arg.as_ref())?;
            }
            ExpressionKind::MemberAccess(datatype, member) => {
                let var = self.subst_table.fresh();
                self.node_types.insert(expr.id, var.into());
                self.introduce(datatype.as_ref());
            }
            ExpressionKind::CondBranch(branch) => {
                let var = self.subst_table.fresh();
                self.node_types.insert(expr.id, var.into());
                let cond_id = self.get_or_introduce_inference_id(branch.condition.as_ref())?;
                let true_id = self.get_or_introduce_inference_id(branch.true_case.as_ref())?;
                if let Some(ref false_case) = branch.false_case {
                    let false_id = self.get_or_introduce_inference_id(false_case.as_ref())?;
                    self.unify_ids(false_id, true_id)?;
                    self.unify_ids(cond_id, self.type_bool_id())?
                }
            }
            ExpressionKind::ExpandedBlock(b) => {
                let var = self.subst_table.fresh();
                self.node_types.insert(expr.id, var.into());
                let self_id = self.get_or_introduce_inference_id(expr)?;

                let last_id = self.get_or_introduce_inference_id(&b.last)?;
                self.unify_ids(self_id, last_id)?;
            }
            ExpressionKind::Block(_) => {
                unreachable!("AST should not contain any un-expanded blocks")
            }
            _ => unreachable!(
                "AST node with id {} is not a non-trivial expression: {:?}",
                expr.id, expr.kind
            ),
        };
        Ok(())
    }

    pub fn get_or_introduce_inference_id(
        &mut self,
        node: &zea::Expression,
    ) -> Result<InferenceId, TypeCheckError> {
        if !self.node_types.contains_key(&node.id) {
            self.introduce(node)?;
        }
        Ok(*self.node_types.get(&node.id).unwrap())
    }

    pub fn follow_inference_id(&mut self, var: InferenceId) -> InferenceId {
        match var {
            InferenceId::TypeConcrete(c) => var,
            InferenceId::TypeVar(v) => self.subst_table.get_resolved_type(v),
        }
    }

    pub fn try_unify(
        &mut self,
        expr: &zea::Expression,
        with: &zea::Expression,
    ) -> Result<(), TypeCheckError> {
        let expr = self.get_or_introduce_inference_id(expr)?;
        let with = self.get_or_introduce_inference_id(with)?;
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
                if self.valid_subtype(ta, tb) {
                    Ok(())
                } else {
                    Err(TypeCheckError::UnifyError(a, b))
                }
            }
            (InferenceId::TypeConcrete(c), InferenceId::TypeVar(v)) => self.unify_ids(with, expr),
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
            (zea::Type::ArrayOf(t1), zea::Type::ArrayOf(t2)) => self.valid_subtype(t1, t2),
            (zea::Type::Pointer(t1), zea::Type::Pointer(t2)) => self.valid_subtype(t1, t2),
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

    pub fn infer_binop(
        op: BinOp,
        lhs_id: InferenceId,
        rhs_id: InferenceId,
    ) -> Result<InferenceId, TypeCheckError> {
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
            _ => unreachable!()
        }
    }
}
