use fxhash::FxHashMap;
use icfg::{
    walk_args,
    walk_basic_block,
    walk_basic_blocks,
    walk_cfg,
    walk_local_mems,
    walk_result_mems,
    BasicBlockId,
    Cfg,
    CfgVisitor,
    Const,
    Icfg,
    Operand,
    PlaceKind,
};
use op::{ ArithmeticOp, BinaryOp, ComparisonOp };
use ir::{ CfgFnKind, GetTyAttr, PrimTy, ResolvedInformation, Ty };
use std::{ fmt::Write, fs::File, process::Command };

const INDENTATION: usize = 4;

// pub(crate) fn get_ty_stack_size_in_bytes(ty: &Ty) -> usize {}

/// Used to allocate places (e.g. results of operations, variables, etc.) before the actual code gen
pub(crate) struct CodeGenUnitHelper<'a> {
    cfg: &'a Cfg,
    resolved_information: &'a ResolvedInformation<'a>,
    next_ssa_id: usize,
    place_to_ssa_id: FxHashMap<PlaceKind, usize>,
    basic_block_id_to_ssa_id: FxHashMap<BasicBlockId, usize>,
}

impl<'a> CodeGenUnitHelper<'a> {
    pub(crate) fn new(cfg: &'a Cfg, resolved_information: &'a ResolvedInformation) -> Self {
        Self {
            cfg,
            resolved_information,
            next_ssa_id: 1,
            place_to_ssa_id: Default::default(),
            basic_block_id_to_ssa_id: Default::default(),
        }
    }

    pub(crate) fn allocate_places(
        mut self
    ) -> (FxHashMap<PlaceKind, usize>, FxHashMap<BasicBlockId, usize>) {
        self.visit_cfg(self.cfg);

        (self.place_to_ssa_id, self.basic_block_id_to_ssa_id)
    }

    pub(crate) fn get_next_ssa_id(&mut self) -> usize {
        self.next_ssa_id += 1;
        self.next_ssa_id - 1
    }
}

impl<'a> CfgVisitor for CodeGenUnitHelper<'a> {
    type Result = ();
    fn default_result() -> Self::Result {}

    fn visit_cfg(&mut self, cfg: &Cfg) -> Self::Result {
        walk_args(self, cfg);
        self.next_ssa_id += 1;
        walk_local_mems(self, cfg);
        walk_result_mems(self, cfg);
        walk_basic_blocks(self, cfg)
    }

    fn visit_basic_block(&mut self, basic_block: &icfg::BasicBlock, cfg: &Cfg) -> Self::Result {
        let next_ssa_id = self.get_next_ssa_id();
        self.basic_block_id_to_ssa_id.insert(basic_block.basic_block_id, next_ssa_id);
        walk_basic_block(self, basic_block, cfg)
    }

    fn visit_arg(&mut self, arg: &(icfg::TempId, Ty)) -> Self::Result {
        let next_ssa_id = self.get_next_ssa_id();
        self.place_to_ssa_id.insert(PlaceKind::TempId(arg.0), next_ssa_id);
    }

    fn visit_local_mem(&mut self, local_mem: &icfg::LocalMem) -> Self::Result {
        let next_ssa_id = self.get_next_ssa_id();
        self.place_to_ssa_id.insert(PlaceKind::LocalMemId(local_mem.local_mem_id), next_ssa_id);
    }

    fn visit_result_mem(&mut self, result_mem: &icfg::ResultMem) -> Self::Result {
        let next_ssa_id = self.get_next_ssa_id();
        self.place_to_ssa_id.insert(PlaceKind::ResultMemId(result_mem.result_mem_id), next_ssa_id);
    }

    fn visit_binary_node(&mut self, binary_node: &icfg::BinaryNode, _cfg: &Cfg) -> Self::Result {
        let next_ssa_id = self.get_next_ssa_id();
        self.place_to_ssa_id.insert(PlaceKind::TempId(binary_node.result_place), next_ssa_id);
    }

    fn visit_load_node(&mut self, load_node: &icfg::LoadNode, _cfg: &Cfg) -> Self::Result {
        let next_ssa_id = self.get_next_ssa_id();
        self.place_to_ssa_id.insert(PlaceKind::TempId(load_node.result_place), next_ssa_id);
    }

    fn visit_index_node(&mut self, index_node: &icfg::IndexNode, _cfg: &Cfg) -> Self::Result {
        let next_ssa_id = self.get_next_ssa_id();
        self.place_to_ssa_id.insert(PlaceKind::TempId(index_node.result_place), next_ssa_id);
    }

    fn visit_byte_access_node(
        &mut self,
        byte_access_node: &icfg::ByteAccessNode,
        _cfg: &Cfg
    ) -> Self::Result {
        let next_ssa_id = self.get_next_ssa_id();
        self.place_to_ssa_id.insert(byte_access_node.result_place, next_ssa_id);
    }
}

pub(crate) struct CodeGenUnit<'a> {
    cfg: &'a Cfg,
    resolved_information: &'a ResolvedInformation<'a>,
    buffer: String,
    place_to_ssa_id: FxHashMap<PlaceKind, usize>,
    basic_block_id_to_ssa_id: FxHashMap<BasicBlockId, usize>,
}

impl<'a> CodeGenUnit<'a> {
    pub(crate) fn new(cfg: &'a Cfg, resolved_information: &'a ResolvedInformation<'a>) -> Self {
        let (place_to_ssa_id, basic_block_id_to_ssa_id) = {
            CodeGenUnitHelper::new(cfg, resolved_information).allocate_places()
        };

        Self {
            cfg,
            resolved_information,
            buffer: String::with_capacity(2048),
            place_to_ssa_id,
            basic_block_id_to_ssa_id,
        }
    }

    pub(crate) fn gen_code(mut self, buffer: &mut String) {
        let err_msg = "Unexpected write error";
        self.visit_cfg(self.cfg).expect(err_msg);
        writeln!(buffer, "{}", self.buffer).expect(err_msg)
    }

    pub(crate) fn get_ssa_id_from_place(&self, place: &PlaceKind) -> usize {
        *self.place_to_ssa_id.get(place).expect("Expected place")
    }

    pub(crate) fn get_bb_id(&self, bb_id: &BasicBlockId) -> usize {
        *self.basic_block_id_to_ssa_id.get(bb_id).expect("Expected BasicBlockId")
    }

    pub(crate) fn get_llvm_ty(&self, ty: Ty) -> String {
        match &ty {
            Ty::PrimTy(prim_ty) => {
                match prim_ty {
                    PrimTy::Bool => "i1".to_string(),
                    PrimTy::Int => "i32".to_string(),
                    PrimTy::Void => "void".to_string(),
                }
            }
            Ty::Ptr(_) => "ptr".to_string(),
            ty @ Ty::Tuple(_) => {
                let ty_attr = ty.get_ty_attr(self.resolved_information);
                format!("[{} x i8]", ty_attr.size_bytes)
            }
            ty @ Ty::Adt(_) => {
                let ty_attr = ty.get_ty_attr(self.resolved_information);
                format!("[{} x i8]", ty_attr.size_bytes)
            }
            Ty::FnSig(_) => { todo!("Yay I got this far") }
            Ty::FnDef(_) => {
                todo!("Yay I got this far (pretty much the same functionality as above, I think)")
            }

            Ty::Unkown => panic!("Unkown type (should not be this far in compilation)"),
        }
    }

    pub(crate) fn get_llvm_operand(&self, operand: &Operand) -> String {
        let string = match &operand {
            Operand::TempId(place) => {
                format!("%{}", self.get_ssa_id_from_place(&PlaceKind::TempId(*place)))
            }
            Operand::Const(const_val) => {
                match const_val {
                    Const::Bool(bool) =>
                        (
                            match bool {
                                true => "1",
                                false => "0",
                            }
                        ).to_string(),
                    Const::Int(int) => int.to_string(),
                    Const::Void => panic!("Void cannot be used as an operand"),
                }
            }
        };
        string
    }
}

impl<'a> CfgVisitor for CodeGenUnit<'a> {
    type Result = Result<(), std::fmt::Error>;

    fn default_result() -> Self::Result {
        Ok(())
    }

    fn visit_cfg(&mut self, cfg: &Cfg) -> Self::Result {
        match cfg.cfg_fn_kind {
            CfgFnKind::Main => writeln!(self.buffer, "define i32 @main() {{")?,
            CfgFnKind::Fn(def_id) => {
                write!(
                    self.buffer,
                    "define {} @{}{}(",
                    self.get_llvm_ty(cfg.ret_ty),
                    def_id.symbol.get(),
                    def_id.node_id.0
                )?;

                for (i, (temp_id, arg_ty)) in cfg.args.iter().enumerate() {
                    write!(
                        self.buffer,
                        "{} noundef %{}",
                        self.get_llvm_ty(*arg_ty),
                        self.get_ssa_id_from_place(&PlaceKind::TempId(*temp_id))
                    )?;
                    if i != cfg.args.len() - 1 {
                        write!(self.buffer, ", ")?;
                    }
                }

                writeln!(self.buffer, ") {{")?;
            }
        }

        walk_local_mems(self, cfg)?;
        walk_result_mems(self, cfg)?;

        let first_bb_id = self.basic_block_id_to_ssa_id
            .get(&BasicBlockId(0))
            .expect("Expected at least one BasicBlock");
        writeln!(self.buffer, "{}br label %{}", " ".repeat(INDENTATION), first_bb_id)?;

        walk_basic_blocks(self, cfg)?;
        if cfg.cfg_fn_kind == CfgFnKind::Main {
            writeln!(self.buffer, "{}ret i32 0", " ".repeat(INDENTATION))?;
        } else if cfg.ret_ty.is_void() {
            writeln!(self.buffer, "{}ret void", " ".repeat(INDENTATION))?;
        }
        writeln!(self.buffer, "}}")
    }

    fn visit_local_mem(&mut self, local_mem: &icfg::LocalMem) -> Self::Result {
        let ssa_id = self.get_ssa_id_from_place(&PlaceKind::LocalMemId(local_mem.local_mem_id));

        let ty_attr = local_mem.ty.get_ty_attr(self.resolved_information);

        writeln!(
            self.buffer,
            "{}%{} = alloca [{} x i8], align {}",
            " ".repeat(INDENTATION),
            ssa_id,
            ty_attr.size_bytes,
            ty_attr.alignment_bytes
        )
    }

    fn visit_result_mem(&mut self, result_mem: &icfg::ResultMem) -> Self::Result {
        let ssa_id = self.get_ssa_id_from_place(&PlaceKind::ResultMemId(result_mem.result_mem_id));

        let ty_attr = result_mem.ty.get_ty_attr(self.resolved_information);

        writeln!(
            self.buffer,
            "{}%{} = alloca [{} x i8], align {}",
            " ".repeat(INDENTATION),
            ssa_id,
            ty_attr.size_bytes,
            ty_attr.alignment_bytes
        )
    }

    fn visit_basic_block(&mut self, basic_block: &icfg::BasicBlock, cfg: &Cfg) -> Self::Result {
        let bb_id = self.get_bb_id(&basic_block.basic_block_id);
        writeln!(self.buffer, "{}:", bb_id)?;
        walk_basic_block(self, basic_block, cfg)
    }

    fn visit_branch_node(&mut self, branch_node: &icfg::BranchNode, cfg: &Cfg) -> Self::Result {
        let branch_id = self.get_bb_id(&branch_node.branch);
        writeln!(self.buffer, "{}br label %{}", " ".repeat(INDENTATION), branch_id)
    }

    fn visit_store_node(&mut self, store_node: &icfg::StoreNode, cfg: &Cfg) -> Self::Result {
        let var_place = self.get_ssa_id_from_place(&store_node.setter);
        let val = self.get_llvm_operand(&store_node.value);

        writeln!(
            self.buffer,
            "{}store {} {}, ptr %{}",
            " ".repeat(INDENTATION),
            self.get_llvm_ty(store_node.op_ty),
            val,
            var_place
        )
    }

    fn visit_index_node(&mut self, index_node: &icfg::IndexNode, cfg: &Cfg) -> Self::Result {
        let temp_id = self.get_ssa_id_from_place(&PlaceKind::TempId(index_node.result_place));
        let array_id = self.get_ssa_id_from_place(&index_node.array_place);

        writeln!(
            self.buffer,
            "{}%{} = getelementptr inbounds {}, ptr %{}, i64 0, i64 {}",
            " ".repeat(INDENTATION),
            temp_id,
            self.get_llvm_ty(index_node.place_ty),
            array_id,
            index_node.index
        )
    }

    fn visit_byte_access_node(
        &mut self,
        byte_access_node: &icfg::ByteAccessNode,
        cfg: &Cfg
    ) -> Self::Result {
        let temp_id = self.get_ssa_id_from_place(&byte_access_node.result_place);
        let array_id = self.get_ssa_id_from_place(&byte_access_node.access_place);

        writeln!(
            self.buffer,
            "{}%{} = getelementptr inbounds i8, ptr %{}, i64 {}",
            " ".repeat(INDENTATION),
            temp_id,
            array_id,
            byte_access_node.byte_offset
        )
    }

    fn visit_branch_cond_node(
        &mut self,
        branch_cond_node: &icfg::BranchCondNode,
        cfg: &Cfg
    ) -> Self::Result {
        let cond = self.get_llvm_operand(&branch_cond_node.condition);
        let true_branch = self.get_bb_id(&branch_cond_node.true_branch);
        let false_branch = self.get_bb_id(&branch_cond_node.false_branch);

        writeln!(
            self.buffer,
            "{}br i1 {}, label %{}, label %{}",
            " ".repeat(INDENTATION),
            cond,
            true_branch,
            false_branch
        )
    }

    fn visit_load_node(&mut self, load_node: &icfg::LoadNode, cfg: &Cfg) -> Self::Result {
        let ssa_id = self.get_ssa_id_from_place(&PlaceKind::TempId(load_node.result_place));
        let var_place = self.get_ssa_id_from_place(&load_node.load_place);
        writeln!(
            self.buffer,
            "{}%{} = load {}, ptr %{}",
            " ".repeat(INDENTATION),
            ssa_id,
            self.get_llvm_ty(load_node.load_ty),
            var_place
        )
    }

    fn visit_binary_node(&mut self, binary_node: &icfg::BinaryNode, cfg: &Cfg) -> Self::Result {
        let lhs_op = self.get_llvm_operand(&binary_node.lhs);
        let rhs_op = self.get_llvm_operand(&binary_node.rhs);

        let ssa_id = self.get_ssa_id_from_place(&PlaceKind::TempId(binary_node.result_place));

        let op_kw = match binary_node.op {
            BinaryOp::ArithmeticOp(ArithmeticOp::Add) => "add nsw",
            BinaryOp::ArithmeticOp(ArithmeticOp::Sub) => "sub nsw",
            BinaryOp::ArithmeticOp(ArithmeticOp::Mul) => "mul nsw",
            BinaryOp::ArithmeticOp(ArithmeticOp::Div) => "div nsw",
            BinaryOp::ComparisonOp(ComparisonOp::Eq) => "icmp eq",
            BinaryOp::ComparisonOp(ComparisonOp::Ne) => "icmp ne",
            BinaryOp::ComparisonOp(ComparisonOp::Ge) => "icmp sge",
            BinaryOp::ComparisonOp(ComparisonOp::Gt) => "icmp sgt",
            BinaryOp::ComparisonOp(ComparisonOp::Le) => "icmp sle",
            BinaryOp::ComparisonOp(ComparisonOp::Lt) => "icmp slt",
        };

        writeln!(
            self.buffer,
            "{}%{} = {} {} {}, {}",
            " ".repeat(INDENTATION),
            ssa_id,
            op_kw,
            self.get_llvm_ty(binary_node.op_ty),
            lhs_op,
            rhs_op
        )
    }
}

pub struct CodeGen<'icfg> {
    icfg: &'icfg Icfg,
}

impl<'icfg> CodeGen<'icfg> {
    pub fn new(icfg: &'icfg Icfg) -> Self {
        Self { icfg }
    }

    pub fn gen_code(self, file_name: &str, resolved_information: &ResolvedInformation) {
        use std::io::Write;

        let file_name_with_extension = format!("{}.ll", file_name);

        let mut buffer = String::with_capacity(4096);

        for cfg in self.icfg.cfgs.iter() {
            CodeGenUnit::new(cfg, resolved_information).gen_code(&mut buffer);
        }

        let mut file = File::create(&file_name_with_extension).expect("Error creating file");
        file.write_all(buffer.as_bytes()).expect("Error writing to file");
        Command::new("clang")
            .arg(&file_name_with_extension)
            .arg("-o")
            .arg(&file_name)
            .output()
            .expect("Error compiling file");
    }
}
