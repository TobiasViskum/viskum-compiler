use fxhash::FxHashMap;
use icfg::{
    walk_args,
    walk_basic_block,
    walk_basic_blocks,
    walk_local_mems,
    walk_result_mems,
    BasicBlockId,
    CallNode,
    Cfg,
    CfgVisitor,
    Const,
    Icfg,
    Operand,
    PlaceKind,
    ReturnNode,
    TyCastNode,
};
use op::{ ArithmeticOp, BinaryOp, ComparisonOp };
use ir::{
    CfgFnKind,
    DefId,
    Externism,
    GetTyAttr,
    IntTy,
    LocalMem,
    NameBindingKind,
    PrimTy,
    ResolvedInformation,
    ResultMem,
    TempId,
    Ty,
    UintTy,
    VOID_TY,
};
use threadpool::ThreadPool;
use threadpool_scope::scope_with;
use std::{ fmt::{ Display, Write }, fs::File, process::Command, sync::{ Arc, Mutex } };

const INDENTATION: usize = 4;

// pub(crate) fn get_ty_stack_size_in_bytes(ty: &Ty) -> usize {}

/// Used to allocate places (e.g. results of operations, variables, etc.) before the actual code gen
pub(crate) struct CodeGenUnitHelper<'a> {
    cfg: &'a Cfg<'a>,
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
            next_ssa_id: 0,
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

    fn visit_arg(&mut self, arg: &(TempId, Ty)) -> Self::Result {
        let next_ssa_id = self.get_next_ssa_id();
        self.place_to_ssa_id.insert(PlaceKind::TempId(arg.0), next_ssa_id);
    }

    fn visit_local_mem(&mut self, local_mem: &LocalMem) -> Self::Result {
        let next_ssa_id = self.get_next_ssa_id();
        self.place_to_ssa_id.insert(PlaceKind::LocalMemId(local_mem.local_mem_id), next_ssa_id);
    }

    fn visit_result_mem(&mut self, result_mem: &ResultMem) -> Self::Result {
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

    fn visit_call_node(&mut self, call_node: &CallNode, cfg: &Cfg) -> Self::Result {
        if call_node.ret_ty.is_void() {
            return;
        }
        let next_ssa_id = self.get_next_ssa_id();
        self.place_to_ssa_id.insert(PlaceKind::TempId(call_node.result_place), next_ssa_id);
    }

    fn visit_return_node(&mut self, return_node: &ReturnNode, cfg: &Cfg) -> Self::Result {
        self.next_ssa_id += 1;
    }

    fn visit_ty_cast_node(&mut self, ty_cast_node: &TyCastNode, cfg: &Cfg) -> Self::Result {
        let next_ssa_id = self.get_next_ssa_id();
        self.place_to_ssa_id.insert(PlaceKind::TempId(ty_cast_node.result_place), next_ssa_id);
    }
}

pub enum LLVMSSA {
    SSAId(usize),
    Global(GlobalSSA),
}

pub enum GlobalSSA {
    Fn(DefId),
    Str(DefId),
}

impl Display for GlobalSSA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlobalSSA::Fn(def_id) => write!(f, "{}", def_id.display_as_fn()),
            GlobalSSA::Str(def_id) => write!(f, "{}", def_id.display_as_str()),
        }
    }
}

impl Display for LLVMSSA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLVMSSA::SSAId(ssa_id) => write!(f, "%{}", ssa_id),
            LLVMSSA::Global(global_ssa) => write!(f, "{}", global_ssa),
        }
    }
}

pub(crate) struct CodeGenUnit<'a> {
    cfg: &'a Cfg<'a>,
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

    pub(crate) fn get_ssa_id_from_place(&self, place: &PlaceKind) -> LLVMSSA {
        LLVMSSA::SSAId(*self.place_to_ssa_id.get(place).expect("Expected place"))
    }

    pub(crate) fn get_bb_id(&self, bb_id: &BasicBlockId) -> usize {
        *self.basic_block_id_to_ssa_id.get(bb_id).expect("Expected BasicBlockId")
    }

    pub(crate) fn get_llvm_operand(&self, operand: &Operand) -> String {
        let string = match &operand {
            Operand::PlaceKind(place) => { format!("{}", self.get_ssa_id_from_place(place)) }
            Operand::Const(const_val) => {
                match const_val {
                    Const::FnPtr(def_id) => {
                        if self.resolved_information.is_clib_fn(def_id) {
                            format!("@{}", def_id.symbol.get())
                        } else {
                            def_id.display_as_fn()
                        }
                    }
                    Const::Str(def_id) => { def_id.display_as_str() }
                    Const::Bool(bool) =>
                        (
                            match bool {
                                true => "1",
                                false => "0",
                            }
                        ).to_string(),
                    Const::Null => "null".to_string(),
                    Const::Int(int, _) => int.to_string(),
                    Const::Void => panic!("Void cannot be used as an operand"),
                }
            }
        };
        string
    }
}

fn get_llvm_ty<'a>(ty: Ty, resolved_information: &ResolvedInformation<'a>) -> String {
    match &ty {
        Ty::PrimTy(prim_ty) => {
            match prim_ty {
                PrimTy::Bool => "i8".to_string(),
                PrimTy::Int(int_ty) => {
                    match int_ty {
                        IntTy::Int8 => "i8".to_string(),
                        IntTy::Int16 => "i16".to_string(),
                        IntTy::Int32 => "i32".to_string(),
                        IntTy::Int64 => "i64".to_string(),
                    }
                }
                PrimTy::Uint(uint_ty) => {
                    match uint_ty {
                        UintTy::Uint8 => "i8".to_string(),
                        UintTy::Uint16 => "i16".to_string(),
                        UintTy::Uint32 => "i32".to_string(),
                        UintTy::Uint64 => "i64".to_string(),
                    }
                }
                PrimTy::Float(float_ty) => todo!(),
                PrimTy::Void => "void".to_string(),
                PrimTy::Str => "ptr".to_string(),
            }
        }
        Ty::VariadicArgs => "...".to_string(),
        Ty::Ptr(_, _) => "ptr".to_string(),
        Ty::ManyPtr(_, _) => "ptr".to_string(),
        Ty::StackPtr(_, _) => "ptr".to_string(),
        Ty::Null => "ptr".to_string(),
        ty @ Ty::Tuple(_) => {
            let ty_attr = ty.get_ty_attr(resolved_information);
            format!("[{} x i8]", ty_attr.size_bytes)
        }
        ty @ Ty::Adt(_) => {
            let ty_attr = ty.get_ty_attr(resolved_information);
            format!("[{} x i8]", ty_attr.size_bytes)
        }
        Ty::FnSig(_) => "ptr".to_string(),
        Ty::FnDef(_) => "ptr".to_string(),
        Ty::AtdConstructer(_) =>
            panic!("AdtConstructer type (should not be this far in compilation)"),
        Ty::Package => panic!("Package type (should not be this far in compilation)"),
        t @ (Ty::Unkown | Ty::Never | Ty::ZeroSized) =>
            panic!("{} type (should not be this far in compilation)", t),
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
                    "define {} {}(",
                    get_llvm_ty(cfg.ret_ty, self.resolved_information),
                    def_id.display_as_fn()
                )?;

                for (i, (temp_id, arg_ty)) in cfg.args.iter().enumerate() {
                    write!(
                        self.buffer,
                        "{} noundef {}",
                        get_llvm_ty(*arg_ty, self.resolved_information),
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
        } else {
            writeln!(self.buffer, "{}unreachable", " ".repeat(INDENTATION))?;
        }
        writeln!(self.buffer, "}}")
    }

    fn visit_local_mem(&mut self, local_mem: &LocalMem) -> Self::Result {
        let ssa_id = self.get_ssa_id_from_place(&PlaceKind::LocalMemId(local_mem.local_mem_id));

        let ty_attr = local_mem.ty.get_ty_attr(self.resolved_information);

        writeln!(
            self.buffer,
            "{}{} = alloca [{} x i8], align {}",
            " ".repeat(INDENTATION),
            ssa_id,
            ty_attr.size_bytes,
            ty_attr.alignment_bytes
        )
    }

    fn visit_result_mem(&mut self, result_mem: &ResultMem) -> Self::Result {
        let ssa_id = self.get_ssa_id_from_place(&PlaceKind::ResultMemId(result_mem.result_mem_id));

        let ty_attr = result_mem.ty.get_ty_attr(self.resolved_information);

        writeln!(
            self.buffer,
            "{}{} = alloca [{} x i8], align {}",
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
            "{}store {} {}, ptr {}",
            " ".repeat(INDENTATION),
            get_llvm_ty(store_node.op_ty, self.resolved_information),
            val,
            var_place
        )
    }

    fn visit_index_node(&mut self, index_node: &icfg::IndexNode, cfg: &Cfg) -> Self::Result {
        let temp_id = self.get_ssa_id_from_place(&PlaceKind::TempId(index_node.result_place));
        let array_id = self.get_ssa_id_from_place(&index_node.array_place);

        writeln!(
            self.buffer,
            "{}{} = getelementptr inbounds {}, ptr {}, i64 {}",
            " ".repeat(INDENTATION),
            temp_id,
            get_llvm_ty(index_node.place_ty, self.resolved_information),
            array_id,
            self.get_llvm_operand(&index_node.index)
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
            "{}{} = getelementptr inbounds i8, ptr {}, i64 {}",
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
            "{}{} = load {}, ptr {}",
            " ".repeat(INDENTATION),
            ssa_id,
            get_llvm_ty(load_node.load_ty, self.resolved_information),
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
            BinaryOp::ArithmeticOp(ArithmeticOp::Div) => "sdiv",
            BinaryOp::ComparisonOp(ComparisonOp::Eq) => "icmp eq",
            BinaryOp::ComparisonOp(ComparisonOp::Ne) => "icmp ne",
            BinaryOp::ComparisonOp(ComparisonOp::Ge) => "icmp sge",
            BinaryOp::ComparisonOp(ComparisonOp::Gt) => "icmp sgt",
            BinaryOp::ComparisonOp(ComparisonOp::Le) => "icmp sle",
            BinaryOp::ComparisonOp(ComparisonOp::Lt) => "icmp slt",
        };

        writeln!(
            self.buffer,
            "{}{} = {} {} {}, {}",
            " ".repeat(INDENTATION),
            ssa_id,
            op_kw,
            get_llvm_ty(binary_node.op_ty, self.resolved_information),
            lhs_op,
            rhs_op
        )
    }

    fn visit_return_node(&mut self, return_node: &ReturnNode, cfg: &Cfg) -> Self::Result {
        match return_node.ret_ty {
            VOID_TY => writeln!(self.buffer, "{}ret void", " ".repeat(INDENTATION)),
            ty => {
                let return_operand = self.get_llvm_operand(&return_node.ret_val);
                writeln!(
                    self.buffer,
                    "{}ret {} {}",
                    " ".repeat(INDENTATION),
                    get_llvm_ty(ty, self.resolved_information),
                    return_operand
                )
            }
        }
    }

    fn visit_call_node(&mut self, call_node: &CallNode, cfg: &Cfg) -> Self::Result {
        let callee = self.get_llvm_operand(&call_node.callee);

        write!(self.buffer, "{}", " ".repeat(INDENTATION))?;

        if !call_node.ret_ty.is_void() {
            let ssa_id = self.get_ssa_id_from_place(&PlaceKind::TempId(call_node.result_place));
            write!(self.buffer, "{} = ", ssa_id)?;
        }

        write!(self.buffer, "call {} ", get_llvm_ty(call_node.ret_ty, self.resolved_information))?;

        write!(self.buffer, "(")?;
        for (i, arg_ty) in call_node.args_ty.iter().enumerate() {
            write!(self.buffer, "{}", get_llvm_ty(*arg_ty, self.resolved_information))?;
            if arg_ty.is_variadic_args() {
                break;
            }
            if i != call_node.args_ty.len() - 1 {
                write!(self.buffer, ", ")?;
            }
        }
        write!(self.buffer, ")")?;

        write!(self.buffer, " {}(", callee)?;

        let mut found_variadic_args = false;
        for (i, arg_ty) in call_node.args_ty.iter().enumerate() {
            let i = if found_variadic_args { i - 1 } else { i };

            if arg_ty.is_variadic_args() {
                found_variadic_args = true;
                continue;
            }

            let arg = &call_node.args[i];
            let arg_str = self.get_llvm_operand(arg);
            write!(
                self.buffer,
                "{} noundef {}",
                get_llvm_ty(*arg_ty, self.resolved_information),
                arg_str
            )?;
            if i != call_node.args.len() - 1 {
                write!(self.buffer, ", ")?;
            }
        }

        writeln!(self.buffer, ")")
    }

    fn visit_ty_cast_node(&mut self, ty_cast_node: &TyCastNode, cfg: &Cfg) -> Self::Result {
        writeln!(
            self.buffer,
            "{}{} = {} {} {} to {}",
            " ".repeat(INDENTATION),
            self.get_ssa_id_from_place(&PlaceKind::TempId(ty_cast_node.result_place)),
            ty_cast_node.cast_kind,
            get_llvm_ty(ty_cast_node.from_ty, self.resolved_information),
            self.get_llvm_operand(&ty_cast_node.operand),
            get_llvm_ty(ty_cast_node.to_ty, self.resolved_information)
        )
    }
}

pub struct CodeGen<'icfg> {
    icfg: &'icfg Icfg<'icfg>,
    threadpool: &'icfg ThreadPool,
}

impl<'icfg> CodeGen<'icfg> {
    pub fn new(icfg: &'icfg Icfg, threadpool: &'icfg ThreadPool) -> Self {
        Self { icfg, threadpool }
    }

    pub fn gen_code(&self, file_name: &str) {
        let now = std::time::Instant::now();

        let buffer = Mutex::new(String::with_capacity(65536));

        let iter = self.icfg.resolved_information.clib_fns.iter().map(|def_id| {
            let name_binding = self.icfg.resolved_information.get_name_binding_from_def_id(def_id);
            match name_binding.kind {
                NameBindingKind::Fn(fn_sig, _, Externism::Clib) => (def_id.symbol, fn_sig),
                _ => panic!("Expected extern function"),
            }
        });

        scope_with(self.threadpool, |s| {
            let buffer = &buffer;
            for (symbol, fn_sig) in iter {
                s.execute(move || {
                    let mut local_buffer = String::with_capacity(128);
                    write!(
                        local_buffer,
                        "declare {} @{}(",
                        get_llvm_ty(*fn_sig.ret_ty, &self.icfg.resolved_information),
                        symbol.get()
                    ).expect("Error writing to buffer");

                    for (i, arg_ty) in fn_sig.args.iter().enumerate() {
                        write!(
                            local_buffer,
                            "{}",
                            get_llvm_ty(*arg_ty, &self.icfg.resolved_information)
                        ).expect("Error writing to buffer");
                        if *arg_ty != Ty::VariadicArgs {
                            write!(local_buffer, " noundef").expect("Error writing to buffer");
                        }
                        if i != fn_sig.args.len() - 1 {
                            write!(local_buffer, ", ").expect("Error writing to buffer");
                        }
                    }

                    writeln!(local_buffer, ")").expect("Error writing to buffer");

                    buffer.lock().unwrap().push_str(&local_buffer);
                });
            }
        });

        // This part isn't parallelized because there's so little to do in each loop iteration (about 1% of the total time)
        {
            let mut locked_buffer = buffer.lock().unwrap();
            writeln!(locked_buffer).expect("Error writing to buffer");

            for (const_str, const_str_len) in self.icfg.resolved_information.const_strs.iter() {
                writeln!(
                    locked_buffer,
                    "{} = private unnamed_addr constant [{} x i8] c\"{}\"",
                    const_str.display_as_str(),
                    const_str_len.0,
                    const_str.symbol.get()
                ).expect("Error writing to buffer");
            }

            writeln!(locked_buffer).expect("Error writing to buffer");
        }

        scope_with(self.threadpool, |s| {
            let resolved_information = &self.icfg.resolved_information;

            for cfg in self.icfg.cfgs.iter() {
                let buffer = &buffer;
                s.execute(move || {
                    let mut local_buffer = String::with_capacity(1024);
                    CodeGenUnit::new(cfg, resolved_information).gen_code(&mut local_buffer);
                    buffer.lock().unwrap().push_str(&local_buffer);
                });
            }
        });

        let file_name_with_extension = format!("./viskum/dist/main.ll");
        {
            // Ensure directory exists
            std::fs::create_dir_all("./viskum/dist").expect("Error creating directory");

            let mut file = File::create(&file_name_with_extension).expect("Error creating file");
            use std::io::Write;
            file.write_all(buffer.lock().unwrap().as_bytes()).expect("Error writing to file");
        }

        println!("Code generation took: {:?}", now.elapsed());

        let result = Command::new("clang")
            .arg("-O0")
            .arg(&file_name_with_extension)
            .arg("-o")
            .arg("./viskum/dist/main")
            .output()
            .expect("Failed to execute clang");

        println!("{}", String::from_utf8(result.stderr).expect("Error converting to string"));
    }
}
