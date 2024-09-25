use std::{ marker::PhantomData, process::id };

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
    walk_expr,
    walk_pat,
    walk_place_expr,
    AssignStmt,
    Ast,
    BinaryExpr,
    BlockExpr,
    BoolExpr,
    DefineStmt,
    Expr,
    GroupExpr,
    IdentExpr,
    IdentPat,
    IfExpr,
    IntegerExpr,
    Pat,
    PatKind,
    PlaceKind,
};
use error::{ Error, ErrorKind };
use ir_defs::{ DefId, DefKind, Mutability, NameBinding, NameBindingKind, NodeId, Res };
use op::{ ArithmeticOp, BinaryOp };
use symbol::Symbol;
use ty::{ PrimTy, Ty };

/// Visits the entire Ast from left to right
///
/// When the visitor is done, the ast will transition into the next state
#[derive(Debug)]
pub struct AstVisitor<'ctx, 'ast, 'b, T, E> where T: AstState, E: AstVisitEmitter<'ctx, 'ast, T> {
    pub ast: Ast<'ast, T>,
    src: &'b str,
    marker: PhantomData<&'ctx ()>,

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
    where 'ast: 'b, E: AstVisitEmitter<'ctx, 'ast, AstState2>
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
        Self { ast, src, ast_visit_emitter, marker: PhantomData }
    }
}

/// This can call functions on the Resolver struct in the resolver crate,
/// which also implements this trait
///
/// The reason it's not using the Resolver directly is to avoid cyclic references
pub trait AstVisitEmitter<'ctx, 'ast, T>: Sized where T: AstState {
    /* Methods available during all passes  */
    fn report_error(&mut self, error: Error<'ctx>);

    /* Methods for the first pass (name resolution) */
    fn start_scope(&mut self) where T: AstState<ThisState = AstState1>;
    fn end_scope(&mut self) where T: AstState<ThisState = AstState1>;
    fn define_var(&mut self, ident_pat: &'ast IdentPat) where T: AstState<ThisState = AstState1>;
    fn lookup_var(&mut self, ident_expr: &'ast IdentExpr) where T: AstState<ThisState = AstState1>;

    /* Methods for the second pass (type checking) */
    fn intern_type(&mut self, ty: Ty) -> &'ctx Ty where T: AstState<ThisState = AstState2>;
    fn set_type_to_node_id(&mut self, node_id: NodeId, ty: &'ctx Ty)
        where T: AstState<ThisState = AstState2>;
    fn get_def_id_from_node_id(&self, node_id: NodeId) -> DefId
        where T: AstState<ThisState = AstState2>;
    fn set_namebinding_and_ty_to_def_id(
        &mut self,
        def_id: DefId,
        name_binding: NameBinding,
        ty: &'ctx Ty
    )
        where T: AstState<ThisState = AstState2>;
    fn get_namebinding_and_ty_from_def_id(&self, def_id: DefId) -> (NameBinding, &'ctx Ty)
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

    fn visit_pat(&mut self, pat: &'ast Pat) -> Self::Result {
        self.insert_query_entry(pat.ast_node_id, AstQueryEntry::Pat(pat));
        walk_pat(self, pat)
    }

    fn visit_place_expr(&mut self, place_expr: &'ast crate::PlaceExpr) -> Self::Result {
        self.insert_query_entry(place_expr.ast_node_id, AstQueryEntry::PlaceExpr(place_expr));
        walk_place_expr(self, place_expr)
    }

    fn visit_expr(&mut self, expr: &'ast Expr<'ast>) -> Self::Result {
        self.insert_query_entry(expr.ast_node_id, AstQueryEntry::Expr(expr));
        walk_expr(self, expr)
    }

    fn visit_bool_expr(&mut self, bool_expr: &'ast BoolExpr) -> Self::Result {
        self.insert_query_entry(bool_expr.ast_node_id, AstQueryEntry::BoolExpr(bool_expr));
    }

    fn visit_def_stmt(&mut self, def_stmt: &'ast DefineStmt<'ast>) -> Self::Result {
        self.insert_query_entry(def_stmt.ast_node_id, AstQueryEntry::DefineStmt(def_stmt));
        walk_def_stmt(self, def_stmt)
    }

    fn visit_assign_stmt(&mut self, assign_stmt: &'ast AssignStmt<'ast>) -> Self::Result {
        self.insert_query_entry(assign_stmt.ast_node_id, AstQueryEntry::AssignStmt(assign_stmt));
        walk_assign_stmt(self, assign_stmt)
    }

    fn visit_block_expr(&mut self, expr: &'ast BlockExpr<'ast>) -> Self::Result {
        self.insert_query_entry(expr.ast_node_id, AstQueryEntry::BlockExpr(expr));
        walk_stmts(self, expr.stmts)
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
        self.insert_query_entry(binary_expr.ast_node_id, AstQueryEntry::BinarExpr(binary_expr));
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
    where 'ast: 'b, E: AstVisitEmitter<'ctx, 'ast, AstState2>
{
    type Result = &'ctx Ty;

    fn default_result() -> Self::Result {
        &Ty::PrimTy(PrimTy::Void)
    }

    fn visit_interger_expr(&mut self, interger_expr: &'ast IntegerExpr) -> Self::Result {
        let interned_type = self.ast_visit_emitter.intern_type(Ty::PrimTy(PrimTy::Int));
        self.ast_visit_emitter.set_type_to_node_id(interger_expr.ast_node_id, interned_type);
        interned_type
    }

    fn visit_place_expr(&mut self, place_expr: &'ast crate::PlaceExpr) -> Self::Result {
        let ty = walk_place_expr(self, place_expr);
        self.ast_visit_emitter.set_type_to_node_id(place_expr.ast_node_id, ty);
        ty
    }

    fn visit_bool_expr(&mut self, bool_expr: &'ast BoolExpr) -> Self::Result {
        let interned_type = self.ast_visit_emitter.intern_type(Ty::PrimTy(PrimTy::Bool));
        self.ast_visit_emitter.set_type_to_node_id(bool_expr.ast_node_id, interned_type);
        interned_type
    }

    fn visit_ident_expr(&mut self, ident_expr: &'ast IdentExpr) -> Self::Result {
        let def_id = self.ast_visit_emitter.get_def_id_from_node_id(ident_expr.ast_node_id);
        let (_, def_type) = self.ast_visit_emitter.get_namebinding_and_ty_from_def_id(def_id);
        self.ast_visit_emitter.set_type_to_node_id(ident_expr.ast_node_id, def_type);
        def_type
    }

    fn visit_assign_stmt(&mut self, assign_stmt: &'ast AssignStmt<'ast>) -> Self::Result {
        let setter_ty = self.visit_place_expr(assign_stmt.setter_expr);
        let value_ty = self.visit_expr(assign_stmt.value_expr);

        let (name_binding, symbol) = match &assign_stmt.setter_expr.kind {
            PlaceKind::IdentExpr(ident_expr) => {
                let def_id = self.ast_visit_emitter.get_def_id_from_node_id(ident_expr.ast_node_id);
                let symbol = Symbol::new(&self.src[ident_expr.span.get_byte_range()]);
                let name_binding = self.ast_visit_emitter.get_namebinding_and_ty_from_def_id(
                    def_id
                ).0;
                (name_binding, symbol)
            }
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
            let void_type = self.ast_visit_emitter.intern_type(Ty::PrimTy(PrimTy::Void));
            self.ast_visit_emitter.set_type_to_node_id(assign_stmt.ast_node_id, void_type);
            // Returns void type, because assignments in itself return void
            void_type
        } else {
            panic!("Not same type in assignment")
        }
    }

    fn visit_def_stmt(&mut self, def_stmt: &'ast DefineStmt<'ast>) -> Self::Result {
        // When tuples are implemented, compare if tuple on lhs, is same as tuple type on rhs
        let value_type = self.visit_expr(&def_stmt.value_expr);
        self.ast_visit_emitter.set_type_to_node_id(def_stmt.setter_expr.ast_node_id, value_type);

        match &def_stmt.setter_expr.kind {
            PatKind::IdentPat(ident_pat) => {
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

        let void_type = self.ast_visit_emitter.intern_type(Ty::PrimTy(PrimTy::Void));

        // Even though def stmts doesn't return a value,
        // it still sets the type of the def stmt for ease of use in the prettifier
        //
        // It will, however, not have any effect on the actual program,
        // since below it returns void
        self.ast_visit_emitter.set_type_to_node_id(def_stmt.ast_node_id, void_type);

        // Returns void, since definitions cannot return a value
        void_type
    }

    fn visit_expr(&mut self, expr: &'ast Expr<'ast>) -> Self::Result {
        let ty = walk_expr(self, expr);
        self.ast_visit_emitter.set_type_to_node_id(expr.ast_node_id, ty);
        ty
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
            .map(|expr| self.visit_if_false_branch_expr(expr));

        let if_expr_type = if let Some(false_type) = false_type {
            if true_type == false_type {
                true_type
            } else {
                self.ast_visit_emitter.intern_type(Ty::Unkown)
            }
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
                    (Ty::PrimTy(PrimTy::Int), Ty::PrimTy(PrimTy::Int)) => {
                        self.ast_visit_emitter.intern_type(Ty::PrimTy(PrimTy::Int))
                    }
                    _ => panic!("Only integers are implemented (add, div, mul, sub)"),
                }
            }
            BinaryOp::ComparisonOp(_) => {
                match (lhs_type, rhs_type) {
                    (Ty::PrimTy(PrimTy::Int), Ty::PrimTy(PrimTy::Int)) => {
                        self.ast_visit_emitter.intern_type(Ty::PrimTy(PrimTy::Bool))
                    }
                    (Ty::PrimTy(PrimTy::Bool), Ty::PrimTy(PrimTy::Bool)) => {
                        self.ast_visit_emitter.intern_type(Ty::PrimTy(PrimTy::Bool))
                    }
                    _ => panic!("Only integers are implemented (eq, ne, ge, gt, le, gt)"),
                }
            }
        };
        // let result_type = match (lhs_type, rhs_type) {
        //     (Ty::PrimTy(PrimTy::Int), Ty::PrimTy(PrimTy::Int)) => { &Ty::PrimTy(PrimTy::Int) }
        //     _ => {
        //         println!(
        //             "Report error in binary expr: {} {} {}",
        //             lhs_type,
        //             binary_expr.op,
        //             rhs_type
        //         );
        //         self.ast_visit_emitter.intern_type(Ty::Unkown)
        //     }
        // };

        self.ast_visit_emitter.set_type_to_node_id(binary_expr.ast_node_id, result_ty);

        result_ty
    }

    fn visit_group_expr(&mut self, group_expr: &'ast GroupExpr<'ast>) -> Self::Result {
        let expr_type = self.visit_expr(group_expr.expr);

        self.ast_visit_emitter.set_type_to_node_id(group_expr.ast_node_id, expr_type);

        expr_type
    }
}
