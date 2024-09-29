use std::{ cell::{ LazyCell, RefCell }, fmt::Display, hash::Hash, marker::PhantomData };

use bumpalo::Bump;
use fxhash::FxHashSet;
use typed_arena::Arena;

pub const VOID_TY: Ty = Ty::PrimTy(PrimTy::Void);
pub const INT_TY: Ty = Ty::PrimTy(PrimTy::Int);
pub const BOOL_TY: Ty = Ty::PrimTy(PrimTy::Bool);

/// We can use a bump arena here, since we don't care if the drop implementations are called
///
/// This is because this object is destroyed once the main thread ends
struct GlobalSession {
    type_arena: Bump,
    interned_types: RefCell<FxHashSet<&'static Ty>>,
    str_arena: Arena<Box<str>>,
    interned_strings: RefCell<FxHashSet<&'static str>>,
}

thread_local! {
    static TYPE_ARENA: LazyCell<Bump> = LazyCell::new(|| Bump::new());
    static GLOBAL_CTX: LazyCell<GlobalCtx> = LazyCell::new(|| GlobalCtx::default());
}

fn with_type_arena<T>(f: impl FnOnce(&Bump) -> T) -> T {
    TYPE_ARENA.with(|type_arena: &LazyCell<Bump>| f(type_arena))
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

    pub fn intern_vec_of_types(&self, types: Vec<Ty>) -> &'static [Ty] {
        let interned_ty = with_type_arena(|type_arena| unsafe {
            &*(type_arena.alloc_slice_fill_iter(types.into_iter()) as *mut [Ty])
        });

        interned_ty
    }

    pub fn intern_type(&self, ty: Ty) -> &'static Ty {
        if let Some(found_type) = self.interned_types.borrow().get(&ty) {
            return found_type;
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

    pub fn tuple_type(fields: Vec<Ty>) -> Ty {
        let fields = with_global_ctx(|global| global.intern_vec_of_types(fields));

        Ty::Tuple(fields)
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

pub struct TyAttr {
    pub size_bytes: usize,
    pub alignment_bytes: usize,
}

impl TyAttr {
    pub fn new(size_bytes: usize, alignment_bytes: usize) -> Self {
        Self { size_bytes, alignment_bytes }
    }
}

pub trait GetTyAttr {
    fn get_size_and_alignment(&self) -> TyAttr;
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum Ty {
    Tuple(&'static [Ty]),
    PrimTy(PrimTy),
    Unkown,
}

impl Ty {
    pub fn is_void(&self) -> bool {
        *self == Ty::PrimTy(PrimTy::Void)
    }
}

impl GetTyAttr for Ty {
    fn get_size_and_alignment(&self) -> TyAttr {
        match self {
            Self::Tuple(tuple) => {
                let mut total_size = 0;
                let mut alignment = None;

                for ty in tuple.iter() {
                    let ty_attr = ty.get_size_and_alignment();
                    total_size += ty_attr.size_bytes;
                    if let Some(alignment) = &mut alignment {
                        if ty_attr.size_bytes < *alignment {
                            *alignment = ty_attr.size_bytes;
                        }
                    } else {
                        alignment = Some(ty_attr.size_bytes);
                    }
                }

                TyAttr::new(total_size, alignment.unwrap_or(0))
            }
            Self::PrimTy(prim_ty) => prim_ty.get_size_and_alignment(),
            Self::Unkown => panic!("Unkown has no size and alignment"),
        }
    }
}

impl Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unkown => write!(f, "{{unkown}}"),
            Self::PrimTy(prim_ty) => prim_ty.fmt(f),
            Self::Tuple(types) => {
                write!(f, "(")?;
                for (i, ty) in types.iter().enumerate() {
                    let len = types.len();
                    write!(f, "{}", ty)?;
                    if i != len - 1 {
                        write!(f, ", ")?;
                    }
                }

                write!(f, ")")
            }
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum PrimTy {
    Int,
    Bool,
    Void,
}

impl GetTyAttr for PrimTy {
    fn get_size_and_alignment(&self) -> TyAttr {
        match self {
            Self::Int => TyAttr::new(4, 4),
            Self::Bool => TyAttr::new(1, 1),
            Self::Void => TyAttr::new(0, 0),
        }
    }
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
