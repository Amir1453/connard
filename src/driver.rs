use std::collections::{HashMap, hash_map::Entry};
use std::path::PathBuf;

use anyhow::{Context, bail};

use crate::frontend::ast::{Declaration, Program};
use crate::frontend::{BXParser, Lexer, SemChecker, TypeChecker};

use crate::ir::MM;

use crate::optimizer::Optimizer;

use crate::backend::Asm;

use crate::options::CompilerOptions;
use crate::types::{Name, ProcType, SemanticType};

pub struct Driver {}

impl Driver {
    pub fn cli() -> anyhow::Result<()> {
        let options = CompilerOptions::from_env();

        if options.cu_is_empty() {
            bail!("no input files")
        }

        Driver::compile(options)
    }

    pub fn drive(cu: &PathBuf, out_path: &PathBuf) -> anyhow::Result<()> {
        let options = CompilerOptions::from_singular(cu, out_path);

        Driver::compile(options)
    }

    fn compile(options: CompilerOptions) -> anyhow::Result<()> {
        // Generate the parser
        let parser = BXParser::new();

        // Accumulate compilation unit programs
        let mut cu_programs: Vec<(PathBuf, Program)> = Vec::new();
        for cu in options.compilation_units.into_iter() {
            let cu_source_code = std::fs::read_to_string(&cu)?;
            let cu_lexer = Lexer::new(&cu_source_code);
            let cu_program = parser.parse(cu_lexer)?;

            cu_programs.push((cu, cu_program));
        }

        let mut global_decls: HashMap<Name, SemanticType> = HashMap::new();

        let mut insert_global_decl = |name: Name, ty: SemanticType| match global_decls.entry(name) {
            Entry::Vacant(entry) => {
                entry.insert(ty);
                Ok(())
            }
            Entry::Occupied(entry) => {
                bail!("duplicate global declaration: {}", entry.key())
            }
        };

        for (_, cu_program) in &cu_programs {
            for decl in &cu_program.0 {
                match decl {
                    Declaration::Variable(variable) => {
                        let var = variable.as_ref();

                        for name in var.names.clone().into_iter() {
                            insert_global_decl(name, SemanticType::SimpleType(var.ty))?
                        }
                    }

                    Declaration::Proc {
                        proc_name,
                        proc_args,
                        return_type,
                        ..
                    } => {
                        let ty = SemanticType::ProcType(ProcType {
                            args_type: proc_args.clone(),
                            return_type: *return_type,
                        });
                        insert_global_decl(proc_name.clone(), ty)?
                    }
                }
            }
        }

        // Making sure a main function exists
        if !(global_decls.contains_key("main")
            && matches!(global_decls.get("main"), Some(SemanticType::ProcType(_))))
        {
            bail!("No main() ! Maybe I am a linker ?")
        }

        // Handling the case for Driver::drive()
        if let Some(out_path) = options.out_path {
            let (_cu, mut cu_program) = cu_programs
                .into_iter()
                .next()
                .context("Program List Empty!")?;

            SemChecker::check(&cu_program, global_decls.keys().cloned().collect())?;
            TypeChecker::check(&mut cu_program, global_decls.clone())?;

            // use crate::ir::mm_llvm;
            // use inkwell::context;
            //
            // let context = context::Context::create();
            // let module = context.create_module("test");
            // let _ = mm_llvm::MMLLVM::munch(&context, module, cu_program, global_decls)?;

            let cutac = MM::munch(cu_program);
            let cutac = Optimizer::optimize(cutac);
            std::fs::write(out_path.with_added_extension("tac"), format!("{cutac}"))?;


            let asm = Asm::lower(cutac);
            std::fs::write(out_path, &asm)?;

            return Ok(());
        }

        for (cu, mut cu_program) in cu_programs.into_iter() {
            SemChecker::check(&cu_program, global_decls.keys().cloned().collect())?;
            TypeChecker::check(&mut cu_program, global_decls.clone())?;

            // println!("Munching {}...", cu);
            let cutac = MM::munch(cu_program);
            let cutac = Optimizer::optimize(cutac);
            println!("{}", cutac);

            // let asm = Asm::lower(optimized_cutac);
            let asm = Asm::lower(cutac);
            // println!("{}", asm);

            let stem = cu
                .file_stem()
                .and_then(|s| s.to_str())
                .context("failed to get file stem")?;
            let mut out_path = PathBuf::from(stem);
            out_path.set_extension("s");

            std::fs::write(out_path, &asm)?;
        }

        Ok(())
    }
}
