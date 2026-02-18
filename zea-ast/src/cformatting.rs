trait CStatement {
    fn as_c_statement(&self) -> String;
}

trait CExpr {
    fn as_c_expression(&self) -> String;
}