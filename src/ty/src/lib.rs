use std::{ cell::{ LazyCell, RefCell }, fmt::Display, hash::{ Hash, Hasher } };

use data_structures::FxIndexSet;
use fxhash::FxHashMap;
use ir_defs::NodeId;
use typed_arena::Arena;

thread_local! {
    static TYPE_ARENA: LazyCell<Arena<Ty>> = LazyCell::new(|| Arena::new());
}

fn with_type_arena<T>(f: impl FnOnce(&Arena<Ty>) -> T) -> T {
    TYPE_ARENA.with(move |type_arena: &LazyCell<Arena<Ty>>| f(type_arena))
}

pub struct TyCtx<'ctx> {
    interned_types: FxIndexSet<&'ctx Ty>,
    // May be needed when multithreading is implemented
    // global_ty_ctx: &'ctx GlobalCtx<'ctx>,
}

impl<'ctx> Default for TyCtx<'ctx> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'ctx> TyCtx<'ctx> {
    pub fn new() -> Self {
        Self {
            interned_types: Default::default(),
        }
    }

    pub fn intern_type(&mut self, ty: Ty) -> &'ctx Ty {
        if let Some(found_type) = self.interned_types.get(&ty) {
            return *found_type;
        }

        // This is safe, because the arena lives as long as the program
        let interned_type = with_type_arena(|type_arena| unsafe {
            &*(type_arena.alloc(ty) as *mut Ty)
        });
        self.interned_types.insert(interned_type);

        interned_type
    }
}

#[derive(Hash, Eq, PartialEq)]
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

#[derive(Hash, Eq, PartialEq)]
pub enum PrimTy {
    Int,
    Void,
}

impl Display for PrimTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int => write!(f, "Int"),
            Self::Void => write!(f, "Void"),
        }
    }
}
