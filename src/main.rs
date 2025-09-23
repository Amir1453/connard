mod ast;
mod lexer;
mod tokens;

use crate::lexer::Lexer;
use lalrpop_util::lalrpop_mod;
use std::fs;

lalrpop_mod!(pub bxgrammar);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = bxgrammar::BXParser::new();

    let entries = fs::read_dir("tests/examples")?;
    for entry in entries {
        let path = entry?.path();
        println!("{:?}", path);

        let source_code = std::fs::read_to_string(&path)?;
        let lexer = Lexer::new(&source_code);
        let ast = parser.parse(lexer)?;

        println!("{:?}", ast);
    }

    Ok(())
}
