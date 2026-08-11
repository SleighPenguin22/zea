use cranelift::codegen as Ccdg;
use cranelift::codegen::ir::{self as Cir};
use cranelift::codegen::isa as Cisa;
use cranelift::module as Cmod;
use cranelift::object as Cobj;
use cranelift::object::object::Architecture;
use cranelift::prelude as crane;
use log::trace;
use zea_ipr::{
    InternKey, InternTable,
    ast::{
        thr::{
            FloatWidth, IntegerWidth, THRBlock, THRExpression, THRFunction, THRModule,
            THRStatement, THRSymbol, THRTypeSpecifier,
        },
        visitors::annotating::SymbolKind,
    },
};
#[derive(Copy, Clone, InternKey, Debug, PartialEq, Eq)]
pub struct CraneTypeID(u32);

fn get_triple() -> Cisa::OwnedTargetIsa {
    use target_lexicon::Triple;
    Triple::host();
    let builder = Ccdg::settings::builder();
    let flags = Ccdg::settings::Flags::new(builder);
    Cisa::lookup_by_name("x86_64")
        .unwrap()
        .finish(flags)
        .expect("cannot build isa triple")
}

pub struct THRtoCraneLift<'m> {
    thr_module: &'m THRModule,
    crane_types: InternTable<CraneTypeID, cranelift::prelude::Type>,
    temp_generator: usize,
    block_label_generator: usize,
    crane_object: Cobj::ObjectModule,
}

struct FuncContext<'f, 'ctx> {
    builder: crane::FunctionBuilder<'ctx>,
    sig: &'f THRFunction,
}

impl<'f, 'ctx> FuncContext<'f, 'ctx> {
    fn new(builder: crane::FunctionBuilder<'ctx>, sig: &'f THRFunction) -> Self {
        Self { builder, sig }
    }
}

impl<'m> THRtoCraneLift<'m> {
    pub fn new(module: &'m THRModule) -> Self {
        let triple = get_triple();
        let objext_ctx =
            Cobj::ObjectBuilder::new(triple, module.name.clone(), Cmod::default_libcall_names())
                .unwrap();
        let crane_object = Cobj::ObjectModule::new(objext_ctx);
        Self {
            thr_module: module,
            crane_types: InternTable::new(),
            temp_generator: 0,
            block_label_generator: 0,
            crane_object,
        }
    }

    pub fn walk_global_data_blocks(&mut self) {
        let glob_data = self
            .thr_module
            .get_block(self.thr_module.global_data_block());
        for stmt in glob_data.items.iter() {
            todo!()
        }
    }
    /// recursively emit the instructions necessary to represent the given statement
    fn emit_stmt(&mut self, bctx: &mut FuncContext, stmt: &THRStatement) {
        todo!()
    }
    fn emit_stmt_init(
        &mut self,
        bctx: &mut FuncContext,
        symb: &THRSymbol,
        typ: &THRTypeSpecifier,
        value: &THRExpression,
    ) {
        todo!()
    }
    /// Recursively emit instructions necessary to compute the given expression, then return the temporary it is saved to.
    fn emit_expr(&mut self, bctx: &mut FuncContext, expr: &THRExpression) {
        todo!()
    }

    fn emit_type(&mut self, typ: &THRTypeSpecifier) -> CraneTypeID {
        match typ {
            THRTypeSpecifier::Integer { width, .. } => self.emit_type_integer(*width),
            THRTypeSpecifier::Float { width } => self.emit_type_float(*width),
            THRTypeSpecifier::Pointer(_t) => self
                .crane_types
                .intern(crane::Type::int_with_byte_size(8).unwrap()),
            THRTypeSpecifier::Boolean => self
                .crane_types
                .intern(crane::Type::int_with_byte_size(1).unwrap()),
            THRTypeSpecifier::Unit => todo!(),
            THRTypeSpecifier::Never => todo!(),
            THRTypeSpecifier::Struct {
                name: _,
                layout: _layout,
            } => todo!(),
            THRTypeSpecifier::Tuple(_thrstruct_layout) => todo!(),
        }
    }

    fn emit_type_and_get(&mut self, typ: &THRTypeSpecifier) -> crane::Type {
        let t = self.emit_type(typ);
        self.crane_types.get_by_id(t).unwrap().clone()
    }

    fn emit_type_integer(&mut self, width: IntegerWidth) -> CraneTypeID {
        let t = match width {
            IntegerWidth::_8 => crane::types::I8,
            IntegerWidth::_16 => crane::types::I16,
            IntegerWidth::_32 => crane::types::I32,
            IntegerWidth::_64 => crane::types::I64,
        };
        self.crane_types.intern(t)
    }

    fn emit_type_float(&mut self, width: FloatWidth) -> CraneTypeID {
        let t = match width {
            FloatWidth::_32 => crane::types::F32,
            FloatWidth::_64 => crane::types::F64,
        };
        self.crane_types.intern(t)
    }

    fn get_module_func_prefix(&self, bctx: &FuncContext) -> String {
        format!("{}_{}_", self.thr_module.name, bctx.sig.name)
    }
    fn disambiguate_nonglobal_symbol(&self, bctx: &FuncContext, ident: &THRSymbol) -> String {
        let prefix = self.get_module_prefix();
        let demangle = match ident.kind {
            SymbolKind::LocalVar => &format!("{}_local_", bctx.sig.name),
            SymbolKind::GlobalVar => "global_",
            SymbolKind::FunctionName => "func_",
            SymbolKind::FunctionParam => &format!("{}_param_", bctx.sig.name),
            SymbolKind::ImportItem => todo!(),
        };

        format!("{prefix}{demangle}{}", ident.name)
    }
    fn disambiguate_global_symbol(&self, ident: &THRSymbol) -> String {
        let prefix = self.get_module_prefix();
        let demangle = match ident.kind {
            SymbolKind::GlobalVar => "global_",
            SymbolKind::FunctionName => "func_",
            SymbolKind::ImportItem => todo!(),
            _ => unreachable!(
                "only funcnames and globals can be disambiguated without a block context"
            ),
        };

        format!("{prefix}{demangle}{}", ident.name)
    }

    fn emit_datadef(&mut self, object: &mut Cobj::ObjectModule, stmt: &THRStatement) {
        let THRStatement::Init(decl, val) = stmt else {
            unreachable!("global statement must be an init");
        };
        let name = self.thr_module.get_symbol(decl.symbol);
        let name = self.disambiguate_global_symbol(name);
        let thr_typ = self.thr_module.get_type(decl.typ);
        let crane_typ_id = self.emit_type(thr_typ);
        let align = self.thr_module.alignment_of(decl.typ);
        let items = match self.thr_module.get_expr(*val) {
            THRExpression::ConstInt(i) => {}
            THRExpression::ConstFloat(_) => todo!(),
            THRExpression::ConstBool(b) => {}
            THRExpression::Binop(..) => todo!(),
            THRExpression::Unop(..) => todo!(),
            THRExpression::Ident(..) => todo!(),
        };
    }

    // generate the userfuncname associated with a function
    fn get_func_crane_ref(&self, func: &THRFunction) -> Cir::UserFuncName {
        let func_id = self
            .thr_module
            .functions()
            .get_unchecked(func)
            .destruct_key() as u32;
        let mod_id: u32 = self.thr_module.ipr_id.as_usize();
        Cir::UserFuncName::user(mod_id, func_id.into())
    }

    fn emit_function(&mut self, builder: &mut crane::FunctionBuilderContext, func: &THRFunction) {
        trace!("walking function `{}`", func.name);
        let (crane_name, sig) = self.build_func_sig_and_name(func);

        let mut crane_func = Cir::Function::with_name_signature(crane_name, sig);
        let builder = crane::FunctionBuilder::new(&mut crane_func, builder);
        let mut bctx = FuncContext::new(builder, func);
        let body = self.thr_module.get_block(func.body);
        self.emit_block(&mut bctx, body);
    }
    fn build_func_sig_and_name(
        &mut self,
        func: &THRFunction,
    ) -> (Cir::UserFuncName, crane::Signature) {
        let mut sig = crane::Signature::new(crane::isa::CallConv::SystemV);
        let return_ty = self.thr_module.get_type(func.ret);
        let return_ty = self.emit_type_and_get(return_ty);
        let return_abi = Cir::AbiParam::new(return_ty);
        sig.returns.push(return_abi);

        for param in func.params.iter() {
            let arg_ty = self.thr_module.get_type(param.typ);
            let arg_ty = self.emit_type_and_get(arg_ty);
            let value = Cir::AbiParam::new(arg_ty);
            sig.params.push(value)
        }
        let name = self.get_func_crane_ref(func);
        (name, sig)
    }

    fn emit_block(&mut self, bctx: &mut FuncContext, body: &THRBlock) {
        for stmt in body.items.iter().copied() {
            let stmt = self.thr_module.get_statement(stmt);
            trace!("emitting block statement `{stmt:?}`");
            self.emit_stmt(bctx, stmt);
        }
    }

    fn current_block(&mut self, bctx: &mut FuncContext) -> Cir::Block {
        bctx.builder
            .current_block()
            .unwrap_or_else(|| bctx.builder.create_block())
    }

    fn get_module_prefix(&self) -> &str {
        &self.thr_module.name
    }

    pub fn lower(&self) {
        todo!()
    }
}
