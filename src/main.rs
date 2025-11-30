mod ast;
mod cfg;
mod driver;
mod lexer;
mod mm;
mod optimizer;
mod options;
mod semchecker;
mod tac;
mod tokens;
mod typechecker;
mod types;

// #[cfg(test)]
mod tests;

use lalrpop_util::lalrpop_mod;

use crate::driver::Driver;

lalrpop_mod!(pub bxgrammar);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Driver::drive()
}
