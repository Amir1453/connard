use compact_str::CompactString;

use crate::ir::tac::{Label, ProcDecl, TACInst};
use crate::types::{InstBlock, Name};

pub struct BasicBlocks {
    pub blocks: Vec<BasicBlock>,
    pub block_index: i64,
    pub proc_name: Name,
    pub arguments: Option<Vec<Name>>,
}

impl BasicBlocks {
    fn empty() -> Self {
        Self {
            blocks: Vec::new(),
            proc_name: CompactString::new(""),
            arguments: None,
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

impl From<ProcDecl> for BasicBlocks {
    fn from(proc: ProcDecl) -> Self {
        use TACInst::*;

        let instructions = proc.instructions;

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
            Some(LabelDecl(label)) => {
                basic_blocks.push_block(label);
                if let Some(LabelDecl(label2)) = inst_iter.peek() {
                    basic_blocks.push_instruction_last_block(UnconditionalJump(label2.clone()));
                }
            }

            Some(inst) => {
                let title = format!("{}_entry", proc.name).into();
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

        basic_blocks.proc_name = proc.name;
        basic_blocks.arguments = proc.arguments;
        basic_blocks
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

    #[allow(dead_code)]
    pub fn get_terminator(&self) -> Option<TACInst> {
        let last = self.instructions.last();
        match last {
            Some(TACInst::UnconditionalJump(_) | TACInst::Return(_)) => last.cloned(),
            Some(_) => None,
            None => None,
        }
    }

    #[allow(dead_code)]
    pub fn get_pre_terminator(&self) -> Option<TACInst> {
        let last = self.instructions.last();
        if !matches!(
            last,
            Some(TACInst::UnconditionalJump(_) | TACInst::Return(_))
        ) {
            return None;
        }

        let len = self.instructions.len();
        self.instructions.get(len - 1).cloned()
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
