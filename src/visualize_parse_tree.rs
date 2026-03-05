use crate::ast::{AssignmentPattern, Expression, Initialisation, Literal, Statement, Type};
use vizoxide::attr::node::SHAPE as NODE_SHAPE;
use vizoxide::attr::node::{COLOR, FILLCOLOR, LABEL as dot_node_label};
use vizoxide::layout::{apply_layout, Engine};
use vizoxide::render::{render_to_file, Format};
use vizoxide::{Context, Graph, GraphBuilder, Node};

fn basic_labeled_node<'a>(graph: &'a Graph, id: usize, label: &str) -> Node<'a> {
    graph
        .create_node(&id.to_string())
        .attribute(dot_node_label, label)
        .build()
        .unwrap()
}
pub fn graphify(dot_node: impl DotNode, path: &str) {
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
    dot_node.render(&mut g, &mut labeler);
    apply_layout(&context, &mut g, Engine::Dot);
    render_to_file(&context, &g, Format::Png, path).unwrap();
}

#[repr(transparent)]
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
    fn render<'a>(&self, graph: &'a Graph, labeler: &mut Labeler) -> (usize, Node<'a>);
}

fn render_block<'a>(
    graph: &'a Graph,
    labeler: &mut Labeler,
    block: &Vec<Statement>,
) -> (usize, Node<'a>) {
    let id = labeler.get();
    let node = graph
        .create_node(&id.to_string())
        .attribute(dot_node_label, "block")
        .attribute(COLOR, "grey")
        .attribute(FILLCOLOR, "grey")
        .build()
        .unwrap();

    for statement in block {
        let (stmt_id, stmt) = statement.render(graph, labeler);
        graph.create_edge(&node, &stmt, None).build().unwrap();
    }
    (id, node)
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
    fn render<'a>(&self, graph: &'a Graph, labeler: &mut Labeler) -> (usize, Node<'a>) {
        let s = labeler.get();
        let node = graph
            .create_node(&s.to_string())
            .attribute(vizoxide::attr::node::FILLCOLOR, "blue")
            .attribute(vizoxide::attr::node::COLOR, "blue")
            .attribute(NODE_SHAPE, "box")
            .attribute(dot_node_label, &self.node_label())
            .build()
            .unwrap();
        (s, node)
    }
}

impl DotNode for String {
    fn render<'a>(&self, graph: &'a Graph, labeler: &mut Labeler) -> (usize, Node<'a>) {
        let id = labeler.get();
        let node = basic_labeled_node(graph, id, self);
        (id, node)
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
    fn render_unit<'a>(&self, graph: &'a Graph, labeler: &mut Labeler) -> (usize, Node<'a>) {
        let s = labeler.get();
        (s, basic_labeled_node(graph, s, "unit"))
    }
    fn render_binary_operation<'a>(
        &self,
        a: &Expression,
        b: &Expression,
        graph: &'a Graph,
        labeler: &mut Labeler,
    ) -> (usize, Node<'a>) {
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
        let (a_id, a) = a.render(graph, labeler);
        let (b_id, b) = b.render(graph, labeler);

        graph.create_edge(&node, &a, None).build().unwrap();
        graph.create_edge(&node, &b, None).build().unwrap();
        (id, node)
    }

    fn render_unary_operation<'a>(
        &self,
        arg: &Expression,
        graph: &'a Graph,
        labeler: &mut Labeler,
    ) -> (usize, Node<'a>) {
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
        let (arg_id, arg) = arg.render(graph, labeler);

        graph.create_edge(&node, &arg, None).build().unwrap();
        (id, node)
    }
}

impl DotNode for Expression {
    fn render<'a>(&self, graph: &'a Graph, labeler: &mut Labeler) -> (usize, Node<'a>) {
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
            _ => unimplemented!("render expression\n{self:?}\n"),
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
    fn render<'a>(&self, graph: &'a Graph, labeler: &mut Labeler) -> (usize, Node<'a>) {
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

        (id, node)
    }
}

impl AssignmentPattern {
    fn render_recursive<'a>(
        &self,
        graph: &'a Graph,
        labeler: &mut Labeler,
        parent_node: &Node<'a>,
    ) -> (usize, Node<'a>) {
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
                for assignee in tup {
                    let (id, node) = assignee.render_recursive(graph, labeler, &unpacking_node);
                    graph
                        .create_edge(&unpacking_node, &node, None)
                        .build()
                        .unwrap();
                }
                unpacking_node
            }
        };
        graph.create_edge(parent_node, &node, None).build().unwrap();
        (id, node)
    }
}

impl DotNode for AssignmentPattern {
    fn render<'a>(&self, graph: &'a Graph, labeler: &mut Labeler) -> (usize, Node<'a>) {
        let id = labeler.get();

        let node = graph
            .create_node(&id.to_string())
            .attribute(dot_node_label, "assign to")
            .build()
            .unwrap();

        let (inner_id, inner_node) = self.render_recursive(graph, labeler, &node);
        graph.create_edge(&node, &inner_node, None).build().unwrap();
        (id, node)
    }
}

impl DotNode for Initialisation {
    fn render<'a>(&self, graph: &'a Graph, labeler: &mut Labeler) -> (usize, Node<'a>) {
        let id = labeler.get();

        let node = graph
            .create_node(&id.to_string())
            .attribute(NODE_SHAPE, "box")
            .attribute(dot_node_label, "initialise")
            .build()
            .unwrap();

        let (typ_id, typ) = self.typ.render(graph, labeler);
        graph
            .create_edge(&node, &typ, Some("type"))
            .build()
            .unwrap();

        let (assignee_id, assignee) = self.assignee.render(graph, labeler);
        graph.create_edge(&node, &assignee, None).build().unwrap();

        let (value_id, value) = self.value.render(graph, labeler);
        graph.create_edge(&node, &value, None).build().unwrap();

        (id, node)
    }
}

impl DotNode for Statement {
    fn render<'a>(&self, graph: &'a Graph, labeler: &mut Labeler) -> (usize, Node<'a>) {
        match self {
            Statement::Initialisation(init) => init.render(graph, labeler),
            Statement::Return(expr) => {
                let id = labeler.get();
                let node = graph
                    .create_node(&id.to_string())
                    .attribute(dot_node_label, "return")
                    .build()
                    .unwrap();

                let (_, inner) = expr.render(graph, labeler);
                graph.create_edge(&node, &inner, None).build().unwrap();
                (id, node)
            }
            Statement::Block(block) => render_block(graph, labeler, block),
            _ => unimplemented!("cannot yet render statement:\n {self:?}\n"),
        }
    }
}
