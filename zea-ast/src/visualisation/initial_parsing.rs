use crate::visualisation::{Labeler, RenderingNodeBuilder, Visualise, VisualizeResult};
use crate::zea::expression::Expression;
use crate::zea::statement::Statement;
use crate::zea::{
    expression::Literal, patterns::AssignmentPattern, Function, Initialisation, Module, StatementBlock, TopLevelStatement,
    Type, TypedIdentifier,
};
use vizoxide::attr::edge::LABEL;
use vizoxide::attr::node::{COLOR, FILLCOLOR, SHAPE};
use vizoxide::{Graph, Node};

pub(crate) fn render_block<'graph>(
    graph: &'graph Graph,
    labeler: &mut Labeler,
    block: &StatementBlock,
) -> VisualizeResult<'graph> {
    let id = labeler.get();
    let label = if block.statements.is_empty() {
        "empty block"
    } else {
        "block"
    };
    let node = RenderingNodeBuilder::new(graph, id)
        .label(label)
        .filled()
        .fillcolor("grey")
        .color("grey")
        .build();

    render_block_next(graph, labeler, &block.statements[..], &node)?;

    Ok((id, node))
}

fn render_block_next<'graph>(
    graph: &'graph Graph,
    labeler: &mut Labeler,
    block: &[Statement],
    prev: &Node<'graph>,
) -> Result<(), String> {
    if block.is_empty() {
        return Ok(());
    }
    let (stmt, rest) = (&block[0], &block[1..]);

    let edge_label = "block".to_string();
    let (id, node) = stmt.render(graph, labeler)?;
    graph
        .create_edge(prev, &node, None)
        .attribute(vizoxide::attr::edge::COLOR, "orange")
        .attribute(vizoxide::attr::edge::LABEL, &edge_label)
        .build()
        .unwrap();
    render_block_next(graph, labeler, rest, &node)?;
    Ok(())
}

impl Literal {
    fn node_label(&self) -> String {
        match self {
            Literal::Integer(i) => i.to_string(),
            Literal::Float(f) => f.to_string(),
            Literal::Boolean(b) => b.to_string(),
            Literal::String(s) => s.to_owned(),
        }
    }
}

impl Visualise for Literal {
    fn render<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        let s = labeler.get();
        let node = graph
            .create_node(&s.to_string())
            .attribute(vizoxide::attr::node::FILLCOLOR, "blue")
            .attribute(vizoxide::attr::node::COLOR, "blue")
            .attribute(SHAPE, "box")
            .attribute(LABEL, &self.node_label())
            .build()
            .unwrap();
        Ok((s, node))
    }
}

impl Expression {
    pub fn render_unit<'graph>(
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        let id = labeler.get();
        let node = RenderingNodeBuilder::new(graph, id)
            .label("unit")
            .filled()
            .fillcolor("purple")
            .shape("box")
            .build();
        Ok((id, node))
    }
    pub fn render_binary_operation<'graph>(
        &self,
        a: &Expression,
        b: &Expression,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        let id = labeler.get();
        let operator_label = match self {
            Expression::Add(..) => "+",
            Expression::Sub(..) => "-",
            Expression::Mod(..) => "%",
            Expression::Mul(..) => "*",
            Expression::Div(..) => "/",
            Expression::BitAnd(..) => "&",
            Expression::BitOr(..) => "|",
            Expression::LogAnd(..) => "and",
            Expression::LogOr(..) => "or",
            Expression::BitXor(..) => "^",
            Expression::LogXor(..) => "xor",
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
        arg: &Expression,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        let id = labeler.get();
        let operator_label = match self {
            Expression::BitNot(_) => "~",
            Expression::LogNot(_) => "!",
            Expression::Neg(_) => "-",
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

impl Visualise for Expression {
    fn render<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        match self {
            Expression::Add(a, b)
            | Expression::Sub(a, b)
            | Expression::Mod(a, b)
            | Expression::Mul(a, b)
            | Expression::Div(a, b)
            | Expression::BitAnd(a, b)
            | Expression::BitOr(a, b)
            | Expression::LogAnd(a, b)
            | Expression::LogOr(a, b)
            | Expression::BitXor(a, b)
            | Expression::LogXor(a, b) => {
                self.render_binary_operation(a.as_ref(), b.as_ref(), graph, labeler)
            }
            Expression::BitNot(arg) | Expression::LogNot(arg) | Expression::Neg(arg) => {
                self.render_unary_operation(arg.as_ref(), graph, labeler)
            }
            Expression::Literal(l) => l.render(graph, labeler),
            Expression::Ident(i) => i.render(graph, labeler),
            Expression::Block(block) => render_block(graph, labeler, block),
            Expression::Unit => Self::render_unit(graph, labeler),
            _ => {
                println!("bob");
                Err(format!("cannot yet render expression\n{self:?}\n"))
            }
        }
    }
}

impl Type {
    fn recurse_format(&self) -> String {
        match self {
            Type::Basic(t) => t.clone(),
            Type::ArrayOf(t) => "[".to_string() + &t.recurse_format() + "]",
            Type::Pointer(t) => "*".to_string() + &t.recurse_format(),
        }
    }
}

impl Visualise for Option<Type> {
    fn render<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        self.as_ref().render(graph, labeler)
    }
}

impl Visualise for Option<&Type> {
    fn render<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        let id = labeler.get();
        let builder = graph
            .create_node(&id.to_string())
            .attribute(vizoxide::attr::node::COLOR, "green")
            .attribute(vizoxide::attr::node::FILLCOLOR, "green")
            .attribute("style", "filled")
            .attribute(SHAPE, "diamond");
        let node = match *self {
            None => builder.attribute(LABEL, "TBI").build().unwrap(),
            Some(t) => builder
                .attribute(LABEL, &t.recurse_format())
                .build()
                .unwrap(),
        };

        Ok((id, node))
    }
}

impl AssignmentPattern {
    fn render_recursive<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
        parent_node: &Node<'graph>,
    ) -> VisualizeResult<'graph> {
        let id = labeler.get();

        let node = match self {
            AssignmentPattern::Identifier(i) => graph
                .create_node(&id.to_string())
                .attribute(LABEL, i)
                .build()
                .unwrap(),
            AssignmentPattern::Tuple(tup) => {
                let unpacking_node = graph
                    .create_node(&id.to_string())
                    .attribute(LABEL, "unpack")
                    .attribute(COLOR, "blue")
                    .build()
                    .unwrap();
                for (i, assignee) in tup.iter().enumerate() {
                    let (id, node) = assignee.render_recursive(graph, labeler, &unpacking_node)?;
                    graph
                        .create_edge(&unpacking_node, &node, None)
                        .attribute(LABEL, &("_".to_string() + &i.to_string()))
                        .attribute(vizoxide::attr::edge::COLOR, "blue")
                        .build()
                        .unwrap();
                }
                unpacking_node
            }
        };
        graph.create_edge(parent_node, &node, None).build().unwrap();
        Ok((id, node))
    }
}

impl Visualise for AssignmentPattern {
    fn render<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        let id = labeler.get();

        let node = graph
            .create_node(&id.to_string())
            .attribute(LABEL, "assign to")
            .build()
            .unwrap();

        let (inner_id, inner_node) = self.render_recursive(graph, labeler, &node)?;
        graph.create_edge(&node, &inner_node, None).build().unwrap();
        Ok((id, node))
    }
}

impl Visualise for Initialisation {
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

impl Visualise for Statement {
    fn render<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        match self {
            Statement::Initialisation(init) => init.render(graph, labeler),
            Statement::Return(expr) => {
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
            Statement::Block(block) => render_block(graph, labeler, block),
            _ => unimplemented!("cannot yet render statement:\n {self:?}\n"),
        }
    }
}

impl Function {
    fn render_paramlist<'graph>(
        graph: &'graph Graph,
        labeler: &mut Labeler,
        params: &Vec<TypedIdentifier>,
    ) -> VisualizeResult<'graph> {
        let id = labeler.get();
        let label = if params.is_empty() {
            "no params"
        } else {
            "params"
        };
        let paramlist_node = graph
            .create_node(&id.to_string())
            .attribute(LABEL, label)
            .attribute(FILLCOLOR, "grey")
            .attribute(COLOR, "grey")
            .build()
            .unwrap();

        for param in params {
            Self::render_param(graph, labeler, param, &paramlist_node)?
        }

        Ok((id, paramlist_node))
    }
    fn render_param<'graph>(
        graph: &'graph Graph,
        labeler: &mut Labeler,
        param: &TypedIdentifier,
        paramlist_node: &Node<'graph>,
    ) -> Result<(), String> {
        let id = labeler.get();
        let node = graph
            .create_node(&id.to_string())
            .attribute(LABEL, "param")
            .build()
            .unwrap();

        let (_, typ) = Some(param.typ()).render(graph, labeler)?;
        graph
            .create_edge(&node, &typ, None)
            .attribute(vizoxide::attr::edge::COLOR, "green")
            .build()
            .unwrap();
        let (_, name) = param.ident().to_string().render(graph, labeler)?;
        graph.create_edge(&node, &name, None).build().unwrap();
        graph
            .create_edge(paramlist_node, &node, None)
            .build()
            .unwrap();
        Ok(())
    }
}
impl Visualise for Function {
    fn render<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        let id = labeler.get();
        let label = self.name.clone() + "()";
        let node = graph
            .create_node(&id.to_string())
            .attribute(LABEL, &label)
            .attribute(COLOR, "grey")
            .attribute(FILLCOLOR, "grey")
            .build()
            .unwrap();

        let (_, returns) = Some(&self.returns).render(graph, labeler)?;
        graph
            .create_edge(&node, &returns, Some("returns"))
            .attribute(LABEL, "returns")
            .attribute(vizoxide::attr::edge::COLOR, "green")
            .build()
            .unwrap();

        let (_, params) = Self::render_paramlist(graph, labeler, &self.args)?;
        graph.create_edge(&node, &params, None).build().unwrap();

        render_block_next(graph, labeler, &self.body.as_slice(), &node)?;

        Ok((id, node))
    }
}

impl Module {
    fn render_imports<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        let id = labeler.get();
        let label = if self.imports.is_empty() {
            "no imports"
        } else {
            "imports"
        };
        let node = graph
            .create_node(&id.to_string())
            .attribute(LABEL, label)
            .build()
            .unwrap();
        for import in self.imports.iter() {
            let (_, import_node) = import.render(graph, labeler)?;
            graph
                .create_edge(&node, &import_node, None)
                .build()
                .unwrap();
        }
        Ok((id, node))
    }

    fn render_exports<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        let id = labeler.get();
        let label = if self.exports.is_empty() {
            "no exports"
        } else {
            "exports"
        };
        let node = graph
            .create_node(&id.to_string())
            .attribute(LABEL, label)
            .build()
            .unwrap();
        for export in self.exports.iter() {
            let (_, export_node) = export.render(graph, labeler)?;
            graph
                .create_edge(&node, &export_node, None)
                .build()
                .unwrap();
        }
        Ok((id, node))
    }
}

impl Visualise for TopLevelStatement {
    fn render<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        match self {
            TopLevelStatement::GlobalConst(init) => init.render(graph, labeler),
            TopLevelStatement::FuncDefinition(function) => function.render(graph, labeler),
        }
    }
}

impl Visualise for Module {
    fn render<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        let id = labeler.get();
        let node = graph
            .create_node(&id.to_string())
            .attribute(LABEL, "module")
            .attribute(FILLCOLOR, "purple")
            .build()
            .unwrap();

        let (_, imports) = self.render_imports(graph, labeler)?;
        graph.create_edge(&node, &imports, None).build().unwrap();

        let (_, exports) = self.render_exports(graph, labeler)?;
        graph.create_edge(&node, &exports, None).build().unwrap();

        for symbol in self.symbols.iter() {
            let (_, symbol_node) = symbol.render(graph, labeler)?;
            graph
                .create_edge(&node, &symbol_node, None)
                .attribute(LABEL, "symbol")
                .build()
                .unwrap();
        }
        Ok((id, node))
    }
}
