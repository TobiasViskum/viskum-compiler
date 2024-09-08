use ast::Ast;
use ast_state::{ AstUnvalidated, AstValidated };

/// Change name to AstResolver
pub struct AstValidator<'a> {
    ast: Ast<'a, AstUnvalidated>,
}

impl<'a> AstValidator<'a> {
    pub fn new(ast: Ast<'a, AstUnvalidated>) -> Self {
        Self { ast }
    }

    pub fn validate_ast(self) -> Ast<'a, AstValidated> {
        self.ast._next_state()
    }
}
