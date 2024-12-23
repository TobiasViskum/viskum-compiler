use std::fmt::Debug;

use derive_new::new;
use ir::Ty;
use op::BinaryOp;

#[derive(Debug, Clone, Copy)]
pub enum StackLoc {
    Explicit(ExplicitStackLoc),
    Implicit(ImplicitStackLoc),
}

#[derive(Debug, Clone, Copy)]
pub enum Liveness {
    Live,
    Dead,
}

#[derive(Debug, Clone, Copy)]
pub enum FreeBlockId {}

/// Refers to an explicit variable in the program e.g. `a` in `a := 5`
#[derive(Debug, Clone, Copy)]
pub struct ExplicitStackLoc;

/// Refers to an implicit variable in the program e.g. the result of conditional expressions e.g. `if true { 1 } else { 0 }`
#[derive(Debug, Clone, Copy)]
pub struct ImplicitStackLoc;

pub struct Icfg {}

/*

A Cfg is represented as an adjacency list, where each node is a basic block and each edge is a control flow edge.
A Cfg may be cyclic

A CfgEdge is a directed edge between two basic blocks. It may return a data location which can be used in the next basic block.

*/

#[derive(Debug, Clone, Copy)]
pub enum TraverseDirection {
    /// Traverse the graph in the forward direction (START .. prev -> node -> next .. END)
    Outgoing,
    /// Traverse the graph in the backward direction (START .. prev <- node <- next .. END)
    Incoming,
}

pub trait Index {
    fn get_index(&self) -> usize;
}

#[derive(Debug, Clone, Copy)]
pub struct CfgEdgeIndex(pub u32);

impl Index for CfgEdgeIndex {
    fn get_index(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BbEdgeIndex(pub u32);

impl Index for BbEdgeIndex {
    fn get_index(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CfgNodeIndex(pub u32);

impl Index for CfgNodeIndex {
    fn get_index(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BbNodeIndex(pub u32);

impl Index for BbNodeIndex {
    fn get_index(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StackLocEdge {
    pub to: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct StackLocEdgeIndex(pub u32);

impl Index for StackLocEdgeIndex {
    fn get_index(&self) -> usize {
        self.0 as usize
    }
}

/// A control flow graph
pub struct Cfg<'a> {
    pub stack_locs: Vec<StackLoc>,
    pub nodes: Vec<CfgNode<'a>>,
    pub edges: Vec<CfgEdge>,
    pub stack_loc_indexes: Vec<StackLocEdge>,
}

/// A CfgNode is a basic block in the control flow graph
pub struct CfgNode<'a> {
    pub basic_block: BasicBlock<'a>,
}

/// A CfgEdge is a directed edge between two basic blocks
pub struct CfgEdge {
    pub from: CfgNodeIndex,
    pub to: CfgNodeIndex,
}

#[derive(Debug, new, Clone, Copy)]
pub struct Incoming<T: Index> {
    kind: T,
}

#[derive(Debug, new, Clone, Copy)]
pub struct Outgoing<T: Index> {
    kind: T,
}

/// A basic block contains a list of instructions that do not break control flow, however it may end with a branch instruction
///
/// The instructions are not necessarily in order, but the edges describes the "data flow" between the instructions
/// However, right after creation they will be in order, but the data flow (edges) may be reordered later (e.g. by an analysis)
pub struct BasicBlock<'a> {
    pub nodes: Vec<BasicBlockNode<'a>>,
    pub edges: Vec<BasicBlockEdge>,
}

pub struct BasicBlockNode<'a> {
    pub instr_kind: InstrKind<'a>,
}

pub enum Const {}

pub enum BasicBlockEdgeKind {
    ConstEdge(Const),
    InstrEdge(Incoming<BbNodeIndex>, Outgoing<BbNodeIndex>),
}

/// A BasicBlockEdge is a directed edge between two basic blocks
pub struct BasicBlockEdge(pub BasicBlockEdgeKind);

/// A high level mapping of an llvm instruction
pub enum InstrKind<'a> {
    StoreNode(StoreNode),
    BinaryNode(BinaryNode),
    TyCastNode(TyCastNode),
    CallNode(CallNode<'a>),
    ReturnNode(ReturnNode),
    BranchNode(BranchNode),
    CondBranchNode(CondBranchNode),
}

#[derive(Debug, Clone, Copy)]
pub enum StoreKind {
    /// Store a value through initialization
    Init,
    /// Store a value through assignment
    Assign,
}

#[derive(Debug, new, Clone, Copy)]
pub struct StoreNode {
    pub operand: Incoming<BbEdgeIndex>,
    pub store_kind: StoreKind,
    pub store_ty: Ty,
    pub store_loc: Outgoing<StackLocEdgeIndex>,
}

#[derive(Debug, new, Clone, Copy)]
pub struct BranchNode {
    pub bb_id: Outgoing<CfgEdgeIndex>,
}

#[derive(Debug, new, Clone, Copy)]
pub struct CondBranchNode {
    pub operand: Incoming<BbEdgeIndex>,
    pub true_bb_id: Outgoing<CfgEdgeIndex>,
    pub false_bb_id: Outgoing<CfgEdgeIndex>,
}

#[derive(Debug, new, Clone, Copy)]
pub struct ReturnNode {
    pub operand: Incoming<BbEdgeIndex>,
}

#[derive(Debug, new, Clone, Copy)]
pub struct BinaryNode {
    pub op: BinaryOp,
    pub op_ty: Ty,
    pub lhs: Incoming<BbEdgeIndex>,
    pub rhs: Incoming<BbEdgeIndex>,
}

#[derive(Debug, new, Clone, Copy)]
pub struct CallNode<'a> {
    pub args_ty: &'a [Ty],
    pub ret_ty: Ty,
}

#[derive(Debug, new, Clone, Copy)]
pub struct TyCastNode {
    pub operand: Incoming<BbEdgeIndex>,
    pub ty_cast_kind: TyCastKind,
}

pub trait TyCastSize: Clone + Copy + Debug {
    fn get_size() -> usize;
}

#[derive(Debug, Clone, Copy)]
pub struct Size8;

impl TyCastSize for Size8 {
    fn get_size() -> usize {
        8
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Size16;

impl TyCastSize for Size16 {
    fn get_size() -> usize {
        16
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Size32;

impl TyCastSize for Size32 {
    fn get_size() -> usize {
        32
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Size64;

impl TyCastSize for Size64 {
    fn get_size() -> usize {
        64
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TyCastOp<T: TyCastSize> {
    Int(T),
    Uint(T),
}

#[derive(Debug, Clone, Copy)]
pub enum TyCastKind {
    Size8toSize16(TyCastOp<Size8>, TyCastOp<Size16>),
    Size8toSize32(TyCastOp<Size8>, TyCastOp<Size32>),
    Size8toSize64(TyCastOp<Size8>, TyCastOp<Size64>),

    Size16toSize8(TyCastOp<Size16>, TyCastOp<Size8>),
    Size16toSize32(TyCastOp<Size16>, TyCastOp<Size32>),
    Size16toSize64(TyCastOp<Size16>, TyCastOp<Size64>),

    Size32toSize8(TyCastOp<Size16>, TyCastOp<Size8>),
    Size32toSize16(TyCastOp<Size16>, TyCastOp<Size16>),
    Size32toSize64(TyCastOp<Size16>, TyCastOp<Size64>),

    Size64toSize8(TyCastOp<Size16>, TyCastOp<Size8>),
    Size64toSize16(TyCastOp<Size16>, TyCastOp<Size16>),
    Size64toSize32(TyCastOp<Size16>, TyCastOp<Size32>),
}
