use std::{ fmt::Debug, ops::Deref };

use bumpalo_herd::{ Herd, Member };

#[derive(Default)]
pub struct AstArena {
    ast_arena: Herd,
}

impl Deref for AstArena {
    type Target = Herd;

    fn deref(&self) -> &Self::Target {
        &self.ast_arena
    }
}

impl AstArena {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self) -> AstArenaObject<'_> {
        AstArenaObject {
            member: self.ast_arena.get(),
        }
    }
}


pub struct AstArenaObject<'a> {
    member: Member<'a>,
}

impl<'a> Deref for AstArenaObject<'a> {
    type Target = Member<'a>;

    fn deref(&self) -> &Self::Target {
        &self.member
    }
}

impl<'a> AstArenaObject<'a> {
    /// Only allocate objects that doesn't require its Drop implementation to be run
    pub fn alloc_expr_or_stmt<T>(&self, expr: T) -> &'a T {
        self.member.alloc(expr)
    }

    pub fn alloc_vec<T>(&self, vec: Vec<T>) -> &'a [T] {
        self.member.alloc_slice_fill_iter(vec)
    }
}

impl Debug for AstArena {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<AstArena>")
    }
}
