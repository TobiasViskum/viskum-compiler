use ast::{
    ast_state::{ AstResolved, AstState, AstState1, AstTypeChecked, AstUnvalidated },
    ast_visitor::AstVisitEmitter,
    Ast,
    IdentExpr,
    IdentPat,
    // AstNodeKind,
    // AstVisitEvent,
    // ScopeChange,
};
use error::{ Error, ErrorKind };
use fxhash::FxHashMap;
use symbol::Symbol;
use ty::{ NodeId, Ty, TyCtx };

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
struct UniqueId(u32);

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
struct DefId {
    symbol: Symbol,
    ast_node_id: NodeId,
}

impl DefId {
    pub fn new(symbol: Symbol, ast_node_id: NodeId) -> Self {
        Self { symbol, ast_node_id }
    }
}

struct NameBinding {
    kind: NameBindingKind,
    scope_id: usize,
}

enum NameBindingKind {
    Res(Res),
    // Module
    // Import
}

enum Res {
    Def(DefId, Def),
    PrimTy,
}

struct Def {}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
struct ScopeId(pub u32);

pub struct Resolver<'a> {
    /// Type context (used during typechecking)
    ty_ctx: TyCtx<'a>,
    def_id_to_ty: FxHashMap<DefId, &'a Ty>,
    def_id_to_name_binding: FxHashMap<DefId, NameBinding>,
    node_id_to_def_id: FxHashMap<NodeId, DefId>,
    symbol_and_scope_to_def_id: FxHashMap<(Symbol, ScopeId), DefId>,
    scope_stack: Vec<ScopeId>,
    next_scope_id: ScopeId,
    src: &'a str,
    errors: Vec<Error>,
}

impl<'a, T> AstVisitEmitter<'a, T> for Resolver<'a> where T: AstState {
    /* Used during the first pass (name resolution) */
    fn start_scope(&mut self) {
        self.scope_stack.push(self.next_scope_id);
        self.next_scope_id = ScopeId(self.next_scope_id.0 + 1);
    }
    fn end_scope(&mut self) {
        self.scope_stack.pop();
    }
    fn define_var(&mut self, ident_pat: &'a IdentPat) {
        let symbol = Symbol::new(ident_pat.get_lexeme(self.src));
        let def_id = DefId::new(symbol, ident_pat.ast_node_id);
        self.symbol_and_scope_to_def_id.insert((symbol, self.get_current_scope_id()), def_id);
        self.node_id_to_def_id.insert(ident_pat.ast_node_id, def_id);
    }
    fn lookup_var(&mut self, ident_expr: &'a IdentExpr) {
        let symbol = Symbol::new(ident_expr.get_lexeme(&self.src));
        for scope_id in self.scope_stack.iter().rev() {
            if self.symbol_and_scope_to_def_id.get(&(symbol, *scope_id)).is_some() {
                return;
            }
        }
        self.errors.push(Error::new(ErrorKind::UndefinedVariable(symbol), ident_expr.span))
    }

    /* Used during the second pass (type checking) */
    fn set_type_to_node(&mut self, node_id: NodeId, ty: Ty) -> &'a Ty {
        self.ty_ctx.set_type_to_node(node_id, ty)
    }
    fn get_ty_from_node_id(&self, node_id: NodeId) -> Option<&'a Ty>
        where T: AstState<ThisState = ast::ast_state::AstState2>
    {
        if let Some(def_id) = self.node_id_to_def_id.get(&node_id) {
            if let Some(ty) = self.def_id_to_ty.get(def_id) { Some(ty) } else { None }
        } else {
            None
        }
    }
}

impl<'a> Resolver<'a> {
    pub fn new(src: &'a str) -> Self {
        let mut scope_stack = Vec::with_capacity(16);
        scope_stack.push(ScopeId(0));

        Self {
            ty_ctx: TyCtx::new(),
            def_id_to_ty: FxHashMap::default(),
            def_id_to_name_binding: FxHashMap::default(),
            node_id_to_def_id: FxHashMap::default(),
            symbol_and_scope_to_def_id: FxHashMap::default(),
            scope_stack,
            next_scope_id: ScopeId(1),
            src,
            errors: Vec::new(),
        }
    }

    /// Performs name resolution
    pub fn resolve_ast(&mut self, ast: Ast<'a, AstUnvalidated>) -> Ast<'a, AstResolved> {
        let ast_visitor = ast.get_visitor(self);

        let resolved_ast = ast_visitor.visit();

        if self.has_errors() {
            self.report_errors();
        }

        resolved_ast
    }

    /// Performs type checking
    pub fn type_check_ast(&mut self, ast: Ast<'a, AstResolved>) -> Ast<'a, AstTypeChecked> {
        let ast_visitor = ast.get_visitor(self);

        let type_checked_ast = ast_visitor.visit();

        if self.has_errors() {
            self.report_errors();
        }

        type_checked_ast
    }

    // fn do_scope_change(&mut self, scope_change: ScopeChange) {
    //     match scope_change {
    //         ScopeChange::Increment => self.start_scope(),
    //         ScopeChange::Decrement => self.end_scope(),
    //     }
    // }

    // fn visit_ast_node<T>(
    //     &mut self,
    //     ast_node_kind: AstNodeKind<'a, T>,
    //     symbol_and_scope_to_def_id: &mut FxHashMap<(Symbol, ScopeId), DefId>
    // )
    //     where T: AstState
    // {
    //     match ast_node_kind {
    //         AstNodeKind::Define(ident) => {
    //             let symbol = Symbol::new(ident.get_lexeme(self.src));

    //             let def_id = DefId::new(symbol, ident.ast_node_id);
    //             symbol_and_scope_to_def_id.insert((symbol, self.get_current_scope_id()), def_id);
    //             self.ast_node_id_to_def_id.insert(ident.ast_node_id, def_id);
    //         }
    //         AstNodeKind::Lookup(ident) => {
    //             let symbol = Symbol::new(ident.get_lexeme(&self.src));

    //             for scope_id in self.scope_stack.iter().rev() {
    //                 if symbol_and_scope_to_def_id.get(&(symbol, *scope_id)).is_some() {
    //                     return;
    //                 }
    //             }
    //             self.errors.push(Error::new(ErrorKind::UndefinedVariable(symbol), ident.span))
    //         }
    //     }
    // }

    fn has_errors(&self) -> bool {
        self.errors.len() > 0
    }

    fn report_errors(&self) -> ! {
        let mut buffer = String::with_capacity(2048);

        for error in self.errors.iter() {
            error.write_msg(&mut buffer);
        }

        println!("{}", buffer);

        std::process::exit(1)
    }

    fn get_current_scope_id(&self) -> ScopeId {
        *self.scope_stack.last().expect("Expected at least one scope")
    }
}
