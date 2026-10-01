use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::LazyLock;

use crate::structs::ErrorAggregate;

#[allow(dead_code)]
pub enum MetaOptions {
    CompilationThreads(u8),
}

#[derive(Copy, Clone)]
pub enum WarningOptions {
    W,
    All,
    Extra,
    Pedantic,
}

static WARNINGS: LazyLock<HashMap<&'static str, WarningOptions>> = LazyLock::new(|| {
    use WarningOptions::*;
    HashMap::from([
        ("-W", W),
        ("-Wall", All),
        ("-Wextra", Extra),
        ("-Wpedantic", Pedantic),
    ])
});

#[derive(Copy, Clone)]
pub enum CFGInstrumentationOptions {
    Nonce,
    All,
    CoalesceBlocks,
    JumpThreading,
    PruneDeadCode,
}

static INST: LazyLock<HashMap<&'static str, CFGInstrumentationOptions>> = LazyLock::new(|| {
    use CFGInstrumentationOptions::*;
    HashMap::from([
        ("-fnone", Nonce),
        ("-fall", All),
        ("-fcoalesce-blocks", CoalesceBlocks),
        ("-fthread-jumps", JumpThreading),
        ("-fprune-dead-code", PruneDeadCode),
    ])
});

pub struct CompilerOptions {
    pub compilation_units: Vec<PathBuf>,
    #[allow(dead_code)]
    pub warning_options: Vec<WarningOptions>,
    #[allow(dead_code)]
    pub cfg_instrumentation_options: Vec<CFGInstrumentationOptions>,
    pub out_path: Option<PathBuf>,
}

impl CompilerOptions {
    pub fn from_env() -> Result<Self, ErrorAggregate<CompilerOptionsErrorType>> {
        use CompilerOptionsErrorType::*;

        let mut compilation_units = Vec::new();
        let mut warning_options = Vec::new();
        let mut cfg_instrumentation_options = Vec::new();
        let mut out_path = None;

        let mut errors = ErrorAggregate::<CompilerOptionsErrorType>::new();

        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            if arg.ends_with(".bx") {
                compilation_units.push(PathBuf::from(arg));
                continue;
            }

            if let Some(&opt) = WARNINGS.get(arg.as_str()) {
                warning_options.push(opt);
                continue;
            }

            if let Some(&opt) = INST.get(arg.as_str()) {
                cfg_instrumentation_options.push(opt);
                continue;
            }

            if arg == "-o" {
                let Some(output) = args.next() else {
                    errors.add_error(OArgumentNotSpecified);
                    continue;
                };

                if out_path.is_some() {
                    errors.add_error(OArgumentDuplicated);
                    continue;
                }

                if !output.ends_with(".s") {
                    errors.add_error(OArgumentNotASM);
                    continue;
                }

                out_path = Some(PathBuf::from(output));
                continue;
            }

            errors.add_error(UnknownOption(arg.clone()));
        }

        if compilation_units.is_empty() {
            errors.add_error(EmptyInput);
        }

        if out_path.is_some() && compilation_units.len() > 1 {
            errors.add_error(OArgumentTooManyCU);
        }

        if let Err(errors) = errors.resolve() {
            return Err(errors);
        }

        Ok(Self {
            compilation_units,
            warning_options,
            cfg_instrumentation_options,
            out_path,
        })
    }

    pub fn from_paths(input: impl Into<PathBuf>, output: impl Into<PathBuf>) -> Self {
        Self {
            compilation_units: vec![input.into()],
            warning_options: Vec::new(),
            cfg_instrumentation_options: Vec::new(),
            out_path: Some(output.into()),
        }
    }
}

#[derive(thiserror::Error, Clone, Debug, PartialEq)]
pub enum CompilerOptionsErrorType {
    #[error("-o argument not specified")]
    OArgumentNotSpecified,
    #[error("-o argument must be an assembly file (.s)")]
    OArgumentNotASM,
    #[error("-o argument was provided too many times")]
    OArgumentDuplicated,
    #[error("cannot specify -o with multiple files")]
    OArgumentTooManyCU,

    #[error("no input files")]
    EmptyInput,

    #[error("unrecognized option: {0}")]
    UnknownOption(String),
}
