use std::marker::PhantomData;

use crate::{
    ast_pre_resolver::AstPreResolver,
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
    PkgIdentNode,
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

/// This can call functions on the Resolver struct in the resolver crate,
/// which also implements this trait
///
/// The reason it's not using the Resolver directly is to avoid cyclic references
pub trait ResolverHandle<'ctx, 'ast, T> where T: AstState {
    /* Methods available during all passes  */
    fn report_error(&self, error: Error);

    fn lookup_pkg_member(&self, symbol: Symbol) -> Option<DefId>;
    fn lookup_pkg_member_name_binding(&self, def_id: &DefId) -> Option<&NameBinding<'ctx>>;
    fn lookup_pkg_member_ty(&self, def_id: &DefId) -> Option<Ty>;
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
