use ast::{
    get_node_id_from_expr,
    Ast,
    AstTypeChecked,
    Expr,
    FnItem,
    IdentNode,
    IfExpr,
    IfFalseBranchExpr,
    LoopExpr,
    Pat,
    PlaceExpr,
    Visitor,
};

use fxhash::FxHashMap;
use icfg::{
    BasicBlock,
    BasicBlockId,
    BinaryNode,
    BranchCondNode,
    BranchNode,
    ByteAccessNode,
    Cfg,
    Const,
    Icfg,
    LoadNode,
    LocalMem,
    LocalMemId,
    Node,
    NodeKind,
    Operand,
    PlaceKind,
    ResultMem,
    ResultMemId,
    StoreKind,
    StoreNode,
    TempId,
};
use ir_defs::{ DefId, Mutability, NodeId };
use resolver::ResolvedInformation;
use symbol::Symbol;
use ty::{ GetTyAttr, Ty, VOID_TY };

pub struct IcfgBuilder<'ast> {
    pending_functions: Vec<&'ast FnItem<'ast>>,
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

    pub(crate) fn append_function(&mut self, function: &'ast FnItem<'ast>) {
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

pub struct CfgBuilder<'ast, 'b> {
    icfg_builder: &'b mut IcfgBuilder<'ast>,

    /* These is transfered over to the cfg */
    local_mems: Vec<LocalMem>,
    result_mems: Vec<ResultMem>,
    basic_blocks: Vec<BasicBlock>,

    def_id_to_local_mem_id: FxHashMap<DefId, LocalMemId>,
    node_id_to_result_mem_id: FxHashMap<NodeId, ResultMemId>,

    /* For loops */
    break_bb_ids: Vec<(BasicBlockId, Option<Operand>)>,
    continue_bb_ids: Vec<BasicBlockId>,

    // entry_function: &'ast FnItem<'ast>,
    next_ssa_id: u32,
}

impl<'ast, 'b> CfgBuilder<'ast, 'b> {
    pub fn new(
        icfg_builder: &'b mut IcfgBuilder<'ast>
        // entry_function: &'ast FnItem<'ast>
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
            node_id_to_result_mem_id: Default::default(),
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

    pub(crate) fn get_temp_id(&mut self) -> TempId {
        TempId(self.get_next_ssa_id())
    }

    pub(crate) fn get_local_mem_id_from_def_id(&self, def_id: DefId) -> LocalMemId {
        *self.def_id_to_local_mem_id.get(&def_id).expect("Expected LocalMem from DefId")
    }

    pub(crate) fn set_def_id_to_local_mem_id(&mut self, def_id: DefId, local_mem_id: LocalMemId) {
        self.def_id_to_local_mem_id.insert(def_id, local_mem_id);
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

    pub(crate) fn new_result_mem(&mut self, ty: Ty) -> ResultMemId {
        let result_mem_id = ResultMemId(self.result_mems.len() as u32);
        self.result_mems.push(ResultMem::new(result_mem_id, ty));
        result_mem_id
    }

    pub(crate) fn set_result_mem_id_to_expr_result(
        &mut self,
        node_id: NodeId,
        ty: Ty
    ) -> ResultMemId {
        if let Some(result_mem_id) = self.node_id_to_result_mem_id.get(&node_id) {
            *result_mem_id
        } else {
            let result_mem_id = self.new_result_mem(ty);
            self.node_id_to_result_mem_id.insert(node_id, result_mem_id);
            result_mem_id
        }
    }

    /// This function takes a VisitResult and tries to get a value from it.
    ///
    /// It will as insert many load (deref) instructions as needed to use it as a value
    pub(crate) fn get_operand_from_visit_result(
        &mut self,
        visit_result: VisitResult
    ) -> (Operand, Ty) {
        match visit_result {
            VisitResult::PlaceKind(place_kind, place_ty) => {
                let (temp_id, ty) = match place_kind {
                    PlaceKind::TempId(temp_id) => (temp_id, place_ty),
                    // Now we have to load, because it refers to a stack allocated place
                    required_load_place => {
                        let temp_id = self.get_temp_id();
                        let new_ty = place_ty.try_deref_once();

                        if let Some(new_ty) = new_ty {
                            self.push_node(
                                Node::new(
                                    NodeKind::LoadNode(
                                        LoadNode::new(temp_id, required_load_place, new_ty)
                                    )
                                )
                            );
                            (temp_id, new_ty)
                        } else {
                            panic!("Stack allocated space should be able to be dereffed")
                        }
                    }
                };

                let (mut temp_id, mut ty) = (temp_id, ty);

                loop {
                    if let Ty::Ptr(inner_ty) = ty {
                        let new_temp_id = self.get_temp_id();
                        self.push_node(
                            Node::new(
                                NodeKind::LoadNode(
                                    LoadNode::new(
                                        new_temp_id,
                                        PlaceKind::TempId(temp_id),
                                        *inner_ty
                                    )
                                )
                            )
                        );
                        temp_id = new_temp_id;
                        ty = *inner_ty;
                        continue;
                    }
                    break;
                }

                (Operand::TempId(temp_id), ty)
            }
            VisitResult::Const(const_val) => (Operand::Const(const_val), const_val.get_ty()),
        }
    }
}

#[derive(Debug)]
pub enum VisitResult {
    PlaceKind(PlaceKind, Ty),
    Const(Const),
}

impl VisitResult {
    pub fn get_ty(&self) -> Ty {
        match self {
            Self::PlaceKind(_, ty) => *ty,
            Self::Const(const_val) => const_val.get_ty(),
        }
    }
}

impl Default for VisitResult {
    fn default() -> Self {
        Self::Const(Const::Void)
    }
}

impl<'ast, 'c> Visitor<'ast> for CfgBuilder<'ast, 'c> {
    /// This is a kind of "lazy-load" result. For example when visiting a variable, it just returns the place it lives in.
    /// It doesn't add the load node before requesting so with the method `get_operand_from_visit_result`.
    /// This useful because for example in a field expr, we don't necessarily want to add a load node (we may want to add a ByteAccessNode instead)
    type Result = VisitResult;

    fn default_result() -> Self::Result {
        Default::default()
    }

    fn visit_interger_expr(&mut self, integer_expr: &'ast ast::IntegerExpr) -> Self::Result {
        VisitResult::Const(Const::Int(integer_expr.val))
    }

    fn visit_bool_expr(&mut self, bool_expr: &'ast ast::BoolExpr) -> Self::Result {
        VisitResult::Const(Const::Bool(bool_expr.val))
    }

    fn visit_ident_expr(&mut self, ident_node: &'ast IdentNode) -> Self::Result {
        let def_id = self.icfg_builder.get_def_id_from_node_id(ident_node.ast_node_id);
        let ty = self.icfg_builder.get_ty_from_node_id(ident_node.ast_node_id);
        let local_mem_id = self.get_local_mem_id_from_def_id(def_id);

        VisitResult::PlaceKind(PlaceKind::LocalMemId(local_mem_id), ty)
    }

    fn visit_if_expr(&mut self, if_expr: &'ast IfExpr<'ast>) -> Self::Result {
        fn set_result_mem_id_to_if_expr_result<'ast, 'b>(
            cfg_builder: &mut CfgBuilder<'ast, 'b>,
            if_expr: &'ast IfExpr<'ast>
        ) -> ResultMemId {
            let result_mem_id = cfg_builder.set_result_mem_id_to_expr_result(
                if_expr.ast_node_id,
                cfg_builder.icfg_builder.get_ty_from_node_id(if_expr.ast_node_id)
            );

            if let Some(IfFalseBranchExpr::ElifExpr(elif_expr)) = if_expr.false_block {
                cfg_builder.node_id_to_result_mem_id.insert(elif_expr.ast_node_id, result_mem_id);
            }

            result_mem_id
        }

        let if_expr_ty = self.icfg_builder.get_ty_from_node_id(if_expr.ast_node_id).to_ptr_ty();

        let result_mem_id = if !if_expr_ty.is_void() {
            Some(set_result_mem_id_to_if_expr_result(self, if_expr))
        } else {
            None
        };

        let bb_id_before_if_expr = BasicBlockId((self.basic_blocks.len() - 1) as u32);

        let (cond_operand, _) = {
            let cond_visit_result = self.visit_expr(if_expr.condition);
            self.get_operand_from_visit_result(cond_visit_result)
        };

        let (first_true_bb_id, last_true_bb_id) = {
            let true_bb_id = self.new_basic_block();
            let true_visit_result = self.visit_block_expr(if_expr.true_block);
            if let Some(result_mem_id) = result_mem_id {
                let (true_operand, true_ty) = self.get_operand_from_visit_result(true_visit_result);

                self.push_node(
                    Node::new(
                        NodeKind::StoreNode(
                            StoreNode::new(
                                PlaceKind::ResultMemId(result_mem_id),
                                true_ty,
                                true_operand,
                                StoreKind::Init
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
                    // let cond_ty = self.icfg_builder.get_ty_from_node_id(
                    //     get_node_id_from_expr(elif_expr.condition)
                    // );
                    self.push_node_to(
                        bb_id_before_if_expr,
                        Node::new(
                            NodeKind::BranchCondNode(
                                BranchCondNode::new(cond_operand, first_true_bb_id, false_bb_id)
                            )
                        )
                    );
                    self.push_node_to(last_true_bb_id, branch_out_node);
                    self.push_node_to(false_bb_id, branch_out_node);
                }
                IfFalseBranchExpr::ElseExpr(else_expr) => {
                    let false_visit_result = self.visit_block_expr(else_expr);
                    let (false_operand, false_ty) =
                        self.get_operand_from_visit_result(false_visit_result);
                    if let Some(result_mem_id) = result_mem_id {
                        self.push_node(
                            Node::new(
                                NodeKind::StoreNode(
                                    StoreNode::new(
                                        PlaceKind::ResultMemId(result_mem_id),
                                        false_ty,
                                        false_operand,
                                        StoreKind::Init
                                    )
                                )
                            )
                        );
                    }
                    self.push_node_to(
                        bb_id_before_if_expr,
                        Node::new(
                            NodeKind::BranchCondNode(
                                BranchCondNode::new(cond_operand, first_true_bb_id, false_bb_id)
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
                        BranchCondNode::new(cond_operand, first_true_bb_id, bb_id_after_true_expr)
                    )
                )
            );
            self.new_basic_block();
        }

        if let Some(result_mem_id) = result_mem_id {
            VisitResult::PlaceKind(PlaceKind::ResultMemId(result_mem_id), if_expr_ty)
        } else {
            Self::default_result()
        }
    }

    fn visit_loop_expr(&mut self, loop_expr: &'ast LoopExpr<'ast>) -> Self::Result {
        fn set_result_mem_id_to_loop_expr_result<'ast, 'b>(
            cfg_builder: &mut CfgBuilder<'ast, 'b>,
            loop_expr: &'ast LoopExpr<'ast>
        ) -> ResultMemId {
            cfg_builder.set_result_mem_id_to_expr_result(
                loop_expr.ast_node_id,
                cfg_builder.icfg_builder.get_ty_from_node_id(loop_expr.ast_node_id)
            )
        }

        let loop_expr_ty = self.icfg_builder.get_ty_from_node_id(loop_expr.ast_node_id).to_ptr_ty();

        let result_mem_id = if !loop_expr_ty.is_void() {
            Some(set_result_mem_id_to_loop_expr_result(self, loop_expr))
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

        if let Some(result_mem_id) = result_mem_id {
            VisitResult::PlaceKind(PlaceKind::ResultMemId(result_mem_id), loop_expr_ty)
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
    fn visit_tuple_field_expr(
        &mut self,
        tuple_field_expr: &'ast ast::TupleFieldExpr<'ast>
    ) -> Self::Result {
        // This will always result in a plac
        let lhs_visit_result = self.visit_expr(tuple_field_expr.lhs);
        let lhs_place = match lhs_visit_result {
            VisitResult::PlaceKind(place_kind, _) => place_kind,
            _ => unreachable!("This should be unreachable if type checking was successful"),
        };

        let tuple_ty = match
            self.icfg_builder
                .get_ty_from_node_id(get_node_id_from_expr(tuple_field_expr.lhs))
                .try_deref_as_tuple()
        {
            Some(tuple_ty) => tuple_ty,
            None => unreachable!("Should not be able to go here if previous pass was successfull"),
        };

        let idx = tuple_field_expr.rhs.val as usize;

        // We make it a pointer, because when accessing a field of a tuple, we are doing a
        let elem_type = tuple_ty[idx].to_ptr_ty();

        if idx == 0 {
            VisitResult::PlaceKind(lhs_place, elem_type)
        } else {
            let mut byte_offset = 0;
            for (i, ty) in tuple_ty.iter().enumerate() {
                byte_offset += ty.get_ty_attr().size_bytes;

                if i == idx - 1 {
                    break;
                }
            }

            let temp_id = self.get_temp_id();
            self.push_node(
                Node::new(
                    NodeKind::ByteAccessNode(
                        ByteAccessNode::new(PlaceKind::TempId(temp_id), lhs_place, byte_offset)
                    )
                )
            );

            VisitResult::PlaceKind(PlaceKind::TempId(temp_id), elem_type)
        }
    }

    /// Assumes read for now
    fn visit_field_expr(&mut self, field_expr: &'ast ast::FieldExpr<'ast>) -> Self::Result {
        let lhs_visit_result = self.visit_expr(field_expr.lhs);
        let lhs_place = match lhs_visit_result {
            VisitResult::PlaceKind(place_kind, _) => place_kind,
            _ => unreachable!("This should be unreachable if type checking was successful"),
        };

        let access_symbol = Symbol::new(
            &self.icfg_builder.src[field_expr.rhs.span.get_byte_range()]
        );

        let (struct_name, struct_fields) = match
            self.icfg_builder
                .get_ty_from_node_id(get_node_id_from_expr(field_expr.lhs))
                .try_deref_as_struct()
        {
            Some(struct_ty) => struct_ty,
            None => unreachable!("Should not be able to go here if previous pass was successfull"),
        };

        let mut byte_offset = 0;
        let elem_ty = struct_fields
            .iter()
            .find(|(symbol, ty)| (
                if symbol.get() == access_symbol.get() {
                    true
                } else {
                    byte_offset += ty.get_ty_attr().size_bytes;
                    false
                }
            ))
            .map(|x| x.1)
            .expect("Expected field")
            .to_ptr_ty();

        if byte_offset == 0 {
            VisitResult::PlaceKind(lhs_place, elem_ty)
        } else {
            let temp_id = self.get_temp_id();
            self.push_node(
                Node::new(
                    NodeKind::ByteAccessNode(
                        ByteAccessNode::new(PlaceKind::TempId(temp_id), lhs_place, byte_offset)
                    )
                )
            );

            VisitResult::PlaceKind(PlaceKind::TempId(temp_id), elem_ty)
        }
    }

    fn visit_assign_stmt(&mut self, assign_stmt: &'ast ast::AssignStmt<'ast>) -> Self::Result {
        let local_mem_id = match assign_stmt.setter_expr {
            PlaceExpr::IdentExpr(ident_expr) => {
                // let ty = self.icfg_builder.get_ty_from_node_id(ident_expr.ast_node_id);
                let def_id = self.icfg_builder.get_def_id_from_node_id(ident_expr.ast_node_id);
                let local_mem_id = self.get_local_mem_id_from_def_id(def_id);
                local_mem_id
            }
            PlaceExpr::TupleFieldExpr(_) => panic!("Tuple field expr not yet supported"),
            PlaceExpr::FieldExpr(_) => panic!("Field expr not yet supported"),
        };

        let value_visit_result = self.visit_expr(assign_stmt.value_expr);
        let (operand, op_ty) = self.get_operand_from_visit_result(value_visit_result);

        self.push_node(
            Node::new(
                NodeKind::StoreNode(
                    StoreNode::new(
                        PlaceKind::LocalMemId(local_mem_id),
                        op_ty,
                        operand,
                        StoreKind::Assign
                    )
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

        let value_visit_result = self.visit_expr(def_stmt.value_expr);

        let (operand, op_ty) = self.get_operand_from_visit_result(value_visit_result);

        self.set_def_id_to_local_mem_id(def_id, local_mem_id);

        self.push_node(
            Node::new(
                NodeKind::StoreNode(
                    StoreNode::new(
                        PlaceKind::LocalMemId(local_mem_id),
                        op_ty,
                        operand,
                        StoreKind::Init
                    )
                )
            )
        );

        Self::default_result()
    }

    fn visit_struct_expr(&mut self, struct_expr: &'ast ast::StructExpr<'ast>) -> Self::Result {
        let struct_ty = self.icfg_builder.get_ty_from_node_id(struct_expr.ast_node_id);
        let result_mem_id = self.new_result_mem(struct_ty);
        self.node_id_to_result_mem_id.insert(struct_expr.ast_node_id, result_mem_id);

        let mut byte_offset: usize = 0;

        let (_, struct_fields) = self.icfg_builder
            .get_ty_from_node_id(struct_expr.ast_node_id)
            .try_deref_as_struct()
            .expect("Expected ty to be struct");

        'outer: for (field_symbol, _) in struct_fields {
            for field in struct_expr.field_initializations.iter() {
                let access_field_symbol = Symbol::new(
                    &self.icfg_builder.src[field.ident.span.get_byte_range()]
                );
                if access_field_symbol.get() == field_symbol.get() {
                    byte_offset = self.init_tuple_or_struct_field(
                        field.value,
                        result_mem_id,
                        byte_offset
                    );
                    continue 'outer;
                }
            }
            unreachable!("Should not be hit if validation of the Ast was correct");
        }

        VisitResult::PlaceKind(PlaceKind::ResultMemId(result_mem_id), struct_ty.to_ptr_ty())
    }

    fn visit_tuple_expr(&mut self, tuple_expr: &'ast ast::TupleExpr<'ast>) -> Self::Result {
        let tuple_ty = self.icfg_builder.get_ty_from_node_id(tuple_expr.ast_node_id);
        let result_mem_id = self.new_result_mem(tuple_ty);
        self.node_id_to_result_mem_id.insert(tuple_expr.ast_node_id, result_mem_id);

        let mut byte_offset: usize = 0;

        for expr in tuple_expr.fields.iter() {
            byte_offset = self.init_tuple_or_struct_field(*expr, result_mem_id, byte_offset);
        }

        VisitResult::PlaceKind(PlaceKind::ResultMemId(result_mem_id), tuple_ty.to_ptr_ty())
    }

    fn visit_binary_expr(&mut self, binary_expr: &'ast ast::BinaryExpr<'ast>) -> Self::Result {
        let (lhs_operand, lhs_ty) = {
            let lhs_visit_result = self.visit_expr(binary_expr.lhs);
            self.get_operand_from_visit_result(lhs_visit_result)
        };
        let (rhs_operand, rhs_ty) = {
            let rhs_visit_result = self.visit_expr(binary_expr.rhs);
            self.get_operand_from_visit_result(rhs_visit_result)
        };

        let result_ty = lhs_ty.test_binary(rhs_ty, binary_expr.op).unwrap();

        let result_place = self.get_temp_id();

        self.push_node(
            Node::new(
                NodeKind::BinaryNode(
                    BinaryNode::new(result_place, lhs_ty, binary_expr.op, lhs_operand, rhs_operand)
                )
            )
        );

        VisitResult::PlaceKind(PlaceKind::TempId(result_place), result_ty)
    }
}

impl<'ast, 'b> CfgBuilder<'ast, 'b> {
    fn init_tuple_or_struct_field(
        &mut self,
        expr: Expr<'ast>,
        result_mem_id: ResultMemId,
        byte_offset: usize
    ) -> usize {
        let visit_result = self.visit_expr(expr);

        let (operand, operand_ty) = self.get_operand_from_visit_result(visit_result);

        if byte_offset == 0 {
            self.push_node(
                Node::new(
                    NodeKind::StoreNode(
                        StoreNode::new(
                            PlaceKind::ResultMemId(result_mem_id),
                            operand_ty,
                            operand,
                            StoreKind::Init
                        )
                    )
                )
            );
        } else {
            let temp_id = self.get_temp_id();
            self.push_node(
                Node::new(
                    NodeKind::ByteAccessNode(
                        ByteAccessNode::new(
                            PlaceKind::TempId(temp_id),
                            PlaceKind::ResultMemId(result_mem_id),
                            byte_offset
                        )
                    )
                )
            );
            self.push_node(
                Node::new(
                    NodeKind::StoreNode(
                        StoreNode::new(
                            PlaceKind::TempId(temp_id),
                            operand_ty,
                            operand,
                            StoreKind::Init
                        )
                    )
                )
            );
        }

        let operand_ty_attr = operand_ty.get_ty_attr();
        byte_offset + operand_ty_attr.size_bytes
    }
}
