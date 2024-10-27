use std::marker::PhantomData;

use crate::{
    ast_query_system::AstQueryEntry,
    ast_state::{ AstState, AstState0, AstState1, AstState2, AstState3 },
    get_node_id_from_expr,
    typechecker::{ ArgCmp, TypeChecker },
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
    walk_call_expr,
    walk_comp_decl_item,
    walk_field_expr,
    walk_index_expr,
    walk_loop_expr,
    walk_path_field,
    walk_stmts_none_items_but_fns,
    walk_struct_expr,
    walk_tuple_expr,
    walk_tuple_field_expr,
    walk_tuple_struct_pat,
    AssignStmt,
    Ast,
    BinaryExpr,
    BlockExpr,
    BoolExpr,
    BreakExpr,
    CallExpr,
    CompDeclItem,
    CompFnDeclItem,
    CondKind,
    ContinueExpr,
    DefineStmt,
    EnumItem,
    FieldExpr,
    FnItem,
    GroupExpr,
    IdentNode,
    IfExpr,
    IntegerExpr,
    ItemStmt,
    ItemType,
    LoopExpr,
    NullExpr,
    Pat,
    PathField,
    PlaceExpr,
    ReturnExpr,
    Stmt,
    StructExpr,
    StructItem,
    TupleExpr,
    TupleFieldExpr,
    TupleStructPat,
    TypedefItem,
    Typing,
};
use error::{ Error, ErrorKind };
use ir::{
    Adt,
    DefId,
    DefIdToNameBinding,
    EmumVaraintId,
    Externism,
    FnSig,
    Mutability,
    NameBinding,
    NameBindingKind,
    NodeId,
    ResKind,
    NEVER_TY,
    NULL_TY,
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

    /// First option checks if it's present (meain if we're inside a loop or function)
    /// Second one checks whether or not the ty is set
    loop_ret_ty: Option<Option<Ty>>,
    fn_ret_ty: Option<Ty>,

    /// Can call functions on the Resolver
    pub ast_visit_emitter: &'b mut E,
}

impl<'ctx, 'ast, 'b, T, E> AstVisitor<'ctx, 'ast, 'b, T, E>
    where T: AstState, E: AstVisitEmitter<'ctx, 'ast, T>
{
    pub fn new(ast: Ast<'ast, T>, src: &'b str, ast_visit_emitter: &'b mut E) -> Self {
        Self {
            ast,
            loop_ret_ty: None,
            fn_ret_ty: None,
            src,
            ast_visit_emitter,
            marker: PhantomData,
        }
    }
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
    fn append_comp_decl(&mut self, comp_fn_decl: CompDeclItem<'ast>);
    fn set_def_id_to_global_mem(&mut self, def_id: DefId);
    fn get_ty_from_node_id(&self, node_id: NodeId) -> Ty;

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

    fn visit_tuple_expr(&mut self, tuple_expr: &'ast TupleExpr<'ast>) -> Self::Result {
        self.insert_query_entry(tuple_expr.ast_node_id, AstQueryEntry::TupleExpr(tuple_expr));
        walk_tuple_expr(self, tuple_expr)
    }

    fn visit_bool_expr(&mut self, bool_expr: &'ast BoolExpr) -> Self::Result {
        self.insert_query_entry(bool_expr.ast_node_id, AstQueryEntry::BoolExpr(bool_expr));
    }

    fn visit_index_expr(&mut self, index_expr: &'ast crate::IndexExpr<'ast>) -> Self::Result {
        self.insert_query_entry(index_expr.ast_node_id, AstQueryEntry::IndexExpr(index_expr));
        walk_index_expr(self, index_expr)
    }

    fn visit_def_stmt(&mut self, def_stmt: &'ast DefineStmt<'ast>) -> Self::Result {
        self.insert_query_entry(def_stmt.ast_node_id, AstQueryEntry::DefineStmt(def_stmt));
        walk_def_stmt(self, def_stmt)
    }

    fn visit_loop_expr(&mut self, loop_expr: &'ast LoopExpr<'ast>) -> Self::Result {
        self.insert_query_entry(loop_expr.ast_node_id, AstQueryEntry::LoopExpr(loop_expr));
        walk_loop_expr(self, loop_expr)
    }

    fn visit_break_expr(&mut self, break_expr: &'ast BreakExpr<'ast>) -> Self::Result {
        self.insert_query_entry(break_expr.ast_node_id, AstQueryEntry::BreakExpr(break_expr));
        walk_break_expr(self, break_expr)
    }

    fn visit_null_expr(&mut self, null_expr: &'ast NullExpr) -> Self::Result {
        self.insert_query_entry(null_expr.ast_node_id, AstQueryEntry::NullExpr(null_expr));
    }

    fn visit_tuple_field_expr(
        &mut self,
        tuple_field_expr: &'ast TupleFieldExpr<'ast>
    ) -> Self::Result {
        self.insert_query_entry(
            tuple_field_expr.ast_node_id,
            AstQueryEntry::TupleFieldExpr(tuple_field_expr)
        );
        walk_tuple_field_expr(self, tuple_field_expr)
    }

    fn visit_call_expr(&mut self, call_expr: &'ast CallExpr<'ast>) -> Self::Result {
        self.insert_query_entry(call_expr.ast_node_id, AstQueryEntry::CallExpr(call_expr));
        walk_call_expr(self, call_expr)
    }

    fn visit_field_expr(&mut self, field_expr: &'ast FieldExpr<'ast>) -> Self::Result {
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

    fn visit_path_field(&mut self, path_field: &'ast PathField<'ast>) -> Self::Result {
        self.insert_query_entry(path_field.ast_node_id, AstQueryEntry::PathField(path_field));
        walk_path_field(self, path_field)
    }

    fn visit_path_segment(&mut self, path_segment: &'ast IdentNode) -> Self::Result {
        self.insert_query_entry(path_segment.ast_node_id, AstQueryEntry::IdentNode(path_segment))
    }

    fn visit_tuple_struct_pat(&mut self, tuple_pat: &'ast TupleStructPat<'ast>) -> Self::Result {
        self.insert_query_entry(tuple_pat.ast_node_id, AstQueryEntry::TupleStructPat(tuple_pat));
        walk_tuple_struct_pat(self, tuple_pat)
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

    fn visit_comp_fn_decl_item(
        &mut self,
        comp_fn_decl_item: &'ast CompFnDeclItem<'ast>
    ) -> Self::Result {
        self.insert_query_entry(
            comp_fn_decl_item.ast_node_id,
            AstQueryEntry::CompFnDeclItem(comp_fn_decl_item)
        );
        self.visit_ident_expr(comp_fn_decl_item.ident_node);
        for arg in comp_fn_decl_item.args {
            self.visit_ident_expr(arg.ident);
        }
    }

    fn visit_struct_item(&mut self, struct_item: &'ast StructItem<'ast>) -> Self::Result {
        self.insert_query_entry(struct_item.ast_node_id, AstQueryEntry::StructItem(struct_item));
        self.visit_ident_expr(struct_item.ident_node);
        for field_decl in struct_item.field_declarations {
            self.visit_ident_expr(field_decl.ident);
        }
    }

    fn visit_enum_item(&mut self, enum_item: &'ast EnumItem<'ast>) -> Self::Result {
        self.insert_query_entry(enum_item.ast_node_id, AstQueryEntry::EnumItem(enum_item));
        self.visit_ident_expr(enum_item.ident_node);
        for variant in enum_item.variants {
            self.visit_ident_expr(variant.ident_node);
        }
    }

    fn visit_typedef_item(&mut self, typedef_item: &'ast TypedefItem<'ast>) -> Self::Result {
        self.insert_query_entry(typedef_item.ast_node_id, AstQueryEntry::TypedefItem(typedef_item));
        self.visit_ident_expr(typedef_item.ident_node);
    }

    fn visit_fn_item(&mut self, fn_item: &'ast FnItem<'ast>) -> Self::Result {
        self.insert_query_entry(fn_item.ast_node_id, AstQueryEntry::FnItem(fn_item));
        self.visit_ident_expr(fn_item.ident_node);
        for arg in fn_item.args {
            self.visit_ident_expr(arg.ident);
        }
        self.visit_stmts(fn_item.body);
    }

    fn visit_return_expr(&mut self, return_expr: &'ast ReturnExpr) -> Self::Result {
        self.insert_query_entry(return_expr.ast_node_id, AstQueryEntry::ReturnExpr(return_expr));
        if let Some(expr) = return_expr.value {
            self.visit_expr(expr);
        }
    }
}

fn type_from_typing<'ctx, 'ast, 'b, E, T: AstState>(
    src: &str,
    typing: &Typing<'ast>,
    e: &'b mut E,
    item_type: ItemType
) -> Ty
    where E: AstVisitEmitter<'ctx, 'ast, T>
{
    match typing {
        Typing::Ident(span) => {
            let lexeme = &src[span.get_byte_range()];
            match lexeme {
                "Int" => INT_TY,
                "Bool" => BOOL_TY,
                "Void" => VOID_TY,
                str => {
                    if let Some(def_id) = e.lookup_ident_declaration(*span, ResKind::Adt) {
                        Ty::Adt(def_id)
                    } else {
                        panic!("Expected Algebric Data Type: {}", str);
                    }
                }
            }
        }
        Typing::Tuple(tuple) => {
            let mut tuple_ty = Vec::with_capacity(tuple.len());
            for typing in tuple.iter() {
                tuple_ty.push(type_from_typing(src, typing, e, item_type));
            }
            Ty::Tuple(TyCtx::intern_many_types(tuple_ty))
        }
        Typing::Fn(args_typing, ret_typing) => {
            let args_ty = {
                let mut args_ty = Vec::with_capacity(args_typing.len());
                for typing in args_typing.iter() {
                    args_ty.push(type_from_typing(src, typing, e, item_type));
                }
                TyCtx::intern_many_types(args_ty)
            };
            let ret_ty = ret_typing.map(|typing| type_from_typing(src, typing, e, item_type));
            Ty::FnSig(FnSig::new(args_ty, TyCtx::intern_type(ret_ty.unwrap_or(VOID_TY))))
        }
        Typing::Ptr(typing, mutability) => {
            if item_type == ItemType::Normal {
                todo!("Report error: Cannot use pointers in this context");
            }
            Ty::Ptr(TyCtx::intern_type(type_from_typing(src, typing, e, item_type)), *mutability)
        }
        Typing::ManyPtr(typing) => {
            if item_type == ItemType::Normal {
                todo!("Report error: Cannot use pointers in this context");
            }

            Ty::ManyPtr(
                TyCtx::intern_type(type_from_typing(src, typing, e, item_type)),
                Mutability::Immutable
            )
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

    fn visit_null_expr(&mut self, null_expr: &'ast NullExpr) -> Self::Result {
        self.ast_visit_emitter.set_type_to_node_id(null_expr.ast_node_id, NULL_TY);
        NULL_TY
    }

    fn visit_block_expr(&mut self, expr: &'ast BlockExpr<'ast>) -> Self::Result {
        self.ast_visit_emitter.start_scope();
        let block_type = self.visit_stmts(expr.stmts);
        self.ast_visit_emitter.end_scope();
        self.ast_visit_emitter.set_type_to_node_id(expr.ast_node_id, block_type);
        block_type
    }

    fn visit_stmts(&mut self, stmts: &'ast [Stmt<'ast>]) -> Self::Result {
        macro_rules! bind_def_id_to_lexical_binding {
            ($res_kind:ident, $item:ident) => {
                {
                    let def_id = self.ast_visit_emitter.make_def_id(
                        $item.ident_node.ast_node_id,
                        Symbol::new(&self.src[$item.ident_node.span.get_byte_range()])
                    );

                    self.ast_visit_emitter.bind_def_id_to_lexical_binding(def_id, ResKind::$res_kind);
                }
            };
        }

        let item_iter = stmts.iter().filter_map(|x| {
            match x {
                Stmt::ItemStmt(item_stmt) => Some(item_stmt),
                _ => None,
            }
        });

        for item_stmt in item_iter.clone() {
            match item_stmt {
                ItemStmt::FnItem(item) => bind_def_id_to_lexical_binding!(Fn, item),
                ItemStmt::StructItem(item) => bind_def_id_to_lexical_binding!(Adt, item),
                ItemStmt::TypedefItem(item) => bind_def_id_to_lexical_binding!(Adt, item),
                ItemStmt::EnumItem(item) => bind_def_id_to_lexical_binding!(Adt, item),
                ItemStmt::CompDeclItem(comp_decl) => {
                    match comp_decl {
                        CompDeclItem::CompFnDeclItem(item) =>
                            bind_def_id_to_lexical_binding!(Fn, item),
                    }
                }
            }
        }

        for item_stmt in item_iter {
            match item_stmt {
                ItemStmt::FnItem(fn_item) => {
                    let def_id = self.ast_visit_emitter.get_def_id_from_node_id(
                        fn_item.ident_node.ast_node_id
                    );

                    let ret_ty = fn_item.return_ty
                        .map(|ty_expr|
                            type_from_typing(
                                &self.src,
                                &ty_expr,
                                self.ast_visit_emitter,
                                fn_item.item_type
                            )
                        )
                        .unwrap_or(VOID_TY);

                    let args_tys = fn_item.args
                        .iter()
                        .map(|arg| {
                            let def_id = self.ast_visit_emitter.make_def_id(
                                arg.ident.ast_node_id,
                                Symbol::new(&self.src[arg.ident.span.get_byte_range()])
                            );
                            self.ast_visit_emitter.set_def_id_to_node_id(
                                arg.ident.ast_node_id,
                                def_id
                            );

                            let ty = type_from_typing(
                                self.src,
                                &arg.type_expr,
                                self.ast_visit_emitter,
                                fn_item.item_type
                            );
                            self.ast_visit_emitter.set_type_to_node_id(arg.ident.ast_node_id, ty);
                            ty
                        })
                        .collect::<Vec<_>>();

                    let name_binding = NameBinding::new(
                        NameBindingKind::Fn(
                            FnSig::new(
                                TyCtx::intern_many_types(args_tys),
                                TyCtx::intern_type(ret_ty)
                            ),
                            Externism::NoExtern
                        )
                    );
                    self.ast_visit_emitter.set_namebinding_to_def_id(def_id, name_binding);
                }
                item => {
                    self.visit_item(*item);
                }
            }
        }

        /*
        TODO: Add last pass of items, where structs, tuples etc. are laid out most optimally in memory

        But I should probably not do it here anyways (vielleicht the last pass of ICFG),
        since each field could potentially be a pointer after analysis in the ICFG¨
        */

        walk_stmts_none_items_but_fns(self, stmts)
    }

    fn visit_ident_expr(&mut self, ident_node: &'ast IdentNode) -> Self::Result {
        if
            let Some(def_id) = self.ast_visit_emitter.lookup_ident_declaration(
                ident_node.span,
                ResKind::Variable
            )
        {
            let mutability = {
                let name_binding = self.ast_visit_emitter.get_namebinding_from_def_id(def_id);
                match name_binding.kind {
                    NameBindingKind::Variable(mutability) => mutability,
                    _ => panic!("Expected variable"),
                }
            };

            self.ast_visit_emitter.set_def_id_to_node_id(ident_node.ast_node_id, def_id);
            let ident_ty = Ty::Ptr(
                TyCtx::intern_type(self.ast_visit_emitter.get_ty_from_def_id(def_id)),
                mutability
            );
            self.ast_visit_emitter.set_type_to_node_id(ident_node.ast_node_id, ident_ty);
            ident_ty
        } else if
            let Some(def_id) = self.ast_visit_emitter.lookup_ident_declaration(
                ident_node.span,
                ResKind::Fn
            )
        {
            self.ast_visit_emitter.set_def_id_to_node_id(ident_node.ast_node_id, def_id);

            match self.ast_visit_emitter.get_namebinding_from_def_id(def_id).kind {
                NameBindingKind::Fn(_, _) => {}
                _ => panic!("Expected function"),
            }
            let ty = Ty::FnDef(def_id);
            self.ast_visit_emitter.set_type_to_node_id(ident_node.ast_node_id, ty);
            ty
        } else if
            let Some(def_id) = self.ast_visit_emitter.lookup_ident_declaration(
                ident_node.span,
                ResKind::Adt
            )
        {
            self.ast_visit_emitter.set_def_id_to_node_id(ident_node.ast_node_id, def_id);
            let ty = Ty::AtdConstructer(def_id);
            self.ast_visit_emitter.set_type_to_node_id(ident_node.ast_node_id, ty);
            ty
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

            let ty = type_from_typing(
                self.src,
                &typedef_item.type_expr,
                self.ast_visit_emitter,
                typedef_item.item_type
            );

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

    fn visit_comp_fn_decl_item(
        &mut self,
        comp_fn_decl_item: &'ast CompFnDeclItem<'ast>
    ) -> Self::Result {
        let def_id = {
            let def_id = self.ast_visit_emitter.get_def_id_from_node_id(
                comp_fn_decl_item.ident_node.ast_node_id
            );
            let ret_ty = comp_fn_decl_item.return_ty
                .map(|ty_expr|
                    type_from_typing(
                        &self.src,
                        &ty_expr,
                        self.ast_visit_emitter,
                        comp_fn_decl_item.item_type
                    )
                )
                .unwrap_or(VOID_TY);

            let mut args_tys = Vec::with_capacity(comp_fn_decl_item.args.len());
            for arg in comp_fn_decl_item.args {
                let def_id = self.ast_visit_emitter.make_def_id(
                    arg.ident.ast_node_id,
                    Symbol::new(&self.src[arg.ident.span.get_byte_range()])
                );
                let res_ty = type_from_typing(
                    self.src,
                    &arg.type_expr,
                    self.ast_visit_emitter,
                    comp_fn_decl_item.item_type
                );
                self.ast_visit_emitter.set_type_to_node_id(arg.ident.ast_node_id, res_ty);

                args_tys.push(res_ty);

                /* Defines arg as a variable */
                self.ast_visit_emitter.set_namebinding_to_def_id(
                    def_id,
                    NameBinding::new(NameBindingKind::Variable(Mutability::Immutable))
                );
                self.ast_visit_emitter.bind_def_id_to_lexical_binding(def_id, ResKind::Variable);
            }

            let name_binding = NameBinding::new(
                NameBindingKind::Fn(
                    FnSig::new(TyCtx::intern_many_types(args_tys), TyCtx::intern_type(ret_ty)),
                    Externism::Clib
                )
            );
            self.ast_visit_emitter.set_namebinding_to_def_id(def_id, name_binding);

            def_id
        };

        self.ast_visit_emitter.set_type_to_node_id(
            comp_fn_decl_item.ident_node.ast_node_id,
            Ty::FnDef(def_id)
        );

        self.ast_visit_emitter.set_type_to_node_id(comp_fn_decl_item.ast_node_id, VOID_TY);

        VOID_TY
    }

    fn visit_fn_item(&mut self, fn_item: &'ast FnItem<'ast>) -> Self::Result {
        let def_id = {
            let def_id = self.ast_visit_emitter.get_def_id_from_node_id(
                fn_item.ident_node.ast_node_id
            );
            let fn_sig = {
                let name_binding = self.ast_visit_emitter.get_namebinding_from_def_id(def_id);
                match name_binding.kind {
                    NameBindingKind::Fn(fn_sig, _) => fn_sig,
                    _ => panic!("Expected function"),
                }
            };
            let ret_ty = *fn_sig.ret_ty;

            {
                let prev_ret_ty = std::mem::replace(&mut self.fn_ret_ty, Some(ret_ty));
                self.fn_ret_ty = Some(ret_ty);
                self.ast_visit_emitter.start_context();

                for arg in fn_item.args {
                    let def_id = self.ast_visit_emitter.get_def_id_from_node_id(
                        arg.ident.ast_node_id
                    );

                    self.ast_visit_emitter.set_namebinding_to_def_id(
                        def_id,
                        NameBinding::new(NameBindingKind::Variable(Mutability::Immutable))
                    );
                    self.ast_visit_emitter.bind_def_id_to_lexical_binding(
                        def_id,
                        ResKind::Variable
                    );
                }

                // Type of the body (not equal to the return type)
                // If the body_ty is the NEVER_TY then the functions ends with a return statement no matter the path (e.g. if expresions)
                let body_ty = self.visit_stmts(fn_item.body);
                if ret_ty != VOID_TY && body_ty != NEVER_TY {
                    self.ast_visit_emitter.report_error(
                        Error::new(ErrorKind::MissingReturn, Span::dummy())
                    );
                }

                self.ast_visit_emitter.end_context();
                self.fn_ret_ty = prev_ret_ty;
            }

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

        self.ast_visit_emitter.set_def_id_to_global_mem(def_id);

        VOID_TY
    }

    fn visit_call_expr(&mut self, call_expr: &'ast CallExpr<'ast>) -> Self::Result {
        let calle_ty = self.visit_expr(call_expr.callee);

        if let Ty::AtdConstructer(enum_variant_def_id) = calle_ty {
            let name_binding =
                self.ast_visit_emitter.get_namebinding_from_def_id(enum_variant_def_id);

            let (enum_def_id, variant_ty) = match name_binding.kind {
                NameBindingKind::Adt(Adt::EnumVariant(enum_def_id, _, variant_ty)) =>
                    (enum_def_id, variant_ty),
                _ => panic!("Expected enum variant"),
            };

            let arg_tys = call_expr.args
                .iter()
                .map(|arg| self.visit_expr(*arg))
                .collect::<Vec<_>>();

            if arg_tys.len() != variant_ty.len() {
                todo!("Expected {} arguments, got {}", variant_ty.len(), arg_tys.len());
            } else {
                for (i, arg_ty) in arg_tys.iter().enumerate() {
                    let is_valid_arg = TypeChecker::test_valid_arg(
                        ArgCmp {
                            arg_ty: variant_ty[i],
                            provided_ty: *arg_ty,
                        },
                        self.ast_visit_emitter.borrow_def_id_to_name_binding()
                    );

                    if let Err(errors) = is_valid_arg {
                        for error in errors {
                            println!("{:?}", error);
                        }

                        todo!("Report error");
                    }
                }
            }

            self.ast_visit_emitter.set_type_to_node_id(
                call_expr.ast_node_id,
                Ty::Adt(enum_variant_def_id)
            );

            return Ty::Adt(enum_def_id);
        }

        let fn_sig = match calle_ty.auto_deref() {
            Ty::FnDef(def_id) => {
                if
                    let NameBindingKind::Fn(fn_sig, _) =
                        self.ast_visit_emitter.get_namebinding_from_def_id(def_id).kind
                {
                    fn_sig
                } else {
                    panic!("Expected function");
                }
            }
            Ty::FnSig(fn_sig) => fn_sig,
            _ => {
                // self.ast_visit_emitter.report_error(
                //     Error::new(ErrorKind::NotCallable, call_expr.callee.span)
                // );
                println!("Not callable");
                return Ty::Unkown;
            }
        };
        let ret_ty = *fn_sig.ret_ty;

        for (i, arg_ty) in fn_sig.args.iter().enumerate() {
            let arg = if let Some(arg) = call_expr.args.get(i) {
                arg
            } else {
                // self.ast_visit_emitter.report_error(
                //     Error::new(ErrorKind::MissingArg, call_expr.span)
                // );
                panic!("Missing arg");
                break;
            };

            let given_arg_ty = self.visit_expr(*arg);

            let arg_cmp = ArgCmp {
                arg_ty: *arg_ty,
                provided_ty: given_arg_ty,
            };

            let is_valid_arg = TypeChecker::test_valid_arg(
                arg_cmp,
                self.ast_visit_emitter.borrow_def_id_to_name_binding()
            );

            if let Err(errors) = is_valid_arg {
                for error in errors {
                    println!("{:?}", error);
                }

                todo!("Report error");
            }
        }

        self.ast_visit_emitter.set_type_to_node_id(call_expr.ast_node_id, ret_ty);
        ret_ty
    }

    fn visit_return_expr(&mut self, return_expr: &'ast ReturnExpr) -> Self::Result {
        let ret_ty = if let Some(expr) = return_expr.value {
            self.visit_expr(expr)
        } else {
            VOID_TY
        };

        if let Some(fn_ret_ty) = self.fn_ret_ty {
            if
                let Err(errors) = TypeChecker::test_eq_loose(
                    ret_ty,
                    fn_ret_ty,
                    self.ast_visit_emitter.borrow_def_id_to_name_binding()
                )
            {
                for error in errors {
                    println!("{:?}", error);
                }

                self.ast_visit_emitter.report_error(
                    Error::new(ErrorKind::MismatchedReturnTypes(fn_ret_ty, ret_ty), Span::dummy())
                );
            }
        } else {
            self.ast_visit_emitter.report_error(
                Error::new(ErrorKind::ReturnOutsideFn, Span::dummy())
            );
        }

        self.ast_visit_emitter.set_type_to_node_id(return_expr.ast_node_id, NEVER_TY);
        NEVER_TY
    }

    fn visit_enum_item(&mut self, enum_item: &'ast EnumItem<'ast>) -> Self::Result {
        let def_id = {
            let enum_def_id = self.ast_visit_emitter.get_def_id_from_node_id(
                enum_item.ident_node.ast_node_id
            );

            let mut variants = Vec::with_capacity(enum_item.variants.len());

            self.ast_visit_emitter.start_scope();

            for (i, variant) in enum_item.variants.iter().enumerate() {
                let def_id = self.ast_visit_emitter.make_def_id(
                    variant.ident_node.ast_node_id,
                    Symbol::new(&self.src[variant.ident_node.span.get_byte_range()])
                );

                let res_ty = {
                    let res_ty = match variant.enum_data {
                        Some(typing) => {
                            let mut res_ty = Vec::with_capacity(typing.len());
                            for field in typing.iter() {
                                res_ty.push(
                                    type_from_typing(
                                        self.src,
                                        field,
                                        self.ast_visit_emitter,
                                        enum_item.item_type
                                    )
                                );
                            }
                            res_ty
                        }
                        None => vec![Ty::ZeroSized],
                    };
                    TyCtx::intern_many_types(res_ty)
                };

                self.ast_visit_emitter.set_type_to_node_id(
                    variant.ident_node.ast_node_id,
                    Ty::AtdConstructer(def_id)
                );

                let name_binding = NameBinding::new(
                    NameBindingKind::Adt(
                        Adt::EnumVariant(enum_def_id, EmumVaraintId(i as u32), res_ty)
                    )
                );
                self.ast_visit_emitter.set_namebinding_to_def_id(def_id, name_binding);

                variants.push(def_id);
            }
            self.ast_visit_emitter.end_scope();

            let name_binding = NameBinding::new(
                NameBindingKind::Adt(Adt::Enum(self.ast_visit_emitter.alloc_vec(variants)))
            );

            self.ast_visit_emitter.set_namebinding_to_def_id(enum_def_id, name_binding);

            enum_def_id
        };

        self.ast_visit_emitter.set_type_to_node_id(
            enum_item.ident_node.ast_node_id,
            Ty::Adt(def_id)
        );

        self.ast_visit_emitter.set_type_to_node_id(enum_item.ast_node_id, VOID_TY);
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
                let res_ty = type_from_typing(
                    self.src,
                    &field.type_expr,
                    self.ast_visit_emitter,
                    struct_item.item_type
                );

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
        let lhs_ty = self.visit_ident_expr(struct_expr.ident_node);

        let atd_constructer_def_id = match lhs_ty {
            Ty::AtdConstructer(def_id) => def_id,
            _ => {
                self.ast_visit_emitter.report_error(
                    Error::new(ErrorKind::InvalidStruct(lhs_ty), Span::dummy())
                );
                self.ast_visit_emitter.set_type_to_node_id(struct_expr.ast_node_id, Ty::Unkown);
                return Ty::Unkown;
            }
        };

        let name_binding =
            self.ast_visit_emitter.get_namebinding_from_def_id(atd_constructer_def_id);

        let struct_fields = match name_binding.kind {
            NameBindingKind::Adt(Adt::Struct(struct_fields)) => struct_fields,
            _ => panic!("Expected struct got something else"),
        };

        let tys_iter = struct_expr.field_initializations
            .iter()
            .map(|field| self.visit_expr(field.value))
            .collect::<Vec<_>>();

        for (i, given_ty) in tys_iter.iter().enumerate() {
            let (field_name, ty) = struct_fields[i];

            if
                let Err(errors) = TypeChecker::test_eq_loose(
                    *given_ty,
                    ty,
                    self.ast_visit_emitter.borrow_def_id_to_name_binding()
                )
            {
                self.ast_visit_emitter.report_error(
                    Error::new(
                        ErrorKind::MismatchedFieldTypes(
                            atd_constructer_def_id.symbol,
                            field_name.symbol,
                            *given_ty,
                            ty
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
        let struct_ty = Ty::Adt(atd_constructer_def_id);
        self.ast_visit_emitter.set_type_to_node_id(struct_expr.ident_node.ast_node_id, struct_ty);
        self.ast_visit_emitter.set_type_to_node_id(struct_expr.ast_node_id, struct_ty);
        struct_ty
    }

    fn visit_field_expr(&mut self, field_expr: &'ast FieldExpr<'ast>) -> Self::Result {
        let lhs_ty = self.visit_expr(field_expr.lhs);

        if let Ty::AtdConstructer(def_id) = lhs_ty {
            // This must be an enum, because an enum's constructor requires a field expression e.g. Option.Some(5)
            let name_binding = self.ast_visit_emitter.get_namebinding_from_def_id(def_id);
            if let NameBindingKind::Adt(Adt::Enum(enum_variants)) = name_binding.kind {
                let rhs_symbol = Symbol::new(&self.src[field_expr.rhs.span.get_byte_range()]);

                let variant_def_id = enum_variants
                    .iter()
                    .find(|x| x.symbol.get() == rhs_symbol.get());

                if let Some(variant_def_id) = variant_def_id {
                    let field_ty = Ty::AtdConstructer(*variant_def_id);
                    self.ast_visit_emitter.set_def_id_to_node_id(
                        field_expr.rhs.ast_node_id,
                        *variant_def_id
                    );
                    self.ast_visit_emitter.set_type_to_node_id(
                        field_expr.rhs.ast_node_id,
                        field_ty
                    );
                    self.ast_visit_emitter.set_type_to_node_id(field_expr.ast_node_id, field_ty);

                    return field_ty;
                } else {
                    todo!("Undefined variant: {}", rhs_symbol.get());
                }
            } else {
                panic!("Expected enum");
            }
        } else {
            // As of now if we are here, we should expect a struct (tuples have their own field expression)
            let (def_id, adt) = {
                let adt = lhs_ty.try_deref_as_adt(
                    self.ast_visit_emitter.borrow_def_id_to_name_binding()
                );

                match adt {
                    Some(adt) => adt,
                    None => {
                        self.ast_visit_emitter.report_error(
                            Error::new(ErrorKind::InvalidStruct(lhs_ty), Span::dummy())
                        );

                        self.ast_visit_emitter.set_type_to_node_id(
                            field_expr.rhs.ast_node_id,
                            Ty::Unkown
                        );
                        self.ast_visit_emitter.set_type_to_node_id(
                            field_expr.ast_node_id,
                            Ty::Unkown
                        );

                        return Ty::Unkown;
                    }
                }
            };

            let struct_fields = match adt {
                Adt::Struct(struct_fields) => struct_fields,
                _ => panic!("Expected struct in field expression"),
            };

            let field_access_symbol = Symbol::new(&self.src[field_expr.rhs.span.get_byte_range()]);

            for (field_symbol, field_ty) in struct_fields {
                if field_symbol.symbol.get() == field_access_symbol.get() {
                    let mutability = if lhs_ty.deref_until_single_ptr().is_mut_ptr() {
                        Mutability::Mutable
                    } else {
                        Mutability::Immutable
                    };

                    let field_ty = Ty::Ptr(TyCtx::intern_type(*field_ty), mutability);

                    self.ast_visit_emitter.set_type_to_node_id(field_expr.ast_node_id, field_ty);
                    self.ast_visit_emitter.set_type_to_node_id(
                        field_expr.rhs.ast_node_id,
                        field_ty
                    );
                    self.ast_visit_emitter.set_def_id_to_node_id(
                        field_expr.rhs.ast_node_id,
                        def_id
                    );

                    return field_ty;
                }
            }

            self.ast_visit_emitter.report_error(
                Error::new(
                    ErrorKind::UndefinedStructField(def_id.symbol, field_access_symbol),
                    Span::dummy()
                )
            );
            self.ast_visit_emitter.set_type_to_node_id(field_expr.ast_node_id, Ty::Unkown);
            self.ast_visit_emitter.set_type_to_node_id(field_expr.rhs.ast_node_id, Ty::Unkown);

            Ty::Unkown
        }
    }

    fn visit_tuple_struct_pat(&mut self, tuple_pat: &'ast TupleStructPat<'ast>) -> Self::Result {
        let ty = self.visit_path(tuple_pat.path);

        let (def_id, adt) = match ty {
            Ty::AtdConstructer(def_id) => {
                let name_binding = self.ast_visit_emitter.get_namebinding_from_def_id(def_id);
                match name_binding.kind {
                    NameBindingKind::Adt(adt) => (def_id, adt),
                    _ => panic!("Expected adt"),
                }
            }
            _ => panic!("Expected adt"),
        };

        match adt {
            Adt::EnumVariant(enum_def_id, enum_variant_id, enum_ty) => {
                let ty = Ty::Adt(def_id);
                self.ast_visit_emitter.set_type_to_node_id(tuple_pat.ast_node_id, ty);
                self.ast_visit_emitter.set_def_id_to_node_id(tuple_pat.ast_node_id, def_id);

                if enum_ty.len() != tuple_pat.fields.len() {
                    todo!("Expected {} fields, got {}", enum_ty.len(), tuple_pat.fields.len());
                }

                for (i, pat) in tuple_pat.fields.iter().enumerate() {
                    let field_ty = enum_ty[i];

                    if let Pat::IdentPat(ident_pat) = pat {
                        let symbol = Symbol::new(&self.src[ident_pat.span.get_byte_range()]);
                        let def_id = self.ast_visit_emitter.make_def_id(
                            ident_pat.ast_node_id,
                            symbol
                        );

                        self.ast_visit_emitter.bind_def_id_to_lexical_binding(
                            def_id,
                            ResKind::Variable
                        );
                        self.ast_visit_emitter.set_namebinding_to_def_id(
                            def_id,
                            NameBinding::new(NameBindingKind::Variable(Mutability::Immutable))
                        );
                        self.ast_visit_emitter.set_type_to_node_id(ident_pat.ast_node_id, field_ty);
                    } else {
                        self.visit_pat(*pat);
                    }
                }

                return ty;
            }
            // Adt::TupleStruct(_) => todo!("Tuple struct"),
            _ => panic!("Expected enum variant"),
        }
    }

    fn visit_ident_pat(&mut self, ident_node: &'ast IdentNode) -> Self::Result {
        Ty::Unkown
    }

    fn visit_path_field(&mut self, path_field: &'ast PathField<'ast>) -> Self::Result {
        let lhs_ty = self.visit_path(path_field.lhs);
        let rhs_symbol = Symbol::new(&self.src[path_field.rhs.span.get_byte_range()]);

        let def_id = match lhs_ty {
            Ty::AtdConstructer(def_id) => def_id,
            _ => panic!("Invalid lhs of path field"),
        };

        let name_binding = self.ast_visit_emitter.get_namebinding_from_def_id(def_id);

        match name_binding.kind {
            NameBindingKind::Adt(Adt::Struct(fields)) => {
                todo!("Lookup constructer method for struct");
            }
            NameBindingKind::Adt(Adt::Enum(variants)) => {
                let variant_def_id = variants.iter().find(|x| x.symbol.get() == rhs_symbol.get());

                if let Some(variant_def_id) = variant_def_id {
                    let atd_constructor_ty = Ty::AtdConstructer(*variant_def_id);
                    self.ast_visit_emitter.set_type_to_node_id(
                        path_field.rhs.ast_node_id,
                        atd_constructor_ty
                    );
                    self.ast_visit_emitter.set_type_to_node_id(
                        path_field.ast_node_id,
                        atd_constructor_ty
                    );
                    return atd_constructor_ty;
                } else {
                    todo!(
                        "Undefined variant: {}. Lookup constructer method instead (maybe this should be illegal here)",
                        rhs_symbol.get()
                    );
                }
            }
            _ => panic!("Invalid lhs of path field"),
        }
    }

    fn visit_path_segment(&mut self, path_segment: &'ast IdentNode) -> Self::Result {
        let ty = self.visit_ident_expr(path_segment);
        self.ast_visit_emitter.set_type_to_node_id(path_segment.ast_node_id, ty);
        ty
    }

    fn visit_tuple_field_expr(
        &mut self,
        tuple_field_expr: &'ast TupleFieldExpr<'ast>
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
            let mutability = if lhs_ty.deref_until_single_ptr().is_mut_ptr() {
                Mutability::Mutable
            } else {
                Mutability::Immutable
            };

            let access_ty = Ty::Ptr(
                TyCtx::intern_type(tuple_ty[tuple_field_expr.rhs.val as usize]),
                mutability
            );

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
                let name_binding = self.ast_visit_emitter.get_namebinding_from_def_id(def_id);
                (name_binding, def_id.symbol)
            }
            PlaceExpr::TupleFieldExpr(_) => {
                panic!("Not working yet (tuple_field_expr in assignment)")
            }

            PlaceExpr::FieldExpr(field_expr) => {
                let def_id = self.ast_visit_emitter.get_def_id_from_node_id(
                    field_expr.rhs.ast_node_id
                );
                let name_binding = self.ast_visit_emitter.get_namebinding_from_def_id(def_id);
                (name_binding, def_id.symbol)
            }
            PlaceExpr::IndexExpr(index_expr) => { todo!("Index expression in assignment") }
        };

        if !setter_ty.is_mut_ptr() {
            self.ast_visit_emitter.report_error(
                Error::new(ErrorKind::AssignmentToImmutable(symbol), assign_stmt.span)
            );
        }

        if
            let Err(errors) = TypeChecker::test_eq_loose(
                setter_ty,
                value_ty,
                self.ast_visit_emitter.borrow_def_id_to_name_binding()
            )
        {
            panic!("Not same type in assignment: {}, {}", setter_ty, value_ty);
        } else {
            self.ast_visit_emitter.set_type_to_node_id(assign_stmt.ast_node_id, VOID_TY);
            // Returns void type, because assignments in itself return void
            VOID_TY
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

                let value_type = self.visit_expr(def_stmt.value_expr);

                self.ast_visit_emitter.set_type_to_node_id(ident_pat.ast_node_id, value_type);
            }
            _ => todo!("Tuple patterns"),
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
        let prev_loop_ret_ty = std::mem::replace(&mut self.loop_ret_ty, Some(None));

        self.visit_block_expr(loop_expr.body);

        let ty = self.loop_ret_ty
            .expect("This is always present pushed above")
            .unwrap_or(Self::default_result());

        self.loop_ret_ty = prev_loop_ret_ty;

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

        if let Some(loop_ret_ty) = &mut self.loop_ret_ty {
            if let Some(expected_ty) = loop_ret_ty {
                if
                    let Err(errors) = TypeChecker::test_eq_loose(
                        *expected_ty,
                        break_ty,
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
        match if_expr.cond_kind {
            CondKind::CondExpr(cond_expr) => {
                let cond_type = self.visit_expr(cond_expr);
                if !cond_type.can_be_dereffed_to_bool() {
                    self.ast_visit_emitter.report_error(
                        Error::new(ErrorKind::ExpectedBoolExpr(cond_type), if_expr.span)
                    );
                }
                self.ast_visit_emitter.start_scope();
            }
            CondKind::CondPat(pat, rhs_expr) => {
                self.visit_expr(rhs_expr);

                self.ast_visit_emitter.start_scope();

                self.visit_pat(pat);
            }
        }

        let true_type = self.visit_stmts(if_expr.true_block);

        self.ast_visit_emitter.end_scope();

        let false_type = if_expr.false_block
            .as_ref()
            .map(|expr| self.visit_if_false_branch_expr(*expr));

        let if_expr_ty = if let Some(false_type) = false_type {
            if
                let Err(errors) = TypeChecker::test_eq_loose(
                    true_type,
                    false_type,
                    self.ast_visit_emitter.borrow_def_id_to_name_binding()
                )
            {
                Ty::Unkown
            } else {
                true_type
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
