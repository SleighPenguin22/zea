use indexmap::IndexSet;
use std::borrow::Cow;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct BLockId(usize);

#[derive(Debug)]
struct ControlFlowItem {
    pub block_id: BLockId,
    pub node_id: usize,
    /// All the blocks that this block might jump to
    successors: Vec<BLockId>,
    kind: ControlFlowItemKind,
}

#[derive(Debug)]
enum ControlFlowItemKind {
    Statement,
    Block,
    FunctionBody,
    Branch,
}

struct NodeDag {
    block_arena: Vec<ControlFlowItem>,
}

fn vector_replace_block_ids(vec: &mut Vec<BLockId>, from: BLockId, to: BLockId) {
    for id in vec.iter_mut() {
        if *id == from {
            *id = to;
        }
    }
}

fn replace_block_id(id: &mut BLockId, from: BLockId, to: BLockId) {
    if *id == from {
        *id = to;
    }
}

impl NodeDag {
    const fn new() -> Self {
        Self {
            block_arena: Vec::new(),
        }
    }

    fn update_referees(&mut self, node_id: BLockId, to: BLockId) {
        for node in self.block_arena.iter_mut() {
            vector_replace_block_ids(&mut node.successors, node_id, to)
        }
    }

    fn follow(&self, id: BLockId) -> Option<&ControlFlowItem> {
        self.block_arena.iter().find(|node| node.block_id == id)
    }

    fn predecessors(&self, id: BLockId) -> IndexSet<BLockId> {
        self.block_arena
            .iter()
            .filter_map(|node| {
                node.successors
                    .iter()
                    .find_map(|i| (*i == id).then_some(&node.block_id))
            })
            .cloned()
            .collect()
    }

    fn recursive_predecessors(&self, id: BLockId) -> IndexSet<BLockId> {
        let mut res: IndexSet<_> = self.predecessors(id);

        loop {
            let before = res.len();
            let preds: Vec<_> = res.iter().flat_map(|id| self.predecessors(*id)).collect();
            res.extend(preds);
            let after = res.len();
            if after == before {
                break;
            }
        }

        res
    }

    fn successors(&self, id: BLockId) -> IndexSet<BLockId> {
        self.follow(id)
            .into_iter()
            .flat_map(|id| id.successors.iter())
            .cloned()
            .collect()
    }
    fn recursive_successors(&self, id: BLockId) -> IndexSet<BLockId> {
        let mut res: IndexSet<_> = self.successors(id);
        loop {
            let before = res.len();
            let descendants: Vec<_> = res.iter().flat_map(|id| self.successors(*id)).collect();
            res.extend(descendants);
            let after = res.len();
            if after == before {
                break;
            }
        }

        res
    }

    fn indirectly_recursive(&self, id: BLockId) -> bool {
        self.recursive_successors(id).contains(&id)
    }

    fn directly_recursive(&self, id: BLockId) -> bool {
        self.successors(id).contains(&id)
    }

    fn dfs_construct_partition(&self, id: BLockId) -> IndexSet<BLockId> {
        let mut res = self.recursive_predecessors(id);
        let mut frontier: IndexSet<_> = IndexSet::from_iter(res.clone());

        while let Some(cur) = frontier.pop() {
            let mut new = self.recursive_successors(cur);
            let preds = self.recursive_predecessors(cur);
            new.extend(preds);
            let new: IndexSet<_> = new.difference(&res).cloned().collect();
            res.extend(new.iter());
            frontier.extend(new.iter())
        }

        res
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn dfs_construct_partition_stops() {
        let _graph = todo!();
    }
}
