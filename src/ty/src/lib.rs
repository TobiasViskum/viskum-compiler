use std::{ cell::{ LazyCell, RefCell }, fmt::Display, hash::Hash, marker::PhantomData };

use fxhash::FxHashSet;
use typed_arena::Arena;

/// We can use a bump arena here, since we don't care if the drop implementations are called
///
/// This is because this object is destroyed once the main thread ends
struct GlobalSession {
    type_arena: Arena<Ty>,
    interned_types: RefCell<FxHashSet<&'static Ty>>,
    str_arena: Arena<Box<str>>,
    interned_strings: RefCell<FxHashSet<&'static str>>,
}

thread_local! {
    static TYPE_ARENA: LazyCell<Arena<Ty>> = LazyCell::new(|| Arena::new());
    static GLOBAL_CTX: LazyCell<GlobalCtx> = LazyCell::new(|| GlobalCtx::default());
}

fn with_type_arena<T>(f: impl FnOnce(&Arena<Ty>) -> T) -> T {
    TYPE_ARENA.with(|type_arena: &LazyCell<Arena<Ty>>| f(type_arena))
}

fn with_global_ctx<T>(f: impl FnOnce(&GlobalCtx) -> T) -> T {
    GLOBAL_CTX.with(move |global_ctx: &LazyCell<GlobalCtx>| f(global_ctx))
}

struct GlobalCtx {
    interned_types: RefCell<FxHashSet<&'static Ty>>,
}

impl Default for GlobalCtx {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalCtx {
    pub fn new() -> Self {
        Self {
            interned_types: Default::default(),
        }
    }

    pub fn intern_type(&self, ty: Ty) -> &'static Ty {
        if let Some(found_type) = self.interned_types.borrow().get(&ty) {
            return *found_type;
        }

        // This is safe, because the arena lives as long as the program
        let interned_type = with_type_arena(|type_arena| unsafe {
            &*(type_arena.alloc(ty) as *mut Ty)
        });
        self.interned_types.borrow_mut().insert(interned_type);

        interned_type
    }
}

/// For now, this is just used as a way to intern types
pub struct TyCtx;

impl Default for TyCtx {
    fn default() -> Self {
        Self
    }
}

impl TyCtx {
    pub fn intern_type(&mut self, ty: Ty) -> &'static Ty {
        with_global_ctx(|global| global.intern_type(ty))
    }

    // fn _intern_type(&mut self, ty: Ty) -> &'ctx Ty {
    //     if let Some(found_type) = self.interned_types.get(&ty) {
    //         return *found_type;
    //     }

    //     // This is safe, because the arena lives as long as the program
    //     let interned_type = with_type_arena(|type_arena| unsafe {
    //         &*(type_arena.alloc(ty) as *mut Ty)
    //     });
    //     self.interned_types.insert(interned_type);

    //     interned_type
    // }
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum Ty {
    PrimTy(PrimTy),
    Unkown,
}

impl Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unkown => write!(f, "{{unkown}}"),
            Self::PrimTy(prim_ty) => prim_ty.fmt(f),
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum PrimTy {
    Int,
    Bool,
    Void,
}

impl Display for PrimTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int => write!(f, "Int"),
            Self::Bool => write!(f, "Bool"),
            Self::Void => write!(f, "Void"),
        }
    }
}
