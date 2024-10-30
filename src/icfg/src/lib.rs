/*

Useful links:
- https://rustc-dev-guide.rust-lang.org/appendix/background.html#cfg
- 

*/

use std::{ cell::RefCell, fmt::Display, rc::Rc };

use data_structures::Either;
use derive_new::new;
use op::{ ArithmeticOp, BinaryOp };
use span::Span;
use ir::{
    CfgFnKind,
    DefId,
    GlobalMem,
    GlobalMemId,
    LocalMem,
    LocalMemId,
    Mutability,
    ResolvedInformation,
    ResultMem,
    ResultMemId,
    Symbol,
    TempId,
    Ty,
    TyCtx,
    BOOL_TY,
    INT_TY,
    STR_TY,
    VOID_TY,
};
mod icfg_prettifier;
mod cfg_visitor;
mod cfg_analyzer;

pub use icfg_prettifier::IcfgPrettifier;
pub use cfg_visitor::*;
pub use cfg_analyzer::*;

pub enum Liveness {
    Alive,
    Dead,
}

pub struct Icfg<'a> {
    pub cfgs: Vec<Cfg<'a>>,
    pub global_mems: &'a RefCell<Vec<GlobalMem>>,
    pub resolved_information: ResolvedInformation<'a>,
    pub clib_fns: Vec<DefId>,
}
impl<'a> Icfg<'a> {
    pub fn new(
        cfgs: Vec<Cfg<'a>>,
        global_mems: &'a RefCell<Vec<GlobalMem>>,
        resolved_information: ResolvedInformation<'a>,
        clib_fns: Vec<DefId>
    ) -> Self {
        Self { cfgs, global_mems, resolved_information, clib_fns }
    }

    // pub fn analyze(&mut self) {
    //     for cfg in self.cfgs.iter_mut() {
    //         let mut analyzer = CfgAnalyzer { cfg };
    //         analyzer.visit_cfg();
    //     }
    // }
}

/// One Cfg is constructed for each function
pub struct Cfg<'a> {
    /// All variables used or referenced in the function
    pub global_mems: &'a RefCell<Vec<GlobalMem>>,
    pub args: Vec<(TempId, Ty)>,
    pub local_mems: Vec<LocalMem>,
    pub result_mems: Vec<ResultMem>,
    pub basic_blocks: Vec<BasicBlock<'a>>,
    /// Based on if the function is called or not
    liveness: Liveness,
    pub cfg_fn_kind: CfgFnKind,
    pub ret_ty: Ty,
}

impl<'a> Cfg<'a> {
    pub fn new(
        global_mems: &'a RefCell<Vec<GlobalMem>>,
        args: Vec<(TempId, Ty)>,
        local_mems: Vec<LocalMem>,
        result_mems: Vec<ResultMem>,
        basic_blocks: Vec<BasicBlock<'a>>,
        cfg_fn_kind: CfgFnKind,
        ret_ty: Ty
    ) -> Self {
        Self {
            global_mems,
            args,
            local_mems,
            result_mems,
            basic_blocks,
            cfg_fn_kind,
            ret_ty,
            liveness: Liveness::Dead,
        }
    }

    pub fn get_global_mem(&self, global_mem_id: GlobalMemId) -> GlobalMem {
        *self.global_mems
            .borrow()
            .get(global_mem_id.0 as usize)
            .expect("Expected GlobalMem")
    }

    pub fn get_local_mem(&self, local_mem_id: LocalMemId) -> &LocalMem {
        self.local_mems.get(local_mem_id.0 as usize).expect("Expected LocalMem")
    }

    pub fn get_result_mem(&self, result_mem_id: ResultMemId) -> &ResultMem {
        self.result_mems.get(result_mem_id.0 as usize).expect("Expected LocalMem")
    }
}

#[derive(Debug)]
pub struct BasicBlock<'a> {
    pub basic_block_id: BasicBlockId,
    pub nodes: Vec<Node<'a>>,
}

impl<'a> BasicBlock<'a> {
    pub fn new(basic_block_id: BasicBlockId) -> Self {
        Self {
            basic_block_id,
            nodes: Vec::with_capacity(8),
        }
    }

    pub fn push_node(&mut self, node: Node<'a>) {
        self.nodes.push(node);
    }
}
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct BasicBlockId(pub u32);

pub struct LocalMemAccess {
    pub mem: LocalMemId,
    pub access_kind: AccessKind,
}

pub enum AccessKind {
    Read,
    Write,
}

#[derive(Debug, new, Clone, Copy)]
pub struct Node<'a> {
    pub kind: NodeKind<'a>,
}

#[derive(Debug, Clone, Copy)]
pub enum NodeKind<'a> {
    BranchNode(BranchNode),
    BranchCondNode(BranchCondNode),
    BinaryNode(BinaryNode),
    StoreNode(StoreNode),
    LoadNode(LoadNode),
    IndexNode(IndexNode),
    ByteAccessNode(ByteAccessNode),
    ReturnNode(ReturnNode),
    CallNode(CallNode<'a>),
}

/// A hint to the optimizer whether or not the store is used for initializing a complicated data structure
/// (e.g. a tuple, struct etc.) or if it's an assignment (the data structure has already been initialized)
///
/// **EXAMPLE:**
///
/// This code:
/// ```
/// mut tuple := (1, 2, 3)
/// tuple.1 = 8
/// ```
/// Translates to (not valid code):
/// ```
/// mut tuple := Int(i32 * 3)
/// tuple.0 = 1 // (StoreKind::Init)
/// tuple.1 = 2 // (StoreKind::Init)
/// tuple.2 = 3 // (StoreKind::Init)
/// tuple.1 = 8 // (StoreKind::Assign)
/// ```
#[derive(Debug, Clone, Copy)]
pub enum StoreKind {
    /// Doesn't require target to be mutable
    Init,
    /// Requires target to be mutable
    Assign,
}

/// If used with LoadNode, it can access fields of structs and tuples
///
/// If used with StoreNode, it can write to fields of structs and tuples
///
/// LLVM instruction:
///
/// `%{result_place} = getelementptr inbounds i8, ptr %{access_place}, i64 {byte_offset}`
#[derive(Debug, new, Clone, Copy)]
pub struct ByteAccessNode {
    pub result_place: PlaceKind,
    pub access_place: PlaceKind,
    pub byte_offset: usize,
}

/// Used to return a value from a function
///
/// LLVM instruction:
///
/// `ret {ret_ty} {ret_val}`
#[derive(Debug, new, Clone, Copy)]
pub struct ReturnNode {
    pub ret_val: Operand,
    pub ret_ty: Ty,
}

/// Used to call a function
///
/// LLVM instruction:
///
/// `%{result_place} = call {ret_ty} {fn_name}({args})`
#[derive(Debug, new, Clone, Copy)]
pub struct CallNode<'a> {
    pub result_place: TempId,
    pub callee: Operand,
    pub args: &'a [Operand],
    pub args_ty: &'a [Ty],
    pub ret_ty: Ty,
}

/// Different from `ByteAccessNode` as this is only used with actual indexing supplied by the user.
/// E.g. `indexableOperand[2]`
#[derive(Debug, new, Clone, Copy)]
pub struct IndexNode {
    pub result_place: TempId,
    pub array_place: PlaceKind,
    pub place_ty: Ty,
    pub index: Operand,
}

/// Used to load a value from the stack
///
/// LLVM instruction:
///
/// `%{result_place} = load {load_ty}, ptr %{load_place}`
#[derive(Debug, new, Clone, Copy)]
pub struct LoadNode {
    pub result_place: TempId,
    pub load_place: PlaceKind,
    pub load_ty: Ty,
}

/// Used to goto either one of two basic blocks based on a condition
///
/// LLVM instruction:
///
/// `br i1 {condition}, label %{true_branch}, label %{false_branch}`
#[derive(Debug, new, Clone, Copy)]
pub struct BranchCondNode {
    pub condition: Operand,
    pub true_branch: BasicBlockId,
    pub false_branch: BasicBlockId,
}

/// Unconditional goto
///
/// LLVM instruction:
///
/// `br label %{branch}`
#[derive(Debug, new, Clone, Copy)]
pub struct BranchNode {
    pub branch: BasicBlockId,
}

/// Used to write to some place (e.g. value on stack, field of tuple/struct, etc.)
///
/// LLVM instruction:
///
/// `store {op_ty} {value}, ptr %{setter}`
#[derive(Debug, new, Clone, Copy)]
pub struct StoreNode {
    pub setter: PlaceKind,
    pub op_ty: Ty,
    pub value: Operand,
    pub store_kind: StoreKind,
}

/// Translates to any binary instruction. That is, either an arithmetic instruction or a comparison
///
/// LLVM instruction:
///
/// `%{result_place} = {op} {op_ty} {lhs}, {rhs}`
#[derive(Debug, new, Clone, Copy)]
pub struct BinaryNode {
    pub result_place: TempId,
    pub op_ty: Ty,
    pub op: BinaryOp,
    pub lhs: Operand,
    pub rhs: Operand,
}

#[derive(Debug, Clone, Copy)]
pub enum Operand {
    PlaceKind(PlaceKind),
    Const(Const),
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlaceKind(place) => write!(f, "{:?}", place),
            Self::Const(const_val) => write!(f, "{}", const_val),
        }
    }
}

impl From<TempId> for Operand {
    fn from(value: TempId) -> Self {
        Self::PlaceKind(PlaceKind::TempId(value))
    }
}

impl From<i64> for Operand {
    fn from(value: i64) -> Self {
        Self::Const(Const::Int(value))
    }
}

impl From<bool> for Operand {
    fn from(value: bool) -> Self {
        Self::Const(Const::Bool(value))
    }
}

impl Default for Operand {
    fn default() -> Self {
        Operand::Const(Const::default())
    }
}

/// LocalMemId: Memory location of an explicit variable (e.g. `a := value`)
///
/// ResultMemId: Memory location of an implicit variable (e.g. the result of an `IfExpr` or a `TupleExpr`)
///
/// TempId: Memory location usable only once and occurs as a result of many instructions (e.g. `tempId = 2 + 8` or `tempId = load a`)
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum PlaceKind {
    // GlobalMemId(GlobalMemId),
    LocalMemId(LocalMemId),
    ResultMemId(ResultMemId),
    TempId(TempId),
}

impl PlaceKind {
    pub fn get_id(&self) -> usize {
        let id = match self {
            // Self::GlobalMemId(global_mem_id) => global_mem_id.0,
            Self::LocalMemId(local_mem_id) => local_mem_id.0,
            Self::ResultMemId(result_mem_id) => result_mem_id.0,
            Self::TempId(temp_id) => temp_id.0,
        };

        id as usize
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Const {
    Int(i64),
    Bool(bool),
    FnPtr(DefId),
    Str(DefId),
    Null,
    Void,
}

impl Const {
    pub fn get_ty(&self) -> Ty {
        match self {
            Self::Void => VOID_TY,
            Self::Int(_) => INT_TY,
            Self::Bool(_) => BOOL_TY,
            Self::Null => Ty::Null,
            Self::FnPtr(def_id) => Ty::FnDef(*def_id),
            Self::Str(_) => STR_TY,
        }
    }
}

impl Display for Const {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(bool) => write!(f, "{}", bool),
            Self::Int(int) => write!(f, "{}", int),
            Self::Void => write!(f, "()"),
            Self::Null => write!(f, "null"),
            Self::FnPtr(def_id) => write!(f, "{}", def_id.display_as_fn()),
            Self::Str(def_id) => write!(f, "{}", def_id.display_as_str()),
        }
    }
}

impl Default for Const {
    fn default() -> Self {
        Self::Void
    }
}
