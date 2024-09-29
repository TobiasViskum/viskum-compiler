use ast::{
    get_node_id_from_expr,
    Ast,
    AstTypeChecked,
    FunctionStmt,
    IfExpr,
    IfFalseBranchExpr,
    Pat,
    PlaceExpr,
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
    ByteAccessNode,
    Cfg,
    Icfg,
    IndexNode,
    LoadNode,
    LocalMem,
    LocalMemId,
    Node,
    NodeKind,
    Operand,
    OperandKind,
    PlaceKind,
    ResultMem,
    ResultMemId,
    StoreNode,
    TempId,
};
use ir_defs::{ DefId, Mutability, NodeId, ResultLoc };
use resolver::ResolvedInformation;
use symbol::Symbol;
use ty::{ GetTyAttr, PrimTy, Ty, TyCtx, VOID_TY };

pub struct IcfgBuilder<'ast> {
    pending_functions: Vec<&'ast FunctionStmt<'ast>>,
    ast: Ast<'ast, AstTypeChecked>,
    cfgs: Vec<Cfg>,
    resolved_information: ResolvedInformation,
    src: &'ast str,
}

impl<'ast> IcfgBuilder<'ast> {
    pub fn new(
        ast: Ast<'ast, AstTypeChecked>,
        resolved_information: ResolvedInformation,
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

    pub fn build(mut self) -> Icfg {
        let cfg_builder = self.new_cfg_builder();
        let cfg = cfg_builder.build_cfg();
        self.cfgs.push(cfg);

        Icfg::new(self.cfgs)
    }

    pub(crate) fn new_cfg_builder<'c>(&'c mut self) -> CfgBuilder<'ast, 'c> {
        CfgBuilder::new(self)
    }

    pub(crate) fn append_function(&mut self, function: &'ast FunctionStmt<'ast>) {
        self.pending_functions.push(function);
    }

    pub(crate) fn get_ty_from_node_id(&self, node_id: NodeId) -> Ty {
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

pub struct CfgBuilder<'ast, 'c> {
    icfg_builder: &'c mut IcfgBuilder<'ast>,

    /* These is transfered over to the cfg */
    local_mems: Vec<LocalMem>,
    result_mems: Vec<ResultMem>,
    basic_blocks: Vec<BasicBlock>,

    def_id_to_local_mem_id: FxHashMap<DefId, LocalMemId>,
    result_loc_to_result_mem_id: FxHashMap<ResultLoc, ResultMemId>,

    /* For loops */
    break_bb_ids: Vec<(BasicBlockId, Option<Operand>)>,
    continue_bb_ids: Vec<BasicBlockId>,

    // entry_function: &'ast FunctionStmt<'ast>,
    next_ssa_id: u32,
}

impl<'ast, 'c> CfgBuilder<'ast, 'c> {
    pub fn new(
        icfg_builder: &'c mut IcfgBuilder<'ast>
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
            break_bb_ids: Default::default(),
            continue_bb_ids: Default::default(),
            // entry_function,
            next_ssa_id: 0,
        }
    }

    pub fn build_cfg(mut self) -> Cfg {
        let ast = self.icfg_builder.get_ast();
        self.visit_stmts(ast.main_scope.stmts);

        // self.push_stmt(Stmt::new(kind));

        Cfg::new(self.local_mems, self.result_mems, self.basic_blocks)
    }

    // pub(crate) fn get_ty(ty: Ty) -> Ty {
    //     TyCtx.intern_type(ty)
    // }

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
        make_result_mem_ty: impl Fn() -> Ty
    ) -> ResultMemId {
        if let Some(result_mem_id) = self.result_loc_to_result_mem_id.get(&result_loc) {
            *result_mem_id
        } else {
            let result_mem_ty = make_result_mem_ty();
            let result_mem_id = ResultMemId(self.result_mems.len() as u32);
            let result_mem = ResultMem::new(result_mem_id, result_mem_ty);
            self.result_mems.push(result_mem);
            self.result_loc_to_result_mem_id.insert(result_loc, result_mem_id);
            result_mem_id
        }
    }

    pub(crate) fn get_curr_bb_id(&self) -> BasicBlockId {
        BasicBlockId((self.basic_blocks.len() - 1) as u32)
    }

    pub(crate) fn get_next_bb_id(&self) -> BasicBlockId {
        BasicBlockId(self.basic_blocks.len() as u32)
    }

    pub(crate) fn new_basic_block(&mut self) -> BasicBlockId {
        let bb_id = BasicBlockId(self.basic_blocks.len() as u32);
        self.basic_blocks.push(BasicBlock::new(bb_id));
        bb_id
    }

    pub(crate) fn push_node(&mut self, node: Node) {
        let bb_id = BasicBlockId((self.basic_blocks.len() - 1) as u32);
        self.push_node_to(bb_id, node);
    }

    pub(crate) fn push_node_to(&mut self, basic_block_id: BasicBlockId, node: Node) {
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

impl<'ast, 'c> Visitor<'ast> for CfgBuilder<'ast, 'c> {
    type Result = Operand;

    fn default_result() -> Self::Result {
        Operand::new(Default::default(), VOID_TY)
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

    fn visit_if_expr(&mut self, if_expr: &'ast IfExpr<'ast>) -> Self::Result {
        let if_expr_ty = self.icfg_builder.get_ty_from_node_id(if_expr.ast_node_id);

        let cond_ty = self.icfg_builder.get_ty_from_node_id(
            get_node_id_from_expr(if_expr.condition)
        );

        let result_mem_id = if !if_expr_ty.is_void() {
            let result_mem_id = self.set_result_loc_to_local_mem(if_expr.result_loc, || if_expr_ty);
            Some(result_mem_id)
        } else {
            None
        };

        let bb_id_before_if_expr = BasicBlockId((self.basic_blocks.len() - 1) as u32);
        let cond = self.visit_expr(if_expr.condition);

        let (first_true_bb_id, last_true_bb_id) = {
            let if_expr_ty = self.icfg_builder.get_ty_from_node_id(if_expr.ast_node_id);
            let true_bb_id = self.new_basic_block();
            let true_operand = self.visit_block_expr(if_expr.true_block);
            if let Some(result_mem_id) = result_mem_id {
                self.push_node(
                    Node::new(
                        NodeKind::StoreNode(
                            StoreNode::new(
                                PlaceKind::ResultMemId(result_mem_id),
                                if_expr_ty,
                                true_operand
                            )
                        )
                    )
                );
            }
            (true_bb_id, self.get_curr_bb_id())
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
                        get_node_id_from_expr(elif_expr.condition)
                    );
                    self.push_node_to(
                        bb_id_before_if_expr,
                        Node::new(
                            NodeKind::BranchCondNode(
                                BranchCondNode::new(cond, cond_ty, first_true_bb_id, false_bb_id)
                            )
                        )
                    );
                    self.push_node_to(last_true_bb_id, branch_out_node);
                    self.push_node_to(false_bb_id, branch_out_node);
                }
                IfFalseBranchExpr::ElseExpr(else_expr) => {
                    let false_operand = self.visit_block_expr(else_expr);
                    if let Some(result_mem_id) = result_mem_id {
                        self.push_node(
                            Node::new(
                                NodeKind::StoreNode(
                                    StoreNode::new(
                                        PlaceKind::ResultMemId(result_mem_id),
                                        if_expr_ty,
                                        false_operand
                                    )
                                )
                            )
                        );
                    }
                    self.push_node_to(
                        bb_id_before_if_expr,
                        Node::new(
                            NodeKind::BranchCondNode(
                                BranchCondNode::new(cond, cond_ty, first_true_bb_id, false_bb_id)
                            )
                        )
                    );
                    self.push_node_to(last_true_bb_id, branch_out_node);
                    self.push_node_to(false_bb_id, branch_out_node);

                    self.new_basic_block();
                }
            }
        } else {
            let bb_id_after_true_expr = BasicBlockId(last_true_bb_id.0 + 1);
            self.push_node(Node::new(NodeKind::BranchNode(BranchNode::new(bb_id_after_true_expr))));
            self.push_node_to(
                bb_id_before_if_expr,
                Node::new(
                    NodeKind::BranchCondNode(
                        BranchCondNode::new(cond, cond_ty, first_true_bb_id, bb_id_after_true_expr)
                    )
                )
            );
            self.new_basic_block();
        }

        if let Some(result_mem_id) = result_mem_id {
            let result_place = TempId(self.get_next_ssa_id());
            self.push_node(
                Node::new(
                    NodeKind::LoadNode(
                        LoadNode::new(result_place, Either::Right(result_mem_id), if_expr_ty)
                    )
                )
            );
            Operand::new(OperandKind::from(result_place), if_expr_ty)
        } else {
            Self::default_result()
        }
    }

    fn visit_loop_expr(&mut self, loop_expr: &'ast ast::LoopExpr<'ast>) -> Self::Result {
        let loop_expr_ty = self.icfg_builder.get_ty_from_node_id(loop_expr.ast_node_id);

        let result_mem_id = if !loop_expr_ty.is_void() {
            let result_mem_id = self.set_result_loc_to_local_mem(
                loop_expr.result_loc,
                || loop_expr_ty
            );
            Some(result_mem_id)
        } else {
            None
        };

        let prev_break_bb_ids = std::mem::replace(&mut self.break_bb_ids, vec![]);
        let prev_continue_bb_ids = std::mem::replace(&mut self.continue_bb_ids, vec![]);

        // Pushes branch node to loop, to the BasicBlock before the loop
        let loop_bb_id = self.get_next_bb_id();
        self.push_node(Node::new(NodeKind::BranchNode(BranchNode::new(loop_bb_id))));
        self.new_basic_block();

        self.visit_block_expr(loop_expr.body);

        // Pushes branch to start of loop
        self.push_node(Node::new(NodeKind::BranchNode(BranchNode::new(loop_bb_id))));
        self.new_basic_block();
        let after_loop_bb_id = self.get_curr_bb_id();

        let break_bb_ids = std::mem::replace(&mut self.break_bb_ids, prev_break_bb_ids);
        let continue_bb_ids = std::mem::replace(&mut self.continue_bb_ids, prev_continue_bb_ids);

        for (break_bb_id, break_operand) in break_bb_ids {
            if let Some(break_operand) = break_operand {
                // Push Store node
                panic!("Break with value is not yet supported");
            }
            self.push_node_to(
                break_bb_id,
                Node::new(NodeKind::BranchNode(BranchNode::new(after_loop_bb_id)))
            );
        }
        for continue_bb_id in continue_bb_ids {
            self.push_node_to(
                continue_bb_id,
                Node::new(NodeKind::BranchNode(BranchNode::new(loop_bb_id)))
            );
        }

        if !loop_expr_ty.is_void() {
            let result_place = TempId(self.get_next_ssa_id());
            // self.push_node(
            //     Node::new(
            //         NodeKind::LoadNode(
            //             LoadNode::new(result_place, Either::Right(result_mem_id), loop_expr_ty)
            //         )
            //     )
            // );
            Operand::new(OperandKind::from(result_place), loop_expr_ty)
        } else {
            Self::default_result()
        }
    }

    fn visit_break_expr(&mut self, break_expr: &'ast ast::BreakExpr<'ast>) -> Self::Result {
        self.break_bb_ids.push((self.get_curr_bb_id(), None));
        self.new_basic_block();

        // Break expressions always return void
        Self::default_result()
    }

    fn visit_continue_expr(&mut self, continue_expr: &'ast ast::ContinueExpr) -> Self::Result {
        self.continue_bb_ids.push(self.get_curr_bb_id());
        self.new_basic_block();

        // Continue expressions always return void
        Self::default_result()
    }

    /// Assumes read for now
    /// IMPLEMENT: GET Result loc for tuples, if_exprs, loop_exprs etc.
    fn visit_tuple_field_expr(
        &mut self,
        tuple_field_expr: &'ast ast::TupleFieldExpr<'ast>
    ) -> Self::Result {
        let lhs = self.visit_expr(tuple_field_expr.lhs);
        let tuple_ty = match
            self.icfg_builder.get_ty_from_node_id(get_node_id_from_expr(tuple_field_expr.lhs))
        {
            Ty::Tuple(tuple_ty) => tuple_ty,
            _ => unreachable!("Should not be able to go here if previous pass was successfull"),
        };

        let temp_id = TempId(self.get_next_ssa_id());

        let idx = tuple_field_expr.rhs.val as usize;
        let elem_type = tuple_ty[idx];

        // if idx == 0 {
        //     self.push_node(Node::new(NodeKind::LoadNode(LoadNode::new(temp_id, loc_id, ty))));
        // } else {
        //     self.push_node();
        // }

        panic!("sdfsf");

        Operand::new(OperandKind::Place(temp_id), elem_type)
    }

    fn visit_assign_stmt(&mut self, assign_stmt: &'ast ast::AssignStmt<'ast>) -> Self::Result {
        let (local_mem_id, ty) = match assign_stmt.setter_expr {
            PlaceExpr::IdentExpr(ident_expr) => {
                let ty = self.icfg_builder.get_ty_from_node_id(ident_expr.ast_node_id);
                let def_id = self.icfg_builder.get_def_id_from_node_id(ident_expr.ast_node_id);
                let local_mem_id = self.get_local_mem_id_from_def_id(def_id);
                (local_mem_id, ty)
            }
            PlaceExpr::TupleFieldExpr(_) => panic!("Tuple field expr not yet supported"),
        };

        let result_operand = self.visit_expr(assign_stmt.value_expr);

        self.push_node(
            Node::new(
                NodeKind::StoreNode(
                    StoreNode::new(PlaceKind::LocalMemId(local_mem_id), ty, result_operand)
                )
            )
        );

        Self::default_result()
    }

    fn visit_def_stmt(&mut self, def_stmt: &'ast ast::DefineStmt<'ast>) -> Self::Result {
        let ty = self.icfg_builder.get_ty_from_node_id(get_node_id_from_expr(def_stmt.value_expr));

        if ty.is_void() {
            return Self::default_result();
        }

        let (local_mem_id, def_id) = match def_stmt.setter_expr {
            Pat::IdentPat(ident_pat) => {
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
                (local_mem_id, def_id)
            }
        };

        let result_operand = self.visit_expr(def_stmt.value_expr);

        self.set_def_id_to_local_mem_id(def_id, local_mem_id);

        self.push_node(
            Node::new(
                NodeKind::StoreNode(
                    StoreNode::new(PlaceKind::LocalMemId(local_mem_id), ty, result_operand)
                )
            )
        );

        Self::default_result()
    }

    fn visit_tuple_expr(&mut self, tuple_expr: &'ast ast::TupleExpr<'ast>) -> Self::Result {
        let tuple_ty = self.icfg_builder.get_ty_from_node_id(tuple_expr.ast_node_id);
        let result_mem_id = ResultMemId(self.result_mems.len() as u32);
        self.result_mems.push(ResultMem::new(result_mem_id, tuple_ty));

        let mut byte_offset: usize = 0;

        for expr in tuple_expr.fields.iter() {
            let operand = self.visit_expr(*expr);

            if byte_offset == 0 {
                self.push_node(
                    Node::new(
                        NodeKind::StoreNode(
                            StoreNode::new(
                                PlaceKind::ResultMemId(result_mem_id),
                                operand.ty,
                                operand
                            )
                        )
                    )
                );
            } else {
                let temp_id = TempId(self.get_next_ssa_id() as u32);
                self.push_node(
                    Node::new(
                        NodeKind::ByteAccessNode(
                            ByteAccessNode::new(
                                temp_id,
                                PlaceKind::ResultMemId(result_mem_id),
                                byte_offset
                            )
                        )
                    )
                );
                self.push_node(
                    Node::new(
                        NodeKind::StoreNode(
                            StoreNode::new(PlaceKind::TempId(temp_id), operand.ty, operand)
                        )
                    )
                );
            }

            let operand_ty_attr = operand.ty.get_size_and_alignment();
            byte_offset += operand_ty_attr.size_bytes;
        }

        let temp_id = TempId(self.get_next_ssa_id() as u32);
        self.push_node(
            Node::new(
                NodeKind::LoadNode(LoadNode::new(temp_id, Either::Right(result_mem_id), tuple_ty))
            )
        );

        Operand::new(OperandKind::Place(temp_id), tuple_ty)
    }

    fn visit_binary_expr(&mut self, binary_expr: &'ast ast::BinaryExpr<'ast>) -> Self::Result {
        let lhs = self.visit_expr(binary_expr.lhs);
        let rhs = self.visit_expr(binary_expr.rhs);

        let op_ty = self.icfg_builder.get_ty_from_node_id(get_node_id_from_expr(binary_expr.lhs));
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
