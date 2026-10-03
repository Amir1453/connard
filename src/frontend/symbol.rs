// The following code was directly taken and/or inspired by the rustc_span code.
#![allow(dead_code)]

use std::hash::BuildHasher;

use hashbrown::HashTable;
use hashbrown::hash_table::Entry;

use crate::structs::DroplessArena;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol(SymbolIndex);

impl Symbol {
    /// Avoid this except for things like deserialization of previously
    /// serialized symbols, and testing. Use `intern` instead.
    pub const fn new(n: u32) -> Self {
        Symbol(SymbolIndex::from_u32(n))
    }

    /// Maps a string to its interned representation.
    #[inline]
    pub fn intern(str: &str) -> Self {
        crate::with_session_globals(|session_globals| {
            session_globals.get_symbol_interner().intern_str(str)
        })
    }

    /// Access the underlying string. This is a slowish operation because it
    /// requires locking the symbol interner.
    ///
    /// Note that the lifetime of the return value is a lie. It's not the same
    /// as `&self`, but actually tied to the lifetime of the underlying
    /// interner. Interners are long-lived, and there are very few of them, and
    /// this function is typically used for short-lived things, so in practice
    /// it works out ok.
    #[inline]
    pub fn as_str(&self) -> &str {
        crate::with_session_globals(|session_globals| unsafe {
            std::mem::transmute::<&str, &str>(session_globals.get_symbol_interner().get_str(*self))
        })
    }

    pub fn as_u32(self) -> u32 {
        self.0.as_u32()
    }
}

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_str(), f)
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.as_str(), f)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolIndex(u32);

impl SymbolIndex {
    #[inline]
    const fn new(value: usize) -> Self {
        assert!(value <= u32::MAX as usize);
        SymbolIndex(value as u32)
    }

    #[inline]
    const fn index(self) -> usize {
        self.0 as usize
    }

    #[inline]
    const fn from_usize(value: usize) -> Self {
        Self::new(value)
    }

    #[inline]
    const fn from_u32(value: u32) -> Self {
        Self(value)
    }

    #[inline]
    const fn as_u32(self) -> u32 {
        self.0
    }

    #[inline]
    const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

pub struct Interner(parking_lot::Mutex<InternerInner>);

struct InternerInner {
    arena: DroplessArena,
    indices: HashTable<(&'static [u8], u32)>,
    byte_strs: Vec<&'static [u8]>,
    hasher: hashbrown::DefaultHashBuilder,
}

impl Interner {
    pub fn new() -> Self {
        let inner = InternerInner {
            arena: Default::default(),
            indices: Default::default(),
            byte_strs: Default::default(),
            hasher: Default::default(),
        };

        Self(parking_lot::Mutex::new(inner))
    }

    fn intern_str(&self, str: &str) -> Symbol {
        Symbol::new(self.intern_inner(str.as_bytes()))
    }

    fn get_str(&self, symbol: Symbol) -> &str {
        let byte_str = self.get_inner(symbol.0.as_usize());
        // SAFETY: known to be a UTF8 string because it's a `Symbol`.
        unsafe { str::from_utf8_unchecked(byte_str) }
    }

    fn get_inner(&self, index: usize) -> &[u8] {
        let inner = self.0.lock();
        inner.byte_strs[index]
    }

    #[inline]
    fn intern_inner(&self, byte_str: &[u8]) -> u32 {
        let mut inner = self.0.lock();

        let InternerInner {
            arena,
            indices,
            byte_strs,
            hasher,
        } = &mut *inner;

        let hash_of_byte_str = hasher.hash_one(byte_str);

        match indices.entry(
            hash_of_byte_str,
            |&(s, _)| s == byte_str,
            |&(s, _)| hasher.hash_one(s),
        ) {
            Entry::Occupied(v) => v.get().1,

            Entry::Vacant(view) => {
                let byte_str: &[u8] = arena.alloc_slice(byte_str);

                // SAFETY: the arena outlives every stored slice and is not dropped
                // while these references are used.
                let byte_str: &'static [u8] = unsafe { &*(byte_str as *const [u8]) };

                let idx = byte_strs.len() as u32;

                view.insert((byte_str, idx));
                inner.byte_strs.push(byte_str);

                idx
            }
        }
    }
}
