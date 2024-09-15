use std::cell::{ LazyCell, RefCell };
use data_structures::FxIndexSet;
use typed_arena::Arena;

thread_local! {
    static GLOBALS: LazyCell<Globals> = LazyCell::new(|| Globals::new());
}

fn with_globals<T>(f: impl Fn(&Globals) -> T) -> T {
    GLOBALS.with(|globals: &LazyCell<Globals>| f(globals))
}

pub(crate) struct Globals {
    symbol_interner: SymbolInterner,
}

impl Globals {
    pub fn new() -> Self {
        Self {
            symbol_interner: SymbolInterner::new(),
        }
    }
}

pub(crate) struct SymbolInterner {
    arena: Arena<Box<str>>,
    strings: RefCell<FxIndexSet<&'static str>>,
}

impl SymbolInterner {
    pub(crate) fn new() -> Self {
        Self { arena: Arena::new(), strings: RefCell::new(Default::default()) }
    }

    pub(crate) fn alloc_str(&self, string: &str) -> Symbol {
        if let Some(id) = self.strings.borrow().get_index_of(string) {
            return Symbol::from_id(id);
        }

        let string = self.arena.alloc(string.into());
        // Safe because the symbol interner lives as long as the program
        let string: &'static str = unsafe { &*(string.as_ref() as *const str) };

        let (idx, _) = self.strings.borrow_mut().insert_full(string);
        Symbol::from_id(idx)
    }

    pub(crate) fn get_str(&self, id: usize) -> &'static str {
        *self.strings.borrow().get_index(id).expect("Expected string")
    }
}

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
struct SymbolId(pub u32);

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct Symbol {
    id: SymbolId,
}

impl Symbol {
    pub fn new(str: &str) -> Self {
        with_globals(|globals| globals.symbol_interner.alloc_str(str))
    }

    pub fn get(&self) -> &'static str {
        with_globals(|globals| globals.symbol_interner.get_str(self.id.0 as usize))
    }

    pub(crate) fn from_id(id: usize) -> Self {
        Self { id: SymbolId(id as u32) }
    }
}
