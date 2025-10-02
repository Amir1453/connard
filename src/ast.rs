use crate::types::Type;
use std::fmt::Debug;

pub type Program = Block;

#[derive(Clone, Debug, PartialEq)]
pub struct Block(pub Vec<Statement>);

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Variable {
        name: String,
        value: Box<Expression>,
        ty: Type,
    },

    Block(Box<Block>),

    Assignment {
        name: String,
        value: Box<Expression>,
    },

    Print {
        value: Box<Expression>,
    },

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

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Variable(String),
    Number(i64),
    Bool(bool),

    BinaryOperation {
        lhs: Box<Expression>,
        operator: Operator,
        rhs: Box<Expression>,
        ty: Option<Type>,
    },

    UnaryOperation {
        operator: Operator,
        value: Box<Expression>,
        ty: Option<Type>,
    },
}

#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum JumpState {
    Break,
    Continue,
}
