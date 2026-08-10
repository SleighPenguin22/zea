#![allow(dead_code, unused_imports, unused_macro_rules, unused_macros)]
use crate::ast::ipr::IPRASTNode;
use crate::ast::ipr::IPRModule;
use crate::ast::visitors::IPRVisitor;
/// This module contains the AST definition for the Zea language.
/// Any node that encompasses some structure with meaningful data has an id, this id has the following guarantees:
/// - the id is unique
/// - there is no specified order in the id's of nodes.
/// - the ID of a node stays the same through the whole compilation process.
///
/// As such, you can use these id's as keys in hashtables tables that annotate nodes.
///
/// To maintain these invariants, any AST-visitors must not change the ID of an existing node.
/// When the visitor places a new node in the AST, that node must get a new unique ID.
///
/// To make this easier, the [`visitors::altering::NodeLabeler`] trait
/// may be implemented on a visitor.
/// This trait provides the [`visitors::altering::NodeLabeler::labeler_from`] method
/// to maintain the generation of unique ID's.
///
/// A node having an ID of 0 signals a sentinel ID,
/// it signifies that the node still requires a unique ID.
///
/// The [`visitors::altering::BareNodeLabeler`] visitor
/// grants a unique ID to node with a sentinel ID.
use crate::impls::StructuralEq;
use arbitrary::{Arbitrary, Unstructured};

mod impls;
pub mod visitors;

pub mod ipr;
pub mod thr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, VariantToStr, Arbitrary)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    LogAnd,
    LogOr,
    LogXor,
    BitAnd,
    BitOr,
    BitXor,
    Subscript,
    Lsh,
    Rsh,
    Eq,
    Neq,
    Geq,
    Leq,
    LT,
    GT,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, VariantToStr, Arbitrary)]
pub enum UnOp {
    Neg,
    LogNot,
    BitNot,
}
#[cfg(test)]
pub(crate) mod test_ast_macros {
    macro_rules! label_ast {
        (fresh $ast:expr) => {{
            use crate::ast::Module;
            use crate::ast::visitors::altering::{BareNodeLabeler, NodeLabeler};

            let mut node_labeler = BareNodeLabeler::new();
            let mut ast = $ast;
            ast.accept_sentinel_labeler(&mut node_labeler);
            (ast, node_labeler)
        }};
        (using $l:expr ; $ast:expr) => {{
            let mut ast = $ast;
            let mut l = $l;
            ast.accept_sentinel_labeler(&mut l);
            (ast, l)
        }};
    }
    pub(crate) use label_ast;
    macro_rules! block {
        {} => {
           {
               BlockExpression {
                    id: NodeId::sentinel(),
                    statements: vec![],
                   last: Expression::unit()
               }
           }
        };
        {$($e:expr);* ; $last:expr} => {
           {
               let last: Expression = $last;
               BlockExpression {
                    id: NodeId::sentinel(),
                    statements: vec![$($e),*]
                    last
               }
           }
        };
        {exp $($e:expr);+ $(;)?} => {
            {
                let be = crate::ast::BlockExpander::new();
                let mut b: Statement = stmt!(block
                StatementBlock {
                    id: NodeId::sentinel(),
                    statements: vec![$($e),+]
               };);
                b.accept_block_expander(&mut be);
                let StatementKind::ExpandedBlock(eb) = b.kind else {panic!("expected expanded block")};
                eb
            }
        }
    }

    pub(crate) use block;

    macro_rules! stmt {
        (ret $e:expr) => {
            {use crate::ast::{Statement,StatementKind, NodeId};
            Statement {
                id: NodeId::sentinel(),
                kind: StatementKind::Return($e)
            }
        }};
        (block $e:expr) => {
            {
                use crate::ast::{Statement, StatementKind};
            Statement {
                id: NodeId::sentinel(),
                kind: StatementKind::Block($e)
            }
        }};
        (tail $e:expr) => {
            {use crate::ast::{Statement,StatementKind, NodeId};
            Statement {
                id: NodeId::sentinel(),
                kind: StatementKind::BlockTail($e)
            }
        }};
        (call $name:ident ($($e:expr),*)) => {
            {
                use crate::ast::{Statement, StatementKind, FunctionCall}
            Statement {
                id: NodeId::sentinel(),
                kind: StatementKind::FunctionCall(FunctionCall {
                    id: NodeId::sentinel(),
                    subject: $name,
                    args: vec![$($e),*]
                })
            }
        }};

        (init $p:expr ;= $val:expr) => {
            {
                use crate::ast::{AssignmentPattern,InitializationBlock,Statement,StatementKind};
            Statement {
                id: NodeId::sentinel(),
                kind: StatementKind::Initialization(InitializationBlock {
                    id: NodeId::sentinel(),
                    kind: InitializationKind::Packed(
                        PackedInitialization {
                    assignee: $p,
                    typ: None,
                    value: $val,
                        }
                    )
                })
            }
        }};
    }
    pub(crate) use stmt;
    macro_rules! init {
        ($p:expr ;= $val:expr) => {{
            use crate::ast::InitializationBlock;
            InitializationBlock {
                id: NodeId::sentinel(),
                kind: InitializationKind::Packed(PackedInitialization {
                    assignee: $p,
                    typ: None,
                    value: $val,
                }),
            }
        }};
    }
    pub(crate) use init;

    // generated by claude code, prompt:
    // "can you make the pat macro a tt muncher
    // that accepts things like (a,(b,c)) and converts it to a nested assignment pattern"
    // + pat macro
    macro_rules! pat {
    // Single identifier — base case
    ($i:ident) => {
        AssignmentPattern::Identifier(String::from(stringify!($i)))
    };
    // Outer tuple — kick off the muncher with an empty accumulator
    (($($t:tt)*)) => {
        pat!(@munch [] $($t)*)
    };
    // Muncher: accumulator is complete, nothing left to consume
    (@munch [$($acc:expr),*]) => {
        AssignmentPattern::Tuple(vec![$($acc),*])
    };
    // Muncher: next item is a nested tuple, more items follow
    (@munch [$($acc:expr),*] ($($inner:tt)*), $($rest:tt)*) => {
        pat!(@munch [$($acc,)* pat!(($($inner)*))] $($rest)*)
    };
    // Muncher: next item is a nested tuple, nothing follows
    (@munch [$($acc:expr),*] ($($inner:tt)*)) => {
        pat!(@munch [$($acc,)* pat!(($($inner)*))])
    };
    // Muncher: next item is an identifier, more items follow
    (@munch [$($acc:expr),*] $i:ident, $($rest:tt)*) => {
        pat!(@munch [$($acc,)* pat!($i)] $($rest)*)
    };
    // Muncher: next item is an identifier, nothing follows
    (@munch [$($acc:expr),*] $i:ident) => {
        pat!(@munch [$($acc,)* pat!($i)])
    };

}
    pub(crate) use pat;
    macro_rules! expr {
        (ident $($l:tt)+) => {{
            use crate::ast::{Expression, ExpressionKind, NodeId};
            Expression {
                id: NodeId::sentinel(),
                kind: ExpressionKind::UnScopedIdent(String::from(stringify!($($l)+))),
            }
        }};
        (litint $l:literal) => {{
            use crate::ast::{Expression, ExpressionKind, NodeId};
            Expression {
                id: NodeId::sentinel(),
                kind: ExpressionKind::IntegerLiteral($l),
            }
        }};
        (litfloat $l:literal) => {{
            use crate::ast::{Expression, ExpressionKind, NodeId};
            Expression {
                id: NodeId::sentinel(),
                kind: ExpressionKind::FloatLiteral($l),
            }
        }};
        (litbool $l:literal) => {{
            use crate::ast::{Expression, ExpressionKind, NodeId};
            Expression {
                id: NodeId::sentinel(),
                kind: ExpressionKind::BoolLiteral($l),
            }
        }};
        (litstr $l:literal) => {
            Expression {
                id: NodeId::sentinel(),
                kind: ExpressionKind::StringLiteral(stringify!($l)),
            }
        };
        (unit) => {{
            use crate::ast::{Expression, ExpressionKind, NodeId};
            Expression {
                id: NodeId::sentinel(),
                kind: ExpressionKind::Unit,
            }
        }};
        (block $block:expr) => {{
            use crate::ast::{Expression,ExpressionKind,StatementBlock};
            Expression {
                id: NodeId::sentinel(),
                kind: ExpressionKind::Block(Box::new($block))
            }
            }
        }
    }
    pub(crate) use expr;

    macro_rules! zea_module {
        (imports {$($imp:ident),* $(,)?}
         exports {$($exp:ident),* $(,)?}
         globs   {$($glob:expr);* $(;)?}
         funcs   {$($func:expr);* $(;)?}
         structs {$($struct_def:expr);* $(;)?}
        ) => {
            {
                use crate::ast::{Module, NodeId};
                Module {
                    id: NodeId::sentinel(),
                    imports: vec![$(String::from(stringify!($imp))),*],
                    exports: vec![$(String::from(stringify!($exp))),*],
                    global_vars: vec![$($glob),*],
                    functions: vec![$($func),*],
                    struct_definitions: vec![$($struct_def),*]
                }
            }
        };
        (imports {$($imp:ident),* $(,)?}
         globs   {$($glob:expr);* $(;)?}
         funcs   {$($func:expr);* $(;)?}
        ) => {
            {
                use crate::ast::{Module, NodeId};
                Module {
                    id: NodeId::sentinel(),
                    imports: vec![$(String::from(stringify!($imp))),*],
                    exports: vec![],
                    global_vars: vec![$($glob),*],
                    functions: vec![$($func),*],
                    struct_definitions: vec![]
                }
            }
        };
    }
    pub(crate) use zea_module;
    macro_rules! func {
        {$name:ident ( $($arg:ident: $typ:expr),* ) -> $ret:expr; $body:expr } => {
            {
                use crate::ast::{Function, TypedIdentifier};
                let args = vec![$(
                TypedIdentifier(String::from(stringify!($arg)), $typ)
                ),*];
                Function {
                    id: NodeId::sentinel(),
                    name: String::from(stringify!($name)),
                    params:args,
                    returns: $ret,
                    body: $body,
                }
            }
        };
    }
    pub(crate) use func;
    macro_rules! ztyp {
        (U8) => {
            {
            use crate::ast::ipr::IPRTypeSpecifier;
            IPRTypeSpecifier::t_U8()
        }
    };
        (I8) => {{
            use crate::ast::ipr::IPRTypeSpecifier;
            IPRTypeSpecifier::t_I8()
        }
    };
        ($t:ident) => {
            {
            use crate::ast::ipr::IPRTypeSpecifier;
                IPRTypeSpecifier::NonScalar(String::from(stringify!($t)))
            }
        };
        (*$($t:tt)+) => {
            {
            use crate::ast::ipr::IPRTypeSpecifier;
                IPRTypeSpecifier::Pointer(Box::new(ztyp!($($t)+)))
            }
        };
        ([ ]$($t:tt)+) => {
            {
            use crate::ast::ipr::IPRTypeSpecifier;
                IPRTypeSpecifier::ArrayOf(Box::new(ztyp!($($t)+)))
            }
        };
    }
    pub(crate) use ztyp;
}

pub use crate::ast::visitors::altering::{BareNodeLabeler, NodeLabeler};
pub use crate::ast::visitors::annotating::IPRScopedIdentifier;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use zea_internal_macros::{ASTStructuralEq, VariantToStr};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct NodeId(u32);
impl<'a> Arbitrary<'a> for NodeId {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let _ = u;
        Ok(NodeId::sentinel())
    }
}

impl NodeId {
    pub const fn sentinel() -> Self {
        Self(0)
    }

    pub const fn as_usize(&self) -> u32 {
        self.0
    }
}
impl Display for NodeId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

/// This visitor provides a way to query a node by its id, the returned node is of type [`IPRASTNode`],
/// which provides methods to destructure it into its inner node.
/// This visitor is really only useful if you know the type of the node beforehands.
pub struct ZeaNodeQuery {
    id: NodeId,
}
impl ZeaNodeQuery {
    pub fn query_ipr_node(id: NodeId, module: &IPRModule) -> Option<IPRASTNode> {
        let mut s = Self { id };
        match s.visit_module(module) {
            Ok(Some(n)) => Some(n),
            _ => None,
        }
    }
}

fn float_total_cmp(a: f64, b: f64) -> bool {
    if a.is_nan() && b.is_nan() {
        true
    } else {
        a == b
    }
}
