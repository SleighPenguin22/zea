use indexmap::IndexSet;
use std::hash::{Hash, Hasher};
use zea_config::CompilerConfig;
use zea_internal_macros::{HashEqById, InternKey};

use crate::{
    zea::{
        float_total_cmp,
        hir_nodes::{HIRStructDataTypeDefinition, HIRTypeSpecifier},
        visitors::annotating::ScopedIdentifierKind,
    },
    InternTable,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash, InternKey)]
pub struct MIRInstructionID(u32);
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash, InternKey)]
pub struct MIRTypeID(u32);
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash, InternKey)]
pub struct MIRBlockID(u32);
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash, InternKey)]
pub struct MIRGlobalID(u32);

#[derive(Debug)]
pub struct MIRModule<'cfg> {
    config: &'cfg CompilerConfig,
    globs: InternTable<MIRGlobalID, MIRGlobal>,
    types: InternTable<MIRTypeID, MIRType>,
    blocks: InternTable<MIRBlockID, MIRBlock>,
    instrs: InternTable<MIRInstructionID, MIRInstruction>,
}
impl<'cfg> MIRModule<'cfg> {
    pub fn new(config: &'cfg CompilerConfig) -> Self {
        Self {
            globs: InternTable::new(),
            types: InternTable::new(),
            blocks: InternTable::new(),
            instrs: InternTable::new(),
            config,
        }
    }
    pub fn add_type(&mut self, typ: MIRType) -> MIRTypeID {
        self.types.intern(typ)
    }
    pub fn get_type_by_id(&self, id: MIRTypeID) -> &MIRType {
        self.types.get_by_id(id).expect("type shoulda been added")
    }
    pub fn hir_to_mir_type(&mut self, t: &HIRTypeSpecifier) -> MIRType {
        let t = match t {
            HIRTypeSpecifier::NonScalar(_) => todo!(),
            HIRTypeSpecifier::Unit => MIRType::Unit,
            HIRTypeSpecifier::Bool => MIRType::Bool,
            HIRTypeSpecifier::Integer { width, signed } => match (width, signed) {
                (..=32, true) => MIRType::I32,
                (33.., true) => MIRType::I64,
                (..=32, false) => MIRType::U32,
                (33.., false) => MIRType::U64,
            },

            HIRTypeSpecifier::Float { width } => {
                if *width <= 32 {
                    MIRType::F32
                } else {
                    MIRType::F64
                }
            }
            HIRTypeSpecifier::ArrayOf(t) | HIRTypeSpecifier::Pointer(t) => {
                let t_inner = self.hir_to_mir_type(t);

                let t_inner = self.add_type(t_inner);
                MIRType::Pointer(t_inner)
            }
            HIRTypeSpecifier::Never => MIRType::Never,
        };
        self.add_type(t.clone());
        t
    }
    pub fn build_struct_layout(&mut self, s: &HIRStructDataTypeDefinition) -> MIRType {
        let mut ordered_fields: Vec<(String, MIRType)> = s
            .members
            .iter()
            .map(|mem| (mem.name.clone(), self.hir_to_mir_type(&mem.typ)))
            .collect();
        let (size, largest_field) = self.struct_size_and_largest_field(ordered_fields.as_slice());

        if self.should_reorder_fields(s) {
            ordered_fields.sort_by(|(_, a), (_, b)| a.size(self).cmp(&b.size(self)).reverse());
        };

        let mut alignment = largest_field;
        while alignment < size {
            alignment += largest_field
        }
        let mut fields = vec![];
        let mut offset = 0;
        for (name, typ) in ordered_fields {
            let cur_offset = offset + typ.size(self);
            let field = MIRStructField {
                name,
                typ: self.add_type(typ),
                byte_offset: cur_offset,
            };
            fields.push(field);
            offset = cur_offset;
        }

        MIRType::Struct {
            name: s.name.clone(),
            size,
            alignment,
            fields,
        }
    }
    fn should_reorder_fields(&self, s: &HIRStructDataTypeDefinition) -> bool {
        s.reorder_fields.is_none_or(|b| b)
    }
    fn struct_size_and_largest_field(&self, mirred_fields: &[(String, MIRType)]) -> (usize, usize) {
        let sizes: Vec<_> = mirred_fields.iter().map(|(_, t)| t.size(self)).collect();
        let size = sizes.iter().sum();
        let largest = sizes.into_iter().max().unwrap_or(0);
        (size, largest)
    }

    pub fn add_block(&mut self, block: MIRBlock) -> MIRBlockID {
        self.blocks.intern(block)
    }
    pub fn add_glob(&mut self, glob: MIRGlobal) -> MIRGlobalID {
        self.globs.intern(glob)
    }

    fn add_instr(&mut self, item: MIRInstruction) -> MIRInstructionID {
        self.instrs.intern(item)
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct MIRBlock {
    label: String,
    items: Vec<MIRInstructionID>,
    terminator: MIRTerminator,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum MIRTerminator {
    Ret {
        with: Option<MIRInstructionID>,
    },
    Jmp {
        to: MIRBlockID,
    },
    Branch {
        cond: MIRInstructionID,
        then: MIRBlockID,
        other: MIRBlockID,
    },
    Unreachable,
}
#[derive(Default)]
struct MIRBlockBuilder {
    label: Option<String>,
    items: Vec<MIRInstructionID>,
    terminator: Option<MIRTerminator>,
}
impl MIRBlockBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }
    pub fn add_item_from_id(mut self, item: MIRInstructionID) -> Self {
        self.items.push(item);
        self
    }
    pub fn add_item_from_data(mut self, item: MIRInstruction, ctx: &mut MIRModule) -> Self {
        let item = ctx.add_instr(item);
        self.items.push(item);
        self
    }
    pub fn with_terminator(mut self, term: MIRTerminator) -> Self {
        self.terminator = Some(term);
        self
    }
    pub fn build(self) -> Option<MIRBlock> {
        let terminator = self.terminator?;
        let label = self.label?;
        Some(MIRBlock {
            label,
            items: self.items,
            terminator,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MIRStructField {
    name: String,
    typ: MIRTypeID,
    byte_offset: usize,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum MIRType {
    Bool,
    U64,
    U32,
    F32,
    F64,
    Unit,
    Pointer(MIRTypeID),
    Array(MIRTypeID, usize),
    Struct {
        name: String,
        size: usize,
        alignment: usize,
        fields: Vec<MIRStructField>,
    },
    I32,
    I64,
    Never,
}
impl MIRType {
    pub fn size(&self, ctx: &MIRModule) -> usize {
        match self {
            MIRType::Bool => 1,
            MIRType::U64 => 8,
            MIRType::U32 => 4,
            MIRType::F32 => 4,
            MIRType::F64 => 8,
            MIRType::Unit => 0,
            MIRType::Pointer(_mirtype_id) => 8,
            MIRType::Array(t, len) => ctx.get_type_by_id(*t).size(ctx) * len,
            MIRType::Struct { size, .. } => *size,
            MIRType::I32 => 4,
            MIRType::I64 => 8,
            MIRType::Never => 0,
        }
    }
    pub fn alignment(&self, ctx: &MIRModule) -> usize {
        match self {
            MIRType::Bool => 1,
            MIRType::U64 => 8,
            MIRType::U32 => 4,
            MIRType::F32 => 4,
            MIRType::F64 => 8,
            MIRType::Unit => 0,
            MIRType::Pointer(_mirtype_id) => 8,
            MIRType::Array(t, _) => ctx.get_type_by_id(*t).size(ctx),
            MIRType::Struct { alignment, .. } => *alignment,
            MIRType::I32 => 4,
            MIRType::I64 => 8,
            MIRType::Never => 0,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct MIRInstruction {
    typ: MIRTypeID,
    kind: MIRInstructionKind,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct MIRGlobal {
    name: String,
    typ: MIRTypeID,
    value: MIRInstructionID,
}

#[derive(Debug, Clone, Copy)]
pub enum MIRInstructionKind {
    ConstInt(u64),
    ConstFloat(f64),
    Load(MIRInstructionID),
    Store(MIRInstructionID),
    GlobalPtr(MIRGlobalID),
}
impl PartialEq for MIRInstructionKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MIRInstructionKind::ConstInt(a), MIRInstructionKind::ConstInt(b)) => a == b,
            (MIRInstructionKind::ConstFloat(a), MIRInstructionKind::ConstFloat(b)) => {
                float_total_cmp(*a, *b)
            }
            (MIRInstructionKind::Load(a), MIRInstructionKind::Load(b)) => a == b,
            (MIRInstructionKind::Store(a), MIRInstructionKind::Store(b)) => a == b,
            _ => false,
        }
    }
}
impl Eq for MIRInstructionKind {}
impl Hash for MIRInstructionKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            MIRInstructionKind::ConstInt(a) => a.hash(state),
            MIRInstructionKind::ConstFloat(f) => {
                if f.is_nan() {
                    let nan_as_u64 = f64::NAN.to_bits();
                    nan_as_u64.hash(state);
                }
            }
            MIRInstructionKind::Load(mirinstruction_id) => mirinstruction_id.hash(state),
            MIRInstructionKind::Store(mirinstruction_id) => mirinstruction_id.hash(state),
            MIRInstructionKind::GlobalPtr(mirglobal_id) => mirglobal_id.hash(state),
        }
    }
}
