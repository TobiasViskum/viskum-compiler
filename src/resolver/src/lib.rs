use std::{ cell::RefCell, path::Path };

use ast::{
    Ast,
    AstMetadata,
    AstPartlyResolved,
    AstQueryEntry,
    AstResolved,
    AstState,
    AstState0,
    AstTypeChecked,
    AstUnvalidated,
    FnItem,
    ResolverHandle,
    StringExpr,
};
use bumpalo::Bump;
use error::{ Error, Severity };
use fxhash::{ FxBuildHasher, FxHashMap };
use ir::{
    ConstStrLen,
    ContextId,
    DefId,
    DefIdToNameBinding,
    GlobalMem,
    GlobalMemId,
    ImplId,
    LexicalBinding,
    LexicalContext,
    ModId,
    NameBinding,
    NameBindingKind,
    NodeId,
    ResKind,
    ResolvedInformation,
    ScopeId,
    Symbol,
    TraitImplId,
    Ty,
    TyCtx,
};
use span::Span;

/// Main resolver struct. This is responsible for validating the Ast
pub struct Resolver<'ctx, 'ast> {
    lexical_binding_to_def_id: FxHashMap<LexicalBinding, DefId>,
    next_mod_id: ModId,
    node_id_to_lexical_context: FxHashMap<NodeId, LexicalContext>,

    mod_name_to_mod_id: FxHashMap<Symbol, ModId>,

    def_id_to_impl_id: FxHashMap<TraitImplId, Vec<DefId>>,

    /// Built during the first pass (name resolution) and then used in the rest of the passes
    node_id_to_def_id: FxHashMap<NodeId, DefId>,

    def_id_to_name_binding: FxHashMap<DefId, NameBinding<'ctx>>,
    node_id_to_ty: FxHashMap<NodeId, Ty>,

    def_id_to_global_mem_id: FxHashMap<DefId, GlobalMemId>,
    global_mems: &'ctx RefCell<Vec<GlobalMem>>,

    found_main_fn: Option<&'ast FnItem<'ast>>,
    pending_functions: Vec<&'ast FnItem<'ast>>,
    /// This is all const strings
    str_symbol_to_def_id: FxHashMap<Symbol, (DefId, ConstStrLen)>,

    /* Arena */
    arena: &'ctx Bump,

    /* Used for error reporting */
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
                const_strs: self.str_symbol_to_def_id.into_values().collect(),
            },
        )
    }

    pub fn new(arena: &'ctx Bump, global_mems: &'ctx RefCell<Vec<GlobalMem>>) -> Self {
        // macro_rules! hashmap_with_capacity {
        //     ($capacity:expr) => {
        //         FxHashMap::with_capacity_and_hasher(
        //             $capacity,
        //             FxBuildHasher::default()
        //         )
        //     };
        // }

        Self {
            global_mems,
            next_mod_id: ModId(0),
            mod_name_to_mod_id: Default::default(),
            def_id_to_global_mem_id: Default::default(),
            def_id_to_name_binding: Default::default(),
            lexical_binding_to_def_id: Default::default(),
            node_id_to_lexical_context: Default::default(),
            str_symbol_to_def_id: Default::default(),
            def_id_to_impl_id: Default::default(),
            node_id_to_def_id: FxHashMap::default(),
            node_id_to_ty: FxHashMap::default(),
            arena,
            found_main_fn: None,
            pending_functions: Vec::new(),

            errors: Default::default(),
        }
    }

    pub fn resolve_all_modules(&mut self, entry_dir: &Path, entry_file: &Path) {
        let mod_name = Symbol::new(entry_file.file_stem().unwrap().to_str().unwrap());
        let mod_id = self.get_or_make_mod_id_from_mod_name(mod_name);

        let file_content = entry_dir.join(entry_file);

        println!("Entry dir: {:?}", entry_dir);
        println!("Entry file: {:?}, {}", entry_file, mod_name.get());
    }

    pub fn forward_declare_ast(
        &mut self,
        ast: Ast<'ast, AstUnvalidated>
    ) -> (Ast<'ast, AstPartlyResolved>, FxHashMap<LexicalContext, LexicalContext>) {
        let ast_visitor = ast.into_visitor(self);

        let visit_result = ast_visitor.visit();

        if self.has_errors() {
            self.exit_if_has(Severity::Fatal);
        }

        return visit_result;
    }

    /// Performs name resolution
    pub fn resolve_ast(
        &mut self,
        ast: Ast<'ast, AstPartlyResolved>,
        lexical_context_to_parent_lexical_context: &FxHashMap<LexicalContext, LexicalContext>
    ) -> Ast<'ast, AstResolved> {
        let ast_visitor = ast.into_visitor(self, lexical_context_to_parent_lexical_context);

        let resolved_ast = ast_visitor.visit();

        if self.has_errors() {
            self.exit_if_has(Severity::Fatal);
        }

        return resolved_ast;
    }

    /// Performs type checking
    pub fn type_check_ast(
        &mut self,
        ast: Ast<'ast, AstResolved>,
        lexical_context_to_parent_lexical_context: &FxHashMap<LexicalContext, LexicalContext>
    ) -> Ast<'ast, AstTypeChecked> {
        let ast_visitor = ast.into_visitor(self, lexical_context_to_parent_lexical_context);

        let type_checked_ast = ast_visitor.visit();

        if self.has_errors() {
            self.exit_if_has(Severity::NoImpact);
        }

        // self.assert_type_to_all_nodes(&type_checked_ast);

        type_checked_ast
    }

    // fn assert_type_to_all_nodes(&self, type_checked_ast: &Ast<'ast, AstTypeChecked>) {
    //     // It has previously been asserted, that all nodes is inserted into the query system
    //     // So therefore this is also going to assert the condition for every node
    //     type_checked_ast.query_all(|node_id, query_entry| {
    //         let str = match query_entry {
    //             AstQueryEntry::IdentNode(ident_node) =>
    //                 Some(&self.src[ident_node.span.get_byte_range()]),
    //             _ => None,
    //         };

    //         assert_eq!(
    //             true,
    //             self.node_id_to_ty.get(node_id).is_some(),
    //             "Expected all nodes to have a type. Node {} is missing one. Details:\nName: {:?}\n\nMore:\n{:?}",
    //             node_id.node_id,
    //             str,
    //             query_entry
    //         )
    //     });
    // }

    fn has_errors(&self) -> bool {
        self.errors.len() > 0
    }

    fn get_or_make_mod_id_from_mod_name(&mut self, mod_name: Symbol) -> ModId {
        if let Some(mod_id) = self.mod_name_to_mod_id.get(&mod_name) {
            return *mod_id;
        }

        let mod_id = self.next_mod_id;
        self.next_mod_id = ModId(mod_id.0 + 1);
        self.mod_name_to_mod_id.insert(mod_name, mod_id);
        mod_id
    }

    fn print_errors(&self) {
        let mut buffer = String::with_capacity(2048);

        for error in self.errors.iter() {
            error.write_msg(&mut buffer);
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
}

impl<'ctx, 'ast, T> ResolverHandle<'ctx, 'ast, T> for Resolver<'ctx, 'ast> where T: AstState {
    /* Methods used during all passes */
    fn compile_rel_file(&mut self, path: ast::Path<'ast>) -> Result<u32, String> {
        todo!()
    }
    fn set_node_id_to_lexical_context(&mut self, node_id: NodeId, lexical_context: LexicalContext) {
        self.node_id_to_lexical_context.insert(node_id, lexical_context);
    }

    fn try_get_def_id_from_trait_impl_id(
        &self,
        trait_impl_id: TraitImplId,
        symbol: Symbol
    ) -> Option<DefId> {
        let def_ids = self.def_id_to_impl_id.get(&trait_impl_id)?;
        def_ids
            .iter()
            .find(|def_id| def_id.symbol.get() == symbol.get())
            .copied()
    }
    fn set_def_id_to_global_mem(&mut self, def_id: DefId) {
        let global_mem_id = GlobalMemId(self.global_mems.borrow().len() as u32);
        let global_mem = GlobalMem::new(global_mem_id, def_id, Span::dummy());
        self.global_mems.borrow_mut().push(global_mem);
        self.def_id_to_global_mem_id.insert(def_id, global_mem_id);
    }
    fn get_ty_from_node_id(&self, node_id: NodeId) -> Ty {
        println!("Node id: {:?}", node_id);
        *self.node_id_to_ty.get(&node_id).expect("Expected type to node id")
    }
    fn get_mut_or_create_def_ids_from_trait_impl_id(
        &mut self,
        trait_impl_id: TraitImplId
    ) -> &mut Vec<DefId> {
        self.def_id_to_impl_id.entry(trait_impl_id).or_insert(Vec::new())
    }

    fn make_const_str(&mut self, str_expr: &'ast StringExpr) -> DefId {
        let symbol = Symbol::from_node_id(str_expr.ast_node_id);
        if let Some(&(def_id, _)) = self.str_symbol_to_def_id.get(&symbol) {
            self.node_id_to_def_id.insert(str_expr.ast_node_id, def_id);
            return def_id;
        }

        let def_id = <Resolver<'ctx, 'ast> as ResolverHandle<
            '_,
            '_,
            T
        >>::make_def_id_and_bind_to_node_id(self, str_expr.ast_node_id, symbol);

        self.node_id_to_def_id.insert(str_expr.ast_node_id, def_id);
        self.def_id_to_name_binding.insert(
            def_id,
            NameBinding::new(NameBindingKind::ConstStr(ConstStrLen(str_expr.len as u32)))
        );
        self.str_symbol_to_def_id.insert(symbol, (def_id, ConstStrLen(str_expr.len as u32)));
        <Resolver<'ctx, 'ast> as ResolverHandle<'_, '_, T>>::set_def_id_to_global_mem(self, def_id);
        def_id
    }
    fn report_error(&mut self, error: Error) {
        self.errors.push(error);
    }
    fn is_main_scope(&mut self) -> bool {
        true
    }
    fn append_fn(&mut self, fn_item: &'ast FnItem<'ast>) {
        self.pending_functions.push(fn_item);
    }
    fn append_comp_decl(&mut self, comp_fn_decl: ast::CompDeclItem<'ast>) {
        panic!()
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

    fn make_def_id_and_bind_to_node_id(&mut self, node_id: NodeId, symbol: Symbol) -> DefId {
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
    fn try_get_def_id_from_node_id(&self, node_id: NodeId) -> Option<DefId> {
        self.node_id_to_def_id.get(&node_id).copied()
    }

    fn bind_def_id_to_lexical_binding(&mut self, def_id: DefId, res_kind: ResKind) {
        let lexical_context = self.node_id_to_lexical_context
            .get(&def_id.node_id)
            .expect(format!("Expected lexical context: {}", def_id.symbol.get()).as_str());

        let mock_lexical_binding = LexicalBinding::new(
            *lexical_context,
            def_id.symbol,
            res_kind,
            def_id.node_id.mod_id
        );
        if let Some(_prev) = self.lexical_binding_to_def_id.get(&mock_lexical_binding) {
            panic!("Adt or Fn already exists: {:?}", def_id.symbol.get());
        }

        let lexical_binding = LexicalBinding {
            lexical_context: *lexical_context,
            res_kind,
            symbol: def_id.symbol,
            mod_id: def_id.node_id.mod_id,
        };
        self.lexical_binding_to_def_id.insert(lexical_binding, def_id);
    }

    fn lookup_ident_declaration(
        &mut self,
        symbol: Symbol,
        res_kind: ResKind,
        node_id: NodeId,
        lexical_context_to_parent_lexical_context: &FxHashMap<LexicalContext, LexicalContext>
    ) -> Option<DefId> {
        match res_kind {
            ResKind::ConstStr => {
                if let Some(&(def_id, _)) = self.str_symbol_to_def_id.get(&symbol) {
                    return Some(def_id);
                }
            }
            ResKind::Variable => {
                let start_context = self.node_id_to_lexical_context
                    .get(&node_id)
                    .expect(
                        format!(
                            "Expected lexical context: {}\n{:#?}",
                            symbol.get(),
                            node_id
                        ).as_str()
                    );

                let mut current_context = *start_context;
                loop {
                    // Can't lookup variables in other contexts (e.g. outside of a function)
                    if current_context.context_id != start_context.context_id {
                        break;
                    }

                    let lexical_binding = LexicalBinding::new(
                        current_context,
                        symbol,
                        res_kind,
                        node_id.mod_id
                    );
                    if let Some(def_id) = self.lexical_binding_to_def_id.get(&lexical_binding) {
                        return Some(*def_id);
                    }
                    if
                        let Some(parent_context) = lexical_context_to_parent_lexical_context.get(
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
                let start_context = self.node_id_to_lexical_context
                    .get(&node_id)
                    .expect(
                        format!(
                            "Expected lexical context: {}\n{:#?}",
                            symbol.get(),
                            node_id
                        ).as_str()
                    );

                let mut current_context = *start_context;
                loop {
                    let lexical_binding = LexicalBinding::new(
                        current_context,
                        symbol,
                        res_kind,
                        node_id.mod_id
                    );
                    if let Some(def_id) = self.lexical_binding_to_def_id.get(&lexical_binding) {
                        return Some(*def_id);
                    }
                    if
                        let Some(parent_context) = lexical_context_to_parent_lexical_context.get(
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

    fn lookup_ident_definition(
        &mut self,
        symbol: Symbol,
        res_kind: ResKind,
        node_id: NodeId,
        lexical_context_to_parent_lexical_context: &FxHashMap<LexicalContext, LexicalContext>
    ) -> Option<(DefId, NameBinding<'ctx>)> {
        let def_id = <Resolver<'ctx, 'ast> as ResolverHandle<'_, '_, T>>::lookup_ident_declaration(
            self,
            symbol,
            res_kind,
            node_id,
            lexical_context_to_parent_lexical_context
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
            .expect(format!("Expected name to be binded: {}", def_id.symbol.get()).as_str());
        *name_binding
    }
    fn borrow_def_id_to_name_binding(&self) -> &DefIdToNameBinding<'ctx> {
        &self.def_id_to_name_binding
    }
}
