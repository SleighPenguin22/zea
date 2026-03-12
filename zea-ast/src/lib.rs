pub mod c;
pub mod zea;

pub trait VisitExpr {
    fn visit_expr(&mut self, f: impl FnMut(zea::Expression));
}

pub trait VisitStmt {
    fn visit_stmt(&mut self, f: impl FnMut(zea::Statement));
}

pub trait VisitModule: VisitExpr + VisitStmt {
    fn visit_module(&mut self, f: impl FnMut(zea::Module));
}

#[cfg(feature = "visualisation")]
pub mod visualisation;
