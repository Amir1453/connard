use std::{collections::HashMap, env, sync::LazyLock};

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
    pub compilation_units: Vec<String>,
    #[allow(dead_code)]
    pub warning_options: Vec<WarningOptions>,
    #[allow(dead_code)]
    pub cfg_instrumentation_options: Vec<CFGInstrumentationOptions>,
}

impl CompilerOptions {
    pub fn from_env() -> Self {
        let mut compilation_units = Vec::new();
        let mut warning_options = Vec::new();
        let mut cfg_instrumentation_options = Vec::new();

        for i in env::args().skip(1) {
            if i.ends_with(".bx") {
                compilation_units.push(i);
                continue;
            }

            if let Some(&opt) = WARNINGS.get(i.as_str()) {
                warning_options.push(opt);
                continue;
            }

            if let Some(&opt) = INST.get(i.as_str()) {
                cfg_instrumentation_options.push(opt);
                continue;
            }

            println!("Ignoring unrecognized option: {:?}", i);
        }

        Self {
            compilation_units,
            warning_options,
            cfg_instrumentation_options,
        }
    }
}
