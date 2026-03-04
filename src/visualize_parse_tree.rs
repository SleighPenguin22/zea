use crate::ast::{Expression, Literal};
use vizoxide::layout::{apply_layout, Engine};
use vizoxide::render::{render_to_file, Format};
use vizoxide::{Context, Graph, GraphBuilder, Node};

pub trait DotNode {
    fn node_id(&self, labeler: &mut usize) -> String {
        let s = labeler.to_string();
        *labeler += 1;
        s
    }
    fn node_label(&self) -> String;
    fn render_node<'a>(&self, graph: &'a Graph, labeler: &mut usize) -> Node<'a>;

    fn render_children(&self, graph: &Graph, labeler: &mut usize, self_label: &str) {}
}

impl DotNode for Literal {
    fn node_label(&self) -> String {
        match self {
            Literal::Integer(i) => i.to_string(),
            Literal::Float(f) => f.to_string(),
            Literal::Boolean(b) => b.to_string(),
            Literal::String(s) => s.to_owned(),
        }
    }

    fn render_node<'a>(&self, graph: &'a Graph, labeler: &mut usize) -> Node<'a> {
        let s = self.node_id(labeler);

        graph
            .create_node(&s)
            .attribute(vizoxide::attr::node::COLOR, "blue")
            .attribute(vizoxide::attr::node::LABEL, &self.node_label())
            .build()
            .unwrap()
    }
}

impl DotNode for Expression {
    fn node_label(&self) -> String {
        match self {
            Expression::Unit => "unit".to_string(),
            Expression::Ident(i) => i.to_owned(),
            Expression::Literal(l) => l.node_label(),
            Expression::Add(_, _) => "+".to_string(),
            _ => unimplemented!("dot repr for expression {self:?}"),
        }
    }
    fn render_node<'a>(&self, graph: &'a Graph, labeler: &mut usize) -> Node<'a> {
        let s = self.node_id(labeler);
        let node = graph
            .create_node(&s)
            .attribute(vizoxide::attr::node::COLOR, "purple")
            .attribute(vizoxide::attr::node::LABEL, &self.node_label())
            .build()
            .unwrap();

        self.render_children(graph, labeler, &s);
        node
    }

    fn render_children(&self, graph: &Graph, labeler: &mut usize, self_label: &str) {
        match self {
            Expression::Literal(_) | Expression::Ident(_) | Expression::Unit => {}
            Expression::Add(a, b) => {
                let parent = graph
                    .get_node(self_label)
                    .unwrap()
                    .expect(&format!("node with label {self_label} does not exist"));
                let a = a.render_node(graph, labeler);
                let b = b.render_node(graph, labeler);

                graph.create_edge(&parent, &a, None).build().unwrap();
                graph.create_edge(&parent, &b, None).build().unwrap();
            }
            _ => unimplemented!("rendering children for expression {self:?}"),
        }
    }
}

pub fn graphify_expression(expr: Expression, path: &str) {
    let context = Context::new().unwrap();
    let mut g = GraphBuilder::new("expression")
        .attribute(vizoxide::attr::graph::RANKDIR, "TB")
        .attribute(vizoxide::attr::graph::FONTNAME, "Helvetica")
        .attribute(vizoxide::attr::graph::NODE_SHAPE, "box")
        .directed(true)
        .build()
        .unwrap();
    let mut labeler = 0;
    expr.render_node(&mut g, &mut labeler);
    apply_layout(&context, &mut g, Engine::Dot);
    render_to_file(&context, &g, Format::Png, path).unwrap();
}
