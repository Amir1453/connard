use std::collections::HashMap;
use std::iter::zip;

use crate::ast::*;
use crate::types::{Name, ProcType, Span, Stack, Type};

// #[derive(Clone, PartialEq)]
pub struct TypeChecker {
    scopes: Stack<HashMap<Name, SemanticType>>,
    current_proc_name: Name,
    errors: Vec<TypeError>,
}

impl TypeChecker {
    fn new() -> Self {
        Self {
            scopes: Stack::new_with(HashMap::new()),
            current_proc_name: Name::with_capacity(50),
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
        for decl in &mut program.0 {
            match decl {
                Declaration::Proc {
                    proc_name,
                    proc_args,
                    return_type,
                    ..
                } => {
                    self.current_scope_insert_proc(
                        proc_name.clone(),
                        proc_args.clone(),
                        return_type.clone(),
                    );
                }

                Declaration::Variable(var) => self.check_global_variable(var),
            }
        }

        for decl in &mut program.0 {
            match decl {
                Declaration::Proc {
                    proc_name, block, ..
                } => {
                    self.current_proc_name = proc_name.clone();
                    self.check_block(block);
                    self.current_proc_name.clear();
                }

                _ => {}
            }
        }
    }

    fn check_global_variable(&mut self, var: &mut Variable) {
        let var_type = var.ty;
        for (name, expr) in zip(&var.names, &mut var.values) {
            let value_type = expr.get_type();
            if value_type != var_type {
                self.insert_error(TypeErrorType::TypeMismatchGlobalDecl, None);
            }

            self.current_scope_insert_simple(name.clone(), value_type);
        }
    }

    fn check_block(&mut self, block: &mut Block) {
        self.scopes.push(HashMap::new());
        for stmt in &mut block.0 {
            self.check_statement(stmt);
        }
        self.scopes.pop();
    }

    fn check_statement(&mut self, stmt: &mut Statement) {
        match stmt {
            Statement::Variable(var) => self.check_variable(var),

            Statement::Block(block) => {
                self.check_block(block);
            }

            Statement::Assignment { name, value } => {
                self.check_expression(value);
                let value_type = value.get_type();
                if self.find_simple_type(name) != value_type {
                    self.insert_error(TypeErrorType::TypeMismatchAssign, None);
                }
            }

            Statement::Eval(expr) => self.check_expression(expr),

            Statement::Return(expr) => {
                match self.find_proc_type(&self.current_proc_name).return_type {
                    Some(expected) => {
                        if let Some(exp) = expr {
                            self.check_expression(exp);
                            let ty = exp.get_type();
                            if ty != expected {
                                self.insert_error(TypeErrorType::ReturnTypeMismatch, None);
                            }
                        } else {
                            self.insert_error(TypeErrorType::ReturnValueMissing, None);
                        }
                    }

                    None => {
                        if let Some(exp) = expr {
                            self.check_expression(exp);
                            let ty = exp.get_type();
                            if ty != Type::Void {
                                self.insert_error(TypeErrorType::ReturnTypeMismatchSub, None);
                            }
                        }
                    }
                }
            }

            Statement::If {
                condition,
                then_block,
                else_branch,
            } => {
                self.check_expression(condition);
                if condition.get_type() != Type::Bool {
                    self.insert_error(TypeErrorType::ConditionNotBool, None);
                }

                self.check_block(then_block);

                if let Some(ebr) = else_branch {
                    self.check_statement(ebr);
                }
            }

            Statement::While { condition, block } => {
                self.check_expression(condition);
                if condition.get_type() != Type::Bool {
                    self.insert_error(TypeErrorType::ConditionNotBool, None);
                }

                self.check_block(block);
            }

            Statement::Jump(_) => {}
        }
    }

    fn check_variable(&mut self, var: &mut Variable) {
        let var_type = var.ty;
        for (name, expr) in zip(&var.names, &mut var.values) {
            self.check_expression(expr);
            let value_type = expr.get_type();
            if value_type != var_type {
                self.insert_error(TypeErrorType::TypeMismatchDecl, None);
            }

            self.current_scope_insert_simple(name.clone(), value_type);
        }
    }

    fn check_expression(&mut self, expr: &mut Expression) {
        match expr {
            Expression::Variable(name, ty) => {
                *ty = Some(self.find_simple_type(name));
            }

            Expression::UnaryOperation {
                operator,
                value,
                ty,
            } => {
                self.check_expression(value);
                let value_type = value.get_type();
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
                let lhs_type = lhs.get_type();
                let rhs_type = rhs.get_type();

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
            Expression::ProcCall {
                proc_name,
                proc_args,
                ty,
            } => {
                let proc_type = self.find_proc_type(proc_name);
                *ty = Some(proc_type.return_type.unwrap_or(Type::Void));

                if let (Some(args), Some(arg_supposed_types)) =
                    (proc_args, proc_type.args_type.as_ref())
                {
                    for (arg, (_name, expected_ty)) in
                        args.iter_mut().zip(arg_supposed_types.iter())
                    {
                        self.check_expression(arg);
                        let arg_type = arg.get_type();
                        if arg_type != *expected_ty {
                            self.insert_error(TypeErrorType::TypeMismatchProcArgument, None);
                        }
                    }
                }
            }

            _ => {}
        }
    }

    // Stack helpers

    pub fn current_scope_insert_simple(&mut self, name: Name, ty: Type) {
        if let Some(current) = self.scopes.top_mut() {
            current.insert(name, SemanticType::SimpleType(ty));
        }
    }

    pub fn current_scope_insert_proc(
        &mut self,
        name: Name,
        args_type: Option<Vec<(Name, Type)>>,
        return_type: Option<Type>,
    ) {
        if let Some(current) = self.scopes.top_mut() {
            current.insert(
                name,
                SemanticType::ProcType(ProcType {
                    args_type,
                    return_type,
                }),
            );
        }
    }

    pub fn find_simple_type(&self, name: &str) -> Type {
        for s in self.scopes.iter().rev() {
            if let Some(t) = s.get(name)
                && let SemanticType::SimpleType(ty) = t
            {
                return ty.clone();
            }
        }
        Type::Error
    }

    pub fn find_proc_type(&self, name: &str) -> ProcType {
        for s in self.scopes.iter().rev() {
            if let Some(t) = s.get(name)
                && let SemanticType::ProcType(pt) = t
            {
                return pt.clone();
            }
        }
        ProcType::default()
    }

    // Error helper

    fn insert_error(&mut self, error_type: TypeErrorType, _span: Option<Span>) {
        self.errors.push(TypeError::new(error_type));
    }
}

pub enum SemanticType {
    SimpleType(Type),
    ProcType(ProcType),
}

// Type Errors

#[derive(Clone, Debug, PartialEq)]
pub struct TypeError {
    error_type: TypeErrorType,
    span: Option<Span>,
}

impl TypeError {
    pub fn new(error_type: TypeErrorType) -> Self {
        Self {
            error_type,
            span: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeErrorType {
    TypeMismatchDecl,
    TypeMismatchGlobalDecl,
    TypeMismatchAssign,
    TypeMismatchProcArgument,

    ConditionNotBool,

    OpNegMismatch,
    OpTildeMismatch,
    OpLNotMismatch,
    OpArithMismatch,
    OpBitwiseMismatch,
    OpCompareMismatch,
    OpLogicalMismatch,

    ReturnValueMissing,
    ReturnTypeMismatch,
    ReturnTypeMismatchSub,
}
