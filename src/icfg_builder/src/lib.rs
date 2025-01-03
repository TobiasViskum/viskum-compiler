use std::sync::Mutex;

use ast::{
    get_ident_node_from_arg_kind,
    get_node_id_from_expr,
    AsigneeExpr,
    CallExpr,
    CondKind,
    Expr,
    FieldExpr,
    FnItem,
    IdentNode,
    IfExpr,
    IfFalseBranchExpr,
    IndexExpr,
    LoopExpr,
    NullExpr,
    Pat,
    PlaceExpr,
    ReturnExpr,
    Stmt,
    StringExpr,
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
    CallNode,
    Cfg,
    Const,
    Icfg,
    IndexNode,
    LoadNode,
    Node,
    NodeKind,
    Operand,
    PlaceKind,
    ReturnNode,
    StoreKind,
    StoreNode,
    TyCastKind,
    TyCastNode,
};

use ir::{
    Adt,
    CfgFnKind,
    DefId,
    GetTyAttr,
    HasSelfArg,
    IntTy,
    LocalMem,
    LocalMemId,
    Mutability,
    NameBindingKind,
    NodeId,
    PrimTy,
    ResolvedInformation,
    ResultMem,
    ResultMemId,
    Symbol,
    TempId,
    Ty,
    TyCtx,
    BOOL_TY,
    INT_16_TY,
    INT_32_TY,
    INT_64_TY,
    INT_8_TY,
    NEVER_TY,
    UINT_16_TY,
    UINT_32_TY,
    UINT_64_TY,
    UINT_8_TY,
    VOID_TY,
};
use op::{ BinaryOp, ComparisonOp };
use resolver::ResolvedFunctions;
use threadpool::ThreadPool;
use threadpool_scope::scope_with;

pub struct IcfgBuilder<'icfg, 'th> where 'icfg: 'th {
    cfgs: Mutex<Vec<Cfg<'icfg>>>,
    // global_mems: &'icfg RefCell<Vec<GlobalMem>>,
    resolved_information: ResolvedInformation<'icfg>,
    threadpool: &'th ThreadPool,
}

impl<'icfg, 'th> IcfgBuilder<'icfg, 'th> where 'icfg: 'th {
    pub fn new(
        resolved_information: ResolvedInformation<'icfg>,
        // global_mems: &'icfg RefCell<Vec<GlobalMem>>,
        threadpool: &'th ThreadPool
    ) -> Self {
        Self {
            cfgs: Default::default(),
            // global_mems,
            threadpool,
            resolved_information,
        }
    }

    pub fn build<'ast>(self, resolved_functions: ResolvedFunctions<'ast>) -> Icfg<'icfg> {
        let main_fn = resolved_functions.main_fn.expect("Do something if main doesn't exist");
        let cfg_builder = self.new_cfg_builder(main_fn, true);
        let cfg = cfg_builder.build_cfg();
        self.cfgs.lock().unwrap().push(cfg);

        scope_with(self.threadpool, |s| {
            for fn_item in resolved_functions.pending_functions {
                s.execute(|| {
                    let cfg_builder = self.new_cfg_builder(fn_item, false);
                    let cfg = cfg_builder.build_cfg();
                    self.cfgs.lock().unwrap().push(cfg);
                });
            }
        });

        // // TODO: Make this multi-threaded
        // for fn_item in resolved_functions.pending_functions {
        //     let cfg_builder = self.new_cfg_builder(fn_item, false);
        //     let cfg = cfg_builder.build_cfg();
        //     self.cfgs.push(cfg);
        // }

        Icfg::new(self.cfgs.into_inner().unwrap(), self.resolved_information)
    }

    pub(crate) fn new_cfg_builder<'ast, 'c>(
        &'c self,
        fn_item: &'ast FnItem<'ast>,
        is_main: bool
    ) -> CfgBuilder<'icfg, 'ast, 'c> {
        CfgBuilder::new(self, fn_item, is_main)
    }

    pub(crate) fn get_ty_from_node_id(&self, node_id: NodeId) -> Ty {
        *self.resolved_information.node_id_to_ty.get(&node_id).expect("Expected ty from node_id")
    }

    pub(crate) fn get_def_id_from_node_id(&self, node_id: NodeId) -> DefId {
        *self.resolved_information.node_id_to_def_id
            .get(&node_id)
            .expect("Expected DefId from NodeId")
    }
}

pub struct CfgBuilder<'icfg, 'ast, 'c> {
    icfg_builder: &'c IcfgBuilder<'icfg, 'c>,

    is_main_fn: bool,
    compiling_fn: &'ast FnItem<'ast>,

    /* These is transfered over to the cfg */
    args: Vec<(TempId, Ty)>,
    local_mems: Vec<LocalMem>,
    result_mems: Vec<ResultMem>,
    basic_blocks: Vec<BasicBlock<'icfg>>,

    def_id_to_local_mem_id: FxHashMap<DefId, LocalMemId>,
    node_id_to_result_mem_id: FxHashMap<NodeId, ResultMemId>,

    /* For loops */
    break_bb_ids: Vec<(BasicBlockId, Option<Operand>)>,
    continue_bb_ids: Vec<BasicBlockId>,

    next_ssa_id: u32,
}

impl<'icfg, 'ast, 'c> CfgBuilder<'icfg, 'ast, 'c> {
    pub fn new(
        icfg_builder: &'c IcfgBuilder<'icfg, 'c>,
        compiling_fn: &'ast FnItem<'ast>,
        is_main_fn: bool
    ) -> Self {
        let mut basic_blocks = Vec::with_capacity(16);
        let basic_block = BasicBlock::new(BasicBlockId(0));
        basic_blocks.push(basic_block);

        Self {
            icfg_builder,
            compiling_fn,
            is_main_fn,
            args: Vec::with_capacity(compiling_fn.args.len()),
            local_mems: Vec::with_capacity(8),
            result_mems: Vec::with_capacity(8),
            basic_blocks,
            def_id_to_local_mem_id: Default::default(),
            node_id_to_result_mem_id: Default::default(),
            break_bb_ids: Default::default(),
            continue_bb_ids: Default::default(),
            next_ssa_id: 0,
        }
    }

    pub fn build_cfg(mut self) -> Cfg<'icfg> {
        let def_id = self.icfg_builder.get_def_id_from_node_id(
            self.compiling_fn.ident_node.ast_node_id
        );

        let name_binding = self.icfg_builder.resolved_information.get_name_binding_from_def_id(
            &def_id
        );

        let ret_ty = if let NameBindingKind::Fn(fn_sig, _, _) = name_binding.kind {
            fn_sig.ret_ty
        } else {
            panic!("Expected fn")
        };

        if self.is_main_fn {
            self.visit_stmts(self.compiling_fn.body);
            Cfg::new(
                // self.icfg_builder.global_mems,
                self.args,
                self.local_mems,
                self.result_mems,
                self.basic_blocks,
                CfgFnKind::Main,
                *ret_ty
            )
        } else {
            for arg_kind in self.compiling_fn.args.iter() {
                let ident_node = get_ident_node_from_arg_kind(*arg_kind);
                let arg_ty = self.icfg_builder.get_ty_from_node_id(ident_node.ast_node_id);
                let arg_temp_id = {
                    let temp_id = self.get_temp_id();
                    self.args.push((temp_id, arg_ty));
                    temp_id
                };

                let arg_var_local_mem_id = {
                    let local_mem_id = LocalMemId(self.local_mems.len() as u32);
                    let local_mem = LocalMem::new(
                        local_mem_id,
                        Symbol::from_node_id(ident_node.ast_node_id),
                        ident_node.span,
                        arg_ty,
                        Mutability::Immutable
                    );
                    self.local_mems.push(local_mem);

                    let def_id = self.icfg_builder.get_def_id_from_node_id(ident_node.ast_node_id);

                    self.def_id_to_local_mem_id.insert(def_id, local_mem_id);
                    local_mem_id
                };

                self.push_node(
                    Node::new(
                        NodeKind::StoreNode(
                            StoreNode::new(
                                PlaceKind::LocalMemId(arg_var_local_mem_id),
                                arg_ty,
                                Operand::from(arg_temp_id),
                                StoreKind::Init
                            )
                        )
                    )
                );
            }
            self.visit_stmts(self.compiling_fn.body);

            Cfg::new(
                // self.icfg_builder.global_mems,
                self.args,
                self.local_mems,
                self.result_mems,
                self.basic_blocks,
                CfgFnKind::Fn(def_id),
                *ret_ty
            )
        }
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
        visit_result: VisitResult,
        ty_to_match: Ty
    ) -> (Operand, Option<Operand>, Ty) {
        // Number coercion
        fn number_coerceion(
            builder: &mut CfgBuilder<'_, '_, '_>,
            ty: Ty,
            ty_to_match: Ty,
            operand: Operand
        ) -> Option<(Ty, TempId)> {
            fn push_ty_cast_node(
                cfg_builder: &mut CfgBuilder<'_, '_, '_>,
                cast_kind: TyCastKind,
                ty: Ty,
                ty_to_match: Ty,
                operand: Operand
            ) -> Option<(Ty, TempId)> {
                let result_temp_id = cfg_builder.get_temp_id();
                cfg_builder.push_node(
                    Node::new(
                        NodeKind::TyCastNode(
                            TyCastNode::new(result_temp_id, cast_kind, ty, ty_to_match, operand)
                        )
                    )
                );
                Some((ty_to_match, result_temp_id))
            }

            match (ty, ty_to_match) {
                | (INT_8_TY, UINT_8_TY)
                | (UINT_8_TY, INT_8_TY)
                | (INT_16_TY, UINT_16_TY)
                | (UINT_16_TY, INT_16_TY)
                | (INT_32_TY, UINT_32_TY)
                | (UINT_32_TY, INT_32_TY)
                | (INT_64_TY, UINT_64_TY)
                | (UINT_64_TY, INT_64_TY) => {
                    let temp_id = match operand {
                        Operand::PlaceKind(PlaceKind::TempId(temp_id)) => temp_id,
                        _ => panic!("Expected TempId"),
                    };

                    Some((ty_to_match, temp_id))
                }
                (
                    INT_8_TY | UINT_8_TY,
                    INT_16_TY | INT_32_TY | INT_64_TY | UINT_16_TY | UINT_32_TY | UINT_64_TY,
                ) => {
                    push_ty_cast_node(builder, TyCastKind::Sext, ty, ty_to_match, operand)
                }
                (INT_16_TY | UINT_16_TY, INT_32_TY | INT_64_TY | UINT_32_TY | UINT_64_TY) => {
                    push_ty_cast_node(builder, TyCastKind::Sext, ty, ty_to_match, operand)
                }
                (INT_32_TY | UINT_32_TY, INT_64_TY | UINT_64_TY) => {
                    push_ty_cast_node(builder, TyCastKind::Sext, ty, ty_to_match, operand)
                }
                (
                    INT_64_TY | UINT_64_TY,
                    INT_8_TY | INT_16_TY | INT_32_TY | UINT_8_TY | UINT_16_TY | UINT_32_TY,
                ) => {
                    push_ty_cast_node(builder, TyCastKind::Trunc, ty, ty_to_match, operand)
                }
                (INT_32_TY | UINT_32_TY, INT_8_TY | INT_16_TY | UINT_8_TY | UINT_16_TY) => {
                    push_ty_cast_node(builder, TyCastKind::Trunc, ty, ty_to_match, operand)
                }
                (INT_16_TY | UINT_16_TY, INT_8_TY | UINT_8_TY) => {
                    push_ty_cast_node(builder, TyCastKind::Trunc, ty, ty_to_match, operand)
                }

                _ => None,
            }
        }

        match visit_result {
            VisitResult::PlaceKind(place_kind, place_ty) => {
                if
                    place_ty.test_eq_strict(
                        ty_to_match,
                        &self.icfg_builder.resolved_information.def_id_to_name_binding
                    )
                {
                    return (Operand::PlaceKind(place_kind), None, place_ty);
                }

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
                    if
                        ty.test_eq_strict(
                            ty_to_match,
                            &self.icfg_builder.resolved_information.def_id_to_name_binding
                        )
                    {
                        break;
                    }

                    if let Ty::Ptr(inner_ty, _) | Ty::StackPtr(inner_ty, _) = ty {
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

                if
                    let Some((new_ty, new_temp_id)) = number_coerceion(
                        self,
                        ty,
                        ty_to_match,
                        Operand::from(temp_id)
                    )
                {
                    (ty, temp_id) = (new_ty, new_temp_id);
                }

                if
                    ty.test_eq_strict(
                        ty_to_match,
                        &self.icfg_builder.resolved_information.def_id_to_name_binding
                    )
                {
                    (Operand::from(temp_id), None, ty_to_match)
                } else {
                    println!("visit_result: {:?}", visit_result);
                    panic!("Expected to match ty: {:?}, got: {:?}", ty_to_match, ty);
                }
            }
            VisitResult::Const(const_val, self_operand) => {
                if
                    let Some((new_ty, new_temp_id)) = number_coerceion(
                        self,
                        const_val.get_ty(),
                        ty_to_match,
                        Operand::Const(const_val)
                    )
                {
                    (Operand::from(new_temp_id), None, new_ty)
                } else {
                    (Operand::Const(const_val), self_operand, const_val.get_ty())
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum VisitResult {
    PlaceKind(PlaceKind, Ty),
    Const(Const, Option<Operand>),
}

impl VisitResult {
    pub fn get_ty(&self) -> Ty {
        match self {
            Self::PlaceKind(_, ty) => *ty,
            Self::Const(const_val, _) => const_val.get_ty(),
        }
    }
}

impl Default for VisitResult {
    fn default() -> Self {
        Self::Const(Const::Void, None)
    }
}

fn set_result_mem_id_to_if_expr_result<'ast>(
    cfg_builder: &mut CfgBuilder<'_, 'ast, '_>,
    if_expr: &'ast IfExpr<'ast>
) -> ResultMemId {
    let result_mem_id = cfg_builder.set_result_mem_id_to_expr_result(
        if_expr.ast_node_id,
        cfg_builder.icfg_builder.get_ty_from_node_id(if_expr.ast_node_id)
    );

    if let Some(IfFalseBranchExpr::ElifExpr(if_expr)) = if_expr.false_block {
        cfg_builder.node_id_to_result_mem_id.insert(if_expr.ast_node_id, result_mem_id);
    }

    result_mem_id
}

impl<'ast> Visitor<'ast> for CfgBuilder<'_, 'ast, '_> {
    /// This is a kind of "lazy-load" result. For example when visiting a variable, it just returns the place it lives in.
    /// It doesn't add the load node before requesting so with the method `get_operand_from_visit_result`.
    /// This useful because for example in a field expr, we don't necessarily want to add a load node (we may want to add a ByteAccessNode instead)
    type Result = VisitResult;

    fn default_result() -> Self::Result {
        Default::default()
    }

    fn visit_interger_expr(&mut self, integer_expr: &'ast ast::IntegerExpr) -> Self::Result {
        let ty = self.icfg_builder.get_ty_from_node_id(integer_expr.ast_node_id);
        let int_ty = match ty {
            Ty::PrimTy(PrimTy::Int(int_ty)) => int_ty,
            _ => panic!("Expected integer type"),
        };

        VisitResult::Const(Const::Int(integer_expr.val, int_ty), None)
    }

    fn visit_bool_expr(&mut self, bool_expr: &'ast ast::BoolExpr) -> Self::Result {
        VisitResult::Const(Const::Bool(bool_expr.val), None)
    }

    fn visit_string_expr(&mut self, string_expr: &'ast StringExpr) -> Self::Result {
        let def_id = self.icfg_builder.get_def_id_from_node_id(string_expr.ast_node_id);

        VisitResult::Const(Const::Str(def_id), None)
    }

    fn visit_null_expr(&mut self, _: &'ast NullExpr) -> Self::Result {
        VisitResult::Const(Const::Null, None)
    }

    fn visit_ident_expr(&mut self, ident_node: &'ast IdentNode) -> Self::Result {
        let def_id = self.icfg_builder.get_def_id_from_node_id(ident_node.ast_node_id);

        let ty = self.icfg_builder.get_ty_from_node_id(ident_node.ast_node_id);

        match ty {
            Ty::FnDef(_) | Ty::FnSig(_) => { VisitResult::Const(Const::FnPtr(def_id), None) }
            Ty::AtdConstructer(_) => { Default::default() }
            _ => {
                let local_mem_id = self.get_local_mem_id_from_def_id(def_id);
                VisitResult::PlaceKind(PlaceKind::LocalMemId(local_mem_id), ty)
            }
        }
    }

    fn visit_return_expr(&mut self, return_expr: &'ast ReturnExpr) -> Self::Result {
        let ret_ty = self.icfg_builder.get_ty_from_node_id(return_expr.ast_node_id);

        let mut push_void_node = || {
            self.push_node(
                Node::new(
                    NodeKind::ReturnNode(ReturnNode::new(Operand::Const(Const::Void), VOID_TY))
                )
            );
        };

        match return_expr.value {
            Some(expr) => {
                match ret_ty {
                    NEVER_TY => panic!("Cannot return never"),
                    VOID_TY => push_void_node(),
                    ty => {
                        let return_operand = {
                            let return_visit_result = self.visit_expr(expr);
                            self.get_operand_from_visit_result(return_visit_result, ret_ty).0
                        };

                        self.push_node(
                            Node::new(NodeKind::ReturnNode(ReturnNode::new(return_operand, ty)))
                        );
                    }
                }
            }
            None => push_void_node(),
        }

        Self::default_result()
    }

    fn visit_if_expr(&mut self, if_expr: &'ast IfExpr<'ast>) -> Self::Result {
        fn make_cond_and_locals_from_cond_pat<'ast>(
            cfg_builder: &mut CfgBuilder<'_, 'ast, '_>,
            pat: Pat<'ast>,
            expr_visit_result: VisitResult
            // expr_operand: Operand,
            // expr_ty: Ty
        ) -> (Vec<(Operand, BasicBlockId)>, Vec<(ByteAccessNode, LoadNode, StoreNode)>) {
            match pat {
                Pat::IdentPat(_) => unreachable!(),
                Pat::TupleStructPat(tuple_struct_pat) => {
                    let (enum_variant_id, enum_data) = {
                        let def_id = cfg_builder.icfg_builder.get_def_id_from_node_id(
                            tuple_struct_pat.ast_node_id
                        );

                        let name_binding =
                            cfg_builder.icfg_builder.resolved_information.get_name_binding_from_def_id(
                                &def_id
                            );

                        match name_binding.kind {
                            NameBindingKind::Adt(Adt::EnumVariant(_, enum_variant_id, enum_data)) =>
                                (enum_variant_id, enum_data),
                            _ => panic!("Expected enum variant"),
                        }
                    };

                    let load_temp_id = cfg_builder.get_temp_id();
                    let (access_place, access_ty) = match expr_visit_result {
                        VisitResult::PlaceKind(place_kind, ty) => (place_kind, ty),
                        _ => panic!("Expected TempId"),
                    };

                    cfg_builder.push_node(
                        Node::new(
                            NodeKind::LoadNode(LoadNode::new(load_temp_id, access_place, INT_64_TY))
                        )
                    );

                    let cmp_tmp_id = cfg_builder.get_temp_id();
                    cfg_builder.push_node(
                        Node::new(
                            NodeKind::BinaryNode(
                                BinaryNode::new(
                                    cmp_tmp_id,
                                    INT_64_TY,
                                    BinaryOp::ComparisonOp(ComparisonOp::Eq),
                                    Operand::from(load_temp_id),
                                    Operand::Const(
                                        Const::Int(enum_variant_id.0 as i64, IntTy::Int64)
                                    )
                                )
                            )
                        )
                    );

                    let current_bb_id = cfg_builder.get_curr_bb_id();

                    if
                        tuple_struct_pat.fields
                            .iter()
                            .any(|x| !matches!(&x, Pat::IdentPat(_)))
                    {
                        cfg_builder.new_basic_block();
                    }

                    let (mut false_bb_ids, mut store_nodes) = (
                        vec![(Operand::from(cmp_tmp_id), current_bb_id)],
                        vec![],
                    );
                    let mut byte_offset = 8;
                    for (i, pat) in tuple_struct_pat.fields.iter().enumerate() {
                        let ty = &enum_data[i];

                        if let Pat::IdentPat(ident_pat) = pat {
                            let local_mem_id = {
                                let local_mem_id = LocalMemId(cfg_builder.local_mems.len() as u32);
                                let local_mem = LocalMem::new(
                                    local_mem_id,
                                    Symbol::from_node_id(ident_pat.ast_node_id),
                                    ident_pat.span,
                                    *ty,
                                    Mutability::Immutable
                                );
                                cfg_builder.local_mems.push(local_mem);

                                let def_id = cfg_builder.icfg_builder.get_def_id_from_node_id(
                                    ident_pat.ast_node_id
                                );

                                cfg_builder.set_def_id_to_local_mem_id(def_id, local_mem_id);
                                local_mem_id
                            };

                            let byte_access_temp_id = cfg_builder.get_temp_id();
                            let byte_access_node = ByteAccessNode::new(
                                PlaceKind::TempId(byte_access_temp_id),
                                access_place,
                                byte_offset
                            );
                            let load_temp_id = cfg_builder.get_temp_id();
                            let load_node = LoadNode::new(
                                load_temp_id,
                                PlaceKind::TempId(byte_access_temp_id),
                                *ty
                            );
                            let store_node = StoreNode::new(
                                PlaceKind::LocalMemId(local_mem_id),
                                *ty,
                                Operand::from(load_temp_id),
                                StoreKind::Init
                            );

                            store_nodes.push((byte_access_node, load_node, store_node));
                        } else {
                            let byte_access_temp_id = cfg_builder.get_temp_id();
                            cfg_builder.push_node(
                                Node::new(
                                    NodeKind::ByteAccessNode(
                                        ByteAccessNode::new(
                                            PlaceKind::TempId(byte_access_temp_id),
                                            access_place,
                                            byte_offset
                                        )
                                    )
                                )
                            );

                            let other_false_bb_ids = make_cond_and_locals_from_cond_pat(
                                cfg_builder,
                                *pat,
                                VisitResult::PlaceKind(PlaceKind::TempId(byte_access_temp_id), *ty)
                            );
                            false_bb_ids.extend(other_false_bb_ids.0);
                            store_nodes.extend(other_false_bb_ids.1);
                        }

                        byte_offset += ty.get_ty_attr(
                            &cfg_builder.icfg_builder.resolved_information
                        ).size_bytes;
                    }

                    (false_bb_ids, store_nodes)
                }
            }
        }

        let ty_to_match = self.icfg_builder.get_ty_from_node_id(if_expr.ast_node_id);

        fn compile_true_block<'ast>(
            cfg_builder: &mut CfgBuilder<'_, 'ast, '_>,
            true_block: &'ast [Stmt<'ast>],
            result_mem_id: Option<ResultMemId>,
            ty_to_match: Ty
        ) -> (BasicBlockId, BasicBlockId) {
            let first_true_bb_id = cfg_builder.new_basic_block();
            let true_visit_result = cfg_builder.visit_stmts(true_block);
            if let Some(result_mem_id) = result_mem_id {
                let (true_operand, _, true_ty) = cfg_builder.get_operand_from_visit_result(
                    true_visit_result,
                    ty_to_match
                );

                cfg_builder.push_node(
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
            (first_true_bb_id, cfg_builder.get_curr_bb_id())
        }

        let if_expr_ty = self.icfg_builder.get_ty_from_node_id(if_expr.ast_node_id).to_ptr_ty();

        let result_mem_id = if !if_expr_ty.is_void() && !if_expr_ty.is_never() {
            Some(set_result_mem_id_to_if_expr_result(self, if_expr))
        } else {
            None
        };

        let bb_id_before_if_expr = BasicBlockId((self.basic_blocks.len() - 1) as u32);

        match if_expr.cond_kind {
            CondKind::CondPat(pat, rhs_expr) => {
                // let (expr_operand, expr_ty) = {
                //     let expr_visit_result = self.visit_expr(rhs_expr);
                //     self.get_operand_from_visit_result(expr_visit_result)
                // };
                let expr_visit_result = self.visit_expr(rhs_expr);
                let (bb_ids, store_nodes) = make_cond_and_locals_from_cond_pat(
                    self,
                    pat,
                    expr_visit_result
                    // expr_operand,
                    // expr_ty
                );

                let (first_true_bb_id, last_true_bb_id) = {
                    let first_true_bb_id = self.new_basic_block();
                    for nodes in store_nodes {
                        self.push_node(Node::new(NodeKind::ByteAccessNode(nodes.0)));
                        self.push_node(Node::new(NodeKind::LoadNode(nodes.1)));
                        self.push_node(Node::new(NodeKind::StoreNode(nodes.2)));
                    }

                    let true_visit_result = self.visit_stmts(if_expr.true_block);
                    if let Some(result_mem_id) = result_mem_id {
                        let (true_operand, _, true_ty) = self.get_operand_from_visit_result(
                            true_visit_result,
                            ty_to_match
                        );

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
                    (first_true_bb_id, self.get_curr_bb_id())
                };

                if let Some(false_branch) = &if_expr.false_block {
                    let false_bb_id = self.new_basic_block();
                    let bb_id_after_false_expr = BasicBlockId(false_bb_id.0 + 1);
                    let branch_out_node = Node::new(
                        NodeKind::BranchNode(BranchNode::new(bb_id_after_false_expr))
                    );

                    match false_branch {
                        IfFalseBranchExpr::ElifExpr(if_expr) => {
                            self.visit_if_expr(if_expr);

                            for (cond, bb) in &bb_ids {
                                self.push_node_to(
                                    *bb,
                                    Node::new(
                                        NodeKind::BranchCondNode(
                                            BranchCondNode::new(
                                                *cond,
                                                first_true_bb_id,
                                                false_bb_id
                                            )
                                        )
                                    )
                                );
                            }

                            self.push_node_to(last_true_bb_id, branch_out_node);
                            self.push_node_to(false_bb_id, branch_out_node);
                        }
                        IfFalseBranchExpr::ElseExpr(else_expr) => {
                            let false_visit_result = self.visit_block_expr(else_expr);
                            let (false_operand, _, false_ty) = self.get_operand_from_visit_result(
                                false_visit_result,
                                ty_to_match
                            );
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
                            for (cond, bb) in &bb_ids {
                                self.push_node_to(
                                    *bb,
                                    Node::new(
                                        NodeKind::BranchCondNode(
                                            BranchCondNode::new(
                                                *cond,
                                                first_true_bb_id,
                                                false_bb_id
                                            )
                                        )
                                    )
                                );
                            }

                            self.push_node_to(last_true_bb_id, branch_out_node);
                            self.push_node_to(false_bb_id, branch_out_node);

                            self.new_basic_block();
                        }
                    }
                } else {
                    let bb_id_after_true_expr = BasicBlockId(last_true_bb_id.0 + 1);
                    self.push_node(
                        Node::new(NodeKind::BranchNode(BranchNode::new(bb_id_after_true_expr)))
                    );

                    for (cond, bb) in &bb_ids {
                        self.push_node_to(
                            *bb,
                            Node::new(
                                NodeKind::BranchCondNode(
                                    BranchCondNode::new(
                                        *cond,
                                        first_true_bb_id,
                                        bb_id_after_true_expr
                                    )
                                )
                            )
                        );
                    }

                    self.new_basic_block();
                }
            }
            CondKind::CondExpr(cond_expr) => {
                let (cond_operand, _, _) = {
                    let cond_visit_result = self.visit_expr(cond_expr);
                    self.get_operand_from_visit_result(cond_visit_result, BOOL_TY)
                };

                let (first_true_bb_id, last_true_bb_id) = compile_true_block(
                    self,
                    if_expr.true_block,
                    result_mem_id,
                    ty_to_match
                );

                if let Some(false_branch) = &if_expr.false_block {
                    let false_bb_id = self.new_basic_block();
                    let bb_id_after_false_expr = BasicBlockId(false_bb_id.0 + 1);
                    let branch_out_node = Node::new(
                        NodeKind::BranchNode(BranchNode::new(bb_id_after_false_expr))
                    );

                    match false_branch {
                        IfFalseBranchExpr::ElifExpr(if_expr) => {
                            self.visit_if_expr(if_expr);
                            // let cond_ty = self.icfg_builder.get_ty_from_node_id(
                            //     get_node_id_from_expr(if_expr.condition)
                            // );
                            self.push_node_to(
                                bb_id_before_if_expr,
                                Node::new(
                                    NodeKind::BranchCondNode(
                                        BranchCondNode::new(
                                            cond_operand,
                                            first_true_bb_id,
                                            false_bb_id
                                        )
                                    )
                                )
                            );
                            self.push_node_to(last_true_bb_id, branch_out_node);
                            self.push_node_to(false_bb_id, branch_out_node);
                        }
                        IfFalseBranchExpr::ElseExpr(else_expr) => {
                            let false_visit_result = self.visit_block_expr(else_expr);
                            let (false_operand, _, false_ty) = self.get_operand_from_visit_result(
                                false_visit_result,
                                ty_to_match
                            );
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
                                        BranchCondNode::new(
                                            cond_operand,
                                            first_true_bb_id,
                                            false_bb_id
                                        )
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
                    self.push_node(
                        Node::new(NodeKind::BranchNode(BranchNode::new(bb_id_after_true_expr)))
                    );
                    self.push_node_to(
                        bb_id_before_if_expr,
                        Node::new(
                            NodeKind::BranchCondNode(
                                BranchCondNode::new(
                                    cond_operand,
                                    first_true_bb_id,
                                    bb_id_after_true_expr
                                )
                            )
                        )
                    );
                    self.new_basic_block();
                }
            }
        }

        if let Some(result_mem_id) = result_mem_id {
            VisitResult::PlaceKind(PlaceKind::ResultMemId(result_mem_id), if_expr_ty)
        } else {
            Self::default_result()
        }
    }

    fn visit_loop_expr(&mut self, loop_expr: &'ast LoopExpr<'ast>) -> Self::Result {
        fn set_result_mem_id_to_loop_expr_result<'ast>(
            cfg_builder: &mut CfgBuilder<'_, 'ast, '_>,
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

        let prev_break_bb_ids = std::mem::take(&mut self.break_bb_ids);
        let prev_continue_bb_ids = std::mem::take(&mut self.continue_bb_ids);

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

    fn visit_fn_item(&mut self, fn_item: &'ast FnItem<'ast>) -> Self::Result {
        Default::default()
    }

    fn visit_call_expr(&mut self, call_expr: &'ast CallExpr<'ast>) -> Self::Result {
        let ty = self.icfg_builder.get_ty_from_node_id(get_node_id_from_expr(call_expr.callee));

        if let Ty::AtdConstructer(enum_variant_def_id) = ty {
            let name_binding = self.icfg_builder.resolved_information.get_name_binding_from_def_id(
                &enum_variant_def_id
            );
            let (enum_ty, enum_variant_id, enum_data) = match name_binding.kind {
                NameBindingKind::Adt(Adt::EnumVariant(enum_def_id, enum_variant_id, enum_data)) => {
                    (Ty::Adt(enum_def_id), enum_variant_id, enum_data)
                }
                t => panic!("Expected enum variant, got {:?}", t),
            };

            let result_mem_id = self.new_result_mem(enum_ty);

            // First store the enum variant id (the discriminant)
            self.push_node(
                Node::new(
                    NodeKind::StoreNode(
                        StoreNode::new(
                            PlaceKind::ResultMemId(result_mem_id),
                            INT_64_TY,
                            Operand::Const(Const::Int(enum_variant_id.0 as i64, IntTy::Int64)),
                            StoreKind::Init
                        )
                    )
                )
            );

            // Now we store the data
            let arg_operands = call_expr.args
                .iter()
                .enumerate()
                .map(|(i, arg)| {
                    let arg_ty = enum_data[i];
                    let visit_result = self.visit_expr(*arg);
                    self.get_operand_from_visit_result(visit_result, arg_ty)
                })
                .collect::<Vec<_>>();

            let mut byte_offset = 8; // The discriminant is 8 bytes
            for (operand, _, operand_ty) in arg_operands {
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

                byte_offset += operand_ty.get_ty_attr(
                    &self.icfg_builder.resolved_information
                ).size_bytes;
            }

            return VisitResult::PlaceKind(
                PlaceKind::ResultMemId(result_mem_id),
                enum_ty.to_ptr_ty()
            );
        }

        let fn_args_tys = match ty.auto_deref() {
            Ty::FnDef(def_id) => {
                let name_binding =
                    self.icfg_builder.resolved_information.get_name_binding_from_def_id(&def_id);
                if let NameBindingKind::Fn(fn_sig, _, _) = name_binding.kind {
                    fn_sig.args
                } else {
                    panic!("Expected fn")
                }
            }
            Ty::FnSig(fn_sig) => fn_sig.args,
            ty => panic!("Expected fn, got {}", ty),
        };

        let visit_result = self.visit_expr(call_expr.callee);
        let (callee_operand, self_operand, _) = self.get_operand_from_visit_result(
            visit_result,
            ty.auto_deref()
        );
        let mut call_args_tys = Vec::with_capacity(fn_args_tys.len());
        if self_operand.is_some() {
            call_args_tys.push(fn_args_tys[0]);
        }

        let mut found_variadic = false;
        let arg_operands = {
            let mut arg_operands = Vec::with_capacity(fn_args_tys.len());
            if let Some(self_operand) = self_operand {
                arg_operands.push(self_operand);
            }

            for (i, arg) in call_expr.args.iter().enumerate() {
                let i = if self_operand.is_some() { i + 1 } else { i };

                let mut ty_to_match = if found_variadic {
                    self.icfg_builder.get_ty_from_node_id(get_node_id_from_expr(*arg))
                } else {
                    fn_args_tys[i]
                };

                if ty_to_match.is_variadic_args() {
                    found_variadic = true;
                    call_args_tys.push(Ty::VariadicArgs);
                    ty_to_match = self.icfg_builder.get_ty_from_node_id(
                        get_node_id_from_expr(*arg)
                    );
                }
                if found_variadic {
                    ty_to_match = ty_to_match.deref_if_stack_ptr();
                }

                call_args_tys.push(ty_to_match);

                let visit_result = self.visit_expr(*arg);
                let arg_operand = self.get_operand_from_visit_result(visit_result, ty_to_match).0;
                arg_operands.push(arg_operand);
            }

            TyCtx::intern_many_types(arg_operands)
        };
        let ret_ty = self.icfg_builder.get_ty_from_node_id(call_expr.ast_node_id);

        let temp_id = self.get_temp_id();
        let call_node = Node::new(
            NodeKind::CallNode(
                CallNode::new(
                    temp_id,
                    callee_operand,
                    arg_operands,
                    TyCtx::intern_many_types(call_args_tys),
                    ret_ty
                )
            )
        );
        self.push_node(call_node);
        VisitResult::PlaceKind(PlaceKind::TempId(temp_id), ret_ty)
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
                .try_deref_as_tuple(|def_id| {
                    self.icfg_builder.resolved_information.def_id_to_name_binding.get(&def_id)
                })
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
                byte_offset += ty.get_ty_attr(&self.icfg_builder.resolved_information).size_bytes;

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

    fn visit_field_expr(&mut self, field_expr: &'ast FieldExpr<'ast>) -> Self::Result {
        let lhs_ty = self.icfg_builder.get_ty_from_node_id(get_node_id_from_expr(field_expr.lhs));

        if let Ty::Package = lhs_ty {
            return self.visit_ident_expr(field_expr.rhs);
        }

        let visit_result = self.visit_expr(field_expr.lhs);
        if let Ty::AtdConstructer(def_id) = lhs_ty {
            if
                let Some(rhs_def_id) =
                    self.icfg_builder.resolved_information.try_get_def_id_from_node_id(
                        &field_expr.rhs.ast_node_id
                    )
            {
                let name_binding =
                    self.icfg_builder.resolved_information.get_name_binding_from_def_id(
                        &rhs_def_id
                    );

                match name_binding.kind {
                    NameBindingKind::Adt(Adt::EnumVariant(_, variant_id, _)) => {
                        // Zerosized enum variant
                        let result_mem_id = self.new_result_mem(Ty::Adt(def_id));

                        self.push_node(
                            Node::new(
                                NodeKind::StoreNode(
                                    StoreNode::new(
                                        PlaceKind::ResultMemId(result_mem_id),
                                        INT_64_TY,
                                        Operand::Const(
                                            Const::Int(variant_id.0 as i64, IntTy::Int64)
                                        ),
                                        StoreKind::Init
                                    )
                                )
                            )
                        );

                        return VisitResult::PlaceKind(
                            PlaceKind::ResultMemId(result_mem_id),
                            Ty::Adt(def_id).to_ptr_ty()
                        );
                    }
                    NameBindingKind::Fn(_, _, _) => {
                        // Constructor method e.g. `Adt.new()`
                        return VisitResult::Const(Const::FnPtr(rhs_def_id), None);
                    }
                    name_binding => unreachable!("Expected fn or enum: {:?}", name_binding),
                }
            }

            unreachable!("Hopefully this is unreachable");
        } else if
            let Some(def_id) = self.icfg_builder.resolved_information.try_get_def_id_from_node_id(
                &field_expr.rhs.ast_node_id
            )
        {
            let (fn_sig, has_self_arg) = {
                let name_binding =
                    self.icfg_builder.resolved_information.get_name_binding_from_def_id(&def_id);
                match name_binding.kind {
                    NameBindingKind::Fn(fn_sig, has_self_arg, _) => (fn_sig, has_self_arg),
                    _ => unreachable!("Expected fn"),
                }
            };

            let mut tys_iter = fn_sig.args.iter();

            if has_self_arg == HasSelfArg::Yes {
                let ty_to_match = *tys_iter.next().expect("Expected at least one arg");

                let lhs_place = match self.get_operand_from_visit_result(visit_result, ty_to_match) {
                    (Operand::PlaceKind(place), _, _) => place,
                    _ => unreachable!("This should be unreachable if type checking was successful"),
                };

                return VisitResult::Const(
                    Const::FnPtr(def_id),
                    Some(Operand::PlaceKind(lhs_place))
                );
            } else {
                return VisitResult::Const(Const::FnPtr(def_id), None);
            }
        }

        let lhs_place = match
            self.get_operand_from_visit_result(
                visit_result,
                lhs_ty.deref_until_stack_ptr_and_one_more_if_ptr()
            )
        {
            (Operand::PlaceKind(place), _, _) => place,
            _ => unreachable!("This should be unreachable if type checking was successful"),
        };

        let access_symbol = Symbol::from_node_id(field_expr.rhs.ast_node_id);

        let (struct_name, struct_fields) = match
            self.icfg_builder
                .get_ty_from_node_id(get_node_id_from_expr(field_expr.lhs))
                .try_deref_as_struct(&self.icfg_builder.resolved_information.def_id_to_name_binding)
        {
            Some(struct_ty) => struct_ty,
            None => unreachable!("Should not be able to go here if previous pass was successfull"),
        };

        let is_mutable = lhs_ty.deref_until_stack_ptr().is_mut_ptr();

        let mut byte_offset = 0;
        let elem_ty = struct_fields
            .iter()
            .find(|(symbol, ty)| (
                if symbol.symbol.get() == access_symbol.get() {
                    true
                } else {
                    byte_offset += ty.get_ty_attr(
                        &self.icfg_builder.resolved_information
                    ).size_bytes;
                    false
                }
            ))
            .map(|x| x.1)
            .expect("Field not found");

        let elem_ty = if is_mutable { elem_ty.to_mut_ptr_ty() } else { elem_ty.to_ptr_ty() };

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

    fn visit_index_expr(&mut self, index_expr: &'ast IndexExpr<'ast>) -> Self::Result {
        let lhs_ty = self.icfg_builder.get_ty_from_node_id(get_node_id_from_expr(index_expr.lhs));

        let lhs_place = {
            let lhs_visit_result = self.visit_expr(index_expr.lhs);
            let operand = self.get_operand_from_visit_result(
                lhs_visit_result,
                lhs_ty.deref_until_stack_ptr_and_one_more_if_ptr()
            ).0;

            match operand {
                Operand::PlaceKind(place) => place,
                _ => unreachable!("Expected PlaceKind"),
            }
        };

        let value_operand = {
            let value_visit_result = self.visit_expr(index_expr.value_expr);
            self.get_operand_from_visit_result(value_visit_result, INT_64_TY).0
        };

        let elem_ty = self.icfg_builder.get_ty_from_node_id(index_expr.ast_node_id);

        let temp_id = self.get_temp_id();
        self.push_node(
            Node::new(
                NodeKind::IndexNode(
                    IndexNode::new(
                        temp_id,
                        lhs_place,
                        elem_ty.try_deref_once().expect("Expected to be able to deref once"),
                        value_operand
                    )
                )
            )
        );

        VisitResult::PlaceKind(PlaceKind::TempId(temp_id), elem_ty)
    }

    fn visit_assign_stmt(&mut self, assign_stmt: &'ast ast::AssignStmt<'ast>) -> Self::Result {
        // let (setter_place, value_ty) = {
        //     let assingment_ty = self.icfg_builder
        //         .get_ty_from_node_id(get_node_id_from_place_expr(assign_stmt.setter_expr))
        //         .deref_until_stack_ptr_and_one_more_if_ptr();
        //     let visit_result = self.visit_place_expr(assign_stmt.setter_expr);
        //     let operand = self.get_operand_from_visit_result(visit_result, assingment_ty).0;

        //     match operand {
        //         Operand::PlaceKind(place) =>
        //             (place, assingment_ty.try_deref_once().expect("Expected ptr")),
        //         _ => unreachable!("Expected PlaceKind"),
        //     }
        // };

        macro_rules! visit_expr {
            ($visit_expr:ident, $expr:ident) => {
                {
                    let visit_result = self.$visit_expr($expr);

                    match visit_result {
                        VisitResult::PlaceKind(place_kind, ty) => (place_kind, ty),
                        _ => unreachable!("Expected PlaceKind"),
                    }
                }
            };
        }

        let (setter_place, value_ty) = match assign_stmt.setter_expr {
            AsigneeExpr::CallExpr(call_expr) => {
                let (place, ty) = visit_expr!(visit_call_expr, call_expr);
                (place, ty)
            }

            AsigneeExpr::PlaceExpr(place_expr) => {
                match place_expr {
                    PlaceExpr::PkgIdentExpr(_) => { panic!("Invalid assignee (pkg)") }
                    PlaceExpr::TupleFieldExpr(tuple_field_expr) => {
                        visit_expr!(visit_tuple_field_expr, tuple_field_expr)
                    }
                    PlaceExpr::FieldExpr(field_expr) => {
                        visit_expr!(visit_field_expr, field_expr)
                    }
                    PlaceExpr::IndexExpr(index_expr) => {
                        visit_expr!(visit_index_expr, index_expr)
                    }
                    PlaceExpr::IdentExpr(ident_expr) => {
                        let assingment_ty = self.icfg_builder
                            .get_ty_from_node_id(ident_expr.ast_node_id)
                            .deref_until_stack_ptr_and_one_more_if_ptr();
                        let visit_result = self.visit_ident_expr(ident_expr);
                        let operand = self.get_operand_from_visit_result(
                            visit_result,
                            assingment_ty
                        ).0;

                        match operand {
                            Operand::PlaceKind(place) => (place, assingment_ty),
                            _ => unreachable!("Expected PlaceKind"),
                        }
                    }
                }
            }
        };

        let ty_to_match = value_ty.try_deref_once().expect("Expected ptr");

        let value_visit_result = self.visit_expr(assign_stmt.value_expr);

        let (operand, _, op_ty) = self.get_operand_from_visit_result(
            value_visit_result,
            ty_to_match
        );

        self.push_node(
            Node::new(
                NodeKind::StoreNode(StoreNode::new(setter_place, op_ty, operand, StoreKind::Assign))
            )
        );

        Self::default_result()
    }

    fn visit_def_stmt(&mut self, def_stmt: &'ast ast::DefineStmt<'ast>) -> Self::Result {
        let ty = self.icfg_builder.get_ty_from_node_id(get_node_id_from_expr(def_stmt.value_expr));

        // If ty is an enum variant, get the type of the whole enum
        let ty = match ty {
            Ty::Adt(enum_variant_def_id) => {
                let name_binding =
                    self.icfg_builder.resolved_information.get_name_binding_from_def_id(
                        &enum_variant_def_id
                    );
                match name_binding.kind {
                    NameBindingKind::Adt(Adt::EnumVariant(enum_def_id, _, _)) =>
                        Ty::Adt(enum_def_id),
                    _ => ty,
                }
            }
            _ => ty,
        };

        if ty.is_void() {
            self.visit_expr(def_stmt.value_expr);
            return Self::default_result();
        }

        let local_mem_id = match def_stmt.setter_expr {
            Pat::IdentPat(ident_pat) => {
                let local_mem_id = LocalMemId(self.local_mems.len() as u32);
                let local_mem = LocalMem::new(
                    local_mem_id,
                    Symbol::from_node_id(ident_pat.ast_node_id),
                    ident_pat.span,
                    ty,
                    Mutability::Immutable
                );
                self.local_mems.push(local_mem);

                let def_id = self.icfg_builder.get_def_id_from_node_id(ident_pat.ast_node_id);

                self.set_def_id_to_local_mem_id(def_id, local_mem_id);
                local_mem_id
            }
            Pat::TupleStructPat(_) => unreachable!("Should have been caught by type checking"),
        };

        let ty_to_match = ty;

        let value_visit_result = self.visit_expr(def_stmt.value_expr);
        let (operand, _, op_ty) = self.get_operand_from_visit_result(
            value_visit_result,
            ty_to_match
        );

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
            .try_deref_as_struct(&self.icfg_builder.resolved_information.def_id_to_name_binding)
            .expect("Expected ty to be struct");

        'outer: for (field_symbol, ty_to_match) in struct_fields {
            for field in struct_expr.field_initializations.iter() {
                let access_field_symbol = Symbol::from_node_id(field.ident.ast_node_id);
                if access_field_symbol.get() == field_symbol.symbol.get() {
                    byte_offset = self.init_tuple_or_struct_field(
                        field.value,
                        result_mem_id,
                        *ty_to_match,
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
            let ty_to_match = self.icfg_builder.get_ty_from_node_id(get_node_id_from_expr(*expr));
            byte_offset = self.init_tuple_or_struct_field(
                *expr,
                result_mem_id,
                ty_to_match,
                byte_offset
            );
        }

        VisitResult::PlaceKind(PlaceKind::ResultMemId(result_mem_id), tuple_ty.to_ptr_ty())
    }

    fn visit_binary_expr(&mut self, binary_expr: &'ast ast::BinaryExpr<'ast>) -> Self::Result {
        let result_ty = self.icfg_builder.get_ty_from_node_id(binary_expr.ast_node_id);

        let (lhs_ty, rhs_ty) = {
            let lhs_ty = self.icfg_builder.get_ty_from_node_id(
                get_node_id_from_expr(binary_expr.lhs)
            );
            let rhs_ty = self.icfg_builder.get_ty_from_node_id(
                get_node_id_from_expr(binary_expr.rhs)
            );
            (lhs_ty, rhs_ty)
        };

        let op_ty = if let Some(biggest_num_ty) = Ty::get_biggest_num_ty(lhs_ty, rhs_ty) {
            biggest_num_ty
        } else {
            lhs_ty.auto_deref()
        };

        let (lhs_operand, _, _) = {
            let lhs_visit_result = self.visit_expr(binary_expr.lhs);
            self.get_operand_from_visit_result(lhs_visit_result, op_ty)
        };
        let (rhs_operand, _, _) = {
            let rhs_visit_result = self.visit_expr(binary_expr.rhs);
            self.get_operand_from_visit_result(rhs_visit_result, op_ty)
        };

        let result_place = self.get_temp_id();

        self.push_node(
            Node::new(
                NodeKind::BinaryNode(
                    BinaryNode::new(result_place, op_ty, binary_expr.op, lhs_operand, rhs_operand)
                )
            )
        );

        VisitResult::PlaceKind(PlaceKind::TempId(result_place), result_ty)
    }
}

impl<'ast> CfgBuilder<'_, 'ast, '_> {
    fn init_tuple_or_struct_field(
        &mut self,
        expr: Expr<'ast>,
        result_mem_id: ResultMemId,
        ty_to_match: Ty,
        byte_offset: usize
    ) -> usize {
        let visit_result = self.visit_expr(expr);

        let (operand, _, operand_ty) = self.get_operand_from_visit_result(
            visit_result,
            ty_to_match
        );

        if byte_offset == 0 {
            self.push_node(
                Node::new(
                    NodeKind::StoreNode(
                        StoreNode::new(
                            PlaceKind::ResultMemId(result_mem_id),
                            ty_to_match,
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
                            ty_to_match,
                            operand,
                            StoreKind::Init
                        )
                    )
                )
            );
        }

        let operand_ty_attr = operand_ty.get_ty_attr(&self.icfg_builder.resolved_information);
        byte_offset + operand_ty_attr.size_bytes
    }
}
