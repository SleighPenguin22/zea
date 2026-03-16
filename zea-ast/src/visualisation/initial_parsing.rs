use crate::visualisation::{Labeler, RenderingNodeBuilder, Visualise, VisualizeResult};
use crate::zea::Statement;
use crate::zea::StatementBlock;
use vizoxide::{Graph, Node};
// 
// pub(crate) fn render_block<'graph>(
//     graph: &'graph Graph,
//     labeler: &mut Labeler,
//     block: &StatementBlock,
// ) -> VisualizeResult<'graph> {
//     let id = labeler.get();
//     let label = if block.statements.is_empty() {
//         "empty block"
//     } else {
//         "block"
//     };
//     let node = RenderingNodeBuilder::new(graph, id)
//         .label(label)
//         .filled()
//         .fillcolor("grey")
//         .color("grey")
//         .build();
// 
//     render_block_next(graph, labeler, &block.statements[..], &node)?;
// 
//     Ok((id, node))
// }
// 
// fn render_block_next<'graph>(
//     graph: &'graph Graph,
//     labeler: &mut Labeler,
//     block: &[Statement],
//     prev: &Node<'graph>,
// ) -> Result<(), String> {
//     if block.is_empty() {
//         return Ok(());
//     }
//     let (stmt, rest) = (&block[0], &block[1..]);
// 
//     let edge_label = "block".to_string();
//     let (id, node) = stmt.render(graph, labeler)?;
//     graph
//         .create_edge(prev, &node, None)
//         .attribute(vizoxide::attr::edge::COLOR, "orange")
//         .attribute(vizoxide::attr::edge::LABEL, &edge_label)
//         .build()
//         .unwrap();
//     render_block_next(graph, labeler, rest, &node)?;
//     Ok(())
// }
