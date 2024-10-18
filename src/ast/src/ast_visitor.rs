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
    walk_field_expr,
    walk_loop_expr,
    walk_struct_expr,
    walk_tuple_expr,
    walk_tuple_field_expr,
    AssignStmt,
    Ast,
    BinaryExpr,
    BlockExpr,
    BoolExpr,
    BreakExpr,
    ContinueExpr,
    DefineStmt,
    FieldExpr,
    FnItem,
    GroupExpr,
    IdentNode,
    IfExpr,
    IntegerExpr,
    ItemStmt,
    LoopExpr,
    Pat,
    PlaceExpr,
    Stmt,
    StructExpr,
    StructItem,
    TupleExpr,
    TypedefItem,
    Typing,
};
use error::{ Error, ErrorKind };
use ir::{
    Adt,
    DefId,
    DefIdToNameBinding,
    FnSig,
    Mutability,
    NameBinding,
    NameBindingKind,
    NodeId,
    ResKind,
};

use span::Span;
use ir::{ Symbol, PrimTy, Ty, TyCtx, BOOL_TY, INT_TY, VOID_TY };

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
    where 'ctx: 'ast, E: AstVisitEmitter<'ctx, 'ast, AstState1>
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
    fn start_scope(&mut self);
    fn end_scope(&mut self);
    fn start_context(&mut self);
    fn end_context(&mut self);
    fn report_error(&mut self, error: Error);
    fn alloc_vec<K>(&self, vec: Vec<K>) -> &'ctx [K];
    fn borrow_def_id_to_name_binding(&self) -> &DefIdToNameBinding<'ctx>;
    fn make_def_id(&mut self, node_id: NodeId, symbol: Symbol) -> DefId;
    fn set_namebinding_to_def_id(&mut self, def_id: DefId, name_binding: NameBinding<'ctx>);
    fn set_def_id_to_node_id(&mut self, node_id: NodeId, def_id: DefId);
    fn lookup_ident_declaration(&mut self, span: Span, res_kind: ResKind) -> Option<DefId>;
    fn lookup_ident_definition(
        &mut self,
        span: Span,
        res_kind: ResKind
    ) -> Option<(DefId, NameBinding<'ctx>)>;
    fn bind_def_id_to_lexical_binding(&mut self, def_id: DefId, res_kind: ResKind);
    fn set_main_fn(&mut self, main_fn: &'ast FnItem<'ast>) -> bool;
    fn is_main_scope(&mut self) -> bool;
    fn append_fn(&mut self, fn_item: &'ast FnItem<'ast>);

    // fn define(&mut self, node_id: NodeId, symbol: Symbol, name_binding: NameBinding<'ctx>) -> DefId;
    // fn lookup_ident(
    //     &mut self,
    //     ident_node: &'ast IdentNode,
    //     kind: ResKind
    // ) -> Option<NameBinding<'ctx>>;
    // fn new_lookup_ident(&mut self, span: Span, kind: ResKind) -> Option<(DefId, NameBinding<'ctx>)>;

    /* Methods for the second pass (type checking) */
    fn intern_type(&mut self, ty: Ty) -> &'ctx Ty;
    fn set_type_to_node_id(&mut self, node_id: NodeId, ty: Ty);

    fn get_def_id_from_node_id(&self, node_id: NodeId) -> DefId;

    // fn set_namebinding_and_ty_to_def_id(&mut self, def_id: DefId, name_binding: NameBinding, ty: Ty);
    fn get_namebinding_from_def_id(&self, def_id: DefId) -> NameBinding<'ctx>;

    fn get_ty_from_def_id(&self, def_id: DefId) -> Ty;
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

    fn visit_field_expr(&mut self, field_expr: &'ast crate::FieldExpr<'ast>) -> Self::Result {
        self.insert_query_entry(field_expr.ast_node_id, AstQueryEntry::FieldExpr(field_expr));
        walk_field_expr(self, field_expr)
    }

    fn visit_struct_expr(&mut self, struct_expr: &'ast StructExpr<'ast>) -> Self::Result {
        self.insert_query_entry(struct_expr.ast_node_id, AstQueryEntry::StructExpr(struct_expr));
        walk_struct_expr(self, struct_expr)
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

    fn visit_ident_expr(&mut self, ident_node: &'ast IdentNode) -> Self::Result {
        self.insert_query_entry(ident_node.ast_node_id, AstQueryEntry::IdentNode(ident_node))
    }

    fn visit_ident_pat(&mut self, ident_node: &'ast IdentNode) -> Self::Result {
        self.insert_query_entry(ident_node.ast_node_id, AstQueryEntry::IdentNode(ident_node))
    }

    fn visit_binary_expr(&mut self, binary_expr: &'ast BinaryExpr<'ast>) -> Self::Result {
        self.insert_query_entry(binary_expr.ast_node_id, AstQueryEntry::BinaryExpr(binary_expr));
        walk_binary_expr(self, binary_expr)
    }

    fn visit_group_expr(&mut self, group_expr: &'ast GroupExpr<'ast>) -> Self::Result {
        self.insert_query_entry(group_expr.ast_node_id, AstQueryEntry::GroupExpr(group_expr));
        walk_group_expr(self, group_expr)
    }

    fn visit_struct_item(&mut self, struct_item: &'ast StructItem<'ast>) -> Self::Result {
        self.insert_query_entry(struct_item.ast_node_id, AstQueryEntry::StructItem(struct_item));
        self.visit_ident_expr(struct_item.ident_node);
        for field_decl in struct_item.field_declarations {
            self.visit_ident_expr(field_decl.ident);
        }
    }

    fn visit_typedef_item(&mut self, typedef_item: &'ast crate::TypedefItem<'ast>) -> Self::Result {
        self.insert_query_entry(typedef_item.ast_node_id, AstQueryEntry::TypedefItem(typedef_item));
        self.visit_ident_expr(typedef_item.ident_node);
    }

    fn visit_fn_item(&mut self, fn_item: &'ast crate::FnItem<'ast>) -> Self::Result {
        self.insert_query_entry(fn_item.ast_node_id, AstQueryEntry::FnItem(fn_item));
        self.visit_ident_expr(fn_item.ident_node);
        for arg in fn_item.args {
            self.visit_ident_expr(arg.ident);
        }
        self.visit_stmts(fn_item.body);
    }

    fn visit_return_expr(&mut self, return_expr: &'ast crate::ReturnExpr) -> Self::Result {
        self.insert_query_entry(return_expr.ast_node_id, AstQueryEntry::ReturnExpr(return_expr));
        if let Some(expr) = return_expr.value {
            self.visit_expr(expr);
        }
    }
}

fn type_from_typing<'ctx, 'ast, 'b, E, T: AstState>(
    src: &str,
    typing: &Typing<'ast>,
    e: &'b mut E
) -> Ty
    where E: AstVisitEmitter<'ctx, 'ast, T>
{
    match typing {
        Typing::Ident(span) => {
            let lexeme = &src[span.get_byte_range()];
            match lexeme {
                "Int" => Ty::PrimTy(PrimTy::Int),
                "Bool" => Ty::PrimTy(PrimTy::Bool),
                _ => {
                    if let Some(def_id) = e.lookup_ident_declaration(*span, ResKind::Adt) {
                        Ty::Adt(def_id)
                    } else {
                        panic!("Expected Algebric Data Type");
                    }
                }
            }
        }
        Typing::Tuple(tuple) => {
            let mut tuple_ty = Vec::with_capacity(tuple.len());
            for typing in tuple.iter() {
                tuple_ty.push(type_from_typing(src, typing, e));
            }
            Ty::Tuple(TyCtx::intern_many_types(tuple_ty))
        }
    }
}

/// Implements the Visitor trait for the first pass (name resolution)
impl<'ctx, 'ast, 'b, E> Visitor<'ast>
    for AstVisitor<'ctx, 'ast, 'b, AstState1, E>
    where 'ctx: 'ast, E: AstVisitEmitter<'ctx, 'ast, AstState1>
{
    type Result = ();

    fn default_result() -> Self::Result {
        println!("This shouldn't run");
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

    fn visit_block_expr(&mut self, expr: &'ast BlockExpr<'ast>) -> Self::Result {
        self.ast_visit_emitter.start_scope();
        let block_type = self.visit_stmts(expr.stmts);
        self.ast_visit_emitter.end_scope();
        self.ast_visit_emitter.set_type_to_node_id(expr.ast_node_id, block_type);
        block_type
    }

    fn visit_stmts(&mut self, stmts: &'ast [Stmt<'ast>]) -> Self::Result {
        let iter = stmts.iter().map_while(|x| {
            match x {
                Stmt::ItemStmt(item_stmt) => Some(item_stmt),
                _ => None,
            }
        });

        for item_stmt in iter {
            let (def_id, res_kind) = match item_stmt {
                ItemStmt::FnItem(fn_item) => {
                    let def_id = self.ast_visit_emitter.make_def_id(
                        fn_item.ident_node.ast_node_id,
                        Symbol::new(&self.src[fn_item.ident_node.span.get_byte_range()])
                    );
                    (def_id, ResKind::Fn)
                }
                ItemStmt::StructItem(struct_item) => {
                    let def_id = self.ast_visit_emitter.make_def_id(
                        struct_item.ident_node.ast_node_id,
                        Symbol::new(&self.src[struct_item.ident_node.span.get_byte_range()])
                    );
                    (def_id, ResKind::Adt)
                }
                ItemStmt::TypedefItem(typedef_item) => {
                    let def_id = self.ast_visit_emitter.make_def_id(
                        typedef_item.ident_node.ast_node_id,
                        Symbol::new(&self.src[typedef_item.ident_node.span.get_byte_range()])
                    );
                    (def_id, ResKind::Adt)
                }
            };
            self.ast_visit_emitter.bind_def_id_to_lexical_binding(def_id, res_kind);
        }

        walk_stmts(self, stmts)
    }

    fn visit_ident_expr(&mut self, ident_node: &'ast IdentNode) -> Self::Result {
        if
            let Some(def_id) = self.ast_visit_emitter.lookup_ident_declaration(
                ident_node.span,
                ResKind::Variable
            )
        {
            self.ast_visit_emitter.set_def_id_to_node_id(ident_node.ast_node_id, def_id);
            let ident_ty = Ty::Ptr(
                TyCtx::intern_type(self.ast_visit_emitter.get_ty_from_def_id(def_id))
            );
            self.ast_visit_emitter.set_type_to_node_id(ident_node.ast_node_id, ident_ty);
            ident_ty
        } else {
            let symbol = Symbol::new(&self.src[ident_node.span.get_byte_range()]);
            self.ast_visit_emitter.report_error(
                Error::new(ErrorKind::UndefinedLookup(symbol, ResKind::Variable), ident_node.span)
            );
            Ty::Unkown
        }
    }

    fn visit_typedef_item(&mut self, typedef_item: &'ast TypedefItem<'ast>) -> Self::Result {
        let name_binding = {
            let symbol = Symbol::new(&self.src[typedef_item.ident_node.span.get_byte_range()]);

            match symbol.get() {
                ty_str @ ("Int" | "Bool") => panic!("Cannot overwrite type `{}`", ty_str),
                _ => {}
            }

            let ty = type_from_typing(self.src, &typedef_item.type_expr, self.ast_visit_emitter);

            let def_id = self.ast_visit_emitter.make_def_id(
                typedef_item.ident_node.ast_node_id,
                symbol
            );
            let name_binding = NameBinding::new(NameBindingKind::Adt(Adt::Typedef(ty)));
            self.ast_visit_emitter.set_namebinding_to_def_id(def_id, name_binding);

            name_binding
        };

        if let NameBindingKind::Adt(Adt::Typedef(ty)) = name_binding.kind {
            self.ast_visit_emitter.set_type_to_node_id(typedef_item.ident_node.ast_node_id, ty);
        } else {
            panic!("Expected typedef");
        }

        self.ast_visit_emitter.set_type_to_node_id(typedef_item.ast_node_id, VOID_TY);
        VOID_TY
    }

    fn visit_fn_item(&mut self, fn_item: &'ast FnItem<'ast>) -> Self::Result {
        let def_id = {
            let def_id = self.ast_visit_emitter.get_def_id_from_node_id(
                fn_item.ident_node.ast_node_id
            );

            let args_tys = {
                self.ast_visit_emitter.start_context();
                let mut args_tys = Vec::with_capacity(fn_item.args.len());
                for arg in fn_item.args {
                    let def_id = self.ast_visit_emitter.make_def_id(
                        arg.ident.ast_node_id,
                        Symbol::new(&self.src[arg.ident.span.get_byte_range()])
                    );
                    let res_ty = type_from_typing(self.src, &arg.type_expr, self.ast_visit_emitter);
                    self.ast_visit_emitter.set_type_to_node_id(arg.ident.ast_node_id, res_ty);

                    args_tys.push((def_id, res_ty));

                    /* Defines arg as a variable */
                    self.ast_visit_emitter.set_namebinding_to_def_id(
                        def_id,
                        NameBinding::new(NameBindingKind::Variable(Mutability::Immutable))
                    );
                    self.ast_visit_emitter.bind_def_id_to_lexical_binding(
                        def_id,
                        ResKind::Variable
                    );
                }

                self.visit_stmts(fn_item.body);
                self.ast_visit_emitter.end_context();
                args_tys
            };

            let name_binding = NameBinding::new(
                NameBindingKind::Fn(
                    FnSig::new(TyCtx::intern_many_types(args_tys), TyCtx::intern_type(VOID_TY))
                )
            );
            self.ast_visit_emitter.set_namebinding_to_def_id(def_id, name_binding);

            let symbol = Symbol::new(&self.src[fn_item.ident_node.span.get_byte_range()]);
            if symbol.get() == "main" && self.ast_visit_emitter.is_main_scope() {
                if !self.ast_visit_emitter.set_main_fn(fn_item) {
                    panic!(
                        "Duplicate definitions of entry point `main` in global scope (report error)"
                    );
                }
            } else {
                self.ast_visit_emitter.append_fn(fn_item);
            }

            def_id
        };

        self.ast_visit_emitter.set_type_to_node_id(
            fn_item.ident_node.ast_node_id,
            Ty::FnDef(def_id)
        );

        self.ast_visit_emitter.set_type_to_node_id(fn_item.ast_node_id, VOID_TY);

        VOID_TY
    }

    fn visit_struct_item(&mut self, struct_item: &'ast StructItem<'ast>) -> Self::Result {
        let def_id = {
            let mut struct_binding_fields = Vec::with_capacity(
                struct_item.field_declarations.len()
            );
            self.ast_visit_emitter.start_scope();
            for field in struct_item.field_declarations {
                let def_id = self.ast_visit_emitter.make_def_id(
                    field.ident.ast_node_id,
                    Symbol::new(&self.src[field.ident.span.get_byte_range()])
                );
                let res_ty = type_from_typing(self.src, &field.type_expr, self.ast_visit_emitter);

                self.ast_visit_emitter.set_type_to_node_id(field.ident.ast_node_id, res_ty);
                struct_binding_fields.push((def_id, res_ty));
            }
            self.ast_visit_emitter.end_scope();

            let def_id = self.ast_visit_emitter.get_def_id_from_node_id(
                struct_item.ident_node.ast_node_id
            );

            let name_binding = NameBinding::new(
                NameBindingKind::Adt(
                    Adt::Struct(self.ast_visit_emitter.alloc_vec(struct_binding_fields))
                )
            );

            self.ast_visit_emitter.set_namebinding_to_def_id(def_id, name_binding);
            def_id
        };

        self.ast_visit_emitter.set_type_to_node_id(
            struct_item.ident_node.ast_node_id,
            Ty::Adt(def_id)
        );

        self.ast_visit_emitter.set_type_to_node_id(struct_item.ast_node_id, VOID_TY);
        VOID_TY
    }

    fn visit_struct_expr(&mut self, struct_expr: &'ast StructExpr<'ast>) -> Self::Result {
        let symbol = Symbol::new(&self.src[struct_expr.ident_node.span.get_byte_range()]);

        if
            let Some((def_id, name_binding)) = self.ast_visit_emitter.lookup_ident_definition(
                struct_expr.ident_node.span,
                ResKind::Adt
            )
        {
            if let NameBindingKind::Adt(Adt::Struct(struct_fields)) = name_binding.kind {
                let tys_iter = struct_expr.field_initializations
                    .iter()
                    .map(|field| self.visit_expr(field.value))
                    .collect::<Vec<_>>();

                for (i, given_ty) in tys_iter.iter().enumerate() {
                    let (field_name, ty) = struct_fields[i];

                    if
                        !given_ty.test_eq(
                            ty,
                            self.ast_visit_emitter.borrow_def_id_to_name_binding()
                        )
                    {
                        self.ast_visit_emitter.report_error(
                            Error::new(
                                ErrorKind::MismatchedFieldTypes(
                                    def_id.symbol,
                                    field_name.symbol,
                                    ty,
                                    *given_ty
                                ),
                                Span::dummy()
                            )
                        );
                    }

                    self.ast_visit_emitter.set_type_to_node_id(
                        struct_expr.field_initializations[i].ident.ast_node_id,
                        *given_ty
                    );
                }
                let struct_ty = Ty::Adt(def_id);
                self.ast_visit_emitter.set_type_to_node_id(
                    struct_expr.ident_node.ast_node_id,
                    struct_ty
                );
                self.ast_visit_emitter.set_type_to_node_id(struct_expr.ast_node_id, struct_ty);
                struct_ty
            } else {
                // Panics for now
                panic!("Expected struct, got something else");
            }
        } else {
            self.ast_visit_emitter.report_error(
                Error::new(
                    ErrorKind::UndefinedLookup(symbol, ResKind::Adt),
                    struct_expr.ident_node.span
                )
            );
            Ty::Unkown
        }
    }

    fn visit_field_expr(&mut self, field_expr: &'ast FieldExpr<'ast>) -> Self::Result {
        let lhs_ty = self.visit_expr(field_expr.lhs);

        let (struct_name, struct_fields) = match
            lhs_ty.try_deref_as_struct(self.ast_visit_emitter.borrow_def_id_to_name_binding())
        {
            Some(struct_ty) => struct_ty,
            None => {
                self.ast_visit_emitter.report_error(
                    Error::new(ErrorKind::InvalidTuple(lhs_ty), Span::dummy())
                );
                self.ast_visit_emitter.set_type_to_node_id(field_expr.rhs.ast_node_id, Ty::Unkown);
                self.ast_visit_emitter.set_type_to_node_id(field_expr.ast_node_id, Ty::Unkown);
                return Ty::Unkown;
            }
        };

        let field_access_symbol = Symbol::new(&self.src[field_expr.rhs.span.get_byte_range()]);

        for (field_symbol, field_ty) in struct_fields {
            if field_symbol.symbol.get() == field_access_symbol.get() {
                self.ast_visit_emitter.set_type_to_node_id(field_expr.ast_node_id, *field_ty);
                self.ast_visit_emitter.set_type_to_node_id(field_expr.rhs.ast_node_id, *field_ty);
                return *field_ty;
            }
        }

        self.ast_visit_emitter.report_error(
            Error::new(
                ErrorKind::UndefinedStructField(struct_name, field_access_symbol),
                Span::dummy()
            )
        );
        self.ast_visit_emitter.set_type_to_node_id(field_expr.ast_node_id, Ty::Unkown);
        self.ast_visit_emitter.set_type_to_node_id(field_expr.rhs.ast_node_id, Ty::Unkown);
        Ty::Unkown
    }

    fn visit_tuple_field_expr(
        &mut self,
        tuple_field_expr: &'ast crate::TupleFieldExpr<'ast>
    ) -> Self::Result {
        let lhs_ty = self.visit_expr(tuple_field_expr.lhs);
        self.visit_interger_expr(tuple_field_expr.rhs);

        let tuple_ty = match
            lhs_ty.try_deref_as_tuple(self.ast_visit_emitter.borrow_def_id_to_name_binding())
        {
            Some(tuple_ty) => tuple_ty,
            None => {
                self.ast_visit_emitter.report_error(
                    Error::new(ErrorKind::InvalidTuple(lhs_ty), Span::dummy())
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
                let name_binding = self.ast_visit_emitter.get_namebinding_from_def_id(def_id);
                (name_binding, symbol)
            }
            PlaceExpr::TupleFieldExpr(_) => {
                panic!("Not working yet (tuple_field_expr in assignment)")
            }

            PlaceExpr::FieldExpr(_) => { panic!("Not working yet (field_expr in assignment)") }
        };

        match &name_binding.kind {
            NameBindingKind::Variable(mutability) => {
                if *mutability == Mutability::Immutable {
                    self.ast_visit_emitter.report_error(
                        Error::new(ErrorKind::AssignmentToImmutable(symbol), assign_stmt.span)
                    );
                }
            }
            _ => panic!("sfd"),
        }

        if setter_ty.test_eq(value_ty, self.ast_visit_emitter.borrow_def_id_to_name_binding()) {
            self.ast_visit_emitter.set_type_to_node_id(assign_stmt.ast_node_id, VOID_TY);
            // Returns void type, because assignments in itself return void
            VOID_TY
        } else {
            panic!("Not same type in assignment: {}, {}", setter_ty, value_ty);
        }
    }

    fn visit_tuple_expr(&mut self, tuple_expr: &'ast TupleExpr<'ast>) -> Self::Result {
        let mut tuple_types = Vec::with_capacity(8);
        for expr in tuple_expr.fields {
            let ty = self.visit_expr(*expr);
            tuple_types.push(ty);
        }

        let tuple_ty = Ty::Tuple(TyCtx::intern_many_types(tuple_types));

        self.ast_visit_emitter.set_type_to_node_id(tuple_expr.ast_node_id, tuple_ty);

        tuple_ty
    }

    fn visit_def_stmt(&mut self, def_stmt: &'ast DefineStmt<'ast>) -> Self::Result {
        // When tuple patterns are implemented, compare if tuple on lhs, is same as tuple type on rhs
        let value_type = self.visit_expr(def_stmt.value_expr);

        match &def_stmt.setter_expr {
            Pat::IdentPat(ident_pat) => {
                let symbol = Symbol::new(&self.src[ident_pat.span.get_byte_range()]);
                let mutability = if def_stmt.mut_span.get().is_some() {
                    Mutability::Mutable
                } else {
                    Mutability::Immutable
                };
                let def_id = self.ast_visit_emitter.make_def_id(ident_pat.ast_node_id, symbol);
                self.ast_visit_emitter.set_namebinding_to_def_id(
                    def_id,
                    NameBinding::new(NameBindingKind::Variable(mutability))
                );
                self.ast_visit_emitter.bind_def_id_to_lexical_binding(def_id, ResKind::Variable);

                self.ast_visit_emitter.set_type_to_node_id(ident_pat.ast_node_id, value_type);
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

    fn visit_loop_expr(&mut self, loop_expr: &'ast LoopExpr<'ast>) -> Self::Result {
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
        self.ast_visit_emitter.set_type_to_node_id(continue_expr.ast_node_id, VOID_TY);
        VOID_TY
    }

    fn visit_break_expr(&mut self, break_expr: &'ast BreakExpr<'ast>) -> Self::Result {
        let break_ty = break_expr.value
            .map(|expr| self.visit_expr(expr))
            .unwrap_or(Self::default_result());

        if let Some(loop_ret_ty) = self.loop_ret_ty_stack.last_mut() {
            if let Some(expected_ty) = loop_ret_ty {
                if
                    !break_ty.test_eq(
                        *expected_ty,
                        self.ast_visit_emitter.borrow_def_id_to_name_binding()
                    )
                {
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
        let cond_type = self.visit_expr(if_expr.condition);
        if !cond_type.can_be_dereffed_to_bool() {
            self.ast_visit_emitter.report_error(
                Error::new(ErrorKind::ExpectedBoolExpr(cond_type), if_expr.span)
            );
        }

        let true_type = self.visit_block_expr(if_expr.true_block);
        let false_type = if_expr.false_block
            .as_ref()
            .map(|expr| self.visit_if_false_branch_expr(*expr));

        let if_expr_ty = if let Some(false_type) = false_type {
            if
                true_type.test_eq(
                    false_type,
                    self.ast_visit_emitter.borrow_def_id_to_name_binding()
                )
            {
                true_type
            } else {
                Ty::Unkown
            }
        } else {
            true_type
        };

        self.ast_visit_emitter.set_type_to_node_id(if_expr.ast_node_id, if_expr_ty);

        // Returns a pointer to its type
        if_expr_ty
    }

    fn visit_binary_expr(&mut self, binary_expr: &'ast BinaryExpr<'ast>) -> Self::Result {
        let lhs_type = self.visit_expr(binary_expr.lhs);
        let rhs_type = self.visit_expr(binary_expr.rhs);

        let result_ty = lhs_type.test_binary(
            rhs_type,
            binary_expr.op,
            self.ast_visit_emitter.borrow_def_id_to_name_binding()
        );

        if let Some(result_ty) = result_ty {
            self.ast_visit_emitter.set_type_to_node_id(binary_expr.ast_node_id, result_ty);

            result_ty
        } else {
            self.ast_visit_emitter.report_error(
                Error::new(
                    ErrorKind::BinaryExprTypeError(binary_expr.op, lhs_type, rhs_type),
                    Span::dummy()
                )
            );

            Ty::Unkown
        }
    }

    fn visit_group_expr(&mut self, group_expr: &'ast GroupExpr<'ast>) -> Self::Result {
        let expr_type = self.visit_expr(group_expr.expr);

        self.ast_visit_emitter.set_type_to_node_id(group_expr.ast_node_id, expr_type);

        expr_type
    }
}
