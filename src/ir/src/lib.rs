mod ty;
mod symbol;
mod ir_defs;

use std::sync::{ LazyLock, Mutex };

use bumpalo::Bump;
use data_structures::FxIndexSet;

use fxhash::FxHashMap;
use token::TokenKind;
pub use ty::*;
pub use symbol::*;
pub use ir_defs::*;

static GLOBAL_SESSION: LazyLock<Mutex<GlobalSession>> = LazyLock::new(||
    Mutex::new(GlobalSession::new())
);

pub(crate) fn with_global_session<T>(f: impl FnOnce(&mut GlobalSession) -> T) -> T {
    let mut session = GLOBAL_SESSION.lock().unwrap();
    f(&mut session)
}

// We can use a bump arena here, since we don't care if the drop implementations are called
///
/// This is because this object is destroyed once the main thread ends
struct GlobalSession {
    interned_types: FxIndexSet<&'static Ty>,
    /// We own them because we want to mutate them later
    // real_interned_types: FxIndexSet<Ty>,
    /// For the string interner
    arena: Bump,
    interned_strings: FxIndexSet<&'static str>,
    node_id_to_symbol: FxHashMap<NodeId, Symbol>,
}

impl GlobalSession {
    pub(crate) fn new() -> Self {
        let mut global_session = Self {
            interned_types: FxIndexSet::default(),
            arena: Bump::new(),
            interned_strings: FxIndexSet::default(),
            node_id_to_symbol: FxHashMap::default(),
        };

        for token in enum_iterator::all::<TokenKind>() {
            let keyword_str = token.to_keyword_str();

            if !keyword_str.is_empty() {
                global_session.intern_str(keyword_str);
            }
        }

        global_session
    }

    pub(crate) fn intern_vec_of_types<T>(&self, types: Vec<T>) -> &'static [T] {
        let interned_ty = unsafe {
            &*(self.arena.alloc_slice_fill_iter(types) as *const [T])
        };

        interned_ty
    }

    pub(crate) fn intern_type(&mut self, ty: Ty) -> &'static Ty {
        if let Some(found_type) = self.interned_types.get(&ty) {
            return found_type;
        }

        // This is safe, because the arena lives as long as the program
        let interned_type = unsafe { &mut *(self.arena.alloc(ty) as *mut Ty) };
        self.interned_types.insert(interned_type);

        interned_type
    }

    pub(crate) fn intern_str(&mut self, string: &str) -> Symbol {
        if let Some(id) = self.interned_strings.get_index_of(string) {
            return Symbol::from_id(id);
        }

        let string: &mut str = self.arena.alloc_str(string);
        // Safe because the symbol interner lives as long as the program
        let string: &'static str = unsafe { &*(string as *const str) };

        let (idx, _) = self.interned_strings.insert_full(string);
        Symbol::from_id(idx)
    }

    pub(crate) fn get_str(&self, symbol: &Symbol) -> &'static str {
        self.interned_strings.get_index(symbol.id.0 as usize).expect("Expected string")
    }
}
