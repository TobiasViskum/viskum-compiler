use std::{ cell::{ LazyCell, RefCell }, fmt::Display, hash::Hash, marker::PhantomData };

use bumpalo::Bump;
use fxhash::FxHashSet;
use op::{ ArithmeticOp, BinaryOp, ComparisonOp };
use symbol::Symbol;
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

    fn intern_vec_of_types<T>(&self, types: Vec<T>) -> &'static [T] {
        let interned_ty = with_type_arena(|type_arena: &Bump| unsafe {
            &*(type_arena.alloc_slice_fill_iter(types.into_iter()) as *mut [T])
        });

        interned_ty
    }

    fn intern_type(&self, ty: Ty) -> &'static Ty {
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

    pub fn intern_many_types<T>(types: Vec<T>) -> &'static [T] {
        with_global_ctx(|global| global.intern_vec_of_types(types))
    }
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
    fn get_ty_attr(&self) -> TyAttr;
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum Ty {
    Tuple(&'static [Ty]),
    Struct(Symbol, &'static [(Symbol, Ty)]),
    Ptr(&'static Ty),
    PrimTy(PrimTy),
    Unkown,
}

impl Ty {
    pub fn to_ptr_ty(&self) -> Ty {
        match self {
            Self::Unkown | Self::PrimTy(PrimTy::Void) => *self,
            _ => Self::Ptr(TyCtx.intern_type(*self)),
        }
    }

    pub fn is_void(&self) -> bool {
        self.auto_deref() == VOID_TY
    }

    pub fn is_ptr(&self) -> bool {
        if let Self::Ptr(_) = self { true } else { false }
    }

    pub fn test_eq(&self, other: Ty) -> bool {
        self.auto_deref() == other.auto_deref()
    }

    pub fn test_binary(&self, other: Ty, op: BinaryOp) -> Option<Ty> {
        let lhs = self.auto_deref();
        let rhs = other.auto_deref();

        match op {
            BinaryOp::ArithmeticOp(arithmetic_op) => {
                use ArithmeticOp::*;
                match (lhs, arithmetic_op, rhs) {
                    (INT_TY, Add | Sub | Mul | Div, INT_TY) => Some(INT_TY),
                    _ => None,
                }
            }
            BinaryOp::ComparisonOp(comparison_op) => {
                use ComparisonOp::*;
                match (lhs, comparison_op, rhs) {
                    (INT_TY, Eq | Ne | Ge | Gt | Le | Lt, INT_TY) => Some(BOOL_TY),
                    (BOOL_TY, Eq | Ne | Ge | Gt | Le | Lt, BOOL_TY) => Some(BOOL_TY),
                    _ => None,
                }
            }
        }
    }

    pub fn try_deref_as_tuple(&self) -> Option<&'static [Ty]> {
        let ty = self.auto_deref();
        match ty {
            Ty::Tuple(tuple_ty) => Some(tuple_ty),
            _ => None,
        }
    }

    pub fn try_deref_as_struct(&self) -> Option<(Symbol, &'static [(Symbol, Ty)])> {
        let ty = self.auto_deref();
        match ty {
            Ty::Struct(struct_name, fields) => Some((struct_name, fields)),
            _ => None,
        }
    }

    pub fn can_be_dereffed_to_bool(&self) -> bool {
        self.auto_deref() == BOOL_TY
    }

    fn auto_deref(&self) -> Ty {
        let mut ty = *self;
        loop {
            if let Ty::Ptr(inner_ty) = ty {
                ty = *inner_ty;
            } else {
                break ty;
            }
        }
    }

    pub fn try_deref_once(&self) -> Option<Ty> {
        if let Ty::Ptr(inner_ty) = *self { Some(*inner_ty) } else { None }
    }
}

impl GetTyAttr for Ty {
    fn get_ty_attr(&self) -> TyAttr {
        match self {
            Self::Tuple(tuple) => {
                let mut total_size = 0;
                let mut alignment = None;

                for ty in tuple.iter() {
                    let ty_attr = ty.get_ty_attr();
                    total_size += ty_attr.size_bytes;
                    if let Some(alignment) = &mut alignment {
                        if ty_attr.alignment_bytes < *alignment {
                            *alignment = ty_attr.alignment_bytes;
                        }
                    } else {
                        alignment = Some(ty_attr.alignment_bytes);
                    }
                }

                TyAttr::new(total_size, alignment.unwrap_or(0))
            }
            Self::Struct(_, fields) => {
                let mut total_size = 0;
                let mut alignment = None;

                for (_, ty) in fields.iter() {
                    let ty_attr = ty.get_ty_attr();
                    total_size += ty_attr.size_bytes;
                    if let Some(alignment) = &mut alignment {
                        if ty_attr.alignment_bytes < *alignment {
                            *alignment = ty_attr.alignment_bytes;
                        }
                    } else {
                        alignment = Some(ty_attr.alignment_bytes);
                    }
                }

                TyAttr::new(total_size, alignment.unwrap_or(0))
            }
            Self::Ptr(_) => TyAttr::new(8, 8),
            Self::PrimTy(prim_ty) => prim_ty.get_ty_attr(),
            Self::Unkown => panic!("Unkown has no size and alignment"),
        }
    }
}

impl Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ptr(inner) => write!(f, "*{}", inner),
            Self::Unkown => write!(f, "{{unkown}}"),
            Self::PrimTy(prim_ty) => prim_ty.fmt(f),
            Self::Struct(struct_name, fields) => {
                write!(f, "{} {{", struct_name.get())?;
                for (i, (symbol, ty)) in fields.iter().enumerate() {
                    let len = fields.len();
                    write!(f, " {}: {}", symbol.get(), ty)?;
                    if i != len - 1 {
                        write!(f, ",")?;
                    } else {
                        write!(f, " ")?;
                    }
                }

                write!(f, "}}")
            }
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
    fn get_ty_attr(&self) -> TyAttr {
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
