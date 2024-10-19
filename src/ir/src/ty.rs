use std::fmt::Display;

use op::*;

use crate::{
    with_global_session,
    Adt,
    DefId,
    DefIdToNameBinding,
    FnSig,
    NameBindingKind,
    ResolvedInformation,
    Symbol,
};

pub const VOID_TY: Ty = Ty::PrimTy(PrimTy::Void);
pub const INT_TY: Ty = Ty::PrimTy(PrimTy::Int);
pub const BOOL_TY: Ty = Ty::PrimTy(PrimTy::Bool);
pub const NEVER_TY: Ty = Ty::Never;
pub const UNKOWN_TY: Ty = Ty::Unkown;

/// For now, this is just used as a way to intern types
pub struct TyCtx;
impl Default for TyCtx {
    fn default() -> Self {
        Self
    }
}

impl TyCtx {
    pub fn intern_type(ty: Ty) -> &'static Ty {
        with_global_session(|session| session.intern_type(ty))
    }

    pub fn intern_many_types<T>(types: Vec<T>) -> &'static [T] {
        with_global_session(|session| session.intern_vec_of_types(types))
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
    fn get_ty_attr(&self, resolved_information: &ResolvedInformation) -> TyAttr;
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum Ty {
    /// `(T, K, ...)`
    Tuple(&'static [Ty]),
    /// `fn(T, ...) -> K`
    FnSig(FnSig),
    /// Reference to a function definition
    FnDef(DefId),
    /// Reference to an algebraic data type definition
    Adt(DefId),
    /// Used internally by the compiler `*Ty`
    Ptr(&'static Ty),
    /// Compiler types e.g. `Int, Uint, Float, String, etc.`
    PrimTy(PrimTy),
    /// All code after this point is unreachable
    Never,
    /// If the resulting type of an operation is unkown (error)
    Unkown,
}

impl Ty {
    pub fn to_ptr_ty(&self) -> Ty {
        match self {
            Self::Unkown | Self::PrimTy(PrimTy::Void) => *self,
            _ => Self::Ptr(TyCtx::intern_type(*self)),
        }
    }

    pub fn is_unkown(&self) -> bool {
        self.auto_deref() == Ty::Unkown
    }

    pub fn is_void(&self) -> bool {
        self.auto_deref() == VOID_TY
    }

    pub fn is_never(&self) -> bool {
        self.auto_deref() == NEVER_TY
    }

    pub fn is_ptr(&self) -> bool {
        if let Self::Ptr(_) = self { true } else { false }
    }

    pub fn test_eq<'a>(&self, other: Ty, def_id_to_name_binding: &DefIdToNameBinding<'a>) -> bool {
        let lhs = self.get_expanded_ty(def_id_to_name_binding);
        let rhs = other.get_expanded_ty(def_id_to_name_binding);

        lhs == rhs
    }

    fn get_expanded_ty<'a>(&self, def_id_to_name_binding: &DefIdToNameBinding<'a>) -> Ty {
        match self {
            Self::Adt(def_id) => {
                let name_binding = def_id_to_name_binding.get(&def_id).unwrap();
                match name_binding.kind {
                    NameBindingKind::Adt(adt) => {
                        match adt {
                            Adt::Struct(fields) => {
                                let mut expanded_fields = Vec::with_capacity(fields.len());
                                for (_, ty) in fields.iter() {
                                    expanded_fields.push(
                                        ty.get_expanded_ty(def_id_to_name_binding)
                                    );
                                }

                                Ty::Tuple(TyCtx::intern_many_types(expanded_fields))
                            }
                            Adt::Typedef(ty) => ty.get_expanded_ty(def_id_to_name_binding),
                        }
                    }
                    _ => panic!("Invalid ADT"),
                }
            }
            Self::FnDef(def_id) => {
                let name_binding = def_id_to_name_binding.get(&def_id).unwrap();
                match name_binding.kind {
                    NameBindingKind::Fn(fn_sig) => Ty::FnSig(fn_sig),
                    _ => panic!("Expected fn"),
                }
            }
            _ => self.auto_deref(),
        }
    }

    pub fn test_binary(
        &self,
        other: Ty,
        op: BinaryOp,
        def_id_to_name_binding: &DefIdToNameBinding
    ) -> Option<Ty> {
        let lhs = self.auto_deref().get_expanded_ty(def_id_to_name_binding);
        let rhs = other.auto_deref().get_expanded_ty(def_id_to_name_binding);

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

    pub fn try_deref_as_tuple(
        &self,
        def_id_to_name_binding: &DefIdToNameBinding
    ) -> Option<&'static [Ty]> {
        let ty = self.auto_deref().get_expanded_ty(def_id_to_name_binding);
        match ty {
            Ty::Tuple(tuple_ty) => Some(tuple_ty),
            _ => None,
        }
    }

    pub fn try_deref_as_struct<'a>(
        &self,
        def_id_to_name_binding: &DefIdToNameBinding<'a>
    ) -> Option<(Symbol, &'a [(DefId, Ty)])> {
        let ty = self.auto_deref();
        match ty {
            Ty::Adt(def_id) => {
                let name_binding = def_id_to_name_binding.get(&def_id).unwrap();
                match name_binding.kind {
                    NameBindingKind::Adt(Adt::Struct(fields)) => Some((def_id.symbol, fields)),
                    _ => None,
                }
            }
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
    fn get_ty_attr(&self, resolved_information: &ResolvedInformation) -> TyAttr {
        match self {
            Self::FnDef(_) => TyAttr::new(8, 8),
            Self::FnSig(_) => TyAttr::new(8, 8),
            Self::Tuple(tuple) => {
                let mut total_size = 0;
                let mut alignment = None;

                for ty in tuple.iter() {
                    let ty_attr = ty.get_ty_attr(resolved_information);
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
            Self::Adt(def_id) => {
                let name_binding = resolved_information.get_name_binding_from_def_id(def_id);

                let ty_attr = match name_binding.kind {
                    NameBindingKind::Adt(adt) => {
                        match adt {
                            Adt::Typedef(ty) => ty.get_ty_attr(resolved_information),
                            Adt::Struct(fields) => {
                                let mut total_size = 0;
                                let mut alignment = None;

                                for (_, ty) in fields.iter() {
                                    let ty_attr = ty.get_ty_attr(resolved_information);
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
                        }
                    }

                    _ => panic!("Invalid ADT"),
                };

                ty_attr
            }
            Self::Ptr(_) => TyAttr::new(8, 8),
            Self::PrimTy(prim_ty) => prim_ty.get_ty_attr(resolved_information),
            t @ (Self::Unkown | Self::Never) => panic!("{} has no size and alignment", t),
        }
    }
}

impl Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FnDef(def_id) => write!(f, "FnDef({})", def_id.symbol.get()),
            Self::FnSig(_) => write!(f, "FnSig"),
            Self::Ptr(inner) => write!(f, "*{}", inner),
            Self::Unkown => write!(f, "{{unkown}}"),
            Self::Never => write!(f, "!"),
            Self::PrimTy(prim_ty) => prim_ty.fmt(f),
            Self::Adt(def_id) => {
                write!(f, "{} {{", def_id.symbol.get())?;
                // for (i, (symbol, ty)) in fields.iter().enumerate() {
                //     let len = fields.len();
                //     write!(f, " {}: {}", symbol.get(), ty)?;
                //     if i != len - 1 {
                //         write!(f, ",")?;
                //     } else {
                //         write!(f, " ")?;
                //     }
                // }

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
    fn get_ty_attr(&self, _: &ResolvedInformation) -> TyAttr {
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
