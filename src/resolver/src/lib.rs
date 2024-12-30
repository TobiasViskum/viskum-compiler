use std::sync::{ Mutex, OnceLock };

use ast::{ AstState, FnItem, ResolverHandle, StringExpr };
use bumpalo::Bump;
use error::{ Error, Severity };
use fxhash::{ FxBuildHasher, FxHashMap };
use ir::{
    ConstStrLen,
    DefId,
    LexicalBinding,
    LexicalContext,
    NameBinding,
    NodeId,
    ResKind,
    ResolvedInformation,
    Symbol,
    TraitImplId,
    Ty,
    PKG_SYMBOL,
};

/// Main resolver struct. This is responsible for validating all the Asts in a package
pub struct Resolver<'ctx, 'ast> {
    // Package information
    pkg_symbol_to_def_id: FxHashMap<Symbol, DefId>,
    pkg_def_id_to_res_kind: FxHashMap<DefId, ResKind>,
    pkg_def_id_to_name_binding: FxHashMap<DefId, NameBinding<'ctx>>,
    pkg_trait_impl_id_to_def_ids: FxHashMap<TraitImplId, Vec<DefId>>,
    pkg_def_id: OnceLock<DefId>,

    node_id_to_def_id: FxHashMap<NodeId, DefId>,
    node_id_to_ty: FxHashMap<NodeId, Ty>,
    def_id_to_name_binding: FxHashMap<DefId, NameBinding<'ctx>>,

    /// Replace with `OnceLock<&'ast FnItem<'ast>>`
    found_main_fn: OnceLock<&'ast FnItem<'ast>>,
    pending_functions: Vec<&'ast FnItem<'ast>>,

    /// This is all const strings
    str_symbol_to_def_id: Mutex<FxHashMap<Symbol, (DefId, ConstStrLen)>>,
    constants: Vec<DefId>,

    clib_fns: Vec<DefId>,

    /* Arena */
    // arena: &'ctx Bump,

    /* Used for error reporting */
    errors: Mutex<Vec<Error>>,
}

pub struct LocalNodeId {
    pub node_id: u32,
}

pub struct LocalDefId {
    pub symbol: Symbol,
    pub def_id: u32,
}

pub struct LocalTraitImplId {
    pub implementor_def_id: LocalDefId,
    pub trait_def_id: Option<LocalDefId>,
}

// pub struct ModuleResolver<'ctx, 'ast> {
//     lexical_binding_to_def_id: FxHashMap<LexicalBinding, LocalDefId>,
//     node_id_to_lexical_context: FxHashMap<LocalNodeId, LexicalContext>,
//     def_id_to_impl_id: FxHashMap<TraitImplId, Vec<LocalDefId>>,
//     node_id_to_def_id: FxHashMap<LocalNodeId, LocalDefId>,
//     def_id_to_name_binding: FxHashMap<LocalDefId, NameBinding<'ctx>>,
// }

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
                main_fn: self.found_main_fn.get().map(|v| &**v),
                pending_functions: self.pending_functions,
            },
            ResolvedInformation {
                node_id_to_def_id: self.node_id_to_def_id,
                node_id_to_ty: self.node_id_to_ty,
                def_id_to_name_binding: self.def_id_to_name_binding,
                // def_id_to_global_mem_id: self.def_id_to_global_mem_id,
                const_strs: self.str_symbol_to_def_id.into_inner().unwrap().into_values().collect(),
                clib_fns: self.clib_fns,
            },
        )
    }

    pub fn new(
        _arena: &'ctx Bump,
        total_nodes: usize,
        total_def_count: usize
        // global_mems: &'ctx RefCell<Vec<GlobalMem>>
    ) -> Self {
        macro_rules! hashmap_with_capacity {
            ($capacity:expr) => {
                FxHashMap::with_capacity_and_hasher(
                    $capacity,
                    FxBuildHasher::default()
                )
            };
        }

        Self {
            def_id_to_name_binding: hashmap_with_capacity!(total_def_count),
            str_symbol_to_def_id: Default::default(),

            pkg_def_id_to_name_binding: Default::default(),
            pkg_def_id_to_res_kind: Default::default(),
            pkg_symbol_to_def_id: Default::default(),
            pkg_trait_impl_id_to_def_ids: Default::default(),
            pkg_def_id: OnceLock::new(),
            constants: Vec::new(),

            node_id_to_def_id: hashmap_with_capacity!(total_nodes),
            node_id_to_ty: hashmap_with_capacity!(total_nodes),
            // arena,
            found_main_fn: OnceLock::new(),
            pending_functions: Vec::new(),
            clib_fns: Vec::new(),
            errors: Default::default(),
        }
    }

    pub fn use_visit_result_from_pre_resolve(
        &mut self,
        global_visit_result: ast::ast_pre_resolver::GlobalVisitResult
    ) {
        for symbol in global_visit_result.pkg_symbol_to_def_id.keys() {
            if self.pkg_symbol_to_def_id.contains_key(symbol) {
                panic!("Symbol in package already exists: {:?}", symbol.get());
            }
        }

        self.pkg_symbol_to_def_id.extend(global_visit_result.pkg_symbol_to_def_id);
        self.pkg_def_id_to_res_kind.extend(global_visit_result.pkg_def_id_to_res_kind);
    }

    pub fn use_visit_result_from_resolve(
        &mut self,
        global_visit_result: ast::ast_resolver::GlobalVisitResult<'ctx, 'ast>
    ) {
        self.pending_functions.extend(global_visit_result.fns);
        self.clib_fns.extend(global_visit_result.clib_fns);
        self.pkg_def_id_to_name_binding.extend(global_visit_result.pkg_def_id_to_name_binding);

        for (
            trait_impl_id,
            new_def_ids,
        ) in global_visit_result.trait_impl_id_to_def_ids.into_iter() {
            let def_ids = self.pkg_trait_impl_id_to_def_ids.entry(trait_impl_id).or_default();

            def_ids.extend(new_def_ids);
        }
    }

    pub fn use_visit_result_from_type_check(
        &mut self,
        global_visit_result: ast::ast_type_checker::GlobalVisitResult<'ctx>
    ) {
        self.node_id_to_ty.extend(global_visit_result.node_id_to_type);
        self.def_id_to_name_binding.extend(global_visit_result.def_id_to_name_binding);
        self.node_id_to_def_id.extend(global_visit_result.node_id_to_def_id);
    }

    fn has_errors(&self) -> bool {
        self.errors.lock().unwrap().len() > 0
    }

    fn print_errors(&self) {
        let mut buffer = String::with_capacity(2048);

        let errors = self.errors.lock().unwrap();

        for error in errors.iter() {
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
        let errors = self.errors.lock().unwrap();

        errors.iter().any(|e| e.get_severity() == severity)
    }
}

impl<'ctx, 'ast, T> ResolverHandle<'ctx, 'ast, T> for Resolver<'ctx, 'ast> where T: AstState {
    /* Methods used during all passes */

    fn lookup_pkg_member(&self, symbol: Symbol) -> Option<DefId> {
        self.pkg_symbol_to_def_id.get(&symbol).copied()
    }
    fn get_or_set_pkg_def_id(&self, pkg_ident_node: &'ast ast::PkgIdentNode) -> DefId {
        *self.pkg_def_id.get_or_init(|| DefId::new(*PKG_SYMBOL, pkg_ident_node.ast_node_id))
    }

    fn lookup_pkg_member_res_kind(&self, def_id: &DefId) -> ResKind {
        *self.pkg_def_id_to_res_kind.get(def_id).expect("Expected ResKind")
    }
    fn lookup_pkg_member_name_binding(&self, def_id: &DefId) -> Option<&NameBinding<'ctx>> {
        self.pkg_def_id_to_name_binding.get(def_id)
    }
    fn lookup_trait_impl_def_ids(&self, trait_impl_id: &TraitImplId) -> Option<&Vec<DefId>> {
        self.pkg_trait_impl_id_to_def_ids.get(trait_impl_id)
    }

    fn set_main_fn(&self, fn_item: &'ast FnItem<'ast>) -> bool {
        self.found_main_fn.set(fn_item).is_ok()
    }

    fn make_const_str(
        &self,
        str_expr: &'ast StringExpr,
        mut make_def_id: impl FnMut() -> DefId
    ) -> DefId {
        let mut str_symbol_to_def_id = self.str_symbol_to_def_id.lock().unwrap();

        let symbol = Symbol::from_node_id(str_expr.ast_node_id);
        if let Some(&(def_id, _)) = str_symbol_to_def_id.get(&symbol) {
            make_def_id();
            return def_id;
        }

        let def_id = make_def_id();

        str_symbol_to_def_id.insert(symbol, (def_id, ConstStrLen(str_expr.len as u32)));

        def_id
    }
    fn report_error(&self, error: Error) {
        self.errors.lock().unwrap().push(error);
    }
}
