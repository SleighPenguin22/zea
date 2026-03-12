use crate::zea::Statement;

pub mod c;
pub mod zea;

pub trait HasStatements {
    fn get_expressions(&self) -> Vec<&Statement>;
}

pub trait VisitExpr {
    fn visit_expr(&mut self, f: &impl FnMut(zea::Expression), expression: &mut zea::Expression);
}

pub trait VisitStmt {
    fn visit_stmt(&mut self, f: impl FnMut(zea::Statement), stmt: &mut zea::Statement);
}

pub trait VisitModule: VisitExpr + VisitStmt {
    fn visit_module(&mut self, f: impl FnMut(zea::Module), module: &mut zea::Module);
}

#[cfg(feature = "visualisation")]
pub mod visualisation;
