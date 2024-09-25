use std::fmt::Display;
use symbol::Symbol;

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
    symbol: Symbol,
    ast_node_id: NodeId,
}

impl DefId {
    pub fn new(symbol: Symbol, ast_node_id: NodeId) -> Self {
        Self { symbol, ast_node_id }
    }
}

/// Information about a definition
#[derive(Debug, Clone, Copy)]
pub struct NameBinding {
    pub kind: NameBindingKind,
}

impl From<DefKind> for NameBinding {
    fn from(value: DefKind) -> Self {
        Self { kind: NameBindingKind::DefKind(value) }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NameBindingKind {
    DefKind(DefKind),
    // Module
    // Import
}

#[derive(Debug, Clone, Copy)]
/// Resoluted def
pub enum Res {
    Def(DefKind),
}

#[derive(Debug, Clone, Copy)]
pub enum DefKind {
    Variable(Mutability),
}
