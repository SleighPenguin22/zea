#![allow(unused)]
pub mod first_lowering;
pub mod initial_parsing;

use vizoxide::attr::edge::LABEL;
use vizoxide::attr::graph::STYLE;
use vizoxide::attr::node::COLOR;
use vizoxide::layout::{apply_layout, Engine};
use vizoxide::render::{render_to_file, Format};
use vizoxide::{Context, Graph, GraphBuilder, Node, NodeBuilder};

pub struct Labeler {
    cur: usize,
}

impl Labeler {
    pub fn new() -> Self {
        Self { cur: 0 }
    }
    pub fn get(&mut self) -> usize {
        let cur = self.cur;
        self.cur += 1;
        cur
    }
}
struct RenderingNodeBuilder<'graph>(usize, NodeBuilder<'graph>);
impl<'graph> RenderingNodeBuilder<'graph> {
    pub fn fresh(graph: &'graph Graph, labeler: &mut Labeler) -> RenderingNodeBuilder<'graph> {
        let id = labeler.get();
        RenderingNodeBuilder(id, NodeBuilder::new(graph, &id.to_string()))
    }

    pub fn new(graph: &'graph Graph, id: usize) -> RenderingNodeBuilder<'graph> {
        RenderingNodeBuilder(id, NodeBuilder::new(graph, &id.to_string()))
    }

    pub fn filled(mut self) -> RenderingNodeBuilder<'graph> {
        Self(self.0, self.1.attribute("style", "filled"))
    }
    pub fn label<'a>(mut self, label: &'a str) -> RenderingNodeBuilder<'graph> {
        Self(self.0, self.1.attribute("label", label))
    }
    pub fn color<'a>(mut self, color: &'a str) -> RenderingNodeBuilder<'graph> {
        Self(self.0, self.1.attribute("color", color))
    }

    pub fn fillcolor<'a>(mut self, color: &'a str) -> RenderingNodeBuilder<'graph> {
        Self(self.0, self.1.attribute("fillcolor", color))
    }

    pub fn shape<'a>(mut self, shape: &'a str) -> RenderingNodeBuilder<'graph> {
        Self(self.0, self.1.attribute("shape", shape))
    }

    pub fn build(self) -> Node<'graph> {
        self.1.build().unwrap()
    }

    pub fn build_vis(self) -> VisualizeResult<'graph> {
        Ok((self.0, self.1.build().unwrap()))
    }
}

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
    let node = RenderingNodeBuilder::new(graph, id).label(label).build();
    Ok((id, node))
}

impl Visualise for String {
    fn render<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph> {
        basic_labeled_node(graph, labeler, self)
    }
}

pub fn graphify(dot_node: &impl Visualise, path: &str) -> Result<(), String> {
    let context = Context::new().unwrap();
    let mut g = GraphBuilder::new("expression")
        .attribute(vizoxide::attr::graph::RANKDIR, "TB")
        // .attribute(BGCOLOR, "black")
        .attribute(vizoxide::attr::graph::EDGE_COLOR, "white")
        .attribute(vizoxide::attr::graph::FONTCOLOR, "black")
        .attribute(vizoxide::attr::graph::NODE_COLOR, "blue")
        .attribute(STYLE, "filled")
        // .attribute(vizoxide::attr::graph::, "white")
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
    Ok(())
}

pub fn graphify_list(
    dot_node: &[impl Visualise],
    chainer: impl for<'graph> Fn(&'graph Graph, &Node<'graph>, &Node<'graph>),
    path: &str,
) -> Result<(), String> {
    let context = Context::new().unwrap();
    let mut g = GraphBuilder::new("expression")
        .attribute(vizoxide::attr::graph::RANKDIR, "TB")
        // .attribute(BGCOLOR, "black")
        .attribute(vizoxide::attr::graph::EDGE_COLOR, "white")
        .attribute(vizoxide::attr::graph::FONTCOLOR, "black")
        .attribute(vizoxide::attr::graph::NODE_COLOR, "blue")
        .attribute(STYLE, "filled")
        // .attribute(vizoxide::attr::graph::, "white")
        // .attribute(vizoxide::attr::graph::FONTNAME, "Helvetica")
        // .attribute(vizoxide::attr::graph::NODE_SHAPE, "box")
        .directed(true)
        .strict(true)
        .build()
        .unwrap();
    let mut labeler = Labeler::new();
    chain_nodes(&mut g, &mut labeler, dot_node, chainer);
    apply_layout(&context, &mut g, Engine::Dot);
    render_to_file(&context, &g, Format::Png, path).unwrap();
    Ok(())
}

pub trait Visualise {
    fn render<'graph>(
        &self,
        graph: &'graph Graph,
        labeler: &mut Labeler,
    ) -> VisualizeResult<'graph>;
}

/// Construct a chain of nodes.
///
/// # Arguments
///
/// * `graph`: the graph to construct the chain in
/// * `labeler`: the id generator
/// * `nodes`: the list of renderable objects to link
/// * `chainer`: a function that constructs and styles an edge between two adjacent nodes.
///
/// returns: Option<Result<(Node, Node), String>>
/// - `None` if `nodes` is empty
/// - `Some(Ok((first: Node, last: Node)))` if `nodes` is not empty
///     and every object was rendered succesfully.
/// - `Some(Err(String))` if `nodes` is not empty
///     and there was some object that could not be rendered
///
pub fn chain_nodes<'graph>(
    graph: &'graph Graph,
    labeler: &mut Labeler,
    nodes: &[impl Visualise],
    chainer: impl for<'chainer> Fn(&'chainer Graph, &Node<'chainer>, &Node<'chainer>),
) -> Option<Result<(Node<'graph>, Node<'graph>), String>> {
    let (head, tail) = nodes.split_first()?;
    let (head_id, head) = match head.render(graph, labeler) {
        Ok(tup) => tup,
        Err(e) => return Some(Err(e)),
    };

    let last = tail.iter().fold(
        Ok(head),
        |prev_node: Result<Node<'graph>, String>, cur_node| {
            let (_, cur_node) = cur_node.render(graph, labeler)?;
            chainer(graph, &prev_node?, &cur_node);
            Ok(cur_node)
        },
    );
    let last = match last {
        Ok(node) => node,
        Err(e) => return Some(Err(e)),
    };
    let head = graph.get_node(&head_id.to_string()).unwrap().unwrap();

    Some(Ok((head, last)))
}

pub fn basic_chainer<'graph>(graph: &'graph Graph, frm: &Node<'graph>, to: &Node<'graph>) {
    graph.create_edge(frm, to, None).build().unwrap();
}

pub fn block_chainer<'graph>(graph: &'graph Graph, frm: &Node<'graph>, to: &Node<'graph>) {
    graph
        .create_edge(frm, to, None)
        .attribute(COLOR, "orange")
        .attribute(LABEL, "block")
        .build()
        .unwrap();
}
pub fn assignment_chainer<'graph>(graph: &'graph Graph, frm: &Node<'graph>, to: &Node<'graph>) {
    graph
        .create_edge(frm, to, None)
        .attribute(COLOR, "blue")
        .attribute(LABEL, "then")
        .build()
        .unwrap();
}
