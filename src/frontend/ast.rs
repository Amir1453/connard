use super::Symbol;
use crate::types::{Promise, Type};

use thin_vec::ThinVec;

pub struct Program(pub Vec<Declaration>);

pub enum Declaration {
    Variable(Box<Variable>),

    Proc {
        proc_name: Symbol,
        proc_args: Option<ThinVec<(Symbol, Type)>>,
        return_type: Option<Type>,
        block: Box<Block>,
    },
}

pub struct Variable {
    pub names: ThinVec<Symbol>,
    pub values: ThinVec<Box<Expression>>,
    pub ty: Type,
    #[allow(unused)]
    pub scope: ScopeState,
}

pub struct Block(pub ThinVec<Statement>);

pub enum Statement {
    Variable(Box<Variable>),

    Block(Box<Block>),

    Assignment {
        name: Symbol,
        value: Box<Expression>,
    },

    Eval(Box<Expression>),

    Return(Option<Box<Expression>>),

    If {
        condition: Box<Expression>,
        then_block: Box<Block>,
        else_branch: Option<Box<Statement>>,
    },

    While {
        condition: Box<Expression>,
        block: Box<Block>,
    },

    Jump(JumpState),
}

pub enum Expression {
    Variable(Symbol, Promise<Type>),
    Number(i64),
    Bool(bool),

    BinaryOperation {
        lhs: Box<Expression>,
        operator: Operator,
        rhs: Box<Expression>,
        ty: Promise<Type>,
    },

    UnaryOperation {
        operator: Operator,
        value: Box<Expression>,
        ty: Promise<Type>,
    },

    ProcCall {
        proc_name: Symbol,
        proc_args: Option<ThinVec<Box<Expression>>>,
        ty: Promise<Type>,
    },
}

impl Expression {
    pub fn get_type(&self) -> Type {
        match self {
            Expression::Variable(_, ty) => ty.unwrap_or(Type::Promised),
            Expression::Number(_) => Type::Int,
            Expression::Bool(_) => Type::Bool,
            Expression::UnaryOperation { ty, .. } => ty.unwrap_or(Type::Promised),
            Expression::BinaryOperation { ty, .. } => ty.unwrap_or(Type::Promised),
            Expression::ProcCall { ty, .. } => ty.unwrap_or(Type::Promised),
        }
    }
}

pub enum Operator {
    Plus,
    Minus,
    Star,
    Slash,
    Mod,
    Neg,

    Pipe,
    Caret,
    Ampersand,
    Tilde,
    LShift,
    RShift,

    Equal,
    NEqual,
    L,
    LTE,
    G,
    GTE,
    LAnd,
    LOr,
    LNot,
}

pub enum ScopeState {
    Global,
    Local,
}

pub enum JumpState {
    Break,
    Continue,
}
