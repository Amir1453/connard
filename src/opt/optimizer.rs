use crate::ir::tac::{CUTAC, TACDeclaration};

use super::cfg::CFG;

pub struct Optimizer {}

impl Optimizer {
    pub fn optimize(cutac: CUTAC) -> CUTAC {
        use TACDeclaration::*;
        let mut optimized_cutac: CUTAC = CUTAC(Vec::with_capacity(cutac.len()));

        for decl in cutac.0.into_iter() {
            if let ProcDecl(proc) = decl {
                let mut cfg = CFG::from(proc);

                cfg.jump_threading();
                cfg.remove_unreachable();
                cfg.coalesce_blocks();

                let proc = cfg.into();

                optimized_cutac.push(ProcDecl(proc));
            } else {
                optimized_cutac.push(decl);
            }
        }
        optimized_cutac
    }
}
