mod ast;
mod lexer;
mod mm;
mod synchecker;
mod tac;
mod tokens;
mod typechecker;
mod types;

#[cfg(test)]
mod tests;

use crate::{
    lexer::Lexer, mm::MM, synchecker::SynChecker, tac::TACWrapper, typechecker::TypeChecker,
};
use lalrpop_util::lalrpop_mod;
use std::{env, fs};

lalrpop_mod!(pub bxgrammar);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = env::args().nth(1).ok_or("Usage: bx-compiler <filename>")?;

    let source_code = std::fs::read_to_string(&filename)?;
    let lexer = Lexer::new(&source_code);
    let parser = bxgrammar::BXParser::new();
    let mut program = parser.parse(lexer)?;

    match SynChecker::check(&program) {
        Some(err) => {
            for error in err {
                println!("{:?}", error);
            }
        }
        None => println!("All good?"),
    }

    match TypeChecker::check(&mut program) {
        Some(err) => {
            for error in err {
                println!("{:?}", error);
            }
        }
        None => println!("All good!"),
    }

    println!("{:?}", program);

    let tac = MM::munch(&program);
    let printable_tac = TACWrapper::new(tac);

    let pretty = serde_json::to_string_pretty(&vec![printable_tac])?;
    // println!("{}", pretty);
    fs::write("source.tac.json", pretty)?;

    Ok(())
}
