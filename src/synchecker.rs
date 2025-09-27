#![allow(dead_code)]

use crate::ast::*;
use std::collections::HashSet;

type Span = (usize, usize);

pub enum SyntaxErrorType {
    DuplicateVariable,
    MissingVariable,
}

pub struct SyntaxError {
    pub error_type: SyntaxErrorType,
    pub span: Option<Span>,
}

pub struct SynChecker {
    vars: HashSet<String>,
    errors: Vec<SyntaxError>,
}

impl SynChecker {
    fn new() -> Self {
        Self {
            vars: HashSet::new(),
            errors: Vec::new(),
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

    fn clear(&mut self) {
        self.vars.clear();
        self.errors.clear();
    }

    fn check_program(&mut self, program: &Program) {
        for stmt in &program.0 {
            self.check_statement(stmt);
        }
    }

    fn check_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Variable { name, value } => {
                self.check_expression(value);
                if self.check_non_declared(name) {
                    self.vars.insert(name.clone());
                }
            }

            Statement::Assignment { name, value } => {
                self.check_declared(name);
                self.check_expression(value);
            }

            Statement::Print { value } => {
                self.check_expression(value);
            }

            _ => todo!(),
        }
    }

    fn check_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Variable(name) => {
                self.check_declared(name);
            }

            Expression::Number(value) => {
                self.check_valid_integer(value);
            }

            Expression::BinaryOperation {
                lhs,
                operator: _,
                rhs,
            } => {
                self.check_expression(lhs);
                self.check_expression(rhs);
            }

            Expression::UnaryOperation { operator: _, value } => {
                self.check_expression(value);
            }
            _ => todo!(),
        }
    }

    fn check_non_declared(&mut self, name: &String) -> bool {
        if self.vars.contains(name) {
            self.errors.push(SyntaxError {
                error_type: SyntaxErrorType::DuplicateVariable,
                span: None,
            });
            return false;
        }
        return true;
    }

    fn check_declared(&mut self, name: &String) -> bool {
        if !(self.vars.contains(name)) {
            self.errors.push(SyntaxError {
                error_type: SyntaxErrorType::MissingVariable,
                span: None,
            });
            return false;
        }
        return true;
    }

    fn check_valid_integer(&mut self, _value: &i64) -> bool {
        return true;
    }
}
