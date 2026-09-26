use std::vec;

use compact_str::CompactString;

use crate::ir::tac::TACInst;

pub type Span = (usize, usize);

pub type Name = CompactString;

pub type Promise<T> = Option<T>;

pub type InstBlock = Vec<TACInst>;

// Typechecking types

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Bool,
    Void,

    Promised,
    Error,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ProcType {
    pub args_type: Option<Vec<(Name, Type)>>,
    pub return_type: Option<Type>,
}

#[derive(Clone)]
pub enum SemanticType {
    SimpleType(Type),
    ProcType(ProcType),
}

// Scope

#[derive(Clone, PartialEq)]
pub struct Stack<T> {
    stack: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn new_with(t: T) -> Self {
        Self { stack: vec![t] }
    }

    pub fn push(&mut self, t: T) {
        self.stack.push(t);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.stack.pop()
    }

    pub fn top(&self) -> Option<&T> {
        self.stack.last()
    }

    pub fn top_mut(&mut self) -> Option<&mut T> {
        self.stack.last_mut()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.stack.iter()
    }
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self {
            stack: Default::default(),
        }
    }
}

// Error types

#[derive(Debug)]
pub struct ErrorAggregate<T: std::error::Error>(Vec<ErrorBy<T>>);

impl<T: std::error::Error> ErrorAggregate<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn resolve(self) -> Result<(), Self> {
        match self.0.is_empty() {
            true => Ok(()),
            false => Err(self),
        }
    }

    pub fn add_error(&mut self, error_type: T) {
        self.0.push(ErrorBy::new(error_type));
    }
}

impl<T> std::fmt::Display for ErrorAggregate<T>
where
    T: std::error::Error,
    ErrorBy<T>: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, error) in self.0.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{error}")?;
        }

        Ok(())
    }
}

impl<T> std::error::Error for ErrorAggregate<T>
where
    T: std::error::Error,
    ErrorBy<T>: std::fmt::Display,
{
}

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
#[error("{error_type} at {span:?}")]
pub struct ErrorBy<T: std::error::Error> {
    error_type: T,
    span: Option<Span>,
}

impl<T: std::error::Error> ErrorBy<T> {
    pub fn new(error_type: T) -> Self {
        Self {
            error_type,
            span: None,
        }
    }
}
