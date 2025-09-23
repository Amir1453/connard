use logos::Logos;
use std::fmt;
use std::num::ParseIntError;

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexicalError {
    InvalidInteger(ParseIntError),
    #[default]
    InvalidToken,
}

impl From<ParseIntError> for LexicalError {
    fn from(err: ParseIntError) -> Self {
        LexicalError::InvalidInteger(err)
    }
}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexicalError::InvalidInteger(err) => write!(f, "Invalid integer: {}", err),
            LexicalError::InvalidToken => write!(f, "Invalid token encountered"),
        }
    }
}

#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+", skip r"//.*\n?", error = LexicalError)]
pub enum Token {
    #[token("def")]
    KeywordDef,
    #[token("main")]
    KeywordMain,
    #[token("var")]
    KeywordVar,
    #[token("int")]
    KeywordInt,
    #[token("print")]
    KeywordPrint,

    #[regex("[_a-zA-Z][_0-9a-zA-Z]*", |lex| lex.slice().to_string())]
    Identifier(String),
    #[regex("[0-9]*", |lex| lex.slice().parse())]
    Number(i64),

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LCurlyParen,
    #[token("}")]
    RCurlyParen,
    #[token("=")]
    Assign,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,

    #[token("+")]
    OperatorAdd,
    #[token("-")]
    OperatorSub,
    #[token("*")]
    OperatorMul,
    #[token("/")]
    OperatorDiv,
    #[token("%")]
    OperatorMod,

    #[token("|")]
    OperatorOr,
    #[token("^")]
    OperatorXor,
    #[token("&")]
    OperatorAnd,
    #[token("~")]
    OperatorComp,
    #[token("<<")]
    OperatorLShift,
    #[token(">>")]
    OperatorRShift,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
