use ast::{
    Ast,
    AstPrettifier,
    AstState,
    AstState0,
    AstState1,
    AstState2,
    AstState3,
    AstTypeChecked,
    AstVisitEmitter,
    IdentNode,
    StructItem,
};
use bumpalo::Bump;
use error::{ Error, ErrorKind, Severity };
use fxhash::FxHashMap;
use ir_defs::{ DefId, DefKind, NameBinding, NodeId, ResKind, ScopeId };
use symbol::Symbol;
use ty::{ Ty, TyCtx };

pub struct ResolvedInformation {
    pub node_id_to_ty: FxHashMap<NodeId, Ty>,
    pub node_id_to_def_id: FxHashMap<NodeId, DefId>,
}

/// Main resolver struct. This is responsible for validating the Ast and creating the Cfg from it
pub struct Resolver<'a> {
    ty_ctx: TyCtx,

    /// Only used during the first pass (name resolution)
    ///
    /// This is reset after the first pass to save some space
    symbol_and_scope_to_def_id: FxHashMap<(Symbol, ScopeId, ResKind), DefId>,
    scope_stack: Vec<ScopeId>,
    next_scope_id: ScopeId,

    /// Built during the first pass (name resolution) and then used in the rest of the passes
    node_id_to_def_id: FxHashMap<NodeId, DefId>,

    def_id_to_name_binding: FxHashMap<DefId, NameBinding<'a>>,
    node_id_to_ty: FxHashMap<NodeId, Ty>,

    /* Arena */
    arena: &'a Bump,

    /* Used for error reporting */
    src: &'a str,
    errors: Vec<Error>,
}

impl<'a> Resolver<'a> {
    pub fn take_resolved_information(self) -> ResolvedInformation {
        if self.has_errors() {
            self.print_errors();
            self.exit_if_has(Severity::NoImpact);
        }

        ResolvedInformation {
            node_id_to_def_id: self.node_id_to_def_id,
            node_id_to_ty: self.node_id_to_ty,
        }
    }

    /// Makes a Resolver and builds the query system in the Ast
    pub fn from_ast(
        src: &'a str,
        ast: Ast<'a, AstState0>,
        arena: &'a Bump
    ) -> (Self, Ast<'a, AstState1>) {
        /// Sets up query system
        fn build_ast_query_system<'a>(
            resolver: &mut Resolver<'a>,
            ast: Ast<'a, AstState0>
        ) -> Ast<'a, AstState1> {
            let ast_visitor = ast.get_visitor(resolver.src, resolver);

            let unvalidated_ast = ast_visitor.visit();

            unvalidated_ast
        }

        let mut scope_stack = Vec::with_capacity(16);
        scope_stack.push(ScopeId(0));

        let mut resolver = Self {
            ty_ctx: Default::default(),
            def_id_to_name_binding: Default::default(),
            node_id_to_def_id: Default::default(),
            symbol_and_scope_to_def_id: Default::default(),
            node_id_to_ty: Default::default(),
            arena,
            scope_stack,
            next_scope_id: ScopeId(1),
            src,
            errors: Default::default(),
        };

        let ast_next_state = build_ast_query_system(&mut resolver, ast);

        (resolver, ast_next_state)
    }

    /// Performs name resolution
    pub fn resolve_ast(&mut self, ast: Ast<'a, AstState1>) -> Ast<'a, AstState2> {
        fn remove_temp_storage<'a>(resolver: &mut Resolver<'a>) {
            resolver.symbol_and_scope_to_def_id = Default::default();
            resolver.scope_stack = vec![];
            resolver.next_scope_id = ScopeId(0);
        }

        let ast_visitor = ast.get_visitor(self.src, self);

        let resolved_ast = ast_visitor.visit();

        if self.has_errors() {
            self.exit_if_has(Severity::Fatal);
        }

        remove_temp_storage(self);

        resolved_ast
    }

    /// Performs type checking
    pub fn type_check_ast(&mut self, ast: Ast<'a, AstState2>) -> Ast<'a, AstState3> {
        let ast_visitor = ast.get_visitor(self.src, self);

        let type_checked_ast = ast_visitor.visit();

        if self.has_errors() {
            self.exit_if_has(Severity::Fatal);
        }

        self.assert_type_to_all_nodes(&type_checked_ast);

        // AstPrettifier::new(&type_checked_ast, self.src, Some(&self.node_id_to_ty)).print_ast();

        type_checked_ast
    }

    fn assert_type_to_all_nodes(&self, type_checked_ast: &Ast<'a, AstTypeChecked>) {
        // It has previously been asserted, that all nodes is inserted into the query system
        // So therefore this is also going to assert the condition for every node
        type_checked_ast.query_all(|node_id, query_entry| {
            assert_eq!(
                true,
                self.node_id_to_ty.get(node_id).is_some(),
                "Expected all nodes to have a type. Node {} is missing one. Details:\n{:?}",
                node_id,
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
        *self.scope_stack.last().expect("Expected at least one scope")
    }
}

impl<'ctx, 'a, T> AstVisitEmitter<'ctx, 'a, T> for Resolver<'a> where T: AstState {
    /* Methods used during all passes */
    fn report_error(&mut self, error: Error) {
        self.errors.push(error);
    }

    fn alloc_vec<K>(&self, vec: Vec<K>) -> &'a [K] {
        self.arena.alloc_slice_fill_iter(vec.into_iter())
    }
    /* Used during the first pass (name resolution) */
    fn start_scope(&mut self) {
        self.scope_stack.push(self.next_scope_id);
        self.next_scope_id = ScopeId(self.next_scope_id.0 + 1);
    }
    fn end_scope(&mut self) {
        self.scope_stack.pop();
    }
    fn define(&mut self, node_id: NodeId, symbol: Symbol, name_binding: NameBinding<'a>) -> DefId
        where T: AstState<ThisState = AstState1>
    {
        let def_id = DefId::new(symbol, node_id);
        self.node_id_to_def_id.insert(node_id, def_id);
        self.def_id_to_name_binding.insert(def_id, name_binding);
        self.symbol_and_scope_to_def_id.insert(
            (symbol, self.get_current_scope_id(), name_binding.get_res_kind()),
            def_id
        );
        def_id
    }

    fn lookup_ident(
        &mut self,
        ident_node: &'a IdentNode,
        kind: ResKind
    ) -> Option<NameBinding<'a>> {
        let symbol = Symbol::new(&self.src[ident_node.span.get_byte_range()]);

        for scope_id in self.scope_stack.iter().rev() {
            if let Some(def_id) = self.symbol_and_scope_to_def_id.get(&(symbol, *scope_id, kind)) {
                self.node_id_to_def_id.insert(ident_node.ast_node_id, *def_id);
                let name_binding = self.def_id_to_name_binding
                    .get(def_id)
                    .expect("Expected name to be binded");
                return Some(*name_binding);
            }
        }

        self.errors.push(Error::new(ErrorKind::UndefinedLookup(symbol, kind), ident_node.span));
        None
    }

    /* Used during the second pass (type checking) */
    fn set_type_to_node_id(&mut self, node_id: NodeId, ty: Ty) {
        self.node_id_to_ty.insert(node_id, ty);
    }
    fn intern_type(&mut self, ty: Ty) -> &'ctx Ty {
        self.ty_ctx.intern_type(ty)
    }
    fn get_def_id_from_node_id(&self, node_id: NodeId) -> DefId {
        *self.node_id_to_def_id.get(&node_id).expect("Expected DefId")
    }
    fn get_ty_from_def_id(&self, def_id: DefId) -> Ty {
        let ty = self.node_id_to_ty.get(&def_id.node_id).expect("Expected type to def id");
        *ty
    }
    fn get_namebinding_from_def_id(&self, def_id: DefId) -> NameBinding<'a> {
        let name_binding = self.def_id_to_name_binding
            .get(&def_id)
            .expect("Expected name to be binded");
        *name_binding
    }
}
