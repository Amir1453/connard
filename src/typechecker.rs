use crate::ast::*;
use crate::types::{Span, Type, TypeError, TypeErrorType, TypeScopes};

#[derive(Clone, Debug, PartialEq)]
pub struct TypeChecker {
    scopes: TypeScopes,
    errors: Vec<TypeError>,
}

impl TypeChecker {
    fn new() -> Self {
        Self {
            scopes: TypeScopes::new(),
            errors: Vec::new(),
        }
    }

    pub fn check(program: &mut Program) -> Option<Vec<TypeError>> {
        let mut tc = TypeChecker::new();
        tc.check_program(program);
        match tc.errors.is_empty() {
            true => None,
            false => Some(tc.errors),
        }
    }

    fn check_program(&mut self, program: &mut Program) {
        self.check_block(program);
    }

    fn check_block(&mut self, block: &mut Block) {
        self.scopes.push_scope();
        for stmt in &mut block.0 {
            self.check_statement(stmt);
        }
        self.scopes.pop_scope();
    }

    fn check_statement(&mut self, stmt: &mut Statement) {
        match stmt {
            Statement::Variable { name, value, ty } => {
                self.check_expression(value);
                let value_type = self.get_exp_type(value);
                if value_type != *ty {
                    self.insert_error(TypeErrorType::TypeMismatchDecl, None);
                }
                self.scopes.current_insert(name.clone(), value_type);
            }

            Statement::Block(block) => {
                self.check_block(block);
            }

            Statement::Assignment { name, value } => {
                self.check_expression(value);
                let value_type = self.get_exp_type(value);
                if self.scopes.find_type(name).unwrap() != value_type {
                    self.insert_error(TypeErrorType::TypeMismatchAssign, None);
                }
            }

            Statement::Print { value } => {
                self.check_expression(value);
                let value_type = self.get_exp_type(value);
                if value_type != Type::Int {
                    self.insert_error(TypeErrorType::TypeMismatchPrint, None);
                }
            }

            Statement::If {
                condition,
                then_block,
                else_branch,
            } => {
                self.check_expression(condition);
                if self.get_exp_type(condition) != Type::Bool {
                    self.insert_error(TypeErrorType::ConditionNotBool, None);
                }

                self.check_block(then_block);

                if let Some(ebr) = else_branch {
                    self.check_statement(ebr);
                }
            }

            Statement::While { condition, block } => {
                self.check_expression(condition);
                if self.get_exp_type(condition) != Type::Bool {
                    self.insert_error(TypeErrorType::ConditionNotBool, None);
                }

                self.check_block(block);
            }

            Statement::Jump(_) => {}
        }
    }

    fn check_expression(&mut self, expr: &mut Expression) {
        match expr {
            Expression::UnaryOperation {
                operator,
                value,
                ty,
            } => {
                self.check_expression(value);
                let value_type = self.get_exp_type(value);
                match operator {
                    Operator::Neg => {
                        if value_type != Type::Int {
                            self.insert_error(TypeErrorType::OpNegMismatch, None);
                            *ty = Some(Type::Error);
                        } else {
                            *ty = Some(Type::Int);
                        }
                    }

                    Operator::Tilde => {
                        if value_type != Type::Int {
                            self.insert_error(TypeErrorType::OpTildeMismatch, None);
                            *ty = Some(Type::Error);
                        } else {
                            *ty = Some(Type::Int);
                        }
                    }

                    Operator::LNot => {
                        if value_type != Type::Bool {
                            self.insert_error(TypeErrorType::OpLNotMismatch, None);
                            *ty = Some(Type::Error);
                        } else {
                            *ty = Some(Type::Bool);
                        }
                    }

                    _ => {}
                }
            }

            Expression::BinaryOperation {
                lhs,
                operator,
                rhs,
                ty,
            } => {
                self.check_expression(lhs);
                self.check_expression(rhs);
                let lhs_type = self.get_exp_type(lhs);
                let rhs_type = self.get_exp_type(rhs);

                use Operator::*;
                let exp_type = match operator {
                    Plus | Minus | Star | Slash | Mod => {
                        if lhs_type == Type::Int && rhs_type == Type::Int {
                            Some(Type::Int)
                        } else {
                            self.insert_error(TypeErrorType::OpArithMismatch, None);
                            Some(Type::Error)
                        }
                    }

                    Pipe | Caret | Ampersand | LShift | RShift => {
                        if lhs_type == Type::Int && rhs_type == Type::Int {
                            Some(Type::Int)
                        } else {
                            self.insert_error(TypeErrorType::OpBitwiseMismatch, None);
                            Some(Type::Error)
                        }
                    }

                    Equal | NEqual | L | LTE | G | GTE => {
                        if lhs_type == Type::Int && rhs_type == Type::Int {
                            Some(Type::Bool)
                        } else {
                            self.insert_error(TypeErrorType::OpCompareMismatch, None);
                            Some(Type::Error)
                        }
                    }

                    LAnd | LOr => {
                        if lhs_type == Type::Bool && rhs_type == Type::Bool {
                            Some(Type::Bool)
                        } else {
                            self.insert_error(TypeErrorType::OpLogicalMismatch, None);
                            Some(Type::Error)
                        }
                    }

                    _ => None,
                };
                *ty = exp_type;
            }

            _ => {}
        }
    }

    fn get_exp_type(&self, expr: &Expression) -> Type {
        match expr {
            Expression::Variable(name) => match self.scopes.find_type(name) {
                Some(t) => t,
                None => Type::Error,
            },
            Expression::Number(_) => Type::Int,
            Expression::Bool(_) => Type::Bool,
            Expression::UnaryOperation { ty, .. } => ty.unwrap_or(Type::Error),
            Expression::BinaryOperation { ty, .. } => ty.unwrap_or(Type::Error),
        }
    }

    fn insert_error(&mut self, error_type: TypeErrorType, _span: Option<Span>) {
        self.errors.push(TypeError::new(error_type));
    }
}
