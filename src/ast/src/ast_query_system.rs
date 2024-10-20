use fxhash::FxHashMap;
use ir::NodeId;

use crate::{
    AssignStmt,
    BinaryExpr,
    BlockExpr,
    BoolExpr,
    BreakExpr,
    CallExpr,
    ContinueExpr,
    DefineStmt,
    EnumItem,
    FieldExpr,
    FnItem,
    GroupExpr,
    IdentNode,
    IfDefExpr,
    IfExpr,
    IntegerExpr,
    LoopExpr,
    PathField,
    ReturnExpr,
    StructExpr,
    StructItem,
    TupleExpr,
    TupleFieldExpr,
    TuplePat,
    TypedefItem,
};

/// Used to lookup any node inside the ast and get a reference to it
///
/// It's built when making the Resolver
#[derive(Debug)]
pub struct AstQuerySystem<'ast> {
    ast_node_id_to_ast_node: FxHashMap<NodeId, AstQueryEntry<'ast>>,
    pub expected_nodes_count: usize,
}

impl<'ast> AstQuerySystem<'ast> {
    pub fn new(expected_nodes_count: usize) -> Self {
        Self {
            ast_node_id_to_ast_node: Default::default(),
            expected_nodes_count,
        }
    }

    pub fn assert_nodes_amount(&self) {
        assert_eq!(
            self.expected_nodes_count,
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
    TupleExpr(&'ast TupleExpr<'ast>),
    TypedefItem(&'ast TypedefItem<'ast>),
    IfDefExpr(&'ast IfDefExpr<'ast>),
    LoopExpr(&'ast LoopExpr<'ast>),
    BreakExpr(&'ast BreakExpr<'ast>),
    TuplePat(&'ast TuplePat<'ast>),
    StructExpr(&'ast StructExpr<'ast>),
    FieldExpr(&'ast FieldExpr<'ast>),
    PathField(&'ast PathField<'ast>),
    TupleFieldExpr(&'ast TupleFieldExpr<'ast>),
    ContinueExpr(&'ast ContinueExpr),
    BinaryExpr(&'ast BinaryExpr<'ast>),
    FnItem(&'ast FnItem<'ast>),
    StructItem(&'ast StructItem<'ast>),
    DefineStmt(&'ast DefineStmt<'ast>),
    AssignStmt(&'ast AssignStmt<'ast>),
    GroupExpr(&'ast GroupExpr<'ast>),
    EnumItem(&'ast EnumItem<'ast>),
    CallExpr(&'ast CallExpr<'ast>),
    IntegerExpr(&'ast IntegerExpr),
    BoolExpr(&'ast BoolExpr),
    IdentNode(&'ast IdentNode),
    ReturnExpr(&'ast ReturnExpr<'ast>),
    BlockExpr(&'ast BlockExpr<'ast>),
    IfExpr(&'ast IfExpr<'ast>),
}
