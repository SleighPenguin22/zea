use std::{
    arch::x86_64::_SIDD_MASKED_NEGATIVE_POLARITY, collections::HashMap, hash::DefaultHasher,
    ops::ControlFlow, process::exit, thread::sleep, time::Duration,
};

use indexmap::{Equivalent, IndexMap, IndexSet, map::raw_entry_v1::RawEntryBuilderMut};
use log::{error, info, trace};
use zea_internal_macros::{InternKey, VariantToStr};

use crate::{
    InternTable, ZeaError,
    visualisation::IndentPrint,
    zea::{BinOp, NodeId, ZeaNodeQuery, ipr::*, visitors::annotating::SymbolKind},
};
pub fn typecheck_module(module: &mut IPRModule) -> IPRModuleTypeInfo {
    let mut tc = ZeaTypeChecker::new();
    tc.introduce_module(module)
        .expect("error introducing module");
    tc.check_module(module)
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

    fn follow_once_mut(&mut self, t: TypeVariable) -> Result<&mut usize, TypeCheckError> {
        self.typevar_disjoint_set
            .get_mut(t.0)
            .ok_or(TypeCheckError::MissingTypeVariable(t))
    }
    fn follow_once(&self, t: usize) -> Result<usize, TypeCheckError> {
        self.typevar_disjoint_set
            .get(t)
            .cloned()
            .ok_or(TypeCheckError::MissingTypeVariable(TypeVariable(t)))
    }

    fn union(&mut self, a: TypeVariable, b: TypeVariable) -> Result<(), TypeCheckError> {
        let follow_a = self.follow_var(a);
        let follow_a_representative = self.follow_once_mut(follow_a)?;
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
            new.introduce(t);
        }
        new
    }

    /// introduce some type into the table, generating an id associated with that specifier.
    /// If the type was already introduced, return its id
    pub fn introduce(&mut self, typ: &IPRTypeSpecifier) -> InternedTypeId {
        self.interned_types.intern(typ.clone())
    }

    /// try to lookup some [`TypeSpecifier`] by its associated ID
    /// Returns [`TypeCheckError::MissingInternedType`] if the id is not present in the table
    pub fn get_specifier_by_id(
        &self,
        id: InternedTypeId,
    ) -> Result<&IPRTypeSpecifier, TypeCheckError> {
        self.interned_types
            .get_by_id(id)
            .ok_or(TypeCheckError::MissingInternedType(id))
    }
}

#[derive(Debug, Clone, Copy)]
enum TypeCheckError {
    MissingInternedType(InternedTypeId),
    MissingTypeVariable(TypeVariable),
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

struct ZeaTypeChecker {
    type_interning_table: TypeInterningTable,
    typevar_interning_table: TypeVariableInterningTable,
    node_variables: HashMap<NodeId, TypeVariable>,
    node_types: HashMap<NodeId, InternedTypeId>,
}

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

macro_rules! coercion_rule_widen {
    ($t_a:ident, $t_b:ident, $width_a:ident, $width_b:ident, $t_res:ident) => {{
        if $width_a < $width_b {
            Ok($t_res.into())
        } else {
            Err(TypeCheckError::IllegalTypeCoercion($t_a, $t_b))
        }
    }};
}

impl<'m> ZeaError<'m> for TypeCheckError {
    type ErrContext = ZeaTypeChecker;
    fn zea_error_format(&'m self, ctx: &Self::ErrContext) -> String {
        match self {
            Self::IllegalTypeCoercion(a, b, kind) => {
                let t_a = ctx.type_interning_table.get_specifier_by_id(*a).unwrap();
                let t_b = ctx.type_interning_table.get_specifier_by_id(*b).unwrap();
                format!("illegal type coercion of types {t_a:?} and {t_b:?}: {kind:?}")
            }
            Self::InvalidOperands(id, op) => {
                format!(
                    "illegal operands for operator {} in {id:?}",
                    op.variant_as_str()
                )
            }
            _ => todo!(),
        }
    }
}

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

    pub fn inner_check_module(&mut self, module: &mut IPRModule) -> Result<(), TypeCheckError> {
        self.introduce_module(module)?;
        let (_solved, frac, vars_solved, vars_total) = self.all_vars_solved();
        trace!("\tinitially solved {frac}% ({vars_solved} of {vars_total}) of typevars");
        while self.check_module_once(module).is_ok() {
            let (solved, frac, vars_solved, vars_total) = self.all_vars_solved();
            trace!("\tsolved {frac}% ({vars_solved} of {vars_total}) of typevars");
            if solved {
                self.check_module_once(module)?;
                break;
            }
            sleep(Duration::from_millis(300));
        }
        Ok(())
    }
    pub fn check_module(mut self, module: &mut IPRModule) -> IPRModuleTypeInfo {
        match self.inner_check_module(module) {
            Ok(_) => {}
            Err(_) => {
                error!("exiting...");
                exit(1)
            }
        }
        self.finish()
    }
    fn check_module_once(&mut self, module: &mut IPRModule) -> Result<(), TypeCheckError> {
        for glob in module.global_vars.iter_mut() {
            match self.check_assignment(glob) {
                Ok(_) => {}
                Err(TypeCheckError::ExpectedSolvedTypeVariable(t)) => {
                    trace!("\tinsufficient information to solve type variable {t:?}, moving on...");
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
                    trace!("\tinsufficient information to solve type variable {t:?}, moving on...");
                }
                Err(other) => {
                    error!("{}", other.zea_error_format(self));
                    return Err(other);
                }
            }
        }

        Ok(())
    }

    fn all_vars_solved(&self) -> (bool, f64, usize, usize) {
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

    /// Get the type variable associated with some expression node,
    /// or generate it if it does not yet exist
    fn get_inference_id(&mut self, id: NodeId) -> &mut TypeVariable {
        self.node_variables
            .entry(id)
            .or_insert_with(|| self.typevar_interning_table.fresh_var())
    }
    fn solve_to_type(
        &mut self,
        inf_var: TypeVariable,
        typ: &IPRTypeSpecifier,
    ) -> Result<(), TypeCheckError> {
        let t_id = self.type_interning_table.introduce(typ);
        trace!("\tsolving type variable {inf_var:?} of literal to type {typ:?}");
        let _ = self.typevar_interning_table.set_solved(inf_var, t_id);
        Ok(())
    }

    fn introduce_assignment(&mut self, assigment: &IPRInitializationBlock) {
        let IPRInitializationKind::Unpacked(inits) = &assigment.kind else {
            unreachable!("initializations should be unpacked before typechecks")
        };
        for init in inits.iter() {
            self.introduce_simple_assignment(init);
        }
    }
    fn introduce_simple_assignment(&mut self, assignment: &IPRSimpleInitialization) {
        if let Some(t) = &assignment.typ {
            let _ = self.type_interning_table.introduce(t);
        }
        let _ = *self.get_inference_id(assignment.id);
        self.introduce_expression(&assignment.value);
    }

    fn introduce_expression(&mut self, expr: &IPRExpression) {
        let inf_var = *self.get_inference_id(expr.id);
        match &expr.kind {
            IPRExpressionKind::Unit => {
                self.solve_to_type(inf_var, &IPRTypeSpecifier::Unit)
                    .expect("unit literal should solve just fine...");
                let t_unit = self.type_interning_table.introduce(&IPRTypeSpecifier::Unit);
                self.node_types.insert(expr.id, t_unit);
            }
            IPRExpressionKind::IntegerLiteral(i) => {
                let typ = narrowest_int_type(*i);
                let typ_id = self.type_interning_table.introduce(&typ);
                self.solve_to_type(inf_var, &typ)
                    .expect("integer literals should solve just fine...");
                self.node_types.insert(expr.id, typ_id);
            }
            IPRExpressionKind::BoolLiteral(_) => {
                let t_bool = self.type_interning_table.introduce(&IPRTypeSpecifier::Bool);
                self.solve_to_type(inf_var, &IPRTypeSpecifier::Bool)
                    .expect("boolean literal should solve just fine...");
                self.node_types.insert(expr.id, t_bool);
            }
            IPRExpressionKind::FloatLiteral(_) => {
                self.solve_to_type(inf_var, &IPRTypeSpecifier::t_F64())
                    .expect("float literal should solve just fine...");
            }
            IPRExpressionKind::StringLiteral(_) => todo!(),
            IPRExpressionKind::UnScopedIdent(_) => {
                unreachable!("identifiers should be scoped before type checking")
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
            IPRExpressionKind::MemberAccess(_, _) => todo!(),
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
            self.introduce_assignment(glob);
        }
        for f in module.functions.iter() {
            self.introduce_function(f);
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
        let t_from = self.type_interning_table.get_specifier_by_id(typ)?;
        let t_to = self.type_interning_table.get_specifier_by_id(to)?;
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

    fn check_function(&mut self, f: &mut IPRFunction) -> Result<(), TypeCheckError> {
        trace!("checking function `{}`", f.name);
        let body_returns = self.check_block(&mut f.body)?;
        let signature_expects = &f.returns;
        let ret_typ = self.type_interning_table.introduce(signature_expects);
        let ret_tvar = ret_typ.as_typevar(&mut self.typevar_interning_table);
        self.hindley_milner_unify(body_returns, ret_tvar)?;

        Ok(())
    }
    fn introduce_block(&mut self, b: &IPRBlockExpression) {
        let _ = *self.get_inference_id(b.id);
        for s in b.statements.iter() {
            self.introduce_stmt(s);
        }
        self.introduce_expression(&b.tail);
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

    fn introduce_stmt(&mut self, s: &IPRStatement) {
        match &s.kind {
            IPRStatementKind::Initialization(i) => self.introduce_assignment(i),
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
            let t = self.type_interning_table.introduce(typ);
            self.node_types.insert(*id, t);
            todo!()
        }
        self.type_interning_table.introduce(&f.returns);
        self.introduce_block(&f.body);
    }
}

impl ZeaTypeChecker {
    fn check_assignment(
        &mut self,
        assign: &mut IPRInitializationBlock,
    ) -> Result<(), TypeCheckError> {
        let IPRInitializationKind::Unpacked(inits) = &mut assign.kind else {
            unreachable!("inits should be unpacked before type checking");
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
            let t_actual_id = self.type_interning_table.introduce(t_actual);
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
                .get_specifier_by_id(t_conc_id)?
                .clone();
            trace!(
                "\tannotating symbol `{:?}` with type {:?}",
                assign.assignee, t_conc
            );
            assign.typ = Some(t_conc);
            self.node_types.insert(assign.id, t_conc_id);
        }
        Ok(())
    }

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
                self.trace_expr_typevar(expr);
                if let Some(int_solved) = self.typevar_interning_table.get_solved(t_referrant) {
                    self.node_types.insert(expr.id, int_solved);
                }
                Ok(t_referrant)
            }
            IPRExpressionKind::StringLiteral(_)
            | IPRExpressionKind::FunctionCall(_)
            | IPRExpressionKind::UnOpExpr(_, _)
            | IPRExpressionKind::MemberAccess(_, _)
            | IPRExpressionKind::IfThenElse(_) => todo!(),
            IPRExpressionKind::UnScopedIdent(_) => {
                unreachable!("identifiers should be scoped before typechecking")
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

    fn get_solved_typespec(
        &self,
        inf_var: TypeVariable,
    ) -> Result<&IPRTypeSpecifier, TypeCheckError> {
        let solved = self
            .typevar_interning_table
            .get_solved(inf_var)
            .ok_or(TypeCheckError::ExpectedSolvedTypeVariable(inf_var))?;
        self.type_interning_table.get_specifier_by_id(solved)
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
        let bool_t = self.bool_t();
        let u64_t = self.u64_t();
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

    fn bool_t(&mut self) -> TypeVariable {
        self.type_interning_table
            .introduce(&IPRTypeSpecifier::Bool)
            .as_typevar(&mut self.typevar_interning_table)
    }

    fn u64_t(&mut self) -> TypeVariable {
        self.type_interning_table
            .introduce(&IPRTypeSpecifier::t_U64())
            .as_typevar(&mut self.typevar_interning_table)
    }

    fn trace_expr_typevar(&mut self, expr: &IPRExpression) {
        let var = *self.get_inference_id(expr.id);
        trace!("{var:?} for expr {expr:?}");
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

    use crate::zea::test_ast_macros::ztyp;

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
