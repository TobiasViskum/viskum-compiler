use std::fmt::Display;

use op::*;

use crate::{
    with_global_session,
    Adt,
    DefId,
    DefIdToNameBinding,
    FnSig,
    Mutability,
    NameBinding,
    NameBindingKind,
    ResolvedInformation,
    Symbol,
};

pub const INT_8_TY: Ty = Ty::PrimTy(PrimTy::Int(IntTy::Int8));
pub const INT_16_TY: Ty = Ty::PrimTy(PrimTy::Int(IntTy::Int16));
pub const INT_32_TY: Ty = Ty::PrimTy(PrimTy::Int(IntTy::Int32));
pub const INT_64_TY: Ty = Ty::PrimTy(PrimTy::Int(IntTy::Int64));

pub const UINT_8_TY: Ty = Ty::PrimTy(PrimTy::Uint(UintTy::Uint8));
pub const UINT_16_TY: Ty = Ty::PrimTy(PrimTy::Uint(UintTy::Uint16));
pub const UINT_32_TY: Ty = Ty::PrimTy(PrimTy::Uint(UintTy::Uint32));
pub const UINT_64_TY: Ty = Ty::PrimTy(PrimTy::Uint(UintTy::Uint64));

pub const FLOAT_32_TY: Ty = Ty::PrimTy(PrimTy::Float(FloatTy::Float32));
pub const FLOAT_64_TY: Ty = Ty::PrimTy(PrimTy::Float(FloatTy::Float64));

pub const VOID_TY: Ty = Ty::PrimTy(PrimTy::Void);
pub const STR_TY: Ty = Ty::PrimTy(PrimTy::Str);
pub const BOOL_TY: Ty = Ty::PrimTy(PrimTy::Bool);
pub const NEVER_TY: Ty = Ty::Never;
pub const UNKOWN_TY: Ty = Ty::Unkown;
pub const NULL_TY: Ty = Ty::Null;

enum NumTyPrecedence {
    Int = 0,
    Uint = 1,
    Float = 2,
}

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
    /// Package type
    Package,
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

    pub fn is_num_ty(&self) -> bool {
        match self.auto_deref() {
            | Self::PrimTy(PrimTy::Int(_))
            | Self::PrimTy(PrimTy::Uint(_))
            | Self::PrimTy(PrimTy::Float(_)) => true,
            _ => false,
        }
    }

    pub fn get_biggest_num_ty(lhs: Ty, rhs: Ty) -> Option<Ty> {
        let original_lhs = lhs.auto_deref();
        let original_rhs = rhs.auto_deref();

        let (lhs, rhs) = match (original_lhs, original_rhs) {
            (Ty::PrimTy(lhs), Ty::PrimTy(rhs)) => (lhs, rhs),
            _ => {
                return None;
            }
        };

        let get_biggest = |lhs: PrimTy, rhs: PrimTy| -> Option<Ty> {
            match (lhs, rhs) {
                (PrimTy::Int(lhs), PrimTy::Int(rhs)) => {
                    if lhs.get_ty_attr().size_bytes > rhs.get_ty_attr().size_bytes {
                        Some(original_lhs)
                    } else {
                        Some(original_rhs)
                    }
                }
                (PrimTy::Int(lhs), PrimTy::Uint(rhs)) => {
                    if lhs.get_ty_attr().size_bytes > rhs.get_ty_attr().size_bytes {
                        Some(original_lhs)
                    } else {
                        Some(original_rhs)
                    }
                }
                (PrimTy::Int(lhs), PrimTy::Float(rhs)) => {
                    if lhs.get_ty_attr().size_bytes > rhs.get_ty_attr().size_bytes {
                        Some(original_lhs)
                    } else {
                        Some(original_rhs)
                    }
                }

                (PrimTy::Uint(lhs), PrimTy::Uint(rhs)) => {
                    if lhs.get_ty_attr().size_bytes > rhs.get_ty_attr().size_bytes {
                        Some(original_lhs)
                    } else {
                        Some(original_rhs)
                    }
                }
                (PrimTy::Uint(lhs), PrimTy::Int(rhs)) => {
                    if lhs.get_ty_attr().size_bytes > rhs.get_ty_attr().size_bytes {
                        Some(original_lhs)
                    } else {
                        Some(original_rhs)
                    }
                }
                (PrimTy::Uint(lhs), PrimTy::Float(rhs)) => {
                    if lhs.get_ty_attr().size_bytes > rhs.get_ty_attr().size_bytes {
                        Some(original_lhs)
                    } else {
                        Some(original_rhs)
                    }
                }

                (PrimTy::Float(lhs), PrimTy::Float(rhs)) => {
                    if lhs.get_ty_attr().size_bytes > rhs.get_ty_attr().size_bytes {
                        Some(original_lhs)
                    } else {
                        Some(original_rhs)
                    }
                }
                (PrimTy::Float(lhs), PrimTy::Int(rhs)) => {
                    if lhs.get_ty_attr().size_bytes > rhs.get_ty_attr().size_bytes {
                        Some(original_lhs)
                    } else {
                        Some(original_rhs)
                    }
                }
                (PrimTy::Float(lhs), PrimTy::Uint(rhs)) => {
                    if lhs.get_ty_attr().size_bytes > rhs.get_ty_attr().size_bytes {
                        Some(original_lhs)
                    } else {
                        Some(original_rhs)
                    }
                }
                _ => None,
            }
        };

        if let Some(ty) = get_biggest(lhs, rhs) {
            Some(ty)
        } else {
            get_biggest(rhs, lhs)
        }
    }

    pub fn from_int(int: i64) -> Self {
        if int >= i8::MIN.into() && int <= i8::MAX.into() {
            INT_8_TY
        } else if int >= i16::MIN.into() && int <= i16::MAX.into() {
            INT_16_TY
        } else if int >= i32::MIN.into() && int <= i32::MAX.into() {
            INT_32_TY
        } else {
            INT_64_TY
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

    pub fn test_eq_strict(
        &self,
        other: Ty,
        def_id_to_name_binding: &DefIdToNameBinding<'_>
    ) -> bool {
        if self.is_ptr() && other.is_null() {
            return true;
        } else if self.is_null() && other.is_ptr() {
            return true;
        }

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
            (Self::ManyPtr(inner_ty1, mutability1), Self::Ptr(inner_ty2, mutability2)) => {
                inner_ty1.test_eq_strict(*inner_ty2, def_id_to_name_binding) &&
                    (mutability1 as u8) >= (mutability2 as u8)
            }
            (Self::Ptr(inner_ty1, mutability1), Self::ManyPtr(inner_ty2, mutability2)) => {
                inner_ty1.test_eq_strict(*inner_ty2, def_id_to_name_binding) &&
                    (mutability1 as u8) >= (mutability2 as u8)
            }
            (Self::Ptr(inner_ty1, mutability1), Self::Ptr(inner_ty2, mutability2)) => {
                inner_ty1.test_eq_strict(*inner_ty2, def_id_to_name_binding) &&
                    (mutability1 as u8) >= (mutability2 as u8)
            }
            (Self::ManyPtr(inner_ty1, mutability1), Self::ManyPtr(inner_ty2, mutability2)) => {
                inner_ty1.test_eq_strict(*inner_ty2, def_id_to_name_binding) &&
                    (mutability1 as u8) >= (mutability2 as u8)
            }
            (Self::StackPtr(inner_ty1, mutability1), Self::StackPtr(inner_ty2, mutability2)) => {
                inner_ty1.test_eq_strict(*inner_ty2, def_id_to_name_binding) &&
                    (mutability1 as u8) >= (mutability2 as u8)
            }
            _ => *self == other,
        }
    }

    pub fn get_expanded_dereffed_ty<'a>(
        &self,
        get_def_id_to_name_binding: impl Fn(DefId) -> Option<&'a NameBinding<'a>>
    ) -> Ty {
        self.auto_deref().get_expanded_ty(get_def_id_to_name_binding)
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

    fn get_expanded_ty<'a>(
        &self,
        get_def_id_to_name_binding: impl Fn(DefId) -> Option<&'a NameBinding<'a>>
    ) -> Ty {
        match self {
            Self::FnDef(def_id) => {
                let name_binding = get_def_id_to_name_binding(*def_id).unwrap();
                match name_binding.kind {
                    NameBindingKind::Fn(fn_sig, _, _) => Ty::FnSig(fn_sig),
                    _ => panic!("Expected fn"),
                }
            }
            Self::Adt(def_id) => {
                let name_binding = get_def_id_to_name_binding(*def_id).unwrap();
                match name_binding.kind {
                    NameBindingKind::Adt(Adt::Typedef(ty)) =>
                        ty.get_expanded_ty(get_def_id_to_name_binding),
                    _ => Ty::Adt(*def_id),
                }
            }

            _ => *self,
        }
    }

    pub fn test_binary<'a>(
        &self,
        other: Ty,
        op: BinaryOp,
        get_def_id_to_name_binding: &impl Fn(DefId) -> Option<&'a NameBinding<'a>>
    ) -> Option<Ty> {
        let lhs = self.get_expanded_dereffed_ty(get_def_id_to_name_binding);
        let rhs = other.get_expanded_dereffed_ty(get_def_id_to_name_binding);

        match op {
            BinaryOp::ArithmeticOp(arithmetic_op) => {
                if lhs.is_num_ty() && rhs.is_num_ty() {
                    return Self::get_biggest_num_ty(lhs, rhs).map(|x| x.auto_deref());
                }

                None
            }
            BinaryOp::ComparisonOp(comparison_op) => {
                use ComparisonOp::*;

                if lhs.is_num_ty() && rhs.is_num_ty() {
                    return Some(BOOL_TY);
                }

                match (lhs, comparison_op, rhs) {
                    (BOOL_TY, Eq | Ne | Ge | Gt | Le | Lt, BOOL_TY) => Some(BOOL_TY),
                    _ => None,
                }
            }
        }
    }

    pub fn try_deref_as_tuple<'a>(
        &self,
        get_def_id_to_name_binding: impl Fn(DefId) -> Option<&'a NameBinding<'a>>
    ) -> Option<&'static [Ty]> {
        let ty = self.auto_deref().get_expanded_ty(get_def_id_to_name_binding);

        match ty {
            Ty::Tuple(tuple_ty) => Some(tuple_ty),
            _ => None,
        }
    }

    pub fn try_deref_as_adt<'a>(
        &self,
        get_def_id_to_name_binding: impl Fn(DefId) -> Option<&'a NameBinding<'a>>
    ) -> Option<(DefId, Adt<'a>)> {
        let ty = self.auto_deref();
        match ty {
            Ty::Adt(def_id) => {
                let name_binding = get_def_id_to_name_binding(def_id).unwrap();
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
            if let Ty::Ptr(inner_ty, _) | Ty::StackPtr(inner_ty, _) = ty {
                ty = *inner_ty;
            } else {
                break ty;
            }
        }
    }

    pub fn try_deref_once(&self) -> Option<Ty> {
        if let Ty::Ptr(inner_ty, _) | Ty::StackPtr(inner_ty, _) = *self {
            Some(*inner_ty)
        } else {
            None
        }
    }
}

impl GetTyAttr for Ty {
    fn get_ty_attr(&self, resolved_information: &ResolvedInformation) -> TyAttr {
        let mut ty_attr = match self {
            Self::AtdConstructer(_) => panic!("Constructer function"),
            Self::Package => panic!("Package"),
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

                match name_binding.kind {
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
                }
            }
            Self::Ptr(_, _) => TyAttr::new(8, 8),
            Self::ManyPtr(_, _) => TyAttr::new(8, 8),
            Self::StackPtr(_, _) => TyAttr::new(8, 8),
            Self::PrimTy(prim_ty) => prim_ty.get_ty_attr(resolved_information),
            t @ (Self::Unkown | Self::Never) => panic!("{} has no size and alignment", t),
        };

        if ty_attr.alignment_bytes == 0 {
            ty_attr.alignment_bytes = 1;
        }
        ty_attr
    }
}

impl Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Null => write!(f, "null"),
            Self::AtdConstructer(def_id) => write!(f, "{}", def_id.symbol.get()),
            Self::Package => write!(f, "pkg"),
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

/// Primitive types is defined by the compiler
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum PrimTy {
    /// Integer type `Int`
    Int(IntTy),
    /// Unsigned integer type `Uint`
    Uint(UintTy),
    // Float type `Float`
    Float(FloatTy),
    Str,
    /// 8-bit boolean `Bool`
    Bool,
    /// Type `Void`
    Void,
}

impl GetTyAttr for PrimTy {
    fn get_ty_attr(&self, _: &ResolvedInformation) -> TyAttr {
        match self {
            Self::Int(int_ty) => int_ty.get_ty_attr(),
            Self::Uint(uint_ty) => uint_ty.get_ty_attr(),
            Self::Float(float_ty) => float_ty.get_ty_attr(),
            Self::Bool => TyAttr::new(1, 1),
            Self::Void => TyAttr::new(0, 0),
            Self::Str => TyAttr::new(8, 8),
        }
    }
}

impl Display for PrimTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(int_ty) => int_ty.fmt(f),
            Self::Uint(uint_ty) => uint_ty.fmt(f),
            Self::Float(float_ty) => float_ty.fmt(f),
            Self::Bool => write!(f, "Bool"),
            Self::Void => write!(f, "Void"),
            Self::Str => write!(f, "Str"),
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum IntTy {
    Int8,
    Int16,
    Int32,
    Int64,
}

impl IntTy {
    fn get_ty_attr(&self) -> TyAttr {
        match self {
            Self::Int8 => TyAttr::new(1, 1),
            Self::Int16 => TyAttr::new(2, 2),
            Self::Int32 => TyAttr::new(4, 4),
            Self::Int64 => TyAttr::new(8, 8),
        }
    }
}

impl Display for IntTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int8 => write!(f, "int8"),
            Self::Int16 => write!(f, "int16"),
            Self::Int32 => write!(f, "int32"),
            Self::Int64 => write!(f, "int64"),
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum UintTy {
    Uint8,
    Uint16,
    Uint32,
    Uint64,
}

impl UintTy {
    fn get_ty_attr(&self) -> TyAttr {
        match self {
            Self::Uint8 => TyAttr::new(1, 1),
            Self::Uint16 => TyAttr::new(2, 2),
            Self::Uint32 => TyAttr::new(4, 4),
            Self::Uint64 => TyAttr::new(8, 8),
        }
    }
}

impl Display for UintTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uint8 => write!(f, "uint8"),
            Self::Uint16 => write!(f, "uint16"),
            Self::Uint32 => write!(f, "uint32"),
            Self::Uint64 => write!(f, "uint64"),
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum FloatTy {
    Float32,
    Float64,
}

impl FloatTy {
    fn get_ty_attr(&self) -> TyAttr {
        match self {
            Self::Float32 => TyAttr::new(4, 4),
            Self::Float64 => TyAttr::new(8, 8),
        }
    }
}

impl Display for FloatTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Float32 => write!(f, "float32"),
            Self::Float64 => write!(f, "float64"),
        }
    }
}
