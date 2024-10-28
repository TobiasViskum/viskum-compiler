use std::fmt::Display;

use op::*;

use crate::{
    with_global_session,
    Adt,
    DefId,
    DefIdToNameBinding,
    FnSig,
    Mutability,
    NameBindingKind,
    ResolvedInformation,
    Symbol,
};

pub const VOID_TY: Ty = Ty::PrimTy(PrimTy::Void);
pub const INT_TY: Ty = Ty::PrimTy(PrimTy::Int);
pub const STR_TY: Ty = Ty::PrimTy(PrimTy::Str);
pub const BOOL_TY: Ty = Ty::PrimTy(PrimTy::Bool);
pub const NEVER_TY: Ty = Ty::Never;
pub const UNKOWN_TY: Ty = Ty::Unkown;
pub const NULL_TY: Ty = Ty::Null;

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

#[derive(Debug)]
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

enum Delimiter {
    Parentheses,
    CurlyBrackets,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum Ty {
    /// `(T, K, ...)`
    Tuple(&'static [Ty]),
    /// `fn(T, ...) -> K`
    FnSig(FnSig),
    /// Reference to a function definition
    FnDef(DefId),
    // Constructor for a struct or enum
    // E.g. Option.Some(...) or Point { ... }
    // `Option` and `Point` are the constructor types
    AtdConstructer(DefId),
    /// Reference to an algebraic data type definition
    Adt(DefId),
    /// Reference to a variable on the stack (meaning to use the value, it should be dereferenced)
    /// Only used internally by the compiler
    StackPtr(&'static Ty, Mutability),
    /// Used internally by the compiler `*Ty` or in C-mode
    Ptr(&'static Ty, Mutability),
    /// Used internally by the compiler `[*]Ty` or in C-mode
    ManyPtr(&'static Ty, Mutability),
    /// Compiler types e.g. `Int, Uint, Float, String, etc.`
    PrimTy(PrimTy),
    /// Type `null` can be coerced to any pointer type
    Null,
    /// All code after this point is unreachable
    Never,
    /// Zero sized type
    ZeroSized,
    /// `...` Only used when declaring C-functions
    VariadicArgs,
    /// If the resulting type of an operation is unkown (error)
    Unkown,
}

impl Ty {
    pub fn to_ptr_ty(&self) -> Ty {
        match self {
            Self::Unkown | Self::PrimTy(PrimTy::Void) => *self,
            _ => Self::Ptr(TyCtx::intern_type(*self), Mutability::Immutable),
        }
    }

    pub fn to_mut_ptr_ty(&self) -> Ty {
        match self {
            Self::Unkown | Self::PrimTy(PrimTy::Void) => *self,
            _ => Self::Ptr(TyCtx::intern_type(*self), Mutability::Mutable),
        }
    }

    pub fn deref_if_stack_ptr(&self) -> Ty {
        match *self {
            Self::StackPtr(inner_ty, _) => *inner_ty,
            _ => *self,
        }
    }

    pub fn is_variadic_args(&self) -> bool {
        *self == Self::VariadicArgs
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
        if let Self::Ptr(_, _) | Self::ManyPtr(_, _) | Self::StackPtr(_, _) = self {
            true
        } else {
            false
        }
    }

    pub fn is_mut_ptr(&self) -> bool {
        if
            let
            | Self::Ptr(_, Mutability::Mutable)
            | Self::ManyPtr(_, Mutability::Mutable)
            | Self::StackPtr(_, Mutability::Mutable) = self
        {
            true
        } else if let Self::StackPtr(inner_ty, _) = self {
            match *inner_ty {
                Self::Ptr(_, Mutability::Mutable) | Self::ManyPtr(_, Mutability::Mutable) => true,
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn is_null(&self) -> bool {
        self.auto_deref() == NULL_TY
    }

    pub fn test_eq_strict<'a>(
        &self,
        other: Ty,
        def_id_to_name_binding: &DefIdToNameBinding<'a>
    ) -> bool {
        match (*self, other) {
            (Self::StackPtr(inner_ty1, mutability1), Self::Ptr(inner_ty2, mutability2)) => {
                inner_ty1.test_eq_strict(*inner_ty2, def_id_to_name_binding) &&
                    (mutability1 as u8) >= (mutability2 as u8)
            }
            (Self::Ptr(inner_ty1, mutability1), Self::StackPtr(inner_ty2, mutability2)) => {
                inner_ty1.test_eq_strict(*inner_ty2, def_id_to_name_binding) &&
                    (mutability1 as u8) >= (mutability2 as u8)
            }
            (Self::StackPtr(inner_ty1, mutability1), Self::ManyPtr(inner_ty2, mutability2)) => {
                inner_ty1.test_eq_strict(*inner_ty2, def_id_to_name_binding) &&
                    (mutability1 as u8) >= (mutability2 as u8)
            }
            (Self::ManyPtr(inner_ty1, mutability1), Self::StackPtr(inner_ty2, mutability2)) => {
                inner_ty1.test_eq_strict(*inner_ty2, def_id_to_name_binding) &&
                    (mutability1 as u8) >= (mutability2 as u8)
            }
            _ => *self == other,
        }
    }

    pub fn get_expanded_dereffed_ty<'a>(
        &self,
        def_id_to_name_binding: &DefIdToNameBinding<'a>
    ) -> Ty {
        self.auto_deref().get_expanded_ty(def_id_to_name_binding)
    }

    pub fn deref_until_stack_ptr_and_one_more_if_ptr(&self) -> Ty {
        let ty = self.deref_until_stack_ptr();
        if let Ty::StackPtr(inner_ty, _) = ty {
            if let Ty::Ptr(_, _) | Ty::ManyPtr(_, _) = *inner_ty { *inner_ty } else { ty }
        } else {
            ty
        }
    }

    pub fn deref_until_stack_ptr(&self) -> Ty {
        let mut ty = *self;

        loop {
            if let Ty::StackPtr(_, _) = ty {
                break ty;
            } else if let Ty::Ptr(inner_ty, _) | Ty::ManyPtr(inner_ty, _) = ty {
                ty = *inner_ty;
            } else {
                break ty;
            }
        }
    }

    fn get_expanded_ty<'a>(&self, def_id_to_name_binding: &DefIdToNameBinding<'a>) -> Ty {
        match self {
            Self::FnDef(def_id) => {
                let name_binding = def_id_to_name_binding.get(&def_id).unwrap();
                match name_binding.kind {
                    NameBindingKind::Fn(fn_sig, _) => Ty::FnSig(fn_sig),
                    _ => panic!("Expected fn"),
                }
            }
            Self::Adt(def_id) => {
                let name_binding = def_id_to_name_binding.get(&def_id).unwrap();
                match name_binding.kind {
                    NameBindingKind::Adt(Adt::Typedef(ty)) =>
                        ty.get_expanded_ty(def_id_to_name_binding),
                    _ => Ty::Adt(*def_id),
                }
            }

            _ => *self,
        }
    }

    pub fn test_binary(
        &self,
        other: Ty,
        op: BinaryOp,
        def_id_to_name_binding: &DefIdToNameBinding
    ) -> Option<Ty> {
        let lhs = self.get_expanded_dereffed_ty(def_id_to_name_binding);
        let rhs = other.get_expanded_dereffed_ty(def_id_to_name_binding);

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

    pub fn try_deref_as_adt<'a>(
        &self,
        def_id_to_name_binding: &DefIdToNameBinding<'a>
    ) -> Option<(DefId, Adt<'a>)> {
        let ty = self.auto_deref();
        match ty {
            Ty::Adt(def_id) => {
                let name_binding = def_id_to_name_binding.get(&def_id).unwrap();
                match name_binding.kind {
                    NameBindingKind::Adt(adt) => Some((def_id, adt)),
                    _ => None,
                }
            }
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

    pub fn auto_deref(&self) -> Ty {
        let mut ty = *self;
        loop {
            if let Ty::Ptr(inner_ty, _) | Ty::ManyPtr(inner_ty, _) | Ty::StackPtr(inner_ty, _) = ty {
                ty = *inner_ty;
            } else {
                break ty;
            }
        }
    }

    pub fn try_deref_once(&self) -> Option<Ty> {
        if let Ty::Ptr(inner_ty, _) | Ty::ManyPtr(inner_ty, _) | Ty::StackPtr(inner_ty, _) = *self {
            Some(*inner_ty)
        } else {
            None
        }
    }
}

impl GetTyAttr for Ty {
    fn get_ty_attr(&self, resolved_information: &ResolvedInformation) -> TyAttr {
        match self {
            Self::AtdConstructer(_) => todo!("Constructer function"),
            Self::VariadicArgs => panic!("`...` should not be used in this context"),
            Self::ZeroSized => TyAttr::new(0, 0),
            Self::FnDef(_) => TyAttr::new(8, 8),
            Self::FnSig(_) => TyAttr::new(8, 8),
            Self::Null => TyAttr::new(8, 8),
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
                            Adt::EnumVariant(parent_def_id, _, tys) => {
                                // This is the base size based on only the data inside the enum variant
                                let mut total_size = 0;
                                let mut alignment = None;

                                for ty in tys.iter() {
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
                            Adt::Enum(variants) => {
                                let mut largest_variant_size = 0;

                                for def_id in variants {
                                    let variant_ty_attr = Ty::Adt(*def_id).get_ty_attr(
                                        resolved_information
                                    );

                                    if variant_ty_attr.size_bytes > largest_variant_size {
                                        largest_variant_size = variant_ty_attr.size_bytes;
                                    }
                                }

                                // The size of the enum is the size of the largest variant + 8 bytes for the discriminant
                                //
                                // In the future we want to fit the size of the discriminant to the nearst byte
                                // whilst still having enough space in the discriminant to fit all the variants
                                let total_size = largest_variant_size + 8;

                                TyAttr::new(total_size, 4)
                            }
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
            Self::Ptr(_, _) => TyAttr::new(8, 8),
            Self::ManyPtr(_, _) => TyAttr::new(8, 8),
            Self::StackPtr(_, _) => TyAttr::new(8, 8),
            Self::PrimTy(prim_ty) => prim_ty.get_ty_attr(resolved_information),
            t @ (Self::Unkown | Self::Never) => panic!("{} has no size and alignment", t),
        }
    }
}

impl Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::AtdConstructer(def_id) => write!(f, "{}", def_id.symbol.get()),
            Self::ZeroSized => write!(f, "ZeroSized"),
            Self::VariadicArgs => write!(f, "..."),
            Self::FnDef(def_id) => write!(f, "FnDef({})", def_id.symbol.get()),
            Self::FnSig(_) => write!(f, "FnSig"),
            Self::Ptr(inner, mutability) => { write!(f, "*{}{}", mutability, inner) }
            Self::ManyPtr(inner, mutability) => { write!(f, "[*{}]{}", mutability, inner) }
            Self::StackPtr(inner, mutability) => { write!(f, "stack_ptr<{}{}>", mutability, inner) }
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
    /// 32-bit signed integer `Int`
    Int,
    /// 64-bit signed integer `Int64`
    Int64,
    /// Constant string `Str` (actually a pointer to a constant string)
    Str,
    /// 8-bit boolean `Bool`
    Bool,
    /// Type `Void`
    Void,
}

impl GetTyAttr for PrimTy {
    fn get_ty_attr(&self, _: &ResolvedInformation) -> TyAttr {
        match self {
            Self::Int => TyAttr::new(4, 4),
            Self::Int64 => TyAttr::new(8, 8),
            Self::Bool => TyAttr::new(1, 1),
            Self::Void => TyAttr::new(0, 0),
            Self::Str => TyAttr::new(8, 8),
        }
    }
}

impl Display for PrimTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int => write!(f, "Int"),
            Self::Int64 => write!(f, "Int64"),
            Self::Bool => write!(f, "Bool"),
            Self::Void => write!(f, "Void"),
            Self::Str => write!(f, "Str"),
        }
    }
}
