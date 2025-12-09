use std::collections::HashSet;
use std::iter::zip;

use crate::ast::*;
use crate::types::{Name, Span, Stack};

pub struct SemChecker {
    scopes: Stack<HashSet<Name>>,
    errors: Vec<SyntaxError>,
    loop_depth: usize,
}

impl SemChecker {
    fn new() -> Self {
        Self {
            scopes: Stack::new(),
            errors: Vec::new(),
            loop_depth: 0,
        }
    }

    pub fn check(program: &Program, global_decls: HashSet<Name>) -> Option<Vec<SyntaxError>> {
        let mut sc = SemChecker::new();
        sc.scopes.push(global_decls);
        sc.current_scope_insert(Name::from("print"));

        sc.check_program(program);
        match sc.errors.is_empty() {
            true => None,
            false => Some(sc.errors),
        }
    }

    fn check_program(&mut self, program: &Program) {
        for decl in &program.0 {
            match decl {
                Declaration::Variable(var) => self.check_global_variable(var),

                Declaration::Proc {
                    block, proc_args, ..
                } => {
                    self.scopes.push(HashSet::new());
                    if let Some(args) = proc_args {
                        args.iter()
                            .for_each(|(name, _)| self.current_scope_insert(name.clone()));
                    }

                    self.check_block(block);
                    self.scopes.pop();
                }
            }
        }
    }

    fn check_block(&mut self, block: &Block) {
        self.scopes.push(HashSet::new());
        for stmt in &block.0 {
            self.check_statement(stmt);
        }
        self.scopes.pop();
    }

    fn check_global_variable(&mut self, var: &Variable) {
        for expr in &var.values {
            self.check_is_constant_expression(expr);
        }
    }

    fn check_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Variable(var) => self.check_variable(var),

            Statement::Block(block) => {
                self.check_block(block);
            }

            Statement::Assignment { name, value } => {
                self.check_declared(name);
                self.check_expression(value);
            }

            Statement::Eval(value) => {
                self.check_expression(value);
            }

            Statement::Return(expr) => {
                if let Some(expr) = expr {
                    self.check_expression(expr)
                }
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

    fn check_variable(&mut self, var: &Variable) {
        for (name, expr) in zip(&var.names, &var.values) {
            self.check_expression(expr);
            self.check_non_declared(name);
        }
    }

    fn check_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Variable(name, ..) => {
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

            Expression::ProcCall {
                proc_name,
                proc_args,
                ..
            } => {
                self.check_declared(proc_name);

                if let Some(args) = proc_args {
                    for expr in args {
                        self.check_expression(expr);
                    }
                }
            }
        }
    }

    fn check_is_constant_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Number(_) | Expression::Bool(_) => {}

            _ => self.insert_error(SyntaxErrorType::NonConstantExpression, None),
        }
    }

    fn check_non_declared(&mut self, name: &Name) {
        if self.current_scope_contains(name) {
            self.insert_error(SyntaxErrorType::DuplicateVariable, None);
        } else {
            self.current_scope_insert(name.clone());
        }
    }

    fn check_declared(&mut self, name: &Name) {
        if !(self.any_scope_contains(name)) {
            self.insert_error(SyntaxErrorType::MissingVariable, None);
        }
    }

    // Stack Helper functions

    fn current_scope_contains(&mut self, name: &Name) -> bool {
        self.scopes.top().is_some_and(|s| s.contains(name))
    }

    fn current_scope_insert(&mut self, name: Name) {
        if let Some(current) = self.scopes.top_mut() {
            current.insert(name);
        }
    }

    pub fn any_scope_contains(&self, name: &str) -> bool {
        for s in self.scopes.iter().rev() {
            if s.contains(name) {
                return true;
            }
        }
        false
    }

    // Error helper

    fn insert_error(&mut self, error_type: SyntaxErrorType, _span: Option<Span>) {
        self.errors.push(SyntaxError::new(error_type));
    }
}

// Syntax Errors

#[derive(Clone, Debug, PartialEq)]
pub struct SyntaxError {
    error_type: SyntaxErrorType,
    span: Option<Span>,
}

impl SyntaxError {
    pub fn new(error_type: SyntaxErrorType) -> Self {
        Self {
            error_type,
            span: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SyntaxErrorType {
    DuplicateVariable,
    MissingVariable,
    NonConstantExpression,
    JumpOutsideLoop,
    // NotYetImplemented,
}
