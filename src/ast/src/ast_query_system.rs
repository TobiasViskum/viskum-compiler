use fxhash::FxHashMap;
use ir_defs::NodeId;

use crate::{
    AssignStmt,
    BinaryExpr,
    BlockExpr,
    DefineStmt,
    FunctionStmt,
    GroupExpr,
    IdentExpr,
    IdentPat,
    IfExpr,
    IntegerExpr,
};

/// Used to lookup any node inside the ast
///
/// It's built during parsing, and is a part of the Ast struct
#[derive(Debug)]
pub struct AstQuerySystem<'ast> {
    ast_node_id_to_ast_node: FxHashMap<NodeId, AstQueryEntry<'ast>>,
    nodes_count: usize,
}

impl<'ast> AstQuerySystem<'ast> {
    pub fn new(nodes_count: usize) -> Self {
        Self {
            ast_node_id_to_ast_node: Default::default(),
            nodes_count,
        }
    }

    pub fn assert_nodes_amount(&self) {
        assert_eq!(
            self.nodes_count,
            self.ast_node_id_to_ast_node.len(),
            "Expected all nodes to be inserted into the AstQuerySystem. Cannot continue compilation"
        )
    }

    pub fn query_all(&self, mut f: impl FnMut(&NodeId, &AstQueryEntry)) {
        self.ast_node_id_to_ast_node
            .iter()
            .for_each(|(node_id, query_entry)| f(node_id, query_entry));
    }

    pub fn query_node(&self, node_id: NodeId) -> AstQueryEntry<'ast> {
        self.ast_node_id_to_ast_node.get(&node_id).copied().expect("Expected AstQueryEntry")
    }

    pub fn insert_entry(&mut self, node_id: NodeId, ast_query_entry: AstQueryEntry<'ast>) {
        self.ast_node_id_to_ast_node.insert(node_id, ast_query_entry);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum AstQueryEntry<'ast> {
    BinarExpr(&'ast BinaryExpr<'ast>),
    FunctionStmt(&'ast FunctionStmt<'ast>),
    DefineStmt(&'ast DefineStmt<'ast>),
    AssignStmt(&'ast AssignStmt<'ast>),
    GroupExpr(&'ast GroupExpr<'ast>),
    IntegerExpr(&'ast IntegerExpr),
    IdentExpr(&'ast IdentExpr),
    IdentPat(&'ast IdentPat),
    BlockExpr(&'ast BlockExpr<'ast>),
    IfExpr(&'ast IfExpr<'ast>),
}
