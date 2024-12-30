use std::marker::PhantomData;

use fxhash::{ FxBuildHasher, FxHashMap };
use ir::{
    ContextId,
    DefId,
    LexicalBinding,
    LexicalContext,
    NodeId,
    ResKind,
    ScopeId,
    Symbol,
    BOOL_SYMBOL,
    FLOAT_32_SYMBOL,
    FLOAT_64_SYMBOL,
    FLOAT_SYMBOL,
    INT16_SYMBOL,
    INT32_SYMBOL,
    INT64_SYMBOL,
    INT8_SYMBOL,
    INT_SYMBOL,
    STR_SYMBOL,
    UINT16_SYMBOL,
    UINT32_SYMBOL,
    UINT64_SYMBOL,
    UINT8_SYMBOL,
    UINT_SYMBOL,
    VOID_SYMBOL,
};

use crate::{
    ArgKind,
    Ast,
    AstState,
    AstUnvalidated,
    BlockExpr,
    CompFnDeclItem,
    CondKind,
    DefineStmt,
    EnumItem,
    FieldExpr,
    FnItem,
    IdentNode,
    IfExpr,
    ImplItem,
    ImportItem,
    Pat,
    Path,
    PathField,
    PkgIdentNode,
    ResolverHandle,
    StructExpr,
    StructItem,
    TypedefItem,
    Typing,
    VisitAst,
    Visitor,
};

#[derive(Debug)]
pub struct LocalVisitResult {
    pub lexical_context_to_parent_lexical_context: FxHashMap<LexicalContext, LexicalContext>,
    pub node_id_to_lexical_context: FxHashMap<NodeId, LexicalContext>,
    pub lexical_binding_to_def_id: FxHashMap<LexicalBinding, DefId>,
    pub node_id_to_def_id: FxHashMap<NodeId, DefId>,
}

#[derive(Debug)]
pub struct GlobalVisitResult {
    pub pkg_symbol_to_def_id: FxHashMap<Symbol, DefId>,
    pub pkg_def_id_to_res_kind: FxHashMap<DefId, ResKind>,
}

#[derive(Debug)]
pub struct AstPreResolver<'ctx, 'ast, 'b, E> where E: ResolverHandle<'ctx, 'ast, AstUnvalidated> {
    ast: Ast<'ast, AstUnvalidated>,
    resolver_handle: &'b E,
    lexical_context_stack: Vec<LexicalContext>,
    lexical_context_to_parent_lexical_context: FxHashMap<LexicalContext, LexicalContext>,
    node_id_to_lexical_context: FxHashMap<NodeId, LexicalContext>,
    node_id_to_def_id: FxHashMap<NodeId, DefId>,
    lexical_binding_to_def_id: FxHashMap<LexicalBinding, DefId>,
    pkg_symbol_to_def_id: FxHashMap<Symbol, DefId>,
    pkg_def_id_to_res_kind: FxHashMap<DefId, ResKind>,
    next_scope_id: ScopeId,
    next_context_id: ContextId,
    is_in_impl: bool,
    marker: PhantomData<&'ctx ()>,
}

impl<'ctx, 'ast, 'b, E> VisitAst<'ast, AstUnvalidated>
    for AstPreResolver<'ctx, 'ast, 'b, E>
    where 'ctx: 'ast, 'ast: 'b, E: ResolverHandle<'ctx, 'ast, AstUnvalidated>
{
    type GlobalVisitResult = GlobalVisitResult;
    type LocalVisitResult = LocalVisitResult;

    fn visit<N>(mut self) -> (Ast<'ast, N>, Self::GlobalVisitResult, Self::LocalVisitResult)
        where AstUnvalidated: AstState<NextState = N>, N: AstState
    {
        self.visit_stmts(self.ast.main_scope.stmts);
        (
            self.ast.next_state(),
            GlobalVisitResult {
                pkg_symbol_to_def_id: self.pkg_symbol_to_def_id,
                pkg_def_id_to_res_kind: self.pkg_def_id_to_res_kind,
            },
            LocalVisitResult {
                lexical_context_to_parent_lexical_context: self.lexical_context_to_parent_lexical_context,
                node_id_to_lexical_context: self.node_id_to_lexical_context,
                lexical_binding_to_def_id: self.lexical_binding_to_def_id,
                node_id_to_def_id: self.node_id_to_def_id,
            },
        )
    }
}

impl<'ast, 'ctx, 'c> Ast<'ast, AstUnvalidated> where 'ctx: 'ast, 'ast: 'c {
    pub fn into_visitor<E>(self, resolver_handle: &'c E) -> AstPreResolver<'ctx, 'ast, 'c, E>
        where E: ResolverHandle<'ctx, 'ast, AstUnvalidated>
    {
        AstPreResolver::new(self, resolver_handle)
    }
}

impl<'ctx, 'ast, 'b, E> AstPreResolver<'ctx, 'ast, 'b, E>
    where E: ResolverHandle<'ctx, 'ast, AstUnvalidated>, 'ctx: 'ast, 'ast: 'b
{
    pub fn new(ast: Ast<'ast, AstUnvalidated>, resolver_handle: &'b E) -> Self {
        Self {
            resolver_handle,
            lexical_context_stack: vec![LexicalContext::new(ContextId(0), ScopeId(0))],
            lexical_context_to_parent_lexical_context: FxHashMap::default(),
            node_id_to_lexical_context: FxHashMap::with_capacity_and_hasher(
                ast.metadata.node_count,
                FxBuildHasher::default()
            ),
            node_id_to_def_id: FxHashMap::with_capacity_and_hasher(
                ast.metadata.node_count,
                FxBuildHasher::default()
            ),
            lexical_binding_to_def_id: FxHashMap::with_capacity_and_hasher(
                ast.metadata.def_count,
                FxBuildHasher::default()
            ),
            pkg_symbol_to_def_id: FxHashMap::default(),
            pkg_def_id_to_res_kind: FxHashMap::default(),
            next_scope_id: ScopeId(1),
            next_context_id: ContextId(1),
            is_in_impl: false,
            ast,
            marker: PhantomData,
        }
    }

    fn make_def_id_and_bind_to_node_id(&mut self, node_id: NodeId, symbol: Symbol) -> DefId {
        let def_id = DefId::new(symbol, node_id);
        self.node_id_to_def_id.insert(node_id, def_id);
        def_id
    }

    fn make_lexical_binding_to_def_id(&mut self, def_id: DefId, res_kind: ResKind) {
        let lexical_context = self.get_lexical_context();
        let lexical_binding = LexicalBinding::new(
            lexical_context,
            def_id.symbol,
            res_kind
            // def_id.node_id.mod_id
        );

        self.pkg_def_id_to_res_kind.insert(def_id, res_kind);
        self.lexical_binding_to_def_id.insert(lexical_binding, def_id);
    }

    fn set_node_id_to_lexical_context(&mut self, node_id: NodeId, lexical_context: LexicalContext) {
        self.node_id_to_lexical_context.insert(node_id, lexical_context);
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
    fn is_in_main_scope(&self) -> bool {
        self.lexical_context_stack.len() == 1
    }
    fn make_pkg_def_if_in_main_scope(&mut self, def_id: DefId) {
        if self.is_in_main_scope() {
            self.pkg_symbol_to_def_id.insert(def_id.symbol, def_id);
        }
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
                self.make_def_id_and_bind_to_node_id(
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
            Path::PathPkg(_) => {}
        }
    }

    fn get_current_context_id(&self) -> ContextId {
        self.lexical_context_stack.last().expect("Expected at least one context").context_id
    }

    fn get_lexical_context(&self) -> LexicalContext {
        *self.lexical_context_stack.last().expect("Expected at least one lexical context")
    }

    fn bind_node_id_to_lexical_context(&mut self, node_id: NodeId) {
        let lexical_context = self.get_lexical_context();
        self.set_node_id_to_lexical_context(node_id, lexical_context);
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

impl<'ctx, 'ast, 'b, E> Visitor<'ast>
    for AstPreResolver<'ctx, 'ast, 'b, E>
    where 'ctx: 'ast, E: ResolverHandle<'ctx, 'ast, AstUnvalidated>, 'ast: 'b
{
    type Result = ();

    fn default_result() -> Self::Result {
        
    }

    fn visit_string_expr(&mut self, string_expr: &'ast crate::StringExpr) -> Self::Result {
        let def_id = self.resolver_handle.make_const_str(string_expr, || -> DefId {
            self.make_def_id_and_bind_to_node_id(
                string_expr.ast_node_id,
                Symbol::from_node_id(string_expr.ast_node_id)
            )
        });
        self.node_id_to_def_id.insert(string_expr.ast_node_id, def_id);
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

    fn visit_path_field(&mut self, path_field: &'ast PathField<'ast>) -> Self::Result {
        self.visit_path(path_field.lhs);
        self.bind_node_id_to_lexical_context(path_field.rhs.ast_node_id);
        self.bind_node_id_to_lexical_context(path_field.ast_node_id);
    }

    fn visit_path_segment(&mut self, path_segment: &'ast IdentNode) -> Self::Result {
        self.bind_node_id_to_lexical_context(path_segment.ast_node_id);
    }

    fn visit_path_pkg(&mut self, pkg_ident_expr: &'ast PkgIdentNode) -> Self::Result {
        self.bind_node_id_to_lexical_context(pkg_ident_expr.ast_node_id);
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
            self.visit_path(*path);
        }
    }

    fn visit_comp_fn_decl_item(
        &mut self,
        comp_fn_decl_item: &'ast CompFnDeclItem<'ast>
    ) -> Self::Result {
        let def_id = self.make_def_id_and_bind_to_node_id(
            comp_fn_decl_item.ident_node.ast_node_id,
            Symbol::from_node_id(comp_fn_decl_item.ident_node.ast_node_id)
        );

        self.bind_node_id_to_lexical_context(comp_fn_decl_item.ident_node.ast_node_id);
        self.make_lexical_binding_to_def_id(def_id, ResKind::Fn);

        self.visit_ident_expr(comp_fn_decl_item.ident_node);

        self.start_scope();
        for arg in comp_fn_decl_item.args.iter() {
            let def_id = self.make_def_id_and_bind_to_node_id(
                arg.ident.ast_node_id,
                Symbol::from_node_id(arg.ident.ast_node_id)
            );
            self.make_pkg_def_if_in_main_scope(def_id);
            self.bind_node_id_to_lexical_context(arg.ident.ast_node_id);
            self.visit_typing(&arg.type_expr);
        }
        self.end_scope();

        if let Some(x) = comp_fn_decl_item.return_ty { self.visit_typing(&x) }
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
        self.traverse_path_and_bind_idents(impl_item.implementor_path);
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

        let def_id = self.make_def_id_and_bind_to_node_id(
            typedef_item.ident_node.ast_node_id,
            typedef_name_symbol
        );
        self.make_pkg_def_if_in_main_scope(def_id);
        self.bind_node_id_to_lexical_context(typedef_item.ident_node.ast_node_id);
        self.make_lexical_binding_to_def_id(def_id, ResKind::Adt);

        self.visit_typing(&typedef_item.type_expr);
    }

    fn visit_struct_item(&mut self, struct_item: &'ast StructItem<'ast>) -> Self::Result {
        let struct_name_symbol = Symbol::from_node_id(struct_item.ident_node.ast_node_id);

        if self.is_compiler_adt_name(struct_name_symbol) {
            panic!("Cannot shadow compiler ADT name: {}", struct_name_symbol.get());
        }

        let def_id = self.make_def_id_and_bind_to_node_id(
            struct_item.ident_node.ast_node_id,
            struct_name_symbol
        );
        self.make_pkg_def_if_in_main_scope(def_id);
        self.bind_node_id_to_lexical_context(struct_item.ident_node.ast_node_id);
        self.make_lexical_binding_to_def_id(def_id, ResKind::Adt);

        self.start_scope();

        for field in struct_item.field_declarations.iter() {
            self.make_def_id_and_bind_to_node_id(
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

        let def_id = self.make_def_id_and_bind_to_node_id(
            enum_item.ident_node.ast_node_id,
            enum_name_symbol
        );
        self.make_pkg_def_if_in_main_scope(def_id);
        self.bind_node_id_to_lexical_context(enum_item.ident_node.ast_node_id);
        self.make_lexical_binding_to_def_id(def_id, ResKind::Adt);

        self.start_scope();

        for variant in enum_item.variants.iter() {
            self.make_def_id_and_bind_to_node_id(
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
        let def_id = self.make_def_id_and_bind_to_node_id(
            fn_item.ident_node.ast_node_id,
            Symbol::from_node_id(fn_item.ident_node.ast_node_id)
        );
        self.make_pkg_def_if_in_main_scope(def_id);
        self.bind_node_id_to_lexical_context(fn_item.ident_node.ast_node_id);
        self.make_lexical_binding_to_def_id(def_id, ResKind::Fn);

        if let Some(x) = fn_item.return_ty { self.visit_typing(&x) }

        self.start_context();

        for arg in fn_item.args.iter() {
            match arg {
                | ArgKind::MutPtrSelf(ident_node)
                | ArgKind::PtrSelf(ident_node)
                | ArgKind::MutSelf(ident_node)
                | ArgKind::NormalSelf(ident_node) => {
                    self.make_def_id_and_bind_to_node_id(
                        ident_node.ast_node_id,
                        Symbol::from_node_id(ident_node.ast_node_id)
                    );
                    self.bind_node_id_to_lexical_context(ident_node.ast_node_id);
                }
                ArgKind::Arg(arg) => {
                    self.make_def_id_and_bind_to_node_id(
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
