use std::fmt::Debug;
use expr::Expr;
use typed_arena::Arena;

pub struct AstArena<'ast> {
    arena: Arena<ArenaItem<'ast>>,
}

impl<'ast> AstArena<'ast> {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
        }
    }

    pub fn alloc_expr(&self, expr: Expr<'ast>) -> &mut Expr<'ast> {
        let arena_item = self.arena.alloc(ArenaItem::Expr(expr));
        match arena_item {
            ArenaItem::Expr(expr) => expr,
        }
    }
}

impl<'ast> Debug for AstArena<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<AstArena>")
    }
}

enum ArenaItem<'ast> {
    Expr(Expr<'ast>),
}
