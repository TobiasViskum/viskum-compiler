mod ty;
mod symbol;
mod ir_defs;

use std::sync::{ LazyLock, Mutex, RwLock };

use bumpalo::Bump;
use data_structures::FxIndexSet;

use fxhash::FxHashMap;
use token::TokenKind;
pub use ty::*;
pub use symbol::*;
pub use ir_defs::*;

static GLOBAL_SESSION: LazyLock<GlobalSession> = LazyLock::new(|| GlobalSession::new());

pub(crate) fn with_global_session<T>(f: impl FnOnce(&GlobalSession) -> T) -> T {
    f(&GLOBAL_SESSION)
}

// We can use a bump arena here, since we don't care if the drop implementations are called
///
/// This is because this object is destroyed once the main thread ends
struct GlobalSession {
    interned_types: Mutex<FxIndexSet<&'static Ty>>,
    /// We own them because we want to mutate them later
    // real_interned_types: FxIndexSet<Ty>,
    /// For the string interner
    arena: Mutex<Bump>,
    interned_strings: RwLock<FxIndexSet<&'static str>>,
    node_id_to_symbol: RwLock<FxHashMap<NodeId, Symbol>>,
}

impl GlobalSession {
    pub(crate) fn new() -> Self {
        let global_session = Self {
            interned_types: Mutex::new(FxIndexSet::default()),
            arena: Mutex::new(Bump::new()),
            interned_strings: RwLock::new(FxIndexSet::default()),
            node_id_to_symbol: RwLock::new(FxHashMap::default()),
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
            &*(self.arena.lock().unwrap().alloc_slice_fill_iter(types) as *const [T])
        };

        interned_ty
    }

    pub(crate) fn intern_type(&self, ty: Ty) -> &'static Ty {
        let mut lock = self.interned_types.lock().unwrap();

        if let Some(found_type) = lock.get(&ty) {
            return found_type;
        }

        // This is safe, because the arena lives as long as the program
        let interned_type = unsafe { &mut *(self.arena.lock().unwrap().alloc(ty) as *mut Ty) };
        lock.insert(interned_type);

        interned_type
    }

    pub(crate) fn insert_symbol_to_node_id(&self, node_id: NodeId, symbol: Symbol) {
        self.node_id_to_symbol.write().unwrap().insert(node_id, symbol);
    }

    pub(crate) fn get_symbol_from_node_id(&self, node_id: NodeId) -> Symbol {
        *self.node_id_to_symbol.read().unwrap().get(&node_id).expect("Expected symbol")
    }

    pub(crate) fn intern_str(&self, string: &str) -> Symbol {
        if let Some(id) = self.interned_strings.read().unwrap().get_index_of(string) {
            return Symbol::from_id(id);
        }

        let string = {
            let arena = self.arena.lock().unwrap();
            let string: &mut str = arena.alloc_str(string);
            let string: &'static str = unsafe { &*(string as *const str) };
            string
        };
        // Safe because the symbol interner lives as long as the program

        let (idx, _) = self.interned_strings.write().unwrap().insert_full(string);
        Symbol::from_id(idx)
    }

    pub(crate) fn get_str(&self, symbol: &Symbol) -> &'static str {
        self.interned_strings
            .read()
            .unwrap()
            .get_index(symbol.id.0 as usize)
            .expect("Expected string")
    }
}
