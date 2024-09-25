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

pub struct Icfg<'ctx> {
    pub cfgs: Vec<Cfg<'ctx>>,
}
impl<'ctx> Icfg<'ctx> {
    pub fn new(cfgs: Vec<Cfg<'ctx>>) -> Self {
        Self { cfgs }
    }
}

/// One Cfg is constructed for each function
pub struct Cfg<'ctx> {
    /// All variables used in the function
    local_mems: Vec<LocalMem<'ctx>>,
    result_mems: Vec<ResultMem<'ctx>>,
    basic_blocks: Vec<BasicBlock<'ctx>>,
    /// Based on if the function is called or not
    liveness: Liveness,
}

impl<'ctx> Cfg<'ctx> {
    pub fn new(
        local_mems: Vec<LocalMem<'ctx>>,
        result_mems: Vec<ResultMem<'ctx>>,
        basic_blocks: Vec<BasicBlock<'ctx>>
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
pub struct LocalMem<'ctx> {
    pub local_mem_id: LocalMemId,
    pub symbol: Symbol,
    pub span: Span,
    pub ty: &'ctx Ty,
    pub mutability: Mutability,
}

impl<'ctx> Display for LocalMem<'ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.symbol.get(), get_subscript(self.local_mem_id.0))
    }
}

#[derive(Debug, Clone, Copy, new)]
pub struct ResultMem<'ctx> {
    pub result_mem_id: ResultMemId,
    pub ty: &'ctx Ty,
}

impl<'ctx> Display for ResultMem<'ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.result_mem_id.0)
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
        write!(f, "_{}", self.0)
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
pub struct BasicBlock<'ctx> {
    pub basic_block_id: BasicBlockId,
    pub nodes: Vec<Node<'ctx>>,
}

impl<'ctx> BasicBlock<'ctx> {
    pub fn new(basic_block_id: BasicBlockId) -> Self {
        Self {
            basic_block_id,
            nodes: Vec::with_capacity(8),
        }
    }

    pub fn push_node(&mut self, node: Node<'ctx>) {
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
pub struct Node<'ctx> {
    pub kind: NodeKind<'ctx>,
}

#[derive(Debug, Clone, Copy)]
pub enum NodeKind<'ctx> {
    BranchNode(BranchNode),
    BranchCondNode(BranchCondNode<'ctx>),
    BinaryNode(BinaryNode<'ctx>),
    StoreNode(StoreNode<'ctx>),
    LoadNode(LoadNode<'ctx>),
}

#[derive(Debug, new, Clone, Copy)]
pub struct LoadNode<'ctx> {
    pub result_place: TempId,
    pub loc_id: Either<LocalMemId, ResultMemId>,
    pub ty: &'ctx Ty,
}

#[derive(Debug, new, Clone, Copy)]
pub struct BranchCondNode<'ctx> {
    pub condition: Operand<'ctx>,
    /// Right now only Bool
    pub ty: &'ctx Ty,
    pub true_branch: BasicBlockId,
    pub false_branch: BasicBlockId,
}

#[derive(Debug, new, Clone, Copy)]
pub struct BranchNode {
    pub branch: BasicBlockId,
}

#[derive(Debug, new, Clone, Copy)]
pub struct StoreNode<'ctx> {
    pub setter: Either<LocalMemId, ResultMemId>,
    pub result_ty: &'ctx Ty,
    pub value: Operand<'ctx>,
}

#[derive(Debug, new, Clone, Copy)]
pub struct BinaryNode<'ctx> {
    pub result_place: TempId,
    /// This should be removed in the future
    pub result_ty: &'ctx Ty,
    pub op_ty: &'ctx Ty,
    pub op: BinaryOp,
    pub lhs: Operand<'ctx>,
    pub rhs: Operand<'ctx>,
}

#[derive(Debug, Clone, Copy, new)]
pub struct Operand<'ctx> {
    pub kind: OperandKind,
    pub ty: &'ctx Ty,
}

impl<'ctx> Display for Operand<'ctx> {
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
