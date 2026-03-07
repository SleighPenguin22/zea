use crate::ast::Expression;
use crate::lowering::{
    ExpandedBlockExpr, ExpandedExpression, ExpandedInitialisation, ExpandedStatement,
    SimpleInitialisation,
};
use crate::visualisation::{
    assignment_chainer, block_chainer, chain_nodes, Labeler, RenderingNodeBuilder, Visualise,
    VisualizeResult,
};
use vizoxide::attr::edge::LABEL;
use vizoxide::attr::node::{COLOR, SHAPE};
use vizoxide::Graph;

impl ExpandedExpression {
    pub fn render_unit<'graph>(
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        Expression::render_unit(graph, labeler)
    }
    pub fn render_binary_operation<'graph>(
        &self,
        a: &ExpandedExpression,
        b: &ExpandedExpression,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        let id = labeler.get();
        let operator_label = match self {
            ExpandedExpression::Add(..) => "+",
            ExpandedExpression::Sub(..) => "-",
            ExpandedExpression::Mod(..) => "%",
            ExpandedExpression::Mul(..) => "*",
            ExpandedExpression::Div(..) => "/",
            ExpandedExpression::BitAnd(..) => "&",
            ExpandedExpression::BitOr(..) => "|",
            ExpandedExpression::LogAnd(..) => "and",
            ExpandedExpression::LogOr(..) => "or",
            ExpandedExpression::BitXor(..) => "^",
            _ => unreachable!(),
        };
        let node = graph
            .create_node(&id.to_string())
            .attribute(LABEL, operator_label)
            .attribute(SHAPE, "diamond")
            .build()
            .unwrap();
        let (a_id, a) = a.render(graph, labeler)?;
        let (b_id, b) = b.render(graph, labeler)?;

        graph.create_edge(&node, &a, None).build().unwrap();
        graph.create_edge(&node, &b, None).build().unwrap();
        Ok((id, node))
    }

    pub fn render_unary_operation<'graph>(
        &self,
        arg: &ExpandedExpression,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        let id = labeler.get();
        let operator_label = match self {
            ExpandedExpression::BitNot(_) => "~",
            ExpandedExpression::LogNot(_) => "!",
            ExpandedExpression::Neg(_) => "-",
            _ => unreachable!(),
        };
        let node = graph
            .create_node(&id.to_string())
            .attribute(LABEL, operator_label)
            .build()
            .unwrap();
        let (arg_id, arg) = arg.render(graph, labeler)?;

        graph.create_edge(&node, &arg, None).build().unwrap();
        Ok((id, node))
    }
}

impl Visualise for ExpandedExpression {
    fn render<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        match self {
            ExpandedExpression::Add(a, b)
            | ExpandedExpression::Sub(a, b)
            | ExpandedExpression::Mod(a, b)
            | ExpandedExpression::Mul(a, b)
            | ExpandedExpression::Div(a, b)
            | ExpandedExpression::BitAnd(a, b)
            | ExpandedExpression::BitOr(a, b)
            | ExpandedExpression::LogAnd(a, b)
            | ExpandedExpression::LogOr(a, b)
            | ExpandedExpression::BitXor(a, b) => {
                self.render_binary_operation(a.as_ref(), b.as_ref(), graph, labeler)
            }
            ExpandedExpression::BitNot(arg)
            | ExpandedExpression::LogNot(arg)
            | ExpandedExpression::Neg(arg) => {
                self.render_unary_operation(arg.as_ref(), graph, labeler)
            }
            ExpandedExpression::Literal(l) => l.render(graph, labeler),
            ExpandedExpression::Ident(i) => i.render(graph, labeler),
            ExpandedExpression::Block(block) => block.render(graph, labeler),
            ExpandedExpression::Unit => Self::render_unit(graph, labeler),
            _ => {
                println!("bob");
                Err(format!("cannot yet render expression\n{self:?}\n"))
            }
        }
    }
}

impl Visualise for ExpandedStatement {
    fn render<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        match self {
            ExpandedStatement::Initialisation(init) => init.render(graph, labeler),
            ExpandedStatement::Return(expr) => {
                let id = labeler.get();
                let node = graph
                    .create_node(&id.to_string())
                    .attribute(LABEL, "return")
                    .attribute(SHAPE, "box")
                    .attribute(COLOR, "red")
                    .attribute("style", "filled")
                    .build()
                    .unwrap();

                let (_, inner) = expr.render(graph, labeler)?;
                graph.create_edge(&node, &inner, None).build().unwrap();
                Ok((id, node))
            }
            ExpandedStatement::LoweredBlock(block) => block.render(graph, labeler),
            _ => unimplemented!("cannot yet render statement:\n {self:?}\n"),
        }
    }
}

impl Visualise for ExpandedBlockExpr {
    fn render<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        let id = labeler.get();
        let (id, b) = RenderingNodeBuilder::new(graph, id)
            .fillcolor("orange")
            .filled()
            .label("block")
            .build_vis()?;
        let (_, result) = self.last.render(graph, labeler)?;

        if let Some((head, tail)) = self.statements.split_first() {
            let (head, last) =
                chain_nodes(graph, labeler, self.statements.as_slice(), block_chainer).unwrap()?;

            block_chainer(graph, &b, &head);
            graph
                .create_edge(&last, &result, None)
                .attribute(LABEL, "block\nreturn")
                .attribute(COLOR, "orange")
                .build()
                .unwrap();
        } else {
            graph
                .create_edge(&b, &result, None)
                .attribute(LABEL, "block\nreturn")
                .attribute(COLOR, "orange")
                .build()
                .unwrap();
        }
        Ok((id, b))
    }
}

impl Visualise for SimpleInitialisation {
    fn render<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        let id = labeler.get();

        let node = graph
            .create_node(&id.to_string())
            .attribute(SHAPE, "box")
            .attribute(LABEL, "initialise")
            .build()
            .unwrap();

        let (typ_id, typ) = self.typ.render(graph, labeler)?;
        graph
            .create_edge(&node, &typ, Some("type"))
            .attribute(LABEL, "type")
            .attribute(vizoxide::attr::edge::COLOR, "green")
            .build()
            .unwrap();

        let (assignee_id, assignee) = self.assignee.render(graph, labeler)?;
        graph
            .create_edge(&node, &assignee, None)
            .attribute(LABEL, "assignee")
            .build()
            .unwrap();

        let (value_id, value) = self.value.render(graph, labeler)?;
        graph
            .create_edge(&node, &value, None)
            .attribute(LABEL, "val")
            .build()
            .unwrap();

        Ok((id, node))
    }
}
impl Visualise for ExpandedInitialisation {
    fn render<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        let (id_temp, temp) = self.temporary.render(graph, labeler)?;

        if let Some((head, tail)) = self.unpacked_assignments.split_first() {
            let (first, last) = chain_nodes(
                graph,
                labeler,
                &self.unpacked_assignments[..],
                assignment_chainer,
            )
            .expect(&format!("empty: {:?}", self.unpacked_assignments))?;
            graph
                .create_edge(&temp, &first, None)
                .attribute(LABEL, "assigning to")
                .attribute(COLOR, "blue")
                .build()
                .unwrap();
        }
        Ok((id_temp, temp))
    }
}
