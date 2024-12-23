use std::marker::PhantomData;

use crate::{
    ast_state::{ AstResolved, AstState, AstTypeChecked, AstUnvalidated },
    get_ident_node_from_arg_kind,
    typechecker::{ ArgCmp, TypeChecker },
    visitor::Visitor,
    walk_impl_item,
    walk_stmts_none_items_but_fns,
    ArgKind,
    AsigneeExpr,
    AssignStmt,
    Ast,
    AstPartlyResolved,
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
    ImplItem,
    ImportItem,
    IndexExpr,
    IntegerExpr,
    ItemStmt,
    ItemType,
    LoopExpr,
    NullExpr,
    Pat,
    Path,
    PathField,
    ReturnExpr,
    Stmt,
    StringExpr,
    StructExpr,
    StructItem,
    TupleExpr,
    TupleFieldExpr,
    TupleStructPat,
    TypedefItem,
    Typing,
};
use error::{ Error, ErrorKind };
use fxhash::FxHashMap;
use ir::{
    Adt,
    ContextId,
    DefId,
    DefIdToNameBinding,
    EmumVaraintId,
    Externism,
    FnSig,
    HasSelfArg,
    LexicalContext,
    Mutability,
    NameBinding,
    NameBindingKind,
    NodeId,
    ResKind,
    ScopeId,
    TraitImplId,
    BIG_SELF_SYMBOL,
    BOOL_SYMBOL,
    FLOAT_32_SYMBOL,
    FLOAT_32_TY,
    FLOAT_64_SYMBOL,
    FLOAT_64_TY,
    FLOAT_SYMBOL,
    INT16_SYMBOL,
    INT32_SYMBOL,
    INT64_SYMBOL,
    INT8_SYMBOL,
    INT_16_TY,
    INT_32_TY,
    INT_64_TY,
    INT_8_TY,
    INT_SYMBOL,
    MAIN_SYMBOL,
    NEVER_TY,
    NULL_TY,
    STR_SYMBOL,
    STR_TY,
    UINT16_SYMBOL,
    UINT32_SYMBOL,
    UINT64_SYMBOL,
    UINT8_SYMBOL,
    UINT_16_TY,
    UINT_32_TY,
    UINT_64_TY,
    UINT_8_TY,
    UINT_SYMBOL,
    UNKOWN_TY,
    VOID_SYMBOL,
};

use span::Span;
use ir::{ Symbol, Ty, TyCtx, BOOL_TY, VOID_TY };

#[derive(Debug)]
pub struct AstPreResolver<'ctx, 'ast, 'b, E> where E: ResolverHandle<'ctx, 'ast, AstUnvalidated> {
    ast: Ast<'ast, AstUnvalidated>,
    resolver_handle: &'b mut E,
    lexical_context_stack: Vec<LexicalContext>,
    lexical_context_to_parent_lexical_context: FxHashMap<LexicalContext, LexicalContext>,
    next_scope_id: ScopeId,
    next_context_id: ContextId,
    is_in_impl: bool,
    marker: PhantomData<&'ctx ()>,
}

impl<'ctx, 'ast, 'b, E> AstPreResolver<'ctx, 'ast, 'b, E>
    where E: ResolverHandle<'ctx, 'ast, AstUnvalidated>, 'ctx: 'ast, 'ast: 'b
{
    pub fn new(ast: Ast<'ast, AstUnvalidated>, resolver_handle: &'b mut E) -> Self {
        Self {
            ast,
            resolver_handle,
            lexical_context_stack: vec![LexicalContext::new(ContextId(0), ScopeId(0))],
            lexical_context_to_parent_lexical_context: FxHashMap::default(),
            next_scope_id: ScopeId(1),
            next_context_id: ContextId(1),
            is_in_impl: false,
            marker: PhantomData,
        }
    }

    pub fn visit(
        mut self
    ) -> (Ast<'ast, AstPartlyResolved>, FxHashMap<LexicalContext, LexicalContext>) {
        self.visit_stmts(self.ast.main_scope.stmts);
        (self.ast.next_state(), self.lexical_context_to_parent_lexical_context)
    }

    fn start_scope(&mut self) {
        let prev_lexical_context = self.get_lexical_context();
        let new_lexical_context = LexicalContext::new(
            self.get_current_context_id(),
            self.next_scope_id
        );
        self.lexical_context_stack.push(new_lexical_context);

        self.lexical_context_to_parent_lexical_context.insert(
            new_lexical_context,
            prev_lexical_context
        );

        self.next_scope_id = ScopeId(self.next_scope_id.0 + 1);
    }
    fn end_scope(&mut self) {
        self.lexical_context_stack.pop();
    }
    fn start_impl_context(&mut self) {
        self.is_in_impl = true;
        self.start_context();
    }
    fn end_impl_context(&mut self) {
        self.is_in_impl = false;
        self.end_context();
    }
    fn start_context(&mut self) {
        let prev_lexical_context = self.get_lexical_context();
        let new_lexical_context = LexicalContext::new(self.next_context_id, self.next_scope_id);
        self.lexical_context_stack.push(new_lexical_context);

        self.lexical_context_to_parent_lexical_context.insert(
            new_lexical_context,
            prev_lexical_context
        );

        self.next_context_id = ContextId(self.next_context_id.0 + 1);
        self.next_scope_id = ScopeId(self.next_scope_id.0 + 1);
    }
    fn end_context(&mut self) {
        let start_context_id = self.get_current_context_id();
        while self.get_current_context_id() == start_context_id {
            self.lexical_context_stack.pop();
        }
    }

    fn visit_typing(&mut self, typing: &Typing<'ast>) {
        match typing {
            Typing::SelfType | Typing::VariadicArgs => {}
            Typing::Ident(ident_node) => {
                self.bind_node_id_to_lexical_context(ident_node.ast_node_id);
            }
            Typing::Tuple(tuple) => {
                for typing in tuple.iter() {
                    self.visit_typing(typing);
                }
            }
            Typing::Fn(args_typing, ret_typing) => {
                for typing in args_typing.iter() {
                    self.visit_typing(typing);
                }
                if let Some(ret_typing) = ret_typing {
                    self.visit_typing(ret_typing);
                }
            }
            Typing::Ptr(typing, _) | Typing::ManyPtr(typing) => {
                self.visit_typing(typing);
            }
        }
    }

    fn traverse_pat_and_bind_idents(&mut self, pat: Pat<'ast>) {
        match pat {
            Pat::IdentPat(ident_node) => {
                self.resolver_handle.make_def_id_and_bind_to_node_id(
                    ident_node.ast_node_id,
                    Symbol::from_node_id(ident_node.ast_node_id)
                );
                self.bind_node_id_to_lexical_context(ident_node.ast_node_id);
            }
            Pat::TupleStructPat(tuple_struct_pat) => {
                self.traverse_path_and_bind_idents(tuple_struct_pat.path);
                for field in tuple_struct_pat.fields.iter() {
                    self.traverse_pat_and_bind_idents(*field);
                }
            }
        }
    }

    fn traverse_path_and_bind_idents(&mut self, path: Path<'ast>) {
        match path {
            Path::PathField(path_field) => {
                self.traverse_path_and_bind_idents(path_field.lhs);
                self.bind_node_id_to_lexical_context(path_field.rhs.ast_node_id);
            }
            Path::PathSegment(ident_node) => {
                self.bind_node_id_to_lexical_context(ident_node.ast_node_id);
            }
        }
    }

    fn get_current_scope_id(&self) -> ScopeId {
        self.lexical_context_stack.last().expect("Expected at least one scope").scope_id
    }

    fn get_current_context_id(&self) -> ContextId {
        self.lexical_context_stack.last().expect("Expected at least one context").context_id
    }

    fn get_lexical_context(&self) -> LexicalContext {
        *self.lexical_context_stack.last().expect("Expected at least one lexical context")
    }

    fn bind_node_id_to_lexical_context(&mut self, node_id: NodeId) {
        let lexical_context = self.get_lexical_context();
        self.resolver_handle.set_node_id_to_lexical_context(node_id, lexical_context);
    }

    fn is_compiler_adt_name(&self, symbol: Symbol) -> bool {
        symbol == *INT_SYMBOL ||
            symbol == *UINT_SYMBOL ||
            symbol == *FLOAT_SYMBOL ||
            symbol == *INT8_SYMBOL ||
            symbol == *INT16_SYMBOL ||
            symbol == *INT32_SYMBOL ||
            symbol == *INT64_SYMBOL ||
            symbol == *UINT8_SYMBOL ||
            symbol == *UINT16_SYMBOL ||
            symbol == *UINT32_SYMBOL ||
            symbol == *UINT64_SYMBOL ||
            symbol == *BOOL_SYMBOL ||
            symbol == *STR_SYMBOL ||
            symbol == *VOID_SYMBOL ||
            symbol == *FLOAT_32_SYMBOL ||
            symbol == *FLOAT_64_SYMBOL
    }
}

/// Second pass. Visits the Ast from left to right, and resolves all top-level names
///
/// Non top-level names like `x` in `self.x` are validated during type checking,
/// because the type of `self` is needed to validate `x`
///
/// The second pass does the following
///
/// - Resolves all top level names, and reports any lexical errors
/// - Resolves static types like the type of a field in a struct or the return type of a function
#[derive(Debug)]
pub struct AstResolver<'ctx, 'ast, 'b, E> where E: ResolverHandle<'ctx, 'ast, AstPartlyResolved> {
    ast: Ast<'ast, AstPartlyResolved>,
    resolver_handle: &'b mut E,
    trait_impl_context: Option<TraitImplId>,
    lexical_context_to_parent_lexical_context: &'b FxHashMap<LexicalContext, LexicalContext>,
    marker: PhantomData<&'ctx ()>,
}

impl<'ctx, 'ast, 'b, E> AstResolver<'ctx, 'ast, 'b, E>
    where E: ResolverHandle<'ctx, 'ast, AstPartlyResolved>, 'ctx: 'ast, 'ast: 'b
{
    pub fn new(
        ast: Ast<'ast, AstPartlyResolved>,
        resolver_handle: &'b mut E,
        lexical_context_to_parent_lexical_context: &'b FxHashMap<LexicalContext, LexicalContext>
    ) -> Self {
        Self {
            ast,
            resolver_handle,
            trait_impl_context: None,
            lexical_context_to_parent_lexical_context,
            marker: PhantomData,
        }
    }

    pub fn visit(mut self) -> Ast<'ast, AstResolved> {
        self.visit_stmts(self.ast.main_scope.stmts);
        self.ast.next_state()
    }

    fn begin_impl_context(&mut self, trait_impl_id: TraitImplId) {
        self.trait_impl_context = Some(trait_impl_id);
    }

    fn end_impl_context(&mut self) {
        self.trait_impl_context = None;
    }

    fn traverse_pat_and_bind_def_ids_to_lexical_bindings(&mut self, pat: Pat<'ast>) {
        match pat {
            Pat::IdentPat(ident_node) => {
                let def_id = self.resolver_handle.get_def_id_from_node_id(ident_node.ast_node_id);
                self.resolver_handle.bind_def_id_to_lexical_binding(def_id, ResKind::Variable);
            }
            Pat::TupleStructPat(tuple_struct_pat) => {
                for field in tuple_struct_pat.fields.iter() {
                    self.traverse_pat_and_bind_def_ids_to_lexical_bindings(*field);
                }
            }
        }
    }

    fn type_from_typing(&mut self, typing: &Typing<'ast>, item_type: ItemType) -> Ty {
        type_from_typing(
            typing,
            self.trait_impl_context,
            self.resolver_handle,
            item_type,
            &self.lexical_context_to_parent_lexical_context
        )
    }
}

fn type_from_typing<'ctx, 'ast, 'b, E, T: AstState>(
    typing: &Typing<'ast>,
    trait_impl_context: Option<TraitImplId>,
    e: &'b mut E,
    item_type: ItemType,
    lexical_context_to_parent_lexical_context: &FxHashMap<LexicalContext, LexicalContext>
) -> Ty
    where E: ResolverHandle<'ctx, 'ast, T>
{
    match typing {
        Typing::SelfType => {
            if let Some(TraitImplId { implementor_def_id, .. }) = trait_impl_context {
                Ty::Adt(implementor_def_id)
            } else {
                panic!("Expected `Self` to be inside an `impl` block");
            }
        }
        Typing::VariadicArgs => Ty::VariadicArgs,
        Typing::Ident(ident_node) => {
            let lexeme_symbol = Symbol::from_node_id(ident_node.ast_node_id);
            match lexeme_symbol {
                t if t == *INT_SYMBOL => INT_32_TY,
                t if t == *UINT_SYMBOL => UINT_32_TY,
                t if t == *FLOAT_SYMBOL => FLOAT_64_TY,

                t if t == *INT8_SYMBOL => INT_8_TY,
                t if t == *INT16_SYMBOL => INT_16_TY,
                t if t == *INT32_SYMBOL => INT_32_TY,
                t if t == *INT64_SYMBOL => INT_64_TY,

                t if t == *UINT8_SYMBOL => UINT_8_TY,
                t if t == *UINT16_SYMBOL => UINT_16_TY,
                t if t == *UINT32_SYMBOL => UINT_32_TY,
                t if t == *UINT64_SYMBOL => UINT_64_TY,

                t if t == *BOOL_SYMBOL => BOOL_TY,
                t if t == *STR_SYMBOL => STR_TY,
                t if t == *VOID_SYMBOL => VOID_TY,

                t if t == *FLOAT_32_SYMBOL => FLOAT_32_TY,
                t if t == *FLOAT_64_SYMBOL => FLOAT_64_TY,

                str => {
                    if
                        let Some(def_id) = e.lookup_ident_declaration(
                            lexeme_symbol,
                            ResKind::Adt,
                            ident_node.ast_node_id,
                            lexical_context_to_parent_lexical_context
                        )
                    {
                        Ty::Adt(def_id)
                    } else {
                        panic!("Expected Algebric Data Type: {}", str.get());
                    }
                }
            }
        }
        Typing::Tuple(tuple) => {
            let mut tuple_ty = Vec::with_capacity(tuple.len());
            for typing in tuple.iter() {
                tuple_ty.push(
                    type_from_typing(
                        typing,
                        trait_impl_context,
                        e,
                        item_type,
                        lexical_context_to_parent_lexical_context
                    )
                );
            }
            Ty::Tuple(TyCtx::intern_many_types(tuple_ty))
        }
        Typing::Fn(args_typing, ret_typing) => {
            let args_ty = {
                let mut args_ty = Vec::with_capacity(args_typing.len());
                for typing in args_typing.iter() {
                    args_ty.push(
                        type_from_typing(
                            typing,
                            trait_impl_context,
                            e,
                            item_type,
                            lexical_context_to_parent_lexical_context
                        )
                    );
                }
                TyCtx::intern_many_types(args_ty)
            };
            let ret_ty = ret_typing.map(|typing|
                type_from_typing(
                    typing,
                    trait_impl_context,
                    e,
                    item_type,
                    lexical_context_to_parent_lexical_context
                )
            );
            Ty::FnSig(FnSig::new(args_ty, TyCtx::intern_type(ret_ty.unwrap_or(VOID_TY))))
        }
        Typing::Ptr(typing, mutability) => {
            if item_type == ItemType::Normal {
                todo!("Report error: Cannot use pointers in this context");
            }
            Ty::Ptr(
                TyCtx::intern_type(
                    type_from_typing(
                        typing,
                        trait_impl_context,
                        e,
                        item_type,
                        lexical_context_to_parent_lexical_context
                    )
                ),
                *mutability
            )
        }
        Typing::ManyPtr(typing) => {
            if item_type == ItemType::Normal {
                todo!("Report error: Cannot use pointers in this context");
            }

            Ty::ManyPtr(
                TyCtx::intern_type(
                    type_from_typing(
                        typing,
                        trait_impl_context,
                        e,
                        item_type,
                        lexical_context_to_parent_lexical_context
                    )
                ),
                Mutability::Immutable
            )
        }
    }
}

/// Visits the entire Ast from left to right
///
/// When the visitor is done, the ast will transition into the next state
#[derive(Debug)]
pub struct AstTypeChecker<'ctx, 'ast, 'b, E> where E: ResolverHandle<'ctx, 'ast, AstResolved> {
    pub ast: Ast<'ast, AstResolved>,
    pub resolver_handle: &'b mut E,
    marker: PhantomData<&'ctx ()>,
    lexical_context_to_parent_lexical_context: &'b FxHashMap<LexicalContext, LexicalContext>,

    /// First option checks if it's present (meain if we're inside a loop or function)
    /// Second one checks whether or not the ty is set
    loop_ret_ty: Option<Option<Ty>>,
    fn_ret_ty: Option<Ty>,
    trait_impl_context: Option<TraitImplId>,
}

impl<'ctx, 'ast, 'b, E> AstTypeChecker<'ctx, 'ast, 'b, E>
    where E: ResolverHandle<'ctx, 'ast, AstResolved>, 'ctx: 'ast, 'ast: 'b
{
    pub fn new(
        ast: Ast<'ast, AstResolved>,
        resolver_handle: &'b mut E,
        lexical_context_to_parent_lexical_context: &'b FxHashMap<LexicalContext, LexicalContext>
    ) -> Self {
        Self {
            ast,
            loop_ret_ty: None,
            fn_ret_ty: None,
            trait_impl_context: None,
            lexical_context_to_parent_lexical_context,
            resolver_handle,
            marker: PhantomData,
        }
    }
    pub fn visit(mut self) -> Ast<'ast, AstTypeChecked> {
        self.visit_stmts(self.ast.main_scope.stmts);
        self.ast.next_state()
    }

    pub fn traverse_pat_and_bind_def_ids_to_variable_namebinding(&mut self, pat: Pat<'ast>) {
        match pat {
            Pat::IdentPat(ident_node) => {
                let def_id = self.resolver_handle.get_def_id_from_node_id(ident_node.ast_node_id);
                let name_binding = NameBinding::new(
                    NameBindingKind::Variable(Mutability::Immutable)
                );
                self.resolver_handle.set_namebinding_to_def_id(def_id, name_binding);
            }
            // Pat::MutIdentPat(ident_node) => {
            //     let def_id = self.resolver_handle.get_def_id_from_node_id(ident_node.ast_node_id);
            //     let name_binding = NameBinding::new(
            //         NameBindingKind::Variable(Mutability::Mutable)
            //     );
            //     self.resolver_handle.set_namebinding_to_def_id(def_id, name_binding);
            // }
            Pat::TupleStructPat(tuple_struct_pat) => {
                self.visit_path(tuple_struct_pat.path);
                for field in tuple_struct_pat.fields.iter() {
                    self.traverse_pat_and_bind_def_ids_to_variable_namebinding(*field);
                }
            }
        }
    }
}

/// This can call functions on the Resolver struct in the resolver crate,
/// which also implements this trait
///
/// The reason it's not using the Resolver directly is to avoid cyclic references
pub trait ResolverHandle<'ctx, 'ast, T> where T: AstState {
    /* Methods available during all passes  */
    fn report_error(&mut self, error: Error);
    fn alloc_vec<K>(&self, vec: Vec<K>) -> &'ctx [K];
    fn borrow_def_id_to_name_binding(&self) -> &DefIdToNameBinding<'ctx>;
    /// Makes a new DefId and binds it to the given NodeId
    fn make_def_id_and_bind_to_node_id(&mut self, node_id: NodeId, symbol: Symbol) -> DefId;
    fn set_namebinding_to_def_id(&mut self, def_id: DefId, name_binding: NameBinding<'ctx>);
    fn set_def_id_to_node_id(&mut self, node_id: NodeId, def_id: DefId);
    fn lookup_ident_declaration(
        &mut self,
        /* 
        ident_node: &'ast IdentNode,
        kind: ResKind
        lexial_context_to_parent_lexical_context: &FxHashMap<LexicalContext, LexicalContext>
        */
        symbol: Symbol,
        res_kind: ResKind,
        node_id: NodeId,
        lexical_context_to_parent_lexical_context: &FxHashMap<LexicalContext, LexicalContext>
    ) -> Option<DefId>;
    fn lookup_ident_definition(
        &mut self,
        /* 
        ident_node: &'ast IdentNode,
        kind: ResKind
        lexial_context_to_parent_lexical_context: &FxHashMap<LexicalContext, LexicalContext>
        */
        symbol: Symbol,
        res_kind: ResKind,
        node_id: NodeId,
        lexical_context_to_parent_lexical_context: &FxHashMap<LexicalContext, LexicalContext>
    ) -> Option<(DefId, NameBinding<'ctx>)>;

    fn get_mut_or_create_def_ids_from_trait_impl_id(
        &mut self,
        trait_impl_id: TraitImplId
    ) -> &mut Vec<DefId>;

    fn bind_def_id_to_lexical_binding(&mut self, def_id: DefId, res_kind: ResKind);
    fn set_main_fn(&mut self, main_fn: &'ast FnItem<'ast>) -> bool;
    fn is_main_scope(&mut self) -> bool;
    fn append_fn(&mut self, fn_item: &'ast FnItem<'ast>);
    fn append_comp_decl(&mut self, comp_fn_decl: CompDeclItem<'ast>);
    fn set_def_id_to_global_mem(&mut self, def_id: DefId);
    fn get_ty_from_node_id(&self, node_id: NodeId) -> Ty;
    fn set_node_id_to_lexical_context(&mut self, node_id: NodeId, lexical_context: LexicalContext);
    /// Makes the string global, so all identical strings after this call, will be bound to the same DefId
    ///
    /// NOTICE: This may need some rework later, when I introduce mulitple packages (crates),
    /// since the Resolver only traverses one package,
    /// meaning that the same string may be bound to a different DefId in another package
    fn make_const_str(&mut self, str_expr: &'ast StringExpr) -> DefId;
    fn try_get_def_id_from_trait_impl_id(
        &self,
        trait_impl_id: TraitImplId,
        symbol: Symbol
    ) -> Option<DefId>;
    fn compile_rel_file(&mut self, file: Path<'ast>) -> Result<u32, String>;

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
    fn try_get_def_id_from_node_id(&self, node_id: NodeId) -> Option<DefId>;

    // fn set_namebinding_and_ty_to_def_id(&mut self, def_id: DefId, name_binding: NameBinding, ty: Ty);
    fn get_namebinding_from_def_id(&self, def_id: DefId) -> NameBinding<'ctx>;

    fn get_ty_from_def_id(&self, def_id: DefId) -> Ty;
}

impl<'ctx, 'ast, 'b, E> Visitor<'ast>
    for AstPreResolver<'ctx, 'ast, 'b, E>
    where 'ctx: 'ast, E: ResolverHandle<'ctx, 'ast, AstUnvalidated>, 'ast: 'b
{
    type Result = ();

    fn default_result() -> Self::Result {
        ()
    }

    fn visit_ident_expr(&mut self, ident_node: &'ast IdentNode) -> Self::Result {
        self.bind_node_id_to_lexical_context(ident_node.ast_node_id);
    }

    fn visit_struct_expr(&mut self, struct_expr: &'ast StructExpr<'ast>) -> Self::Result {
        self.visit_ident_expr(struct_expr.ident_node);

        for field in struct_expr.field_initializations.iter() {
            self.visit_expr(field.value);
        }
    }

    fn visit_field_expr(&mut self, field_expr: &'ast FieldExpr<'ast>) -> Self::Result {
        self.visit_expr(field_expr.lhs);
    }

    fn visit_def_stmt(&mut self, def_stmt: &'ast DefineStmt<'ast>) -> Self::Result {
        self.traverse_pat_and_bind_idents(def_stmt.setter_expr);
        self.visit_expr(def_stmt.value_expr);
    }

    fn visit_import_item(&mut self, import_item: &'ast ImportItem<'ast>) -> Self::Result {
        for path in import_item.import_items_path {
            self.traverse_path_and_bind_idents(*path);
        }
    }

    fn visit_comp_fn_decl_item(
        &mut self,
        comp_fn_decl_item: &'ast CompFnDeclItem<'ast>
    ) -> Self::Result {
        let def_id = self.resolver_handle.make_def_id_and_bind_to_node_id(
            comp_fn_decl_item.ident_node.ast_node_id,
            Symbol::from_node_id(comp_fn_decl_item.ident_node.ast_node_id)
        );

        self.bind_node_id_to_lexical_context(comp_fn_decl_item.ident_node.ast_node_id);
        self.resolver_handle.bind_def_id_to_lexical_binding(def_id, ResKind::Fn);

        self.visit_ident_expr(comp_fn_decl_item.ident_node);

        self.start_scope();
        for arg in comp_fn_decl_item.args.iter() {
            let def_id = self.resolver_handle.make_def_id_and_bind_to_node_id(
                arg.ident.ast_node_id,
                Symbol::from_node_id(arg.ident.ast_node_id)
            );
            self.bind_node_id_to_lexical_context(arg.ident.ast_node_id);
            self.visit_typing(&arg.type_expr);
        }
        self.end_scope();

        comp_fn_decl_item.return_ty.map(|x| self.visit_typing(&x));
    }

    fn visit_block_expr(&mut self, expr: &'ast BlockExpr<'ast>) -> Self::Result {
        self.start_scope();
        self.visit_stmts(expr.stmts);
        self.end_scope();
    }

    fn visit_if_expr(&mut self, if_expr: &'ast IfExpr<'ast>) -> Self::Result {
        match if_expr.cond_kind {
            CondKind::CondExpr(cond_expr) => {
                self.visit_expr(cond_expr);
                self.start_scope();
            }
            CondKind::CondPat(cond_pat, cond_expr) => {
                self.visit_expr(cond_expr);
                self.start_scope();
                self.traverse_pat_and_bind_idents(cond_pat);
            }
        }

        self.visit_stmts(if_expr.true_block);
        self.end_scope();

        if let Some(if_false_branch_expr) = if_expr.false_block {
            self.start_scope();
            self.visit_if_false_branch_expr(if_false_branch_expr);
            self.end_scope();
        }
    }

    fn visit_impl_item(&mut self, impl_item: &'ast ImplItem<'ast>) -> Self::Result {
        self.bind_node_id_to_lexical_context(impl_item.ident_node.ast_node_id);
        self.start_impl_context();
        for fn_item in impl_item.impl_fns.iter() {
            self.visit_fn_item(fn_item);
        }
        self.end_impl_context();
    }

    fn visit_typedef_item(&mut self, typedef_item: &'ast TypedefItem<'ast>) -> Self::Result {
        let typedef_name_symbol = Symbol::from_node_id(typedef_item.ident_node.ast_node_id);

        if self.is_compiler_adt_name(typedef_name_symbol) {
            panic!("Cannot shadow compiler ADT name: {}", typedef_name_symbol.get());
        }

        let def_id = self.resolver_handle.make_def_id_and_bind_to_node_id(
            typedef_item.ident_node.ast_node_id,
            typedef_name_symbol
        );

        self.bind_node_id_to_lexical_context(typedef_item.ident_node.ast_node_id);
        self.resolver_handle.bind_def_id_to_lexical_binding(def_id, ResKind::Adt);

        self.visit_typing(&typedef_item.type_expr);
    }

    fn visit_struct_item(&mut self, struct_item: &'ast StructItem<'ast>) -> Self::Result {
        let struct_name_symbol = Symbol::from_node_id(struct_item.ident_node.ast_node_id);

        if self.is_compiler_adt_name(struct_name_symbol) {
            panic!("Cannot shadow compiler ADT name: {}", struct_name_symbol.get());
        }

        let def_id = self.resolver_handle.make_def_id_and_bind_to_node_id(
            struct_item.ident_node.ast_node_id,
            struct_name_symbol
        );

        self.bind_node_id_to_lexical_context(struct_item.ident_node.ast_node_id);
        self.resolver_handle.bind_def_id_to_lexical_binding(def_id, ResKind::Adt);

        self.start_scope();

        for field in struct_item.field_declarations.iter() {
            self.resolver_handle.make_def_id_and_bind_to_node_id(
                field.ident.ast_node_id,
                Symbol::from_node_id(field.ident.ast_node_id)
            );
            // self.bind_node_id_to_lexical_context(field.ident.ast_node_id);
            self.visit_typing(&field.type_expr);
        }

        self.end_scope();
    }

    fn visit_enum_item(&mut self, enum_item: &'ast EnumItem<'ast>) -> Self::Result {
        let enum_name_symbol = Symbol::from_node_id(enum_item.ident_node.ast_node_id);

        if self.is_compiler_adt_name(enum_name_symbol) {
            panic!("Cannot shadow compiler ADT name: {}", enum_name_symbol.get());
        }

        let def_id = self.resolver_handle.make_def_id_and_bind_to_node_id(
            enum_item.ident_node.ast_node_id,
            enum_name_symbol
        );

        self.bind_node_id_to_lexical_context(enum_item.ident_node.ast_node_id);
        self.resolver_handle.bind_def_id_to_lexical_binding(def_id, ResKind::Adt);

        self.start_scope();

        for variant in enum_item.variants.iter() {
            self.resolver_handle.make_def_id_and_bind_to_node_id(
                variant.ident_node.ast_node_id,
                Symbol::from_node_id(variant.ident_node.ast_node_id)
            );
            // self.bind_node_id_to_lexical_context(variant.ident_node.ast_node_id);

            if let Some(typings) = &variant.enum_data {
                for typing in typings.iter() {
                    self.visit_typing(typing);
                }
            }
        }

        self.end_scope();
    }

    fn visit_fn_item(&mut self, fn_item: &'ast FnItem<'ast>) -> Self::Result {
        let def_id = self.resolver_handle.make_def_id_and_bind_to_node_id(
            fn_item.ident_node.ast_node_id,
            Symbol::from_node_id(fn_item.ident_node.ast_node_id)
        );

        self.bind_node_id_to_lexical_context(fn_item.ident_node.ast_node_id);
        self.resolver_handle.bind_def_id_to_lexical_binding(def_id, ResKind::Fn);

        fn_item.return_ty.map(|x| self.visit_typing(&x));

        self.start_context();

        for arg in fn_item.args.iter() {
            match arg {
                | ArgKind::MutPtrSelf(ident_node)
                | ArgKind::PtrSelf(ident_node)
                | ArgKind::MutSelf(ident_node)
                | ArgKind::NormalSelf(ident_node) => {
                    self.resolver_handle.make_def_id_and_bind_to_node_id(
                        ident_node.ast_node_id,
                        Symbol::from_node_id(ident_node.ast_node_id)
                    );
                    self.bind_node_id_to_lexical_context(ident_node.ast_node_id);
                }
                ArgKind::Arg(arg) => {
                    self.resolver_handle.make_def_id_and_bind_to_node_id(
                        arg.ident.ast_node_id,
                        Symbol::from_node_id(arg.ident.ast_node_id)
                    );
                    self.bind_node_id_to_lexical_context(arg.ident.ast_node_id);
                    self.visit_typing(&arg.type_expr);
                }
            }
        }

        self.is_in_impl = false;
        self.visit_stmts(fn_item.body);
        self.is_in_impl = true;

        self.end_context();
    }
}

impl<'ctx, 'ast, 'b, E> Visitor<'ast>
    for AstResolver<'ctx, 'ast, 'b, E>
    where 'ctx: 'ast, E: ResolverHandle<'ctx, 'ast, AstPartlyResolved>, 'ast: 'b
{
    type Result = ();

    fn default_result() -> Self::Result {
        ()
    }

    fn visit_ident_pat(&mut self, ident_node: &'ast IdentNode) -> Self::Result {
        self.visit_ident_expr(ident_node)
    }

    fn visit_tuple_struct_pat(&mut self, tuple_pat: &'ast TupleStructPat<'ast>) -> Self::Result {
        // Only resolve the lhs e.g. Option in Option.Some(5)
        self.visit_path(tuple_pat.path)
    }

    fn visit_path_field(&mut self, path_field: &'ast PathField<'ast>) -> Self::Result {
        // Only resolve the top-level
        self.visit_path(path_field.lhs)
    }

    fn visit_path_segment(&mut self, path_segment: &'ast IdentNode) -> Self::Result {
        self.visit_ident_expr(path_segment)
    }

    fn visit_field_expr(&mut self, field_expr: &'ast FieldExpr<'ast>) -> Self::Result {
        self.visit_expr(field_expr.lhs)
    }

    fn visit_struct_expr(&mut self, struct_expr: &'ast StructExpr<'ast>) -> Self::Result {
        self.visit_ident_expr(struct_expr.ident_node);

        for field in struct_expr.field_initializations.iter() {
            self.visit_expr(field.value);
        }

        Self::default_result()
    }

    fn visit_ident_expr(&mut self, ident_node: &'ast IdentNode) -> Self::Result {
        if
            let Some(def_id) = self.resolver_handle.lookup_ident_declaration(
                Symbol::from_node_id(ident_node.ast_node_id),
                ResKind::Variable,
                ident_node.ast_node_id,
                &self.lexical_context_to_parent_lexical_context
            )
        {
            self.resolver_handle.set_def_id_to_node_id(ident_node.ast_node_id, def_id);
        } else if
            let Some(def_id) = self.resolver_handle.lookup_ident_declaration(
                Symbol::from_node_id(ident_node.ast_node_id),
                ResKind::Fn,
                ident_node.ast_node_id,
                &self.lexical_context_to_parent_lexical_context
            )
        {
            self.resolver_handle.set_def_id_to_node_id(ident_node.ast_node_id, def_id);
        } else if
            let Some(def_id) = self.resolver_handle.lookup_ident_declaration(
                Symbol::from_node_id(ident_node.ast_node_id),
                ResKind::Adt,
                ident_node.ast_node_id,
                &self.lexical_context_to_parent_lexical_context
            )
        {
            self.resolver_handle.set_def_id_to_node_id(ident_node.ast_node_id, def_id);
        } else if
            let (Some(TraitImplId { implementor_def_id, .. }), true) = (
                self.trait_impl_context,
                Symbol::from_node_id(ident_node.ast_node_id) == *BIG_SELF_SYMBOL,
            )
        {
            self.resolver_handle.set_def_id_to_node_id(ident_node.ast_node_id, implementor_def_id);
        } else {
            let symbol = Symbol::from_node_id(ident_node.ast_node_id);
            self.resolver_handle.report_error(
                Error::new(ErrorKind::UndefinedLookup(symbol, ResKind::Variable), ident_node.span)
            );
        }
    }

    fn visit_def_stmt(&mut self, def_stmt: &'ast DefineStmt<'ast>) -> Self::Result {
        self.traverse_pat_and_bind_def_ids_to_lexical_bindings(def_stmt.setter_expr);

        self.visit_expr(def_stmt.value_expr);
    }

    fn visit_if_expr(&mut self, if_expr: &'ast IfExpr<'ast>) -> Self::Result {
        match if_expr.cond_kind {
            CondKind::CondExpr(cond_expr) => {
                self.visit_expr(cond_expr);
            }
            CondKind::CondPat(pat, rhs_expr) => {
                self.visit_expr(rhs_expr);
                self.traverse_pat_and_bind_def_ids_to_lexical_bindings(pat);
                self.visit_pat(pat);
            }
        }

        self.visit_stmts(if_expr.true_block);

        if let Some(if_false_branch_expr) = if_expr.false_block {
            self.visit_if_false_branch_expr(if_false_branch_expr);
        }
    }

    fn visit_import_item(&mut self, import_item: &'ast ImportItem<'ast>) -> Self::Result {
        println!("Imported file: {:#?}", import_item);

        if let Some(from_path) = import_item.from_path {
            todo!("Importing from other packages not supported yet");
        }
        // let result = self.resolver_handle.compile_rel_file(import_item.from_path);

        todo!();
    }

    fn visit_comp_fn_decl_item(
        &mut self,
        comp_fn_decl_item: &'ast CompFnDeclItem<'ast>
    ) -> Self::Result {
        let ret_ty = comp_fn_decl_item.return_ty
            .map(|ty| self.type_from_typing(&ty, ItemType::C))
            .unwrap_or(VOID_TY);
        let ret_ty = TyCtx::intern_type(ret_ty);

        let args_tys = comp_fn_decl_item.args
            .iter()
            .map(|arg| {
                let ty = self.type_from_typing(&arg.type_expr, ItemType::C);
                self.resolver_handle.set_type_to_node_id(arg.ident.ast_node_id, ty);
                ty
            })
            .collect::<Vec<_>>();
        let args_tys = TyCtx::intern_many_types(args_tys);

        let def_id = self.resolver_handle.get_def_id_from_node_id(
            comp_fn_decl_item.ident_node.ast_node_id
        );
        let name_binding = NameBinding::new(
            NameBindingKind::Fn(FnSig::new(args_tys, ret_ty), HasSelfArg::No, Externism::Clib)
        );
        self.resolver_handle.set_namebinding_to_def_id(def_id, name_binding);
        self.resolver_handle.set_type_to_node_id(
            comp_fn_decl_item.ident_node.ast_node_id,
            Ty::FnDef(def_id)
        );
        self.resolver_handle.set_type_to_node_id(comp_fn_decl_item.ast_node_id, VOID_TY);
    }

    fn visit_struct_item(&mut self, struct_item: &'ast StructItem<'ast>) -> Self::Result {
        let struct_data = struct_item.field_declarations
            .iter()
            .map(|field| {
                let def_id = self.resolver_handle.get_def_id_from_node_id(field.ident.ast_node_id);
                let ty = self.type_from_typing(&field.type_expr, struct_item.item_type);
                (def_id, ty)
            })
            .collect::<Vec<_>>();
        let struct_data = self.resolver_handle.alloc_vec(struct_data);

        let def_id = self.resolver_handle.get_def_id_from_node_id(
            struct_item.ident_node.ast_node_id
        );
        let name_binding = NameBinding::new(NameBindingKind::Adt(Adt::Struct(struct_data)));
        self.resolver_handle.set_namebinding_to_def_id(def_id, name_binding);
        self.resolver_handle.set_type_to_node_id(
            struct_item.ident_node.ast_node_id,
            Ty::Adt(def_id)
        );
        self.resolver_handle.set_type_to_node_id(struct_item.ast_node_id, VOID_TY);
    }

    fn visit_impl_item(&mut self, impl_item: &'ast ImplItem<'ast>) -> Self::Result {
        let implementor_id = self.resolver_handle
            .lookup_ident_declaration(
                Symbol::from_node_id(impl_item.ident_node.ast_node_id),
                ResKind::Adt,
                impl_item.ident_node.ast_node_id,
                &self.lexical_context_to_parent_lexical_context
            )
            .expect(
                format!(
                    "Cannot implement undefined type `{}`",
                    Symbol::from_node_id(impl_item.ident_node.ast_node_id).get()
                ).as_str()
            );

        let trait_impl_id = TraitImplId::new(implementor_id, None);

        self.resolver_handle.set_def_id_to_node_id(
            impl_item.ident_node.ast_node_id,
            implementor_id
        );

        self.begin_impl_context(trait_impl_id);

        for fn_item in impl_item.impl_fns.iter() {
            self.visit_fn_item(fn_item);
        }

        self.end_impl_context();

        self.resolver_handle.set_type_to_node_id(
            impl_item.ident_node.ast_node_id,
            Ty::Adt(implementor_id)
        );
        self.resolver_handle.set_type_to_node_id(impl_item.ast_node_id, VOID_TY);
    }

    fn visit_fn_item(&mut self, fn_item: &'ast FnItem<'ast>) -> Self::Result {
        let def_id = self.resolver_handle.get_def_id_from_node_id(fn_item.ident_node.ast_node_id);
        let implementor_def_id = if let Some(trait_impl_id) = self.trait_impl_context {
            self.resolver_handle
                .get_mut_or_create_def_ids_from_trait_impl_id(trait_impl_id)
                .push(def_id);
            Some(trait_impl_id.implementor_def_id)
        } else {
            None
        };

        let ret_ty = fn_item.return_ty
            .map(|ty_expr| self.type_from_typing(&ty_expr, fn_item.item_type))
            .unwrap_or(VOID_TY);

        let mut has_self_arg = HasSelfArg::No;
        let args_ty = fn_item.args
            .iter()
            .map(|arg_kind| {
                let arg_ty = match arg_kind {
                    // TODO: Check for multiple self args = error
                    ArgKind::Arg(field) => {
                        let ty = self.type_from_typing(&field.type_expr, fn_item.item_type);
                        ty
                    }
                    t => {
                        has_self_arg = HasSelfArg::Yes;
                        let implementor_def_id = if let Some(x) = implementor_def_id {
                            x
                        } else {
                            panic!("Expected `Self` to be inside an `impl` block");
                        };
                        match t {
                            ArgKind::MutPtrSelf(_) => Ty::Adt(implementor_def_id).to_mut_ptr_ty(),
                            ArgKind::MutSelf(_) => Ty::Adt(implementor_def_id).to_mut_ptr_ty(),
                            ArgKind::NormalSelf(_) => Ty::Adt(implementor_def_id).to_ptr_ty(),
                            ArgKind::PtrSelf(_) => Ty::Adt(implementor_def_id).to_ptr_ty(),
                            ArgKind::Arg(_) => unreachable!(),
                        }
                    }
                };

                let ident_node = get_ident_node_from_arg_kind(*arg_kind);
                self.resolver_handle.set_type_to_node_id(ident_node.ast_node_id, arg_ty);

                let def_id = self.resolver_handle.get_def_id_from_node_id(ident_node.ast_node_id);
                self.resolver_handle.set_namebinding_to_def_id(
                    def_id,
                    NameBinding::new(NameBindingKind::Variable(Mutability::Immutable))
                );
                self.resolver_handle.bind_def_id_to_lexical_binding(def_id, ResKind::Variable);
                arg_ty
            })
            .collect::<Vec<_>>();
        let args_ty = TyCtx::intern_many_types(args_ty);

        let fn_sig = FnSig::new(args_ty, TyCtx::intern_type(ret_ty));
        let name_binding = NameBinding::new(
            NameBindingKind::Fn(fn_sig, has_self_arg, Externism::NoExtern)
        );
        self.resolver_handle.set_namebinding_to_def_id(def_id, name_binding);
        self.resolver_handle.set_type_to_node_id(fn_item.ident_node.ast_node_id, Ty::FnDef(def_id));
        self.resolver_handle.set_type_to_node_id(fn_item.ast_node_id, VOID_TY);
        self.resolver_handle.set_def_id_to_global_mem(def_id);

        if def_id.symbol == *MAIN_SYMBOL && self.resolver_handle.is_main_scope() {
            if !self.resolver_handle.set_main_fn(fn_item) {
                panic!(
                    "Duplicate definitions of entry point `main` in global scope (report error)"
                );
            }
        } else {
            self.resolver_handle.append_fn(fn_item);
        }

        self.visit_stmts(fn_item.body);
    }

    fn visit_enum_item(&mut self, enum_item: &'ast EnumItem<'ast>) -> Self::Result {
        let def_id = self.resolver_handle.get_def_id_from_node_id(enum_item.ident_node.ast_node_id);

        let variant_def_ids = enum_item.variants
            .iter()
            .enumerate()
            .map(|(i, variant)| {
                let variant_def_id = self.resolver_handle.make_def_id_and_bind_to_node_id(
                    variant.ident_node.ast_node_id,
                    Symbol::from_node_id(variant.ident_node.ast_node_id)
                );
                let enum_data_ty = variant.enum_data
                    .map(|x|
                        x
                            .iter()
                            .map(|y| self.type_from_typing(y, enum_item.item_type))
                            .collect::<Vec<_>>()
                    )
                    .unwrap_or(vec![Ty::ZeroSized]);
                let enum_data_ty = TyCtx::intern_many_types(enum_data_ty);

                self.resolver_handle.set_type_to_node_id(
                    variant.ident_node.ast_node_id,
                    Ty::AtdConstructer(variant_def_id)
                );

                let name_binding = NameBinding::new(
                    NameBindingKind::Adt(
                        Adt::EnumVariant(def_id, EmumVaraintId(i as u32), enum_data_ty)
                    )
                );
                self.resolver_handle.set_namebinding_to_def_id(variant_def_id, name_binding);
                variant_def_id
            })
            .collect::<Vec<_>>();
        let variant_def_ids = self.resolver_handle.alloc_vec(variant_def_ids);

        let name_binding = NameBinding::new(NameBindingKind::Adt(Adt::Enum(variant_def_ids)));
        self.resolver_handle.set_namebinding_to_def_id(def_id, name_binding);
        self.resolver_handle.set_type_to_node_id(enum_item.ident_node.ast_node_id, Ty::Adt(def_id));
        self.resolver_handle.set_type_to_node_id(enum_item.ast_node_id, VOID_TY);
    }

    fn visit_string_expr(&mut self, string_expr: &'ast StringExpr) -> Self::Result {
        self.resolver_handle.make_const_str(string_expr);
    }

    fn visit_typedef_item(&mut self, typedef_item: &'ast TypedefItem<'ast>) -> Self::Result {
        let def_id = self.resolver_handle.get_def_id_from_node_id(
            typedef_item.ident_node.ast_node_id
        );
        let ty = self.type_from_typing(&typedef_item.type_expr, typedef_item.item_type);
        let name_binding = NameBinding::new(NameBindingKind::Adt(Adt::Typedef(ty)));
        self.resolver_handle.set_namebinding_to_def_id(def_id, name_binding);
        self.resolver_handle.set_type_to_node_id(
            typedef_item.ident_node.ast_node_id,
            Ty::Adt(def_id)
        );
        self.resolver_handle.set_type_to_node_id(typedef_item.ast_node_id, VOID_TY);
    }
}

/// Implements the Visitor trait for the second pass (type checking)
impl<'ctx, 'ast, 'b, E> Visitor<'ast>
    for AstTypeChecker<'ctx, 'ast, 'b, E>
    where 'ctx: 'ast, 'ast: 'b, E: ResolverHandle<'ctx, 'ast, AstResolved>
{
    type Result = Ty;

    fn default_result() -> Self::Result {
        VOID_TY
    }

    fn visit_interger_expr(&mut self, interger_expr: &'ast IntegerExpr) -> Self::Result {
        let ty = Ty::from_int(interger_expr.val);
        self.resolver_handle.set_type_to_node_id(interger_expr.ast_node_id, ty);
        ty
    }

    fn visit_bool_expr(&mut self, bool_expr: &'ast BoolExpr) -> Self::Result {
        self.resolver_handle.set_type_to_node_id(bool_expr.ast_node_id, BOOL_TY);
        BOOL_TY
    }

    fn visit_null_expr(&mut self, null_expr: &'ast NullExpr) -> Self::Result {
        self.resolver_handle.set_type_to_node_id(null_expr.ast_node_id, NULL_TY);
        NULL_TY
    }

    fn visit_string_expr(&mut self, string_expr: &'ast StringExpr) -> Self::Result {
        self.resolver_handle.set_type_to_node_id(string_expr.ast_node_id, STR_TY);
        STR_TY
    }

    fn visit_block_expr(&mut self, expr: &'ast BlockExpr<'ast>) -> Self::Result {
        let block_type = self.visit_stmts(expr.stmts);
        self.resolver_handle.set_type_to_node_id(expr.ast_node_id, block_type);
        block_type
    }

    fn visit_fn_item(&mut self, fn_item: &'ast FnItem<'ast>) -> Self::Result {
        let def_id = self.resolver_handle.get_def_id_from_node_id(fn_item.ident_node.ast_node_id);
        let name_binding = self.resolver_handle.get_namebinding_from_def_id(def_id);
        let fn_sig = match name_binding.kind {
            NameBindingKind::Fn(fn_sig, _, _) => fn_sig,
            _ => unreachable!(),
        };

        self.fn_ret_ty = Some(*fn_sig.ret_ty);

        self.visit_stmts(fn_item.body);

        self.fn_ret_ty = None;

        Self::default_result()
    }

    fn visit_ident_expr(&mut self, ident_node: &'ast IdentNode) -> Self::Result {
        let def_id = self.resolver_handle.try_get_def_id_from_node_id(ident_node.ast_node_id);

        if let Some(def_id) = def_id {
            let name_binding = self.resolver_handle.get_namebinding_from_def_id(def_id);
            match name_binding.kind {
                NameBindingKind::Variable(mutability) => {
                    let ty = Ty::StackPtr(
                        TyCtx::intern_type(self.resolver_handle.get_ty_from_def_id(def_id)),
                        mutability
                    );
                    self.resolver_handle.set_type_to_node_id(ident_node.ast_node_id, ty);
                    ty
                }
                NameBindingKind::Fn(_, _, _) => {
                    let ty = Ty::FnDef(def_id);
                    self.resolver_handle.set_type_to_node_id(ident_node.ast_node_id, ty);
                    ty
                }
                | NameBindingKind::Adt(Adt::Enum(_))
                | NameBindingKind::Adt(Adt::Struct(_))
                | NameBindingKind::Adt(Adt::Typedef(_)) => {
                    let ty = Ty::AtdConstructer(def_id);
                    self.resolver_handle.set_type_to_node_id(ident_node.ast_node_id, ty);
                    ty
                }
                NameBindingKind::Adt(Adt::EnumVariant(_, _, _)) =>
                    panic!(
                        "Allow this if enum variants when enum variants can be used as top level names"
                    ),
                NameBindingKind::ConstStr(_) => unreachable!("Const strings should not be here"),
            }
        } else {
            self.resolver_handle.set_type_to_node_id(ident_node.ast_node_id, UNKOWN_TY);
            UNKOWN_TY
        }
    }

    fn visit_call_expr(&mut self, call_expr: &'ast CallExpr<'ast>) -> Self::Result {
        let calle_ty = self.visit_expr(call_expr.callee);

        if let Ty::AtdConstructer(enum_variant_def_id) = calle_ty {
            let name_binding =
                self.resolver_handle.get_namebinding_from_def_id(enum_variant_def_id);

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
                        self.resolver_handle.borrow_def_id_to_name_binding()
                    );

                    if let Err(errors) = is_valid_arg {
                        for error in errors {
                            println!("{:?}", error);
                        }

                        todo!("Report error");
                    }
                }
            }

            self.resolver_handle.set_type_to_node_id(
                call_expr.ast_node_id,
                Ty::Adt(enum_variant_def_id)
            );

            return Ty::Adt(enum_def_id);
        }

        let (fn_sig, has_self_arg) = match calle_ty.auto_deref() {
            Ty::FnDef(def_id) => {
                if
                    let NameBindingKind::Fn(fn_sig, has_self_arg, _) =
                        self.resolver_handle.get_namebinding_from_def_id(def_id).kind
                {
                    (fn_sig, has_self_arg)
                } else {
                    panic!("Expected function");
                }
            }
            Ty::FnSig(fn_sig) => (fn_sig, HasSelfArg::No),
            _ => {
                // self.resolver_handle.report_error(
                //     Error::new(ErrorKind::NotCallable, call_expr.callee.span)
                // );
                println!("Not callable");
                return Ty::Unkown;
            }
        };

        let ret_ty = *fn_sig.ret_ty;

        let fn_args_count: usize = fn_sig.args
            .iter()
            .enumerate()
            .map_while(|(i, arg)| {
                if i == 0 && has_self_arg == HasSelfArg::Yes {
                    Some(0)
                } else {
                    if arg == &Ty::VariadicArgs { None } else { Some(1) }
                }
            })
            .sum();

        if call_expr.args.len() < fn_args_count {
            // self.resolver_handle.report_error(
            //     Error::new(ErrorKind::MissingArg, call_expr.span)
            // );
            panic!("Missing arg");
        }

        let mut found_variadic = false;
        for (i, arg) in call_expr.args.iter().enumerate() {
            let i = if has_self_arg == HasSelfArg::Yes { i + 1 } else { i };

            let given_arg_ty = self.visit_expr(*arg);
            if found_variadic {
                continue;
            }

            let arg_ty = if let Some(arg) = fn_sig.args.get(i) {
                arg
            } else {
                // self.resolver_handle.report_error(
                //     Error::new(ErrorKind::MissingArg, call_expr.span)
                // );
                panic!("Missing arg (or too many args)");
                break;
            };

            if *arg_ty == Ty::VariadicArgs {
                found_variadic = true;
                continue;
            }

            let arg_cmp = ArgCmp {
                arg_ty: *arg_ty,
                provided_ty: given_arg_ty,
            };

            let is_valid_arg = TypeChecker::test_valid_arg(
                arg_cmp,
                self.resolver_handle.borrow_def_id_to_name_binding()
            );

            if let Err(errors) = is_valid_arg {
                println!("arg_ty: {:?}, given_arg_ty: {:?}", arg_ty, given_arg_ty);

                for error in errors {
                    println!("{:?}", error);
                }

                todo!("Report error");
            }
        }

        self.resolver_handle.set_type_to_node_id(call_expr.ast_node_id, ret_ty);
        ret_ty
    }

    fn visit_return_expr(&mut self, return_expr: &'ast ReturnExpr) -> Self::Result {
        let ret_ty = if let Some(expr) = return_expr.value {
            self.visit_expr(expr)
        } else {
            VOID_TY
        };

        if let Some(fn_ret_ty) = self.fn_ret_ty {
            self.resolver_handle.set_type_to_node_id(return_expr.ast_node_id, fn_ret_ty);
            if
                let Err(errors) = TypeChecker::test_eq_loose(
                    ret_ty,
                    fn_ret_ty,
                    self.resolver_handle.borrow_def_id_to_name_binding()
                )
            {
                for error in errors {
                    println!("{:?}", error);
                }

                self.resolver_handle.report_error(
                    Error::new(ErrorKind::MismatchedReturnTypes(fn_ret_ty, ret_ty), Span::dummy())
                );
            }
        } else {
            self.resolver_handle.report_error(
                Error::new(ErrorKind::ReturnOutsideFn, Span::dummy())
            );
        }

        NEVER_TY
    }

    fn visit_struct_expr(&mut self, struct_expr: &'ast StructExpr<'ast>) -> Self::Result {
        let lhs_ty = self.visit_ident_expr(struct_expr.ident_node);

        let atd_constructer_def_id = match lhs_ty {
            Ty::AtdConstructer(def_id) => def_id,
            _ => {
                println!("Expected struct 1");
                self.resolver_handle.report_error(
                    Error::new(ErrorKind::InvalidStruct(lhs_ty), Span::dummy())
                );
                self.resolver_handle.set_type_to_node_id(struct_expr.ast_node_id, Ty::Unkown);
                return Ty::Unkown;
            }
        };

        let name_binding = self.resolver_handle.get_namebinding_from_def_id(atd_constructer_def_id);

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
                field_name.symbol !=
                Symbol::from_node_id(struct_expr.field_initializations[i].ident.ast_node_id)
            {
                self.resolver_handle.report_error(
                    Error::new(ErrorKind::MissingStructField(field_name.symbol), Span::dummy())
                );
                self.resolver_handle.report_error(
                    Error::new(
                        ErrorKind::UndefinedStructField(
                            atd_constructer_def_id.symbol,
                            Symbol::from_node_id(
                                struct_expr.field_initializations[i].ident.ast_node_id
                            )
                        ),
                        Span::dummy()
                    )
                );
            }

            if
                let Err(errors) = TypeChecker::test_eq_loose(
                    *given_ty,
                    ty,
                    self.resolver_handle.borrow_def_id_to_name_binding()
                )
            {
                self.resolver_handle.report_error(
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

            self.resolver_handle.set_type_to_node_id(
                struct_expr.field_initializations[i].ident.ast_node_id,
                *given_ty
            );
        }
        let struct_ty = Ty::Adt(atd_constructer_def_id);
        self.resolver_handle.set_type_to_node_id(struct_expr.ident_node.ast_node_id, struct_ty);
        self.resolver_handle.set_type_to_node_id(struct_expr.ast_node_id, struct_ty);
        struct_ty
    }

    fn visit_field_expr(&mut self, field_expr: &'ast FieldExpr<'ast>) -> Self::Result {
        let lhs_ty = self.visit_expr(field_expr.lhs);

        if let Ty::AtdConstructer(def_id) = lhs_ty {
            let rhs_symbol = Symbol::from_node_id(field_expr.rhs.ast_node_id);

            // This must be an enum, because an enum's constructor requires a field expression e.g. Option.Some(5)
            let name_binding = self.resolver_handle.get_namebinding_from_def_id(def_id);
            if let NameBindingKind::Adt(Adt::Enum(enum_variants)) = name_binding.kind {
                let variant_def_id = enum_variants
                    .iter()
                    .find(|x| x.symbol.get() == rhs_symbol.get());

                if let Some(variant_def_id) = variant_def_id {
                    let variant_data = self.resolver_handle.get_namebinding_from_def_id(
                        *variant_def_id
                    );
                    let enum_fields = match variant_data.kind {
                        NameBindingKind::Adt(Adt::EnumVariant(_, _, enum_fields)) => enum_fields,
                        _ => panic!("Expected enum variant"),
                    };
                    let field_ty = if enum_fields[0] == Ty::ZeroSized {
                        Ty::Adt(*variant_def_id)
                    } else {
                        Ty::AtdConstructer(*variant_def_id)
                    };

                    self.resolver_handle.set_def_id_to_node_id(
                        field_expr.rhs.ast_node_id,
                        *variant_def_id
                    );
                    self.resolver_handle.set_type_to_node_id(field_expr.rhs.ast_node_id, field_ty);
                    self.resolver_handle.set_type_to_node_id(field_expr.ast_node_id, field_ty);

                    return field_ty;
                } else {
                    todo!("Undefined variant: {}", rhs_symbol.get());
                }
            } else if let NameBindingKind::Adt(Adt::Struct(struct_fields)) = name_binding.kind {
                let impl_def_id = self.resolver_handle
                    .try_get_def_id_from_trait_impl_id(TraitImplId::new(def_id, None), rhs_symbol)
                    .expect("Expected impl method");
                let ty = Ty::FnDef(impl_def_id);

                self.resolver_handle.set_type_to_node_id(field_expr.rhs.ast_node_id, ty);
                self.resolver_handle.set_type_to_node_id(field_expr.ast_node_id, ty);
                self.resolver_handle.set_def_id_to_node_id(field_expr.rhs.ast_node_id, impl_def_id);

                return ty;
            } else {
                panic!("Expected enum");
            }
        } else {
            // As of now if we are here, we should expect a struct field (tuples have their own field expression) or an impl method
            let (adt_def_id, adt) = {
                let adt = lhs_ty.try_deref_as_adt(
                    self.resolver_handle.borrow_def_id_to_name_binding()
                );

                match adt {
                    Some(adt) => adt,
                    None => {
                        println!("Expected struct 2");
                        self.resolver_handle.report_error(
                            Error::new(ErrorKind::InvalidStruct(lhs_ty), Span::dummy())
                        );

                        self.resolver_handle.set_type_to_node_id(
                            field_expr.rhs.ast_node_id,
                            Ty::Unkown
                        );
                        self.resolver_handle.set_type_to_node_id(
                            field_expr.ast_node_id,
                            Ty::Unkown
                        );

                        return Ty::Unkown;
                    }
                }
            };

            let field_access_symbol = Symbol::from_node_id(field_expr.rhs.ast_node_id);

            let struct_fields = match adt {
                Adt::Struct(struct_fields) => Some(struct_fields),
                _ => None,
            };

            if let Some(struct_fields) = struct_fields {
                // If we are here, we should expect a struct field

                for (field_def_id, field_ty) in struct_fields {
                    if field_def_id.symbol.get() == field_access_symbol.get() {
                        let mutability = if lhs_ty.deref_until_stack_ptr().is_mut_ptr() {
                            Mutability::Mutable
                        } else {
                            Mutability::Immutable
                        };

                        let field_ty = Ty::StackPtr(TyCtx::intern_type(*field_ty), mutability);

                        self.resolver_handle.set_type_to_node_id(field_expr.ast_node_id, field_ty);
                        self.resolver_handle.set_type_to_node_id(
                            field_expr.rhs.ast_node_id,
                            field_ty
                        );
                        // self.resolver_handle.set_def_id_to_node_id(
                        //     field_expr.rhs.ast_node_id,
                        //     *field_def_id
                        // );

                        return field_ty;
                    }
                }
                // If we are here, we should expect an impl method on the struct
                let trait_impl_id = TraitImplId::new(adt_def_id, None);

                if
                    let Some(impl_def_id) = self.resolver_handle.try_get_def_id_from_trait_impl_id(
                        trait_impl_id,
                        field_access_symbol
                    )
                {
                    let (fn_sig, has_self_arg) = {
                        let name_binding =
                            self.resolver_handle.get_namebinding_from_def_id(impl_def_id);
                        match name_binding.kind {
                            NameBindingKind::Fn(fn_sig, has_self_arg, _) => (fn_sig, has_self_arg),
                            _ => panic!("Expected function"),
                        }
                    };

                    let fn_ty = Ty::FnDef(impl_def_id);
                    self.resolver_handle.set_type_to_node_id(field_expr.ast_node_id, fn_ty);
                    self.resolver_handle.set_type_to_node_id(field_expr.rhs.ast_node_id, fn_ty);
                    self.resolver_handle.set_def_id_to_node_id(
                        field_expr.rhs.ast_node_id,
                        impl_def_id
                    );

                    if has_self_arg == HasSelfArg::Yes {
                        let arg_cmp = ArgCmp {
                            arg_ty: fn_sig.args[0],
                            provided_ty: lhs_ty,
                        };

                        if
                            let Err(errors) = TypeChecker::test_valid_arg(
                                arg_cmp,
                                self.resolver_handle.borrow_def_id_to_name_binding()
                            )
                        {
                            for error in errors {
                                println!("{:?}", error);
                            }

                            todo!("Report error");
                        }
                    }

                    return fn_ty;
                };
            } else {
                // If we are here, we should expect an impl method
                todo!();
            }

            self.resolver_handle.report_error(
                Error::new(
                    ErrorKind::UndefinedStructField(adt_def_id.symbol, field_access_symbol),
                    Span::dummy()
                )
            );
            self.resolver_handle.set_type_to_node_id(field_expr.ast_node_id, Ty::Unkown);
            self.resolver_handle.set_type_to_node_id(field_expr.rhs.ast_node_id, Ty::Unkown);

            Ty::Unkown
        }
    }

    fn visit_tuple_struct_pat(&mut self, tuple_pat: &'ast TupleStructPat<'ast>) -> Self::Result {
        let ty = self.visit_path(tuple_pat.path);

        let (def_id, adt) = match ty {
            Ty::AtdConstructer(def_id) => {
                let name_binding = self.resolver_handle.get_namebinding_from_def_id(def_id);
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
                self.resolver_handle.set_type_to_node_id(tuple_pat.ast_node_id, ty);
                self.resolver_handle.set_def_id_to_node_id(tuple_pat.ast_node_id, def_id);

                if enum_ty.len() != tuple_pat.fields.len() {
                    todo!("Expected {} fields, got {}", enum_ty.len(), tuple_pat.fields.len());
                }

                for (i, pat) in tuple_pat.fields.iter().enumerate() {
                    let field_ty = enum_ty[i];

                    if let Pat::IdentPat(ident_pat) = pat {
                        let def_id = self.resolver_handle.get_def_id_from_node_id(
                            ident_pat.ast_node_id
                        );
                        self.resolver_handle.set_namebinding_to_def_id(
                            def_id,
                            NameBinding::new(NameBindingKind::Variable(Mutability::Immutable))
                        );
                        self.resolver_handle.set_type_to_node_id(ident_pat.ast_node_id, field_ty);
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
        let rhs_symbol = Symbol::from_node_id(path_field.rhs.ast_node_id);

        let def_id = match lhs_ty {
            Ty::AtdConstructer(def_id) => def_id,
            _ => panic!("Invalid lhs of path field"),
        };

        let name_binding = self.resolver_handle.get_namebinding_from_def_id(def_id);

        match name_binding.kind {
            NameBindingKind::Adt(Adt::Struct(fields)) => {
                todo!("Lookup constructer method for struct");
            }
            NameBindingKind::Adt(Adt::Enum(variants)) => {
                let variant_def_id = variants.iter().find(|x| x.symbol.get() == rhs_symbol.get());

                if let Some(variant_def_id) = variant_def_id {
                    let atd_constructor_ty = Ty::AtdConstructer(*variant_def_id);
                    self.resolver_handle.set_type_to_node_id(
                        path_field.rhs.ast_node_id,
                        atd_constructor_ty
                    );
                    self.resolver_handle.set_type_to_node_id(
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
        self.resolver_handle.set_type_to_node_id(path_segment.ast_node_id, ty);
        ty
    }

    fn visit_tuple_field_expr(
        &mut self,
        tuple_field_expr: &'ast TupleFieldExpr<'ast>
    ) -> Self::Result {
        let lhs_ty = self.visit_expr(tuple_field_expr.lhs);
        self.visit_interger_expr(tuple_field_expr.rhs);

        let tuple_ty = match
            lhs_ty.try_deref_as_tuple(self.resolver_handle.borrow_def_id_to_name_binding())
        {
            Some(tuple_ty) => tuple_ty,
            None => {
                self.resolver_handle.report_error(
                    Error::new(ErrorKind::InvalidTuple(lhs_ty), Span::dummy())
                );
                self.resolver_handle.set_type_to_node_id(tuple_field_expr.ast_node_id, Ty::Unkown);
                return Ty::Unkown;
            }
        };

        if tuple_field_expr.rhs.val > ((tuple_ty.len() - 1) as i64) {
            self.resolver_handle.report_error(
                Error::new(
                    ErrorKind::TupleAccessOutOfBounds(tuple_ty, tuple_field_expr.rhs.val as usize),
                    Span::dummy()
                )
            );
            self.resolver_handle.set_type_to_node_id(tuple_field_expr.ast_node_id, Ty::Unkown);
            return Ty::Unkown;
        } else {
            let mutability = if lhs_ty.deref_until_stack_ptr().is_mut_ptr() {
                Mutability::Mutable
            } else {
                Mutability::Immutable
            };

            let access_ty = Ty::StackPtr(
                TyCtx::intern_type(tuple_ty[tuple_field_expr.rhs.val as usize]),
                mutability
            );

            self.resolver_handle.set_type_to_node_id(tuple_field_expr.ast_node_id, access_ty);

            access_ty
        }
    }

    fn visit_index_expr(&mut self, index_expr: &'ast IndexExpr<'ast>) -> Self::Result {
        let lhs_ty = self.visit_expr(index_expr.lhs);

        let is_mutable = lhs_ty.is_mut_ptr();

        match
            self
                .visit_expr(index_expr.value_expr)
                .get_expanded_dereffed_ty(self.resolver_handle.borrow_def_id_to_name_binding())
        {
            INT_TY => {}
            ty => {
                todo!("Expected integer, got {}", ty);
            }
        }

        let result_ty = match lhs_ty.deref_until_stack_ptr().try_deref_once() {
            Some(Ty::ManyPtr(inner_ty, _)) => {
                let mutability = if is_mutable {
                    Mutability::Mutable
                } else {
                    Mutability::Immutable
                };

                Ty::StackPtr(inner_ty, mutability)
            }
            _ => {
                let full_lhs_ty = lhs_ty.get_expanded_dereffed_ty(
                    self.resolver_handle.borrow_def_id_to_name_binding()
                );
                todo!("Expected many-item-pointer, got {}", full_lhs_ty);
            }
        };

        self.resolver_handle.set_type_to_node_id(index_expr.ast_node_id, result_ty);

        result_ty
    }

    fn visit_assign_stmt(&mut self, assign_stmt: &'ast AssignStmt<'ast>) -> Self::Result {
        let setter_ty = self.visit_asignee_expr(assign_stmt.setter_expr);
        let value_ty = self.visit_expr(assign_stmt.value_expr);

        if !setter_ty.is_mut_ptr() && !setter_ty.deref_until_stack_ptr().is_mut_ptr() {
            // self.resolver_handle.report_error(
            //     Error::new(ErrorKind::AssignmentToImmutable(symbol), assign_stmt.span)
            // );
            println!("Assignment to immutable, expected mutable: {:?}", assign_stmt.setter_expr);
            todo!("Assignment to immutable, expected mutable: {}", setter_ty);
        }

        if
            let Err(_errors) = TypeChecker::test_eq_loose(
                setter_ty,
                value_ty,
                self.resolver_handle.borrow_def_id_to_name_binding()
            )
        {
            panic!("Not same type in assignment: {}, {}", setter_ty, value_ty);
        } else {
            self.resolver_handle.set_type_to_node_id(assign_stmt.ast_node_id, VOID_TY);
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

        self.resolver_handle.set_type_to_node_id(tuple_expr.ast_node_id, tuple_ty);

        tuple_ty
    }

    fn visit_def_stmt(&mut self, def_stmt: &'ast DefineStmt<'ast>) -> Self::Result {
        // When tuple patterns are implemented, compare if tuple on lhs, is same as tuple type on rhs

        match &def_stmt.setter_expr {
            Pat::IdentPat(ident_pat) => {
                let mutability = if def_stmt.mut_span.get().is_some() {
                    Mutability::Mutable
                } else {
                    Mutability::Immutable
                };
                let def_id = self.resolver_handle.get_def_id_from_node_id(ident_pat.ast_node_id);
                self.resolver_handle.set_namebinding_to_def_id(
                    def_id,
                    NameBinding::new(NameBindingKind::Variable(mutability))
                );

                let value_type = self.visit_expr(def_stmt.value_expr);

                self.resolver_handle.set_type_to_node_id(ident_pat.ast_node_id, value_type);
            }
            Pat::TupleStructPat(_) => todo!("Tuple patterns"),
        }

        // Even though def stmts doesn't return a value,
        // it still sets the type of the def stmt for ease of use in the prettifier
        //
        // It will, however, not have any effect on the actual program,
        // since below it returns void
        self.resolver_handle.set_type_to_node_id(def_stmt.ast_node_id, VOID_TY);

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

        self.resolver_handle.set_type_to_node_id(loop_expr.ast_node_id, ty);
        ty
    }

    fn visit_continue_expr(&mut self, continue_expr: &'ast ContinueExpr) -> Self::Result {
        self.resolver_handle.set_type_to_node_id(continue_expr.ast_node_id, VOID_TY);
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
                        self.resolver_handle.borrow_def_id_to_name_binding()
                    )
                {
                    self.resolver_handle.report_error(
                        Error::new(ErrorKind::BreakTypeError(*expected_ty, break_ty), Span::dummy())
                    );
                }
            } else {
                *loop_ret_ty = Some(break_ty);
            }
        } else {
            self.resolver_handle.report_error(
                Error::new(ErrorKind::BreakOutsideLoop, Span::dummy())
            );
        }

        self.resolver_handle.set_type_to_node_id(break_expr.ast_node_id, break_ty);

        Self::default_result()
    }

    fn visit_if_expr(&mut self, if_expr: &'ast IfExpr<'ast>) -> Self::Result {
        match if_expr.cond_kind {
            CondKind::CondExpr(cond_expr) => {
                let cond_type = self.visit_expr(cond_expr);
                if !cond_type.can_be_dereffed_to_bool() {
                    self.resolver_handle.report_error(
                        Error::new(ErrorKind::ExpectedBoolExpr(cond_type), if_expr.span)
                    );
                }
            }
            CondKind::CondPat(pat, rhs_expr) => {
                self.visit_expr(rhs_expr);
                self.visit_pat(pat);
            }
        }

        let true_type = self.visit_stmts(if_expr.true_block);

        let false_type = if_expr.false_block
            .as_ref()
            .map(|expr| self.visit_if_false_branch_expr(*expr));

        let if_expr_ty = if let Some(false_type) = false_type {
            if
                let Err(errors) = TypeChecker::test_eq_loose(
                    true_type,
                    false_type,
                    self.resolver_handle.borrow_def_id_to_name_binding()
                )
            {
                for error in errors {
                    println!("{:?}", error);
                }
                panic!("Expected equal types in if expr: {} != {}", true_type, false_type);
            } else {
                if true_type.is_num_ty() && false_type.is_num_ty() {
                    Ty::get_biggest_num_ty(true_type, false_type)
                        .expect("Expected number")
                        .auto_deref()
                } else {
                    true_type
                }
            }
        } else {
            true_type
        };

        self.resolver_handle.set_type_to_node_id(if_expr.ast_node_id, if_expr_ty);

        // Returns a pointer to its type
        if_expr_ty
    }

    fn visit_binary_expr(&mut self, binary_expr: &'ast BinaryExpr<'ast>) -> Self::Result {
        let lhs_type = self.visit_expr(binary_expr.lhs);
        let rhs_type = self.visit_expr(binary_expr.rhs);

        let biggest_num_ty = Ty::get_biggest_num_ty(lhs_type, rhs_type);

        let result_ty = lhs_type.test_binary(
            rhs_type,
            binary_expr.op,
            self.resolver_handle.borrow_def_id_to_name_binding()
        );

        if let Some(result_ty) = result_ty {
            self.resolver_handle.set_type_to_node_id(binary_expr.ast_node_id, result_ty);

            result_ty
        } else {
            self.resolver_handle.report_error(
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

        self.resolver_handle.set_type_to_node_id(group_expr.ast_node_id, expr_type);

        expr_type
    }
}
