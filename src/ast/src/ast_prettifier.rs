use fxhash::FxHashMap;
use ir::{ NodeId, Symbol, Ty };

use crate::{
    ast_state::AstState,
    get_node_id_from_expr,
    visitor::{ walk_stmt, Visitor },
    Ast,
    IdentNode,
    IfFalseBranchExpr,
    Stmt,
    Typing,
};
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

fn write_typing<'ast>(buffer: &mut String, src: &str, typing: &Typing<'ast>) {
    match typing {
        Typing::Ident(span) =>
            write!(buffer, "{}", Symbol::new(&src[span.get_byte_range()]).get()).expect(
                "Unexpected write error"
            ),
        // Typing::NamedTuple(typings) => {
        //     write!(buffer, "(").expect("Unexpected write error");
        //     for (i, (span, typing)) in typings.iter().enumerate() {
        //         write!(buffer, "{} ", Symbol::new(&src[span.get_byte_range()]).get()).expect(
        //             "Unexpected write error"
        //         );
        //         write_typing(buffer, src, typing);
        //         if i < typings.len() - 1 {
        //             write!(buffer, ", ").expect("Unexpected write error");
        //         }
        //     }
        //     write!(buffer, ")").expect("Unexpected write error");
        // }
        Typing::Tuple(typings) => {
            write!(buffer, "(").expect("Unexpected write error");
            for (i, typing) in typings.iter().enumerate() {
                write_typing(buffer, src, typing);
                if i < typings.len() - 1 {
                    write!(buffer, ", ").expect("Unexpected write error");
                }
            }
            write!(buffer, ")").expect("Unexpected write error");
        }
        Typing::Fn(args_typing, ret_typing) => {
            write!(buffer, "fn(").expect("Unexpected write error");

            for (i, typing) in args_typing.iter().enumerate() {
                write_typing(buffer, src, typing);
                if i < args_typing.len() - 1 {
                    write!(buffer, ", ").expect("Unexpected write error");
                }
            }

            write!(buffer, ")").expect("Unexpected write error");

            if let Some(ret_typing) = ret_typing {
                write!(buffer, " ").expect("Unexpected write error");
                write_typing(buffer, src, ret_typing);
            }
        }
        Typing::Ptr(typing, mutability) => {
            write!(buffer, "*").expect("Unexpected write error");
            if *mutability == crate::Mutability::Mutable {
                write!(buffer, "mut ").expect("Unexpected write error");
            }
            write_typing(buffer, src, typing)
        }
        Typing::ManyPtr(typing) => {
            write!(buffer, "[*]").expect("Unexpected write error");
            write_typing(buffer, src, typing)
        }
    }
}

impl<'ast, T> Visitor<'ast> for AstPrettifier<'ast, T> where T: AstState {
    type Result = Result<(), std::fmt::Error>;

    fn default_result() -> Self::Result {
        Ok(())
    }

    fn visit_struct_expr(&mut self, struct_expr: &'ast crate::StructExpr<'ast>) -> Self::Result {
        write!(
            self.buffer,
            "{} {{ ",
            Symbol::new(&self.src[struct_expr.ident_node.span.get_byte_range()]).get()
        )?;

        for (i, field) in struct_expr.field_initializations.iter().enumerate() {
            write!(
                self.buffer,
                "{}: ",
                Symbol::new(&self.src[field.ident.span.get_byte_range()]).get()
            )?;
            self.visit_expr(field.value)?;

            if i < struct_expr.field_initializations.len() - 1 {
                write!(self.buffer, ", ")?;
            }
        }

        write!(self.buffer, " }}")?;

        Self::default_result()
    }

    fn visit_tuple_expr(&mut self, tuple_expr: &'ast crate::TupleExpr<'ast>) -> Self::Result {
        write!(self.buffer, "(")?;

        for (i, expr) in tuple_expr.fields.iter().enumerate() {
            self.visit_expr(*expr)?;

            if i < tuple_expr.fields.len() - 1 {
                write!(self.buffer, ", ")?;
            }
        }

        write!(self.buffer, ")")?;

        Self::default_result()
    }

    fn visit_typedef_item(&mut self, typedef_item: &'ast crate::TypedefItem<'ast>) -> Self::Result {
        write!(
            self.buffer,
            "{}typedef {} ",
            self.get_indentation(),
            Symbol::new(&self.src[typedef_item.ident_node.span.get_byte_range()]).get()
        )?;

        write_typing(&mut self.buffer, self.src, &typedef_item.type_expr);

        write!(self.buffer, "\n")?;

        Self::default_result()
    }

    fn visit_struct_item(&mut self, struct_item: &'ast crate::StructItem<'ast>) -> Self::Result {
        write!(
            self.buffer,
            "{}struct {} {{\n",
            self.get_indentation(),
            Symbol::new(&self.src[struct_item.ident_node.span.get_byte_range()]).get()
        )?;

        self.increment_scope_depth();

        for field in struct_item.field_declarations.iter() {
            write!(self.buffer, "{}", self.get_indentation())?;

            write!(
                self.buffer,
                "{} ",
                Symbol::new(&self.src[field.ident.span.get_byte_range()]).get()
            )?;

            write_typing(&mut self.buffer, self.src, &field.type_expr);

            write!(self.buffer, ",\n")?;
        }

        self.decrement_scope_depth();

        write!(self.buffer, "{}}}\n", self.get_indentation())?;

        Self::default_result()
    }

    fn visit_fn_item(&mut self, fn_item: &'ast crate::FnItem<'ast>) -> Self::Result {
        write!(
            self.buffer,
            "{}fn {}(",
            self.get_indentation(),
            Symbol::new(&self.src[fn_item.ident_node.span.get_byte_range()]).get()
        )?;

        for (i, arg) in fn_item.args.iter().enumerate() {
            write!(
                self.buffer,
                "{} ",
                Symbol::new(&self.src[arg.ident.span.get_byte_range()]).get()
            )?;

            write_typing(&mut self.buffer, self.src, &arg.type_expr);

            if i < fn_item.args.len() - 1 {
                write!(self.buffer, ", ")?;
            }
        }

        write!(self.buffer, ") {{\n")?;

        self.increment_scope_depth();
        self.visit_stmts(fn_item.body)?;
        self.decrement_scope_depth();

        write!(self.buffer, "{}}}\n", self.get_indentation())?;

        Self::default_result()
    }

    fn visit_return_expr(&mut self, return_expr: &'ast crate::ReturnExpr) -> Self::Result {
        write!(self.buffer, "ret")?;
        if let Some(expr) = return_expr.value {
            write!(self.buffer, " ")?;
            self.visit_expr(expr)?;
        }

        write!(self.buffer, "\n")?;

        Self::default_result()
    }

    fn visit_def_stmt(&mut self, def_stmt: &'ast crate::DefineStmt<'ast>) -> Self::Result {
        write!(self.buffer, "{}", self.get_indentation())?;
        if def_stmt.mut_span.get().is_some() {
            write!(self.buffer, "mut ")?;
        }
        self.visit_pat(def_stmt.setter_expr)?;

        if let Some(node_id_to_ty) = self.node_id_to_ty {
            let node_id = get_node_id_from_expr(def_stmt.value_expr);
            let ty = node_id_to_ty.get(&node_id).expect("Expected type");
            write!(self.buffer, ": {}", ty)?;
        }

        write!(self.buffer, " := ")?;
        self.visit_expr(def_stmt.value_expr)?;

        write!(self.buffer, "\n")?;

        Self::default_result()
    }

    fn visit_stmt(&mut self, stmt: Stmt<'ast>) -> Self::Result {
        match stmt {
            Stmt::ExprStmt(expr) => {
                write!(self.buffer, "{}", self.get_indentation())?;
                self.visit_expr(expr)
            }
            _ => walk_stmt(self, stmt),
        }
    }

    fn visit_tuple_field_expr(
        &mut self,
        tuple_field_expr: &'ast crate::TupleFieldExpr<'ast>
    ) -> Self::Result {
        self.visit_expr(tuple_field_expr.lhs)?;
        write!(self.buffer, ".")?;
        self.visit_interger_expr(tuple_field_expr.rhs)
    }

    fn visit_field_expr(&mut self, field_expr: &'ast crate::FieldExpr<'ast>) -> Self::Result {
        self.visit_expr(field_expr.lhs)?;
        write!(self.buffer, ".")?;
        self.visit_ident_expr(field_expr.rhs)
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
        self.visit_place_expr(assign_stmt.setter_expr)?;
        write!(self.buffer, " = ")?;
        self.visit_expr(assign_stmt.value_expr)?;

        write!(self.buffer, "\n")
    }

    fn visit_if_expr(&mut self, if_expr: &'ast crate::IfExpr<'ast>) -> Self::Result {
        write!(self.buffer, "if ")?;
        self.visit_cond_kind(if_expr.cond_kind)?;
        write!(self.buffer, " then\n")?;
        self.increment_scope_depth();
        self.visit_stmts(if_expr.true_block)?;
        self.decrement_scope_depth();

        match &if_expr.false_block {
            Some(expr) => {
                match expr {
                    IfFalseBranchExpr::ElseExpr(expr) => {
                        write!(self.buffer, "\n{}else\n", self.get_indentation())?;
                        self.increment_scope_depth();
                        self.visit_stmts(expr.stmts)?;
                        self.decrement_scope_depth();
                        write!(self.buffer, "\n{}end", self.get_indentation())?;
                    }
                    IfFalseBranchExpr::ElifExpr(if_expr) => {
                        write!(self.buffer, "\n{}el", self.get_indentation())?;
                        self.visit_if_expr(if_expr)?;
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

    fn visit_ident_pat(&mut self, ident_node: &'ast IdentNode) -> Self::Result {
        write!(self.buffer, "{}", &self.src[ident_node.span.get_byte_range()])?;

        Self::default_result()
    }

    fn visit_ident_expr(&mut self, ident_node: &'ast IdentNode) -> Self::Result {
        write!(self.buffer, "{}", &self.src[ident_node.span.get_byte_range()])?;

        Self::default_result()
    }
}
