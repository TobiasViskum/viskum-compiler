use std::fmt::Debug;

use bumpalo::Bump;

pub struct AstArena {
    ast_arena: Bump,
}

impl Default for AstArena {
    fn default() -> Self {
        Self {
            ast_arena: Default::default(),
        }
    }
}

impl AstArena {
    pub fn new() -> Self {
        Default::default()
    }

    /// Only allocate objects that doesn't require its Drop implementation to be run
    pub fn alloc_expr_or_stmt<T>(&self, expr: T) -> &T {
        self.ast_arena.alloc(expr)
    }

    pub fn alloc_vec<T>(&self, vec: Vec<T>) -> &[T] {
        self.ast_arena.alloc_slice_fill_iter(vec.into_iter())
    }
}

impl Debug for AstArena {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<AstArena>")
    }
}
