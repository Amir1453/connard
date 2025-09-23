use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Variable {
        name: String,
        value: Box<Expression>,
    },

    Assignment {
        name: String,
        value: Box<Expression>,
    },

    Print {
        value: Box<Expression>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Variable(String),
    Number(i64),

    BinaryOperation {
        lhs: Box<Expression>,
        operator: Operator,
        rhs: Box<Expression>,
    },

    UnaryOperation {
        operator: Operator,
        value: Box<Expression>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Or,
    Xor,
    And,
    Comp,
    LShift,
    RShift,
}
