use crate::{
    BasicBlock,
    BinaryNode,
    BranchCondNode,
    BranchNode,
    Cfg,
    LoadNode,
    LocalMem,
    Node,
    NodeKind,
    ResultMem,
    StoreNode,
};

struct Linear;
struct LeftToRight;

impl CfgVisitMode for Linear {
    fn is_linear() -> bool {
        true
    }
}

pub trait CfgVisitMode {
    fn is_linear() -> bool;
}

/// This is the visitor trait that visits a cfg
///
/// First it visits all local mems (variables).
/// Then it visits all basic blocks
pub trait CfgVisitor<'ctx>: Sized {
    type Result: Sized;

    fn default_result() -> Self::Result;

    fn visit_cfg(&mut self, cfg: &Cfg<'ctx>) -> Self::Result {
        walk_cfg(self, cfg)
    }

    fn visit_local_mem(&mut self, local_mem: &LocalMem<'ctx>) -> Self::Result {
        Self::default_result()
    }

    fn visit_result_mem(&mut self, result_mem: &ResultMem<'ctx>) -> Self::Result {
        Self::default_result()
    }

    fn visit_basic_block(
        &mut self,
        basic_block: &BasicBlock<'ctx>,
        cfg: &Cfg<'ctx>
    ) -> Self::Result {
        walk_basic_block(self, basic_block, cfg)
    }

    fn visit_node(&mut self, node: &Node<'ctx>, cfg: &Cfg<'ctx>) -> Self::Result {
        walk_node(self, node, cfg)
    }

    #[allow(unused_variables)]
    fn visit_binary_node(&mut self, binary_node: &BinaryNode, cfg: &Cfg<'ctx>) -> Self::Result {
        Self::default_result()
    }

    #[allow(unused_variables)]
    fn visit_branch_cond_node(
        &mut self,
        branch_cond_node: &BranchCondNode,
        cfg: &Cfg<'ctx>
    ) -> Self::Result {
        Self::default_result()
    }

    #[allow(unused_variables)]
    fn visit_branch_node(&mut self, branch_node: &BranchNode, cfg: &Cfg<'ctx>) -> Self::Result {
        Self::default_result()
    }

    #[allow(unused_variables)]
    fn visit_load_node(&mut self, load_node: &LoadNode, cfg: &Cfg<'ctx>) -> Self::Result {
        Self::default_result()
    }

    #[allow(unused_variables)]
    fn visit_init_node(&mut self, init_node: &StoreNode, cfg: &Cfg<'ctx>) -> Self::Result {
        Self::default_result()
    }
}

pub fn walk_cfg<'ctx, V>(visitor: &mut V, cfg: &Cfg<'ctx>) -> V::Result where V: CfgVisitor<'ctx> {
    walk_local_mems(visitor, cfg);
    walk_result_mems(visitor, cfg);
    walk_basic_blocks(visitor, cfg);
    V::default_result()
}

pub fn walk_local_mems<'ctx, V>(visitor: &mut V, cfg: &Cfg<'ctx>) -> V::Result
    where V: CfgVisitor<'ctx>
{
    cfg.local_mems.iter().for_each(|local_mem| {
        visitor.visit_local_mem(local_mem);
    });

    V::default_result()
}

pub fn walk_result_mems<'ctx, V>(visitor: &mut V, cfg: &Cfg<'ctx>) -> V::Result
    where V: CfgVisitor<'ctx>
{
    cfg.result_mems.iter().for_each(|result_mem| {
        visitor.visit_result_mem(result_mem);
    });

    V::default_result()
}

pub fn walk_basic_blocks<'ctx, V>(visitor: &mut V, cfg: &Cfg<'ctx>) -> V::Result
    where V: CfgVisitor<'ctx>
{
    cfg.basic_blocks.iter().for_each(|basic_block| {
        visitor.visit_basic_block(basic_block, cfg);
    });

    V::default_result()
}

pub fn walk_basic_block<'ctx, V>(
    visitor: &mut V,
    basic_block: &BasicBlock<'ctx>,
    cfg: &Cfg<'ctx>
) -> V::Result
    where V: CfgVisitor<'ctx>
{
    basic_block.nodes.iter().for_each(|stmt| {
        visitor.visit_node(stmt, cfg);
    });

    V::default_result()
}

pub fn walk_node<'ctx, V>(visitor: &mut V, node: &Node<'ctx>, cfg: &Cfg<'ctx>) -> V::Result
    where V: CfgVisitor<'ctx>
{
    match &node.kind {
        NodeKind::BinaryNode(binary_node) => visitor.visit_binary_node(binary_node, cfg),
        NodeKind::BranchCondNode(branch_cond_node) =>
            visitor.visit_branch_cond_node(branch_cond_node, cfg),
        NodeKind::StoreNode(init_node) => visitor.visit_init_node(init_node, cfg),
        NodeKind::BranchNode(branch_node) => visitor.visit_branch_node(branch_node, cfg),
        NodeKind::LoadNode(load_node) => visitor.visit_load_node(load_node, cfg),
    }
}
