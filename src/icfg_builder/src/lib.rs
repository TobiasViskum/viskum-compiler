use ast::{
    Ast,
    AstTypeChecked,
    FunctionStmt,
    IfFalseBranchExpr,
    PatKind,
    PlaceExpr,
    PlaceKind,
    Visitor,
};
use data_structures::Either;
use fxhash::FxHashMap;
use icfg::{
    BasicBlock,
    BasicBlockId,
    BinaryNode,
    BranchCondNode,
    BranchNode,
    Cfg,
    Icfg,
    StoreNode,
    LoadNode,
    LocalMem,
    LocalMemId,
    Node,
    NodeKind,
    Operand,
    OperandKind,
    ResultMem,
    ResultMemId,
    TempId,
};
use ir_defs::{ DefId, Mutability, NodeId, ResultLoc };
use op::BinaryOp;
use resolver::ResolvedInformation;
use symbol::Symbol;
use ty::{ PrimTy, Ty, TyCtx };

pub struct IcfgBuilder<'icfg, 'ast> {
    pending_functions: Vec<&'ast FunctionStmt<'ast>>,
    ast: Ast<'ast, AstTypeChecked>,
    cfgs: Vec<Cfg<'icfg>>,
    resolved_information: ResolvedInformation<'icfg>,
    src: &'ast str,
}

impl<'icfg, 'ast> IcfgBuilder<'icfg, 'ast> {
    pub fn new(
        ast: Ast<'ast, AstTypeChecked>,
        resolved_information: ResolvedInformation<'icfg>,
        src: &'ast str
    ) -> Self {
        Self {
            ast,
            pending_functions: Default::default(),
            cfgs: Default::default(),
            resolved_information,
            src,
        }
    }

    pub fn build(mut self) -> Icfg<'icfg> {
        let cfg_builder = self.new_cfg_builder();
        let cfg = cfg_builder.build_cfg();
        self.cfgs.push(cfg);

        Icfg::new(self.cfgs)
    }

    pub(crate) fn new_cfg_builder<'c>(&'c mut self) -> CfgBuilder<'icfg, 'ast, 'c> {
        CfgBuilder::new(self)
    }

    pub(crate) fn append_function(&mut self, function: &'ast FunctionStmt<'ast>) {
        self.pending_functions.push(function);
    }

    pub(crate) fn get_ty_from_node_id(&self, node_id: NodeId) -> &'icfg Ty {
        *self.resolved_information.node_id_to_ty.get(&node_id).expect("Expected ty from node_id")
    }

    pub(crate) fn get_def_id_from_node_id(&self, node_id: NodeId) -> DefId {
        *self.resolved_information.node_id_to_def_id
            .get(&node_id)
            .expect("Expected DefId from NodeId")
    }

    pub(crate) fn get_ast(&self) -> &Ast<'ast, AstTypeChecked> {
        &self.ast
    }
}

/*
REMOVE: building_basic_block
*/
pub struct CfgBuilder<'icfg, 'ast, 'c> {
    icfg_builder: &'c mut IcfgBuilder<'icfg, 'ast>,
    local_mems: Vec<LocalMem<'icfg>>,
    result_mems: Vec<ResultMem<'icfg>>,
    basic_blocks: Vec<BasicBlock<'icfg>>,
    def_id_to_local_mem_id: FxHashMap<DefId, LocalMemId>,
    result_loc_to_result_mem_id: FxHashMap<ResultLoc, ResultMemId>,
    // entry_function: &'ast FunctionStmt<'ast>,
    next_ssa_id: u32,
}

impl<'icfg, 'ast, 'c> CfgBuilder<'icfg, 'ast, 'c> {
    pub fn new(
        icfg_builder: &'c mut IcfgBuilder<'icfg, 'ast>
        // entry_function: &'ast FunctionStmt<'ast>
    ) -> Self {
        let mut basic_blocks = Vec::with_capacity(16);
        let basic_block = BasicBlock::new(BasicBlockId(0));
        basic_blocks.push(basic_block);

        Self {
            icfg_builder,
            local_mems: Vec::with_capacity(8),
            result_mems: Vec::with_capacity(8),
            basic_blocks,
            def_id_to_local_mem_id: Default::default(),
            result_loc_to_result_mem_id: Default::default(),
            // entry_function,
            next_ssa_id: 0,
        }
    }

    pub fn build_cfg(mut self) -> Cfg<'icfg> {
        let ast = self.icfg_builder.get_ast();
        self.visit_stmts(ast.main_scope.stmts);

        // self.push_stmt(Stmt::new(kind));

        Cfg::new(self.local_mems, self.result_mems, self.basic_blocks)
    }

    pub(crate) fn get_ty(ty: Ty) -> &'icfg Ty {
        TyCtx.intern_type(ty)
    }

    pub(crate) fn get_next_ssa_id(&mut self) -> u32 {
        self.next_ssa_id += 1;
        self.next_ssa_id - 1
    }

    pub(crate) fn get_local_mem_id_from_def_id(&self, def_id: DefId) -> LocalMemId {
        *self.def_id_to_local_mem_id.get(&def_id).expect("Expected LocalMem from DefId")
    }

    pub(crate) fn set_def_id_to_local_mem_id(&mut self, def_id: DefId, local_mem_id: LocalMemId) {
        self.def_id_to_local_mem_id.insert(def_id, local_mem_id);
    }

    pub(crate) fn set_result_loc_to_local_mem(
        &mut self,
        result_loc: ResultLoc,
        make_result_mem: impl Fn() -> ResultMem<'icfg>
    ) -> ResultMemId {
        if let Some(result_mem_id) = self.result_loc_to_result_mem_id.get(&result_loc) {
            *result_mem_id
        } else {
            let result_mem = make_result_mem();
            let result_mem_id = ResultMemId(self.result_mems.len() as u32);
            self.result_mems.push(result_mem);
            self.result_loc_to_result_mem_id.insert(result_loc, result_mem_id);
            result_mem_id
        }
    }

    pub(crate) fn new_basic_block(&mut self) -> BasicBlockId {
        let bb_id = BasicBlockId(self.basic_blocks.len() as u32);
        self.basic_blocks.push(BasicBlock::new(bb_id));
        bb_id
    }

    pub(crate) fn push_node(&mut self, node: Node<'icfg>) {
        let bb_id = BasicBlockId((self.basic_blocks.len() - 1) as u32);
        self.push_node_to(bb_id, node);
    }

    pub(crate) fn push_node_to(&mut self, basic_block_id: BasicBlockId, node: Node<'icfg>) {
        let basic_block = self.basic_blocks
            .get_mut(basic_block_id.0 as usize)
            .expect("Expected basic block");

        if let Some(last_node) = basic_block.nodes.last() {
            match &last_node.kind {
                NodeKind::BranchNode(_) | NodeKind::BranchCondNode(_) => {
                    return;
                }
                _ => {}
            }
        }

        basic_block.push_node(node);
    }
}

impl<'icfg, 'ast, 'c> Visitor<'ast> for CfgBuilder<'icfg, 'ast, 'c> {
    type Result = Operand<'icfg>;

    fn default_result() -> Self::Result {
        Operand::new(Default::default(), Self::get_ty(Ty::PrimTy(PrimTy::Void)))
    }

    fn visit_interger_expr(&mut self, integer_expr: &'ast ast::IntegerExpr) -> Self::Result {
        let ty = self.icfg_builder.get_ty_from_node_id(integer_expr.ast_node_id);

        Operand::new(OperandKind::from(integer_expr.val), ty)
    }

    fn visit_bool_expr(&mut self, bool_expr: &'ast ast::BoolExpr) -> Self::Result {
        let ty = self.icfg_builder.get_ty_from_node_id(bool_expr.ast_node_id);

        Operand::new(OperandKind::from(bool_expr.val), ty)
    }

    fn visit_ident_expr(&mut self, ident_expr: &'ast ast::IdentExpr) -> Self::Result {
        let def_id = self.icfg_builder.get_def_id_from_node_id(ident_expr.ast_node_id);
        let ty = self.icfg_builder.get_ty_from_node_id(ident_expr.ast_node_id);
        let local_mem_id = self.get_local_mem_id_from_def_id(def_id);

        let result_place = TempId(self.get_next_ssa_id());
        self.push_node(
            Node::new(
                NodeKind::LoadNode(LoadNode::new(result_place, Either::Left(local_mem_id), ty))
            )
        );

        Operand::new(OperandKind::from(result_place), ty)
    }

    fn visit_if_expr(&mut self, if_expr: &'ast ast::IfExpr<'ast>) -> Self::Result {
        let if_expr_ty = self.icfg_builder.get_ty_from_node_id(if_expr.ast_node_id);
        let cond_ty = self.icfg_builder.get_ty_from_node_id(if_expr.condition.ast_node_id);

        let result_mem_id = ResultMemId(self.result_mems.len() as u32);
        let result_mem_id = self.set_result_loc_to_local_mem(if_expr.result_loc, || {
            ResultMem::new(result_mem_id, if_expr_ty)
        });

        let bb_id_before_if_expr = BasicBlockId((self.basic_blocks.len() - 1) as u32);
        let cond = self.visit_expr(if_expr.condition);

        let true_bb_id = {
            let true_bb_id = self.new_basic_block();
            let true_operand = self.visit_block_expr(if_expr.true_block);
            self.push_node(
                Node::new(
                    NodeKind::StoreNode(
                        StoreNode::new(Either::Right(result_mem_id), if_expr_ty, true_operand)
                    )
                )
            );
            true_bb_id
        };

        if let Some(false_branch) = &if_expr.false_block {
            let false_bb_id = self.new_basic_block();
            let bb_id_after_false_expr = BasicBlockId(false_bb_id.0 + 1);
            let branch_out_node = Node::new(
                NodeKind::BranchNode(BranchNode::new(bb_id_after_false_expr))
            );

            match false_branch {
                IfFalseBranchExpr::ElifExpr(elif_expr) => {
                    self.visit_if_expr(elif_expr);
                    let cond_ty = self.icfg_builder.get_ty_from_node_id(
                        elif_expr.condition.ast_node_id
                    );
                    // self.push_node(
                    //     Node::new(
                    //         NodeKind::StoreNode(
                    //             StoreNode::new(
                    //                 Place::ResultMemId(result_mem_id),
                    //                 if_expr_ty,
                    //                 false_operand
                    //             )
                    //         )
                    //     )
                    // );
                    self.push_node_to(
                        bb_id_before_if_expr,
                        Node::new(
                            NodeKind::BranchCondNode(
                                BranchCondNode::new(cond, cond_ty, true_bb_id, false_bb_id)
                            )
                        )
                    );
                    self.push_node_to(true_bb_id, branch_out_node);
                    self.push_node_to(false_bb_id, branch_out_node);
                }
                IfFalseBranchExpr::ElseExpr(else_expr) => {
                    let false_operand = self.visit_block_expr(else_expr);
                    self.push_node(
                        Node::new(
                            NodeKind::StoreNode(
                                StoreNode::new(
                                    Either::Right(result_mem_id),
                                    if_expr_ty,
                                    false_operand
                                )
                            )
                        )
                    );
                    self.push_node_to(
                        bb_id_before_if_expr,
                        Node::new(
                            NodeKind::BranchCondNode(
                                BranchCondNode::new(cond, cond_ty, true_bb_id, false_bb_id)
                            )
                        )
                    );
                    self.push_node_to(true_bb_id, branch_out_node);
                    self.push_node_to(false_bb_id, branch_out_node);

                    self.new_basic_block();
                }
            }
        } else {
            todo!("If exprs without false branch are not implemented yet");
        }

        // self.building_basic_block.push_stmt(
        //     Stmt::new(
        //         StmtKind::BranchNode(BranchNode::new(cond, BasicBlockId(true_bb_id), BasicBlockId))
        //     )
        // );

        let result_place = TempId(self.get_next_ssa_id());
        self.push_node(
            Node::new(
                NodeKind::LoadNode(
                    LoadNode::new(result_place, Either::Right(result_mem_id), if_expr_ty)
                )
            )
        );

        Operand::new(OperandKind::from(result_place), if_expr_ty)
    }

    fn visit_assign_stmt(&mut self, assign_stmt: &'ast ast::AssignStmt<'ast>) -> Self::Result {
        let (local_mem_id, ty) = match &assign_stmt.setter_expr.kind {
            PlaceKind::IdentExpr(ident_expr) => {
                let ty = self.icfg_builder.get_ty_from_node_id(ident_expr.ast_node_id);
                let def_id = self.icfg_builder.get_def_id_from_node_id(ident_expr.ast_node_id);
                let local_mem_id = self.get_local_mem_id_from_def_id(def_id);
                (local_mem_id, ty)
            }
        };

        let result_operand = self.visit_expr(assign_stmt.value_expr);

        self.push_node(
            Node::new(
                NodeKind::StoreNode(StoreNode::new(Either::Left(local_mem_id), ty, result_operand))
            )
        );

        Self::default_result()
    }

    fn visit_def_stmt(&mut self, def_stmt: &'ast ast::DefineStmt<'ast>) -> Self::Result {
        let (local_mem, local_mem_id, ty, def_id) = match &def_stmt.setter_expr.kind {
            PatKind::IdentPat(ident_pat) => {
                let ty = self.icfg_builder.get_ty_from_node_id(def_stmt.value_expr.ast_node_id);
                let local_mem_id = LocalMemId(self.local_mems.len() as u32);
                let local_mem = LocalMem::new(
                    local_mem_id,
                    Symbol::new(&self.icfg_builder.src[ident_pat.span.get_byte_range()]),
                    ident_pat.span,
                    ty,
                    Mutability::Immutable
                );
                self.local_mems.push(local_mem);
                let def_id = self.icfg_builder.get_def_id_from_node_id(ident_pat.ast_node_id);
                (local_mem, local_mem_id, ty, def_id)
            }
        };

        let result_operand = self.visit_expr(def_stmt.value_expr);

        self.set_def_id_to_local_mem_id(def_id, local_mem_id);

        self.push_node(
            Node::new(
                NodeKind::StoreNode(StoreNode::new(Either::Left(local_mem_id), ty, result_operand))
            )
        );

        Self::default_result()
    }

    fn visit_binary_expr(&mut self, binary_expr: &'ast ast::BinaryExpr<'ast>) -> Self::Result {
        let lhs = self.visit_expr(binary_expr.lhs);
        let rhs = self.visit_expr(binary_expr.rhs);

        let op_ty = self.icfg_builder.get_ty_from_node_id(binary_expr.lhs.ast_node_id);
        let result_ty = self.icfg_builder.get_ty_from_node_id(binary_expr.ast_node_id);

        let result_loc = TempId(self.get_next_ssa_id() as u32);

        self.push_node(
            Node::new(
                NodeKind::BinaryNode(
                    BinaryNode::new(result_loc, result_ty, op_ty, binary_expr.op, lhs, rhs)
                )
            )
        );

        Operand::new(OperandKind::Place(result_loc), result_ty)
    }
}
