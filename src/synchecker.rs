use crate::ast::*;
use crate::types::{Span, SyntaxError, SyntaxErrorType, VarScopes};

#[derive(Clone, Debug, PartialEq)]
pub struct SynChecker {
    scopes: VarScopes,
    errors: Vec<SyntaxError>,
    loop_depth: usize,
}

impl SynChecker {
    fn new() -> Self {
        Self {
            scopes: VarScopes::new(),
            errors: Vec::new(),
            loop_depth: 0,
        }
    }

    pub fn check(program: &Program) -> Option<Vec<SyntaxError>> {
        let mut sc = SynChecker::new();
        sc.check_program(program);
        match sc.errors.is_empty() {
            true => None,
            false => Some(sc.errors),
        }
    }

    fn check_program(&mut self, program: &Program) {
        self.check_block(program);
    }

    fn check_block(&mut self, block: &Block) {
        self.scopes.push_scope();
        for stmt in &block.0 {
            self.check_statement(stmt);
        }
        self.scopes.pop_scope()
    }

    fn check_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Variable { name, value, .. } => {
                self.check_expression(value);
                if self.check_non_declared(name) {
                    self.scopes.current_insert(name.clone());
                }
            }

            Statement::Block(block) => {
                self.check_block(block);
            }

            Statement::Assignment { name, value } => {
                self.check_declared(name);
                self.check_expression(value);
            }

            Statement::Print { value } => {
                self.check_expression(value);
            }

            Statement::If {
                condition,
                then_block,
                else_branch,
            } => {
                self.check_expression(condition);

                self.check_block(then_block);

                if let Some(ebr) = else_branch {
                    self.check_statement(ebr);
                }
            }

            Statement::While { condition, block } => {
                self.check_expression(condition);

                self.loop_depth += 1;
                self.check_block(block);
                self.loop_depth -= 1;
            }

            Statement::Jump(_) => {
                if self.loop_depth == 0 {
                    self.insert_error(SyntaxErrorType::JumpOutsideLoop, None);
                }
            }
        }
    }

    fn check_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Variable(name) => {
                self.check_declared(name);
            }

            Expression::Number(_) => {}

            Expression::Bool(_) => {}

            Expression::BinaryOperation { lhs, rhs, .. } => {
                self.check_expression(lhs);
                self.check_expression(rhs);
            }

            Expression::UnaryOperation { value, .. } => {
                self.check_expression(value);
            }
        }
    }

    fn check_non_declared(&mut self, name: &String) -> bool {
        if self.scopes.current_contains(name) {
            self.insert_error(SyntaxErrorType::DuplicateVariable, None);
            return false;
        }
        return true;
    }

    fn check_declared(&mut self, name: &String) -> bool {
        if !(self.scopes.any_contains(name)) {
            self.insert_error(SyntaxErrorType::MissingVariable, None);
            return false;
        }
        return true;
    }

    fn insert_error(&mut self, error_type: SyntaxErrorType, _span: Option<Span>) {
        self.errors.push(SyntaxError::new(error_type));
    }
}
