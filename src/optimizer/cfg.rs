use std::collections::HashMap;

use petgraph::{
    Direction,
    graph::NodeIndex,
    prelude::StableGraph,
    visit::{Bfs, Dfs, EdgeRef, IntoNodeReferences, VisitMap},
};

use crate::ir::tac::{Label, ProcDecl, TACInst};
use crate::ir::{BasicBlock, BasicBlocks};
use crate::types::{InstBlock, Name};

pub struct CFG {
    pub graph: StableGraph<BasicBlock, TACInst>,
    name: Name,
    arguments: Option<Vec<Name>>,
}

impl CFG {
    pub fn serialize_tac(mut self) -> ProcDecl {
        let mut schedule: InstBlock = Vec::new();
        let mut schedule_order: Vec<NodeIndex> = Vec::with_capacity(self.graph.node_count());

        let name = self.name;
        let arguments = self.arguments;

        let Some(entry) = self.graph.node_indices().next() else {
            return ProcDecl {
                name,
                arguments,
                instructions: schedule,
            };
        };

        let mut bfs = Bfs::new(&self.graph, entry);

        while let Some(nx) = bfs.next(&self.graph) {
            schedule_order.push(nx);
        }

        for index in schedule_order {
            let Some(block) = self.graph.remove_node(index) else {
                continue;
            };

            schedule.push(TACInst::LabelDecl(block.block_label));
            for inst in block.instructions {
                schedule.push(inst);
            }
        }

        self.graph.clear();

        ProcDecl {
            name,
            arguments,
            instructions: schedule,
        }
    }

    pub fn remove_unreachable(&mut self) {
        let entry = match self.graph.node_indices().next() {
            Some(n) => n,
            None => return,
        };

        let mut dfs = Dfs::new(&self.graph, entry);

        while dfs.next(&self.graph).is_some() {}

        let unreachable_nodes: Vec<NodeIndex> = self
            .graph
            .node_indices()
            .filter(|&n| !dfs.discovered.is_visited(&n))
            .collect();

        for unreachable_node in unreachable_nodes {
            self.graph.remove_node(unreachable_node);
        }
    }

    pub fn coalesce_blocks(&mut self) {
        let nodes: Vec<NodeIndex> = self.graph.node_indices().collect();

        let mut successors: Vec<NodeIndex>;
        let mut predecessors: Vec<NodeIndex>;

        let mut i = 0usize;
        while i < nodes.len() {
            let idx = nodes[i];
            if self.graph.node_weight(idx).is_none() {
                i += 1;
                continue;
            }

            successors = self
                .graph
                .neighbors_directed(idx, Direction::Outgoing)
                .collect();

            if successors.len() != 1 {
                i += 1;
                continue;
            }

            let &successor = successors.first().unwrap();

            predecessors = self
                .graph
                .neighbors_directed(successor, Direction::Incoming)
                .collect();

            if predecessors.len() != 1 || *predecessors.first().unwrap() != idx {
                i += 1;
                continue;
            }

            let out_edges: Vec<(NodeIndex, TACInst)> = self
                .graph
                .edges_directed(successor, Direction::Outgoing)
                .map(|e| (e.target(), e.weight().clone()))
                .collect();

            let successor_block = match self.graph.remove_node(successor) {
                Some(b) => b,
                None => {
                    i += 1;
                    continue;
                }
            };

            let current_block = &mut self.graph[idx];

            if let Some(TACInst::UnconditionalJump(label)) = current_block.instructions.last()
                && *label == successor_block.block_label
            {
                current_block.pop_instruction();
            }

            // current_block.push_instruction(TACInst::LabelDecl(successor_block.block_label));

            for inst in successor_block.instructions {
                current_block.push_instruction(inst);
            }

            for (target, weight) in out_edges {
                if target == idx {
                    continue;
                }
                self.graph.add_edge(idx, target, weight);
            }
        }
    }

    pub fn jump_threading(&mut self) {}
}

impl From<BasicBlocks> for CFG {
    fn from(value: BasicBlocks) -> Self {
        let mut graph: StableGraph<BasicBlock, TACInst> = StableGraph::new();
        let mut edges: Vec<(NodeIndex, NodeIndex, TACInst)> = Vec::new();
        let mut label_to_index: HashMap<Label, NodeIndex> = HashMap::new();

        // Populate the nodes of the graph
        for block in value.blocks.into_iter() {
            let label = block.block_label.clone();
            let idx = graph.add_node(block);
            label_to_index.insert(label, idx);
        }

        // Go through instruction and find edges
        for (node_id, block) in graph.node_references() {
            for instruction in &block.instructions {
                use TACInst::*;
                match instruction {
                    UnconditionalJump(label)
                    | ConditionalJump {
                        destination: label, ..
                    } => {
                        if let Some(&dest_node_id) = label_to_index.get(label) {
                            edges.push((node_id, dest_node_id, instruction.clone()));
                        }
                    }
                    _ => {}
                }
            }
        }

        for edge in edges {
            graph.update_edge(edge.0, edge.1, edge.2);
        }

        Self {
            graph,
            name: value.proc_name,
            arguments: value.arguments,
        }
    }
}
