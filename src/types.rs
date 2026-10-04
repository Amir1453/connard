use compact_str::CompactString;
use thin_vec::ThinVec;

use crate::frontend::Symbol;

pub type Span = (usize, usize);

pub type Name = CompactString;

pub type Promise<T> = Option<T>;

// BX types

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
    pub args_type: Option<ThinVec<(Symbol, Type)>>,
    pub return_type: Option<Type>,
}

#[derive(Clone)]
pub enum SemanticType {
    SimpleType(Type),
    ProcType(ProcType),
}
