use fxhash::FxHashMap;
use ir_defs::NodeId;
use ty::Ty;

use crate::{ ast_state::AstState, visitor::{ walk_stmt, Visitor }, Ast, IfFalseBranchExpr, Stmt };
use std::fmt::Write;

// The correct lifetimes here would be <'ast, 'b> since the ast is not borrowed for the rest of its lifetime
// However because the Ast is never borrowed mutably, it doesn't affect the compiler
//
// If at any point something requires a mutable borrow to the Ast, this should also change (or compiler go mad)
pub struct AstPrettifier<'ast, T> where T: AstState {
    ast: &'ast Ast<'ast, T>,
    src: &'ast str,
    scope_depth: usize,
    node_id_to_ty: Option<&'ast FxHashMap<NodeId, &'ast Ty>>,
    buffer: String,
}

const INDENTATION: usize = 4;

impl<'ast, T> AstPrettifier<'ast, T> where T: AstState {
    pub fn new(
        ast: &'ast Ast<'ast, T>,
        src: &'ast str,
        node_id_to_ty: Option<&'ast FxHashMap<NodeId, &'ast Ty>>
    ) -> Self {
        Self { ast, src, node_id_to_ty, scope_depth: 0, buffer: String::with_capacity(2024) }
    }

    pub fn print_ast(&mut self) {
        self.visit_stmts(self.ast.main_scope.stmts).expect("Unexpected write error");

        println!("{}", self.buffer);
    }

    fn increment_scope_depth(&mut self) {
        self.scope_depth += 1;
    }

    fn decrement_scope_depth(&mut self) {
        self.scope_depth -= 1;
    }

    fn get_indentation(&self) -> String {
        " ".repeat(self.scope_depth * INDENTATION)
    }
}

impl<'ast, T> Visitor<'ast> for AstPrettifier<'ast, T> where T: AstState {
    type Result = Result<(), std::fmt::Error>;

    fn default_result() -> Self::Result {
        Ok(())
    }

    fn visit_def_stmt(&mut self, def_stmt: &'ast crate::DefineStmt<'ast>) -> Self::Result {
        write!(self.buffer, "{}", self.get_indentation())?;
        if def_stmt.mut_span.get().is_some() {
            write!(self.buffer, "mut ")?;
        }
        self.visit_pat(&def_stmt.setter_expr)?;

        if let Some(node_id_to_ty) = self.node_id_to_ty {
            let ty = node_id_to_ty.get(&def_stmt.value_expr.ast_node_id).expect("Expected type");
            write!(self.buffer, ": {}", ty)?;
        }

        write!(self.buffer, " := ")?;
        self.visit_expr(&def_stmt.value_expr)?;

        write!(self.buffer, "\n")?;

        Self::default_result()
    }

    fn visit_stmt(&mut self, stmt: &'ast Stmt<'ast>) -> Self::Result {
        match stmt {
            Stmt::ExprStmt(expr) => {
                write!(self.buffer, "{}", self.get_indentation())?;
                self.visit_expr(expr)
            }
            _ => walk_stmt(self, stmt),
        }
    }

    fn visit_group_expr(&mut self, group_expr: &'ast crate::GroupExpr<'ast>) -> Self::Result {
        write!(self.buffer, "(")?;
        self.visit_expr(group_expr.expr)?;
        write!(self.buffer, ")")?;

        Self::default_result()
    }

    fn visit_binary_expr(&mut self, binary_expr: &'ast crate::BinaryExpr<'ast>) -> Self::Result {
        write!(self.buffer, "(")?;
        self.visit_expr(binary_expr.lhs)?;
        write!(self.buffer, " {} ", binary_expr.op)?;
        self.visit_expr(binary_expr.rhs)?;
        write!(self.buffer, ")")?;
        Self::default_result()
    }

    fn visit_interger_expr(&mut self, interger_expr: &'ast crate::IntegerExpr) -> Self::Result {
        write!(self.buffer, "{}", interger_expr.val)
    }

    fn visit_bool_expr(&mut self, bool_expr: &'ast crate::BoolExpr) -> Self::Result {
        write!(self.buffer, "{}", bool_expr.val)
    }

    fn visit_loop_expr(&mut self, loop_expr: &'ast crate::LoopExpr<'ast>) -> Self::Result {
        write!(self.buffer, "loop\n")?;
        self.increment_scope_depth();
        self.visit_stmts(loop_expr.body.stmts)?;
        self.decrement_scope_depth();
        write!(self.buffer, "\n{}end\n", self.get_indentation())
    }

    fn visit_break_expr(&mut self, break_expr: &'ast crate::BreakExpr<'ast>) -> Self::Result {
        write!(self.buffer, "break ")?;
        break_expr.value.map(|expr| self.visit_expr(expr));
        Self::default_result()
    }

    fn visit_assign_stmt(&mut self, assign_stmt: &'ast crate::AssignStmt<'ast>) -> Self::Result {
        write!(self.buffer, "{}", self.get_indentation())?;
        self.visit_place_expr(&assign_stmt.setter_expr)?;
        write!(self.buffer, " = ")?;
        self.visit_expr(&assign_stmt.value_expr)?;

        write!(self.buffer, "\n")
    }

    fn visit_if_expr(&mut self, if_expr: &'ast crate::IfExpr<'ast>) -> Self::Result {
        write!(self.buffer, "if ")?;
        self.visit_expr(if_expr.condition)?;
        write!(self.buffer, " then\n")?;
        self.increment_scope_depth();
        self.visit_stmts(if_expr.true_block.stmts)?;
        self.decrement_scope_depth();

        match &if_expr.false_block {
            Some(expr) => {
                match expr {
                    IfFalseBranchExpr::ElifExpr(elif_expr) => {
                        write!(self.buffer, "\n{}el", self.get_indentation())?;
                        self.visit_if_expr(elif_expr)?;
                    }
                    IfFalseBranchExpr::ElseExpr(expr) => {
                        write!(self.buffer, "\n{}else\n", self.get_indentation())?;
                        self.increment_scope_depth();
                        self.visit_stmts(expr.stmts)?;
                        self.decrement_scope_depth();
                        write!(self.buffer, "\n{}end", self.get_indentation())?;
                    }
                }
            }
            None => {
                write!(self.buffer, "\n{}end", self.get_indentation())?;
            }
        }

        Self::default_result()
    }

    fn visit_block_expr(&mut self, expr: &'ast crate::BlockExpr<'ast>) -> Self::Result {
        write!(self.buffer, "do\n")?;

        self.increment_scope_depth();
        self.visit_stmts(expr.stmts)?;
        self.decrement_scope_depth();

        write!(self.buffer, "\n{}end", self.get_indentation())?;

        Self::default_result()
    }

    fn visit_ident_pat(&mut self, ident_pat: &'ast crate::IdentPat) -> Self::Result {
        write!(self.buffer, "{}", &self.src[ident_pat.span.get_byte_range()])?;

        Self::default_result()
    }

    fn visit_ident_expr(&mut self, ident_expr: &'ast crate::IdentExpr) -> Self::Result {
        write!(self.buffer, "{}", &self.src[ident_expr.span.get_byte_range()])?;

        Self::default_result()
    }
}
