use std::cell::{ LazyCell, RefCell };
use fxhash::FxBuildHasher;
use indexmap::IndexSet;
use typed_arena::Arena;

pub(crate) const GLOBALS: LazyCell<Globals> = LazyCell::new(|| Globals::new());

pub(crate) struct Globals {
    symbol_interner: SymbolInterner,
}

impl Globals {
    pub fn new() -> Self {
        println!("GLOBALS INITIALIZED");
        Self {
            symbol_interner: SymbolInterner::new(),
        }
    }
}

pub(crate) struct SymbolInterner {
    arena: Arena<Box<str>>,
    strings: RefCell<IndexSet<&'static str, FxBuildHasher>>,
}

impl SymbolInterner {
    pub(crate) fn new() -> Self {
        Self { arena: Arena::new(), strings: RefCell::new(IndexSet::default()) }
    }

    pub(crate) fn alloc_str(&self, string: &str) -> Symbol {
        if let Some(id) = self.strings.borrow().get_index_of(string) {
            return Symbol::from_id(id);
        }

        let string = self.arena.alloc(string.into());
        let string: &'static str = unsafe { &*(string.as_ref() as *const str) };
        let (idx, _) = self.strings.borrow_mut().insert_full(string);
        Symbol::from_id(idx)
    }

    pub(crate) fn get_str(&self, id: usize) -> &'static str {
        *self.strings.borrow().get_index(id).expect("Expected string")
    }
}

struct SymbolId(usize);

pub struct Symbol {
    id: SymbolId,
}

impl Symbol {
    pub fn new(str: &str) -> Self {
        GLOBALS.symbol_interner.alloc_str(str)
    }

    pub fn get(&self) -> &'static str {
        GLOBALS.symbol_interner.get_str(self.id.0)
    }

    pub(crate) fn from_id(id: usize) -> Self {
        Self { id: SymbolId(id) }
    }
}
