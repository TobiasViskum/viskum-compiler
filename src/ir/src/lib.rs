mod ty;
mod symbol;
mod ir_defs;

use std::cell::{ LazyCell, RefCell };

use bumpalo::Bump;
use data_structures::FxIndexSet;

pub use ty::*;
pub use symbol::*;
pub use ir_defs::*;

thread_local! {
    static GLOBAL_SESSION: LazyCell<GlobalSession> = LazyCell::new(|| Default::default());
}

pub(crate) fn with_global_session<T>(f: impl FnOnce(&GlobalSession) -> T) -> T {
    GLOBAL_SESSION.with(|global_session: &LazyCell<GlobalSession>| f(global_session))
}

// We can use a bump arena here, since we don't care if the drop implementations are called
///
/// This is because this object is destroyed once the main thread ends
#[derive(Default)]
struct GlobalSession {
    arena: Bump,
    interned_types: RefCell<FxIndexSet<&'static Ty>>,
    interned_strings: RefCell<FxIndexSet<&'static str>>,
}

impl GlobalSession {
    pub(crate) fn intern_vec_of_types<T>(&self, types: Vec<T>) -> &'static [T] {
        let interned_ty = with_global_session(|session| unsafe {
            &*(session.arena.alloc_slice_fill_iter(types.into_iter()) as *mut [T])
        });

        interned_ty
    }

    pub(crate) fn intern_type(&self, ty: Ty) -> &'static Ty {
        if let Some(found_type) = self.interned_types.borrow().get(&ty) {
            return found_type;
        }

        // This is safe, because the arena lives as long as the program
        let interned_type = with_global_session(|session| unsafe {
            &*(session.arena.alloc(ty) as *mut Ty)
        });
        self.interned_types.borrow_mut().insert(interned_type);

        interned_type
    }

    pub(crate) fn intern_str(&self, string: &str) -> Symbol {
        if let Some(id) = self.interned_strings.borrow().get_index_of(string) {
            return Symbol::from_id(id);
        }

        let string: &mut str = self.arena.alloc_str(string);
        // Safe because the symbol interner lives as long as the program
        let string: &'static str = unsafe { &*(string.as_ref() as *const str) };

        let (idx, _) = self.interned_strings.borrow_mut().insert_full(string);
        Symbol::from_id(idx)
    }

    pub(crate) fn get_str(&self, symbol: &Symbol) -> &'static str {
        *self.interned_strings
            .borrow()
            .get_index(symbol.id.0 as usize)
            .expect("Expected string")
    }
}
