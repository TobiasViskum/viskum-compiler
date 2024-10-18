use std::fmt::Display;

use fxhash::FxHashMap;

use crate::{ Symbol, Ty };

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Mutability {
    Mutable,
    Immutable,
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub struct LexicalBinding {
    pub lexical_context: LexicalContext,
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
pub struct ContextId(pub u32);

/// Used during the first pass of name resolution
#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub struct ScopeId(pub u32);

/// NodeId is used both as AstNodeId and CfgNodeId
#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub struct NodeId(pub u32);

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Refers to any definition
#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct DefId {
    pub symbol: Symbol,
    pub node_id: NodeId,
}

impl DefId {
    pub fn new(symbol: Symbol, node_id: NodeId) -> Self {
        Self { symbol, node_id }
    }
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
            NameBindingKind::Fn(_) => ResKind::Fn,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NameBindingKind<'res> {
    Variable(Mutability),
    Adt(Adt<'res>),
    Fn(FnSig),
    // Module
    // Import
}

/// Algebraic data type (e.g. structs, enums etc.)
#[derive(Debug, Clone, Copy)]
pub enum Adt<'res> {
    Struct(&'res [(DefId, Ty)]),
    Typedef(Ty),
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct FnSig {
    pub args: &'static [(DefId, Ty)],
    pub ret_ty: &'static Ty,
}

impl FnSig {
    pub fn new(args: &'static [(DefId, Ty)], ret_ty: &'static Ty) -> Self {
        Self { args, ret_ty }
    }
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub struct AdtId(pub DefId);

#[derive(Debug, Clone, Copy)]
pub enum DefKind<'res> {
    Variable(Mutability),
    Stuct(&'res [(DefId, Ty)]),
}

/// Only difference between this and `DefKind`, is that this has no data
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum ResKind {
    Variable,
    Adt,
    Fn,
}

pub type NodeIdToTy = FxHashMap<NodeId, Ty>;
pub type NodeIdToDefId = FxHashMap<NodeId, DefId>;
pub type DefIdToNameBinding<'res> = FxHashMap<DefId, NameBinding<'res>>;

pub struct ResolvedInformation<'res> {
    pub node_id_to_ty: NodeIdToTy,
    pub node_id_to_def_id: NodeIdToDefId,
    pub def_id_to_name_binding: DefIdToNameBinding<'res>,
}

impl<'res> ResolvedInformation<'res> {
    pub fn get_ty_from_node_id(&self, node_id: &NodeId) -> Ty {
        *self.node_id_to_ty.get(node_id).expect("Expected type to node id")
    }

    pub fn get_def_id_from_node_id(&self, node_id: &NodeId) -> DefId {
        *self.node_id_to_def_id.get(node_id).expect("Expected DefId to node id")
    }

    pub fn get_name_binding_from_def_id(&self, def_id: &DefId) -> NameBinding<'res> {
        *self.def_id_to_name_binding.get(def_id).expect("Expected name to be binded to def id")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CfgFnKind {
    Main,
    Fn(DefId),
}
