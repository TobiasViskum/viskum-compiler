use std::{ fmt::Debug, u16 };
use bumpalo::Bump;
use typed_arena::Arena;

use crate::{ Expr, Stmt };

pub struct AstArena {
    ast_arena: Bump,
}

impl AstArena {
    pub fn new() -> Self {
        Self {
            ast_arena: Bump::new(),
        }
    }

    /// Only allocate objects that doesn't require its Drop implementation to be run
    pub fn alloc_expr_or_stmt<T>(&self, expr: T) -> &T {
        self.ast_arena.alloc(expr)
    }

    /// In the future the
    pub fn alloc_vec_stmts<'ast>(&self, vec: Vec<Stmt<'ast>>) -> &[Stmt<'ast>] {
        self.ast_arena.alloc_slice_fill_iter(vec.into_iter().map(|stmt| stmt))
    }

    pub fn alloc_vec_exprs<'ast>(&self, vec: Vec<Expr<'ast>>) -> &[Expr<'ast>] {
        self.ast_arena.alloc_slice_fill_iter(vec.into_iter().map(|expr| expr))
    }
}

impl Debug for AstArena {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<AstArena>")
    }
}
