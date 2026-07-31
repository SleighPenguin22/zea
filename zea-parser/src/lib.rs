#![allow(unused)]

#[cfg(feature = "lalrpop_parser")]
mod parser;
#[cfg(feature = "zepast")]
pub mod zepast;

use log::{error, info, log};

#[cfg(feature = "lalrpop_parser")]
pub use parser::ExprParser as ExpressionParser;
#[cfg(feature = "lalrpop_parser")]
pub use parser::ModParser as ModuleParser;
use std::process::exit;
use zea_ipr::zea::BareNodeLabeler;

pub use lalrpop_util::ParseError;
use zea_ipr::zea::ipr::*;
use zea_ipr::zea::visitors::IPRTransfomer;
pub fn parse_module(src: &'_ str) -> (IPRModule, BareNodeLabeler) {
    let p = ModuleParser::new();
    info!("parsing source file...");
    let mut module = match p.parse(src) {
        Ok(module) => module,
        Err(e) => {
            error!("{e}");
            exit(1);
        }
    };
    info!("\tparsed source file succesfully");
    info!("starting node-labeling...");
    let labeler = module.label_nodes();
    info!("\tnode-labeling successful");
    (module, labeler)
}

pub(crate) enum IPRModuleItem {
    Init(IPRInitializationBlock),
    Func(IPRFunction),
    StructDef(IPRStructDataTypeDefinition),
    EnumDef(IPRTaggedUnionDataTypeDefinition),
}

pub(crate) fn separate_module_items(
    items: Vec<IPRModuleItem>,
) -> (
    Vec<IPRInitializationBlock>,
    Vec<IPRFunction>,
    Vec<IPRStructDataTypeDefinition>,
    Vec<IPRTaggedUnionDataTypeDefinition>,
) {
    let mut globs = vec![];
    let mut funcs = vec![];
    let mut structs = vec![];
    let mut tagged_unions = vec![];
    for item in items {
        match item {
            IPRModuleItem::Init(i) => globs.push(i),
            IPRModuleItem::Func(f) => funcs.push(f),
            IPRModuleItem::StructDef(s) => structs.push(s),
            IPRModuleItem::EnumDef(tu) => tagged_unions.push(tu),
        }
    }
    (globs, funcs, structs, tagged_unions)
}
