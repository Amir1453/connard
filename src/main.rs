mod ast;
mod lexer;
mod mm;
mod synchecker;
mod tokens;

#[cfg(test)]
mod tests;

use crate::{lexer::Lexer, mm::MM, synchecker::SynChecker};
use lalrpop_util::lalrpop_mod;
use std::{env, fs, process};

lalrpop_mod!(pub bxgrammar);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = env::args().nth(1).ok_or("Usage: bx-compiler <filename>")?;

    let source_code = std::fs::read_to_string(&filename)?;
    let lexer = Lexer::new(&source_code);
    let parser = bxgrammar::BXParser::new();
    let program = parser.parse(lexer)?;

    match SynChecker::check(&program) {
        Some(_) => process::exit(1),
        None => {}
    }

    let tac = MM::munch(&program);
    let printable_tac = mm::TACWrapper {
        proc: "@main",
        body: tac,
    };

    let pretty = serde_json::to_string(&vec![printable_tac])?;
    println!("{}", pretty);
    fs::write("source.tac.json", pretty)?;

    Ok(())
}
