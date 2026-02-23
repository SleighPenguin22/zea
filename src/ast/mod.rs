#![allow(dead_code)]
pub mod datatype;
pub mod expression;
pub mod patterns;
pub mod statement;

pub use crate::ast::datatype::{TypedIdentifier, ZeaTypeIdent};
pub use crate::ast::expression::{Literal, ZeaExpression};
pub use crate::ast::statement::{StatementBlock, VarDeclAssignment, ZeaStatement};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

#[derive(Debug, Default, Clone)]
pub struct ZeaModule {
    pub path: PathBuf,
    pub imports: ImportList,
    pub exports: ExportList,
    pub symbols: HashSet<TopLevelStatement>,
}
impl ZeaModule {
    pub(crate) fn find_entry_point(&self) -> Option<FuncDefinition> {
        self.iter_symbols().find_map(|symbol| match symbol {
            TopLevelStatement::FuncDefinition(f) if f.declaration.name == "main" => Some(f.clone()),
            _ => None,
        })
    }

    pub fn iter_symbols(&self) -> impl Iterator<Item = &TopLevelStatement> {
        self.symbols.iter()
    }
}
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum TopLevelStatement {
    FuncDefinition(FuncDefinition),
    GlobalConst(GlobalConst),
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct FuncDeclaration {
    pub name: String,
    pub args: Vec<TypedIdentifier>,
    pub returns: ZeaTypeIdent,
}

/// A top-level function definition
///
/// Function may be defined only once within a module, They are compared and [`Hash`]'ed against their signature.
/// Functions may be imported as many times as needed.
#[derive(Debug, Clone)]
pub struct FuncDefinition {
    pub declaration: FuncDeclaration,
    pub body: StatementBlock,
}
impl PartialEq for FuncDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.declaration == other.declaration
    }
}
impl Eq for FuncDefinition {}
impl Hash for FuncDefinition {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.declaration.hash(state)
    }
}

/// A global Constant value, this is a variable declaration-assignment at the module level.
///
/// These declaration are compared and [`Hash`]'ed against their signature/declaration.
/// And may only be defined once.
#[derive(Debug, Clone)]
pub struct GlobalConst(VarDeclAssignment);

impl PartialEq for GlobalConst {
    fn eq(&self, other: &Self) -> bool {
        self.0.decl == other.0.decl
    }
}
impl Eq for GlobalConst {}

impl Hash for GlobalConst {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.decl.hash(state)
    }
}

pub type ImportList = Vec<ZeaImportStatement>;
pub type ExportList = Vec<ZeaExportStatement>;

#[derive(Debug, Clone)]
pub struct ZeaImportStatement {
    path: String,
}
#[derive(Debug, Clone)]
pub enum ZeaExportStatement {
    Func(String),
    GlobalConst(String),
}

pub mod utils {
    use super::*;
    use crate::ast::utils::statements::return_literal_int;
    use crate::ast::{
        FuncDeclaration, FuncDefinition, Literal, StatementBlock, TopLevelStatement, ZeaExpression,
        ZeaModule, ZeaStatement, ZeaTypeIdent,
    };

    pub fn basic_module(
        name: impl Into<String>,
        symbols: impl Iterator<Item = TopLevelStatement>,
    ) -> ZeaModule {
        ZeaModule {
            path: name.into().into(),
            imports: vec![],
            exports: vec![],
            symbols: symbols.collect(),
        }
    }

    pub fn basic_main_returning(value: u8) -> FuncDefinition {
        FuncDefinition {
            declaration: FuncDeclaration {
                name: "main".to_string(),
                args: vec![],
                returns: ZeaTypeIdent::Basic("u8".to_string()),
            },
            body: StatementBlock(vec![return_literal_int(value as u64)]),
        }
    }

    pub mod statements {
        use super::expressions::expr_literal_int;
        use super::*;
        use crate::ast::patterns::ZeaPattern;
        use crate::ast::statement::VarDecl;
        pub fn return_literal_int(value: u64) -> ZeaStatement {
            ZeaStatement::ReturnValue(expr_literal_int(value))
        }

        /// A basic declaration like
        ///
        /// `const int a;`
        /// `char b;`
        pub fn basic_declaration(typ: ZeaTypeIdent, name: impl Into<String>) -> ZeaStatement {
            ZeaStatement::VarDecl(VarDecl {
                mutable: true,
                typ,
                assignee: ZeaPattern::Ident(name.into()),
            })
        }

        /// A basic declaration like
        ///
        /// `const int a = 3;`
        /// `char b = 'b';`
        pub fn basic_assignment_mut(
            typ: ZeaTypeIdent,
            name: impl Into<String>,
            value: ZeaExpression,
        ) -> ZeaStatement {
            ZeaStatement::VarDeclAssignment(VarDeclAssignment {
                decl: VarDecl {
                    mutable: true,
                    typ,
                    assignee: ZeaPattern::Ident(name.into()),
                },
                value,
            })
        }

        /// A basic declaration like
        ///
        /// `const int a = 3;`
        /// `char b = 'b';`
        pub fn basic_assignment_immut(
            typ: ZeaTypeIdent,
            name: impl Into<String>,
            value: ZeaExpression,
        ) -> ZeaStatement {
            ZeaStatement::VarDeclAssignment(VarDeclAssignment {
                decl: VarDecl {
                    mutable: false,
                    typ,
                    assignee: ZeaPattern::Ident(name.into()),
                },
                value,
            })
        }
    }

    pub mod expressions {
        use super::*;
        pub fn expr_literal_int(value: u64) -> ZeaExpression {
            ZeaExpression::Literal(Literal::Integer(value))
        }
        
        macro_rules! expr_return {
            (return $l:literal) => {
                {
                    ZeaExpression::Literal(Literal::from($l))

                }
            };
        }
    }

    pub mod literals {
        use super::*;

        pub fn literal_int(value: u64) -> Literal {
            Literal::Integer(value)
        }

        pub fn literal_float(value: f64) -> Literal {
            Literal::Float(value)
        }

        pub fn literal_bool(value: bool) -> Literal {
            Literal::Boolean(value)
        }
        pub fn literal_string(value: impl Into<String>) -> Literal {
            Literal::String(value.into())
        }
    }
    pub mod types {
        use crate::ast::ZeaTypeIdent;

        pub fn ptr_to(typ: ZeaTypeIdent) -> ZeaTypeIdent {
            ZeaTypeIdent::Ptr(Box::new(typ))
        }
        pub fn array_of(typ: ZeaTypeIdent) -> ZeaTypeIdent {
            ZeaTypeIdent::ArrayOf(Box::new(typ))
        }

        pub fn slice_of(typ: ZeaTypeIdent) -> ZeaTypeIdent {
            ZeaTypeIdent::Slice(Box::new(typ))
        }

        pub fn optional(typ: ZeaTypeIdent) -> ZeaTypeIdent {
            ZeaTypeIdent::Option(Box::new(typ))
        }

        pub fn basic_str() -> ZeaTypeIdent {
            ZeaTypeIdent::Basic("Str".into())
        }

        pub fn basic_int() -> ZeaTypeIdent {
            ZeaTypeIdent::Basic("I32".into())
        }

        pub fn basic_uint() -> ZeaTypeIdent {
            ZeaTypeIdent::Basic("U32".into())
        }

        pub fn basic_float() -> ZeaTypeIdent {
            ZeaTypeIdent::Basic("F32".into())
        }

        pub fn basic_bool() -> ZeaTypeIdent {
            ZeaTypeIdent::Basic("Bool".into())
        }
    }
}
