#![allow(dead_code)]

use crate::{
    cfg::{BasicBlocks, CFG},
    tac::{CUTAC, TACDeclaration},
};

pub struct Optimizer {
    // cfg: CFG,
}

impl Optimizer {
    pub fn optimize(cutac: CUTAC) -> CUTAC {
        use TACDeclaration::*;
        let mut optimized_cutac: CUTAC = CUTAC(Vec::with_capacity(cutac.len()));

        for decl in cutac.0.into_iter() {
            if let ProcDecl(mut proc) = decl {
                let mut cfg = CFG::from(BasicBlocks::from(proc.instructions));

                cfg.remove_unreachable();
                cfg.coalesce_blocks();
                cfg.jump_threading();

                proc.instructions = cfg.serialize_tac();

                optimized_cutac.push(ProcDecl(proc));
            } else {
                optimized_cutac.push(decl);
            }
        }
        optimized_cutac
    }
}
