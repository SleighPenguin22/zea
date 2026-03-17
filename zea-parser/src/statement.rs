use crate::{NodeIdGenerator, ParseResult, ParseState, KW_RETURN};
use zea_ast::zea::{AssignmentPattern, Initialisation, Statement, StatementKind};

macro_rules! wrap_stmt {
    ($kind:ident ( $e:expr ) with $generator:ident, $state:ident) => {{
        let e = Statement {
            id: $generator.get(),
            kind: StatementKind::$kind($e),
        };
        Ok((e, $state))
    }};
}

impl<'state> ParseState<'state> {
    pub fn parse_stmt(
        self,
        node_id_generator: &mut NodeIdGenerator,
    ) -> ParseResult<'state, Statement> {
        self.parse_stmt_return(node_id_generator)
            .or(self.parse_stmt_initialisation(node_id_generator))
    }
    pub fn parse_stmt_return(
        self,
        node_id_generator: &mut NodeIdGenerator,
    ) -> ParseResult<'state, Statement> {
        let state = self.whitespace().keyword(KW_RETURN)?.whitespace();
        let (expr, state) = state.parse_expr(node_id_generator)?;
        wrap_stmt!(Return(expr) with node_id_generator, state)
    }

    pub fn parse_initialisation(
        self,
        node_id_generator: &mut NodeIdGenerator,
    ) -> ParseResult<'state, Initialisation> {
        let state = self.whitespace();
        let (assignee, state) = state.parse_assignment_pattern()?;
        let state = state.colon()?;
        let (typ, state) = match state.parse_type() {
            Ok((typ, p_type)) => (Some(typ), p_type),
            Err(_) => (None, state),
        };
        let state = state.op_assign()?;
        let (value, state) = state.parse_expr(node_id_generator)?;
        let state = state.semicolon()?;
        let init = Initialisation {
            id: node_id_generator.get(),
            typ,
            assignee,
            value,
        };

        Ok((init, state))
    }

    pub fn parse_stmt_initialisation(
        self,
        node_id_generator: &mut NodeIdGenerator,
    ) -> ParseResult<'state, Statement> {
        let (init, state) = self.parse_initialisation(node_id_generator)?;
        wrap_stmt!(Initialisation(init) with node_id_generator, state)
    }

    fn parse_assignment_pattern_simple(self) -> ParseResult<'state, AssignmentPattern> {
        let (ident, state) = self.parse_non_type_identifier()?;
        Ok((AssignmentPattern::Identifier(ident), state))
    }

    fn parse_assignment_pattern_tuple(self) -> ParseResult<'state, AssignmentPattern> {
        let mut state = self.open_paren()?;
        let mut assignees = Vec::new();
        loop {
            match state.whitespace().parse_assignment_pattern() {
                Ok((pat, p_pat)) => {
                    assignees.push(pat);
                    state = p_pat;
                }
                Err(e) => break,
            }
            match state.whitespace().comma() {
                Ok(p_comma) => state = p_comma,
                Err(e) => break,
            }
        }
        let state = state.whitespace().close_paren()?;
        Ok((AssignmentPattern::Tuple(assignees), state))
    }

    fn parse_assignment_pattern(self) -> ParseResult<'state, AssignmentPattern> {
        self.parse_assignment_pattern_simple()
            .or(self.parse_assignment_pattern_tuple())
    }
}
