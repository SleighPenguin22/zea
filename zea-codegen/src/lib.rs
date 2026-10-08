use std::collections::HashMap;

use idset::{KeySet, internkey};
use log::{error, trace};
use qbe::{self as Q};
use zea_common::CompilerError;
use zea_irs::{
    ScalarTypeWidth,
    ast::{
        BinOp, UnOp,
        ipr_walkers::visitors::SymbolKind,
        thr::{
            THRExprID, THRExpression, THRFunction, THRModule, THRStatement, THRStatementID,
            THRSymbol, THRSymbolDecl, THRSymbolID, THRTypeID, THRTypeSpecifier, TypedLiteral,
        },
    },
};
internkey!(QBETypeID);

#[derive(Debug)]
pub struct THRtoQBE<'m> {
    thr_module: &'m THRModule,
    types: KeySet<QBETypeID, Q::Type>,
    symbol_table: HashMap<THRSymbolID, Q::Value>,
    temp_generator: usize,
    block_label_generator: usize,
}

/// The context necessary to construct QBE nodes for blocks within a function body
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
            types: KeySet::new(),
            symbol_table: HashMap::with_capacity(64),
            temp_generator: 0,
            block_label_generator: 0,
        }
    }
    pub fn lower(&mut self) -> Result<Q::Module, CompilerError> {
        let mut m = Q::Module::new();
        self.walk_global_data_blocks(&mut m)?;
        for func in self.thr_module.functions().iter() {
            self.emit_function(&mut m, func);
        }
        Ok(m)
    }
    /// Walk the global data block of a THR module
    fn walk_global_data_blocks(&mut self, module: &mut Q::Module) -> Result<(), CompilerError> {
        let glob_data = self.thr_module.global_data_block();
        for stmt in glob_data.iter().copied() {
            let stmt = self.thr_module.get_statement(stmt);
            let THRStatement::Init { decl, val, .. } = *stmt else {
                return Err(CompilerError::new(
                    zea_common::CompilerStage::CodeGen,
                    zea_common::CompilerErrorKind::GlobalNonInitStmt,
                ));
            };
            let d = self.prepare_datadef(module, decl, val)?;
            module.add_data(d);
        }
        Ok(())
    }
    /// recursively emit the instructions necessary to represent the given statement
    fn emit_stmt(&mut self, bctx: &mut BlockContext, stmt: &THRStatement) {
        match stmt {
            THRStatement::Init {
                decl: symbdecl,
                val: expr,
                ..
            } => {
                let symb = symbdecl.symbol;
                let value = *expr;
                self.emit_stmt_init(bctx, symb, symbdecl.typ, value);
            }
            THRStatement::Ret(e) => {
                let expr = self.thr_module.get_expr(*e);
                let res_t = self
                    .thr_module
                    .get_type_of_expr(*e)
                    .expect("missing THR expr type");
                let temp = self.emit_expr(bctx, expr, res_t);
                let instr = Q::Instr::Ret(Some(temp));
                bctx.sig.add_instr(instr);
            }
            THRStatement::Branch { cond, bthen, belse } => todo!(),
        }
    }
    /// Given all components of an initialization, construct the QBE nodes necessary for it.
    fn emit_stmt_init(
        &mut self,
        bctx: &mut BlockContext,
        symb_id: THRSymbolID,
        typ: THRTypeID,
        value_id: THRExprID,
    ) -> Option<Q::Value> {
        let symb = self.thr_module.get_symbol(symb_id);
        let value = self.thr_module.get_expr(value_id);
        let val_qbe = self.emit_expr(bctx, value, typ);
        // an expression with unit-type cannot be stored, as it holds no data.
        // it may however have side effects, so it must still be emitted
        //
        let typ = self.thr_module.get_type(typ);
        match typ {
            THRTypeSpecifier::Unit | THRTypeSpecifier::Never => {
                bctx.sig.add_instr(Q::Instr::Copy(val_qbe));
                None
            }
            _ => {
                let q_typ = self.emit_lvalue_type_and_get(typ);
                let temp = Q::Value::Temporary(self.disambiguate_symbol_local_usage(bctx, symb));
                bctx.sig
                    .assign_instr(temp.clone(), q_typ, Q::Instr::Copy(val_qbe));
                self.symbol_table.insert(symb_id, temp.clone());
                Some(temp)
            }
        }
    }
    /// Recursively emit instructions necessary to compute the given expression, then return the temporary it is saved to.
    fn emit_expr(
        &mut self,
        bctx: &mut BlockContext,
        expr: &THRExpression,
        resulting_type: THRTypeID,
    ) -> Q::Value {
        match expr {
            THRExpression::ConstInt(TypedLiteral { value, .. }) => Q::Value::Const(*value),
            THRExpression::ConstFloat(TypedLiteral { value, .. }) => {
                Q::Value::Const((*value).to_bits())
            }
            THRExpression::ConstBool(b) => Q::Value::Const(*b as u64),
            THRExpression::Binop(_op, l, r) => {
                self.emit_expr_binop(bctx, *_op, *l, *r, resulting_type)
            }
            THRExpression::Unop(..) => todo!(),
            THRExpression::Ident(i) => {
                let symbol = self.thr_module.get_symbol(*i);
                let disamb = self.disambiguate_symbol_local_usage(bctx, symbol);
                match symbol.kind {
                    SymbolKind::LocalVar | SymbolKind::FunctionParam => Q::Value::Temporary(disamb),
                    SymbolKind::GlobalVar => Q::Value::Global(disamb),
                    SymbolKind::FunctionName => todo!(),
                    SymbolKind::StructName => todo!(),
                }
            }
        }
    }
    /// Recursively emit instructions necessary to compute the given binary expression,
    /// then return the temporary it is saved to.
    fn emit_expr_binop(
        &mut self,
        bctx: &mut BlockContext,
        op: BinOp,
        l: THRExprID,
        r: THRExprID,
        resulting_type: THRTypeID,
    ) -> Q::Value {
        let l_t = self
            .thr_module
            .get_type_of_expr(l)
            .expect("missing type for THR expr");
        let l_expr = self.thr_module.get_expr(l);
        let l_qbe = self.emit_expr(bctx, l_expr, l_t);

        let r_t = self
            .thr_module
            .get_type_of_expr(r)
            .expect("missing THR expr type");
        let r_expr = self.thr_module.get_expr(r);
        let r_qbe = self.emit_expr(bctx, r_expr, r_t);

        let res_type = self.thr_module.get_type(resulting_type);
        let res_t = self.emit_lvalue_type_and_get(res_type);
        match op {
            BinOp::Add => {
                let t = self.fresh_temporary();
                bctx.sig
                    .assign_instr(t.clone(), res_t, Q::Instr::Add(l_qbe, r_qbe));
                t
            }
            BinOp::Sub => {
                let t = self.fresh_temporary();
                bctx.sig
                    .assign_instr(t.clone(), res_t, Q::Instr::Sub(l_qbe, r_qbe));
                t
            }
            BinOp::Mul => {
                let t = self.fresh_temporary();
                bctx.sig
                    .assign_instr(t.clone(), res_t, Q::Instr::Mul(l_qbe, r_qbe));
                t
            }
            BinOp::Div => {
                let t = self.fresh_temporary();
                bctx.sig
                    .assign_instr(t.clone(), res_t, Q::Instr::Div(l_qbe, r_qbe));
                t
            }
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
            BinOp::Eq => {
                let t = self.fresh_temporary();
                bctx.sig.assign_instr(
                    t.clone(),
                    Q::Type::Word,
                    Q::Instr::Cmp(res_t.into_base(), Q::Cmp::Eq, l_qbe, r_qbe),
                );
                t
            }
            BinOp::Neq => {
                let t = self.fresh_temporary();
                bctx.sig.assign_instr(
                    t.clone(),
                    Q::Type::Word,
                    Q::Instr::Cmp(res_t.into_base(), Q::Cmp::Ne, l_qbe, r_qbe),
                );
                t
            }
            BinOp::Geq => {
                let t = self.fresh_temporary();
                bctx.sig.assign_instr(
                    t.clone(),
                    Q::Type::Word,
                    Q::Instr::Cmp(res_t.into_base(), Q::Cmp::Sge, l_qbe, r_qbe),
                );
                t
            }
            BinOp::Leq => {
                let t = self.fresh_temporary();
                bctx.sig.assign_instr(
                    t.clone(),
                    Q::Type::Word,
                    Q::Instr::Cmp(res_t.into_base(), Q::Cmp::Slt, l_qbe, r_qbe),
                );
                t
            }
            BinOp::LT => {
                let t = self.fresh_temporary();
                bctx.sig.assign_instr(
                    t.clone(),
                    Q::Type::Word,
                    Q::Instr::Cmp(res_t.into_base(), Q::Cmp::Slt, l_qbe, r_qbe),
                );
                t
            }
            BinOp::GT => {
                let t = self.fresh_temporary();
                bctx.sig.assign_instr(
                    t.clone(),
                    Q::Type::Word,
                    Q::Instr::Cmp(res_t.into_base(), Q::Cmp::Sgt, l_qbe, r_qbe),
                );
                t
            }
        }
    }

    fn emit_return_type(&mut self, typ: &THRTypeSpecifier) -> Option<QBETypeID> {
        match typ {
            THRTypeSpecifier::Integer { width, signed } => {
                Some(self.emit_basetype_integer(*width, *signed))
            }
            THRTypeSpecifier::Float { width } => Some(self.emit_type_float(*width)),
            THRTypeSpecifier::Pointer(_t) => Some(self.types.get_or_intern(Q::Type::Long)),
            THRTypeSpecifier::Boolean => Some(self.types.get_or_intern(Q::Type::UnsignedByte)),
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
            THRTypeSpecifier::Integer { width, signed } => {
                self.emit_basetype_integer(*width, *signed)
            }
            THRTypeSpecifier::Float { width } => self.emit_type_float(*width),
            THRTypeSpecifier::Pointer(_t) => self.types.get_or_intern(Q::Type::Long),
            THRTypeSpecifier::Boolean => self.types.get_or_intern(Q::Type::Byte),
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

    fn emit_basetype_integer(&mut self, width: ScalarTypeWidth, signed: bool) -> QBETypeID {
        let t = match (width, signed) {
            (ScalarTypeWidth::_8, true) => Q::Type::SignedByte,
            (ScalarTypeWidth::_8, false) => Q::Type::UnsignedByte,
            (ScalarTypeWidth::_16, true) => Q::Type::SignedHalfword,
            (ScalarTypeWidth::_16, false) => Q::Type::UnsignedHalfword,
            (ScalarTypeWidth::_32, true) => Q::Type::Word,
            (ScalarTypeWidth::_32, false) => Q::Type::Word,
            (ScalarTypeWidth::_64, true) => Q::Type::Long,
            (ScalarTypeWidth::_64, false) => Q::Type::Long,
        };
        self.types.get_or_intern(t)
    }

    fn emit_type_float(&mut self, width: ScalarTypeWidth) -> QBETypeID {
        let t = match width {
            ScalarTypeWidth::_32 => Q::Type::Single,
            ScalarTypeWidth::_64 => Q::Type::Double,
            _ => unreachable!("illegal float width"),
        };
        self.types.get_or_intern(t)
    }

    fn fresh_temporary_with_suffix(&mut self, name: &str) -> Q::Value {
        let temp = Q::Value::Temporary(format!("t{}_{name}", self.temp_generator));
        self.temp_generator += 1;
        temp
    }
    fn fresh_temporary(&mut self) -> Q::Value {
        let temp = Q::Value::Temporary(format!("t{}", self.temp_generator));
        self.temp_generator += 1;
        temp
    }
    fn fresh_blocklabel(&mut self, bctx: &BlockContext) -> String {
        let temp = format!(
            "{}B{}",
            self.get_module_func_prefix(bctx),
            self.block_label_generator
        );
        self.block_label_generator += 1;
        temp
    }

    fn get_module_func_prefix(&self, bctx: &BlockContext) -> String {
        format!("{}_{}_", self.thr_module.name, bctx.sig.name)
    }
    /// demangle a symbol that is used in a local context;
    /// Any usage of a symbol within a basic-block/function body
    fn disambiguate_symbol_local_usage(&self, bctx: &BlockContext, ident: &THRSymbol) -> String {
        let prefix = self.get_module_prefix();
        let demangle = match ident.kind {
            SymbolKind::LocalVar => &format!("{}_local_", bctx.sig.name),
            SymbolKind::GlobalVar => "global_",
            SymbolKind::FunctionName => "func_",
            SymbolKind::FunctionParam => &format!("{}_param_", bctx.sig.name),
            SymbolKind::StructName => todo!(),
        };

        format!("{prefix}{demangle}{}", ident.name)
    }

    /// demangle a symbol that is used in a global context, i.e.
    /// any usage of a global symbol within a global context.
    fn disambiguate_symbol_global_usage(&self, ident: &THRSymbol) -> Result<String, CompilerError> {
        let prefix = self.get_module_prefix();
        let demangle = match ident.kind {
            SymbolKind::GlobalVar => "global_",
            SymbolKind::FunctionName => "func_",
            _ => {
                return Err(CompilerError::new(
                    zea_common::CompilerStage::CodeGen,
                    zea_common::CompilerErrorKind::InvalidNonGlobalSymbolUsage,
                ));
            }
        };

        Ok(format!("{prefix}{demangle}{}", ident.name))
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

    fn prepare_datadef<'ctx, 'module: 'ctx>(
        &'ctx mut self,
        module: &'module mut qbe::Module,
        decl: THRSymbolDecl,
        val: THRExprID,
    ) -> Result<Q::DataDef, CompilerError> {
        let name = self.thr_module.get_symbol(decl.symbol);
        let name = self.disambiguate_symbol_global_usage(name)?;
        let thr_typ = self.thr_module.get_type(decl.typ);
        let qbe_typ_id = self.emit_lvalue_type(thr_typ);
        let align = self.thr_module.alignment_of(decl.typ);
        let qbe_typ = self
            .types
            .get_by_id(qbe_typ_id)
            .cloned()
            .expect("emit_type should have interned the supplied typ")
            .into_abi();
        let datalayout = self.prepare_datadef_layout(module, val, qbe_typ);
        let d = Q::DataDef::new(Q::Linkage::public(), name, Some(align as u64), datalayout);
        Ok(d)
    }

    fn prepare_datadef_layout<'ctx, 'module: 'ctx>(
        &'ctx mut self,
        module: &'module mut qbe::Module,
        val: THRExprID,
        qbe_typ: qbe::Type,
    ) -> Vec<(qbe::Type, qbe::DataItem)> {
        match self.thr_module.get_expr(val) {
            THRExpression::ConstInt(TypedLiteral { value, .. }) => {
                let item = Q::DataItem::Const(*value);
                vec![(qbe_typ, item)]
            }
            THRExpression::ConstFloat(TypedLiteral { value, .. }) => {
                let item = Q::DataItem::Const(value.to_bits());
                vec![(qbe_typ, item)]
            }
            THRExpression::ConstBool(b) => {
                let item = Q::DataItem::Const(*b as u64);
                vec![(qbe_typ, item)]
            }
            THRExpression::Binop(binop, id_a, id_b) => {
                match self.eval_literal_expression_u64_binop(*binop, *id_a, *id_b) {
                    Some(v) => {
                        let item = Q::DataItem::Const(v);
                        vec![(qbe_typ, item)]
                    }
                    None => todo!("value cannot be interpreted as u64"),
                }
            }
            THRExpression::Unop(op, id_arg) => {
                match self.eval_literal_expression_u64_unop(*op, *id_arg) {
                    Some(v) => {
                        let item = Q::DataItem::Const(v);
                        vec![(qbe_typ, item)]
                    }
                    None => todo!("value cannot be interpreted as u64"),
                }
            }
            THRExpression::Ident(symbol) => {
                let globs = self.thr_module.global_data_block();
                for glob in globs.iter().copied() {
                    let stmt = self.thr_module.get_statement(glob);
                    if let THRStatement::Init {
                        decl,
                        val: refererred_val,
                        ..
                    } = stmt
                        && (decl.symbol == *symbol && !self.symbol_table.contains_key(&decl.symbol))
                    {
                        // copy the data-representation, but rename it
                        return self.prepare_datadef_layout(module, *refererred_val, qbe_typ);
                    };
                }
                unreachable!()
            }
        }
    }

    fn emit_function(&mut self, m: &mut Q::Module, func: &THRFunction) {
        trace!("walking function `{}`", func.name);
        let mut qbe_func = self.build_function_signature(func);
        let body = &func.body;
        let mut bctx = BlockContext::new(m, &mut qbe_func);
        self.emit_block(&mut bctx, body, &func.name);
        m.add_function(qbe_func);
    }
    fn build_function_signature(&mut self, func: &THRFunction) -> Q::Function {
        let return_ty = self.thr_module.get_type(func.returns);
        let return_ty_qbe = self.emit_return_type_and_get(return_ty).cloned();
        let mut arguments = vec![];
        for param in func.params.iter() {
            let typ = self.thr_module.get_type(param.typ);
            let typ = self.emit_lvalue_type_and_get(typ);
            let name = self.thr_module.get_symbol(param.symbol).name.as_ref();
            let name = self.fresh_temporary_with_suffix(name);
            arguments.push((typ, name));
        }
        Q::Function::new(
            Q::Linkage::public(),
            func.name.clone(),
            arguments,
            return_ty_qbe,
        )
    }

    fn emit_block(&mut self, bctx: &mut BlockContext, body: &[THRStatementID], label: &str) {
        bctx.sig.add_block(label);
        for stmt in body.iter().copied() {
            let stmt = self.thr_module.get_statement(stmt);
            trace!("emitting block statement `{stmt:?}`");
            self.emit_stmt(bctx, stmt);
        }
        bctx.sig
            .blocks
            .last_mut()
            .unwrap()
            .add_comment(format!("END_{label}"));
    }

    fn get_module_prefix(&self) -> String {
        let mut s = self.thr_module.name.clone();
        s.push('_');
        s
    }
}
impl<'m> THRtoQBE<'m> {
    fn eval_literal_expression_u64(&self, id_root: THRExprID) -> Option<u64> {
        let root = self.thr_module.get_expr(id_root);
        match root {
            THRExpression::ConstInt(TypedLiteral { value, .. }) => Some(*value),
            THRExpression::ConstFloat(_) => None,
            THRExpression::ConstBool(b) => Some(*b as u64),
            THRExpression::Binop(inner_op, id_l, id_r) => {
                self.eval_literal_expression_u64_binop(*inner_op, *id_l, *id_r)
            }
            THRExpression::Unop(inner_op, id_arg) => {
                self.eval_literal_expression_u64_unop(*inner_op, *id_arg)
            }
            THRExpression::Ident(_) => None,
        }
    }
    fn eval_literal_expression_u64_binop(
        &self,
        binop: BinOp,
        id_l: THRExprID,
        id_r: THRExprID,
    ) -> Option<u64> {
        let l = self.eval_literal_expression_u64(id_l)?;
        let r = self.eval_literal_expression_u64(id_r)?;
        Some(match binop {
            BinOp::Add => l + r,
            BinOp::Sub => l - r,
            BinOp::Mul => l * r,
            BinOp::Div => l / r,
            BinOp::Mod => l % r,
            BinOp::BitAnd => l & r,
            BinOp::BitOr => l | r,
            BinOp::BitXor => l ^ r,
            BinOp::Subscript => todo!("subscript (a[b]) operations in THR"),
            BinOp::Lsh => l << r,
            BinOp::Rsh => l >> r,
            _ => return None,
        })
    }
    fn eval_literal_expression_u64_unop(&self, op: UnOp, id_arg: THRExprID) -> Option<u64> {
        let arg = self.eval_literal_expression_u64(id_arg)?;
        Some(match op {
            // this assumes 2's complement
            UnOp::Neg => !arg + 1,
            UnOp::LogNot | UnOp::BitNot => !arg,
        })
    }
}
