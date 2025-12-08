use core::fmt;
use std::{collections::HashMap, fmt::Debug};

// use compact_str::CompactString;
use petgraph::{
    Direction,
    graph::NodeIndex,
    prelude::StableGraph,
    visit::{Bfs, Dfs, EdgeRef, IntoNodeReferences, VisitMap},
};

use crate::{
    tac::{Label, TACInst},
    // types::{InstBlock, Name},
    types::InstBlock,
};

pub struct CFG {
    pub graph: StableGraph<BasicBlock, TACInst>,
    // label_to_index: HashMap<Label, NodeIndex>,
}

impl CFG {
    pub fn serialize_tac(&mut self) -> InstBlock {
        let mut schedule: InstBlock = Vec::new();
        let mut schedule_order: Vec<NodeIndex> = Vec::with_capacity(self.graph.node_count());

        let Some(entry) = self.graph.node_indices().next() else {
            return schedule;
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

        schedule
    }

    pub fn remove_unreachable(&mut self) {
        let entry = match self.graph.node_indices().next() {
            Some(n) => n,
            None => return,
        };

        let mut dfs = Dfs::new(&self.graph, entry);

        while let Some(_) = dfs.next(&self.graph) {}

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
            // label_to_index,
        }
    }
}

pub struct BasicBlocks {
    pub blocks: Vec<BasicBlock>,
    // proc_name: Name,
    block_index: i64,
}

impl BasicBlocks {
    fn empty() -> Self {
        Self {
            blocks: Vec::new(),
            // proc_name: CompactString::new(""),
            block_index: -1,
        }
    }

    fn push_block(&mut self, block_label: Label) {
        self.blocks.push(BasicBlock::new(block_label));
        self.block_index += 1;
    }

    fn push_instruction_last_block(&mut self, inst: TACInst) {
        let index: usize = self.block_index.try_into().unwrap_or(0);
        self.blocks[index].push_instruction(inst);
    }
}

impl From<InstBlock> for BasicBlocks {
    fn from(instructions: InstBlock) -> Self {
        use TACInst::*;

        // A way to get fresh labels
        let mut counter: i64 = 0;
        let mut fresh_label = || -> Label {
            counter += 1;
            Label::Named(format!("B{}", counter).into())
        };

        let mut basic_blocks: BasicBlocks = BasicBlocks::empty();

        // Add a label before the first instruction if needed
        let mut inst_iter = instructions.into_iter().peekable();

        let first_inst = inst_iter.next();
        match first_inst {
            Some(LabelDecl(label)) => basic_blocks.push_block(label),
            Some(inst) => {
                basic_blocks.push_block(Label::Named("Entry".into()));
                basic_blocks.push_instruction_last_block(inst);
            }
            None => return basic_blocks,
        }

        // while i < instructions.len() {
        while let Some(instruction) = inst_iter.next() {
            let next_instruction = inst_iter.peek();

            match (&instruction, next_instruction) {
                (UnconditionalJump(_) | Return(_), Some(LabelDecl(_))) => {
                    basic_blocks.push_instruction_last_block(instruction);
                    // ENTRY -> THIS IS STUPIDDD
                    if let Some(LabelDecl(label)) = inst_iter.next() {
                        basic_blocks.push_block(label);
                    }
                }

                (UnconditionalJump(_) | Return(_), None) => {
                    basic_blocks.push_instruction_last_block(instruction);
                }

                (UnconditionalJump(_) | Return(_), _) => {
                    basic_blocks.push_instruction_last_block(instruction);
                    basic_blocks.push_block(fresh_label());
                }

                (ConditionalJump { .. }, Some(UnconditionalJump(_)) | Some(Return(_))) => {
                    basic_blocks.push_instruction_last_block(instruction);
                }

                (ConditionalJump { .. }, Some(LabelDecl(_))) => {
                    basic_blocks.push_instruction_last_block(instruction);
                    // ENTRY -> THIS IS STUPIDDD
                    if let Some(LabelDecl(label)) = inst_iter.next() {
                        basic_blocks.push_instruction_last_block(UnconditionalJump(label.clone()));
                        basic_blocks.push_block(label);
                    }
                }

                (ConditionalJump { .. }, Some(_)) => {
                    basic_blocks.push_instruction_last_block(instruction);
                    let new_label = fresh_label();
                    basic_blocks.push_instruction_last_block(UnconditionalJump(new_label.clone()));
                    basic_blocks.push_block(new_label);
                }

                (LabelDecl(label), _) => {
                    basic_blocks.push_block(label.clone());
                }

                (_, Some(LabelDecl(_))) => {
                    basic_blocks.push_instruction_last_block(instruction);
                    // ENTRY -> THIS IS STUPIDDD
                    if let Some(LabelDecl(label)) = inst_iter.next() {
                        basic_blocks.push_instruction_last_block(UnconditionalJump(label.clone()));
                        basic_blocks.push_block(label);
                    }
                }

                (_, _) => {
                    basic_blocks.push_instruction_last_block(instruction);
                }
            }
        }

        basic_blocks
    }
}

impl From<&InstBlock> for BasicBlocks {
    fn from(instructions: &InstBlock) -> Self {
        use TACInst::*;

        // A way to get fresh labels
        let mut counter: i64 = 0;
        let mut fresh_label = || -> Label {
            counter += 1;
            Label::Named(format!("B{}", counter).into())
        };

        let mut basic_blocks: BasicBlocks = BasicBlocks::empty();

        // Add a label before the first instruction if needed
        let mut i = 0;
        if let Some(LabelDecl(label)) = instructions.first() {
            basic_blocks.push_block(label.clone());
            i += 1;
        } else {
            basic_blocks.push_block(Label::Named("Entry".into()));
        }

        while i < instructions.len() {
            let instruction = &instructions[i];
            let next_instruction = instructions.get(i + 1);

            match (instruction, next_instruction) {
                (UnconditionalJump(_) | Return(_), Some(LabelDecl(label))) => {
                    basic_blocks.push_instruction_last_block(instruction.clone());
                    basic_blocks.push_block(label.clone());
                    i += 1;
                }

                (UnconditionalJump(_) | Return(_), None) => {
                    basic_blocks.push_instruction_last_block(instruction.clone());
                }

                (UnconditionalJump(_) | Return(_), _) => {
                    basic_blocks.push_instruction_last_block(instruction.clone());
                    basic_blocks.push_block(fresh_label());
                }

                (ConditionalJump { .. }, Some(UnconditionalJump(_)) | Some(Return(_))) => {
                    basic_blocks.push_instruction_last_block(instruction.clone());
                }

                (ConditionalJump { .. }, Some(LabelDecl(label))) => {
                    basic_blocks.push_instruction_last_block(instruction.clone());
                    basic_blocks.push_instruction_last_block(UnconditionalJump(label.clone()));
                    basic_blocks.push_block(label.clone());
                    i += 1;
                }

                (ConditionalJump { .. }, Some(_)) => {
                    basic_blocks.push_instruction_last_block(instruction.clone());
                    let new_label = fresh_label();
                    basic_blocks.push_instruction_last_block(UnconditionalJump(new_label.clone()));
                    basic_blocks.push_block(new_label);
                }

                (LabelDecl(label), _) => {
                    basic_blocks.push_block(label.clone());
                }

                (_, Some(LabelDecl(label))) => {
                    basic_blocks.push_instruction_last_block(instruction.clone());
                    basic_blocks.push_instruction_last_block(UnconditionalJump(label.clone()));
                    basic_blocks.push_block(label.clone());
                    i += 1;
                }

                (_, _) => {
                    basic_blocks.push_instruction_last_block(instruction.clone());
                }
            }

            i += 1;
        }

        basic_blocks
    }
}

pub struct BasicBlock {
    block_label: Label,
    instructions: InstBlock,
}

impl BasicBlock {
    pub fn new(block_label: Label) -> Self {
        Self {
            block_label,
            instructions: Vec::new(),
        }
    }

    pub fn push_instruction(&mut self, inst: TACInst) {
        self.instructions.push(inst);
    }

    fn pop_instruction(&mut self) -> Option<TACInst> {
        self.instructions.pop()
    }
}

impl fmt::Display for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", &self.block_label)?;
        for inst in &self.instructions {
            writeln!(f, "{}", inst)?;
        }
        write!(f, "")
    }
}

impl Debug for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
