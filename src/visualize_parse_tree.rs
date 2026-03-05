use crate::ast::{
    AssignmentPattern, Expression, Function, Initialisation, Literal, Module, Statement,
    TopLevelStatement, Type, TypedIdentifier,
};
use vizoxide::attr::edge::LABEL;
use vizoxide::attr::node::SHAPE as NODE_SHAPE;
use vizoxide::attr::node::{COLOR, FILLCOLOR, LABEL as dot_node_label};
use vizoxide::layout::{apply_layout, Engine};
use vizoxide::render::{render_to_file, Format};
use vizoxide::{Context, Graph, GraphBuilder, Node};

macro_rules! err_unimplemented {
    ($lit:literal) => {{
        VisualizeResult::Err(format!($lit))
    }};
    ($lit:literal, $($e:expr),+) => {
        VisualizeResult::Err(format!($lit, $($e),+))
    }
}

pub type VisualizeResult<'graph> = Result<(usize, Node<'graph>), String>;

fn basic_labeled_node<'graph>(
    graph: &'graph Graph,
    labeler: &mut Labeler,
    label: &str,
) -> VisualizeResult<'graph> {
    let id = labeler.get();
    let node = graph
        .create_node(&id.to_string())
        .attribute(dot_node_label, label)
        .build()
        .unwrap();
    Ok((id, node))
}
pub fn graphify(dot_node: &impl DotNode, path: &str) -> Result<(), String> {
    let context = Context::new().unwrap();
    let mut g = GraphBuilder::new("expression")
        .attribute(vizoxide::attr::graph::RANKDIR, "TB")
        // .attribute(vizoxide::attr::graph::FONTNAME, "Helvetica")
        // .attribute(vizoxide::attr::graph::NODE_SHAPE, "box")
        .directed(true)
        .strict(true)
        .build()
        .unwrap();
    let mut labeler = Labeler::new();
    dot_node.render(&mut g, &mut labeler)?;
    apply_layout(&context, &mut g, Engine::Dot);
    render_to_file(&context, &g, Format::Png, path).unwrap();
    println!("{labeler:?}");
    Ok(())
}

#[repr(transparent)]
#[derive(Debug)]
pub struct Labeler(usize);
impl Labeler {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn get(&mut self) -> usize {
        let n = self.0;
        self.0 += 1;
        n
    }
}
pub trait DotNode {
    fn render<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph>;
}

fn render_block<'graph>(
    graph: &'graph Graph,
    labeler: &mut Labeler,
    block: &Vec<Statement>,
) -> VisualizeResult<'graph> {
    let id = labeler.get();
    let label = if block.is_empty() {
        "empty block"
    } else {
        "block"
    };
    let node = graph
        .create_node(&id.to_string())
        .attribute(dot_node_label, label)
        .attribute(COLOR, "grey")
        .attribute(FILLCOLOR, "grey")
        .build()
        .unwrap();

    render_block_next(graph, labeler, &block[..], &node)?;

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
        .attribute(vizoxide::attr::edge::COLOR, "pink")
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

impl DotNode for Literal {
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
            .attribute(NODE_SHAPE, "box")
            .attribute(dot_node_label, &self.node_label())
            .build()
            .unwrap();
        Ok((s, node))
    }
}

impl DotNode for String {
    fn render<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        basic_labeled_node(graph, labeler, self)
    }
}

impl Expression {
    fn is_binary_operation(&self) -> bool {
        use Expression as E;
        match self {
            E::Unit
            | E::Literal(_)
            | E::Ident(_)
            | E::Neg(_)
            | E::LogNot(_)
            | E::BitNot(_)
            | E::IfThenElse(..)
            | E::FuncCall(_)
            | E::ConditionMatch(_)
            | E::PatternMatch(_)
            | E::Block(_) => false,
            _ => true,
        }
    }

    fn is_unary_operation(&self) -> bool {
        match self {
            Expression::BitNot(_) | Expression::LogNot(_) | Expression::Neg(_) => true,
            _ => false,
        }
    }
    fn render_unit<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        basic_labeled_node(graph, labeler, "unit")
    }
    fn render_binary_operation<'graph>(
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
            .attribute(dot_node_label, operator_label)
            .attribute(NODE_SHAPE, "diamond")
            .build()
            .unwrap();
        let (a_id, a) = a.render(graph, labeler)?;
        let (b_id, b) = b.render(graph, labeler)?;

        graph.create_edge(&node, &a, None).build().unwrap();
        graph.create_edge(&node, &b, None).build().unwrap();
        Ok((id, node))
    }

    fn render_unary_operation<'graph>(
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
            .attribute(dot_node_label, operator_label)
            .build()
            .unwrap();
        let (arg_id, arg) = arg.render(graph, labeler)?;

        graph.create_edge(&node, &arg, None).build().unwrap();
        Ok((id, node))
    }
}

impl DotNode for Expression {
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
            Expression::Unit => self.render_unit(graph, labeler),
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

impl DotNode for Option<Type> {
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
            .attribute(NODE_SHAPE, "diamond");
        let node = match self {
            None => builder
                .attribute(dot_node_label, "TO BE INFERRED")
                .build()
                .unwrap(),
            Some(t) => builder
                .attribute(dot_node_label, &t.recurse_format())
                .build()
                .unwrap(),
        };

        Ok((id, node))
    }
}

impl DotNode for Option<&Type> {
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
            .attribute(NODE_SHAPE, "diamond");
        let node = match self {
            None => builder
                .attribute(dot_node_label, "TO BE INFERRED")
                .build()
                .unwrap(),
            Some(t) => builder
                .attribute(dot_node_label, &t.recurse_format())
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
                .attribute(dot_node_label, i)
                .build()
                .unwrap(),
            AssignmentPattern::Tuple(tup) => {
                let unpacking_node = graph
                    .create_node(&id.to_string())
                    .attribute(dot_node_label, "unpack")
                    .build()
                    .unwrap();
                for (i, assignee) in tup.iter().enumerate() {
                    let (id, node) = assignee.render_recursive(graph, labeler, &unpacking_node)?;
                    graph
                        .create_edge(&unpacking_node, &node, None)
                        .attribute(LABEL, &("_".to_string() + &i.to_string()))
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

impl DotNode for AssignmentPattern {
    fn render<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        let id = labeler.get();

        let node = graph
            .create_node(&id.to_string())
            .attribute(dot_node_label, "assign to")
            .build()
            .unwrap();

        let (inner_id, inner_node) = self.render_recursive(graph, labeler, &node)?;
        graph.create_edge(&node, &inner_node, None).build().unwrap();
        Ok((id, node))
    }
}

impl DotNode for Initialisation {
    fn render<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        let id = labeler.get();

        let node = graph
            .create_node(&id.to_string())
            .attribute(NODE_SHAPE, "box")
            .attribute(dot_node_label, "initialise")
            .build()
            .unwrap();

        let (typ_id, typ) = self.typ.render(graph, labeler)?;
        graph
            .create_edge(&node, &typ, Some("type"))
            .attribute(LABEL, "type")
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

impl DotNode for Statement {
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
                    .attribute(dot_node_label, "return")
                    .attribute(NODE_SHAPE, "box")
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
            .attribute(dot_node_label, label)
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
            .attribute(dot_node_label, "param")
            .build()
            .unwrap();

        let (_, typ) = Some(param.typ()).render(graph, labeler)?;
        graph.create_edge(&node, &typ, None).build().unwrap();
        let (_, name) = param.ident().to_string().render(graph, labeler)?;
        graph.create_edge(&node, &name, None).build().unwrap();
        graph
            .create_edge(paramlist_node, &node, None)
            .build()
            .unwrap();
        Ok(())
    }
}
impl DotNode for Function {
    fn render<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        let id = labeler.get();
        let label = self.name.clone() + "()";
        let node = graph
            .create_node(&id.to_string())
            .attribute(dot_node_label, &label)
            .attribute(COLOR, "grey")
            .attribute(FILLCOLOR, "grey")
            .build()
            .unwrap();

        let (_, returns) = Some(&self.returns).render(graph, labeler)?;
        graph
            .create_edge(&node, &returns, Some("returns"))
            .attribute(LABEL, "returns")
            .build()
            .unwrap();

        let (_, params) = Self::render_paramlist(graph, labeler, &self.args)?;
        graph.create_edge(&node, &params, None).build().unwrap();

        render_block_next(graph, labeler, &self.body, &node)?;

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
            .attribute(dot_node_label, label)
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
            .attribute(dot_node_label, label)
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

impl DotNode for TopLevelStatement {
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

impl DotNode for Module {
    fn render<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        let id = labeler.get();
        let node = graph
            .create_node(&id.to_string())
            .attribute(dot_node_label, "module")
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
