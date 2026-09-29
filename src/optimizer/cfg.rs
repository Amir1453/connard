use std::collections::HashMap;

use petgraph::Direction;
use petgraph::graph::NodeIndex;
use petgraph::prelude::StableGraph;
use petgraph::visit::{Bfs, Dfs, EdgeRef, IntoNodeReferences, VisitMap};

use crate::ir::tac::{InstBlock, Label, ProcDecl, TACInst, TACJumpOpcode};
use crate::ir::{BasicBlock, BasicBlocks};
use crate::types::Name;

pub struct CFG {
    pub graph: StableGraph<BasicBlock, usize>,
    pub name: Name,
    pub arguments: Option<Vec<Name>>,
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

            let out_edges: Vec<(NodeIndex, usize)> = self
                .graph
                .edges_directed(successor, Direction::Outgoing)
                .map(|e| (e.target(), *e.weight()))
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

    pub fn jump_threading(&mut self) {
        use TACInst::*;

        let nodes: Vec<NodeIndex> = self.graph.node_indices().collect();

        let mut i = 0usize;
        while i < nodes.len() {
            let idx = nodes[i];
            if self.graph.node_weight(idx).is_none() {
                i += 1;
                continue;
            }

            // Get the outgoing edges of our current block.
            let out_edges: Vec<(NodeIndex, usize)> = self
                .graph
                .edges_directed(idx, Direction::Outgoing)
                .map(|e| (e.target(), *e.weight()))
                .collect();

            // We continue for every edge instruction that is a ConditionalJump.
            for (successor_idx, pred_instr_idx) in out_edges {
                let instr = self.graph[idx].instructions[pred_instr_idx].clone();
                let (opcode1, cond1, _dest1) = match instr {
                    ConditionalJump {
                        opcode,
                        condition,
                        destination,
                    } => (opcode, condition, destination),

                    _ => continue,
                };

                // If the edge is a ConditionalJump, get the predecessors of the block the
                // ConditionalJump points to (successor block).
                // We only work with one predecessor for now.
                if self
                    .graph
                    .neighbors_directed(successor_idx, Direction::Incoming)
                    .count()
                    != 1
                {
                    continue;
                }

                // Now, get the outgoing edges of our successor block. We will be changing all the
                // edges of the successor block that can be changed via opcode matching.
                let out_edges_successor: Vec<(NodeIndex, usize)> = self
                    .graph
                    .edges_directed(successor_idx, Direction::Outgoing)
                    .map(|e| (e.target(), *e.weight()))
                    .collect();

                // Go through the successor edges, and once again match against ConditionalJump
                // instructions.
                for (_successor2_idx, succ_instr_idx) in out_edges_successor {
                    let suc_instr = self.graph[successor_idx].instructions[succ_instr_idx].clone();
                    let (opcode2, cond2, dest2) = match suc_instr {
                        ConditionalJump {
                            opcode,
                            condition,
                            destination,
                        } => (opcode, condition, destination),

                        _ => continue,
                    };

                    // If the conditions are not the same, or if the successor block is modifying
                    // the condition, we cannot make this optimization.
                    if cond1 != cond2
                        || self.graph[successor_idx]
                            .instructions
                            .iter()
                            .any(|instr| instr.modifies_temp(&cond2))
                    {
                        continue;
                    }

                    // Match against the opcodes we have to see what we can do. We will have to
                    // modify the instructions inside of the successor block, as well as updating
                    // the edges.
                    let taken_or_not = jthread_match_opcode(opcode1, opcode2);

                    // This is the branch where the ConditionalJump is always taken.
                    if matches!(taken_or_not, TakenOrNot::AlwaysTaken) {
                        let successor_block = &mut self.graph[successor_idx];

                        // We replace our ConditionalJump instruction with an UnconditionalJump.
                        // Now, we can get rid of the rest of the block body after the
                        // ConditionalJump, since it will not be reached.
                        let new_instr = UnconditionalJump(dest2);
                        successor_block.instructions[succ_instr_idx] = new_instr;
                        successor_block.instructions.truncate(succ_instr_idx + 1);

                        // Since we got rid of the instructions, we get rid of the edges associated
                        // to those instructions as well.
                        let edges_to_remove: Vec<_> = self
                            .graph
                            .edges_directed(successor_idx, Direction::Outgoing)
                            .filter(|edge| *edge.weight() > succ_instr_idx)
                            .map(|edge| edge.id())
                            .collect();

                        for edge_idx in edges_to_remove {
                            self.graph.remove_edge(edge_idx);
                        }
                    }

                    // This is the branch where the ConditionalJump is never taken.
                    if matches!(taken_or_not, TakenOrNot::NeverTaken) {
                        let successor_block = &mut self.graph[successor_idx];

                        // We remove the ConditionalJump instruction, since it will never be taken.
                        successor_block.instructions.remove(succ_instr_idx);

                        // We have to remove the edge associated to this ConditionalJump insruction.
                        let edges_to_remove: Vec<_> = self
                            .graph
                            .edges_directed(successor_idx, Direction::Outgoing)
                            .filter(|edge| *edge.weight() == succ_instr_idx)
                            .map(|edge| edge.id())
                            .collect();

                        for edge_idx in edges_to_remove {
                            self.graph.remove_edge(edge_idx);
                        }

                        // Now we have to update all the affected edges.
                        let edges_to_update: Vec<_> = self
                            .graph
                            .edges_directed(successor_idx, Direction::Outgoing)
                            .filter(|edge| *edge.weight() > succ_instr_idx)
                            .map(|edge| edge.id())
                            .collect();

                        for edge_idx in edges_to_update {
                            if let Some(edge) = self.graph.edge_weight_mut(edge_idx) {
                                *edge -= 1;
                            }
                        }
                    }

                    break;
                }
            }

            i += 1;
        }
    }
}

fn jthread_match_opcode(opcode1: TACJumpOpcode, opcode2: TACJumpOpcode) -> TakenOrNot {
    // Same opcode implies always taken
    if opcode1 == opcode2 {
        return TakenOrNot::AlwaysTaken;
    }

    use TACJumpOpcode::*;
    match (opcode1, opcode2) {
        // x < 0 -> x != 0, x <= 0.
        // x > 0 -> x != 0, x >= 0.
        (JL, JNZ | JLE) | (JNLE, JNZ | JNL) => TakenOrNot::AlwaysTaken,

        // x < 0 -> not x = 0, not x > 0, not x >= 0.
        // x > 0 -> not x = 0, not x < 0, not x <= 0.
        (JL, JZ | JNL | JNLE) | (JNLE, JZ | JL | JLE) => TakenOrNot::NeverTaken,

        // x >= 0 -> not x < 0.
        // x <= 0 -> not x > 0.
        (JNL, JL) | (JLE, JNLE) => TakenOrNot::NeverTaken,

        // Rest we dont care
        _ => TakenOrNot::Unknown,
    }
}

enum TakenOrNot {
    AlwaysTaken,
    NeverTaken,
    Unknown,
}

impl From<BasicBlocks> for CFG {
    fn from(value: BasicBlocks) -> Self {
        let mut graph: StableGraph<BasicBlock, usize> = StableGraph::new();
        let mut edges: Vec<(NodeIndex, NodeIndex, usize)> = Vec::new();
        let mut label_to_index: HashMap<Label, NodeIndex> = HashMap::new();

        // Populate the nodes of the graph
        for block in value.blocks.into_iter() {
            let label = block.block_label.clone();
            let idx = graph.add_node(block);
            label_to_index.insert(label, idx);
        }

        // Go through instruction and find edges
        for (node_id, block) in graph.node_references() {
            for (idx, instruction) in block.instructions.iter().enumerate() {
                use TACInst::*;
                match instruction {
                    UnconditionalJump(label)
                    | ConditionalJump {
                        destination: label, ..
                    } => {
                        if let Some(&dest_node_id) = label_to_index.get(label) {
                            edges.push((node_id, dest_node_id, idx));
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

#[cfg(test)]
mod tests {
    use crate::ir::tac::TACJumpOpcode;
    use crate::ir::tac_builder::ProcBuilder;
    use crate::types::Name;

    #[test]
    fn jump_threading_always_taken_example() {
        let mut proc = ProcBuilder::new(Name::from("class"));
        proc.add_argument("x");

        proc.constant("zero_1", 0)
            .branch_named(TACJumpOpcode::JNLE, "zero_1", "l0")
            .jump("l1");

        proc.label_decl("l1");

        proc.constant("error_value", -69)
            .copy("error_value", "returned_value")
            .jump("ret");

        proc.label_decl("l0");

        proc.branch_named(TACJumpOpcode::JNLE, "zero_1", "l3")
            .jump("ret");

        proc.label_decl("l3");
        proc.constant("error_value", -69)
            .copy("error_value", "returned_value")
            .jump("ret");

        proc.label_decl("ret");
        proc.return_void();

        let procedure = proc.build();

        eprintln!("{procedure}");

        let mut cfg = super::CFG::from(super::BasicBlocks::from(procedure));
        cfg.jump_threading();

        eprintln!("{}", cfg.serialize_tac());
    }

    #[test]
    fn jump_threading_never_taken_example() {
        let mut proc = ProcBuilder::new(Name::from("class"));
        proc.add_argument("x");

        proc.constant("zero_1", 0)
            .branch_named(TACJumpOpcode::JNLE, "zero_1", "l0")
            .jump("l1");

        proc.label_decl("l1");

        proc.constant("error_value", -69)
            .copy("error_value", "returned_value")
            .jump("ret");

        proc.label_decl("l0");

        proc.branch_named(TACJumpOpcode::JZ, "zero_1", "l3")
            .jump("ret");

        proc.label_decl("l3");
        proc.constant("error_value", -69)
            .copy("error_value", "returned_value")
            .jump("ret");

        proc.label_decl("ret");
        proc.return_void();

        let procedure = proc.build();

        eprintln!("{procedure}");

        let mut cfg = super::CFG::from(super::BasicBlocks::from(procedure));
        cfg.jump_threading();

        eprintln!("{}", cfg.serialize_tac());
    }
}
