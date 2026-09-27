use crate::frontend::ast::{Block, Program, Statement};
use crate::types::{ErrorAggregate, Name};

pub struct RetChecker {}

impl RetChecker {
    pub fn check(program: &Program) -> Result<(), ErrorAggregate<NoReturnError>> {
        let mut errors = ErrorAggregate::<NoReturnError>::new();

        for decl in &program.0 {
            match decl {
                super::ast::Declaration::Proc {
                    proc_name, block, ..
                } => {
                    if proc_name == "main" {
                        continue;
                    }

                    if !Self::check_block(block) {
                        errors.add_error(NoReturnError(proc_name.clone()));
                    }
                }

                _ => {}
            }
        }

        errors.resolve()
    }

    fn check_block(block: &Block) -> bool {
        for statement in &block.0 {
            if Self::check_statement(statement) {
                return true;
            }
        }
        false
    }

    fn check_statement(statement: &Statement) -> bool {
        match statement {
            Statement::Return(_) => true,

            Statement::Block(block) => Self::check_block(block),

            Statement::If {
                then_block,
                else_branch: Some(else_branch),
                ..
            } => Self::check_block(then_block) && Self::check_statement(else_branch),

            // Assume that while statements return for now
            // Needs return analysis
            Statement::While { .. } => true,

            _ => false,
        }
    }
}

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
#[error("function {0} does not return")]
pub struct NoReturnError(Name);
