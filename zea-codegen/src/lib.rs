use log::trace;
use qbe::{self as Q};
use zea_internal_macros::InternKey;
use zea_ipr::{
    InternTable,
    zea::{
        BinOp,
        thr::{
            FloatWidth, IntegerWidth, THRBlock, THRExpression, THRFunction, THRModule,
            THRStatement, THRSymbol, THRTypeSpecifier,
        },
    },
};
#[derive(Copy, Clone, Debug, PartialEq, Eq, InternKey)]
pub struct QBETypeID(u32);

pub struct THRtoQBE<'m> {
    thr_module: &'m THRModule,
    types: InternTable<QBETypeID, Q::Type>,
    temp_generator: usize,
}
#[allow(unused)]
struct BlockContext<'b> {
    m: &'b mut Q::Module,
    sig: &'b mut Q::Function,
}

impl<'b> BlockContext<'b> {
    fn new(m: &'b mut Q::Module, sig: &'b mut Q::Function) -> Self {
        Self { m, sig }
    }
}

impl<'m> THRtoQBE<'m> {
    pub fn new(module: &'m THRModule) -> Self {
        Self {
            thr_module: module,
            types: InternTable::new(),
            temp_generator: 0,
        }
    }

    pub fn lower(&mut self) -> Q::Module {
        let mut m = Q::Module::new();
        self.walk_global_data_blocks(&mut m);
        for func in self.thr_module.functions().iter() {
            self.emit_function(&mut m, func);
        }
        m
    }

    pub fn walk_global_data_blocks(&mut self, module: &mut Q::Module) {
        let glob_data = self
            .thr_module
            .get_block(self.thr_module.global_data_block());
        for stmt in glob_data.items.iter() {
            let stmt = self.thr_module.get_statement(*stmt);
            self.emit_datadef(module, stmt);
        }
    }
    /// recursively emit the instructions necessary to represent the given statement
    fn emit_stmt(&mut self, bctx: &mut BlockContext, stmt: &THRStatement) {
        match stmt {
            THRStatement::Init(symbdecl, expr) => {
                let symb = self.thr_module.get_symbol(symbdecl.symbol);
                let typ = self.thr_module.get_type(symbdecl.typ);
                let value = self.thr_module.get_expr(*expr);
                self.emit_stmt_init(bctx, symb, typ, value);
            }
            THRStatement::Jmp(_thrblock_id) => {
                todo!()
            }
            THRStatement::Ret(e) => {
                let expr = self.thr_module.get_expr(*e);
                let temp = self.emit_expr(bctx, expr);
                let instr = Q::Instr::Ret(Some(temp));
                bctx.sig.add_instr(instr);
            }
            THRStatement::SegmentedAssign(_thrsymbol_decl, _thrblock_ids) => {
                todo!()
            }
            THRStatement::SegmentReturn(_thrsymbol_id, _threxpr_id) => todo!(),
        }
    }
    fn emit_stmt_init(
        &mut self,
        bctx: &mut BlockContext,
        symb: &THRSymbol,
        typ: &THRTypeSpecifier,
        value: &THRExpression,
    ) -> Option<Q::Value> {
        let q_val = self.emit_expr(bctx, value);
        // an expression with unit-type cannot be stored, as it holds no data.
        // it may however have side effects, so it must still be emitted
        match typ {
            THRTypeSpecifier::Unit | THRTypeSpecifier::Never => {
                bctx.sig.add_instr(Q::Instr::Copy(q_val));
                None
            }
            _ => {
                let q_typ = self.emit_lvalue_type_and_get(typ);
                let temp = self.fresh_temporary(&symb.name);
                bctx.sig
                    .assign_instr(temp.clone(), q_typ, Q::Instr::Copy(q_val));
                Some(temp)
            }
        }
    }
    /// Recursively emit instructions necessary to compute the given expression, then return the temporary it is saved to.
    fn emit_expr(&mut self, _bctx: &mut BlockContext, expr: &THRExpression) -> Q::Value {
        match expr {
            THRExpression::ConstInt(lit_i) => Q::Value::Const(lit_i.value),
            THRExpression::ConstFloat(lit_f) => Q::Value::Const(lit_f.value.to_bits()),
            THRExpression::ConstBool(b) => Q::Value::Const(*b as u64),
            THRExpression::Binop(_op, _l, _r) => {
                let l = self.thr_module.get_expr(*_l);
                let r = self.thr_module.get_expr(*_r);
                self.emit_expr_binop(_bctx, *_op, l, r)
            }
            THRExpression::Unop(..) => todo!(),
            THRExpression::Ident(_) => todo!(),
        }
    }
    /// Recursively emit instructions necessary to compute the given binary expression, then return the temporary it is saved to.
    fn emit_expr_binop(
        &mut self,
        bctx: &mut BlockContext,
        op: BinOp,
        l: &THRExpression,
        r: &THRExpression,
    ) -> Q::Value {
        let _l = self.emit_expr(bctx, l);
        let _r = self.emit_expr(bctx, r);
        match op {
            BinOp::Add => todo!(),
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

    fn emit_return_type(&mut self, typ: &THRTypeSpecifier) -> Option<QBETypeID> {
        match typ {
            THRTypeSpecifier::Integer { width, signed } => {
                Some(self.emit_type_integer(*width, *signed))
            }
            THRTypeSpecifier::Float { width } => Some(self.emit_type_float(*width)),
            THRTypeSpecifier::Pointer(_t) => Some(self.types.intern(Q::Type::Long)),
            THRTypeSpecifier::Boolean => Some(self.types.intern(Q::Type::Word)),
            THRTypeSpecifier::Unit => None,
            THRTypeSpecifier::Never => None,
            THRTypeSpecifier::Struct { .. } => todo!(),
            THRTypeSpecifier::Tuple(_) => todo!(),
        }
    }
    fn emit_return_type_and_get(&mut self, return_ty: &THRTypeSpecifier) -> Option<&Q::Type> {
        let id = self.emit_return_type(return_ty)?;
        self.types.get_by_id(id)
    }

    fn emit_lvalue_type(&mut self, typ: &THRTypeSpecifier) -> QBETypeID {
        match typ {
            THRTypeSpecifier::Integer { width, signed } => self.emit_type_integer(*width, *signed),
            THRTypeSpecifier::Float { width } => self.emit_type_float(*width),
            THRTypeSpecifier::Pointer(_t) => self.types.intern(Q::Type::Long),
            THRTypeSpecifier::Boolean => self.types.intern(Q::Type::Word),
            THRTypeSpecifier::Unit => todo!(),
            THRTypeSpecifier::Never => todo!(),
            THRTypeSpecifier::Struct {
                name: _,
                layout: _layout,
            } => todo!(),
            THRTypeSpecifier::Tuple(_thrstruct_layout) => todo!(),
        }
    }

    fn emit_lvalue_type_and_get(&mut self, typ: &THRTypeSpecifier) -> Q::Type {
        let t = self.emit_lvalue_type(typ);
        self.types.get_by_id(t).unwrap().clone()
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

    fn fresh_temporary(&mut self, name: &str) -> Q::Value {
        let temp = Q::Value::Temporary(format!("_{}_{name}", self.temp_generator));
        self.temp_generator += 1;
        temp
    }
    #[allow(unused)]
    fn fresh_prelude_block(&mut self) -> Q::Block {
        let label = format!("_{}_predule", self.temp_generator);
        self.temp_generator += 1;
        Q::Block {
            label,
            items: vec![],
        }
    }

    fn emit_datadef(&mut self, module: &mut qbe::Module, stmt: &THRStatement) {
        let THRStatement::Init(decl, val) = stmt else {
            unreachable!("global statement must be an init");
        };
        let name = self.thr_module.get_symbol(decl.symbol).name.clone();
        let thr_typ = self.thr_module.get_type(decl.typ);
        let qbe_typ_id = self.emit_lvalue_type(thr_typ);
        let align = self.thr_module.alignment_of(decl.typ);
        let qbe_typ = self
            .types
            .get_by_id(qbe_typ_id)
            .cloned()
            .expect("emit_type should have interned the supplied typ");
        let items = match self.thr_module.get_expr(*val) {
            THRExpression::ConstInt(i) => {
                let item = Q::DataItem::Const(i.value);
                vec![(qbe_typ, item)]
            }
            THRExpression::ConstFloat(_) => todo!(),
            THRExpression::ConstBool(b) => {
                let item = Q::DataItem::Const(*b as u64);
                vec![(qbe_typ, item)]
            }
            THRExpression::Binop(..) => todo!(),
            THRExpression::Unop(..) => todo!(),
            THRExpression::Ident(..) => todo!(),
        };
        let d = Q::DataDef::new(Q::Linkage::public(), name, Some(align), items);
        module.add_data(d);
    }

    fn emit_function(&mut self, m: &mut Q::Module, func: &THRFunction) {
        trace!("walking function `{}`", func.name);
        let mut qbe_func = self.build_function_signature(func);
        let body = self.thr_module.get_block(func.body);
        let _ = qbe_func.add_block(&func.name);
        let mut bctx = BlockContext::new(m, &mut qbe_func);
        self.emit_block(&mut bctx, body);
        m.add_function(qbe_func);
    }
    fn build_function_signature(&mut self, func: &THRFunction) -> Q::Function {
        let return_ty = self.thr_module.get_type(func.ret);
        let return_ty = self.emit_return_type_and_get(return_ty).cloned();
        let mut arguments = vec![];
        for param in func.params.iter() {
            let typ = self.thr_module.get_type(param.typ);
            let typ = self.emit_lvalue_type_and_get(typ);
            let name = self.thr_module.get_symbol(param.symbol).name.as_str();
            let name = self.fresh_temporary(name);
            arguments.push((typ, name));
        }
        Q::Function::new(
            Q::Linkage::public(),
            func.name.clone(),
            arguments,
            return_ty,
        )
    }

    fn emit_block(&mut self, bctx: &mut BlockContext, body: &THRBlock) {
        for stmt in body.items.iter().copied() {
            let stmt = self.thr_module.get_statement(stmt);
            trace!("emitting block statement `{stmt:?}`");
            self.emit_stmt(bctx, stmt);
        }
    }
}
