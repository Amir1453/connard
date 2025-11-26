use crate::tac::TACInst;
use compact_str::CompactString;
use std::vec;

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

#[derive(Clone, Debug, PartialEq)]
pub struct ProcType {
    pub args_type: Option<Vec<(Name, Type)>>,
    pub return_type: Option<Type>,
}

impl Default for ProcType {
    fn default() -> Self {
        Self {
            args_type: None,
            return_type: None,
        }
    }
}

// Scope

#[derive(Clone, PartialEq)]
pub struct Stack<T> {
    stack: Vec<T>,
}

impl<T> Stack<T> {
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
