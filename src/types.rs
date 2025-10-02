/* Type Type */

use std::collections::{HashMap, HashSet};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Type {
    Int,
    Bool,
    Void,
    Error,
}

/* Scope Types */

#[derive(Clone, Debug, PartialEq)]
pub struct TypeScopes {
    scopes: Vec<HashMap<String, Type>>,
}

impl TypeScopes {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn current_insert(&mut self, name: String, ty: Type) {
        let current = self.scopes.last_mut().unwrap();
        current.insert(name, ty);
    }

    pub fn find_type(&self, name: &str) -> Option<Type> {
        for s in self.scopes.iter().rev() {
            if let Some(t) = s.get(name) {
                return Some(*t);
            }
        }
        None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VarScopes {
    scopes: Vec<HashSet<String>>,
}

impl VarScopes {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashSet::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashSet::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn current_contains(&self, name: &str) -> bool {
        self.scopes.last().map_or(false, |s| s.contains(name))
    }

    pub fn any_contains(&self, name: &str) -> bool {
        for s in self.scopes.iter().rev() {
            if s.contains(name) {
                return true;
            }
        }
        false
    }

    pub fn current_insert(&mut self, name: String) {
        let current = self.scopes.last_mut().unwrap();
        current.insert(name);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoopStack {
    loops: Vec<(i64, i64)>,
}

impl LoopStack {
    pub fn new() -> Self {
        Self { loops: Vec::new() }
    }

    pub fn push_loop(&mut self, cont: i64, brk: i64) {
        self.loops.push((cont, brk));
    }

    pub fn pop_loop(&mut self) {
        self.loops.pop();
    }

    pub fn current_loop(&self) -> Option<(i64, i64)> {
        self.loops.last().cloned()
    }
}

/* Error Types */

pub type Span = (usize, usize);

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
    TypeMismatchAssign,
    TypeMismatchPrint,

    ConditionNotBool,

    OpNegMismatch,
    OpTildeMismatch,
    OpLNotMismatch,
    OpArithMismatch,
    OpBitwiseMismatch,
    OpCompareMismatch,
    OpLogicalMismatch,
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
    JumpOutsideLoop,
}
