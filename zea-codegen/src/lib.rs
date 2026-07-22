use std::{rc::Rc, sync::Arc};

use indexmap::IndexSet;
use qbe::{self as Q, Instr, Value};
use zea_internal_macros::InternKey;
use zea_ipr::{
    zea::{
        typed_highlevel_representation::{FloatWidth, IntegerWidth},
        BinOp, THRExpression, THRModule, THRStatement, THRStructLayout, THRSymbol,
        THRTypeSpecifier,
    },
    InternTable,
};
#[derive(Copy, Clone, Debug, PartialEq, Eq, InternKey)]
pub struct QBETypeID(u32);

pub struct THRtoQBE<'m> {
    thr_module: &THRModule,
    qbe_module: Q::Module,
    types: InternTable<QBETypeID, Q::Type>,
    temp_generator: usize,
}
enum LoweredInit {
    Sized(Q::Value, QBETypeID, Q::Instr),
    Unit(Q::Instr),
}


impl<'m> THRtoQBE<'m> {
    pub fn for_module(module: &'m THRModule) -> Self {
        Self {
            thr_module: module,
            qbe_module: Q::Module::new(),
            types: InternTable::new(),
            temp_generator: 0,
        }
    }
    pub fn traverse_global_data_blocks(&mut self) {
        let glob_data = self
            .thr_module
            .blocks()
            .get_by_id(self.thr_module.global_data_block())
            .expect("missing global data block");
        for stmt in glob_data.items {
            let stmt = self.thr_module.get_statement(stmt);
            self.emit_stmt(stmt);
        }
    }

    fn emit_stmt(&mut self, stmt: &THRStatement) {
        match stmt {
            THRStatement::Init(symbdecl, expr) => {
                let symb = self.thr_module.get_symbol(symbdecl.symbol);
                let typ = self.thr_module.get_type(symbdecl.typ);
                let value = self.thr_module.get_expr(expr);
                self.emit_stmt_init(symb, typ, value);
            }
            THRStatement::Jmp(thrblock_id) => todo!(),
            THRStatement::Ret(threxpr_id) => todo!(),
            THRStatement::SegmentedAssign(thrsymbol_decl, thrblock_ids) => todo!(),
            THRStatement::SegmentReturn(thrsymbol_id, threxpr_id) => todo!(),
        }
    }
    fn emit_stmt_init_sized(
        &mut self,
        symb: &THRSymbol,
        typ: &THRTypeSpecifier,
        value: &THRExpression,
    ) -> LoweredInit {
        let q_val = self.emit_expr(value);
        // an expression with unit-type cannot be stored, as it holds no data.
        // it may have side effects, so it must still be emitted
        match typ {
            THRTypeSpecifier::Unit | THRTypeSpecifier::Never => LoweredInit::Unit(q_val),
            _ => {
                let q_typ = self.emit_type(typ);
                let temp = self.emit_temporary(&symb.name);
                LoweredInit::Sized(temp, q_typ, q_val)
            }
        }
    }

    fn emit_expr(&mut self, expr: &THRExpression) -> Q::Instr {
        match expr {
            THRExpression::ConstInt(lit_i) => Q::Instr::Copy(Q::Value::Const(lit_i.value)),
            THRExpression::ConstFloat(lit_f) => {
                Q::Instr::Copy(Q::Value::Const(lit_f.value.to_bits()))
            }
            THRExpression::ConstBool(b) => Q::Instr::Copy(Q::Value::Const(*b as u64)),
            THRExpression::Binop(op, l, r) => {
                let l = self.thr_module.get_expr(*l);
                let r = self.thr_module.get_expr(*r);
                self.emit_expr_binop(*op, l, r)
            }
            THRExpression::Unop(un_op, threxpr_id) => todo!(),
            THRExpression::Ident(thrsymbol_id) => todo!(),
        }
    }
    fn emit_expr_binop(&mut self, op: BinOp, l: &THRExpression, r: &THRExpression) -> Q::Instr {
        let l = self.emit_expr(l);
        let r = self.emit_expr(r);
        match op {
            BinOp::Add => ,
            BinOp::Sub => todo!(),
            BinOp::Mul => todo!(),
            BinOp::Div => todo!(),
            BinOp::Mod => todo!(),
            BinOp::LogAnd => todo!(),
            BinOp::LogOr => todo!(),
            BinOp::LogXor => todo!(),
            BinOp::BitAnd => todo!(),
            BinOp::BitOr => todo!(),
            BinOp::BitXor => todo!(),
            BinOp::Subscript => todo!(),
            BinOp::Lsh => todo!(),
            BinOp::Rsh => todo!(),
            BinOp::Eq => todo!(),
            BinOp::Neq => todo!(),
            BinOp::Geq => todo!(),
            BinOp::Leq => todo!(),
            BinOp::LT => todo!(),
            BinOp::GT => todo!(),
        }
    }
    

    fn emit_type(&mut self, typ: &THRTypeSpecifier) -> QBETypeID {
        match typ {
            THRTypeSpecifier::Integer { width, signed } => self.emit_type_integer(*width, *signed),
            THRTypeSpecifier::Float { width } => self.emit_type_float(*width),
            THRTypeSpecifier::Pointer(t) => self.types.intern(Q::Type::Long),
            THRTypeSpecifier::Boolean => self.types.intern(Q::Type::Word),
            THRTypeSpecifier::Unit => todo!(),
            THRTypeSpecifier::Never => todo!(),
            THRTypeSpecifier::Struct { name, layout } => todo!(),
            THRTypeSpecifier::Tuple(thrstruct_layout) => todo!(),
        }
    }
    fn emit_type_integer(&mut self, width: IntegerWidth, signed: bool) -> QBETypeID {
        let t = match (width, signed) {
            (IntegerWidth::_8, true) => Q::Type::SignedByte,
            (IntegerWidth::_8, false) => Q::Type::UnsignedByte,
            (IntegerWidth::_16, true) => Q::Type::SignedHalfword,
            (IntegerWidth::_16, false) => Q::Type::UnsignedHalfword,
            (IntegerWidth::_32, true) => Q::Type::Word,
            (IntegerWidth::_32, false) => Q::Type::Word,
            (IntegerWidth::_64, true) => Q::Type::Long,
            (IntegerWidth::_64, false) => Q::Type::Long,
        };
        self.types.intern(t)
    }

    fn emit_type_float(&mut self, width: FloatWidth) -> QBETypeID {
        let t = match width {
            FloatWidth::_32 => Q::Type::Single,
            FloatWidth::_64 => Q::Type::Double,
        };
        self.types.intern(t)
    }
    fn emit_type_aggregate(&mut self, name: &str, layout: THRStructLayout) -> QBETypeID {
        let mut items = vec![];
        for field in layout.fields.iter() {
            let typ = self.thr_module.get_type(field.typ);
            let typ = self.emit_type(typ);
            items.push((typ, 1));
        }
        let align = Some(self.thr_module.alignment_of(&layout) as u64);

        let aggregate = Q::Type::Aggregate(Arc::new(Q::TypeDef::Regular {
            ident: name.to_string(),
            align,
            items,
        }));
        self.types.intern(aggregate)
    }

    fn emit_temporary(&mut self, name: &str) -> Q::Value {
        let temp = Q::Value::Temporary(format!("_{}_{name}", self.temp_generator));
        self.temp_generator += 1;
        temp
    }
}
