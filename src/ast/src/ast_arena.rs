use std::{ collections::VecDeque, fmt::Debug, u16 };
use bumpalo::Bump;
use typed_arena::Arena;

use crate::{ Expr, FieldDeclaration, FieldInitialization, Stmt };

pub struct AstArena<'ast> {
    ast_arena: Bump,
    typed_ast_arenas: TypedAstArenas<'ast>,
}

impl<'ast> Default for AstArena<'ast> {
    fn default() -> Self {
        Self {
            ast_arena: Default::default(),
            typed_ast_arenas: Default::default(),
        }
    }
}

impl<'ast> AstArena<'ast> {
    pub fn new() -> Self {
        Default::default()
    }

    /// Only allocate objects that doesn't require its Drop implementation to be run
    pub fn alloc_expr_or_stmt<T>(&self, expr: T) -> &T {
        self.ast_arena.alloc(expr)
    }

    pub fn alloc_expr_vec(&self, expr_vec: Vec<Expr<'ast>>) -> &[Expr<'ast>] {
        self.typed_ast_arenas.expr_vec_arena.alloc(expr_vec)
    }

    pub fn alloc_stmt_vec(&self, stmt_vec: VecDeque<Stmt<'ast>>) -> &[Stmt<'ast>] {
        self.typed_ast_arenas.stmt_vec_arena.alloc(stmt_vec.into())
    }

    pub fn alloc_field_declaration_vec(
        &self,
        field_vec: Vec<&'ast FieldDeclaration<'ast>>
    ) -> &[&'ast FieldDeclaration<'ast>] {
        self.typed_ast_arenas.field_declaration_vec_arena.alloc(field_vec)
    }

    pub fn alloc_field_initialization_vec(
        &self,
        field_vec: Vec<&'ast FieldInitialization<'ast>>
    ) -> &[&'ast FieldInitialization<'ast>] {
        self.typed_ast_arenas.field_initialization_vec_arena.alloc(field_vec)
    }
}

impl<'ast> Debug for AstArena<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<AstArena>")
    }
}

struct TypedAstArenas<'ast> {
    pub stmt_vec_arena: Arena<Vec<Stmt<'ast>>>,
    pub expr_vec_arena: Arena<Vec<Expr<'ast>>>,
    pub field_declaration_vec_arena: Arena<Vec<&'ast FieldDeclaration<'ast>>>,
    pub field_initialization_vec_arena: Arena<Vec<&'ast FieldInitialization<'ast>>>,
}

impl<'ast> Default for TypedAstArenas<'ast> {
    fn default() -> Self {
        Self {
            expr_vec_arena: Default::default(),
            stmt_vec_arena: Default::default(),
            field_declaration_vec_arena: Default::default(),
            field_initialization_vec_arena: Default::default(),
        }
    }
}
