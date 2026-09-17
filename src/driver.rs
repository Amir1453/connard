use std::{collections::HashMap, path::PathBuf};

use crate::{
    asm::Asm,
    ast::{Declaration, Program},
    bxgrammar,
    lexer::Lexer,
    mm::MM,
    // optimizer::Optimizer,
    options::CompilerOptions,
    semchecker::SemChecker,
    typechecker::{SemanticType, TypeChecker},
    types::{Name, ProcType},
};

pub struct Driver {}

impl Driver {
    pub fn cli() -> Result<(), Box<dyn std::error::Error>> {
        let options = CompilerOptions::from_env();

        if options.cu_is_empty() {
            println!("Maybe specify some files !");
            return Ok(());
        }

        let parser = bxgrammar::BXParser::new();

        let mut global_decls: HashMap<Name, SemanticType> = HashMap::new();
        let mut cu_programs: Vec<(PathBuf, Program)> = Vec::new();

        for cu in options.compilation_units.into_iter() {
            let cu_source_code = std::fs::read_to_string(&cu)?;
            let cu_lexer = Lexer::new(&cu_source_code);
            let cu_program = parser.parse(cu_lexer)?;

            cu_programs.push((cu, cu_program));
        }

        for (_, cu_program) in cu_programs.iter() {
            for decl in cu_program.0.iter() {
                match decl {
                    Declaration::Variable(variable) => {
                        let var = variable.as_ref();
                        for name in var.names.clone().into_iter() {
                            if global_decls.contains_key(&name) {
                                return Err(
                                    format!("Duplicate Global Declaration: {}", &name).into()
                                );
                            }

                            global_decls.insert(name, SemanticType::SimpleType(var.ty));
                        }
                    }

                    Declaration::Proc {
                        proc_name,
                        proc_args,
                        return_type,
                        ..
                    } => {
                        if global_decls.contains_key(proc_name) {
                            return Err(
                                format!("Duplicate Global Declaration: {}", &proc_name).into()
                            );
                        }

                        global_decls.insert(
                            proc_name.clone(),
                            SemanticType::ProcType(ProcType {
                                args_type: proc_args.clone(),
                                return_type: *return_type,
                            }),
                        );
                    }
                }
            }
        }

        if !(global_decls.contains_key("main")
            && matches!(global_decls.get("main"), Some(SemanticType::ProcType(_))))
        {
            panic!()
        }

        for (cu, mut cu_program) in cu_programs.into_iter() {
            match SemChecker::check(&cu_program, global_decls.keys().cloned().collect()) {
                Some(err) => {
                    for error in err {
                        println!("{}: {:?}", cu.display(), error);
                    }
                    panic!()
                }
                None => println!("Semantic Check successful @ {}!", cu.display()),
            }

            match TypeChecker::check(&mut cu_program, global_decls.clone()) {
                Some(err) => {
                    for error in err {
                        println!("{}: {:?}", cu.display(), error);
                    }
                    panic!()
                }
                None => println!("Type Check successful @ {}!", cu.display()),
            }

            // println!("Munching {}...", cu);
            let cutac = MM::munch(cu_program);
            println!("{}", cutac);

            // println!("Optimizing {}...", cu);
            // let optimized_cutac = Optimizer::optimize(cutac);

            // let asm = Asm::lower(optimized_cutac);
            let asm = Asm::lower(cutac);

            let stem = cu
                .file_stem()
                .and_then(|s| s.to_str())
                .expect("failed to get file stem");
            let mut out_path = PathBuf::from(stem);
            out_path.set_extension("s");

            std::fs::write(out_path, &asm)?;
            println!("{}", asm);
        }

        Ok(())
    }

    pub fn drive(cu: &PathBuf, out_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let parser = bxgrammar::BXParser::new();

        let mut global_decls: HashMap<Name, SemanticType> = HashMap::new();

        let cu_source_code = std::fs::read_to_string(cu)?;
        let cu_lexer = Lexer::new(&cu_source_code);
        let mut cu_program = parser.parse(cu_lexer).unwrap();

        for decl in cu_program.0.iter() {
            match decl {
                Declaration::Variable(variable) => {
                    let var = variable.as_ref();
                    for name in var.names.clone().into_iter() {
                        if global_decls.contains_key(&name) {
                            panic!("Duplicate Global Declaration !");
                            // return Err(format!("Duplicate Global Declaration: {}", &name).into());
                        }

                        global_decls.insert(name, SemanticType::SimpleType(var.ty));
                    }
                }

                Declaration::Proc {
                    proc_name,
                    proc_args,
                    return_type,
                    ..
                } => {
                    if global_decls.contains_key(proc_name) {
                        panic!("Duplicate Global Declaration !");
                        // return Err(format!("Duplicate Global Declaration: {}", &proc_name).into());
                    }

                    global_decls.insert(
                        proc_name.clone(),
                        SemanticType::ProcType(ProcType {
                            args_type: proc_args.clone(),
                            return_type: *return_type,
                        }),
                    );
                }
            }
        }

        if !(global_decls.contains_key("main")
            && matches!(global_decls.get("main"), Some(SemanticType::ProcType(_))))
        {
            panic!()
        }

        if let Some(err) = SemChecker::check(&cu_program, global_decls.keys().cloned().collect()) {
            for error in err {
                println!("{}: {:?}", cu.display(), error);
            }
            panic!()
        }

        if let Some(err) = TypeChecker::check(&mut cu_program, global_decls.clone()) {
            for error in err {
                println!("{}: {:?}", cu.display(), error);
            }
            panic!()
        }

        let cutac = MM::munch(cu_program);
        std::fs::write(out_path.with_added_extension("tac"), format!("{cutac}"))?;

        // let optimized_cutac = Optimizer::optimize(cutac);
        // let asm = Asm::lower(optimized_cutac);
        let asm = Asm::lower(cutac);
        std::fs::write(out_path, &asm)?;

        Ok(())
    }
}
