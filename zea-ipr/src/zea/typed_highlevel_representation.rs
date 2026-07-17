use std::io::IntoInnerError;

use indexmap::Equivalent;
use zea_internal_macros::{InternKey, VariantToStr};

use crate::{
    zea::{
        immediate_parsed_representation::{IPRModule, IPRTypeSpecifier, IPRTypedIdentifier},
        BinOp, UnOp,
    },
    InternTable,
};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct THRModule {
    interned: THRInternTables,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct THRInternTables {
    expressions: InternTable<THRExprID, THRExpression>,
    symbols: InternTable<THRSymbolID, THRSymbol>,
    types: InternTable<THRTypeID, THRTypeSpecifier>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntegerWidth {
    _8 = 8,
    _16 = 16,
    _32 = 32,
    _64 = 64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FloatWidth {
    _32 = 32,
    _64 = 64,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct THRStructLayout {
    ordering: StructOrdering,
    fields: Vec<THRStructField>,
}
impl THRStructLayout {
    fn alignment(&self, ctx: &THRInternTables) -> usize {
        self.fields
            .iter()
            .map(|field| field.alignment)
            .max()
            .unwrap_or(0)
    }

    fn width(&self, ctx: &THRInternTables) -> usize {
        let mut offset = 0;
        let mut last_field_width = 0;
        for THRStructField {
            alignment, width, ..
        } in self.fields.iter()
        {
            offset += *width;
            let next_align = Self::next_aligned_offset(offset, *alignment);
            last_field_width = *width;
            offset = next_align;
        }

        offset + last_field_width
    }

    /// calculate the closest multiple `m` of `required_alignment` s.t. `m >= cur_offset` && `m % required_alignment == 0`
    fn next_aligned_offset(cur_offset: usize, required_alignment: usize) -> usize {
        let mut res = 0;
        while res < cur_offset {
            res += required_alignment;
        }
        res
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum StructOrdering {
    Naive,
    Descending,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct THRStructField {
    name: String,
    typ: THRTypeID,
    width: usize,
    alignment: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum THRExpression {
    ConstInt(TypedLiteral<u64>),
    ConstFloat(TypedLiteral<f64>),
    Binop(BinOp, THRExprID, THRExprID),
    Unop(UnOp, THRExprID),
    Ident(THRSymbolID),
}

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

#[derive(Clone, Debug, PartialEq, Eq, Hash, VariantToStr)]
pub enum THRStatement {
    Init(THRInit),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct THRInit {
    assignee: THRSymbolID,
    typ: THRTypeID,
    value: THRExprID,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct THRSymbol {
    name: String,
    kind: THRSymbolKind,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum THRSymbolKind {
    Global,
    Local,
    FuncName,
    FuncParam,
}

impl THRInternTables {
    pub fn new(
        types: InternTable<THRTypeID, THRTypeSpecifier>,
        symbols: InternTable<THRSymbolID, THRSymbol>,
        expressions: InternTable<THRExprID, THRExpression>,
    ) -> Self {
        Self {
            types,
            symbols,
            expressions,
        }
    }

    pub fn lower_type(&mut self, iprtype: &IPRTypeSpecifier, ctx: &IPRModule) -> THRTypeID {
        let t = match iprtype {
            IPRTypeSpecifier::NonScalar(a) => {
                let s = ctx
                    .struct_definitions
                    .iter()
                    .find(|s| s.name == *a)
                    .expect("type checking should resolve struct names");
                for IPRTypedIdentifier { name, typ } in s.members.iter() {
                    let typ = self.lower_type(typ, ctx);
                    let field = THRStructField {
                        name: name.clone(),
                        typ,
                        width: typ.width(self),
                        alignment: typ.alignment(self),
                    };
                }

                todo!()
            }
            IPRTypeSpecifier::Unit => THRTypeSpecifier::Unit,
            IPRTypeSpecifier::Bool => THRTypeSpecifier::Boolean,
            IPRTypeSpecifier::Integer {
                width: _,
                signed: _,
            } => todo!(),
            IPRTypeSpecifier::Float { width: _ } => todo!(),
            IPRTypeSpecifier::Pointer(inner) => {
                let inner_id = self.lower_type(inner, ctx);
                THRTypeSpecifier::Pointer(inner_id)
            }
            IPRTypeSpecifier::ArrayOf(_) => todo!(),
            IPRTypeSpecifier::Never => THRTypeSpecifier::Never,
        };
        self.types.intern(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn struct_field(width: usize, alignment: usize) -> THRStructField {
        THRStructField {
            name: "".to_string(),
            typ: THRTypeID(0),
            width,
            alignment,
        }
    }
    fn struct_field_i8() -> THRStructField {
        struct_field(1, 1)
    }
    fn struct_field_i32() -> THRStructField {
        struct_field(4, 4)
    }
    fn struct_field_i64() -> THRStructField {
        struct_field(8, 8)
    }
    fn struct_field_i16() -> THRStructField {
        struct_field(2, 2)
    }
    #[test]
    fn struct_width() {
        let ctx = &THRInternTables::default();
        let s = THRStructLayout {
            ordering: StructOrdering::Naive,
            fields: vec![struct_field_i8(), struct_field_i8()],
        };

        assert_eq!(s.width(ctx), 2);
        assert_eq!(s.alignment(ctx), 1);

        let s = THRStructLayout {
            ordering: StructOrdering::Naive,
            fields: vec![struct_field_i32(), struct_field_i8()],
        };
        assert_eq!(s.width(ctx), 5);
        assert_eq!(s.alignment(ctx), 4);

        let s = THRStructLayout {
            ordering: StructOrdering::Naive,
            fields: vec![struct_field_i8(), struct_field_i32()],
        };
        assert_eq!(s.width(ctx), 8);
        assert_eq!(s.alignment(ctx), 4);
    }
}
