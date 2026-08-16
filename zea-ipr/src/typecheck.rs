//! This module contains the implementation of the type checker for the zea language
//! The type checker operates on the IPR.
//! When finished, a [`IPRModuleTypeInfo`] struct is returned.
//! The following invariants hold:
//! - every [`IPRExpression`] node is typed, and you can lookup its type by its ID.
//! - Every [`IPRSimpleInitialization`] has its `.typ` field set to `Some(t)`,
//!   where `t` is the inferred, or supplied type.
//! - If the typechecker did not return an error, the supplied IPR is considered well-typed.
//!
//! From now on I will use TC to refer to the type checker as I am too lazy to keep writing it.
//! ## the process of type checking
//!
//! Since this version of Zea does not feature generics,
//! the TC is quite simple conceptually; It assigns each typeable-node
//! (expressions and symbol bindings) a type variable,
//! and then through a system of constraints and rules, solves them to some concrete type.
//!
//!
//! The following inference-rules are in order:
//!
//! arithmetic:
//! - An integer literal gets inferred the narrowest unsigned integer type that it fits.
//! - The type of the result of srithmetic on integer types is the widest type of its operands
//!   if their signs match.
//! - An unisgned integer may be implicitly cast into a signed one,
//!   where it will be widened if necessary; `255: U8 => 255: I16`
//!   because `255` does not fit in `I8`.
//! - negation converts an unsigned integer to a signed one, and keeps a signed one signed
//! - booleans can be widened into any other integer type, where `Falsee => 0` and `True => 1`.
//!
//! block-like-expressions:
//! - The type of a block is the type of its last expression (its `.tail` field)
//! - The twigs of a branch must have the same type
//! - The type of a branch is that of its twigs
//!
//!
//!
//! Lets walk through the type checking of the code `a := -3 + 257`
//!
//! We first introduce each expression:
//!
//! `t0: (3)`
//! `t1: (-3)`
//! `t2: (257)`
//! `t3: (-3+257)`
//! `t4: (a := -3 + 257)`
//!
//! You may notice the binding itself also gets assigned a variable,
//! this is to allow the solving of referring to identifiers like `a := 3 + 257; b := a;`,
//! as an [`IPRScopedIdentifier`] holds the id to the binding site of the symbol, not to its value.
//!
//! During introduction, we can already solve `t0` and `t2`
//! - `t0` fits inside a `U8`; `t0 := U8`
//! - `t2` does not fit inside a `U8`, but it does inside a `U16`; `t2 := U16`
//!
//! Then we can solve each variable in the following order:
//! - `t1` has type `I16`, because negation converts `UN`'s into `IN`'s or `IN+1`'s; `t1 := I16`
//! - `t3` has the widest type of its operands if their signs match
//!     - the signs do not match; we perform an implicit sign conversion `t2 := I16`
//!     - the sign now match; `t3 := I16`
//! - `t4` can now be solved; `t4 := I16`
//!
//! You may notice some relationships between variables.
//! The notation `t[e]` denotes the type variable assigned to the expression `e`.
//! The notation `t[e]: T` denotes that the type variable for expression `e` has the type `T`.
//!
//! For any [`IPRSimpleInitialization`], it holds that
//! `t[init] == t[init.typ] == t[init.value]`
//!
//! For any [`IPRBlockExpression`]: `t[block] == t[block.tail]`
//!
//! For any [`IPRBranch`] with an else: `t[branch] == t[branch.then] == t[branch.otherwise]`
//! For any [`IPRBranch`] without an else: `t[branch]: Unit`
//!
//! For any [`IPRFunction`]: `t[func.returns] == t[func.body] == t[func.body.last]`
//!
//! For any statement of the form `e ;` (some semicolon terminated expression): `e; : Unit`
//! For any call to a function `f` with definition `func`: `t[f()] == t[func.returns]`
//!
//! The TC uses a union-find structure for its type variables.
//! Applying the `find` operation on a type variables returns a (possibly different) type variable.
//! The sepcific variable returned is not really significant.
//! What is significant however,
//! is when two separate variables return the same variable
//! after a `find` call (i.e. `a.find == b.find`). This means the TC considers them equal.
//!
//! When applying some rule specified above, it can apply `unify` to two variables `a` and `b`,
//! This causes `a` and `b` to then be considered equal like specified above.
//!
//! Additionaly, the `set_solved` operation on a type variable can bind an [`InternedTypeID`] `t`
//! to a type variable, which then causes the `get_solved` operation to return `Some(t)`.
//! Applying `get_solved` to some type variable that is not yet bound returns `None`.
//!
//! Consequently, unifying two variables`a` and `b`,
//! and solving one of them to type `t` using `set_solved`
//! will cause both `a` and `b` to have their `get_solved` return `Some(t)`.
//!
//! The TC inference mechanism features three main categories of functions:
//! - `introduce_[node]`
//! - `infer_[node]`
//! - `check_[node]`
//!
//! The `introduce_` family of functions has the following job:
//! - Assign each expression and binding site in the IPR a type variable and add it to `node_variables`
//! - Intern any type that is mentioned in the code
//! - solve the type variables of any trivially solvable expression (literals mostly)
//!
//! The `infer_` functions then walk the tree and attempt to solve the type
//! of the given expression using the available information.
//!
//! The `check_` functions verify that each inferred type satisfies the typing rules, i.e.
//! - An initialization has its annotation and iferred type equal.
//! - return statement return the same type as the function does.
//!
//! Additionally, the `check_` functions will insert an initializations inferred value type
//! into the annotation if it had none
//!
//! Any solved expression is added to the `node_types` map,
//! which maps an Expression- or Binding-site's ID to an [`InternedTypeID`].
//! The `type_interning_table` is used to map an `InternedTypeID` to an [`IPRTypespecifier`].
//!
//! When all variables in the `node_variables` map are bound and verified, type checking is done,
//! the `node_types` and `type_interining_table` are then extracted into a [`IPRModuleTypeInfo`]
//!
use std::{collections::HashMap, process::exit};

use indexmap::{IndexMap, IndexSet};
use log::{error, trace};
use zea_common::internal_compiler_error;
use zea_internal_macros::InternKey;

use crate::{
    InternTable, ZeaError,
    ast::{BinOp, NodeId, ipr::*},
    visualisation::IndentPrint,
};
pub fn typecheck_module(module: &mut IPRModule) -> IPRModuleTypeInfo {
    let mut tc = ZeaTypeChecker::new();
    tc.introduce_module(module)
        .expect("error introducing module");
    tc.check_module_panicking(module)
}

const BUILTIN_SCALAR_TYPES: [IPRTypeSpecifier; 10] = [
    IPRTypeSpecifier::t_Bool(),
    IPRTypeSpecifier::t_I8(),
    IPRTypeSpecifier::t_I16(),
    IPRTypeSpecifier::t_I32(),
    IPRTypeSpecifier::t_I64(),
    IPRTypeSpecifier::t_U8(),
    IPRTypeSpecifier::t_U16(),
    IPRTypeSpecifier::t_U32(),
    IPRTypeSpecifier::t_U64(),
    // TypeSpecifier::t_F32(),
    // TypeSpecifier::t_F64(),
    IPRTypeSpecifier::t_Unit(),
    // TypeSpecifier::t_Never(),
];

/// Determine the narrowst built-in integer type that fits this literal
fn narrowest_int_type(literal: usize) -> IPRTypeSpecifier {
    if literal <= u8::MAX as usize {
        IPRTypeSpecifier::t_U8()
    } else if literal <= u16::MAX as usize {
        IPRTypeSpecifier::t_U16()
    } else if literal <= u32::MAX as usize {
        IPRTypeSpecifier::t_U32()
    } else if literal <= u64::MAX as usize {
        IPRTypeSpecifier::t_U64()
    } else {
        unreachable!("too fucking big literal bra: {literal}")
    }
}

/// The id that a concrete type gets during type-checking
#[derive(Copy, Clone, Eq, PartialEq, Hash, InternKey)]
pub struct InternedTypeId(usize);

impl InternedTypeId {
    fn as_typevar(self, table: &mut TypeVariableInterningTable) -> TypeVariable {
        table.interned_type_as_variable(self)
    }
}

impl std::fmt::Debug for InternedTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TypeConcreteId({})", self.0)
    }
}

/// The id that a type-variable gets during type-checking
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct TypeVariable(usize);

impl std::fmt::Debug for TypeVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TypeVar({})", self.0)
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
        TypeVariable(len)
    }

    fn follow_var(&self, typevar: TypeVariable) -> TypeVariable {
        let mut typevar_id = typevar.0;

        loop {
            let follow = self.typevar_disjoint_set[typevar_id];
            if follow == typevar_id {
                break TypeVariable(typevar_id);
            } else {
                typevar_id = follow;
            }
        }
    }

    fn follow_once_mut(&mut self, t: TypeVariable) -> &mut usize {
        self.typevar_disjoint_set
            .get_mut(t.0)
            .expect("missing expected type variable")
    }
    fn follow_once(&self, t: usize) -> usize {
        self.typevar_disjoint_set
            .get(t)
            .cloned()
            .expect("missing expected type variable")
    }

    fn union(&mut self, a: TypeVariable, b: TypeVariable) -> Result<(), TypeCheckError> {
        let follow_a = self.follow_var(a);
        let follow_a_representative = self.follow_once_mut(follow_a);
        *follow_a_representative = b.0;
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
    ///
    /// // t1->t2
    /// table.union(t1, t2);
    ///
    /// let t2 = table.fresh_var;
    ///
    /// // t1->t2->t3
    /// table.union(t2, t3);
    ///
    /// // t3 <- t2
    /// // /\
    /// // |
    /// // t1
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
                let mut idx_follow = self.follow_once(idx);
                while idx_follow != self.typevar_disjoint_set[idx_follow] {
                    self.typevar_disjoint_set[idx] = idx_repr;
                    idx = idx_follow;
                    idx_follow = self.typevar_disjoint_set[idx_follow];
                }
            }
        }
        Ok(())
    }
    /// follow some index to its representative, and count how many steps where needed to get there
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
            .unwrap_or_else(|| self.fresh_solved(interned_type))
    }
    /// Generate a fresh variable and immediatly solve it to the supplied type
    fn fresh_solved(&mut self, interned_type: InternedTypeId) -> TypeVariable {
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
        let variable = self.follow_var(variable);
        self.solved_variables.insert(variable, to);
        Ok(())
    }

    /// Try to retrieve the interned type id of a variable if it is surrently solved.
    pub fn get_solved(&self, typevar: TypeVariable) -> Option<InternedTypeId> {
        let typevar = self.follow_var(typevar);
        self.solved_variables.get(&typevar).cloned()
    }
    /// Return all unique representatives in the table
    pub fn disjoint_typevars(&self) -> IndexSet<TypeVariable> {
        let mut res = IndexSet::new();

        for t in self.typevar_disjoint_set.iter().map(|i| TypeVariable(*i)) {
            res.insert(self.follow_var(t));
        }
        res
    }
}

/// A table holding all unique types within a module.
#[derive(Debug)]
struct TypeInterningTable {
    interned_types: InternTable<InternedTypeId, IPRTypeSpecifier>,
}

impl TypeInterningTable {
    pub fn new() -> Self {
        Self {
            interned_types: InternTable::new(),
        }
    }
    pub fn with_builtin_types() -> Self {
        let mut new = Self::new();
        for t in BUILTIN_SCALAR_TYPES.iter() {
            new.intern(t);
        }
        new
    }

    /// introduce some type into the table, generating an id associated with that specifier.
    /// If the type was already introduced, return its id
    pub fn intern(&mut self, typ: &IPRTypeSpecifier) -> InternedTypeId {
        self.interned_types.intern(typ.clone())
    }

    /// try to lookup some [`TypeSpecifier`] by its associated ID
    /// Returns [`TypeCheckError::MissingInternedType`] if the id is not present in the table
    /// (was not `intern()`'ed)
    pub fn get_specifier_by_id(&self, id: InternedTypeId) -> &IPRTypeSpecifier {
        self.interned_types
            .get_by_id(id)
            .expect("missing expected type id")
    }
}

#[derive(Debug, Clone, Copy)]
enum TypeCheckError {
    IllegalTypeCoercion(InternedTypeId, InternedTypeId, IllegalTypeCoercionKind),
    ExpectedSolvedTypeVariable(TypeVariable),
    InvalidOperands(NodeId, BinOp),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IllegalTypeCoercionKind {
    NarrowingConversion,
    SignLossConversion,
    InconvertibleTypes,
    PossibleOverflow,
    LossOfPrecision,
    PointerCast,
}

/// A struct which provides a way to lookup the type of:
/// - [`IPRExpression`]
/// - [`IPRFuncParams`]
/// - [`IPRSimpleINitialization`]
pub struct IPRModuleTypeInfo {
    type_interning_table: InternTable<InternedTypeId, IPRTypeSpecifier>,
    node_types: HashMap<NodeId, InternedTypeId>,
}

impl From<ZeaTypeChecker> for IPRModuleTypeInfo {
    fn from(value: ZeaTypeChecker) -> Self {
        Self {
            type_interning_table: value.type_interning_table.interned_types,
            node_types: value.node_types,
        }
    }
}

impl IPRModuleTypeInfo {
    pub fn lookup(&self, id: NodeId) -> &IPRTypeSpecifier {
        let intern_id = self.node_types[&id];
        self.type_interning_table.get_by_id(intern_id).unwrap()
    }
}

impl<'m> ZeaError<'m> for TypeCheckError {
    type ErrContext = ZeaTypeChecker;
    fn zea_error_format(&'m self, ctx: &'m Self::ErrContext) -> String {
        match self {
            Self::IllegalTypeCoercion(a, b, kind) => {
                let t_a = ctx.type_interning_table.get_specifier_by_id(*a);
                let t_b = ctx.type_interning_table.get_specifier_by_id(*b);
                format!("illegal type coercion of types {t_a:?} and {t_b:?}: {kind:?}")
            }
            Self::InvalidOperands(id, op) => {
                format!("illegal operands for operator {op:?} in {id:?}",)
            }
            _ => todo!(),
        }
    }
}

struct ZeaTypeChecker {
    type_interning_table: TypeInterningTable,
    typevar_interning_table: TypeVariableInterningTable,
    node_variables: HashMap<NodeId, TypeVariable>,
    node_types: HashMap<NodeId, InternedTypeId>,
}

// ================================================================================================
// util
// ================================================================================================
impl ZeaTypeChecker {
    pub fn new() -> Self {
        Self {
            type_interning_table: TypeInterningTable::with_builtin_types(),
            typevar_interning_table: TypeVariableInterningTable::new(),
            node_variables: HashMap::with_capacity(64),
            node_types: HashMap::new(),
        }
    }

    pub fn finish(self) -> IPRModuleTypeInfo {
        self.into()
    }
    /// Get the type variable associated with some expression node,
    /// or generate it if it does not yet exist
    fn get_inference_id(&mut self, id: NodeId) -> &mut TypeVariable {
        self.node_variables
            .entry(id)
            .or_insert_with(|| self.typevar_interning_table.fresh_var())
    }

    /// Solve some type variable to a given type specifier
    fn set_solved_typespec(
        &mut self,
        inf_var: TypeVariable,
        typ: &IPRTypeSpecifier,
    ) -> Result<(), TypeCheckError> {
        let t_id = self.type_interning_table.intern(typ);
        trace!("\tsolving type variable {inf_var:?} of literal to type {typ:?}");
        self.typevar_interning_table.set_solved(inf_var, t_id)?;
        Ok(())
    }

    fn get_solved_typespec(
        &self,
        inf_var: TypeVariable,
    ) -> Result<&IPRTypeSpecifier, TypeCheckError> {
        let solved = self
            .typevar_interning_table
            .get_solved(inf_var)
            .ok_or(TypeCheckError::ExpectedSolvedTypeVariable(inf_var))?;
        Ok(self.type_interning_table.get_specifier_by_id(solved))
    }

    fn get_bool_tvar(&mut self) -> TypeVariable {
        self.type_interning_table
            .intern(&IPRTypeSpecifier::Bool)
            .as_typevar(&mut self.typevar_interning_table)
    }

    fn get_u64_tvar(&mut self) -> TypeVariable {
        self.type_interning_table
            .intern(&IPRTypeSpecifier::t_U64())
            .as_typevar(&mut self.typevar_interning_table)
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
                    "\tsetting variable {a:?} to solved {:?}",
                    self.type_interning_table.get_specifier_by_id(b_conc)
                );
                self.typevar_interning_table.set_solved(a, b_conc)?;
                Ok(b)
            }
            (None, None) => {
                trace!("\tunifying variables {a:?} and {b:?}",);
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
        let t_from = self.type_interning_table.get_specifier_by_id(typ);
        let t_to = self.type_interning_table.get_specifier_by_id(to);
        Self::try_coerce_types(t_from, t_to)
            .map_err(|kind| TypeCheckError::IllegalTypeCoercion(typ, to, kind))?;
        Ok(to)
    }

    fn try_coerce_types<'types>(
        typ: &'types IPRTypeSpecifier,
        to: &'types IPRTypeSpecifier,
    ) -> Result<&'types IPRTypeSpecifier, IllegalTypeCoercionKind> {
        match (typ, to) {
            (a, b) if a == b => Ok(to),

            (
                IPRTypeSpecifier::Integer {
                    width: width_a,
                    signed: signed_a,
                },
                IPRTypeSpecifier::Integer {
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
            (
                IPRTypeSpecifier::Float { width: width_a },
                IPRTypeSpecifier::Float { width: width_b },
            ) => {
                if width_a > width_b {
                    Err(IllegalTypeCoercionKind::LossOfPrecision)
                } else {
                    Ok(to)
                }
            }
            // booleans may be widened, where false => 0, true => 1
            (IPRTypeSpecifier::Bool, IPRTypeSpecifier::Integer { .. }) => Ok(to),
            // the Never type can always be cast, as it will never reach code after it anyway
            (IPRTypeSpecifier::Never, _) => Ok(to),

            (IPRTypeSpecifier::Pointer(a), IPRTypeSpecifier::Pointer(b)) if a != b => {
                Err(IllegalTypeCoercionKind::PointerCast)
            }
            _ => Err(IllegalTypeCoercionKind::InconvertibleTypes),
        }
    }
}

// ================================================================================================
// introduce_ methods
// ================================================================================================
impl ZeaTypeChecker {
    /// recursively generate typevars for the given assignment block
    fn introduce_assignment_block(&mut self, assigment: &IPRInitializationBlock) {
        let IPRInitializationKind::Unpacked(inits) = &assigment.kind else {
            internal_compiler_error!(sui)
        };
        for init in inits.iter() {
            self.introduce_assignment(init);
        }
    }

    /// recursively generate typevars for the assignments's symbol and its value
    fn introduce_assignment(&mut self, init: &IPRSimpleInitialization) {
        if let Some(t) = &init.typ {
            let _ = self.type_interning_table.intern(t);
        }
        let _ = *self.get_inference_id(init.id);
        self.introduce_expression(&init.value);
    }

    fn introduce_expression(&mut self, expr: &IPRExpression) {
        let inf_var = *self.get_inference_id(expr.id);
        match &expr.kind {
            IPRExpressionKind::Unit => {
                self.set_solved_typespec(inf_var, &IPRTypeSpecifier::Unit)
                    .expect("unit literal should solve just fine...");
                let t_unit = self.type_interning_table.intern(&IPRTypeSpecifier::Unit);
                self.node_types.insert(expr.id, t_unit);
            }
            IPRExpressionKind::IntegerLiteral(i) => {
                let typ = narrowest_int_type(*i);
                let typ_id = self.type_interning_table.intern(&typ);
                self.set_solved_typespec(inf_var, &typ)
                    .expect("integer literals should solve just fine...");
                self.node_types.insert(expr.id, typ_id);
            }
            IPRExpressionKind::BoolLiteral(_) => {
                let t_bool = self.type_interning_table.intern(&IPRTypeSpecifier::Bool);
                self.set_solved_typespec(inf_var, &IPRTypeSpecifier::Bool)
                    .expect("boolean literal should solve just fine...");
                self.node_types.insert(expr.id, t_bool);
            }
            IPRExpressionKind::FloatLiteral(_) => {
                self.set_solved_typespec(inf_var, &IPRTypeSpecifier::t_F64())
                    .expect("float literal should solve just fine...");
            }
            IPRExpressionKind::StringLiteral(_) => todo!(),
            IPRExpressionKind::UnScopedIdent(_) => {
                internal_compiler_error!(sui)
            }
            IPRExpressionKind::ScopedIdent(_) => {}
            IPRExpressionKind::FunctionCall(_) => todo!(),
            IPRExpressionKind::BinOpExpr(_, l, r) => {
                self.introduce_expression(l.as_ref());
                self.introduce_expression(r.as_ref());
            }
            IPRExpressionKind::UnOpExpr(_, arg) => {
                self.introduce_expression(arg.as_ref());
            }
            IPRExpressionKind::MemberAccess(subject, ..) => {
                self.introduce_expression(subject.as_ref());
            }
            IPRExpressionKind::IfThenElse(_) => todo!(),
            IPRExpressionKind::Block(b) => {
                for stmt in b.statements.iter() {
                    self.introduce_stmt(stmt);
                }
                self.introduce_expression(&b.tail);
            }
        }
    }

    fn introduce_module(&mut self, module: &IPRModule) -> Result<(), TypeCheckError> {
        for glob in module.global_vars.iter() {
            self.introduce_assignment_block(glob);
        }
        for f in module.functions.iter() {
            self.introduce_function(f);
        }

        Ok(())
    }

    fn introduce_block(&mut self, b: &IPRBlockExpression) {
        let _ = *self.get_inference_id(b.id);
        for s in b.statements.iter() {
            self.introduce_stmt(s);
        }
        self.introduce_expression(&b.tail);
    }
    fn introduce_stmt(&mut self, s: &IPRStatement) {
        match &s.kind {
            IPRStatementKind::Initialization(i) => self.introduce_assignment_block(i),
            IPRStatementKind::Reassignment(_iprreassignment) => todo!(),
            IPRStatementKind::FunctionCall(_iprfunction_call) => todo!(),
            IPRStatementKind::Return(_iprexpression) => todo!(),
            IPRStatementKind::Block(_iprblock_expression) => todo!(),
            IPRStatementKind::IfThenElse(_iprbranch) => todo!(),
        }
    }

    fn introduce_function(&mut self, f: &IPRFunction) {
        for param in f.params.iter() {
            let IPRFuncParam { typ, id, .. } = param;
            let t = self.type_interning_table.intern(typ);
            let t_id = *self.get_inference_id(*id);
            self.set_solved_typespec(t_id, typ)
                .expect("cannot set-solved func param");
            self.node_types.insert(*id, t);
            todo!()
        }
        self.type_interning_table.intern(&f.returns);
        self.introduce_block(&f.body);
    }
}

// ================================================================================================
// infer_ methods
// ================================================================================================
impl ZeaTypeChecker {
    fn infer_expression(&mut self, expr: &IPRExpression) -> Result<TypeVariable, TypeCheckError> {
        let t_var = *self.get_inference_id(expr.id);
        if self.get_solved_typespec(t_var).is_ok() {
            trace!("\tskipping solved expression");
            return Ok(t_var);
        }
        let res = match &expr.kind {
            IPRExpressionKind::IntegerLiteral(_)
            | IPRExpressionKind::BoolLiteral(_)
            | IPRExpressionKind::FloatLiteral(_)
            | IPRExpressionKind::Unit => {
                trace!("\tinferring literal yields existing type variable");
                Ok(t_var)
            }

            IPRExpressionKind::BinOpExpr(op, l, r) => {
                trace!("\tinferring binop");
                self.infer_expr_binop(expr.id, *op, l.as_ref(), r.as_ref())
            }
            IPRExpressionKind::Block(b) => {
                let t_tail = self.infer_expression(&b.tail)?;
                self.hindley_milner_unify(t_var, t_tail)?;
                Ok(t_tail)
            }
            IPRExpressionKind::ScopedIdent(s) => {
                let t_referrant = *self.get_inference_id(s.origin);
                self.hindley_milner_unify(t_var, t_referrant)?;
                // self.trace_expr_typevar(expr);
                if let Some(int_solved) = self.typevar_interning_table.get_solved(t_referrant) {
                    self.node_types.insert(expr.id, int_solved);
                }
                Ok(t_referrant)
            }
            IPRExpressionKind::StringLiteral(_)
            | IPRExpressionKind::FunctionCall(_)
            | IPRExpressionKind::IfThenElse(_)
            | IPRExpressionKind::UnOpExpr(_, _) => todo!(),
            IPRExpressionKind::MemberAccess(..) => {
                todo!("member access inference")
            }
            IPRExpressionKind::UnScopedIdent(_) => {
                internal_compiler_error!(sui)
            }
        }?;

        trace!(
            "\t\tinferred expression\n{}\nto be: {:?}",
            expr.indent_print(0),
            self.get_solved_typespec(res)
        );
        self.node_variables.insert(expr.id, res);
        Ok(res)
    }
    fn infer_expr_binop(
        &mut self,
        id: NodeId,
        op: BinOp,
        l: &IPRExpression,
        r: &IPRExpression,
    ) -> Result<TypeVariable, TypeCheckError> {
        let var = *self.get_inference_id(id);
        let l_var = self.infer_expression(l)?;
        let r_var = self.infer_expression(r)?;
        let _l_t = self.get_solved_typespec(l_var)?.clone();
        let r_t = self.get_solved_typespec(r_var)?.clone();
        let bool_t = self.get_bool_tvar();
        let u64_t = self.get_u64_tvar();
        match op {
            BinOp::Add
            | BinOp::Sub
            | BinOp::Mul
            | BinOp::Div
            | BinOp::Mod
            | BinOp::BitAnd
            | BinOp::BitOr
            | BinOp::BitXor
            | BinOp::Lsh
            | BinOp::Rsh => {
                self.hindley_milner_unify(l_var, r_var)?;
                match r_t {
                    IPRTypeSpecifier::Integer { .. } => {
                        self.hindley_milner_unify(var, r_var)?;
                        Ok(r_var)
                    }
                    IPRTypeSpecifier::Bool => {
                        self.hindley_milner_unify(var, u64_t)?;
                        Ok(u64_t)
                    }
                    _ => Err(TypeCheckError::InvalidOperands(id, op)),
                }
            }
            BinOp::Subscript => todo!(),
            BinOp::LT | BinOp::GT => {
                self.hindley_milner_unify(l_var, r_var)?;
                self.hindley_milner_unify(var, bool_t)?;
                Ok(bool_t)
            }
            BinOp::Eq
            | BinOp::Neq
            | BinOp::Geq
            | BinOp::Leq
            | BinOp::LogAnd
            | BinOp::LogOr
            | BinOp::LogXor => {
                self.hindley_milner_unify(r_var, bool_t)?;
                self.hindley_milner_unify(l_var, r_var)?;
                self.hindley_milner_unify(var, bool_t)?;
                Ok(bool_t)
            }
        }
    }
}

// ================================================================================================
// check_ methods
// ================================================================================================
impl ZeaTypeChecker {
    pub fn check_module_panicking(mut self, module: &mut IPRModule) -> IPRModuleTypeInfo {
        match self.check_module(module) {
            Ok(_) => {}
            Err(_) => {
                error!("exiting...");
                exit(1)
            }
        }
        self.finish()
    }
    pub fn check_module(&mut self, module: &mut IPRModule) -> Result<(), TypeCheckError> {
        self.introduce_module(module)?;
        self.trace_solved_stats();

        while self.check_module_once(module).is_ok() {
            let solved = self.trace_solved_stats();
            if solved {
                self.check_module_once(module)?;
                break;
            }
            self.typevar_interning_table.compress_paths()?;
        }
        Ok(())
    }
    /// Iterate once over a module, inserting typespecs where possible.
    /// Stops upon encoutering an unrecoverable error, that is, an error that is not
    /// [`TypeCheckError::ExpectedSolvedTypeVariable`]
    fn check_module_once(&mut self, module: &mut IPRModule) -> Result<(), TypeCheckError> {
        for glob in module.global_vars.iter_mut() {
            match self.check_assignment(glob) {
                Ok(_) => {}
                Err(TypeCheckError::ExpectedSolvedTypeVariable(t)) => {
                    self.trace_insufficient_info_for_solving(t);
                }
                Err(other) => {
                    error!("TYPE ERROR: {}", other.zea_error_format(self));
                    return Err(other);
                }
            }
        }

        for f in module.functions.iter_mut() {
            match self.check_function(f) {
                Ok(_) => {}
                Err(TypeCheckError::ExpectedSolvedTypeVariable(t)) => {
                    self.trace_insufficient_info_for_solving(t);
                }
                Err(other) => {
                    error!("{}", other.zea_error_format(self));
                    return Err(other);
                }
            }
        }

        Ok(())
    }
    fn check_function(&mut self, f: &mut IPRFunction) -> Result<(), TypeCheckError> {
        trace!("checking function `{}`", f.name);
        let body_returns = self.check_block(&mut f.body)?;
        let signature_expects = &f.returns;
        let ret_typ = self.type_interning_table.intern(signature_expects);
        let ret_tvar = ret_typ.as_typevar(&mut self.typevar_interning_table);
        self.hindley_milner_unify(body_returns, ret_tvar)?;

        Ok(())
    }

    fn check_block(
        &mut self,
        body: &mut IPRBlockExpression,
    ) -> Result<TypeVariable, TypeCheckError> {
        let tvar_block = *self.get_inference_id(body.id);
        for stmt in body.statements.iter_mut() {
            self.check_stmt(stmt)?;
        }
        let tvar = self.infer_expression(&body.tail)?;
        let block_rets = self
            .typevar_interning_table
            .get_solved(tvar)
            .ok_or(TypeCheckError::ExpectedSolvedTypeVariable(tvar))?;
        self.hindley_milner_unify(tvar_block, tvar)?;
        self.node_types.insert(body.id, block_rets);
        self.node_types.insert(body.tail.id, block_rets);
        Ok(tvar)
    }

    fn check_stmt(&mut self, stmt: &mut IPRStatement) -> Result<(), TypeCheckError> {
        match &mut stmt.kind {
            IPRStatementKind::Initialization(i) => {
                self.check_assignment(i)?;
            }
            IPRStatementKind::Reassignment(_iprreassignment) => todo!(),
            IPRStatementKind::FunctionCall(_iprfunction_call) => todo!(),
            IPRStatementKind::Return(_iprexpression) => todo!(),
            IPRStatementKind::Block(_iprblock_expression) => todo!(),
            IPRStatementKind::IfThenElse(_iprbranch) => todo!(),
        }
        Ok(())
    }

    fn check_assignment(
        &mut self,
        assign: &mut IPRInitializationBlock,
    ) -> Result<(), TypeCheckError> {
        let IPRInitializationKind::Unpacked(inits) = &mut assign.kind else {
            internal_compiler_error!(spi)
        };
        for init in inits.iter_mut() {
            trace!("\tchecking simple assigment for symbol `{}`", init.assignee);
            self.check_simple_assignment(init)?;
        }
        Ok(())
    }

    fn check_simple_assignment(
        &mut self,
        assign: &mut IPRSimpleInitialization,
    ) -> Result<(), TypeCheckError> {
        if self.node_types.contains_key(&assign.id) {
            trace!("\t\tskipping annotated initialization");
            return Ok(());
        }
        let t_init = *self.get_inference_id(assign.id);
        let t_init_value = *self.get_inference_id(assign.value.id);
        self.hindley_milner_unify(t_init, t_init_value)?;

        let t_inferred = self.infer_expression(&assign.value)?;
        if let Some(t_actual) = &assign.typ {
            let t_actual_id = self.type_interning_table.intern(t_actual);
            let t_actual_as_var = t_actual_id.as_typevar(&mut self.typevar_interning_table);
            self.hindley_milner_unify(t_inferred, t_actual_as_var)?;
            self.node_types.insert(assign.id, t_actual_id);
        } else {
            let t_conc_id = self
                .typevar_interning_table
                .get_solved(t_inferred)
                .ok_or(TypeCheckError::ExpectedSolvedTypeVariable(t_inferred))?;
            let t_conc = self
                .type_interning_table
                .get_specifier_by_id(t_conc_id)
                .clone();
            trace!(
                "\tannotating symbol `{}` with type {:?}",
                assign.assignee, t_conc
            );
            assign.typ = Some(t_conc);
            self.node_types.insert(assign.id, t_conc_id);
        }
        Ok(())
    }
}

// ================================================================================================
// tracing
// ================================================================================================
impl ZeaTypeChecker {
    /// TRACE-print a message about having insufficient info to solve a type variable
    fn trace_insufficient_info_for_solving(&self, t: TypeVariable) {
        trace!("\tinsufficient information to solve type variable {t:?}, moving on...");
    }
    // calculate stats about solving type variables
    fn get_solving_stats(&self) -> (bool, f64, usize, usize) {
        let disj_typevars = self.typevar_interning_table.disjoint_typevars();
        let mut vars_solved = 0;
        let vars_total = disj_typevars.len();
        for t in disj_typevars.iter() {
            if self.typevar_interning_table.get_solved(*t).is_some() {
                vars_solved += 1;
            }
        }
        let frac = (vars_solved as f64 / vars_total as f64) * 100.0;
        (vars_solved == vars_total, frac, vars_solved, vars_total)
    }
    /// TRACE-print info about the percentage of solved type variables,
    /// and return if all of them are solved
    fn trace_solved_stats(&self) -> bool {
        let (solved, frac, vars_solved, vars_total) = self.get_solving_stats();
        trace!("\tsolved {frac:.2}% ({vars_solved} of {vars_total}) of typevars");
        solved
    }
}

#[cfg(test)]
mod tests {

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
        let (_, t3_length) = table.follow_with_path_length(t3.0);
        assert_eq!(t3_length, 2);

        // t1 -> t2
        let (_, t1_length) = table.follow_with_path_length(t1.0);
        assert_eq!(t1_length, 1);

        let (_, t2_length) = table.follow_with_path_length(t2.0);
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

        assert_eq!(table.follow_with_path_length(t1.0).1, 4);

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

        assert_eq!(table.follow_with_path_length(t1.0).1, 2);

        table.compress_paths().unwrap();
        path_compression_invariant(&table);
    }

    use crate::ast::test_ast_macros::ztyp;

    #[test]
    fn coercion_rules() {
        let i64 = IPRTypeSpecifier::t_I64();
        let i8 = IPRTypeSpecifier::t_I8();
        let u64 = IPRTypeSpecifier::t_U64();
        let u8 = IPRTypeSpecifier::t_U8();
        let bool = IPRTypeSpecifier::t_Bool();
        let unit = IPRTypeSpecifier::t_Unit();
        let never = IPRTypeSpecifier::t_Never();
        let f32 = IPRTypeSpecifier::t_F32();
        let f64 = IPRTypeSpecifier::t_F64();

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
