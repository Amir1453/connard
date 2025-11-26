mod ast;
mod cfg;
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

use crate::{
    lexer::Lexer, mm::MM, optimizer::Optimizer, options::CompilerOptions, semchecker::SemChecker,
    typechecker::TypeChecker,
};

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub bxgrammar);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = CompilerOptions::from_env();
    let parser = bxgrammar::BXParser::new();

    for cu in options.compilation_units {
        let cu_source_code = std::fs::read_to_string(&cu)?;
        let cu_lexer = Lexer::new(&cu_source_code);
        let mut cu_program = parser.parse(cu_lexer)?;

        match SemChecker::check(&cu_program) {
            Some(err) => {
                for error in err {
                    println!("{}: {:?}", cu, error);
                }
                panic!()
            }
            None => println!("Semantic Check successful @ {}!", cu),
        }

        match TypeChecker::check(&mut cu_program) {
            Some(err) => {
                for error in err {
                    println!("{}: {:?}", cu, error);
                }
                panic!()
            }
            None => println!("Type Check successful @ {}!", cu),
        }

        println!("Munching {}...", cu);
        let cutac = MM::munch(cu_program);
        println!("{}", cutac);

        println!("Optimizing {}...", cu);
        let optimized_cutac = Optimizer::optimize(cutac);
        println!("{}", optimized_cutac);

        // let proc = (*crate::tests::FIBONACCI).clone();
        // let fake_cutac = tac::CUTAC(vec![tac::TACDeclaration::ProcDecl(proc)]);
        // println!("{}", fake_cutac);
        //
        // let optimized_cutac = Optimizer::optimize(fake_cutac);
        // println!("{}", optimized_cutac);
    }

    Ok(())
}
