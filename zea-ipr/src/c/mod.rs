#![allow(unused)]

use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, PartialEq)]
pub struct TypedIdentifier(pub Type, pub String);

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCall {
    pub name: String,
    pub args: Vec<Expression>,
}

pub enum PreProcessorDirective {
    PragmaOnce,
    Include(String),
    Define(String, String),
    // __FILE__,
}

pub struct FunctionDeclaration {
    pub returns: Type,
    pub name: String,
    pub args: Vec<TypedIdentifier>,
}

pub struct FunctionDefinition {
    pub declaration: FunctionDeclaration,
    pub body: StatementBlock,
}

pub enum TopLevelDecl {
    FuncDecl(FunctionDeclaration),
    VarDecl(VariableDeclaration),
}

pub enum TopLevelDef {
    FuncDef(FunctionDefinition),
    VarInit(Initialisation),
}

pub struct TranslationUnit {
    directives: Vec<PreProcessorDirective>,
    declarations: Vec<TopLevelDecl>,
    symbols: Vec<TopLevelDef>,
}

pub struct CStructDeclaration(String);
pub struct CStructDefinition {
    pub name: String,
    pub members: Vec<TypedIdentifier>,
}
pub struct CEnumDeclaration(String);
pub struct CUnionDeclaration(String);

pub struct CFunctionDeclaration(pub TypedIdentifier, pub Vec<TypedIdentifier>);
#[derive(Debug, PartialEq, Clone)]
pub enum TypeSpecifier {
    Basic(String),
    Pointer(Box<TypeSpecifier>),
}

pub type DerefAssignee = TypeSpecifier;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum TypeQualifier {
    Static,
    Inline,
}

impl From<TypeQualifier> for String {
    fn from(value: TypeQualifier) -> Self {
        match value {
            TypeQualifier::Static => "static".to_string(),
            TypeQualifier::Inline => "inline".to_string(),
        }
    }
}

impl From<&TypeQualifier> for String {
    fn from(value: &TypeQualifier) -> Self {
        match value {
            TypeQualifier::Static => "static".to_string(),
            TypeQualifier::Inline => "inline".to_string(),
        }
    }
}

impl TypeSpecifier {
    /// Get the type behind any pointer variant:
    /// ```ignore
    /// "int" => "int",
    /// Pointer("int") => "int",
    /// Pointer(Pointer("bool")) => "bool"
    /// ```
    pub fn get_deepest(&self) -> String {
        match self {
            TypeSpecifier::Basic(t) => t.clone(),
            TypeSpecifier::Pointer(t) => t.as_ref().get_deepest(),
        }
    }
}

impl From<TypeSpecifier> for Type {
    fn from(value: TypeSpecifier) -> Self {
        Self {
            qualifiers: HashSet::new(),
            specifier: value,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Type {
    pub qualifiers: HashSet<TypeQualifier>,
    pub specifier: TypeSpecifier,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    FuncCall(FunctionCall),
    Literal(Literal),
    Ident(String),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Mod(Box<Expression>, Box<Expression>),
    Neg(Box<Expression>),

    LogAnd(Box<Expression>, Box<Expression>),
    LogOr(Box<Expression>, Box<Expression>),
    LogNot(Box<Expression>),

    BitAnd(Box<Expression>, Box<Expression>),
    BitOr(Box<Expression>, Box<Expression>),
    BitXor(Box<Expression>, Box<Expression>),
    BitNot(Box<Expression>),

    Ternary(Box<TernaryExpression>),

    MemberAccess(Box<Expression>, String),
    PointerMemberAccess(Box<Expression>, String),
}
#[derive(Clone, Debug, PartialEq)]
pub struct TernaryExpression {
    pub condition: Expression,
    pub true_branch: Expression,
    pub false_branch: Expression,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Integer(u64),
    Float(f64),
    Boolean(bool),
    String(String),
}

impl PartialEq for Literal {
    fn eq(&self, other: &Self) -> bool {
        if let (Self::Float(a), Self::Float(b)) = (self, other)
            && a.is_nan()
            && b.is_nan()
        {
            return true;
        }
        match (self, other) {
            (Self::Integer(a), Self::Integer(b)) => a == b,
            (Self::Boolean(a), Self::Boolean(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            _ => unreachable!(),
        }
    }
}

impl Eq for Literal {}

impl Hash for Literal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Float(f) if f.is_nan() => state.write(f64::NAN.to_ne_bytes().as_ref()),
            Self::Float(f) => state.write(f.to_ne_bytes().as_ref()),
            Self::Boolean(b) => b.hash(state),
            Self::String(s) => s.hash(state),
            Self::Integer(i) => i.hash(state),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VariableDeclaration {
    pub typ: Type,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Initialisation {
    pub typ: Type,
    pub name: String,
    pub value: Expression,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfThenElse {
    pub condition: Expression,
    pub true_branch: StatementBlock,
    pub false_branch: StatementBlock,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IfBlock {
    pub condition: Expression,
    pub body: StatementBlock,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ElseBlock {
    body: StatementBlock,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StatementBlock(pub Vec<Statement>);
#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    VariableDeclaration(VariableDeclaration),
    VariableInitialisation(Initialisation),
    VariableReassignment(Reassignment),
    Return(Expression),
    ReturnVoid,
    FunctionCall(FunctionCall),
    Reassignment(Reassignment),
    DerefAssignment(DerefReassignment),
    IfBlock(IfBlock),
    ElseBlock(ElseBlock),
}
#[derive(Clone, Debug, PartialEq)]
pub struct Reassignment {
    pub assignee: String,
    pub value: Expression,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DerefReassignment {
    assignee: DerefAssignee,
    value: Expression,
}
