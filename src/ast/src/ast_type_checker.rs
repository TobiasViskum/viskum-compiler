use std::marker::PhantomData;

use error::{ Error, ErrorKind };
use fxhash::FxHashMap;
use ir::{
    Adt,
    DefId,
    HasSelfArg,
    Mutability,
    NameBinding,
    NameBindingKind,
    NodeId,
    Symbol,
    TraitImplId,
    Ty,
    TyCtx,
    BOOL_TY,
    NEVER_TY,
    NULL_TY,
    STR_TY,
    UNKOWN_TY,
    VOID_TY,
};
use span::Span;

use crate::{
    ast_resolver::{ self },
    typechecker::{ ArgCmp, TypeChecker },
    AssignStmt,
    Ast,
    AstResolved,
    BinaryExpr,
    BlockExpr,
    BoolExpr,
    BreakExpr,
    CallExpr,
    CondKind,
    ContinueExpr,
    DefineStmt,
    FieldExpr,
    FnItem,
    GroupExpr,
    IdentNode,
    IfExpr,
    IndexExpr,
    IntegerExpr,
    LoopExpr,
    NullExpr,
    Pat,
    PathField,
    PkgIdentNode,
    ResolverHandle,
    ReturnExpr,
    StringExpr,
    StructExpr,
    TupleExpr,
    TupleFieldExpr,
    TupleStructPat,
    VisitAst,
    Visitor,
};

#[derive(Debug)]
pub struct GlobalVisitResult<'ctx> {
    pub def_id_to_name_binding: FxHashMap<DefId, NameBinding<'ctx>>,
    pub node_id_to_def_id: FxHashMap<NodeId, DefId>,
    pub node_id_to_type: FxHashMap<NodeId, Ty>,
}

#[derive(Debug)]
pub struct LocalVisitResult {}

/// Visits the entire Ast from left to right
///
/// When the visitor is done, the ast will transition into the next state
#[derive(Debug)]
pub struct AstTypeChecker<'ctx, 'ast, 'b, E>
    where E: ResolverHandle<'ctx, 'ast, AstResolved>, 'ctx: 'ast, 'ast: 'b {
    pub ast: Ast<'ast, AstResolved>,
    pub resolver_handle: &'b E,
    node_id_to_type: FxHashMap<NodeId, Ty>,
    def_id_to_name_binding: FxHashMap<DefId, NameBinding<'ctx>>,
    node_id_to_def_id: FxHashMap<NodeId, DefId>,
    // trait_impl_context: Option<TraitImplId>,
    marker: PhantomData<&'ctx ()>,
    /// First option checks if it's present (meain if we're inside a loop or function)
    /// Second one checks whether or not the ty is set
    loop_ret_ty: Option<Option<Ty>>,
    fn_ret_ty: Option<Ty>,
}

impl<'ast, 'ctx, 'c, E> VisitAst<'ast, AstResolved>
    for AstTypeChecker<'ctx, 'ast, 'c, E>
    where E: ResolverHandle<'ctx, 'ast, AstResolved>, 'ctx: 'ast, 'ast: 'c
{
    type GlobalVisitResult = GlobalVisitResult<'ctx>;

    type LocalVisitResult = LocalVisitResult;

    fn visit<N>(mut self) -> (Ast<'ast, N>, Self::GlobalVisitResult, Self::LocalVisitResult)
        where AstResolved: crate::AstState<NextState = N>, N: crate::AstState
    {
        self.visit_stmts(self.ast.main_scope.stmts);
        (
            self.ast.next_state(),
            GlobalVisitResult {
                def_id_to_name_binding: self.def_id_to_name_binding,
                node_id_to_def_id: self.node_id_to_def_id,
                node_id_to_type: self.node_id_to_type,
            },
            LocalVisitResult {},
        )
    }
}

impl<'ast, 'ctx, 'c> Ast<'ast, AstResolved> where 'ctx: 'ast, 'ast: 'c {
    pub fn into_visitor<E>(
        self,
        resolver_handle: &'c E,
        local_visit_result: ast_resolver::LocalVisitResult<'ctx>
    ) -> AstTypeChecker<'ctx, 'ast, 'c, E>
        where E: ResolverHandle<'ctx, 'ast, AstResolved>
    {
        AstTypeChecker::new(self, resolver_handle, local_visit_result)
    }
}

impl<'ctx, 'ast, 'b, E> AstTypeChecker<'ctx, 'ast, 'b, E>
    where E: ResolverHandle<'ctx, 'ast, AstResolved>, 'ctx: 'ast, 'ast: 'b
{
    pub fn new(
        ast: Ast<'ast, AstResolved>,
        resolver_handle: &'b E,
        local_visit_result: ast_resolver::LocalVisitResult<'ctx>
    ) -> Self {
        Self {
            loop_ret_ty: None,

            fn_ret_ty: None,
            resolver_handle,
            node_id_to_type: local_visit_result.node_id_to_ty,
            def_id_to_name_binding: local_visit_result.def_id_to_name_binding,
            node_id_to_def_id: local_visit_result.node_id_to_def_id,

            ast,
            marker: PhantomData,
        }
    }

    // pub fn traverse_pat_and_bind_def_ids_to_variable_namebinding(&mut self, pat: Pat<'ast>) {
    //     match pat {
    //         Pat::IdentPat(ident_node) => {
    //             let def_id = self.resolver_handle.get_def_id_from_node_id(ident_node.ast_node_id);
    //             let name_binding = NameBinding::new(
    //                 NameBindingKind::Variable(Mutability::Immutable)
    //             );
    //             self.resolver_handle.set_namebinding_to_def_id(def_id, name_binding);
    //         }
    //         // Pat::MutIdentPat(ident_node) => {
    //         //     let def_id = self.resolver_handle.get_def_id_from_node_id(ident_node.ast_node_id);
    //         //     let name_binding = NameBinding::new(
    //         //         NameBindingKind::Variable(Mutability::Mutable)
    //         //     );
    //         //     self.resolver_handle.set_namebinding_to_def_id(def_id, name_binding);
    //         // }
    //         Pat::TupleStructPat(tuple_struct_pat) => {
    //             self.visit_path(tuple_struct_pat.path);
    //             for field in tuple_struct_pat.fields.iter() {
    //                 self.traverse_pat_and_bind_def_ids_to_variable_namebinding(*field);
    //             }
    //         }
    //     }
    // }

    fn set_def_id_to_node_id(&mut self, node_id: NodeId, def_id: DefId) {
        self.node_id_to_def_id.insert(node_id, def_id);
    }

    fn set_type_to_node_id(&mut self, node_id: NodeId, ty: Ty) {
        self.node_id_to_type.insert(node_id, ty);
    }

    fn get_def_id_from_node_id(&self, node_id: NodeId) -> DefId {
        self.node_id_to_def_id.get(&node_id).copied().expect("DefId not found")
    }

    fn try_get_def_id_from_node_id(&self, node_id: NodeId) -> Option<DefId> {
        self.node_id_to_def_id.get(&node_id).copied()
    }

    fn try_get_namebinding_from_def_id(&self, def_id: DefId) -> Option<&NameBinding<'ctx>> {
        if let Some(name_binding) = self.def_id_to_name_binding.get(&def_id) {
            Some(name_binding)
        } else if
            let Some(name_binding) = self.resolver_handle.lookup_pkg_member_name_binding(&def_id)
        {
            Some(name_binding)
        } else {
            None
        }
    }

    fn get_namebinding_from_def_id(&self, def_id: DefId) -> &NameBinding<'ctx> {
        self.try_get_namebinding_from_def_id(def_id).expect("NameBinding not found")
    }

    fn set_namebinding_to_def_id(&mut self, def_id: DefId, name_binding: NameBinding<'ctx>) {
        self.def_id_to_name_binding.insert(def_id, name_binding);
    }

    fn get_type_from_node_id(&self, node_id: NodeId) -> Ty {
        self.node_id_to_type.get(&node_id).copied().expect("Type not found") //.unwrap_or(UNKOWN_TY)
    }

    fn try_get_def_id_from_trait_impl_id(
        &self,
        trait_impl_id: &TraitImplId,
        symbol: Symbol
    ) -> Option<DefId> {
        let def_ids = self.resolver_handle
            .lookup_trait_impl_def_ids(trait_impl_id)
            .expect("Trait impl not found");

        def_ids
            .iter()
            .find(|def_id| def_id.symbol.get() == symbol.get())
            .copied()
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
        self.set_type_to_node_id(interger_expr.ast_node_id, ty);
        ty
    }

    fn visit_bool_expr(&mut self, bool_expr: &'ast BoolExpr) -> Self::Result {
        self.set_type_to_node_id(bool_expr.ast_node_id, BOOL_TY);
        BOOL_TY
    }

    fn visit_null_expr(&mut self, null_expr: &'ast NullExpr) -> Self::Result {
        self.set_type_to_node_id(null_expr.ast_node_id, NULL_TY);
        NULL_TY
    }

    fn visit_string_expr(&mut self, string_expr: &'ast StringExpr) -> Self::Result {
        self.set_type_to_node_id(string_expr.ast_node_id, STR_TY);
        STR_TY
    }

    fn visit_block_expr(&mut self, expr: &'ast BlockExpr<'ast>) -> Self::Result {
        let block_type = self.visit_stmts(expr.stmts);
        self.set_type_to_node_id(expr.ast_node_id, block_type);
        block_type
    }

    fn visit_fn_item(&mut self, fn_item: &'ast FnItem<'ast>) -> Self::Result {
        let def_id = self.get_def_id_from_node_id(fn_item.ident_node.ast_node_id);
        let name_binding = self.get_namebinding_from_def_id(def_id);
        let fn_sig = match name_binding.kind {
            NameBindingKind::Fn(fn_sig, _, _) => fn_sig,
            _ => unreachable!(),
        };

        self.fn_ret_ty = Some(*fn_sig.ret_ty);

        self.visit_stmts(fn_item.body);

        self.fn_ret_ty = None;

        Self::default_result()
    }

    fn visit_pkg_ident_expr(&mut self, pkg_ident_node: &'ast PkgIdentNode) -> Self::Result {
        self.set_type_to_node_id(pkg_ident_node.ast_node_id, Ty::Package);
        Ty::Package
    }

    fn visit_ident_expr(&mut self, ident_node: &'ast IdentNode) -> Self::Result {
        let def_id = self.try_get_def_id_from_node_id(ident_node.ast_node_id);

        if let Some(def_id) = def_id {
            let name_binding = self.get_namebinding_from_def_id(def_id);
            match name_binding.kind {
                NameBindingKind::Pkg(_) => {
                    unimplemented!(
                        "Packages should probably not be here (unless variables can be assigned to packages)"
                    )
                }
                NameBindingKind::Variable(mutability) => {
                    let ty = self.get_type_from_node_id(def_id.node_id);
                    let ty = Ty::StackPtr(TyCtx::intern_type(ty), mutability);

                    self.set_type_to_node_id(ident_node.ast_node_id, ty);
                    ty
                }
                NameBindingKind::Fn(_, _, _) => {
                    let ty = Ty::FnDef(def_id);
                    self.set_type_to_node_id(ident_node.ast_node_id, ty);
                    ty
                }
                | NameBindingKind::Adt(Adt::Enum(_))
                | NameBindingKind::Adt(Adt::Struct(_))
                | NameBindingKind::Adt(Adt::Typedef(_)) => {
                    let ty = Ty::AtdConstructer(def_id);
                    self.set_type_to_node_id(ident_node.ast_node_id, ty);
                    ty
                }
                NameBindingKind::Adt(Adt::EnumVariant(_, _, _)) =>
                    panic!(
                        "Allow this if enum variants when enum variants can be used as top level names"
                    ),
                NameBindingKind::ConstStr(_) => unreachable!("Const strings should not be here"),
            }
        } else {
            self.set_type_to_node_id(ident_node.ast_node_id, UNKOWN_TY);
            UNKOWN_TY
        }
    }

    fn visit_call_expr(&mut self, call_expr: &'ast CallExpr<'ast>) -> Self::Result {
        let calle_ty = self.visit_expr(call_expr.callee);

        if let Ty::AtdConstructer(enum_variant_def_id) = calle_ty {
            let name_binding = self.get_namebinding_from_def_id(enum_variant_def_id);

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
                        &(|def_id: DefId| { self.try_get_namebinding_from_def_id(def_id) })
                    );

                    if let Err(errors) = is_valid_arg {
                        for error in errors {
                            println!("{:?}", error);
                        }

                        todo!("Report error");
                    }
                }
            }

            self.set_type_to_node_id(call_expr.ast_node_id, Ty::Adt(enum_variant_def_id));

            return Ty::Adt(enum_def_id);
        }

        let (fn_sig, has_self_arg) = match calle_ty.auto_deref() {
            Ty::FnDef(def_id) => {
                if
                    let NameBindingKind::Fn(fn_sig, has_self_arg, _) =
                        self.get_namebinding_from_def_id(def_id).kind
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
                } else if arg == &Ty::VariadicArgs { None } else { Some(1) }
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
                &(|def_id: DefId| { self.try_get_namebinding_from_def_id(def_id) })
            );

            if let Err(errors) = is_valid_arg {
                for error in errors {
                    println!("{:?}", error);
                }

                todo!(
                    "Report error at: {:?}. Arg number {}.\n Arg detail: {:?}. Types is not eq: Given: {:?} != {:?}",
                    call_expr,
                    i,
                    arg,
                    given_arg_ty,
                    arg_ty
                );
            }
        }

        self.set_type_to_node_id(call_expr.ast_node_id, ret_ty);
        ret_ty
    }

    fn visit_return_expr(&mut self, return_expr: &'ast ReturnExpr) -> Self::Result {
        let ret_ty = if let Some(expr) = return_expr.value {
            self.visit_expr(expr)
        } else {
            VOID_TY
        };

        if let Some(fn_ret_ty) = self.fn_ret_ty {
            self.set_type_to_node_id(return_expr.ast_node_id, fn_ret_ty);
            if
                let Err(errors) = TypeChecker::test_eq_loose(
                    ret_ty,
                    fn_ret_ty,
                    &(|def_id: DefId| { self.try_get_namebinding_from_def_id(def_id) })
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
                self.set_type_to_node_id(struct_expr.ast_node_id, Ty::Unkown);
                return Ty::Unkown;
            }
        };

        let name_binding = self.get_namebinding_from_def_id(atd_constructer_def_id);

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
                    &(|def_id: DefId| { self.try_get_namebinding_from_def_id(def_id) })
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

            self.set_type_to_node_id(
                struct_expr.field_initializations[i].ident.ast_node_id,
                *given_ty
            );
        }
        let struct_ty = Ty::Adt(atd_constructer_def_id);
        self.set_type_to_node_id(struct_expr.ident_node.ast_node_id, struct_ty);
        self.set_type_to_node_id(struct_expr.ast_node_id, struct_ty);
        struct_ty
    }

    fn visit_field_expr(&mut self, field_expr: &'ast FieldExpr<'ast>) -> Self::Result {
        let lhs_ty = self.visit_expr(field_expr.lhs);

        if let Ty::Package = lhs_ty {
            let def_id = self.get_def_id_from_node_id(field_expr.rhs.ast_node_id);
            let name_binding = self.resolver_handle.lookup_pkg_member_name_binding(&def_id);

            if let Some(name_binding) = name_binding {
                let ty = match name_binding.kind {
                    NameBindingKind::Adt(_) => Ty::AtdConstructer(def_id),
                    NameBindingKind::Fn(_, _, _) => Ty::FnDef(def_id),
                    _ => panic!("Expected adt or function"),
                };

                self.set_type_to_node_id(field_expr.rhs.ast_node_id, ty);
                self.set_type_to_node_id(field_expr.ast_node_id, ty);

                ty
            } else {
                todo!("Undefined package member");
            }
        } else if let Ty::AtdConstructer(def_id) = lhs_ty {
            let rhs_symbol = Symbol::from_node_id(field_expr.rhs.ast_node_id);

            // This must be an enum, because an enum's constructor requires a field expression e.g. Option.Some(5)
            let name_binding = self.get_namebinding_from_def_id(def_id);
            if let NameBindingKind::Adt(Adt::Enum(enum_variants)) = name_binding.kind {
                let variant_def_id = enum_variants
                    .iter()
                    .find(|x| x.symbol.get() == rhs_symbol.get());

                if let Some(variant_def_id) = variant_def_id {
                    let variant_data = self.get_namebinding_from_def_id(*variant_def_id);
                    let enum_fields = match variant_data.kind {
                        NameBindingKind::Adt(Adt::EnumVariant(_, _, enum_fields)) => enum_fields,
                        _ => panic!("Expected enum variant"),
                    };
                    let field_ty = if enum_fields[0] == Ty::ZeroSized {
                        Ty::Adt(*variant_def_id)
                    } else {
                        Ty::AtdConstructer(*variant_def_id)
                    };

                    self.set_def_id_to_node_id(field_expr.rhs.ast_node_id, *variant_def_id);
                    self.set_type_to_node_id(field_expr.rhs.ast_node_id, field_ty);
                    self.set_type_to_node_id(field_expr.ast_node_id, field_ty);

                    return field_ty;
                } else {
                    todo!("Undefined variant: {}", rhs_symbol.get());
                }
            } else if let NameBindingKind::Adt(Adt::Struct(struct_fields)) = name_binding.kind {
                let impl_def_id = self
                    .try_get_def_id_from_trait_impl_id(&TraitImplId::new(def_id, None), rhs_symbol)
                    .expect("Expected impl method");
                let ty = Ty::FnDef(impl_def_id);

                self.set_type_to_node_id(field_expr.rhs.ast_node_id, ty);
                self.set_type_to_node_id(field_expr.ast_node_id, ty);
                self.set_def_id_to_node_id(field_expr.rhs.ast_node_id, impl_def_id);

                return ty;
            } else {
                panic!("Expected enum");
            }
        } else {
            // As of now if we are here, we should expect a struct field (tuples have their own field expression) or an impl method
            let (adt_def_id, adt) = {
                let adt = lhs_ty.try_deref_as_adt(|def_id: DefId| {
                    self.try_get_namebinding_from_def_id(def_id)
                });

                match adt {
                    Some(adt) => adt,
                    None => {
                        println!("Expected struct 2");
                        self.resolver_handle.report_error(
                            Error::new(ErrorKind::InvalidStruct(lhs_ty), Span::dummy())
                        );

                        self.set_type_to_node_id(field_expr.rhs.ast_node_id, Ty::Unkown);
                        self.set_type_to_node_id(field_expr.ast_node_id, Ty::Unkown);

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

                        self.set_type_to_node_id(field_expr.ast_node_id, field_ty);
                        self.set_type_to_node_id(field_expr.rhs.ast_node_id, field_ty);
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
                    let Some(impl_def_id) = self.try_get_def_id_from_trait_impl_id(
                        &trait_impl_id,
                        field_access_symbol
                    )
                {
                    let (fn_sig, has_self_arg) = {
                        let name_binding = self.get_namebinding_from_def_id(impl_def_id);
                        match name_binding.kind {
                            NameBindingKind::Fn(fn_sig, has_self_arg, _) => (fn_sig, has_self_arg),
                            _ => panic!("Expected function"),
                        }
                    };

                    let fn_ty = Ty::FnDef(impl_def_id);
                    self.set_type_to_node_id(field_expr.ast_node_id, fn_ty);
                    self.set_type_to_node_id(field_expr.rhs.ast_node_id, fn_ty);
                    self.set_def_id_to_node_id(field_expr.rhs.ast_node_id, impl_def_id);

                    if has_self_arg == HasSelfArg::Yes {
                        let arg_cmp = ArgCmp {
                            arg_ty: fn_sig.args[0],
                            provided_ty: lhs_ty,
                        };

                        if
                            let Err(errors) = TypeChecker::test_valid_arg(
                                arg_cmp,
                                &(|def_id: DefId| { self.try_get_namebinding_from_def_id(def_id) })
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
            self.set_type_to_node_id(field_expr.ast_node_id, Ty::Unkown);
            self.set_type_to_node_id(field_expr.rhs.ast_node_id, Ty::Unkown);

            Ty::Unkown
        }
    }

    fn visit_tuple_struct_pat(&mut self, tuple_pat: &'ast TupleStructPat<'ast>) -> Self::Result {
        let ty = self.visit_path(tuple_pat.path);

        let (def_id, adt) = match ty {
            Ty::AtdConstructer(def_id) => {
                let name_binding = self.get_namebinding_from_def_id(def_id);
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
                self.set_type_to_node_id(tuple_pat.ast_node_id, ty);
                self.set_def_id_to_node_id(tuple_pat.ast_node_id, def_id);

                if enum_ty.len() != tuple_pat.fields.len() {
                    todo!("Expected {} fields, got {}", enum_ty.len(), tuple_pat.fields.len());
                }

                for (i, pat) in tuple_pat.fields.iter().enumerate() {
                    let field_ty = enum_ty[i];

                    if let Pat::IdentPat(ident_pat) = pat {
                        let def_id = self.get_def_id_from_node_id(ident_pat.ast_node_id);
                        self.set_namebinding_to_def_id(
                            def_id,
                            NameBinding::new(NameBindingKind::Variable(Mutability::Immutable))
                        );
                        self.set_type_to_node_id(ident_pat.ast_node_id, field_ty);
                    } else {
                        self.visit_pat(*pat);
                    }
                }

                ty
            }
            // Adt::TupleStruct(_) => todo!("Tuple struct"),
            _ => panic!("Expected enum variant"),
        }
    }

    fn visit_ident_pat(&mut self, ident_node: &'ast IdentNode) -> Self::Result {
        Ty::Unkown
    }

    fn visit_path_pkg(&mut self, pkg_ident_expr: &'ast PkgIdentNode) -> Self::Result {
        self.set_type_to_node_id(pkg_ident_expr.ast_node_id, Ty::Package);
        Ty::Package
    }

    fn visit_path_field(&mut self, path_field: &'ast PathField<'ast>) -> Self::Result {
        let lhs_ty = self.visit_path(path_field.lhs);
        let rhs_symbol = Symbol::from_node_id(path_field.rhs.ast_node_id);

        let def_id = match lhs_ty {
            Ty::AtdConstructer(def_id) => def_id,
            Ty::Package => {
                let def_id = self.get_def_id_from_node_id(path_field.rhs.ast_node_id);
                let name_binding = self.resolver_handle
                    .lookup_pkg_member_name_binding(&def_id)
                    .expect("Package member not found");

                return match name_binding.kind {
                    NameBindingKind::Adt(_) => Ty::AtdConstructer(def_id),
                    NameBindingKind::Fn(_, _, _) => Ty::FnDef(def_id),
                    _ =>
                        panic!(
                            "Expected adt or function (because that's the only kinds of members a package can have and export)"
                        ),
                };
            }
            ty => { panic!("Invalid lhs of path field: {}\n{:?}", ty, path_field.lhs) }
        };

        let name_binding = self.get_namebinding_from_def_id(def_id);

        match name_binding.kind {
            NameBindingKind::Adt(Adt::Struct(fields)) => {
                todo!("Lookup constructer method for struct");
            }
            NameBindingKind::Adt(Adt::Enum(variants)) => {
                let variant_def_id = variants.iter().find(|x| x.symbol.get() == rhs_symbol.get());

                if let Some(variant_def_id) = variant_def_id {
                    let atd_constructor_ty = Ty::AtdConstructer(*variant_def_id);
                    self.set_type_to_node_id(path_field.rhs.ast_node_id, atd_constructor_ty);
                    self.set_type_to_node_id(path_field.ast_node_id, atd_constructor_ty);
                    atd_constructor_ty
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
        self.set_type_to_node_id(path_segment.ast_node_id, ty);
        ty
    }

    fn visit_tuple_field_expr(
        &mut self,
        tuple_field_expr: &'ast TupleFieldExpr<'ast>
    ) -> Self::Result {
        let lhs_ty = self.visit_expr(tuple_field_expr.lhs);
        self.visit_interger_expr(tuple_field_expr.rhs);

        let tuple_ty = match
            lhs_ty.try_deref_as_tuple(|def_id: DefId| {
                self.try_get_namebinding_from_def_id(def_id)
            })
        {
            Some(tuple_ty) => tuple_ty,
            None => {
                self.resolver_handle.report_error(
                    Error::new(ErrorKind::InvalidTuple(lhs_ty), Span::dummy())
                );
                self.set_type_to_node_id(tuple_field_expr.ast_node_id, Ty::Unkown);
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
            self.set_type_to_node_id(tuple_field_expr.ast_node_id, Ty::Unkown);
            Ty::Unkown
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

            self.set_type_to_node_id(tuple_field_expr.ast_node_id, access_ty);

            access_ty
        }
    }

    fn visit_index_expr(&mut self, index_expr: &'ast IndexExpr<'ast>) -> Self::Result {
        let lhs_ty = self.visit_expr(index_expr.lhs);

        let is_mutable = lhs_ty.is_mut_ptr();

        match
            self
                .visit_expr(index_expr.value_expr)
                .get_expanded_dereffed_ty(|def_id: DefId| {
                    self.try_get_namebinding_from_def_id(def_id)
                })
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
                let full_lhs_ty = lhs_ty.get_expanded_dereffed_ty(|def_id: DefId| {
                    self.try_get_namebinding_from_def_id(def_id)
                });
                todo!("Expected many-item-pointer, got {}", full_lhs_ty);
            }
        };

        self.set_type_to_node_id(index_expr.ast_node_id, result_ty);

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
            todo!("Assignment to immutable, expected mutable: {:?}", setter_ty);
        }

        if
            let Err(_errors) = TypeChecker::test_eq_loose(
                setter_ty,
                value_ty,
                &(|def_id: DefId| { self.try_get_namebinding_from_def_id(def_id) })
            )
        {
            panic!("Not same type in assignment: {}, {}", setter_ty, value_ty);
        } else {
            self.set_type_to_node_id(assign_stmt.ast_node_id, VOID_TY);
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

        self.set_type_to_node_id(tuple_expr.ast_node_id, tuple_ty);

        tuple_ty
    }

    fn visit_def_stmt(&mut self, def_stmt: &'ast DefineStmt<'ast>) -> Self::Result {
        // When tuple patterns are implemented, compare if tuple on lhs, is same as tuple type on rhs

        match &def_stmt.setter_expr {
            Pat::IdentPat(ident_pat) => {
                let mutability = if def_stmt.mut_span.is_some() {
                    Mutability::Mutable
                } else {
                    Mutability::Immutable
                };
                let def_id = self.get_def_id_from_node_id(ident_pat.ast_node_id);
                self.set_namebinding_to_def_id(
                    def_id,
                    NameBinding::new(NameBindingKind::Variable(mutability))
                );

                let value_type = self.visit_expr(def_stmt.value_expr);

                self.set_type_to_node_id(ident_pat.ast_node_id, value_type);
            }
            Pat::TupleStructPat(_) => todo!("Tuple patterns"),
        }

        // Even though def stmts doesn't return a value,
        // it still sets the type of the def stmt for ease of use in the prettifier
        //
        // It will, however, not have any effect on the actual program,
        // since below it returns void
        self.set_type_to_node_id(def_stmt.ast_node_id, VOID_TY);

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

        self.set_type_to_node_id(loop_expr.ast_node_id, ty);
        ty
    }

    fn visit_continue_expr(&mut self, continue_expr: &'ast ContinueExpr) -> Self::Result {
        self.set_type_to_node_id(continue_expr.ast_node_id, VOID_TY);
        VOID_TY
    }

    fn visit_break_expr(&mut self, break_expr: &'ast BreakExpr<'ast>) -> Self::Result {
        let break_ty = break_expr.value
            .map(|expr| self.visit_expr(expr))
            .unwrap_or(Self::default_result());

        if let Some(loop_ret_ty) = &self.loop_ret_ty {
            if let Some(expected_ty) = loop_ret_ty {
                if
                    let Err(errors) = TypeChecker::test_eq_loose(
                        *expected_ty,
                        break_ty,
                        &(|def_id: DefId| { self.try_get_namebinding_from_def_id(def_id) })
                    )
                {
                    self.resolver_handle.report_error(
                        Error::new(ErrorKind::BreakTypeError(*expected_ty, break_ty), Span::dummy())
                    );
                }
            } else {
                self.loop_ret_ty = Some(Some(break_ty));
            }
        } else {
            self.resolver_handle.report_error(
                Error::new(ErrorKind::BreakOutsideLoop, Span::dummy())
            );
        }

        self.set_type_to_node_id(break_expr.ast_node_id, break_ty);

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
                    &(|def_id: DefId| { self.try_get_namebinding_from_def_id(def_id) })
                )
            {
                for error in errors {
                    println!("{:?}", error);
                }
                panic!("Expected equal types in if expr: {} != {}", true_type, false_type);
            } else if true_type.is_num_ty() && false_type.is_num_ty() {
                Ty::get_biggest_num_ty(true_type, false_type)
                    .expect("Expected number")
                    .auto_deref()
            } else {
                true_type
            }
        } else {
            true_type
        };

        self.set_type_to_node_id(if_expr.ast_node_id, if_expr_ty);

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
            &(|def_id: DefId| { self.try_get_namebinding_from_def_id(def_id) })
        );

        if let Some(result_ty) = result_ty {
            self.set_type_to_node_id(binary_expr.ast_node_id, result_ty);

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

        self.set_type_to_node_id(group_expr.ast_node_id, expr_type);

        expr_type
    }
}
