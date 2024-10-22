use std::cell::RefCell;

use ast::{
    Ast,
    AstQueryEntry,
    AstState,
    AstState0,
    AstState1,
    AstState2,
    AstState3,
    AstTypeChecked,
    AstVisitEmitter,
    FnItem,
};
use bumpalo::Bump;
use error::{ Error, ErrorKind, Severity };
use fxhash::{ FxBuildHasher, FxHashMap };
use ir::{
    ContextId,
    DefId,
    DefIdToNameBinding,
    GlobalMem,
    GlobalMemId,
    LexicalBinding,
    LexicalContext,
    NameBinding,
    NodeId,
    ResKind,
    ResolvedInformation,
    ScopeId,
    Symbol,
    Ty,
    TyCtx,
};
use span::Span;

/// Main resolver struct. This is responsible for validating the Ast and creating the Cfg from it
pub struct Resolver<'ctx, 'ast> {
    lexical_context_stack: Vec<LexicalContext>,
    next_scope_id: ScopeId,
    next_context_id: ContextId,
    // next_mod_id: ModId,
    lexical_binding_to_def_id: FxHashMap<LexicalBinding, DefId>,

    /// Built during the first pass (name resolution) and then used in the rest of the passes
    node_id_to_def_id: FxHashMap<NodeId, DefId>,

    def_id_to_name_binding: FxHashMap<DefId, NameBinding<'ctx>>,
    node_id_to_ty: FxHashMap<NodeId, Ty>,

    def_id_to_global_mem_id: FxHashMap<DefId, GlobalMemId>,
    global_mems: &'ctx RefCell<Vec<GlobalMem>>,

    found_main_fn: Option<&'ast FnItem<'ast>>,
    pending_functions: Vec<&'ast FnItem<'ast>>,

    /* Arena */
    arena: &'ctx Bump,

    /* Used for error reporting */
    src: &'ast str,
    errors: Vec<Error>,
}
pub struct ResolvedFunctions<'ast> {
    pub pending_functions: Vec<&'ast FnItem<'ast>>,
    pub main_fn: Option<&'ast FnItem<'ast>>,
}

impl<'ctx, 'ast> Resolver<'ctx, 'ast> where 'ctx: 'ast {
    pub fn take_resolved_information(self) -> (ResolvedFunctions<'ast>, ResolvedInformation<'ctx>) {
        if self.has_errors() {
            self.print_errors();
            self.exit_if_has(Severity::NoImpact);
        }

        (
            ResolvedFunctions {
                main_fn: self.found_main_fn,
                pending_functions: self.pending_functions,
            },
            ResolvedInformation {
                node_id_to_def_id: self.node_id_to_def_id,
                node_id_to_ty: self.node_id_to_ty,
                def_id_to_name_binding: self.def_id_to_name_binding,
                def_id_to_global_mem_id: self.def_id_to_global_mem_id,
            },
        )
    }

    /// Makes a Resolver and builds the query system in the Ast
    pub fn from_ast(
        src: &'ast str,
        ast: Ast<'ast, AstState0>,
        arena: &'ctx Bump,
        global_mems: &'ctx RefCell<Vec<GlobalMem>>
    ) -> (Self, Ast<'ast, AstState1>) {
        /// Sets up query system
        fn build_ast_query_system<'ctx, 'ast>(
            resolver: &mut Resolver<'ctx, 'ast>,
            ast: Ast<'ast, AstState0>
        ) -> Ast<'ast, AstState1>
            where 'ctx: 'ast
        {
            let ast_visitor = ast.get_visitor(resolver.src, resolver);

            let unvalidated_ast = ast_visitor.visit();

            unvalidated_ast
        }

        let mut lexical_context_stack = Vec::with_capacity(16);
        lexical_context_stack.push(LexicalContext::new(ContextId(0), ScopeId(0)));

        macro_rules! hashmap_with_capacity {
            ($capacity:expr) => {
                FxHashMap::with_capacity_and_hasher(
                    $capacity,
                    FxBuildHasher::default()
                )
            };
        }

        let mut resolver = Self {
            global_mems,
            def_id_to_global_mem_id: Default::default(),
            def_id_to_name_binding: Default::default(),
            lexical_binding_to_def_id: Default::default(),
            node_id_to_def_id: hashmap_with_capacity!(ast.expected_node_count()),
            node_id_to_ty: hashmap_with_capacity!(ast.expected_node_count()),
            arena,
            lexical_context_stack,
            next_scope_id: ScopeId(1),
            next_context_id: ContextId(1),
            found_main_fn: None,
            pending_functions: Vec::with_capacity(ast.fn_count),
            src,
            errors: Default::default(),
        };

        let ast_next_state = build_ast_query_system(&mut resolver, ast);

        (resolver, ast_next_state)
    }

    /// Performs name resolution
    pub fn resolve_ast(&mut self, ast: Ast<'ast, AstState1>) -> Ast<'ast, AstState2> {
        return ast.next_state();

        // fn remove_temp_storage<'ctx, 'ast>(resolver: &mut Resolver<'ctx, 'ast>) {
        //     // resolver.symbol_and_scope_to_def_id = Default::default();
        //     // resolver.lexical_context_stack = vec![];
        //     // resolver.next_scope_id = ScopeId(0);
        //     // resolver.next_context_id = ContextId(0);
        // }

        // let ast_visitor = ast.get_visitor(self.src, self);

        // let resolved_ast = ast_visitor.visit();

        // if self.has_errors() {
        //     self.exit_if_has(Severity::Fatal);
        // }

        // remove_temp_storage(self);

        // resolved_ast
    }

    /// Performs type checking
    pub fn type_check_ast(&mut self, ast: Ast<'ast, AstState2>) -> Ast<'ast, AstState3> {
        let ast_visitor = ast.get_visitor(self.src, self);

        let type_checked_ast = ast_visitor.visit();

        if self.has_errors() {
            self.exit_if_has(Severity::NoImpact);
        }

        self.assert_type_to_all_nodes(&type_checked_ast);

        type_checked_ast
    }

    fn assert_type_to_all_nodes(&self, type_checked_ast: &Ast<'ast, AstTypeChecked>) {
        // It has previously been asserted, that all nodes is inserted into the query system
        // So therefore this is also going to assert the condition for every node
        type_checked_ast.query_all(|node_id, query_entry| {
            let str = match query_entry {
                AstQueryEntry::IdentNode(ident_node) =>
                    Some(&self.src[ident_node.span.get_byte_range()]),
                _ => None,
            };

            assert_eq!(
                true,
                self.node_id_to_ty.get(node_id).is_some(),
                "Expected all nodes to have a type. Node {} is missing one. Details:\nName: {:?}\n\nMore:\n{:?}",
                node_id,
                str,
                query_entry
            )
        });
    }

    fn has_errors(&self) -> bool {
        self.errors.len() > 0
    }

    fn print_errors(&self) {
        let mut buffer = String::with_capacity(2048);

        for error in self.errors.iter() {
            error.write_msg(&mut buffer, self.src);
        }

        println!("{}", buffer);
    }

    fn exit_if_has(&self, severity: Severity) {
        self.print_errors();
        if self.has_error_severity(severity) {
            std::process::exit(1)
        }
    }

    fn has_error_severity(&self, severity: Severity) -> bool {
        self.errors
            .iter()
            .find(|e| e.get_severity() == severity)
            .is_some()
    }

    fn get_current_scope_id(&self) -> ScopeId {
        self.lexical_context_stack.last().expect("Expected at least one scope").scope_id
    }

    fn get_current_context_id(&self) -> ContextId {
        self.lexical_context_stack.last().expect("Expected at least one context").context_id
    }
}

impl<'ctx, 'ast, T> AstVisitEmitter<'ctx, 'ast, T> for Resolver<'ctx, 'ast> where T: AstState {
    /* Methods used during all passes */
    fn set_def_id_to_global_mem(&mut self, def_id: DefId) {
        let ty = <Resolver<'_, '_> as AstVisitEmitter<'_, '_, T>>::get_ty_from_def_id(self, def_id);
        let global_mem_id = GlobalMemId(self.global_mems.borrow().len() as u32);
        let global_mem = GlobalMem::new(global_mem_id, def_id, Span::dummy(), ty);
        self.global_mems.borrow_mut().push(global_mem);
        self.def_id_to_global_mem_id.insert(def_id, global_mem_id);
    }
    fn get_ty_from_node_id(&self, node_id: NodeId) -> Ty {
        let ty = self.node_id_to_ty.get(&node_id).expect("Expected type to node id");
        *ty
    }
    fn report_error(&mut self, error: Error) {
        self.errors.push(error);
    }
    fn is_main_scope(&mut self) -> bool {
        self.lexical_context_stack.len() == 1
    }
    fn append_fn(&mut self, fn_item: &'ast FnItem<'ast>) {
        self.pending_functions.push(fn_item);
    }

    fn set_main_fn(&mut self, main_fn: &'ast FnItem<'ast>) -> bool {
        if self.found_main_fn.is_none() {
            self.found_main_fn = Some(main_fn);
            true
        } else {
            false
        }
    }

    fn alloc_vec<K>(&self, vec: Vec<K>) -> &'ctx [K] {
        self.arena.alloc_slice_fill_iter(vec.into_iter())
    }
    /* Used during the first pass (name resolution) */
    fn start_scope(&mut self) {
        let current_context_id = self.get_current_context_id();
        self.lexical_context_stack.push(
            LexicalContext::new(current_context_id, self.next_scope_id)
        );
        self.next_scope_id = ScopeId(self.next_scope_id.0 + 1);
    }
    fn end_scope(&mut self) {
        self.lexical_context_stack.pop();
    }
    fn start_context(&mut self) {
        self.lexical_context_stack.push(
            LexicalContext::new(self.next_context_id, self.next_scope_id)
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

    fn make_def_id(&mut self, node_id: NodeId, symbol: Symbol) -> DefId {
        let def_id = DefId::new(symbol, node_id);
        self.node_id_to_def_id.insert(node_id, def_id);
        def_id
    }
    fn set_namebinding_to_def_id(&mut self, def_id: DefId, name_binding: NameBinding<'ctx>) {
        self.def_id_to_name_binding.insert(def_id, name_binding);
    }
    fn set_def_id_to_node_id(&mut self, node_id: NodeId, def_id: DefId) {
        self.node_id_to_def_id.insert(node_id, def_id);
    }

    fn bind_def_id_to_lexical_binding(&mut self, def_id: DefId, res_kind: ResKind) {
        let prev = self.lexical_binding_to_def_id
            .iter()
            .find(|(binding, exisiting_def_id)| (
                if
                    exisiting_def_id.symbol.get() == def_id.symbol.get() &&
                    binding.res_kind == res_kind &&
                    res_kind == ResKind::Adt
                {
                    true
                } else {
                    false
                }
            ));

        if let Some(_prev) = prev {
            panic!("Already exists");
        }

        let lexical_binding = LexicalBinding {
            lexical_context: LexicalContext::new(
                self.get_current_context_id(),
                self.get_current_scope_id()
            ),
            res_kind,
            symbol: def_id.symbol,
        };
        self.lexical_binding_to_def_id.insert(lexical_binding, def_id);
    }

    fn lookup_ident_declaration(&mut self, span: Span, res_kind: ResKind) -> Option<DefId> {
        let symbol = Symbol::new(&self.src[span.get_byte_range()]);

        match res_kind {
            ResKind::Variable => {
                let start_context = self.get_current_context_id();
                // In the future, when allowing constants, the following should be true for a constant:
                // A constant is only available after its definition, however its available at any context level
                for lexical_context in self.lexical_context_stack.iter().rev() {
                    if lexical_context.context_id != start_context {
                        break;
                    }
                    let lexical_binding = LexicalBinding::new(*lexical_context, symbol, res_kind);
                    if let Some(def_id) = self.lexical_binding_to_def_id.get(&lexical_binding) {
                        return Some(*def_id);
                    }
                }
            }
            ResKind::Adt | ResKind::Fn => {
                for lexical_context in self.lexical_context_stack.iter().rev() {
                    let lexical_binding = LexicalBinding::new(*lexical_context, symbol, res_kind);
                    if let Some(def_id) = self.lexical_binding_to_def_id.get(&lexical_binding) {
                        return Some(*def_id);
                    }
                }
            }
        }

        None
    }

    fn lookup_ident_definition(
        &mut self,
        span: Span,
        res_kind: ResKind
    ) -> Option<(DefId, NameBinding<'ctx>)> {
        let def_id = <Resolver<'ctx, 'ast> as AstVisitEmitter<'_, '_, T>>::lookup_ident_declaration(
            self,
            span,
            res_kind
        );

        match def_id {
            Some(def_id) => {
                let name_binding = self.def_id_to_name_binding
                    .get(&def_id)
                    .expect("Expected DefId to NameBinding");
                Some((def_id, *name_binding))
            }
            None => None,
        }
    }

    /* Used during the second pass (type checking) */
    fn set_type_to_node_id(&mut self, node_id: NodeId, ty: Ty) {
        self.node_id_to_ty.insert(node_id, ty);
    }
    fn intern_type(&mut self, ty: Ty) -> &'ctx Ty {
        TyCtx::intern_type(ty)
    }
    fn get_def_id_from_node_id(&self, node_id: NodeId) -> DefId {
        *self.node_id_to_def_id.get(&node_id).expect("Expected DefId")
    }
    fn get_ty_from_def_id(&self, def_id: DefId) -> Ty {
        let ty = self.node_id_to_ty.get(&def_id.node_id).expect("Expected type to def id");
        *ty
    }
    fn get_namebinding_from_def_id(&self, def_id: DefId) -> NameBinding<'ctx> {
        let name_binding = self.def_id_to_name_binding
            .get(&def_id)
            .expect("Expected name to be binded");
        *name_binding
    }
    fn borrow_def_id_to_name_binding(&self) -> &DefIdToNameBinding<'ctx> {
        &self.def_id_to_name_binding
    }
}
