mod asm;
mod ast;
mod cfg;
pub mod driver;
mod lexer;
mod mm;
mod optimizer;
mod options;
mod semchecker;
mod tac;
mod tokens;
mod typechecker;
mod types;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(pub bxgrammar);
