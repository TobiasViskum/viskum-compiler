use crate::{ ast_state::AstState, FnItem, PkgIdentNode, StringExpr };
use error::Error;
use fxhash::FxHashMap;
use ir::{ DefId, LexicalContext, NameBinding, NodeId, ResKind, TraitImplId };

use ir::Symbol;

/// This can call functions on the Resolver struct in the resolver crate,
/// which also implements this trait
///
/// The reason it's not using the Resolver directly is to avoid cyclic references
pub trait ResolverHandle<'ctx, 'ast, T> where T: AstState {
    /* Methods available during all passes  */
    fn report_error(&self, error: Error);

    fn lookup_pkg_member(&self, symbol: Symbol) -> Option<DefId>;
    fn lookup_pkg_member_name_binding(&self, def_id: &DefId) -> Option<&NameBinding<'ctx>>;
    fn lookup_pkg_member_res_kind(&self, def_id: &DefId) -> ResKind;
    fn lookup_trait_impl_def_ids(&self, trait_impl_id: &TraitImplId) -> Option<&Vec<DefId>>;
    fn get_or_set_pkg_def_id(&self, pkg_ident_node: &'ast PkgIdentNode) -> DefId;

    // /// Saves the DefId of the extern function in the resolver
    // fn mark_as_extern_fn(&mut self, def_id: DefId);
    /// Makes a new DefId and binds it to the given NodeId
    fn lookup_ident_declaration(
        &mut self,
        symbol: Symbol,
        res_kind: ResKind,
        node_id: NodeId,
        lexical_context_to_parent_lexical_context: &FxHashMap<LexicalContext, LexicalContext>
    ) -> Option<DefId>;
    fn lookup_ident_definition(
        &mut self,
        symbol: Symbol,
        res_kind: ResKind,
        node_id: NodeId,
        lexical_context_to_parent_lexical_context: &FxHashMap<LexicalContext, LexicalContext>
    ) -> Option<(DefId, NameBinding<'ctx>)>;

    fn set_main_fn(&self, fn_item: &'ast FnItem<'ast>) -> bool;

    /// Makes the string global, so all identical strings after this call, will be bound to the same DefId
    ///
    /// NOTICE: This may need some rework later, when I introduce mulitple packages (crates),
    /// since the Resolver only traverses one package,
    /// meaning that the same string may be bound to a different DefId in another package
    fn make_const_str(
        &self,
        str_expr: &'ast StringExpr,
        make_def_id: impl FnMut() -> DefId
    ) -> DefId;
}
