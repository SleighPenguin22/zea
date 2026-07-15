use std::{
    arch::x86_64::_SIDD_MASKED_NEGATIVE_POLARITY, collections::HashMap, hash::DefaultHasher,
};

use indexmap::{map::raw_entry_v1::RawEntryBuilderMut, Equivalent, IndexMap, IndexSet};
use log::trace;
use zea_internal_macros::VariantToStr;

use crate::{
    visualisation::IndentPrint,
    zea::{
        self, Expression, InitializationBlock, InitializationKind, Module, NodeId,
        SimpleInitialization, TypeSpecifier,
    },
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

/// Determine the narrowst built-in integer type that fits this literal
fn narrowest_int_type(literal: usize) -> TypeSpecifier {
    if literal <= u8::MAX as usize {
        TypeSpecifier::t_U8()
    } else if literal <= u16::MAX as usize {
        TypeSpecifier::t_U16()
    } else if literal <= u32::MAX as usize {
        TypeSpecifier::t_U32()
    } else if literal <= u64::MAX as usize {
        TypeSpecifier::t_U64()
    } else {
        unreachable!("too fucking big literal bra: {literal}")
    }
}

/// The id that a concrete type gets during type-checking
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct InternedTypeId {
    id: usize,
}

impl InternedTypeId {
    pub fn as_typevar(self, table: &mut TypeVariableInterningTable) -> TypeVariable {
        table.interned_type_as_variable(self)
    }
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
    solved_variables: IndexMap<TypeVariable, InternedTypeId>,
}

impl TypeVariableInterningTable {
    pub fn new() -> Self {
        Self {
            typevar_disjoint_set: vec![],
            solved_variables: IndexMap::new(),
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
    /// As a result, all path_lengths according to
    /// [`TypeVariableInterningTable::follow_with_path_length`]
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

    pub fn interned_type_as_variable(&mut self, interned_type: InternedTypeId) -> TypeVariable {
        trace!("creating type variable for interned type {interned_type:?}");
        self.solved_variables
            .iter()
            // if some typevar has the given type, return that
            .find_map(|(k, v)| (*v == interned_type).then_some(k))
            .copied()
            // otherwise create a dummy type variable that immediatly gets that type
            .unwrap_or_else(|| self.generate_dummy_variable_with_type(interned_type))
    }
    fn generate_dummy_variable_with_type(&mut self, interned_type: InternedTypeId) -> TypeVariable {
        trace!("\tsynthesizing type variable for interned type {interned_type:?}");
        let t_var = self.fresh_var();
        let _ = self.set_solved(t_var, interned_type);
        t_var
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

    pub fn disjoint_typevars(&self) -> IndexSet<TypeVariable> {
        let mut res = IndexSet::new();

        for t in self
            .typevar_disjoint_set
            .iter()
            .map(|i| TypeVariable { id: *i })
        {
            res.insert(self.follow_var(t));
        }
        res
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

#[derive(Debug, Clone, Copy)]
enum TypeCheckError {
    MissingInternedType(InternedTypeId),
    MissingTypeVariable(TypeVariable),
    IllegalTypeCoercion(InternedTypeId, InternedTypeId, IllegalTypeCoercionKind),
    ExpectedSolvedTypeVariable(TypeVariable),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IllegalTypeCoercionKind {
    NarrowingConversion,
    SignLossConversion,
    InconvertibleTypes,
    PossibleOverflow,
    LossOfPrecision,
    PointerCast,
}

struct ZeaTypeChecker {
    type_interning_table: TypeInterningTable,
    typevar_interning_table: TypeVariableInterningTable,
    node_types: HashMap<NodeId, TypeVariable>,
    symbol_types: HashMap<NodeId, InternedTypeId>,
}

macro_rules! coercion_rule_widen {
    ($t_a:ident, $t_b:ident, $width_a:ident, $width_b:ident, $t_res:ident) => {{
        if $width_a < $width_b {
            Ok($t_res.into())
        } else {
            Err(TypeCheckError::IllegalTypeCoercion($t_a, $t_b))
        }
    }};
}

pub fn typecheck_module(module: &mut Module) {
    let mut tc = ZeaTypeChecker::new();
    match tc.check_module(module) {
        Ok(_) => {}
        Err(e) => panic!("{e:?}"),
    }
}

impl ZeaTypeChecker {
    pub fn new() -> Self {
        Self {
            type_interning_table: TypeInterningTable::with_builtin_types(),
            typevar_interning_table: TypeVariableInterningTable::new(),
            node_types: HashMap::with_capacity(64),
            symbol_types: HashMap::new(),
        }
    }
    pub fn check_module(&mut self, module: &mut zea::Module) -> Result<(), TypeCheckError> {
        self.introduce_module(module)?;

        'inner: for glob in module.global_vars.iter_mut() {
            match self.check_assignment(glob) {
                Ok(_) => {}
                Err(TypeCheckError::ExpectedSolvedTypeVariable(t)) => {
                    trace!("TYPECHCEKER: insufficient information to solve type variable {t:?}, moving on...");
                }
                other => return other,
            }
            trace!(
                "TYPECHECKER: all type variables solved: {}",
                self.all_vars_solved()
            );
        }
        Ok(())
    }

    fn all_vars_solved(&self) -> bool {
        dbg!(self.typevar_interning_table.disjoint_typevars())
            .iter()
            .all(|t| self.typevar_interning_table.get_solved(*t).is_some())
    }

    /// Get the type variable associated with some expression node,
    /// or generate it if it does not yet exist
    fn get_inference_id(&mut self, id: NodeId) -> &mut TypeVariable {
        self.node_types
            .entry(id)
            .or_insert_with(|| self.typevar_interning_table.fresh_var())
    }
    fn solve_to_type(
        &mut self,
        inf_var: TypeVariable,
        typ: &TypeSpecifier,
    ) -> Result<(), TypeCheckError> {
        let t_id = self.type_interning_table.introduce(typ);
        trace!("TYPECHCKER: solving type variable {inf_var:?} of literal to type {typ:?}");
        let _ = self.typevar_interning_table.set_solved(inf_var, t_id);
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

    fn introduce_expression(&mut self, expr: &Expression) {
        let inf_var = *self.get_inference_id(expr.id);
        match &expr.kind {
            zea::ExpressionKind::Unit => {}
            zea::ExpressionKind::IntegerLiteral(i) => {
                let typ = narrowest_int_type(*i);
                self.solve_to_type(inf_var, &typ)
                    .expect("integer literals should solve just fine...")
            }
            zea::ExpressionKind::BoolLiteral(_) => {
                self.solve_to_type(inf_var, &TypeSpecifier::Bool)
                    .expect("boolean literal should solve just fine...");
            }
            zea::ExpressionKind::FloatLiteral(_) => {
                self.solve_to_type(inf_var, &TypeSpecifier::t_F64())
                    .expect("float literal should solve just fine...");
            }
            zea::ExpressionKind::StringLiteral(_) => todo!(),
            zea::ExpressionKind::UnScopedIdent(_) => {
                unreachable!("identifiers should be scoped before type checking")
            }
            zea::ExpressionKind::ScopedIdent(_) => todo!(),
            zea::ExpressionKind::FunctionCall(_) => todo!(),
            zea::ExpressionKind::BinOpExpr(_, l, r) => {
                self.introduce_expression(l.as_ref());
                self.introduce_expression(r.as_ref());
            }
            zea::ExpressionKind::UnOpExpr(_, arg) => {
                self.introduce_expression(arg.as_ref());
            }
            zea::ExpressionKind::MemberAccess(_, _) => todo!(),
            zea::ExpressionKind::IfThenElse(_) => todo!(),
            zea::ExpressionKind::Block(_) => todo!(),
        }
    }

    fn introduce_module(&mut self, module: &Module) -> Result<(), TypeCheckError> {
        for glob in module.global_vars.iter() {
            self.introduce_assignment(glob);
        }

        Ok(())
    }

    fn hindley_milner_unify(
        &mut self,
        a: TypeVariable,
        b: TypeVariable,
    ) -> Result<TypeVariable, TypeCheckError> {
        let a_solved = self.typevar_interning_table.get_solved(a);
        let b_solved = self.typevar_interning_table.get_solved(b);
        match (a_solved, b_solved) {
            (Some(a_conc), Some(b_conc)) => {
                self.try_coerce_type_ids(a_conc, b_conc)
                    .map(|interned_type_id| {
                        interned_type_id.as_typevar(&mut self.typevar_interning_table)
                    })
            }
            (Some(_), None) => self.hindley_milner_unify(b, a),
            (None, Some(b_conc)) => {
                trace!(
                    "TYPECHECKER: setting variable {a:?} to solved {:?}",
                    self.type_interning_table.get_specifier_by_id(b_conc)
                );
                self.typevar_interning_table.set_solved(a, b_conc)?;
                Ok(b)
            }
            (None, None) => {
                trace!("TYPECHECKER: unifying variables {a:?} and {b:?}",);
                self.typevar_interning_table.union(a, b)?;
                Ok(self.typevar_interning_table.follow_var(a))
            }
        }
    }

    fn try_coerce_type_ids(
        &mut self,
        typ: InternedTypeId,
        to: InternedTypeId,
    ) -> Result<InternedTypeId, TypeCheckError> {
        let t_from = self.type_interning_table.get_specifier_by_id(typ)?;
        let t_to = self.type_interning_table.get_specifier_by_id(to)?;
        Self::try_coerce_types(t_from, t_to)
            .map_err(|kind| TypeCheckError::IllegalTypeCoercion(typ, to, kind))?;
        Ok(to)
    }

    fn try_coerce_types<'types>(
        typ: &'types TypeSpecifier,
        to: &'types TypeSpecifier,
    ) -> Result<&'types TypeSpecifier, IllegalTypeCoercionKind> {
        match (typ, to) {
            (a, b) if a == b => Ok(to),

            (
                TypeSpecifier::Integer {
                    width: width_a,
                    signed: signed_a,
                },
                TypeSpecifier::Integer {
                    width: width_b,
                    signed: signed_b,
                },
            ) => {
                if width_a > width_b {
                    Err(IllegalTypeCoercionKind::NarrowingConversion)
                }
                // i -> u
                else if *signed_a && !*signed_b {
                    if width_a == width_b {
                        Err(IllegalTypeCoercionKind::SignLossConversion)
                    } else {
                        Ok(to)
                    }
                } else if !*signed_a && *signed_b {
                    if width_a == width_b {
                        Err(IllegalTypeCoercionKind::PossibleOverflow)
                    } else {
                        Ok(to)
                    }
                } else {
                    Ok(to)
                }
            }

            // floats may be widened
            (TypeSpecifier::Float { width: width_a }, TypeSpecifier::Float { width: width_b }) => {
                if width_a > width_b {
                    Err(IllegalTypeCoercionKind::LossOfPrecision)
                } else {
                    Ok(to)
                }
            }
            // booleans may be widened, where false => 0, true => 1
            (TypeSpecifier::Bool, TypeSpecifier::Integer { .. }) => Ok(to),
            // the Never type can always be cast, as it will never reach code after it anyway
            (TypeSpecifier::Never, _) => Ok(to),

            (TypeSpecifier::Pointer(a), TypeSpecifier::Pointer(b)) if a != b => {
                Err(IllegalTypeCoercionKind::PointerCast)
            }
            _ => Err(IllegalTypeCoercionKind::InconvertibleTypes),
        }
    }
}

impl Default for ZeaTypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl ZeaTypeChecker {
    fn check_assignment(&mut self, assign: &mut InitializationBlock) -> Result<(), TypeCheckError> {
        let InitializationKind::Unpacked(inits) = &mut assign.kind else {
            unreachable!("inits should be unpacked before type checking");
        };
        for init in inits.iter_mut() {
            trace!(
                "TYPECHEKER: checking simple assigment for symbol `{}`",
                init.assignee
            );
            self.check_simple_assignment(init)?;
        }
        Ok(())
    }

    fn check_simple_assignment(
        &mut self,
        assign: &mut SimpleInitialization,
    ) -> Result<(), TypeCheckError> {
        let t_inferred = self.infer_expression(&assign.value)?;
        if let Some(t_actual) = &assign.typ {
            let t_actual_id = self.type_interning_table.introduce(t_actual);
            let t_actual_as_var = t_actual_id.as_typevar(&mut self.typevar_interning_table);
            self.hindley_milner_unify(t_inferred, t_actual_as_var)?;
            self.symbol_types.insert(assign.id, t_actual_id);
        } else {
            let t_conc_id = self
                .typevar_interning_table
                .get_solved(t_inferred)
                .ok_or(TypeCheckError::ExpectedSolvedTypeVariable(t_inferred))?;
            let t_conc = self
                .type_interning_table
                .get_specifier_by_id(t_conc_id)?
                .clone();
            trace!(
                "TYPECHECKER: annotating symbol `{:?}` with type {:?}",
                assign.assignee,
                t_conc
            );
            assign.typ = Some(t_conc);
            self.symbol_types.insert(assign.id, t_conc_id);
        }
        Ok(())
    }

    fn infer_expression(&mut self, expr: &Expression) -> Result<TypeVariable, TypeCheckError> {
        match &expr.kind {
            zea::ExpressionKind::IntegerLiteral(_)
            | zea::ExpressionKind::BoolLiteral(_)
            | zea::ExpressionKind::FloatLiteral(_)
            | zea::ExpressionKind::Unit => {
                let id = *self.get_inference_id(expr.id);
                trace!("TYPECHECKER: inferring literal yields existing type variable");
                Ok(id)
            }

            zea::ExpressionKind::StringLiteral(_)
            | zea::ExpressionKind::ScopedIdent(_)
            | zea::ExpressionKind::FunctionCall(_)
            | zea::ExpressionKind::BinOpExpr(_, _, _)
            | zea::ExpressionKind::UnOpExpr(_, _)
            | zea::ExpressionKind::MemberAccess(_, _)
            | zea::ExpressionKind::IfThenElse(_)
            | zea::ExpressionKind::Block(_)
            | zea::ExpressionKind::UnScopedIdent(_) => todo!(),
        }
    }

    fn get_solved(&self, inf_var: TypeVariable) -> Result<&TypeSpecifier, TypeCheckError> {
        let solved = self
            .typevar_interning_table
            .get_solved(inf_var)
            .ok_or(TypeCheckError::ExpectedSolvedTypeVariable(inf_var))?;
        self.type_interning_table.get_specifier_by_id(solved)
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

    use crate::zea::test_ast_macros::ztyp;

    #[test]
    fn coercion_rules() {
        let i64 = TypeSpecifier::t_I64();
        let i8 = TypeSpecifier::t_I8();
        let u64 = TypeSpecifier::t_U64();
        let u8 = TypeSpecifier::t_U8();
        let bool = TypeSpecifier::t_Bool();
        let unit = TypeSpecifier::t_Unit();
        let never = TypeSpecifier::t_Never();
        let f32 = TypeSpecifier::t_F32();
        let f64 = TypeSpecifier::t_F64();

        for t in BUILTIN_SCALAR_TYPES.iter() {
            assert_eq!(ZeaTypeChecker::try_coerce_types(t, t), Ok(t));
            assert_eq!(ZeaTypeChecker::try_coerce_types(&never, t), Ok(t));
            if t != &unit {
                assert_eq!(
                    ZeaTypeChecker::try_coerce_types(&unit, t),
                    Err(IllegalTypeCoercionKind::InconvertibleTypes)
                );
                assert_eq!(
                    ZeaTypeChecker::try_coerce_types(t, &unit),
                    Err(IllegalTypeCoercionKind::InconvertibleTypes)
                );
            }
        }

        let ptr_i8 = ztyp!(*U8);
        let ptr_u8 = ztyp!(*I8);

        let cases = vec![
            (&i8, &i64, Ok(&i64)),
            (&i8, &i64, Ok(&i64)),
            (&u8, &u64, Ok(&u64)),
            (&u8, &i64, Ok(&i64)),
            (&u8, &u64, Ok(&u64)),
            (&bool, &u64, Ok(&u64)),
            (
                &u64,
                &bool,
                Err(IllegalTypeCoercionKind::InconvertibleTypes),
            ),
            (&f32, &f64, Ok(&f64)),
            (&f64, &f32, Err(IllegalTypeCoercionKind::LossOfPrecision)),
            (&i8, &u8, Err(IllegalTypeCoercionKind::SignLossConversion)),
            (&u8, &i8, Err(IllegalTypeCoercionKind::PossibleOverflow)),
            (&ptr_u8, &ptr_u8, Ok(&ptr_u8)),
            (&ptr_u8, &ptr_i8, Err(IllegalTypeCoercionKind::PointerCast)),
        ];
        for (from, to, res) in cases {
            assert_eq!(ZeaTypeChecker::try_coerce_types(from, to), res);
        }
    }
}
