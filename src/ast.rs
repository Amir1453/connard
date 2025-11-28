use crate::types::{Name, Promise, Type};

pub struct Program(pub Vec<Declaration>);

pub enum Declaration {
    Variable(Box<Variable>),

    Proc {
        proc_name: Name,
        proc_args: Option<Vec<(Name, Type)>>,
        return_type: Option<Type>,
        block: Box<Block>,
    },
}

pub struct Variable {
    pub names: Vec<Name>,
    pub values: Vec<Box<Expression>>,
    pub ty: Type,
    pub scope: ScopeState,
}

pub struct Block(pub Vec<Statement>);

pub enum Statement {
    Variable(Box<Variable>),

    Block(Box<Block>),

    Assignment {
        name: Name,
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
    Variable(Name, Promise<Type>),
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
        proc_name: Name,
        proc_args: Option<Vec<Box<Expression>>>,
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

// Printing for Program

// use std::fmt::{self, Display, Formatter};
//
// impl Display for Program {
//     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
//         print_program(f, 0, self)
//     }
// }
//
// fn print_program(f: &mut Formatter<'_>, indent: usize, program: &Program) -> fmt::Result {
//     write_line(f, indent, "Program")?;
//     for decl in &program.0 {
//         print_declaration(f, indent + 1, decl)?;
//         write!(f, "\n")?;
//     }
//     Ok(())
// }
//
// fn write_line(f: &mut Formatter<'_>, indent: usize, s: &str) -> fmt::Result {
//     write_indent(f, indent)?;
//     writeln!(f, "{}", s)
// }
//
// fn write_indent(f: &mut Formatter<'_>, indent: usize) -> fmt::Result {
//     for _ in 0..indent {
//         write!(f, "    ")?;
//     }
//     Ok(())
// }
//
// fn print_declaration(f: &mut Formatter<'_>, indent: usize, d: &Declaration) -> fmt::Result {
//     match d {
//         Declaration::Variable(v) => {
//             write_line(f, indent, "Global Variable")?;
//             print_variable(f, indent + 1, v)
//         }
//         Declaration::Proc {
//             proc_name,
//             proc_args,
//             return_type,
//             block,
//         } => {
//             let mut header = format!("Declaration::Proc {}", proc_name);
//             if let Some(rt) = return_type {
//                 header.push_str(&format!(" -> {}", display_type(rt)));
//             }
//             write_line(f, indent, &header)?;
//             if let Some(args) = proc_args {
//                 write_line(f, indent + 1, "args:")?;
//                 for (name, ty) in args {
//                     write_line(f, indent + 2, &format!("{}: {}", name, display_type(ty)))?;
//                 }
//             } else {
//                 write_line(f, indent + 1, "args: none")?;
//             }
//             write_line(f, indent + 1, "body:")?;
//             print_block(f, indent + 2, block)
//         }
//     }
// }
//
// fn print_variable(f: &mut Formatter<'_>, indent: usize, v: &Variable) -> fmt::Result {
//     write_line(f, indent, &format!("Type {}", display_type(&v.ty)))?;
//     write_line(f, indent + 1, "names:")?;
//     for name in &v.names {
//         write_line(f, indent + 2, name)?;
//     }
//     write_line(f, indent + 1, "values:")?;
//     for val in &v.values {
//         print_expression(f, indent + 2, val)?;
//     }
//     Ok(())
// }
//
// fn print_block(f: &mut Formatter<'_>, indent: usize, b: &Block) -> fmt::Result {
//     write_line(f, indent, "Block")?;
//     for stmt in &b.0 {
//         print_statement(f, indent + 1, stmt)?;
//     }
//     Ok(())
// }
//
// fn print_statement(f: &mut Formatter<'_>, indent: usize, s: &Statement) -> fmt::Result {
//     match s {
//         Statement::Variable(v) => {
//             write_line(f, indent, "Local Variable")?;
//             print_variable(f, indent + 1, v)
//         }
//         Statement::Block(b) => {
//             write_line(f, indent, "Statement::Block")?;
//             print_block(f, indent + 1, b)
//         }
//         Statement::Assignment { name, value } => {
//             write_line(f, indent, &format!("Assignment: {}", name))?;
//             print_expression(f, indent + 1, value)
//         }
//         Statement::Eval(expr) => {
//             write_line(f, indent, "Eval")?;
//             print_expression(f, indent + 1, expr)
//         }
//         Statement::Return(opt) => {
//             if let Some(expr) = opt {
//                 write_line(f, indent, "Return")?;
//                 print_expression(f, indent + 1, expr)
//             } else {
//                 write_line(f, indent, "Return (void)")
//             }
//         }
//         Statement::If {
//             condition,
//             then_block,
//             else_branch,
//         } => {
//             write_line(f, indent, "If")?;
//             write_line(f, indent + 1, "condition:")?;
//             print_expression(f, indent + 2, condition)?;
//             write_line(f, indent + 1, "then:")?;
//             print_block(f, indent + 2, then_block)?;
//             match else_branch {
//                 Some(else_stmt) => {
//                     write_line(f, indent + 1, "else:")?;
//                     print_statement(f, indent + 2, else_stmt)
//                 }
//                 None => write_line(f, indent + 1, "else: none"),
//             }
//         }
//         Statement::While { condition, block } => {
//             write_line(f, indent, "While")?;
//             write_line(f, indent + 1, "condition:")?;
//             print_expression(f, indent + 2, condition)?;
//             write_line(f, indent + 1, "body:")?;
//             print_block(f, indent + 2, block)
//         }
//         Statement::Jump(j) => write_line(f, indent, &format!("Jump::{:?}", j)),
//     }
// }
//
// fn print_expression(f: &mut Formatter<'_>, indent: usize, e: &Expression) -> fmt::Result {
//     match e {
//         Expression::Variable(name) => write_line(f, indent, &format!("VariableExpr: {}", name)),
//         Expression::Number(n) => write_line(f, indent, &format!("Number: {}", n)),
//         Expression::Bool(b) => write_line(f, indent, &format!("Bool: {}", b)),
//         Expression::BinaryOperation {
//             lhs,
//             operator,
//             rhs,
//             ty,
//         } => {
//             write_line(f, indent, &format!("BinaryOperation ({:?})", operator))?;
//             write_line(f, indent + 1, "lhs:")?;
//             print_expression(f, indent + 2, lhs)?;
//             write_line(f, indent + 1, "rhs:")?;
//             print_expression(f, indent + 2, rhs)?;
//             write_line(
//                 f,
//                 indent + 1,
//                 &format!(
//                     "type: {}",
//                     ty.as_ref().map_or("none".into(), |t| display_type(t))
//                 ),
//             )
//         }
//         Expression::UnaryOperation {
//             operator,
//             value,
//             ty,
//         } => {
//             write_line(f, indent, &format!("UnaryOperation ({})", operator))?;
//             write_line(f, indent + 1, "value:")?;
//             print_expression(f, indent + 2, value)?;
//             write_line(
//                 f,
//                 indent + 1,
//                 &format!(
//                     "type: {}",
//                     ty.as_ref().map_or("none".into(), |t| display_type(t))
//                 ),
//             )
//         }
//         Expression::ProcCall {
//             proc_name,
//             proc_args,
//         } => {
//             write_line(f, indent, &format!("ProcCall: {}", proc_name))?;
//             if let Some(args) = proc_args {
//                 write_line(f, indent + 1, "args:")?;
//                 for a in args {
//                     print_expression(f, indent + 2, a)?;
//                 }
//             } else {
//                 write_line(f, indent + 1, "args: none")?;
//             }
//             Ok(())
//         }
//     }
// }
//
// fn display_type(t: &Type) -> String {
//     format!("{:?}", t)
// }
