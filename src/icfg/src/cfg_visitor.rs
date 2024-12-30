use ir::Ty;

use crate::{
    BasicBlock,
    BinaryNode,
    BranchCondNode,
    BranchNode,
    ByteAccessNode,
    CallNode,
    Cfg,
    IndexNode,
    LoadNode,
    LocalMem,
    Node,
    NodeKind,
    ResultMem,
    ReturnNode,
    StoreNode,
    TempId,
    TyCastNode,
};

// struct Linear;
// struct LeftToRight;

// impl CfgVisitMode for Linear {
//     fn is_linear() -> bool {
//         true
//     }
// }

// pub trait CfgVisitMode {
//     fn is_linear() -> bool;
// }

/// This is the visitor trait that visits a cfg
///
/// First it visits all local mems (variables).
/// Then it visits all basic blocks
pub trait CfgVisitor: Sized {
    type Result: Sized;

    fn default_result() -> Self::Result;

    fn visit_cfg(&mut self, cfg: &Cfg) -> Self::Result {
        walk_cfg(self, cfg)
    }

    fn visit_arg(&mut self, arg: &(TempId, Ty)) -> Self::Result {
        Self::default_result()
    }

    fn visit_local_mem(&mut self, local_mem: &LocalMem) -> Self::Result {
        Self::default_result()
    }

    fn visit_result_mem(&mut self, result_mem: &ResultMem) -> Self::Result {
        Self::default_result()
    }

    fn visit_basic_block(&mut self, basic_block: &BasicBlock, cfg: &Cfg) -> Self::Result {
        walk_basic_block(self, basic_block, cfg)
    }

    fn visit_node(&mut self, node: &Node, cfg: &Cfg) -> Self::Result {
        walk_node(self, node, cfg)
    }

    #[allow(unused_variables)]
    fn visit_binary_node(&mut self, binary_node: &BinaryNode, cfg: &Cfg) -> Self::Result {
        Self::default_result()
    }

    #[allow(unused_variables)]
    fn visit_branch_cond_node(
        &mut self,
        branch_cond_node: &BranchCondNode,
        cfg: &Cfg
    ) -> Self::Result {
        Self::default_result()
    }

    #[allow(unused_variables)]
    fn visit_branch_node(&mut self, branch_node: &BranchNode, cfg: &Cfg) -> Self::Result {
        Self::default_result()
    }

    #[allow(unused_variables)]
    fn visit_load_node(&mut self, load_node: &LoadNode, cfg: &Cfg) -> Self::Result {
        Self::default_result()
    }

    #[allow(unused_variables)]
    fn visit_store_node(&mut self, store_node: &StoreNode, cfg: &Cfg) -> Self::Result {
        Self::default_result()
    }

    #[allow(unused_variables)]
    fn visit_index_node(&mut self, index_node: &IndexNode, cfg: &Cfg) -> Self::Result {
        Self::default_result()
    }

    #[allow(unused_variables)]
    fn visit_byte_access_node(
        &mut self,
        byte_access_node: &ByteAccessNode,
        cfg: &Cfg
    ) -> Self::Result {
        Self::default_result()
    }

    #[allow(unused_variables)]
    fn visit_return_node(&mut self, return_node: &ReturnNode, cfg: &Cfg) -> Self::Result {
        Self::default_result()
    }

    #[allow(unused_variables)]
    fn visit_call_node(&mut self, call_node: &CallNode, cfg: &Cfg) -> Self::Result {
        Self::default_result()
    }

    #[allow(unused_variables)]
    fn visit_ty_cast_node(&mut self, ty_cast_node: &TyCastNode, cfg: &Cfg) -> Self::Result {
        Self::default_result()
    }
}

pub fn walk_cfg<'ctx, V>(visitor: &mut V, cfg: &Cfg) -> V::Result where V: CfgVisitor {
    walk_args(visitor, cfg);
    walk_local_mems(visitor, cfg);
    walk_result_mems(visitor, cfg);
    walk_basic_blocks(visitor, cfg);
    V::default_result()
}

pub fn walk_args<'ctx, V>(visitor: &mut V, cfg: &Cfg) -> V::Result where V: CfgVisitor {
    cfg.args.iter().for_each(|arg| {
        visitor.visit_arg(arg);
    });

    V::default_result()
}

pub fn walk_local_mems<'ctx, V>(visitor: &mut V, cfg: &Cfg) -> V::Result where V: CfgVisitor {
    cfg.local_mems.iter().for_each(|local_mem| {
        visitor.visit_local_mem(local_mem);
    });

    V::default_result()
}

pub fn walk_result_mems<'ctx, V>(visitor: &mut V, cfg: &Cfg) -> V::Result where V: CfgVisitor {
    cfg.result_mems.iter().for_each(|result_mem| {
        visitor.visit_result_mem(result_mem);
    });

    V::default_result()
}

pub fn walk_basic_blocks<'ctx, V>(visitor: &mut V, cfg: &Cfg) -> V::Result where V: CfgVisitor {
    cfg.basic_blocks.iter().for_each(|basic_block| {
        visitor.visit_basic_block(basic_block, cfg);
    });

    V::default_result()
}

pub fn walk_basic_block<'ctx, V>(visitor: &mut V, basic_block: &BasicBlock, cfg: &Cfg) -> V::Result
    where V: CfgVisitor
{
    basic_block.nodes.iter().for_each(|stmt| {
        visitor.visit_node(stmt, cfg);
    });

    V::default_result()
}

pub fn walk_node<'ctx, V>(visitor: &mut V, node: &Node, cfg: &Cfg) -> V::Result where V: CfgVisitor {
    match &node.kind {
        NodeKind::BinaryNode(binary_node) => visitor.visit_binary_node(binary_node, cfg),
        NodeKind::BranchCondNode(branch_cond_node) =>
            visitor.visit_branch_cond_node(branch_cond_node, cfg),
        NodeKind::StoreNode(store_node) => visitor.visit_store_node(store_node, cfg),
        NodeKind::BranchNode(branch_node) => visitor.visit_branch_node(branch_node, cfg),
        NodeKind::LoadNode(load_node) => visitor.visit_load_node(load_node, cfg),
        NodeKind::IndexNode(index_node) => visitor.visit_index_node(index_node, cfg),
        NodeKind::ByteAccessNode(byte_access_node) =>
            visitor.visit_byte_access_node(byte_access_node, cfg),
        NodeKind::ReturnNode(return_node) => visitor.visit_return_node(return_node, cfg),
        NodeKind::CallNode(call_node) => visitor.visit_call_node(call_node, cfg),
        NodeKind::TyCastNode(ty_cast_node) => visitor.visit_ty_cast_node(ty_cast_node, cfg),
    }
}
