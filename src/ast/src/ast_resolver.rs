use diagnostics::{ report_diagnostics, Diagnostic, ErrorKind };
use fxhash::{ FxBuildHasher, FxHashMap };
use ir::{
    Adt,
    ContextId,
    DefId,
    DefIdToNameBinding,
    EmumVaraintId,
    Externism,
    FnSig,
    HasSelfArg,
    LexicalBinding,
    LexicalContext,
    Mutability,
    NameBinding,
    NameBindingKind,
    NodeId,
    ResKind,
    ScopeId,
    Symbol,
    TraitImplId,
    Ty,
    TyCtx,
    BIG_SELF_SYMBOL,
    BOOL_SYMBOL,
    BOOL_TY,
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
    VOID_SYMBOL,
    VOID_TY,
};
use span::Span;

use crate::{
    ast_pre_resolver::{ self },
    get_ident_node_from_arg_kind,
    ArgKind,
    Ast,
    AstPartlyResolved,
    AstState,
    CompFnDeclItem,
    CondKind,
    DefineStmt,
    EnumItem,
    Expr,
    ExprWithoutBlock,
    FieldExpr,
    FnItem,
    IdentNode,
    IfExpr,
    ImplItem,
    ImportItem,
    ItemType,
    Pat,
    Path,
    PathField,
    PlaceExpr,
    ResolverHandle,
    StructExpr,
    StructItem,
    TupleStructPat,
    TypedefItem,
    Typing,
    VisitAst,
    Visitor,
};

#[derive(Debug)]
pub struct GlobalVisitResult<'ctx, 'ast> {
    pub fns: Vec<&'ast FnItem<'ast>>,
    pub clib_fns: Vec<DefId>,
    pub pkg_def_id_to_name_binding: FxHashMap<DefId, NameBinding<'ctx>>,
    pub trait_impl_id_to_def_ids: FxHashMap<TraitImplId, Vec<DefId>>,
}

#[derive(Debug)]
pub struct LocalVisitResult<'ctx> {
    pub node_id_to_def_id: FxHashMap<NodeId, DefId>,
    pub def_id_to_name_binding: DefIdToNameBinding<'ctx>,
    pub node_id_to_ty: FxHashMap<NodeId, Ty>,
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
pub struct AstResolver<'ctx, 'ast, 'b, E>
    where 'ctx: 'ast, 'ast: 'b, E: ResolverHandle<'ctx, 'ast, AstPartlyResolved> {
    ast: Ast<'ast, AstPartlyResolved>,
    local_visit_result: ast_pre_resolver::LocalVisitResult,
    resolver_handle: &'b E,
    trait_impl_context: Option<TraitImplId>,
    trait_impl_id_to_def_ids: FxHashMap<TraitImplId, Vec<DefId>>,
    def_id_to_name_binding: FxHashMap<DefId, NameBinding<'ctx>>,
    pkg_def_id_to_name_binding: FxHashMap<DefId, NameBinding<'ctx>>,
    fns: Vec<&'ast FnItem<'ast>>,
    clib_fns: Vec<DefId>,
    node_id_to_type: FxHashMap<NodeId, Ty>,

    diagnostics: Vec<Diagnostic>,
}

impl<'ctx, 'ast, 'b, E> VisitAst<'ast, AstPartlyResolved>
    for AstResolver<'ctx, 'ast, 'b, E>
    where 'ctx: 'ast, 'ast: 'b, E: ResolverHandle<'ctx, 'ast, AstPartlyResolved>
{
    type GlobalVisitResult = GlobalVisitResult<'ctx, 'ast>;
    type LocalVisitResult = LocalVisitResult<'ctx>;

    fn visit<N>(mut self) -> (Ast<'ast, N>, Self::GlobalVisitResult, Self::LocalVisitResult)
        where AstPartlyResolved: AstState<NextState = N>, N: AstState
    {
        self.visit_stmts(self.ast.main_scope.stmts);
        if !self.diagnostics.is_empty() {
            report_diagnostics(self.diagnostics);
        }
        (
            self.ast.next_state(),
            GlobalVisitResult {
                fns: self.fns,
                clib_fns: self.clib_fns,
                pkg_def_id_to_name_binding: self.pkg_def_id_to_name_binding,
                trait_impl_id_to_def_ids: self.trait_impl_id_to_def_ids,
            },
            LocalVisitResult {
                node_id_to_def_id: self.local_visit_result.node_id_to_def_id,
                def_id_to_name_binding: self.def_id_to_name_binding,
                node_id_to_ty: self.node_id_to_type,
            },
        )
    }
}

impl<'ast, 'ctx, 'c> Ast<'ast, AstPartlyResolved> where 'ctx: 'ast, 'ast: 'c {
    pub fn into_visitor<E>(
        self,
        resolver_handle: &'c E,
        local_visit_result: ast_pre_resolver::LocalVisitResult
    ) -> AstResolver<'ctx, 'ast, 'c, E>
        where E: ResolverHandle<'ctx, 'ast, AstPartlyResolved>
    {
        AstResolver::new(self, resolver_handle, local_visit_result)
    }
}

impl<'ctx, 'ast, 'b, E> AstResolver<'ctx, 'ast, 'b, E>
    where E: ResolverHandle<'ctx, 'ast, AstPartlyResolved>, 'ctx: 'ast, 'ast: 'b
{
    pub fn new(
        ast: Ast<'ast, AstPartlyResolved>,
        resolver_handle: &'b E,
        local_visit_result: ast_pre_resolver::LocalVisitResult
    ) -> Self {
        Self {
            resolver_handle,
            trait_impl_context: None,
            local_visit_result,
            pkg_def_id_to_name_binding: Default::default(),
            def_id_to_name_binding: FxHashMap::with_capacity_and_hasher(
                ast.metadata.def_count,
                FxBuildHasher::default()
            ),
            node_id_to_type: FxHashMap::with_capacity_and_hasher(
                ast.metadata.node_count,
                FxBuildHasher::default()
            ),
            trait_impl_id_to_def_ids: FxHashMap::default(),
            clib_fns: Vec::new(),
            fns: Vec::with_capacity(ast.metadata.fn_count),
            ast,
            diagnostics: Vec::new(),
        }
    }

    fn get_mut_or_create_def_ids_from_trait_impl_id(
        &mut self,
        trait_impl_id: TraitImplId
    ) -> &mut Vec<DefId> {
        self.trait_impl_id_to_def_ids.entry(trait_impl_id).or_default()
    }

    fn report_error(&mut self, error_kind: ErrorKind, span: Span) {
        self.diagnostics.push(Diagnostic::new_error(error_kind, span));
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
                let def_id = self.get_def_id_from_node_id(ident_node.ast_node_id);

                let res_kind = if def_id.symbol.can_be_constant() {
                    ResKind::ConstVariable
                } else {
                    ResKind::Variable
                };

                self.make_lexical_binding_to_def_id(def_id, res_kind);
            }
            Pat::TupleStructPat(tuple_struct_pat) => {
                for field in tuple_struct_pat.fields.iter() {
                    self.traverse_pat_and_bind_def_ids_to_lexical_bindings(*field);
                }
            }
        }
    }

    fn resolve_path_def_id(&mut self, path: Path<'ast>) -> DefId {
        match path {
            Path::PathField(path_field) => {
                let _ = self.resolve_path_def_id(path_field.lhs);

                if let Path::PathPkg(_) = path_field.lhs {
                    let rhs_symbol = Symbol::from_node_id(path_field.rhs.ast_node_id);
                    let def_id = self.resolver_handle
                        .lookup_pkg_member(rhs_symbol)
                        .unwrap_or_else(||
                            panic!("`{}` is not a member of package", rhs_symbol.get())
                        );
                    self.set_def_id_to_node_id(path_field.rhs.ast_node_id, def_id);
                    def_id
                } else {
                    panic!("Extern packages are not yet supported")
                }
            }
            Path::PathSegment(ident_node) => {
                let def_id = self
                    .lookup_ident_declaration(ident_node, ResKind::Adt)
                    .unwrap_or_else(||
                        panic!(
                            "Expected adt `{}` to be defined",
                            Symbol::from_node_id(ident_node.ast_node_id).get()
                        )
                    );
                self.set_def_id_to_node_id(ident_node.ast_node_id, def_id);
                def_id
            }
            Path::PathPkg(pkg_ident_node) => {
                let def_id = self.resolver_handle.get_or_set_pkg_def_id(pkg_ident_node);
                self.set_def_id_to_node_id(pkg_ident_node.ast_node_id, def_id);
                def_id
            }
        }
    }

    fn get_lexical_context_from_node_id(&self, node_id: NodeId) -> LexicalContext {
        self.local_visit_result.node_id_to_lexical_context.get(&node_id).copied().unwrap()
    }

    fn is_main_scope(&self, node_id: NodeId) -> bool {
        let lexical_context = self.get_lexical_context_from_node_id(node_id);
        lexical_context == LexicalContext::new(ContextId(0), ScopeId(0))
    }

    fn make_lexical_binding_to_def_id(&mut self, def_id: DefId, res_kind: ResKind) {
        let lexical_context = self.get_lexical_context_from_node_id(def_id.node_id);
        let lexical_binding = LexicalBinding::new(
            lexical_context,
            def_id.symbol,
            res_kind
            // def_id.node_id.mod_id
        );
        self.local_visit_result.lexical_binding_to_def_id.insert(lexical_binding, def_id);
    }

    fn set_def_id_to_node_id(&mut self, node_id: NodeId, def_id: DefId) {
        self.local_visit_result.node_id_to_def_id.insert(node_id, def_id);
    }

    fn set_type_to_node_id(&mut self, node_id: NodeId, ty: Ty) {
        self.node_id_to_type.insert(node_id, ty);
    }

    fn set_namebinding_to_def_id(&mut self, def_id: DefId, name_binding: NameBinding<'ctx>) {
        if let NameBindingKind::Fn(_, _, Externism::Clib) = name_binding.kind {
            self.clib_fns.push(def_id);
        }

        match name_binding.kind {
            NameBindingKind::Adt(_) | NameBindingKind::Fn(_, _, _) => {
                self.pkg_def_id_to_name_binding.insert(def_id, name_binding);
            }
            _ => {}
        }

        self.def_id_to_name_binding.insert(def_id, name_binding);
    }

    fn get_def_id_from_node_id(&self, node_id: NodeId) -> DefId {
        *self.local_visit_result.node_id_to_def_id.get(&node_id).expect("Expected DefId")
    }

    fn type_from_typing(&mut self, typing: &Typing<'ast>, item_type: ItemType) -> Ty {
        match typing {
            Typing::SelfType => {
                if let Some(TraitImplId { implementor_def_id, .. }) = self.trait_impl_context {
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
                            let Some(def_id) = self.lookup_ident_declaration(
                                ident_node,
                                ResKind::Adt
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
                    tuple_ty.push(self.type_from_typing(typing, item_type));
                }
                Ty::Tuple(TyCtx::intern_many_types(tuple_ty))
            }
            Typing::Fn(args_typing, ret_typing) => {
                let args_ty = {
                    let mut args_ty = Vec::with_capacity(args_typing.len());
                    for typing in args_typing.iter() {
                        args_ty.push(self.type_from_typing(typing, item_type));
                    }
                    TyCtx::intern_many_types(args_ty)
                };
                let ret_ty = ret_typing.map(|typing| self.type_from_typing(typing, item_type));
                Ty::FnSig(FnSig::new(args_ty, TyCtx::intern_type(ret_ty.unwrap_or(VOID_TY))))
            }
            Typing::Ptr(typing, mutability) => {
                if item_type == ItemType::Normal {
                    todo!("Report error: Cannot use pointers in this context");
                }
                Ty::Ptr(TyCtx::intern_type(self.type_from_typing(typing, item_type)), *mutability)
            }
            Typing::ManyPtr(typing) => {
                if item_type == ItemType::Normal {
                    todo!("Report error: Cannot use pointers in this context");
                }

                Ty::ManyPtr(
                    TyCtx::intern_type(self.type_from_typing(typing, item_type)),
                    Mutability::Immutable
                )
            }
        }
    }

    fn lookup_ident_declaration(
        &mut self,
        ident_node: &'ast IdentNode,
        res_kind: ResKind
    ) -> Option<DefId> {
        let node_id = ident_node.ast_node_id;
        let symbol = Symbol::from_node_id(node_id);

        match res_kind {
            ResKind::ConstVariable => {
                let start_context = self.get_lexical_context_from_node_id(node_id);

                let mut current_context = start_context;

                loop {
                    if
                        let Some(def_id) = self.local_visit_result.lexical_binding_to_def_id.get(
                            &LexicalBinding::new(current_context, symbol, res_kind)
                        )
                    {
                        return Some(*def_id);
                    }
                    if
                        let Some(parent_context) =
                            self.local_visit_result.lexical_context_to_parent_lexical_context.get(
                                &current_context
                            )
                    {
                        current_context = *parent_context;
                    } else {
                        break;
                    }
                }

                todo!();
            }
            ResKind::ConstStr => {
                unimplemented!("Should not be here (const str in lookup_ident_declaration)");
                // if let Some(&(def_id, _)) = self.str_symbol_to_def_id.get(&symbol) {
                //     return Some(def_id);
                // }
            }
            ResKind::Variable => {
                let start_context = self.get_lexical_context_from_node_id(node_id);

                let mut current_context = start_context;
                loop {
                    // Can't lookup variables in other contexts (e.g. outside of a function)
                    if current_context.context_id != start_context.context_id {
                        break;
                    }

                    if
                        let Some(def_id) = self.local_visit_result.lexical_binding_to_def_id.get(
                            &LexicalBinding::new(current_context, symbol, res_kind)
                        )
                    {
                        return Some(*def_id);
                    }
                    if
                        let Some(parent_context) =
                            self.local_visit_result.lexical_context_to_parent_lexical_context.get(
                                &current_context
                            )
                    {
                        current_context = *parent_context;
                    } else {
                        break;
                    }
                }
            }
            ResKind::Fn | ResKind::Adt => {
                let start_context = self.get_lexical_context_from_node_id(node_id);

                let mut current_context = start_context;
                loop {
                    let lexical_binding = LexicalBinding::new(
                        current_context,
                        symbol,
                        res_kind
                        // node_id.mod_id
                    );
                    if
                        let Some(def_id) = self.local_visit_result.lexical_binding_to_def_id.get(
                            &lexical_binding
                        )
                    {
                        return Some(*def_id);
                    }
                    if
                        let Some(parent_context) =
                            self.local_visit_result.lexical_context_to_parent_lexical_context.get(
                                &current_context
                            )
                    {
                        current_context = *parent_context;
                    } else {
                        break;
                    }
                }
            }
        }

        None
    }
}

impl<'ctx, 'ast, 'b, E> Visitor<'ast>
    for AstResolver<'ctx, 'ast, 'b, E>
    where 'ctx: 'ast, 'ast: 'b, E: ResolverHandle<'ctx, 'ast, AstPartlyResolved>
{
    type Result = ();

    fn default_result() -> Self::Result {}

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
        if
            let Expr::ExprWithoutBlock(ExprWithoutBlock::PlaceExpr(PlaceExpr::PkgIdentExpr(_))) =
                &field_expr.lhs
        {
            let rhs_symbol = Symbol::from_node_id(field_expr.rhs.ast_node_id);
            let pkg_member = self.resolver_handle.lookup_pkg_member(rhs_symbol);
            if let Some(def_id) = pkg_member {
                let is_symbol_in_file = self.local_visit_result.lexical_binding_to_def_id
                    .values()
                    .find(|&x| x.symbol == rhs_symbol);

                if is_symbol_in_file.is_some() {
                    println!(
                        "Suggestion: `{}` is defined in the current file so it's unecessary to access it like this: pkg.{}",
                        rhs_symbol.get(),
                        rhs_symbol.get()
                    );
                }
                self.set_def_id_to_node_id(field_expr.rhs.ast_node_id, def_id);
            } else {
                todo!("Report error: Undefined package member: {}", rhs_symbol.get());
            }
        } else {
            self.visit_expr(field_expr.lhs)
        }
    }

    fn visit_struct_expr(&mut self, struct_expr: &'ast StructExpr<'ast>) -> Self::Result {
        self.visit_ident_expr(struct_expr.ident_node);

        for field in struct_expr.field_initializations.iter() {
            self.visit_expr(field.value);
        }

        Self::default_result()
    }

    fn visit_ident_expr(&mut self, ident_node: &'ast IdentNode) -> Self::Result {
        if let Some(def_id) = self.lookup_ident_declaration(ident_node, ResKind::Fn) {
            self.set_def_id_to_node_id(ident_node.ast_node_id, def_id);
        } else if let Some(def_id) = self.lookup_ident_declaration(ident_node, ResKind::Adt) {
            self.set_def_id_to_node_id(ident_node.ast_node_id, def_id);
        } else {
            let symbol = Symbol::from_node_id(ident_node.ast_node_id);
            let res_kind = if symbol.can_be_constant() {
                ResKind::ConstVariable
            } else {
                ResKind::Variable
            };
            if let Some(def_id) = self.lookup_ident_declaration(ident_node, res_kind) {
                self.set_def_id_to_node_id(ident_node.ast_node_id, def_id);
            } else if
                // This is for the atd constructor `Self`
                let (Some(TraitImplId { implementor_def_id, .. }), true) = (
                    self.trait_impl_context,
                    Symbol::from_node_id(ident_node.ast_node_id) == *BIG_SELF_SYMBOL,
                )
            {
                self.set_def_id_to_node_id(ident_node.ast_node_id, implementor_def_id);
            } else {
                self.report_error(
                    ErrorKind::UndefinedLookup {
                        symbol: Symbol::from_node_id(ident_node.ast_node_id),
                        res_kind: ResKind::Variable,
                    },
                    ident_node.span
                );
            }
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
        for import_item in import_item.import_items_path.iter() {
            let def_id = self.resolve_path_def_id(*import_item);
            let lexical_binding = LexicalBinding::new(
                LexicalContext { context_id: ContextId(0), scope_id: ScopeId(0) },
                def_id.symbol,
                self.resolver_handle.lookup_pkg_member_res_kind(&def_id)
            );
            self.local_visit_result.lexical_binding_to_def_id.insert(lexical_binding, def_id);
        }
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
                self.set_type_to_node_id(arg.ident.ast_node_id, ty);
                ty
            })
            .collect::<Vec<_>>();
        let args_tys = TyCtx::intern_many_types(args_tys);

        let def_id = self.get_def_id_from_node_id(comp_fn_decl_item.ident_node.ast_node_id);
        let name_binding = NameBinding::new(
            NameBindingKind::Fn(FnSig::new(args_tys, ret_ty), HasSelfArg::No, Externism::Clib)
        );
        self.set_namebinding_to_def_id(def_id, name_binding);
        self.set_type_to_node_id(comp_fn_decl_item.ident_node.ast_node_id, Ty::FnDef(def_id));
        self.set_type_to_node_id(comp_fn_decl_item.ast_node_id, VOID_TY);
    }

    fn visit_struct_item(&mut self, struct_item: &'ast StructItem<'ast>) -> Self::Result {
        let struct_data = struct_item.field_declarations
            .iter()
            .map(|field| {
                let def_id = self.get_def_id_from_node_id(field.ident.ast_node_id);
                let ty = self.type_from_typing(&field.type_expr, struct_item.item_type);
                (def_id, ty)
            })
            .collect::<Vec<_>>();
        let struct_data = TyCtx::intern_many_types(struct_data);

        let def_id = self.get_def_id_from_node_id(struct_item.ident_node.ast_node_id);
        let name_binding = NameBinding::new(NameBindingKind::Adt(Adt::Struct(struct_data)));
        self.set_namebinding_to_def_id(def_id, name_binding);
        self.set_type_to_node_id(struct_item.ident_node.ast_node_id, Ty::Adt(def_id));
        self.set_type_to_node_id(struct_item.ast_node_id, VOID_TY);
    }

    fn visit_impl_item(&mut self, impl_item: &'ast ImplItem<'ast>) -> Self::Result {
        let implementor_id = self.resolve_path_def_id(impl_item.implementor_path);

        let trait_impl_id = TraitImplId::new(implementor_id, None);

        self.begin_impl_context(trait_impl_id);

        for fn_item in impl_item.impl_fns.iter() {
            self.visit_fn_item(fn_item);
        }

        self.end_impl_context();

        self.set_type_to_node_id(impl_item.ast_node_id, VOID_TY);
    }

    fn visit_fn_item(&mut self, fn_item: &'ast FnItem<'ast>) -> Self::Result {
        let def_id = self.get_def_id_from_node_id(fn_item.ident_node.ast_node_id);
        let implementor_def_id = if let Some(trait_impl_id) = self.trait_impl_context {
            self.get_mut_or_create_def_ids_from_trait_impl_id(trait_impl_id).push(def_id);
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
                let (arg_ty, mutability) = match arg_kind {
                    // TODO: Check for multiple self args = error
                    ArgKind::Arg(field) => {
                        let ty = self.type_from_typing(&field.type_expr, fn_item.item_type);
                        (ty, Mutability::Immutable)
                    }
                    t => {
                        has_self_arg = HasSelfArg::Yes;
                        let implementor_def_id = if let Some(x) = implementor_def_id {
                            x
                        } else {
                            panic!("Expected `Self` to be inside an `impl` block");
                        };
                        match t {
                            ArgKind::MutPtrSelf(_) =>
                                (Ty::Adt(implementor_def_id).to_mut_ptr_ty(), Mutability::Mutable),
                            ArgKind::MutSelf(_) =>
                                (Ty::Adt(implementor_def_id).to_mut_ptr_ty(), Mutability::Mutable),
                            ArgKind::NormalSelf(_) =>
                                (Ty::Adt(implementor_def_id).to_ptr_ty(), Mutability::Immutable),
                            ArgKind::PtrSelf(_) =>
                                (Ty::Adt(implementor_def_id).to_ptr_ty(), Mutability::Immutable),
                            ArgKind::Arg(_) => unreachable!(),
                        }
                    }
                };

                let def_id = self.get_def_id_from_node_id(
                    get_ident_node_from_arg_kind(*arg_kind).ast_node_id
                );

                self.set_type_to_node_id(def_id.node_id, arg_ty);
                self.set_namebinding_to_def_id(
                    def_id,
                    NameBinding::new(NameBindingKind::Variable(mutability))
                );
                self.make_lexical_binding_to_def_id(def_id, ResKind::Variable);
                arg_ty
            })
            .collect::<Vec<_>>();
        let args_ty = TyCtx::intern_many_types(args_ty);

        let fn_sig = FnSig::new(args_ty, TyCtx::intern_type(ret_ty));
        let name_binding = NameBinding::new(
            NameBindingKind::Fn(fn_sig, has_self_arg, Externism::NoExtern)
        );
        self.set_namebinding_to_def_id(def_id, name_binding);
        self.set_type_to_node_id(fn_item.ident_node.ast_node_id, Ty::FnDef(def_id));
        self.set_type_to_node_id(fn_item.ast_node_id, VOID_TY);
        // self.set_def_id_to_global_mem(def_id);

        if def_id.symbol == *MAIN_SYMBOL && self.is_main_scope(fn_item.ident_node.ast_node_id) {
            if !self.resolver_handle.set_main_fn(fn_item) {
                panic!(
                    "Duplicate definitions of entry point `main` in global scope (report error)"
                );
            }
        } else {
            self.fns.push(fn_item);
        }

        self.visit_stmts(fn_item.body);
    }

    fn visit_enum_item(&mut self, enum_item: &'ast EnumItem<'ast>) -> Self::Result {
        let def_id = self.get_def_id_from_node_id(enum_item.ident_node.ast_node_id);

        let variant_def_ids = enum_item.variants
            .iter()
            .enumerate()
            .map(|(i, variant)| {
                let variant_def_id = self.get_def_id_from_node_id(variant.ident_node.ast_node_id);
                let enum_data_ty = variant.enum_data
                    .map(|x|
                        x
                            .iter()
                            .map(|y| self.type_from_typing(y, enum_item.item_type))
                            .collect::<Vec<_>>()
                    )
                    .unwrap_or(vec![Ty::ZeroSized]);
                let enum_data_ty = TyCtx::intern_many_types(enum_data_ty);

                self.set_type_to_node_id(
                    variant.ident_node.ast_node_id,
                    Ty::AtdConstructer(variant_def_id)
                );

                let name_binding = NameBinding::new(
                    NameBindingKind::Adt(
                        Adt::EnumVariant(def_id, EmumVaraintId(i as u32), enum_data_ty)
                    )
                );
                self.set_namebinding_to_def_id(variant_def_id, name_binding);
                variant_def_id
            })
            .collect::<Vec<_>>();
        let variant_def_ids = TyCtx::intern_many_types(variant_def_ids);

        let name_binding = NameBinding::new(NameBindingKind::Adt(Adt::Enum(variant_def_ids)));
        self.set_namebinding_to_def_id(def_id, name_binding);
        self.set_type_to_node_id(enum_item.ident_node.ast_node_id, Ty::Adt(def_id));
        self.set_type_to_node_id(enum_item.ast_node_id, VOID_TY);
    }

    fn visit_typedef_item(&mut self, typedef_item: &'ast TypedefItem<'ast>) -> Self::Result {
        let def_id = self.get_def_id_from_node_id(typedef_item.ident_node.ast_node_id);
        let ty = self.type_from_typing(&typedef_item.type_expr, typedef_item.item_type);
        let name_binding = NameBinding::new(NameBindingKind::Adt(Adt::Typedef(ty)));
        self.set_namebinding_to_def_id(def_id, name_binding);
        self.set_type_to_node_id(typedef_item.ident_node.ast_node_id, Ty::Adt(def_id));
        self.set_type_to_node_id(typedef_item.ast_node_id, VOID_TY);
    }
}
