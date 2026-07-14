use std::{arch::x86_64::_SIDD_MASKED_NEGATIVE_POLARITY, collections::HashMap};

use indexmap::IndexSet;
use zea_internal_macros::VariantToStr;

use crate::zea::{
    self, Expression, InitializationBlock, InitializationKind, NodeId, SimpleInitialization,
    TypeSpecifier,
};

const BUILTIN_SCALAR_TYPES: [zea::TypeSpecifier; 9] = [
    zea::TypeSpecifier::t_Bool(),
    zea::TypeSpecifier::t_I8(),
    zea::TypeSpecifier::t_I16(),
    zea::TypeSpecifier::t_I32(),
    zea::TypeSpecifier::t_I64(),
    zea::TypeSpecifier::t_U8(),
    zea::TypeSpecifier::t_U16(),
    zea::TypeSpecifier::t_U32(),
    zea::TypeSpecifier::t_U64(),
    // TypeSpecifier::t_F32(),
    // TypeSpecifier::t_F64(),
    // TypeSpecifier::t_Unit(),
    // TypeSpecifier::t_Never(),
];
/// The id that a concrete type gets during type-checking
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct InternedTypeId {
    id: usize,
}

impl std::fmt::Debug for InternedTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TypeConcreteId({})", self.id)
    }
}

/// The id that a type-variable gets during type-checking
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct TypeVariable {
    id: usize,
}

impl std::fmt::Debug for TypeVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TypeVar({})", self.id)
    }
}

struct TypeVariableInterningTable {
    typevar_disjoint_set: Vec<usize>,
    solved_variables: HashMap<TypeVariable, InternedTypeId>,
}

impl TypeVariableInterningTable {
    pub fn new() -> Self {
        Self {
            typevar_disjoint_set: vec![],
            solved_variables: HashMap::new(),
        }
    }

    pub fn fresh_var(&mut self) -> TypeVariable {
        let len = self.typevar_disjoint_set.len();
        self.typevar_disjoint_set.push(len);
        TypeVariable { id: len }
    }

    fn follow_var(&self, typevar: TypeVariable) -> TypeVariable {
        let mut typevar_id = typevar.id;

        loop {
            let follow = self.typevar_disjoint_set[typevar_id];
            if follow == typevar_id {
                break TypeVariable { id: typevar_id };
            } else {
                typevar_id = follow;
            }
        }
    }

    fn follow_once_mut(&mut self, t: TypeVariable) -> Result<&mut usize, TypeCheckError> {
        self.typevar_disjoint_set
            .get_mut(t.id)
            .ok_or(TypeCheckError::MissingTypeVariable(t))
    }
    fn follow_once(&self, t: usize) -> Result<usize, TypeCheckError> {
        self.typevar_disjoint_set
            .get(t)
            .cloned()
            .ok_or(TypeCheckError::MissingTypeVariable(TypeVariable { id: t }))
    }

    fn union(&mut self, a: TypeVariable, b: TypeVariable) -> Result<(), TypeCheckError> {
        let follow_a = self.follow_var(a);
        let follow_a_representative = self.follow_once_mut(follow_a)?;
        *follow_a_representative = b.id;
        Ok(())
    }

    /// Update all paths to point directly at their representative.
    /// As a result, all path_lengths according to [`TypeVariableInterningTable::follow_with_path_length`]
    /// will be 1 or less, while representatives are preserved
    ///
    /// ```ignore
    /// let mut table = Self::new();
    /// let t1 = table.fresh_var();
    /// let t2 = table.fresh_var();
    /// // t1->t2
    /// table.union(t1, t2);
    /// let t2 = table.fresh_var;
    /// // t1->t2->t3
    /// table.union(t2, t3);
    ///
    /// table.compress_paths();
    ///
    /// for t in table.typevar_disjoint_set.iter() {
    ///   let (_, len) = table.follow_with_path_length(t.id);
    ///   assert!(len <= 1);
    /// }
    ///
    /// ````
    fn compress_paths(&mut self) -> Result<(), TypeCheckError> {
        for t in self.typevar_disjoint_set.clone().iter() {
            let idx = *t;
            let (idx_repr, len) = self.follow_with_path_length(idx);
            if len > 1 {
                let mut idx = idx;
                let mut idx_follow = self.follow_once(idx)?;
                while idx_follow != self.typevar_disjoint_set[idx_follow] {
                    self.typevar_disjoint_set[idx] = idx_repr;
                    idx = idx_follow;
                    idx_follow = self.typevar_disjoint_set[idx_follow];
                }
            }
        }
        Ok(())
    }

    fn follow_with_path_length(&self, t: usize) -> (usize, usize) {
        let mut t_id = t;
        let mut path_length = 0;
        loop {
            let follow = self.typevar_disjoint_set[t_id];
            if follow != t_id {
                path_length += 1;
                t_id = follow;
            } else {
                break (t_id, path_length);
            }
        }
    }

    /// add a known type to the table,
    /// such that all type variable within that set now point to the given interned type ID.
    /// Applies path compression
    pub fn set_solved(
        &mut self,
        variable: TypeVariable,
        to: InternedTypeId,
    ) -> Result<(), TypeCheckError> {
        self.compress_paths()?;
        self.solved_variables.insert(variable, to);
        Ok(())
    }

    /// Try to retrieve the interned type id of a variable if it is surrently solved.
    pub fn get_solved(&self, typevar: TypeVariable) -> Option<InternedTypeId> {
        let typevar = self.follow_var(typevar);
        self.solved_variables.get(&typevar).cloned()
    }
}

/// A table holding all unique types within a module.
#[derive(Debug)]
struct TypeInterningTable {
    interned_types: IndexSet<zea::TypeSpecifier>,
}

impl TypeInterningTable {
    pub fn new() -> Self {
        Self {
            interned_types: IndexSet::new(),
        }
    }
    pub fn with_builtin_types() -> Self {
        let mut new = Self::new();
        for t in BUILTIN_SCALAR_TYPES.iter() {
            new.introduce(t);
        }
        new
    }

    /// introduce some type into the table, generating an id associated with that specifier.
    /// If the type was already introduced, return its id
    pub fn introduce(&mut self, typ: &TypeSpecifier) -> InternedTypeId {
        if let Some((existing_idx, _)) = self.interned_types.get_full(typ) {
            InternedTypeId { id: existing_idx }
        } else {
            let (idx, _) = self.interned_types.insert_full(typ.clone());
            InternedTypeId { id: idx }
        }
    }

    pub fn is_introduced(&self, typ: &TypeSpecifier) -> bool {
        self.interned_types.contains(typ)
    }
    /// try to lookup some [`TypeSpecifier`] by its associated ID
    /// Returns [`TypeCheckError::MissingInternedType`] if the id is not present in the table
    pub fn get_specifier_by_id(
        &self,
        id: InternedTypeId,
    ) -> Result<&TypeSpecifier, TypeCheckError> {
        self.interned_types
            .get_index(id.id)
            .ok_or(TypeCheckError::MissingInternedType(id))
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug, VariantToStr)]
enum InferenceId {
    Solved(InternedTypeId), // map some type-id to an actual type within an interning-table
    Unsolved(TypeVariable),
}
impl From<TypeVariable> for InferenceId {
    fn from(value: TypeVariable) -> Self {
        InferenceId::Unsolved(value)
    }
}

impl InferenceId {
    pub fn is_solved(&self) -> bool {
        matches!(self, InferenceId::Solved(_))
    }
}

#[derive(Debug, Clone)]
enum TypeCheckError {
    MissingInternedType(InternedTypeId),
    MissingTypeVariable(TypeVariable),
}

struct ZeaTypeChecker {
    type_interning_table: TypeInterningTable,
    typevar_interning_table: TypeVariableInterningTable,
    node_types: HashMap<NodeId, InferenceId>,
}
impl ZeaTypeChecker {
    pub fn new() -> Self {
        Self {
            type_interning_table: TypeInterningTable::with_builtin_types(),
            typevar_interning_table: TypeVariableInterningTable::new(),
            node_types: HashMap::with_capacity(64),
        }
    }
    pub fn check_module(&mut self, _module: &mut zea::Module) -> Result<(), TypeCheckError> {
        Ok(())
    }

    fn introduce_assignment(&mut self, assigment: &InitializationBlock) {
        let InitializationKind::Unpacked(inits) = &assigment.kind else {
            unreachable!("initializations should be unpacked before typechecks")
        };
        for init in inits.iter() {
            self.introduce_simple_assignment(init);
        }
    }
    fn introduce_simple_assignment(&mut self, assignment: &SimpleInitialization) {
        if let Some(t) = &assignment.typ {
            let _ = self.type_interning_table.introduce(t);
        }
        self.introduce_expression(&assignment.value);
    }

    fn generate_inference_id(&mut self, id: NodeId) -> InferenceId {
        let t_var = self.typevar_interning_table.fresh_var();
        self.node_types.insert(id, t_var.into());
        t_var.into()
    }

    fn introduce_expression(&mut self, expr: &Expression) {
        let inf_var = self.generate_inference_id(expr.id);
        match &expr.kind {
            zea::ExpressionKind::Unit => todo!(),
            zea::ExpressionKind::IntegerLiteral(_) => todo!(),
            zea::ExpressionKind::BoolLiteral(_) => todo!(),
            zea::ExpressionKind::FloatLiteral(_) => todo!(),
            zea::ExpressionKind::StringLiteral(_) => todo!(),
            zea::ExpressionKind::UnScopedIdent(_) => {
                unreachable!("identifiers should be scoped before type checking")
            }
            zea::ExpressionKind::ScopedIdent(scoped_identifier) => todo!(),
            zea::ExpressionKind::FunctionCall(function_call) => todo!(),
            zea::ExpressionKind::BinOpExpr(bin_op, expression, expression1) => todo!(),
            zea::ExpressionKind::UnOpExpr(un_op, expression) => todo!(),
            zea::ExpressionKind::MemberAccess(expression, _) => todo!(),
            zea::ExpressionKind::IfThenElse(if_then_else) => todo!(),
            zea::ExpressionKind::Block(block_expression) => todo!(),
        }
    }

    fn hindley_milner_unify(
        &mut self,
        a: InferenceId,
        b: InferenceId,
    ) -> Result<InferenceId, TypeCheckError> {
        match (a, b) {
            (InferenceId::Solved(a_conc), InferenceId::Solved(b_conc)) => {
                self.try_coerce_types(a_conc, b_conc)
            }
            (InferenceId::Solved(a_conc), InferenceId::Unsolved(b_var)) => {
                self.typevar_interning_table.set_solved(b_var, a_conc);
                Ok(a)
            }
            (InferenceId::Unsolved(a_var), InferenceId::Solved(b_conc)) => {
                self.typevar_interning_table.set_solved(a_var, b_conc);
                Ok(b)
            }
            (InferenceId::Unsolved(a_var), InferenceId::Unsolved(b_var)) => {
                self.typevar_interning_table.union(a_var, b_var)?;
                Ok(self.typevar_interning_table.follow_var(a_var).into())
            }
        }
    }
}

impl Default for ZeaTypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use indexmap::map::MutableKeys;

    use super::*;
    fn path_compression_invariant(table: &TypeVariableInterningTable) {
        for t in table.typevar_disjoint_set.iter() {
            assert!(table.follow_with_path_length(*t).1 <= 1);
        }
    }
    #[test]
    fn typevartable() {
        let mut table = TypeVariableInterningTable::new();
        let t1 = table.fresh_var();
        let t2 = table.fresh_var();

        assert_eq!(table.typevar_disjoint_set.len(), 2);

        assert_eq!(table.follow_var(t1), t1);
        assert_eq!(table.follow_var(t2), t2);
        assert_ne!(table.follow_var(t1), t2);
        assert_ne!(table.follow_var(t2), t1);

        table
            .union(t1, t2)
            .expect("unioning existing typevars should work");

        assert_ne!(table.follow_var(t1), t1);
        assert_eq!(table.follow_var(t1), t2);

        let t3 = table.fresh_var();
        table.union(t3, t1).unwrap();
        assert_eq!(table.follow_var(t3), t2);
        assert_eq!(table.follow_var(t2), t2);
        assert_eq!(table.follow_var(t1), t2);

        // t3 -> t1 -> t2
        let (_, t3_length) = table.follow_with_path_length(t3.id);
        assert_eq!(t3_length, 2);

        // t1 -> t2
        let (_, t1_length) = table.follow_with_path_length(t1.id);
        assert_eq!(t1_length, 1);

        let (_, t2_length) = table.follow_with_path_length(t2.id);
        assert_eq!(t2_length, 0);

        table.compress_paths().unwrap();
        path_compression_invariant(&table);
    }

    #[test]
    fn typevartable_compression() {
        let mut table = TypeVariableInterningTable::new();
        let t1 = table.fresh_var();
        let t2 = table.fresh_var();
        let t3 = table.fresh_var();
        let t4 = table.fresh_var();
        let t5 = table.fresh_var();

        table.union(t1, t2).unwrap();
        table.union(t2, t3).unwrap();
        table.union(t3, t4).unwrap();
        table.union(t4, t5).unwrap();

        assert_eq!(table.follow_with_path_length(t1.id).1, 4);

        table.compress_paths().unwrap();
        path_compression_invariant(&table);

        let mut table = TypeVariableInterningTable::new();
        let t1 = table.fresh_var();
        let t2 = table.fresh_var();
        let t3 = table.fresh_var();
        let t4 = table.fresh_var();
        let t5 = table.fresh_var();

        // t1->t2  t3  t4  t5
        table.union(t1, t2).unwrap();
        // t1->t2->t3  t4  t5
        table.union(t2, t3).unwrap();
        // t1->t2->t3<-t4  t5
        table.union(t4, t3).unwrap();
        // t1->t2->t3<-t4<-t5
        table.union(t5, t4).unwrap();

        assert_eq!(table.follow_with_path_length(t1.id).1, 2);

        table.compress_paths().unwrap();
        path_compression_invariant(&table);
    }
}
