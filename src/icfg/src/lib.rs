/*

Useful links:
- https://rustc-dev-guide.rust-lang.org/appendix/background.html#cfg
- 

*/

use std::fmt::Display;

use data_structures::Either;
use derive_new::new;
use ir_defs::Mutability;
use op::{ ArithmeticOp, BinaryOp };
use span::Span;
use symbol::Symbol;
use ty::Ty;
mod icfg_prettifier;
mod cfg_visitor;

pub use icfg_prettifier::IcfgPrettifier;
pub use cfg_visitor::*;

enum Liveness {
    Alive,
    Dead,
}

pub struct Icfg {
    pub cfgs: Vec<Cfg>,
}
impl Icfg {
    pub fn new(cfgs: Vec<Cfg>) -> Self {
        Self { cfgs }
    }
}

/// One Cfg is constructed for each function
pub struct Cfg {
    /// All variables used in the function
    pub local_mems: Vec<LocalMem>,
    pub result_mems: Vec<ResultMem>,
    pub basic_blocks: Vec<BasicBlock>,
    /// Based on if the function is called or not
    liveness: Liveness,
}

impl Cfg {
    pub fn new(
        local_mems: Vec<LocalMem>,
        result_mems: Vec<ResultMem>,
        basic_blocks: Vec<BasicBlock>
    ) -> Self {
        Self { local_mems, result_mems, basic_blocks, liveness: Liveness::Dead }
    }

    pub fn get_local_mem(&self, local_mem_id: LocalMemId) -> &LocalMem {
        self.local_mems.get(local_mem_id.0 as usize).expect("Expected LocalMem")
    }

    pub fn get_result_mem(&self, result_mem_id: ResultMemId) -> &ResultMem {
        self.result_mems.get(result_mem_id.0 as usize).expect("Expected LocalMem")
    }
}

/// Implement `requires_drop` for when heap objects is implemented
#[derive(Debug, Clone, Copy, new)]
pub struct LocalMem {
    pub local_mem_id: LocalMemId,
    pub symbol: Symbol,
    pub span: Span,
    pub ty: Ty,
    pub mutability: Mutability,
}

impl Display for LocalMem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.symbol.get(), get_subscript(self.local_mem_id.0))
    }
}

#[derive(Debug, Clone, Copy, new)]
pub struct ResultMem {
    pub result_mem_id: ResultMemId,
    pub ty: Ty,
}

impl Display for ResultMem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.result_mem_id)
    }
}
fn get_subscript(mem: u32) -> String {
    let mut subscript = String::new();
    for char in mem.to_string().chars() {
        subscript += match char {
            '0' => "₀",
            '1' => "₁",
            '2' => "₂",
            '3' => "₃",
            '4' => "₄",
            '5' => "₅",
            '6' => "₆",
            '7' => "₇",
            '8' => "₈",
            '9' => "₉",
            _ => unreachable!(),
        };
    }

    subscript
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct LocalMemId(pub u32);

impl Display for LocalMemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct ResultMemId(pub u32);

impl Display for ResultMemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r{}", self.0)
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct TempId(pub u32);

impl Display for TempId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "_{}", self.0)
    }
}

#[derive(Debug)]
pub struct BasicBlock {
    pub basic_block_id: BasicBlockId,
    pub nodes: Vec<Node>,
}

impl BasicBlock {
    pub fn new(basic_block_id: BasicBlockId) -> Self {
        Self {
            basic_block_id,
            nodes: Vec::with_capacity(8),
        }
    }

    pub fn push_node(&mut self, node: Node) {
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
pub struct Node {
    pub kind: NodeKind,
}

#[derive(Debug, Clone, Copy)]
pub enum NodeKind {
    BranchNode(BranchNode),
    BranchCondNode(BranchCondNode),
    BinaryNode(BinaryNode),
    StoreNode(StoreNode),
    LoadNode(LoadNode),
    IndexNode(IndexNode),
    ByteAccessNode(ByteAccessNode),
}

/// If used with LoadNode, it can access fields of structs and tuples
///
/// If used with StoreNode, it can write to fields of structs and tuples
///
/// LLVM instruction:
///
/// %{result_place} = getelementptr inbounds i8, ptr %{access_place}, i64 {byte_offset}
#[derive(Debug, new, Clone, Copy)]
pub struct ByteAccessNode {
    pub result_place: TempId,
    pub access_place: PlaceKind,
    pub byte_offset: usize,
}

/// Not implemented yet
#[derive(Debug, new, Clone, Copy)]
pub struct IndexNode {
    pub result_place: TempId,
    pub array_place: PlaceKind,
    pub place_ty: Ty,
    pub index: usize,
}

/// Used to load a value from the stack
///
/// LLVM instruction:
///
/// %{result_place} = load {ty}, ptr %{loc_id}
#[derive(Debug, new, Clone, Copy)]
pub struct LoadNode {
    pub result_place: TempId,
    pub loc_id: Either<LocalMemId, ResultMemId>,
    pub ty: Ty,
}

/// Used to goto either one of two basic blocks based on a condition
///
/// LLVM instruction:
///
/// br i1 {condition}, label %{true_branch}, label %{false_branch}
#[derive(Debug, new, Clone, Copy)]
pub struct BranchCondNode {
    pub condition: Operand,
    /// Right now only Bool
    pub ty: Ty,
    pub true_branch: BasicBlockId,
    pub false_branch: BasicBlockId,
}

/// Unconditional goto
///
/// LLVM instruction:
///
/// br label %{branch}
#[derive(Debug, new, Clone, Copy)]
pub struct BranchNode {
    pub branch: BasicBlockId,
}

/// Used to write to some place (e.g. value on stack, field of tuple/struct, etc.)
///
/// LLVM instruction:
///
/// store {op_ty} {value}, ptr %{setter}
#[derive(Debug, new, Clone, Copy)]
pub struct StoreNode {
    pub setter: PlaceKind,
    pub op_ty: Ty,
    pub value: Operand,
}

/// Translates to any binary instruction. That is, either an arithmetic instruction or a comparison
///
/// LLVM instruction:
///
/// %{result_place} = {op} {op_ty} {lhs}, {rhs}
#[derive(Debug, new, Clone, Copy)]
pub struct BinaryNode {
    pub result_place: TempId,
    /// This should be removed in the future
    pub result_ty: Ty,
    pub op_ty: Ty,
    pub op: BinaryOp,
    pub lhs: Operand,
    pub rhs: Operand,
}

/// CHANGE:
/// Remove type from this (necessary type information is already in nodes most of the times)
///
/// REASON:
/// A lot of the times the necessary information is already inside the node,
/// so the type in this struct is essentially 24 bytes wasted (e.g. BinaryNode could save 48 bytes from this)
#[derive(Debug, Clone, Copy, new)]
pub struct Operand {
    pub kind: OperandKind,
    pub ty: Ty,
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} as {})", self.kind, self.ty)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OperandKind {
    Place(TempId),
    Const(Const),
}

impl Display for OperandKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Place(place) => write!(f, "{}", place),
            Self::Const(const_val) => write!(f, "{}", const_val),
        }
    }
}

impl From<TempId> for OperandKind {
    fn from(value: TempId) -> Self {
        Self::Place(value)
    }
}

impl From<i64> for OperandKind {
    fn from(value: i64) -> Self {
        Self::Const(Const::Int(value))
    }
}

impl From<bool> for OperandKind {
    fn from(value: bool) -> Self {
        Self::Const(Const::Bool(value))
    }
}

impl Default for OperandKind {
    fn default() -> Self {
        OperandKind::Const(Const::default())
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub enum PlaceKind {
    LocalMemId(LocalMemId),
    ResultMemId(ResultMemId),
    TempId(TempId),
}

impl PlaceKind {
    pub fn get_id(&self) -> usize {
        let id = match self {
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
    Void,
}

impl Display for Const {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(bool) => write!(f, "{}", bool),
            Self::Int(int) => write!(f, "{}", int),
            Self::Void => write!(f, "()"),
        }
    }
}

impl Default for Const {
    fn default() -> Self {
        Self::Void
    }
}
