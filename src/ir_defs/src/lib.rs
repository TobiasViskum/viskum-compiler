use std::fmt::Display;
use symbol::Symbol;
use ty::Ty;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Mutability {
    Mutable,
    Immutable,
}

pub enum LexicalContext {
    GlobalScope,
    Context(ContextId, ScopeId),
}

pub struct ContextId(pub u32);

/// Used during the first pass of name resolution
#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub struct ScopeId(pub u32);

/// Used as a temporary location for where ExprWithBlock results go (e.g. IfExpr, MatchExpr, etc.)
#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub struct ResultLoc(pub u32);

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
            NameBindingKind::DefKind(def_kind) => {
                match def_kind {
                    DefKind::Stuct(_) => ResKind::Struct,
                    DefKind::Variable(_) => ResKind::Variable,
                }
            }
        }
    }
}

impl<'res> From<DefKind<'res>> for NameBinding<'res> {
    fn from(value: DefKind<'res>) -> Self {
        Self { kind: NameBindingKind::DefKind(value) }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NameBindingKind<'res> {
    DefKind(DefKind<'res>),
    // Module
    // Import
}

#[derive(Debug, Clone, Copy)]
pub enum DefKind<'res> {
    Variable(Mutability),
    Stuct(&'res [(DefId, ResTy)]),
}

#[derive(Debug, Clone, Copy)]
pub enum ResTy {
    UserDef(DefId),
    Compiler(Ty),
}

/// Only difference between this and `DefKind`, is that this has no data
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum ResKind {
    Variable,
    Struct,
}
