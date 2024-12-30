use std::fmt::Display;

use derive_new::new;
use fxhash::FxHashMap;
use span::Span;

use crate::{ Symbol, Ty };

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum Mutability {
    Immutable = 0,
    Mutable = 1,
}

impl Display for Mutability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mutability::Mutable => write!(f, "mut "),
            Mutability::Immutable => Ok(()),
        }
    }
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub struct LexicalBinding {
    pub lexical_context: LexicalContext,
    /// The symbol is there to make it unique, when used in a hashmap
    pub symbol: Symbol,
    pub res_kind: ResKind,
}

impl LexicalBinding {
    pub fn new(lexical_context: LexicalContext, symbol: Symbol, res_kind: ResKind) -> Self {
        Self { lexical_context, symbol, res_kind }
    }
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub struct LexicalContext {
    pub context_id: ContextId,
    pub scope_id: ScopeId,
}

impl LexicalContext {
    pub fn new(context_id: ContextId, scope_id: ScopeId) -> Self {
        Self { context_id, scope_id }
    }
}

/// Used during the first pass of name resolution
#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub struct ContextId(pub u16);

/// Used during the first pass of name resolution
#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub struct ScopeId(pub u16);

/// NodeId is used both as AstNodeId and CfgNodeId
#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub struct NodeId {
    pub node_id: u32,
    pub mod_id: ModId,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub struct ModId(pub u32);

/// Refers to any definition
#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct DefId {
    pub symbol: Symbol,
    pub node_id: NodeId,
}

/// Refers to a package
#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct PkgId(pub u32);

impl DefId {
    pub fn new(symbol: Symbol, node_id: NodeId) -> Self {
        Self { symbol, node_id }
    }

    pub fn display_as_fn(&self) -> String {
        format!("@{}{}_{}", self.symbol.get(), self.node_id.mod_id.0, self.node_id.node_id)
    }

    pub fn display_as_str(&self) -> String {
        format!("@.str.{}.{}", self.node_id.mod_id.0, self.node_id.node_id)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Externism {
    /// Refers to a function in the Clib (Clang)
    Clib,
    /// Refers to all other funcitons
    NoExtern,
}

/// Information about a definition
#[derive(Debug, Clone, Copy)]
pub struct NameBinding<'res> {
    pub kind: NameBindingKind<'res>,
}

impl<'res> NameBinding<'res> {
    pub fn new(kind: NameBindingKind<'res>) -> Self {
        Self { kind }
    }

    pub fn get_res_kind(&self) -> ResKind {
        match self.kind {
            NameBindingKind::Variable(_) => ResKind::Variable,
            NameBindingKind::Adt(_) => ResKind::Adt,
            NameBindingKind::Fn(_, _, _) => ResKind::Fn,
            NameBindingKind::ConstStr(_) => ResKind::ConstStr,
            NameBindingKind::Pkg(_) => todo!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HasSelfArg {
    Yes,
    No,
}

#[derive(Debug, Clone, Copy)]
pub enum NameBindingKind<'res> {
    Variable(Mutability),
    Adt(Adt<'res>),
    Fn(FnSig, HasSelfArg, Externism),
    ConstStr(ConstStrLen),
    Pkg(&'res [DefId]),
    // Module
    // Import
}

// #[derive(Debug, Clone, Copy)]
// pub struct ConstStr {
//     pub def_id: DefId,
//     pub len: ConstStrLen,
// }

#[derive(Debug, Clone, Copy)]
pub struct ConstStrLen(pub u32);

#[derive(Debug, Clone, Copy)]
pub struct EmumVaraintId(pub u32);

/// Algebraic data type (e.g. structs, enums etc.)
#[derive(Debug, Clone, Copy)]
pub enum Adt<'res> {
    Struct(&'res [(DefId, Ty)]),
    // TupleStruct(&'res [Ty]),
    Enum(&'res [DefId]),
    EnumVariant(DefId, EmumVaraintId, &'res [Ty]),
    Typedef(Ty),
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct FnSig {
    pub args: &'static [Ty],
    pub ret_ty: &'static Ty,
}

impl FnSig {
    pub fn new(args: &'static [Ty], ret_ty: &'static Ty) -> Self {
        Self { args, ret_ty }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, new)]
pub struct TraitImplId {
    pub implementor_def_id: DefId,
    /// If this is None, it means that it's an implementation for no trait
    pub trait_def_id: Option<DefId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, new)]
pub struct ImplId {
    pub implementor: DefId,
    pub fn_def_id: DefId,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub struct AdtId(pub DefId);

/// Only difference between this and `DefKind`, is that this has no data
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum ResKind {
    Variable,
    ConstVariable,
    Adt,
    Fn,
    ConstStr,
}

#[derive(Debug, Clone, Copy)]
pub enum AdtKind {
    Struct,
    Enum,
    EnumVariant,
    Typedef,
}

pub type NodeIdToTy = FxHashMap<NodeId, Ty>;
pub type NodeIdToDefId = FxHashMap<NodeId, DefId>;
pub type DefIdToNameBinding<'res> = FxHashMap<DefId, NameBinding<'res>>;

#[derive(Debug)]
pub struct ResolvedInformation<'res> {
    pub node_id_to_ty: NodeIdToTy,
    pub node_id_to_def_id: NodeIdToDefId,
    pub def_id_to_name_binding: DefIdToNameBinding<'res>,
    // pub def_id_to_global_mem_id: FxHashMap<DefId, GlobalMemId>,
    pub const_strs: Vec<(DefId, ConstStrLen)>,
    pub clib_fns: Vec<DefId>,
}

impl<'res> ResolvedInformation<'res> {
    pub fn get_ty_from_node_id(&self, node_id: &NodeId) -> Ty {
        *self.node_id_to_ty.get(node_id).expect("Expected type to node id")
    }

    pub fn get_def_id_from_node_id(&self, node_id: &NodeId) -> DefId {
        *self.node_id_to_def_id.get(node_id).expect("Expected DefId to node id")
    }

    pub fn try_get_def_id_from_node_id(&self, node_id: &NodeId) -> Option<DefId> {
        self.node_id_to_def_id.get(node_id).copied()
    }

    pub fn get_name_binding_from_def_id(&self, def_id: &DefId) -> NameBinding<'res> {
        *self.def_id_to_name_binding.get(def_id).expect("Expected namebinding to def_id")
    }

    // pub fn get_global_mem_id_from_def_id(&self, def_id: &DefId) -> GlobalMemId {
    //     *self.def_id_to_global_mem_id
    //         .get(def_id)
    //         .expect("Expected global mem to be binded to def id")
    // }

    pub fn is_clib_fn(&self, def_id: &DefId) -> bool {
        match self.get_name_binding_from_def_id(def_id).kind {
            NameBindingKind::Fn(_, _, Externism::Clib) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CfgFnKind {
    Main,
    Fn(DefId),
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct GlobalMemId(pub u32);

impl Display for GlobalMemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}", self.0)
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct LocalMemId(pub u32);

impl Display for LocalMemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct ResultMemId(pub u32);

impl Display for ResultMemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r{}", self.0)
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct TempId(pub u32);

impl Display for TempId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "_{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, new)]
pub struct GlobalMem {
    pub global_mem_id: GlobalMemId,
    pub def_id: DefId,
    pub span: Span,
}

impl Display for GlobalMem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}", self.def_id.symbol.get())
    }
}

/// Implement `requires_drop` for when heap objects is implemented
#[derive(Debug, Clone, Copy, new)]
pub struct LocalMem {
    pub local_mem_id: LocalMemId,
    pub symbol: Symbol,
    pub span: Span,
    pub ty: Ty,
    pub mutability: Mutability,
}

impl Display for LocalMem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.symbol.get(), get_subscript(self.local_mem_id.0))
    }
}

#[derive(Debug, Clone, Copy, new)]
pub struct ResultMem {
    pub result_mem_id: ResultMemId,
    pub ty: Ty,
}

impl Display for ResultMem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.result_mem_id)
    }
}
fn get_subscript(mem: u32) -> String {
    let mut subscript = String::new();
    for char in mem.to_string().chars() {
        subscript += match char {
            '0' => "₀",
            '1' => "₁",
            '2' => "₂",
            '3' => "₃",
            '4' => "₄",
            '5' => "₅",
            '6' => "₆",
            '7' => "₇",
            '8' => "₈",
            '9' => "₉",
            _ => unreachable!(),
        };
    }

    subscript
}
