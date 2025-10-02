use crate::ast::Operator;
use serde::{Serialize, Serializer};
use std::{collections::HashMap, sync::LazyLock};

#[derive(Serialize)]
pub struct TACWrapper {
    proc: &'static str,
    body: Vec<TAC>,
}

impl TACWrapper {
    pub fn new(body: Vec<TAC>) -> Self {
        Self {
            proc: "@main",
            body,
        }
    }
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct TAC {
    opcode: TACOpcode,
    args: TACArgs,
    result: Option<Temp>,
}

impl TAC {
    pub fn new(opcode: TACOpcode, args: TACArgs, result: Option<Temp>) -> Self {
        Self {
            opcode,
            args,
            result,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TACArgs {
    Temporaries((Temp, Temp)),
    Temporary(Temp),
    TempAndLabel((Temp, i64)),
    Number(i64),
    Label(i64),
}

impl Serialize for TACArgs {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            TACArgs::Temporaries((v, u)) => vec![v, u].serialize(s),
            TACArgs::Temporary(v) => vec![v].serialize(s),
            TACArgs::TempAndLabel((v, u)) => {
                vec![format!("%{}", v.0), format!("%.L{}", u)].serialize(s)
            }
            TACArgs::Number(n) => vec![n].serialize(s),
            TACArgs::Label(l) => vec![format!("%.L{}", l)].serialize(s),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Temp(pub i64);

impl Serialize for Temp {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let key = format!("%{}", self.0);
        key.serialize(s)
    }
}

#[derive(Hash, Clone, Debug, PartialEq, Eq)]
pub enum TACOpcode {
    COPY,
    PRINT,
    CONST,
    LABEL,

    ADD,
    SUB,
    MUL,
    DIV,
    MOD,
    NEG,

    OR,
    XOR,
    AND,
    NOT,
    SHL,
    SHR,

    JMP,
    JZ,
    JNZ,
    JL,
    JNL,
    JLE,
    JNLE,

    ERROR,
}

const OPCODES: LazyLock<HashMap<TACOpcode, &'static str>> = LazyLock::new(|| {
    use TACOpcode::*;
    HashMap::from([
        (COPY, "copy"),
        (PRINT, "print"),
        (CONST, "const"),
        (LABEL, "label"),
        (ADD, "add"),
        (SUB, "sub"),
        (MUL, "mul"),
        (DIV, "div"),
        (MOD, "mod"),
        (NEG, "neg"),
        (OR, "or"),
        (XOR, "xor"),
        (AND, "and"),
        (NOT, "not"),
        (SHL, "shl"),
        (SHR, "shr"),
        (JMP, "jmp"),
        (JZ, "jz"),
        (JNZ, "jnz"),
        (JL, "jl"),
        (JNL, "jnl"),
        (JLE, "jle"),
        (JNLE, "jnle"),
        (ERROR, "error"),
    ])
});

impl Serialize for TACOpcode {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let key = OPCODES[&self];
        key.serialize(s)
    }
}

impl Into<TACOpcode> for Operator {
    fn into(self) -> TACOpcode {
        use Operator::*;
        use TACOpcode::*;
        match self {
            Plus => ADD,
            Minus => SUB,
            Star => MUL,
            Slash => DIV,
            Mod => MOD,
            Neg => NEG,
            Pipe => OR,
            Caret => XOR,
            Ampersand => AND,
            Tilde => NOT,
            LShift => SHL,
            RShift => SHR,
            _ => ERROR,
        }
    }
}
