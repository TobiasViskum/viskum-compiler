use std::marker::PhantomData;

use crate::{
    ast_query_system::AstQueryEntry,
    ast_state::{ AstState, AstState0, AstState1, AstState2, AstState3 },
    visitor::{
        walk_binary_expr,
        walk_def_stmt,
        walk_group_expr,
        walk_if_expr,
        walk_stmts,
        Visitor,
    },
    walk_assign_stmt,
    walk_break_expr,
    walk_loop_expr,
    walk_tuple_expr,
    walk_tuple_field_expr,
    AssignStmt,
    Ast,
    BinaryExpr,
    BlockExpr,
    BoolExpr,
    ContinueExpr,
    DefineStmt,
    GroupExpr,
    IdentExpr,
    IdentPat,
    IfExpr,
    IntegerExpr,
    Pat,
    PlaceExpr,
    TupleExpr,
};
use error::{ Error, ErrorKind };
use ir_defs::{ DefId, DefKind, Mutability, NameBinding, NameBindingKind, NodeId };
use op::{ ArithmeticOp, BinaryOp };
use span::Span;
use symbol::Symbol;
use ty::{ PrimTy, Ty, TyCtx, BOOL_TY, INT_TY, VOID_TY };

/// Visits the entire Ast from left to right
///
/// When the visitor is done, the ast will transition into the next state
#[derive(Debug)]
pub struct AstVisitor<'ctx, 'ast, 'b, T, E> where T: AstState, E: AstVisitEmitter<'ctx, 'ast, T> {
    pub ast: Ast<'ast, T>,
    src: &'b str,
    marker: PhantomData<&'ctx ()>,
    /// All loops: `while`, `loop` etc.
    loop_ret_ty_stack: Vec<Option<Ty>>,

    /// Can call functions on the Resolver
    pub ast_visit_emitter: &'b mut E,
}

impl<'ctx, 'ast, 'b, E> AstVisitor<'ctx, 'ast, 'b, AstState0, E>
    where E: AstVisitEmitter<'ctx, 'ast, AstState0>
{
    pub fn visit(mut self) -> Ast<'ast, AstState1> {
        self.visit_stmts(self.ast.main_scope.stmts);
        self.ast.query_system.assert_nodes_amount();
        self.ast.next_state()
    }

    pub fn insert_query_entry(&mut self, node_id: NodeId, ast_query_entry: AstQueryEntry<'ast>) {
        self.ast.query_system.insert_entry(node_id, ast_query_entry)
    }
}

impl<'ctx, 'ast, 'b, E> AstVisitor<'ctx, 'ast, 'b, AstState1, E>
    where E: AstVisitEmitter<'ctx, 'ast, AstState1>
{
    pub fn visit(mut self) -> Ast<'ast, AstState2> {
        self.visit_stmts(self.ast.main_scope.stmts);
        self.ast.next_state()
    }
}

impl<'ctx, 'ast, 'b, E> AstVisitor<'ctx, 'ast, 'b, AstState2, E>
    where 'ctx: 'ast, 'ast: 'b, E: AstVisitEmitter<'ctx, 'ast, AstState2>
{
    pub fn visit(mut self) -> Ast<'ast, AstState3> {
        self.visit_stmts(self.ast.main_scope.stmts);
        self.ast.next_state()
    }
}

impl<'ctx, 'ast, 'b, T, E> AstVisitor<'ctx, 'ast, 'b, T, E>
    where T: AstState, E: AstVisitEmitter<'ctx, 'ast, T>
{
    pub fn new(ast: Ast<'ast, T>, src: &'b str, ast_visit_emitter: &'b mut E) -> Self {
        Self {
            ast,
            loop_ret_ty_stack: Vec::with_capacity(4),
            src,
            ast_visit_emitter,
            marker: PhantomData,
        }
    }
}

/// This can call functions on the Resolver struct in the resolver crate,
/// which also implements this trait
///
/// The reason it's not using the Resolver directly is to avoid cyclic references
pub trait AstVisitEmitter<'ctx, 'ast, T>: Sized where T: AstState {
    /* Methods available during all passes  */
    fn report_error(&mut self, error: Error);

    /* Methods for the first pass (name resolution) */
    fn start_scope(&mut self) where T: AstState<ThisState = AstState1>;
    fn end_scope(&mut self) where T: AstState<ThisState = AstState1>;
    fn define_var(&mut self, ident_pat: &'ast IdentPat) where T: AstState<ThisState = AstState1>;
    fn lookup_var(&mut self, ident_expr: &'ast IdentExpr) where T: AstState<ThisState = AstState1>;

    /* Methods for the second pass (type checking) */
    fn intern_type(&mut self, ty: Ty) -> &'ctx Ty where T: AstState<ThisState = AstState2>;
    fn set_type_to_node_id(&mut self, node_id: NodeId, ty: Ty)
        where T: AstState<ThisState = AstState2>;
    fn get_def_id_from_node_id(&self, node_id: NodeId) -> DefId
        where T: AstState<ThisState = AstState2>;
    fn set_namebinding_and_ty_to_def_id(&mut self, def_id: DefId, name_binding: NameBinding, ty: Ty)
        where T: AstState<ThisState = AstState2>;
    fn get_namebinding_and_ty_from_def_id(&self, def_id: DefId) -> (NameBinding, Ty)
        where T: AstState<ThisState = AstState2>;
}

/// Implements the visitor trait for the pre-first pass (building the Ast query system)
impl<'ctx, 'ast, 'b, E> Visitor<'ast>
    for AstVisitor<'ctx, 'ast, 'b, AstState0, E>
    where E: AstVisitEmitter<'ctx, 'ast, AstState0>
{
    type Result = ();

    fn default_result() -> Self::Result {
        ()
    }

    fn visit_interger_expr(&mut self, interger_expr: &'ast IntegerExpr) -> Self::Result {
        self.insert_query_entry(
            interger_expr.ast_node_id,
            AstQueryEntry::IntegerExpr(interger_expr)
        )
    }

    fn visit_tuple_expr(&mut self, tuple_expr: &'ast crate::TupleExpr<'ast>) -> Self::Result {
        self.insert_query_entry(tuple_expr.ast_node_id, AstQueryEntry::TupleExpr(tuple_expr));
        walk_tuple_expr(self, tuple_expr)
    }

    fn visit_bool_expr(&mut self, bool_expr: &'ast BoolExpr) -> Self::Result {
        self.insert_query_entry(bool_expr.ast_node_id, AstQueryEntry::BoolExpr(bool_expr));
    }

    fn visit_def_stmt(&mut self, def_stmt: &'ast DefineStmt<'ast>) -> Self::Result {
        self.insert_query_entry(def_stmt.ast_node_id, AstQueryEntry::DefineStmt(def_stmt));
        walk_def_stmt(self, def_stmt)
    }

    fn visit_loop_expr(&mut self, loop_expr: &'ast crate::LoopExpr<'ast>) -> Self::Result {
        self.insert_query_entry(loop_expr.ast_node_id, AstQueryEntry::LoopExpr(loop_expr));
        walk_loop_expr(self, loop_expr)
    }

    fn visit_break_expr(&mut self, break_expr: &'ast crate::BreakExpr<'ast>) -> Self::Result {
        self.insert_query_entry(break_expr.ast_node_id, AstQueryEntry::BreakExpr(break_expr));
        walk_break_expr(self, break_expr)
    }

    fn visit_tuple_field_expr(
        &mut self,
        tuple_field_expr: &'ast crate::TupleFieldExpr<'ast>
    ) -> Self::Result {
        self.insert_query_entry(
            tuple_field_expr.ast_node_id,
            AstQueryEntry::TupleFieldExpr(tuple_field_expr)
        );
        walk_tuple_field_expr(self, tuple_field_expr)
    }

    fn visit_assign_stmt(&mut self, assign_stmt: &'ast AssignStmt<'ast>) -> Self::Result {
        self.insert_query_entry(assign_stmt.ast_node_id, AstQueryEntry::AssignStmt(assign_stmt));
        walk_assign_stmt(self, assign_stmt)
    }

    fn visit_block_expr(&mut self, expr: &'ast BlockExpr<'ast>) -> Self::Result {
        self.insert_query_entry(expr.ast_node_id, AstQueryEntry::BlockExpr(expr));
        walk_stmts(self, expr.stmts)
    }

    fn visit_continue_expr(&mut self, continue_expr: &'ast ContinueExpr) -> Self::Result {
        self.insert_query_entry(
            continue_expr.ast_node_id,
            AstQueryEntry::ContinueExpr(continue_expr)
        );
    }

    fn visit_if_expr(&mut self, if_expr: &'ast IfExpr<'ast>) -> Self::Result {
        self.insert_query_entry(if_expr.ast_node_id, AstQueryEntry::IfExpr(if_expr));
        walk_if_expr(self, if_expr)
    }

    fn visit_ident_expr(&mut self, ident_expr: &'ast IdentExpr) -> Self::Result {
        self.insert_query_entry(ident_expr.ast_node_id, AstQueryEntry::IdentExpr(ident_expr))
    }

    fn visit_ident_pat(&mut self, ident_pat: &'ast IdentPat) -> Self::Result {
        self.insert_query_entry(ident_pat.ast_node_id, AstQueryEntry::IdentPat(ident_pat))
    }

    fn visit_binary_expr(&mut self, binary_expr: &'ast BinaryExpr<'ast>) -> Self::Result {
        self.insert_query_entry(binary_expr.ast_node_id, AstQueryEntry::BinaryExpr(binary_expr));
        walk_binary_expr(self, binary_expr)
    }

    fn visit_group_expr(&mut self, group_expr: &'ast GroupExpr<'ast>) -> Self::Result {
        self.insert_query_entry(group_expr.ast_node_id, AstQueryEntry::GroupExpr(group_expr));
        walk_group_expr(self, group_expr)
    }
}

/// Implements the Visitor trait for the first pass (name resolution)
impl<'ctx, 'ast, 'b, E> Visitor<'ast>
    for AstVisitor<'ctx, 'ast, 'b, AstState1, E>
    where E: AstVisitEmitter<'ctx, 'ast, AstState1>
{
    type Result = ();

    fn default_result() -> Self::Result {
        ()
    }

    fn visit_ident_pat(&mut self, ident_pat: &'ast IdentPat) -> Self::Result {
        self.ast_visit_emitter.define_var(ident_pat)
    }

    fn visit_ident_expr(&mut self, ident_expr: &'ast IdentExpr) -> Self::Result {
        self.ast_visit_emitter.lookup_var(ident_expr)
    }

    fn visit_block_expr(&mut self, expr: &'ast BlockExpr<'ast>) -> Self::Result {
        self.ast_visit_emitter.start_scope();
        self.visit_stmts(expr.stmts);
        self.ast_visit_emitter.end_scope();
    }
}

/// Implements the Visitor trait for the second pass (type checking)
impl<'ctx, 'ast, 'b, E> Visitor<'ast>
    for AstVisitor<'ctx, 'ast, 'b, AstState2, E>
    where 'ctx: 'ast, 'ast: 'b, E: AstVisitEmitter<'ctx, 'ast, AstState2>
{
    type Result = Ty;

    fn default_result() -> Self::Result {
        VOID_TY
    }

    fn visit_interger_expr(&mut self, interger_expr: &'ast IntegerExpr) -> Self::Result {
        self.ast_visit_emitter.set_type_to_node_id(interger_expr.ast_node_id, INT_TY);
        INT_TY
    }

    fn visit_bool_expr(&mut self, bool_expr: &'ast BoolExpr) -> Self::Result {
        self.ast_visit_emitter.set_type_to_node_id(bool_expr.ast_node_id, BOOL_TY);
        BOOL_TY
    }

    fn visit_ident_expr(&mut self, ident_expr: &'ast IdentExpr) -> Self::Result {
        let def_id = self.ast_visit_emitter.get_def_id_from_node_id(ident_expr.ast_node_id);
        let (_, def_type) = self.ast_visit_emitter.get_namebinding_and_ty_from_def_id(def_id);
        self.ast_visit_emitter.set_type_to_node_id(ident_expr.ast_node_id, def_type);
        def_type
    }

    fn visit_tuple_field_expr(
        &mut self,
        tuple_field_expr: &'ast crate::TupleFieldExpr<'ast>
    ) -> Self::Result {
        let lhs_ty = self.visit_expr(tuple_field_expr.lhs);
        self.visit_interger_expr(tuple_field_expr.rhs);

        let tuple_ty = match lhs_ty {
            Ty::Tuple(tuple_ty) => tuple_ty,
            ty => {
                self.ast_visit_emitter.report_error(
                    Error::new(ErrorKind::InvalidTuple(ty), Span::dummy())
                );
                self.ast_visit_emitter.set_type_to_node_id(
                    tuple_field_expr.ast_node_id,
                    Ty::Unkown
                );
                return Ty::Unkown;
            }
        };

        if tuple_field_expr.rhs.val > ((tuple_ty.len() - 1) as i64) {
            self.ast_visit_emitter.report_error(
                Error::new(
                    ErrorKind::TupleAccessOutOfBounds(tuple_ty, tuple_field_expr.rhs.val as usize),
                    Span::dummy()
                )
            );
            self.ast_visit_emitter.set_type_to_node_id(tuple_field_expr.ast_node_id, Ty::Unkown);
            return Ty::Unkown;
        } else {
            let access_ty = tuple_ty[tuple_field_expr.rhs.val as usize];
            self.ast_visit_emitter.set_type_to_node_id(tuple_field_expr.ast_node_id, access_ty);

            access_ty
        }
    }

    fn visit_assign_stmt(&mut self, assign_stmt: &'ast AssignStmt<'ast>) -> Self::Result {
        let setter_ty = self.visit_place_expr(assign_stmt.setter_expr);
        let value_ty = self.visit_expr(assign_stmt.value_expr);

        let (name_binding, symbol) = match &assign_stmt.setter_expr {
            PlaceExpr::IdentExpr(ident_expr) => {
                let def_id = self.ast_visit_emitter.get_def_id_from_node_id(ident_expr.ast_node_id);
                let symbol = Symbol::new(&self.src[ident_expr.span.get_byte_range()]);
                let name_binding = self.ast_visit_emitter.get_namebinding_and_ty_from_def_id(
                    def_id
                ).0;
                (name_binding, symbol)
            }
            PlaceExpr::TupleFieldExpr(_) =>
                panic!("Not working yet (tuple_field_expr in assignment)"),
        };

        match &name_binding.kind {
            NameBindingKind::DefKind(def_kind) => {
                match def_kind {
                    DefKind::Variable(mutability) => {
                        if *mutability == Mutability::Immutable {
                            self.ast_visit_emitter.report_error(
                                Error::new(
                                    ErrorKind::AssignmentToImmutable(symbol),
                                    assign_stmt.span
                                )
                            );
                        }
                    }
                }
            }
        }

        if setter_ty == value_ty {
            self.ast_visit_emitter.set_type_to_node_id(assign_stmt.ast_node_id, VOID_TY);
            // Returns void type, because assignments in itself return void
            VOID_TY
        } else {
            panic!("Not same type in assignment: {}, {}", setter_ty, value_ty)
        }
    }

    fn visit_tuple_expr(&mut self, tuple_expr: &'ast TupleExpr<'ast>) -> Self::Result {
        let mut tuple_types = Vec::with_capacity(8);
        for expr in tuple_expr.fields {
            let ty = self.visit_expr(*expr);
            tuple_types.push(ty);
        }

        let tuple_ty = TyCtx::tuple_type(tuple_types);

        self.ast_visit_emitter.set_type_to_node_id(tuple_expr.ast_node_id, tuple_ty);

        tuple_ty
    }

    fn visit_def_stmt(&mut self, def_stmt: &'ast DefineStmt<'ast>) -> Self::Result {
        // When tuples are implemented, compare if tuple on lhs, is same as tuple type on rhs
        let value_type = self.visit_expr(def_stmt.value_expr);

        match &def_stmt.setter_expr {
            Pat::IdentPat(ident_pat) => {
                let mutability = if def_stmt.mut_span.get().is_some() {
                    Mutability::Mutable
                } else {
                    Mutability::Immutable
                };
                let def_id = self.ast_visit_emitter.get_def_id_from_node_id(ident_pat.ast_node_id);
                self.ast_visit_emitter.set_type_to_node_id(ident_pat.ast_node_id, value_type);
                self.ast_visit_emitter.set_namebinding_and_ty_to_def_id(
                    def_id,
                    NameBinding::from(DefKind::Variable(mutability)),
                    value_type
                );
            }
        }

        // Even though def stmts doesn't return a value,
        // it still sets the type of the def stmt for ease of use in the prettifier
        //
        // It will, however, not have any effect on the actual program,
        // since below it returns void
        self.ast_visit_emitter.set_type_to_node_id(def_stmt.ast_node_id, VOID_TY);

        // Returns void, since definitions cannot return a value
        VOID_TY
    }

    fn visit_loop_expr(&mut self, loop_expr: &'ast crate::LoopExpr<'ast>) -> Self::Result {
        self.loop_ret_ty_stack.push(None);
        self.visit_block_expr(loop_expr.body);

        let ty = self.loop_ret_ty_stack
            .last()
            .expect("This is always present pushed above")
            .unwrap_or(Self::default_result());

        self.loop_ret_ty_stack.pop();

        self.ast_visit_emitter.set_type_to_node_id(loop_expr.ast_node_id, ty);
        ty
    }

    fn visit_continue_expr(&mut self, continue_expr: &'ast ContinueExpr) -> Self::Result {
        let void_ty = Ty::PrimTy(PrimTy::Void);
        self.ast_visit_emitter.set_type_to_node_id(continue_expr.ast_node_id, void_ty);
        void_ty
    }

    fn visit_break_expr(&mut self, break_expr: &'ast crate::BreakExpr<'ast>) -> Self::Result {
        let break_ty = break_expr.value
            .map(|expr| self.visit_expr(expr))
            .unwrap_or(Self::default_result());

        if let Some(loop_ret_ty) = self.loop_ret_ty_stack.last_mut() {
            if let Some(expected_ty) = loop_ret_ty {
                if break_ty != *expected_ty {
                    self.ast_visit_emitter.report_error(
                        Error::new(ErrorKind::BreakTypeError(*expected_ty, break_ty), Span::dummy())
                    );
                }
            } else {
                *loop_ret_ty = Some(break_ty);
            }
        } else {
            self.ast_visit_emitter.report_error(
                Error::new(ErrorKind::BreakOutsideLoop, Span::dummy())
            );
        }

        self.ast_visit_emitter.set_type_to_node_id(break_expr.ast_node_id, break_ty);

        Self::default_result()
    }

    fn visit_if_expr(&mut self, if_expr: &'ast IfExpr<'ast>) -> Self::Result {
        self.visit_expr(if_expr.condition);

        let cond_type = self.visit_expr(if_expr.condition);
        match cond_type {
            Ty::PrimTy(PrimTy::Bool) => {}
            _ =>
                self.ast_visit_emitter.report_error(
                    Error::new(ErrorKind::ExpectedBoolExpr(cond_type), if_expr.span)
                ),
        }

        let true_type = self.visit_block_expr(if_expr.true_block);
        let false_type = if_expr.false_block
            .as_ref()
            .map(|expr| self.visit_if_false_branch_expr(*expr));

        let if_expr_type = if let Some(false_type) = false_type {
            if true_type == false_type { true_type } else { Ty::Unkown }
        } else {
            true_type
        };

        self.ast_visit_emitter.set_type_to_node_id(if_expr.ast_node_id, if_expr_type);

        if_expr_type
    }

    fn visit_block_expr(&mut self, expr: &'ast BlockExpr<'ast>) -> Self::Result {
        let block_type = self.visit_stmts(expr.stmts);
        self.ast_visit_emitter.set_type_to_node_id(expr.ast_node_id, block_type);
        block_type
    }

    fn visit_binary_expr(&mut self, binary_expr: &'ast BinaryExpr<'ast>) -> Self::Result {
        let lhs_type = self.visit_expr(binary_expr.lhs);
        let rhs_type = self.visit_expr(binary_expr.rhs);

        let result_ty = match binary_expr.op {
            | BinaryOp::ArithmeticOp(ArithmeticOp::Add)
            | BinaryOp::ArithmeticOp(ArithmeticOp::Div)
            | BinaryOp::ArithmeticOp(ArithmeticOp::Mul)
            | BinaryOp::ArithmeticOp(ArithmeticOp::Sub) => {
                match (lhs_type, rhs_type) {
                    (INT_TY, INT_TY) => { lhs_type }
                    _ => {
                        self.ast_visit_emitter.report_error(
                            Error::new(
                                ErrorKind::BinaryExprTypeError(binary_expr.op, lhs_type, rhs_type),
                                Span::dummy()
                            )
                        );
                        Ty::Unkown
                    }
                }
            }
            BinaryOp::ComparisonOp(_) => {
                match (lhs_type, rhs_type) {
                    (INT_TY, INT_TY) => { BOOL_TY }
                    (BOOL_TY, BOOL_TY) => { BOOL_TY }
                    _ => {
                        self.ast_visit_emitter.report_error(
                            Error::new(
                                ErrorKind::BinaryExprTypeError(binary_expr.op, lhs_type, rhs_type),
                                Span::dummy()
                            )
                        );
                        Ty::Unkown
                    }
                }
            }
        };

        self.ast_visit_emitter.set_type_to_node_id(binary_expr.ast_node_id, result_ty);

        result_ty
    }

    fn visit_group_expr(&mut self, group_expr: &'ast GroupExpr<'ast>) -> Self::Result {
        let expr_type = self.visit_expr(group_expr.expr);

        self.ast_visit_emitter.set_type_to_node_id(group_expr.ast_node_id, expr_type);

        expr_type
    }
}
