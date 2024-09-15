use std::{ cell::{ LazyCell, RefCell }, hash::{ Hash, Hasher } };

use data_structures::FxIndexSet;
use fxhash::FxHashMap;
use typed_arena::Arena;

thread_local! {
    static TYPE_ARENA: LazyCell<Arena<Ty>> = LazyCell::new(|| Arena::new());
}

fn with_type_arena<T>(f: impl FnOnce(&Arena<Ty>) -> T) -> T {
    TYPE_ARENA.with(move |type_arena: &LazyCell<Arena<Ty>>| f(type_arena))
}

pub struct TyCtx<'ctx> {
    interned_types: FxIndexSet<&'ctx Ty>,
    node_id_to_ty: FxHashMap<NodeId, &'ctx Ty>,
    // May be needed when multithreading is implemented
    // global_ty_ctx: &'ctx GlobalCtx<'ctx>,
}

impl<'ctx> TyCtx<'ctx> {
    pub fn new() -> Self {
        Self {
            interned_types: Default::default(),
            node_id_to_ty: Default::default(),
        }
    }

    pub fn set_type_to_node(&mut self, node_id: NodeId, ty: Ty) -> &'ctx Ty {
        if let Some(found_type) = self.interned_types.get(&ty) {
            return *found_type;
        }

        // This is safe, because he arena lives as long as the program
        let interned_type = with_type_arena(|type_arena| unsafe {
            &*(type_arena.alloc(ty) as *mut Ty)
        });
        self.node_id_to_ty.insert(node_id, interned_type);
        self.interned_types.insert(interned_type);

        interned_type
    }
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
/// NodeId is used both as AstNodeId and CfgNodeId
pub struct NodeId(pub u32);

#[derive(Hash, Eq, PartialEq)]
pub enum Ty {
    PrimTy(PrimTy),
    Unkown,
}

#[derive(Hash, Eq, PartialEq)]
pub enum PrimTy {
    Int,
    Void,
}
