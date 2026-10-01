pub struct SessionGlobals {
    symbol_interner: crate::frontend::Interner,
}

impl SessionGlobals {
    pub fn new(extra_symbols: &[&'static str]) -> SessionGlobals {
        SessionGlobals {
            symbol_interner: crate::frontend::Interner::with_extra_symbols(extra_symbols),
        }
    }

    pub fn get_symbol_interner(&self) -> &crate::frontend::Interner {
        &self.symbol_interner
    }
}

scoped_tls::scoped_thread_local!(static SESSION_GLOBALS: SessionGlobals);

#[inline]
pub fn with_session_globals<R, F>(f: F) -> R
where
    F: FnOnce(&SessionGlobals) -> R,
{
    SESSION_GLOBALS.with(f)
}

pub fn create_session_globals_then<R>(extra_symbols: &[&'static str], f: impl FnOnce() -> R) -> R {
    assert!(
        !SESSION_GLOBALS.is_set(),
        "SESSION_GLOBALS should never be overwritten! \
         Use another thread if you need another SessionGlobals"
    );
    let session_globals = SessionGlobals::new(extra_symbols);
    SESSION_GLOBALS.set(&session_globals, f)
}
