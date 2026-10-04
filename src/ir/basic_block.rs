use crate::frontend::Symbol;
use crate::ir::tac::{InstBlock, Label, TACInst};

pub struct BasicBlocks {
    pub blocks: Vec<BasicBlock>,
}

impl BasicBlocks {
    fn empty() -> Self {
        Self { blocks: Vec::new() }
    }

    pub fn from_instrs(instructions: InstBlock, entry_name: &str) -> Self {
        use TACInst::*;

        // A way to get fresh labels
        let mut counter: i64 = 0;
        let mut fresh_label = || -> Label {
            counter += 1;
            let label_sym = Symbol::intern(&format!("B{}", counter));
            Label::Named(label_sym)
        };

        let mut basic_blocks: BasicBlocks = BasicBlocks::empty();

        // Add a label before the first instruction if needed
        let mut inst_iter = instructions.into_iter().peekable();

        let first_inst = inst_iter.next();
        match first_inst {
            Some(LabelDecl(label)) => {
                basic_blocks.push_block(label);
                if let Some(LabelDecl(label2)) = inst_iter.peek() {
                    basic_blocks.push_instruction_last_block(UnconditionalJump(label2.clone()));
                }
            }

            Some(inst) => {
                let title = Symbol::intern(&format!("{}_entry", entry_name));
                basic_blocks.push_block(Label::Named(title));
                basic_blocks.push_instruction_last_block(inst);
            }

            None => return basic_blocks,
        }

        while let Some(instruction) = inst_iter.next() {
            let next_instruction = inst_iter.peek();

            match (&instruction, next_instruction) {
                (UnconditionalJump(_) | Return(_), Some(LabelDecl(_)) | None) => {
                    basic_blocks.push_instruction_last_block(instruction);
                }

                (UnconditionalJump(_) | Return(_), _) => {
                    basic_blocks.push_instruction_last_block(instruction);
                    basic_blocks.push_block(fresh_label());
                }

                (ConditionalJump { .. }, Some(UnconditionalJump(_)) | Some(Return(_))) => {
                    basic_blocks.push_instruction_last_block(instruction);
                }

                (ConditionalJump { .. }, Some(LabelDecl(label))) => {
                    basic_blocks.push_instruction_last_block(instruction);
                    basic_blocks.push_instruction_last_block(UnconditionalJump(label.clone()));
                }

                (ConditionalJump { .. }, Some(_)) => {
                    basic_blocks.push_instruction_last_block(instruction);
                    let new_label = fresh_label();
                    basic_blocks.push_instruction_last_block(UnconditionalJump(new_label.clone()));
                    basic_blocks.push_block(new_label);
                }

                (LabelDecl(label), Some(LabelDecl(label2))) => {
                    basic_blocks.push_block(label.clone());
                    basic_blocks.push_instruction_last_block(UnconditionalJump(label2.clone()));
                }

                (LabelDecl(label), _) => {
                    basic_blocks.push_block(label.clone());
                }

                (_, Some(LabelDecl(label))) => {
                    basic_blocks.push_instruction_last_block(instruction);
                    basic_blocks.push_instruction_last_block(UnconditionalJump(label.clone()));
                }

                (_, _) => {
                    basic_blocks.push_instruction_last_block(instruction);
                }
            }
        }

        basic_blocks
    }

    fn push_block(&mut self, block_label: Label) {
        self.blocks.push(BasicBlock::new(block_label));
    }

    fn push_instruction_last_block(&mut self, inst: TACInst) {
        self.blocks.last_mut().unwrap().push_instruction(inst);
    }
}

pub struct BasicBlock {
    pub block_label: Label,
    pub instructions: InstBlock,
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

    pub fn pop_instruction(&mut self) -> Option<TACInst> {
        self.instructions.pop()
    }
}

impl std::fmt::Display for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.block_label)?;
        for inst in &self.instructions {
            writeln!(f, "{}", inst)?;
        }
        write!(f, "")
    }
}

impl std::fmt::Debug for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
