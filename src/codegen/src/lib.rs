use data_structures::Either;
use fxhash::FxHashMap;
use icfg::{
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
    LocalMemId,
    Operand,
    OperandKind,
    PlaceKind,
    ResultMemId,
    TempId,
};
use op::{ ArithmeticOp, BinaryOp, ComparisonOp };
use ty::{ PrimTy, Ty };
use std::{ fmt::Write, fs::File, process::Command };

const INDENTATION: usize = 4;

// pub(crate) fn get_ty_stack_size_in_bytes(ty: &Ty) -> usize {}

/// Used to allocate places (e.g. results of operations, variables, etc.) before the actual code gen
pub(crate) struct CodeGenUnitHelper<'a> {
    cfg: &'a Cfg,
    next_ssa_id: usize,
    place_to_ssa_id: FxHashMap<PlaceKind, usize>,
    basic_block_id_to_ssa_id: FxHashMap<BasicBlockId, usize>,
}

impl<'a> CodeGenUnitHelper<'a> {
    pub(crate) fn new(cfg: &'a Cfg) -> Self {
        Self {
            cfg,
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

    fn visit_basic_block(&mut self, basic_block: &icfg::BasicBlock, cfg: &Cfg) -> Self::Result {
        let next_ssa_id = self.get_next_ssa_id();
        self.basic_block_id_to_ssa_id.insert(basic_block.basic_block_id, next_ssa_id);
        walk_basic_block(self, basic_block, cfg)
    }

    fn visit_local_mem(&mut self, local_mem: &icfg::LocalMem) -> Self::Result {
        let next_ssa_id = self.get_next_ssa_id();
        self.place_to_ssa_id.insert(PlaceKind::LocalMemId(local_mem.local_mem_id), next_ssa_id);
    }

    fn visit_result_mem(&mut self, result_mem: &icfg::ResultMem) -> Self::Result {
        let next_ssa_id = self.get_next_ssa_id();
        self.place_to_ssa_id.insert(PlaceKind::ResultMemId(result_mem.result_mem_id), next_ssa_id);
    }

    fn visit_binary_node(&mut self, binary_node: &icfg::BinaryNode, cfg: &Cfg) -> Self::Result {
        let next_ssa_id = self.get_next_ssa_id();
        self.place_to_ssa_id.insert(PlaceKind::TempId(binary_node.result_place), next_ssa_id);
    }

    fn visit_load_node(&mut self, load_node: &icfg::LoadNode, cfg: &Cfg) -> Self::Result {
        let next_ssa_id = self.get_next_ssa_id();
        self.place_to_ssa_id.insert(PlaceKind::TempId(load_node.result_place), next_ssa_id);
    }

    fn visit_init_node(&mut self, _init_node: &icfg::StoreNode, _cfg: &Cfg) -> Self::Result {
        // Should do nothing
    }
}

pub(crate) struct CodeGenUnit<'a> {
    cfg: &'a Cfg,
    buffer: String,
    place_to_ssa_id: FxHashMap<PlaceKind, usize>,
    basic_block_id_to_ssa_id: FxHashMap<BasicBlockId, usize>,
}

impl<'a> CodeGenUnit<'a> {
    pub(crate) fn new(cfg: &'a Cfg) -> Self {
        let (place_to_ssa_id, basic_block_id_to_ssa_id) = {
            CodeGenUnitHelper::new(cfg).allocate_places()
        };

        Self {
            cfg,
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
        let str = match ty {
            Ty::PrimTy(prim_ty) =>
                match prim_ty {
                    PrimTy::Bool => "i1",
                    PrimTy::Int => "i32",
                    PrimTy::Void => "void",
                }
            Ty::Tuple(tuple_ty) => panic!("Tuples not implemented yet"),
            Ty::Unkown => panic!("Unkown type (should not be this far in compilation)"),
        };
        str.to_string()
    }

    // pub(crate) fn get_declaration_llvm_ty(&self, ty: Ty) -> String {
    //     let str = match ty {
    //         Ty::PrimTy(primt_ty) => {
    //             match primt_ty {
    //                 PrimTy::Bool =>
    //             }
    //         }
    //     }
    // }

    pub(crate) fn get_llvm_operand(&self, operand: &Operand) -> String {
        let string = match &operand.kind {
            OperandKind::Place(place) => {
                format!("%{}", self.get_ssa_id_from_place(&PlaceKind::TempId(*place)))
            }
            OperandKind::Const(const_val) => {
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
        writeln!(self.buffer, "define i32 @main() {{")?;
        walk_local_mems(self, cfg)?;
        walk_result_mems(self, cfg)?;

        let first_bb_id = self.basic_block_id_to_ssa_id
            .get(&BasicBlockId(0))
            .expect("Expected at least one BasicBlock");
        writeln!(self.buffer, "{}br label %{}", " ".repeat(INDENTATION), first_bb_id)?;

        walk_basic_blocks(self, cfg)?;
        writeln!(self.buffer, "{}ret i32 0", " ".repeat(INDENTATION))?;
        writeln!(self.buffer, "}}")
    }

    fn visit_local_mem(&mut self, local_mem: &icfg::LocalMem) -> Self::Result {
        let ssa_id = self.get_ssa_id_from_place(&PlaceKind::LocalMemId(local_mem.local_mem_id));
        writeln!(
            self.buffer,
            "{}%{} = alloca {}, align 4",
            " ".repeat(INDENTATION),
            ssa_id,
            self.get_llvm_ty(local_mem.ty)
        )
    }

    fn visit_result_mem(&mut self, result_mem: &icfg::ResultMem) -> Self::Result {
        let ssa_id = self.get_ssa_id_from_place(&PlaceKind::ResultMemId(result_mem.result_mem_id));

        writeln!(
            self.buffer,
            "{}%{} = alloca {}, align 4",
            " ".repeat(INDENTATION),
            ssa_id,
            self.get_llvm_ty(result_mem.ty)
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

    fn visit_init_node(&mut self, init_node: &icfg::StoreNode, cfg: &Cfg) -> Self::Result {
        let var_place = match init_node.setter {
            Either::Left(local_mem_id) =>
                self.get_ssa_id_from_place(&PlaceKind::LocalMemId(local_mem_id)),
            Either::Right(result_mem_id) =>
                self.get_ssa_id_from_place(&PlaceKind::ResultMemId(result_mem_id)),
        };
        let val = self.get_llvm_operand(&init_node.value);

        writeln!(
            self.buffer,
            "{}store {} {}, ptr %{}",
            " ".repeat(INDENTATION),
            self.get_llvm_ty(init_node.result_ty),
            val,
            var_place
        )
    }

    fn visit_branch_cond_node(
        &mut self,
        branch_cond_node: &icfg::BranchCondNode,
        cfg: &Cfg
    ) -> Self::Result {
        let cond = self.get_llvm_operand(&branch_cond_node.condition);
        let ty = self.get_llvm_ty(branch_cond_node.ty);
        let true_branch = self.get_bb_id(&branch_cond_node.true_branch);
        let false_branch = self.get_bb_id(&branch_cond_node.false_branch);

        writeln!(
            self.buffer,
            "{}br {} {}, label %{}, label %{}",
            " ".repeat(INDENTATION),
            ty,
            cond,
            true_branch,
            false_branch
        )
    }

    fn visit_load_node(&mut self, load_node: &icfg::LoadNode, cfg: &Cfg) -> Self::Result {
        let ssa_id = self.get_ssa_id_from_place(&PlaceKind::TempId(load_node.result_place));
        let var_place = match load_node.loc_id {
            Either::Left(local_mem_id) =>
                self.get_ssa_id_from_place(&PlaceKind::LocalMemId(local_mem_id)),
            Either::Right(result_mem_id) =>
                self.get_ssa_id_from_place(&PlaceKind::ResultMemId(result_mem_id)),
        };
        writeln!(
            self.buffer,
            "{}%{} = load {}, ptr %{}",
            " ".repeat(INDENTATION),
            ssa_id,
            self.get_llvm_ty(load_node.ty),
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

    pub fn gen_code(self, file_name: &str) {
        use std::io::Write;

        let file_name_with_extension = format!("{}.ll", file_name);

        let mut buffer = String::with_capacity(4096);
        CodeGenUnit::new(&self.icfg.cfgs[0]).gen_code(&mut buffer);

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
