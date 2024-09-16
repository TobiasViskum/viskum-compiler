use std::fmt::Debug;
use bumpalo::Bump;
use typed_arena::Arena;

use crate::Stmt;

pub struct AstArena<'ast> {
    ast_arena: Bump,
    vec_stmts_arena: Arena<Vec<&'ast Stmt<'ast>>>,
}

impl<'ast> AstArena<'ast> {
    pub fn new() -> Self {
        Self {
            ast_arena: Bump::new(),
            vec_stmts_arena: Arena::new(),
        }
    }

    /// Only allocate objects that doesn't require its Drop implementation to be run
    ///
    /// That's why Copy is required, just to make sure
    pub fn alloc_expr_or_stmt<T>(&self, expr: T) -> &T {
        self.ast_arena.alloc(expr)
    }

    /// Allocates a vector of stmts in a TypedArena instead of Bump
    pub fn alloc_vec_stmts(&self, vec: Vec<&'ast Stmt<'ast>>) -> &[&'ast Stmt<'ast>] {
        self.vec_stmts_arena.alloc(vec)
    }
}

impl<'ast> Debug for AstArena<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<AstArena>")
    }
}
