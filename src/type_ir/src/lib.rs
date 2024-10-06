use std::{ cell::{ LazyCell, RefCell }, fmt::Debug, hash::Hash };

use bumpalo::Bump;
use data_structures::FxIndexSet;

thread_local! {
    static GLOBAL_SESSION: LazyCell<GlobalSession> = LazyCell::new(|| GlobalSession::new());
}

fn with_global_session<T>(f: impl Fn(&GlobalSession) -> T) -> T {
    GLOBAL_SESSION.with(|global_session: &LazyCell<GlobalSession>| f(global_session))
}

pub struct TyCtx;

impl TyCtx {
    fn common_tys(f: impl Fn(&CommonTys) -> Symbol<TyId>) -> Symbol<TyId> {
        with_global_session(|globals| f(&globals.common_tys))
    }
}

pub(crate) struct GlobalSession {
    arena: Bump,
    strings: RefCell<FxIndexSet<&'static str>>,
    ty_interner: TyInterner,
    common_tys: CommonTys,
}

impl GlobalSession {
    fn new() -> Self {
        let arena = Bump::default();
        let ty_interner = TyInterner::default();

        macro_rules! intern_ty {
            ($ty_kind:expr) => {
                ty_interner.new_symbol_ty(&arena, $ty_kind)
            };
        }

        let common_tys = CommonTys {
            bool_ty: intern_ty!(TyKind::Bool),
            int_ty: intern_ty!(TyKind::Int),
            void_ty: intern_ty!(TyKind::Void),
        };

        Self {
            arena,
            common_tys,
            ty_interner,
            strings: Default::default(),
        }
    }
}

struct CommonTys {
    pub int_ty: Symbol<TyId>,
    pub void_ty: Symbol<TyId>,
    pub bool_ty: Symbol<TyId>,
}

/// This struct contains and id for the string it represents.
/// It could store the reference here, but that would be an extra 4 bytes for each symbol
/// (and there are A LOT of symbols throughout the program)
#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct Symbol<T> where T: Hash + PartialEq + Eq + Debug + Clone + Copy + From<usize> {
    id: T,
}

impl<T> Symbol<T> where T: Hash + PartialEq + Eq + Debug + Clone + Copy + From<usize> {
    pub(crate) fn from_id(id: usize) -> Self {
        Self { id: T::from(id) }
    }
}

#[derive(Default)]
struct TyInterner {
    tys: RefCell<FxIndexSet<&'static TyKind>>,
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum TyKind {
    /// Any tuple e.g. (Int, Int, Bool)
    Tuple(&'static [Symbol<TyId>]),
    /// Not used directly in the code, but used by the compiler
    Ptr(Symbol<TyId>),
    /// Any Int (for now only i32)
    Int,
    /// Either `true` or `false`
    Bool,
    /// The Void type
    Void,
    /// The unkown type
    Unkown,
}

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct TyId(pub u32);

impl From<usize> for TyId {
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}

impl Symbol<TyId> {
    pub fn new(ty_kind: TyKind) -> Self {
        with_global_session(|globals| globals.ty_interner.new_symbol_ty(&globals.arena, ty_kind))
    }

    pub fn get(&self) -> &'static TyKind {
        with_global_session(|globals| globals.ty_interner.get_symbol_ty(self))
    }
}

impl TyInterner {
    pub(crate) fn new_symbol_ty(&self, arena: &Bump, ty_kind: TyKind) -> Symbol<TyId> {
        if let Some(id) = self.tys.borrow().get_index_of(&ty_kind) {
            return Symbol::from_id(id);
        }

        let ty_kind = arena.alloc(ty_kind);
        // Safe because the symbol interner lives as long as the program
        let ty_kind: &'static TyKind = unsafe { &*(ty_kind as *const TyKind) };

        let (idx, _) = self.tys.borrow_mut().insert_full(ty_kind);

        Symbol::from_id(idx)
    }

    pub(crate) fn get_symbol_ty(&self, symbol: &Symbol<TyId>) -> &'static TyKind {
        *self.tys
            .borrow()
            .get_index(symbol.id.0 as usize)
            .expect("Expected type")
    }
}

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct StrId(pub u32);

impl From<usize> for StrId {
    fn from(value: usize) -> Self {
        Self(value as u32)
    }
}

impl Symbol<StrId> {
    pub fn new(str: &str) -> Self {
        with_global_session(|globals| globals.new_symbol_str(str))
    }

    pub fn get(&self) -> &'static str {
        with_global_session(|globals| globals.get_symbol_str(self))
    }
}

impl GlobalSession {
    pub(crate) fn new_symbol_str(&self, string: &str) -> Symbol<StrId> {
        if let Some(id) = self.strings.borrow().get_index_of(string) {
            return Symbol::from_id(id);
        }

        let string: &mut str = self.arena.alloc_str(string);
        // Safe because the symbol interner lives as long as the program
        let string: &'static str = unsafe { &*(string.as_ref() as *const str) };

        let (idx, _) = self.strings.borrow_mut().insert_full(string);
        Symbol::from_id(idx)
    }

    pub(crate) fn get_symbol_str(&self, symbol: &Symbol<StrId>) -> &'static str {
        *self.strings
            .borrow()
            .get_index(symbol.id.0 as usize)
            .expect("Expected string")
    }
}
