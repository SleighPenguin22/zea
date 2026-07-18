use std::{collections::HashMap, io::IntoInnerError};

use indexmap::Equivalent;
use zea_internal_macros::{InternKey, VariantToStr};

use crate::{
    zea::{
        immediate_parsed_representation::{
            IPRExpression, IPRInitializationBlock, IPRInitializationKind, IPRModule,
            IPRSimpleInitialization, IPRStatement, IPRTypeSpecifier, IPRTypedIdentifier,
        },
        visitors::{altering::IdentifierScoper, annotating::SymbolKind},
        BinOp, UnOp,
    },
    InternTable,
};

use super::typecheck::IPRModuleTypeInfo;

pub fn lower_module(
    module: IPRModule,
    types: IPRModuleTypeInfo,
    ident_scopes: IdentifierScoper,
) -> THRModule {
    let thr_ctx = THRInternTables::with_integer_types();
    let ipr_ctx = IPRLoweringContext {
        module,
        types,
        ident_scopes,
        cf_stack: vec![],
    };
    thr_ctx.lower_module(&ipr_ctx)
}

#[derive(InternKey, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct THRTypeID(usize);

impl THRTypeID {
    fn alignment(self, ctx: &THRInternTables) -> usize {
        let ty = ctx
            .types
            .get_by_id(self)
            .expect("struct field name should be interned at this point");
        match ty {
            THRTypeSpecifier::Integer { width, signed: _ } => *width as usize,
            THRTypeSpecifier::Float { width } => *width as usize,
            THRTypeSpecifier::Pointer(inner) => inner.alignment(ctx),
            THRTypeSpecifier::Boolean => 1,
            THRTypeSpecifier::Unit => 0,
            THRTypeSpecifier::Never => 0,
            THRTypeSpecifier::Struct { name: _, layout } => layout.alignment(ctx),
            THRTypeSpecifier::Tuple(layout) => layout.alignment(ctx),
        }
    }

    fn width(self, ctx: &THRInternTables) -> usize {
        let ty = ctx
            .types
            .get_by_id(self)
            .expect("struct field name should be interned at this point");
        match ty {
            THRTypeSpecifier::Integer { width, signed: _ } => *width as usize,
            THRTypeSpecifier::Float { width } => *width as usize,
            THRTypeSpecifier::Pointer(inner) => inner.width(ctx),
            THRTypeSpecifier::Boolean => 1,
            THRTypeSpecifier::Unit => 0,
            THRTypeSpecifier::Never => 0,
            THRTypeSpecifier::Struct { name: _, layout } => layout.width(ctx),
            THRTypeSpecifier::Tuple(layout) => layout.width(ctx),
        }
    }
}
#[derive(InternKey, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct THRExprID(usize);
#[derive(InternKey, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct THRSymbolID(usize);
#[derive(InternKey, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct THRStatementID(usize);
#[derive(InternKey, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct THRBlockID(usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct THRModule {
    interned: THRInternTables,
    global_data: THRBlockID,
}

#[derive(Clone, PartialEq, Eq, Default)]
pub struct THRInternTables {
    expressions: InternTable<THRExprID, THRExpression>,
    symbols: InternTable<THRSymbolID, THRSymbol>,
    types: InternTable<THRTypeID, THRTypeSpecifier>,
    statements: InternTable<THRStatementID, THRStatement>,
    blocks: InternTable<THRBlockID, THRBlock>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntegerWidth {
    _8 = 1,
    _16 = 2,
    _32 = 4,
    _64 = 8,
}

impl TryFrom<usize> for IntegerWidth {
    type Error = usize;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            8 => Ok(Self::_8),
            16 => Ok(Self::_16),
            32 => Ok(Self::_32),
            64 => Ok(Self::_64),
            _ => Err(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FloatWidth {
    _32 = 4,
    _64 = 8,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum THRTypeSpecifier {
    Integer {
        width: IntegerWidth,
        signed: bool,
    },
    Float {
        width: FloatWidth,
    },
    Pointer(THRTypeID),
    Boolean,
    Unit,
    Never,
    Struct {
        name: String,
        layout: THRStructLayout,
    },
    Tuple(THRStructLayout),
}

impl THRTypeSpecifier {
    pub const fn i_signed(width: IntegerWidth) -> Self {
        Self::Integer {
            width,
            signed: true,
        }
    }
    pub const fn i_unsigned(width: IntegerWidth) -> Self {
        Self::Integer {
            width,
            signed: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct THRStructLayout {
    ordering: StructOrdering,
    fields: Vec<THRStructField>,
}
impl THRStructLayout {
    fn alignment(&self, ctx: &THRInternTables) -> usize {
        self.fields
            .iter()
            .map(|field| field.typ.alignment(ctx))
            .max()
            .unwrap_or(0)
    }

    fn width(&self, ctx: &THRInternTables) -> usize {
        let struct_alignment = self.alignment(ctx);
        let summed_fields = self.fields.iter().map(|f| f.typ.width(ctx)).sum();
        Self::next_aligned_offset(summed_fields, struct_alignment)
    }

    /// calculate the closest multiple `m` of `required_alignment` s.t. `m >= cur_offset` && `m % required_alignment == 0`
    fn next_aligned_offset(cur_offset: usize, required_alignment: usize) -> usize {
        match (cur_offset, required_alignment) {
            (0, _) => 0,
            (c, a) => {
                if c > a {
                    let (div, rem) = div_rem(c, a);
                    if rem != 0 {
                        (div + 1) * a
                    } else {
                        div * a
                    }
                } else {
                    a
                }
            }
        }
    }
}

pub fn div_rem(x: usize, y: usize) -> (usize, usize) {
    let quot = x.checked_div_euclid(y).unwrap_or(0);
    let rem = x.checked_rem_euclid(y).unwrap_or(0);
    (quot, rem)
}
/// Specify the ordering of a [`THRStructLayout`], note that fields are always aligned
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum StructOrdering {
    /// Do not reorder fields
    Naive,
    /// Order fields in descending size
    Descending,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct THRStructField {
    name: String,
    typ: THRTypeID,
    offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum THRExpression {
    ConstInt(TypedLiteral<u64>),
    ConstFloat(TypedLiteral<f64>),
    ConstBool(bool),
    Binop(BinOp, THRExprID, THRExprID),
    Unop(UnOp, THRExprID),
    Ident(THRSymbolID),
}

impl THRExpression {
    pub const fn const_int(value: u64, typ: THRTypeID) -> THRExpression {
        let literal = TypedLiteral { value, typ };
        THRExpression::ConstInt(literal)
    }
    pub const fn binop(op: BinOp, lhs: THRExprID, rhs: THRExprID) -> THRExpression {
        Self::Binop(op, lhs, rhs)
    }
}

/// used to provide an Eq and Hash impl for floats.
pub trait NumericLiteral: Copy + Clone + PartialEq {
    fn canonical(&self) -> Self;
}

impl NumericLiteral for u64 {
    fn canonical(&self) -> Self {
        *self
    }
}

impl NumericLiteral for f64 {
    fn canonical(&self) -> Self {
        if self.is_nan() {
            f64::NAN
        } else {
            *self
        }
    }
}
impl NumericLiteral for bool {
    fn canonical(&self) -> Self {
        *self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TypedLiteral<T: NumericLiteral> {
    typ: THRTypeID,
    value: T,
}

impl<T: NumericLiteral> PartialEq for TypedLiteral<T> {
    fn eq(&self, other: &Self) -> bool {
        self.typ == other.typ && self.value.canonical() == other.value.canonical()
    }
}
impl<T: NumericLiteral> Eq for TypedLiteral<T> {}

impl std::hash::Hash for TypedLiteral<f64> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.typ.hash(state);
        self.value.canonical().to_bits().hash(state);
    }
}
impl std::hash::Hash for TypedLiteral<u64> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.typ.hash(state);
        self.value.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct THRBlock {
    items: Vec<THRStatementID>,
}

pub enum THRLoweredStatement {
    Single(THRStatementID),
    Multiple(THRBlockID),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, VariantToStr)]
pub enum THRStatement {
    Init(THRInit),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct THRInit {
    symbol: THRSymbolID,
    typ: THRTypeID,
    value: THRExprID,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct THRSymbol {
    name: String,
    kind: SymbolKind,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum THRSymbolKind {
    Global,
    Local,
    FuncName,
    FuncParam,
}

static THR_INTEGER_TYPES: [THRTypeSpecifier; 8] = [
    THRTypeSpecifier::i_signed(IntegerWidth::_8),
    THRTypeSpecifier::i_signed(IntegerWidth::_16),
    THRTypeSpecifier::i_signed(IntegerWidth::_32),
    THRTypeSpecifier::i_signed(IntegerWidth::_64),
    THRTypeSpecifier::i_unsigned(IntegerWidth::_8),
    THRTypeSpecifier::i_unsigned(IntegerWidth::_16),
    THRTypeSpecifier::i_unsigned(IntegerWidth::_32),
    THRTypeSpecifier::i_unsigned(IntegerWidth::_64),
];

impl THRInternTables {
    pub fn with_integer_types() -> Self {
        let mut s = Self::default();
        for t in THR_INTEGER_TYPES.as_slice() {
            s.types.intern(t.clone());
        }
        s
    }

    fn lower_type(&mut self, iprtype: &IPRTypeSpecifier, _ctx: &IPRLoweringContext) -> THRTypeID {
        let t = match iprtype {
            IPRTypeSpecifier::NonScalar(_a) => {
                todo!();
                // let s = ctx
                //     .struct_definitions
                //     .iter()
                //     .find(|s| s.name == *a)
                //     .expect("type checking should resolve struct names");
                // for IPRTypedIdentifier { name, typ } in s.members.iter() {
                //     let typ = self.lower_type(typ, ctx);
                //     let field = THRStructField {
                //         name: name.clone(),
                //         typ,
                //         offset: 0,
                //     };
                // }

                // todo!()
            }
            IPRTypeSpecifier::Unit => THRTypeSpecifier::Unit,
            IPRTypeSpecifier::Bool => THRTypeSpecifier::Boolean,
            IPRTypeSpecifier::Integer { width, signed } => {
                let width = IntegerWidth::try_from(*width)
                    .unwrap_or_else(|_| panic!("illegal integer width: {width}"));
                if *signed {
                    THRTypeSpecifier::i_signed(width)
                } else {
                    THRTypeSpecifier::i_unsigned(width)
                }
            }
            IPRTypeSpecifier::Float { width: _ } => todo!(),
            IPRTypeSpecifier::Pointer(inner) => {
                let inner_id = self.lower_type(inner, _ctx);
                THRTypeSpecifier::Pointer(inner_id)
            }
            IPRTypeSpecifier::ArrayOf(_) => todo!(),
            IPRTypeSpecifier::Never => THRTypeSpecifier::Never,
        };
        self.types.intern(t)
    }

    fn lower_expression(
        &mut self,
        ipr_ctx: &IPRLoweringContext,
        expr: &IPRExpression,
    ) -> THRExprID {
        let ipr_t = ipr_ctx.types.lookup(expr.id);
        let thr_t = self.lower_type(ipr_t, ipr_ctx);
        let node = match &expr.kind {
            super::immediate_parsed_representation::IPRExpressionKind::Unit => todo!(),
            super::immediate_parsed_representation::IPRExpressionKind::IntegerLiteral(i) => {
                THRExpression::const_int(*i as u64, thr_t)
            }
            super::immediate_parsed_representation::IPRExpressionKind::BoolLiteral(b) => {
                THRExpression::ConstBool(*b)
            }
            super::immediate_parsed_representation::IPRExpressionKind::FloatLiteral(_) => todo!(),
            super::immediate_parsed_representation::IPRExpressionKind::StringLiteral(_) => todo!(),
            super::immediate_parsed_representation::IPRExpressionKind::ScopedIdent(i) => {
                let symbol = THRSymbol {
                    name: i.ident.clone(),
                    kind: i.kind,
                };
                let symbol = self.symbols.intern(symbol);
                THRExpression::Ident(symbol)
            }
            super::immediate_parsed_representation::IPRExpressionKind::FunctionCall(
                _iprfunction_call,
            ) => todo!(),
            super::immediate_parsed_representation::IPRExpressionKind::BinOpExpr(op, l, r) => {
                let l = self.lower_expression(ipr_ctx, l.as_ref());
                let r = self.lower_expression(ipr_ctx, r.as_ref());
                THRExpression::binop(*op, l, r)
            }
            super::immediate_parsed_representation::IPRExpressionKind::UnOpExpr(op, arg) => todo!(),
            super::immediate_parsed_representation::IPRExpressionKind::MemberAccess(
                _iprexpression,
                _,
            ) => todo!(),
            super::immediate_parsed_representation::IPRExpressionKind::IfThenElse(_iprbranch) => {
                todo!()
            }
            super::immediate_parsed_representation::IPRExpressionKind::Block(_b) => todo!(),
            super::immediate_parsed_representation::IPRExpressionKind::UnScopedIdent(_) => todo!(),
        };
        self.expressions.intern(node)
    }

    fn lower_stmt(
        &mut self,
        ipr_ctx: &IPRLoweringContext,
        stmt: &IPRStatement,
    ) -> THRLoweredStatement {
        match &stmt.kind {
            super::immediate_parsed_representation::IPRStatementKind::Initialization(init) => {
                let ids = self.lower_init_block(ipr_ctx, init, SymbolKind::LocalVar);
                let block = THRBlock { items: ids };
                let block = self.blocks.intern(block);
                THRLoweredStatement::Multiple(block)
            }
            super::immediate_parsed_representation::IPRStatementKind::Reassignment(
                _iprreassignment,
            ) => todo!(),
            super::immediate_parsed_representation::IPRStatementKind::FunctionCall(
                _iprfunction_call,
            ) => todo!(),
            super::immediate_parsed_representation::IPRStatementKind::Return(_iprexpression) => {
                todo!()
            }
            super::immediate_parsed_representation::IPRStatementKind::BlockTail(_iprexpression) => {
                todo!()
            }
            super::immediate_parsed_representation::IPRStatementKind::Block(
                _iprblock_expression,
            ) => todo!(),
            super::immediate_parsed_representation::IPRStatementKind::IfThenElse(_iprbranch) => {
                todo!()
            }
        }
    }

    fn lower_init_block(
        &mut self,
        ipr_ctx: &IPRLoweringContext,
        init_block: &IPRInitializationBlock,
        kind: SymbolKind,
    ) -> Vec<THRStatementID> {
        let IPRInitializationKind::Unpacked(inits) = &init_block.kind else {
            unreachable!()
        };
        inits
            .iter()
            .map(|init| self.lower_init(ipr_ctx, init, kind))
            .collect()
    }

    fn lower_init(
        &mut self,
        ipr_ctx: &IPRLoweringContext,
        init: &IPRSimpleInitialization,
        kind: SymbolKind,
    ) -> THRStatementID {
        let symbol = THRSymbol {
            name: init.assignee.clone(),
            kind,
        };
        let symbol = self.symbols.intern(symbol);
        let value = self.lower_expression(ipr_ctx, &init.value);
        let typ = init.typ.as_ref().unwrap();
        let typ = self.lower_type(typ, ipr_ctx);

        let init = THRInit { symbol, typ, value };
        self.statements.intern(THRStatement::Init(init))
    }

    fn lower_module(mut self, ipr_ctx: &IPRLoweringContext) -> THRModule {
        let globs = &ipr_ctx.module.global_vars;
        let mut global_block = vec![];
        for glob in globs.iter() {
            let ids = self.lower_init_block(ipr_ctx, glob, SymbolKind::GlobalVar);
            global_block.extend(ids);
        }
        let global_block = THRBlock {
            items: global_block,
        };
        let global_data = self.blocks.intern(global_block);
        THRModule {
            interned: self,
            global_data,
        }
    }
}

struct IPRLoweringContext {
    module: IPRModule,
    types: IPRModuleTypeInfo,
    ident_scopes: IdentifierScoper,
    cf_stack: Vec<THRBlockID>,
}

impl std::fmt::Debug for THRInternTables {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&"TYPES:\n\t", f)?;
        for (i, t) in self.types.set.iter().enumerate() {
            i.fmt(f)?;
            std::fmt::Display::fmt(&"\t", f)?;
            t.fmt(f)?;
            std::fmt::Display::fmt(&"\n\t", f)?;
        }

        std::fmt::Display::fmt(&"\rSYMBOLS:\n\t", f)?;
        for (i, s) in self.symbols.set.iter().enumerate() {
            i.fmt(f)?;
            std::fmt::Display::fmt(&"\t", f)?;
            s.fmt(f)?;
            std::fmt::Display::fmt(&"\n\t", f)?;
        }
        std::fmt::Display::fmt(&"\rEXPRESSIONS:\n\t", f)?;
        for (i, s) in self.expressions.set.iter().enumerate() {
            i.fmt(f)?;
            std::fmt::Display::fmt(&"\t", f)?;
            s.fmt(f)?;
            std::fmt::Display::fmt(&"\n\t", f)?;
        }
        std::fmt::Display::fmt(&"\rSTATEMENTS:\n\t", f)?;
        for (i, s) in self.statements.set.iter().enumerate() {
            i.fmt(f)?;
            std::fmt::Display::fmt(&"\t", f)?;
            s.fmt(f)?;
            std::fmt::Display::fmt(&"\n\t", f)?;
        }
        std::fmt::Display::fmt(&"\rBLOCKS:\n\t", f)?;
        for (i, s) in self.blocks.set.iter().enumerate() {
            i.fmt(f)?;
            std::fmt::Display::fmt(&"\t", f)?;
            s.fmt(f)?;
            std::fmt::Display::fmt(&"\n\t", f)?;
        }
        "\n".fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn struct_field(ctx: &mut THRInternTables, typ: THRTypeSpecifier) -> THRStructField {
        THRStructField {
            name: "".to_string(),
            typ: ctx.types.intern(typ),
            offset: 0,
        }
    }
    fn struct_field_i8(ctx: &mut THRInternTables) -> THRStructField {
        struct_field(ctx, THRTypeSpecifier::i_signed(IntegerWidth::_8))
    }
    fn struct_field_i32(ctx: &mut THRInternTables) -> THRStructField {
        struct_field(ctx, THRTypeSpecifier::i_signed(IntegerWidth::_32))
    }
    fn struct_field_i64(ctx: &mut THRInternTables) -> THRStructField {
        struct_field(ctx, THRTypeSpecifier::i_signed(IntegerWidth::_64))
    }
    fn struct_field_i16(ctx: &mut THRInternTables) -> THRStructField {
        struct_field(ctx, THRTypeSpecifier::i_signed(IntegerWidth::_16))
    }

    fn struct_size_invariant(layout: &THRStructLayout, ctx: &THRInternTables) {
        let s = layout.width(ctx);
        let a = layout.alignment(ctx);
        assert_eq!(s % a, 0, "size {s} should be a multiple of alignment{a}");
    }
    fn run_layout_cases(
        cases: Vec<(StructOrdering, Vec<THRStructField>, usize, usize)>,
        ctx: &THRInternTables,
    ) {
        for (ordering, fields, size, align) in cases {
            let s = THRStructLayout { ordering, fields };

            assert_eq!(s.width(ctx), size);
            assert_eq!(s.alignment(ctx), align);
            struct_size_invariant(&s, ctx);
        }
    }
    #[test]
    fn struct_width() {
        let mut ctx = THRInternTables::with_integer_types();
        let i8 = struct_field_i8(&mut ctx);
        let i16 = struct_field_i16(&mut ctx);
        let i32 = struct_field_i32(&mut ctx);
        let i64 = struct_field_i64(&mut ctx);

        let basic_cases = [
            (StructOrdering::Naive, vec![i8.clone()], 1usize, 1usize),
            (StructOrdering::Naive, vec![i8.clone(), i8.clone()], 2, 1),
            (
                StructOrdering::Naive,
                vec![i8.clone(), i8.clone(), i8.clone(), i8.clone()],
                4,
                1,
            ),
            (StructOrdering::Naive, vec![i16.clone()], 2, 2),
            (StructOrdering::Naive, vec![i16.clone(), i16.clone()], 4, 2),
            (
                StructOrdering::Naive,
                vec![i16.clone(), i16.clone(), i16.clone(), i16.clone()],
                8,
                2,
            ),
            (StructOrdering::Naive, vec![i32.clone()], 4, 4),
            (StructOrdering::Naive, vec![i32.clone(), i32.clone()], 8, 4),
            (
                StructOrdering::Naive,
                vec![i32.clone(), i32.clone(), i32.clone(), i32.clone()],
                16,
                4,
            ),
            (StructOrdering::Naive, vec![i64.clone()], 8, 8),
            (StructOrdering::Naive, vec![i64.clone(), i64.clone()], 16, 8),
            (
                StructOrdering::Naive,
                vec![i64.clone(), i64.clone(), i64.clone(), i64.clone()],
                32,
                8,
            ),
        ];

        let mixed_fields = [
            (
                StructOrdering::Naive,
                vec![i16.clone(), i8.clone()],
                4usize,
                2usize,
            ),
            (StructOrdering::Naive, vec![i32.clone(), i8.clone()], 8, 4),
            (
                StructOrdering::Naive,
                vec![i32.clone(), i8.clone(), i32.clone()],
                12,
                4,
            ),
            (StructOrdering::Naive, vec![i64.clone(), i8.clone()], 16, 8),
            (StructOrdering::Naive, vec![i64.clone(), i8.clone()], 16, 8),
            (
                StructOrdering::Naive,
                vec![i64.clone(), i8.clone(), i64.clone()],
                24,
                8,
            ),
        ];

        run_layout_cases(basic_cases.to_vec(), &ctx);
        run_layout_cases(mixed_fields.to_vec(), &ctx);
    }
}
