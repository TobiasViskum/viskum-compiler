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

    pub fn alloc_expr_or_stmt<T>(&self, expr: T) -> &T where T: Copy {
        self.ast_arena.alloc(expr)
    }

    pub fn alloc_vec_stmts(&self, vec: Vec<&'ast Stmt<'ast>>) -> &[&'ast Stmt<'ast>] {
        self.vec_stmts_arena.alloc(vec)
    }
}

impl<'ast> Debug for AstArena<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<AstArena>")
    }
}
