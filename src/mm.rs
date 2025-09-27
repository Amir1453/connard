use crate::ast::*;
use serde::{Serialize, Serializer};
use std::{collections::HashMap, sync::LazyLock};

#[derive(Serialize)]
pub struct TACWrapper {
    pub proc: &'static str,
    pub body: Vec<TAC>,
}

const OPCODES: LazyLock<HashMap<Operator, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        (Operator::Plus, "add"),
        (Operator::Minus, "sub"),
        (Operator::Star, "mul"),
        (Operator::Slash, "div"),
        (Operator::Mod, "mod"),
        (Operator::Neg, "neg"),
        (Operator::Pipe, "or"),
        (Operator::Caret, "xor"),
        (Operator::Ampersand, "and"),
        (Operator::Tilde, "not"),
        (Operator::LShift, "shl"),
        (Operator::RShift, "shr"),
        // TAC //
        (Operator::CP, "copy"),
        (Operator::PRINT, "print"),
        (Operator::CNST, "const"),
    ])
});

fn serialize_token<S>(token: &Operator, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let key = OPCODES[&token];
    key.serialize(s)
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct TAC {
    #[serde(serialize_with = "serialize_token")]
    opcode: Operator,
    args: TACArgs,
    result: Option<Temp>,
}

#[derive(Clone, Debug, PartialEq)]
enum TACArgs {
    Temporaries(Vec<Temp>),
    Number(i64),
}

impl Serialize for TACArgs {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            TACArgs::Temporaries(v) => v.serialize(s),
            TACArgs::Number(n) => vec![n].serialize(s),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct Temp(i64);

impl Serialize for Temp {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let key = format!("%{}", self.0);
        key.serialize(s)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MM {
    vars: HashMap<String, Temp>,
    tacs: Vec<TAC>,
    counter: i64,
}

impl MM {
    fn new() -> Self {
        Self {
            vars: HashMap::new(),
            tacs: Vec::new(),
            counter: -1,
        }
    }

    pub fn munch(program: &Program) -> Vec<TAC> {
        let mut mm = MM::new();
        mm.munch_program(program);
        mm.tacs
    }

    fn munch_program(&mut self, program: &Program) {
        for stmt in &program.0 {
            self.munch_statement(stmt);
        }
    }

    fn munch_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Variable { name, value } => {
                let fresh_temporary = self.fresh_temp();
                self.vars.insert(name.into(), fresh_temporary);
                let munched_value = self.munch_expression(value);
                self.populate(TAC {
                    opcode: Operator::CP,
                    args: TACArgs::Temporaries(vec![munched_value]),
                    result: Some(fresh_temporary),
                });
            }

            Statement::Assignment { name, value } => {
                let munched_value = self.munch_expression(value);
                self.populate(TAC {
                    opcode: Operator::CP,
                    args: TACArgs::Temporaries(vec![munched_value]),
                    result: Some(self.vars[name]),
                });
            }

            Statement::Print { value } => {
                let munched_value = self.munch_expression(value);
                self.populate(TAC {
                    opcode: Operator::PRINT,
                    args: TACArgs::Temporaries(vec![munched_value]),
                    result: None,
                });
            }

            _ => todo!(),
        }
    }

    fn munch_expression(&mut self, expr: &Expression) -> Temp {
        match expr {
            Expression::Variable(name) => self.vars[name],

            Expression::Number(value) => {
                let target = self.fresh_temp();
                self.populate(TAC {
                    opcode: Operator::CNST,
                    args: TACArgs::Number(value.clone()),
                    result: Some(target),
                });
                target
            }

            Expression::BinaryOperation { lhs, operator, rhs } => {
                let target = self.fresh_temp();
                let munched_lhs = self.munch_expression(lhs);
                let munched_rhs = self.munch_expression(rhs);
                self.populate(TAC {
                    opcode: operator.clone(),
                    args: TACArgs::Temporaries(vec![munched_lhs, munched_rhs]),
                    result: Some(target),
                });
                target
            }

            Expression::UnaryOperation { operator, value } => {
                let target = self.fresh_temp();
                let munched_value = self.munch_expression(value);
                self.populate(TAC {
                    opcode: operator.clone(),
                    args: TACArgs::Temporaries(vec![munched_value]),
                    result: Some(target),
                });
                target
            }
            _ => todo!(),
        }
    }

    fn fresh_temp(&mut self) -> Temp {
        self.counter += 1;
        Temp(self.counter)
    }

    fn populate(&mut self, tac: TAC) {
        self.tacs.push(tac);
    }
}
