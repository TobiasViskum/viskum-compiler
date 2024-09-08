use std::fmt::Debug;
use ast_state::{ AstArenaState, AstState };
use typed_arena::Arena;

use crate::{ Expr, IfExpr };

pub struct AstArena<'ast> {
    /// The generic `AstArenaState` is private, and used only as a placeholder
    arena: Arena<ArenaItem<'ast, AstArenaState>>,
}

impl<'ast> AstArena<'ast> {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
        }
    }

    pub fn alloc_expr<T>(&self, expr: Expr<'ast, T>) -> &mut Expr<'ast, T> where T: AstState {
        let arena_item = self.arena.alloc(ArenaItem::Expr(unsafe { std::mem::transmute(expr) }));

        match arena_item {
            ArenaItem::Expr(expr) => unsafe { std::mem::transmute(expr) }
            _ => panic!("Expected arena item: expr"),
        }
    }

    pub fn alloc_if_expr<T>(&self, if_expr: IfExpr<'ast, T>) -> &mut IfExpr<'ast, T>
        where T: AstState
    {
        let arena_item = self.arena.alloc(
            ArenaItem::IfExpr(unsafe { std::mem::transmute(if_expr) })
        );

        match arena_item {
            ArenaItem::IfExpr(if_expr) => unsafe { std::mem::transmute(if_expr) }
            _ => panic!("Expected arena item: if_expr"),
        }
    }
}

enum ArenaItem<'ast, T> where T: AstState {
    Expr(Expr<'ast, T>),
    IfExpr(IfExpr<'ast, T>),
}

impl<'ast> Debug for AstArena<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<AstArena>")
    }
}
