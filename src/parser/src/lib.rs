use std::sync::LazyLock;

const PARSE_RULE_COUNT: usize = enum_iterator::cardinality::<TokenKind>();
static PARSE_RULES: LazyLock<[ParseRule; PARSE_RULE_COUNT]> = LazyLock::new(|| {
    let mut rules = [ParseRule::dummy(); PARSE_RULE_COUNT];
    for (i, kind) in enum_iterator::all::<TokenKind>().enumerate() {
        rules[i] =
            make_parse_rule!(kind,

            /*  TOKENKIND        PREFIX                 INFIX                               POSTFIX             */
            /*                   method     prec        method      prec                    method      prec    */
                LeftParen   = { (grouping   None),      (call       PrecCall        ),      (None       None) },
                RightParen  = { (None       None),      (None       None            ),      (None       None) },
                LeftCurly   = { (block_expr None),      (None       None            ),      (None       None) },
                RightCurly  = { (None       None),      (None       None            ),      (None       None) },
                LeftSquare  = { (None       None),      (index_expr PrecIndex       ),      (None       None) },
                RightSquare = { (None       None),      (None       None            ),      (None       None) },
                Eq          = { (None       None),      (eq         PrecEquality    ),      (None       None) },
                Ne          = { (None       None),      (ne         PrecEquality    ),      (None       None) },
                Ge          = { (None       None),      (ge         PrecComparison  ),      (None       None) },
                Gt          = { (None       None),      (gt         PrecComparison  ),      (None       None) },
                Le          = { (None       None),      (le         PrecComparison  ),      (None       None) },
                Lt          = { (None       None),      (lt         PrecComparison  ),      (None       None) },
                Plus        = { (None       None),      (add        PrecTerm        ),      (None       None) },
                Minus       = { (None       None),      (sub        PrecTerm        ),      (None       None) },
                Star        = { (None       None),      (mul        PrecFactor      ),      (None       None) },
                Slash       = { (None       None),      (div        PrecFactor      ),      (None       None) },
                Colon       = { (None       None),      (None       None            ),      (None       None) },
                Define      = { (None       None),      (define     PrecAssign      ),      (None       None) },
                Assign      = { (None       None),      (assign     PrecAssign      ),      (None       None) },
                Dot         = { (dot_float  None),      (field_expr PrecCall        ),      (None       None) },
                Comma       = { (None       None),      (None       None            ),      (None       None) },
                Bang        = { (None       None),      (None       None            ),      (None       None) },
                Increment   = { (pre_inc    None),      (None       None            ),      (post_inc   None) },
                Decrement   = { (pre_dec    None),      (None       None            ),      (post_dec   None) },
                DoubleQuote = { (string     None),      (None       None            ),      (None       None) },
                StringChar  = { (None       None),      (None       None            ),      (None       None) },
                Ellipsis    = { (None       None),      (None       None            ),      (None       None) },
                
                    
                // Numbers
                Integer     = { (integer    None),      (None       None            ),      (None       None) },
                Float       = { (float      None),      (None       None            ),      (None       None) },
        
                // Literal `null``
                Null        = { (null_lit   None),      (None       None            ),      (None       None) },
        
                // Booleans
                True        = { (true_lit   None),      (None       None            ),      (None       None) },
                False       = { (false_lit  None),      (None       None            ),      (None       None) },
                    
                // Identifier
                Ident       = { (ident      None),      (None       None            ),      (None       None) },

                // Keywords
                Import      = { (None       None),      (None       None            ),      (None       None) },
                Impl        = { (None       None),      (None       None            ),      (None       None) },
                SmallSelf   = { (ident      None),      (None       None            ),      (None       None) },
                BigSelf     = { (ident      None),      (None       None            ),      (None       None) },
                Fn          = { (None       None),      (None       None            ),      (None       None) },
                Declare     = { (None       None),      (None       None            ),      (None       None) },
                Typedef     = { (None       None),      (None       None            ),      (None       None) },
                Mut         = { (None       None),      (None       None            ),      (None       None) },
                Struct      = { (None       None),      (None       None            ),      (None       None) },
                Enum        = { (None       None),      (None       None            ),      (None       None) },
                While       = { (None       None),      (None       None            ),      (None       None) },
                If          = { (if_expr    None),      (None       None            ),      (None       None) },
                Loop        = { (loop_expr  None),      (None       None            ),      (None       None) },
                Break       = { (None       None),      (None       None            ),      (None       None) },
                Continue    = { (None       None),      (None       None            ),      (None       None) },
                Return      = { (None       None),      (None       None            ),      (None       None) },
                Else        = { (None       None),      (None       None            ),      (None       None) },
                Elif        = { (None       None),      (None       None            ),      (None       None) },
                Pkg         = { (pkg_ident  None),      (None       None            ),      (None       None) },

                Eof         = { (None       None),      (None       None            ),      (None       None) }
                
                );
    }

    rules
});

use ast::{
    is_stmt_adt,
    ArgKind,
    AsigneeExpr,
    Ast,
    AstArenaObject,
    AstMetadata,
    AstState0,
    BlockExpr,
    BoolExpr,
    BreakExpr,
    CompDeclItem,
    CompFnDeclItem,
    CondKind,
    ContinueExpr,
    EnumItem,
    EnumVariant,
    Expr,
    ExprWithoutBlock,
    Field,
    FieldInitialization,
    FnItem,
    GlobalScope,
    IdentNode,
    IfExpr,
    IfFalseBranchExpr,
    ImplItem,
    ImportItem,
    IntegerExpr,
    ItemStmt,
    ItemType,
    LoopExpr,
    NullExpr,
    Pat,
    Path,
    PathField,
    PkgIdentNode,
    PlaceExpr,
    ReturnExpr,
    Stmt,
    StringExpr,
    StructItem,
    TupleStructPat,
    TypedefItem,
    Typing,
    ValueExpr,
};
use diagnostics::{ Diagnostic, ErrorKind };
use error::Error;
use expr_builder::ExprBuilder;
use ir::{
    Delimeter,
    ExpectedSymbolKind,
    ItemErrorKind,
    MissingCommaPlace,
    ModId,
    Mutability,
    NodeId,
    Symbol,
};
use lexer::Lexer;
use make_parse_rule::make_parse_rule;
use op::{ ArithmeticOp, BinaryOp, ComparisonOp };
use precedence::Precedence;
use span::Span;
use token::{ Token, TokenKind };
mod make_parse_rule;
mod expr_builder;
mod precedence;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParsingDeclareFn {
    Yes,
    No,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StopToken {
    None,
    Token(TokenKind),
}

#[derive(Debug, Clone, Copy)]
struct ParsedFnSignature<'a> {
    ident: &'a IdentNode,
    args: &'a [ArgKind<'a>],
    ret_typing: Option<Typing<'a>>,
}

type ParseRuleMethod = for<'a, 'b, 'c> fn(&'c mut Parser<'a, 'b>, &'c mut ExprBuilder<'a, 'b>);

/// Each token kind is asociated with a ParseRule (specified in function: Parser::create_parse_rule)
///
/// A parserule contains the token kind precedences and possibly parse methods
#[derive(Debug, Clone, Copy)]
pub(crate) struct ParseRule {
    pub prefix_method: Option<ParseRuleMethod>,
    pub prefix_prec: Precedence,
    pub infix_method: Option<ParseRuleMethod>,
    pub infix_prec: Precedence,
    pub postfix_method: Option<ParseRuleMethod>,
    pub postfix_prec: Precedence,
}

impl ParseRule {
    pub const fn dummy() -> Self {
        Self {
            prefix_method: None,
            prefix_prec: Precedence::PrecNone,
            infix_method: None,
            infix_prec: Precedence::PrecNone,
            postfix_method: None,
            postfix_prec: Precedence::PrecNone,
        }
    }
}

pub(crate) trait ParserHandle<'ast> {
    fn get_ast_node_id(&mut self) -> NodeId;

    fn forget_node(&mut self, node_id: NodeId);

    fn try_as_pat(&mut self, expr: Expr<'ast>) -> Option<Pat<'ast>>;

    fn try_as_asignee_expr(&mut self, expr: Expr<'ast>) -> Option<AsigneeExpr<'ast>>;

    fn try_as_ident(&mut self, expr: Expr<'ast>) -> Option<&'ast IdentNode>;

    fn try_as_path(&mut self, expr: Expr<'ast>) -> Option<Path<'ast>>;
}

impl<'a> ParserHandle<'a> for Parser<'a, '_> {
    fn forget_node(&mut self, node_id: NodeId) {
        self.forgotten_nodes += 1;
    }

    fn get_ast_node_id(&mut self) -> NodeId {
        Parser::get_ast_node_id(self)
    }

    fn try_as_path(&mut self, expr: Expr<'a>) -> Option<Path<'a>> {
        match expr {
            Expr::ExprWithBlock(_) => None,
            Expr::ExprWithoutBlock(expr) => {
                match expr {
                    ExprWithoutBlock::PlaceExpr(place_expr) => {
                        match place_expr {
                            PlaceExpr::IdentExpr(ident_expr) => {
                                Some(Path::PathSegment(ident_expr))
                            }
                            PlaceExpr::PkgIdentExpr(pkg_ident_expr) => {
                                Some(Path::PathPkg(pkg_ident_expr))
                            }
                            PlaceExpr::TupleFieldExpr(_) => None,
                            PlaceExpr::IndexExpr(_) => None,
                            PlaceExpr::FieldExpr(field_expr) => {
                                let lhs = self.try_as_path(field_expr.lhs);

                                match lhs {
                                    Some(lhs) => {
                                        Some(
                                            Path::PathField(
                                                self.ast_arena.alloc_expr_or_stmt(
                                                    PathField::new(
                                                        lhs,
                                                        field_expr.rhs,
                                                        field_expr.span,
                                                        field_expr.ast_node_id
                                                    )
                                                )
                                            )
                                        )
                                    }
                                    None => None,
                                }
                            }
                        }
                    }
                    ExprWithoutBlock::BreakExpr(_) => None,
                    ExprWithoutBlock::ContinueExpr(_) => None,
                    ExprWithoutBlock::ReturnExpr(_) => None,
                    ExprWithoutBlock::ValueExpr(value_expr) => {
                        match value_expr {
                            ValueExpr::BinaryExpr(_) => None,
                            ValueExpr::CallExpr(_) => None,
                            ValueExpr::ConstExpr(_) => None,
                            ValueExpr::GroupExpr(_) => None,
                            ValueExpr::StructExpr(_) => None,
                            ValueExpr::TupleExpr(_) => None,
                        }
                    }
                }
            }
        }
    }

    fn try_as_pat(&mut self, expr: Expr<'a>) -> Option<Pat<'a>> {
        match expr {
            Expr::ExprWithBlock(_) => None,
            Expr::ExprWithoutBlock(expr) => {
                match expr {
                    ExprWithoutBlock::PlaceExpr(expr) => {
                        match expr {
                            PlaceExpr::TupleFieldExpr(_) => None,
                            PlaceExpr::FieldExpr(_) => None,
                            PlaceExpr::IndexExpr(_) => None,
                            PlaceExpr::PkgIdentExpr(_) => None,
                            PlaceExpr::IdentExpr(ident_expr) =>
                                Some(
                                    Pat::IdentPat(
                                        self.ast_arena.alloc_expr_or_stmt(ident_expr.get_copy())
                                    )
                                ),
                        }
                    }
                    ExprWithoutBlock::BreakExpr(_) => None,
                    ExprWithoutBlock::ContinueExpr(_) => None,
                    ExprWithoutBlock::ReturnExpr(_) => None,
                    ExprWithoutBlock::ValueExpr(expr) => {
                        match expr {
                            ValueExpr::BinaryExpr(_) => None,
                            ValueExpr::CallExpr(call_expr) => {
                                let path = match self.try_as_path(call_expr.callee) {
                                    Some(path) => path,
                                    None => {
                                        return None;
                                    }
                                };

                                let mut has_args_failed = false;

                                let pat_args = call_expr.args
                                    .iter()
                                    .filter_map(|arg| {
                                        let pat = self.try_as_pat(*arg);
                                        if pat.is_none() {
                                            has_args_failed = true;
                                        }
                                        pat
                                    })
                                    .collect::<Vec<_>>();

                                if has_args_failed {
                                    return None;
                                }

                                let final_pat = Pat::TupleStructPat(
                                    self.ast_arena.alloc_expr_or_stmt(
                                        TupleStructPat::new(
                                            path,
                                            self.ast_arena.alloc_vec(pat_args),
                                            call_expr.span,
                                            call_expr.ast_node_id
                                        )
                                    )
                                );

                                Some(final_pat)
                            }
                            ValueExpr::ConstExpr(_) => None,
                            ValueExpr::GroupExpr(_) => None,
                            ValueExpr::StructExpr(_) => None,
                            ValueExpr::TupleExpr(tuple_expr) =>
                                todo!("As pattern: {:#?}", tuple_expr),
                        }
                    }
                }
            }
        }
    }

    fn try_as_asignee_expr(&mut self, expr: Expr<'a>) -> Option<AsigneeExpr<'a>> {
        match expr {
            Expr::ExprWithBlock(_) => None,
            Expr::ExprWithoutBlock(expr) => {
                match expr {
                    ExprWithoutBlock::PlaceExpr(expr) => Some(AsigneeExpr::PlaceExpr(expr)),
                    ExprWithoutBlock::BreakExpr(_) => None,
                    ExprWithoutBlock::ReturnExpr(_) => None,
                    ExprWithoutBlock::ContinueExpr(_) => None,
                    ExprWithoutBlock::ValueExpr(expr) => {
                        match expr {
                            ValueExpr::BinaryExpr(_) => None,
                            ValueExpr::ConstExpr(_) => None,
                            ValueExpr::GroupExpr(_) => None,
                            ValueExpr::CallExpr(call_expr) =>
                                Some(AsigneeExpr::CallExpr(call_expr)),
                            ValueExpr::StructExpr(_) => None,
                            ValueExpr::TupleExpr(tuple_expr) =>
                                todo!("As place expr: {:#?}", tuple_expr),
                        }
                    }
                }
            }
        }
    }

    fn try_as_ident(&mut self, expr: Expr<'a>) -> Option<&'a IdentNode> {
        match expr {
            Expr::ExprWithBlock(_) => None,
            Expr::ExprWithoutBlock(expr) => {
                match expr {
                    ExprWithoutBlock::BreakExpr(_) => None,
                    ExprWithoutBlock::ReturnExpr(_) => None,
                    ExprWithoutBlock::ContinueExpr(_) => None,
                    ExprWithoutBlock::ValueExpr(_) => None,
                    ExprWithoutBlock::PlaceExpr(expr) => {
                        match expr {
                            PlaceExpr::IdentExpr(expr) => Some(expr),
                            PlaceExpr::PkgIdentExpr(_) => None,
                            PlaceExpr::TupleFieldExpr(_) => None,
                            PlaceExpr::FieldExpr(_) => None,
                            PlaceExpr::IndexExpr(_) => None,
                        }
                    }
                }
            }
        }
    }
}

pub struct Parser<'a, 'b> where 'a: 'b {
    lexer: Lexer<'b>,
    ast_arena: &'b AstArenaObject<'a>,
    src: &'b str,
    current: Token,
    prev: Token,
    next_ast_node_id: u32,
    mod_id: ModId,

    // Used for allocating required memory later during AST validation
    forgotten_nodes: usize,
    parsed_fn_count: usize,
    def_count: usize,

    panic_mode: bool,

    /// Used for error reporting
    diagnostics: Vec<Diagnostic>,
}

impl<'a, 'b> Parser<'a, 'b> where 'a: 'b {
    pub fn new(src: &'b str, ast_arena: &'b AstArenaObject<'a>, mod_id: ModId) -> Self {
        let mut lexer = Lexer::new(src);

        Self {
            current: lexer.scan_token(),
            src,
            ast_arena,
            lexer,
            parsed_fn_count: 0,
            prev: Token::dummy(),
            next_ast_node_id: 0,
            mod_id,
            def_count: 0,
            forgotten_nodes: 0,
            panic_mode: false,
            diagnostics: Vec::new(),
        }
    }

    pub(crate) fn report_error(&mut self, error_kind: ErrorKind, span: Span) {
        self.diagnostics.push(Diagnostic::new_error(error_kind, span, self.mod_id));
    }

    pub fn parse_ast<'c>(mut self) -> (Ast<'a, AstState0>, Vec<Diagnostic>) where 'c: 'b {
        let global_scope = GlobalScope::new(self.parse_block_as_stmts(StopToken::None));

        let nodes_count = (self.next_ast_node_id as usize) - self.forgotten_nodes;

        (
            Ast::new(
                global_scope,
                AstMetadata::new(
                    self.parsed_fn_count,
                    nodes_count,
                    self.def_count + self.parsed_fn_count,
                    self.mod_id
                )
            ),
            self.diagnostics,
        )
    }

    pub(crate) fn statement(&mut self) -> Option<Stmt<'a>> {
        match self.current.get_kind() {
            TokenKind::Impl => self.impl_statement(),
            TokenKind::Typedef => {
                self.def_count += 1;
                Some(self.typedef_statement())
            }
            TokenKind::Struct => {
                self.def_count += 1;
                Some(self.struct_item())
            }
            TokenKind::Enum => {
                self.def_count += 1;
                Some(self.enum_item())
            }
            TokenKind::Mut => Some(self.mut_stmt()),
            TokenKind::Break => Some(self.break_expr()),
            TokenKind::Continue => Some(self.continue_expr()),
            TokenKind::Fn => {
                if let Some(fn_item) = self.function_statement() {
                    Some(Stmt::ItemStmt(ItemStmt::FnItem(fn_item)))
                } else {
                    None
                }
            }
            TokenKind::Declare => self.declare_statement(),
            TokenKind::Return => Some(self.return_expr()),
            TokenKind::Import => Some(self.import_statement()),
            _ => self.expression_statement(),
        }
    }

    pub(crate) fn parse_path(&mut self) -> Path<'a> {
        println!("Missing error handling for path parsing");
        let mut start_span = self.current.get_span();

        let mut path = if self.is_curr_kind(TokenKind::Pkg) {
            let pkg_ident = self.consume_pkg_ident("Expected package ident in path");
            Path::PathPkg(self.ast_arena.alloc_expr_or_stmt(pkg_ident))
        } else {
            let first_ident = self.consume_ident("Expected ident in path");
            Path::PathSegment(self.ast_arena.alloc_expr_or_stmt(first_ident))
        };

        while !self.is_eof() {
            if self.is_curr_kind(TokenKind::Dot) {
                self.advance();
                let ident = self.consume_ident("Expected ident in path");
                let end_span = self.current.get_span();

                let span = Span::merge(start_span, end_span);
                path = Path::PathField(
                    self.ast_arena.alloc_expr_or_stmt(
                        PathField::new(
                            path,
                            self.ast_arena.alloc_expr_or_stmt(ident),
                            span,
                            self.get_ast_node_id()
                        )
                    )
                );

                start_span = end_span;
            } else {
                break;
            }
        }

        path
    }

    pub(crate) fn import_statement(&mut self) -> Stmt<'a> {
        let start_span = self.current.get_span();
        self.advance();
        let mut import_items = vec![self.parse_path()];

        while !self.is_eof() {
            if self.is_curr_kind(TokenKind::Comma) {
                self.advance();
                import_items.push(self.parse_path());
                continue;
            } else {
                break;
            }
        }

        let import_stmt = ItemStmt::ImportItem(
            self.ast_arena.alloc_expr_or_stmt(
                ImportItem::new(
                    self.ast_arena.alloc_vec(import_items),
                    Span::merge(start_span, self.current.get_span()),
                    self.get_ast_node_id()
                )
            )
        );

        Stmt::ItemStmt(import_stmt)
    }

    pub(crate) fn impl_statement(&mut self) -> Option<Stmt<'a>> {
        let start_span = self.current.get_span();
        self.advance();

        let impl_path = self.parse_path();

        let success = self.consume_or_report_error(
            TokenKind::LeftCurly,
            ErrorKind::ExpectedToken {
                additional_info: Some("after impl path"),
                expected: TokenKind::LeftCurly,
                found: Symbol::new(self.get_lexeme_of_current()),
            },
            self.current.get_span()
        );

        if !success {
            self.synchronize();
            return None;
        }

        let mut impl_fn_items = Vec::with_capacity(8);

        while !self.is_eof() && !self.is_curr_kind(TokenKind::RightCurly) {
            let fn_item = self.function_statement();

            if let Some(fn_item) = fn_item {
                impl_fn_items.push(fn_item);
            }

            if !self.is_curr_kind(TokenKind::Fn) {
                break;
            }
        }

        // self.consume(TokenKind::RightCurly, "Expected `}` after impl block");
        let success = self.consume_or_report_error(
            TokenKind::RightCurly,
            ErrorKind::ExpectedToken {
                additional_info: Some("after impl block"),
                expected: TokenKind::RightCurly,
                found: Symbol::new(self.get_lexeme_of_current()),
            },
            self.current.get_span()
        );

        if !success {
            self.synchronize();
            if self.is_curr_kind(TokenKind::RightCurly) {
                self.advance();
            }
        }

        let impl_item = ImplItem::new(
            impl_path,
            self.ast_arena.alloc_vec(impl_fn_items),
            Span::merge(start_span, self.current.get_span()),
            self.get_ast_node_id()
        );

        let impl_item_stmt = ItemStmt::ImplItem(self.ast_arena.alloc_expr_or_stmt(impl_item));

        Some(Stmt::ItemStmt(impl_item_stmt))
    }

    pub(crate) fn declare_statement(&mut self) -> Option<Stmt<'a>> {
        let start_span = self.current.get_span();
        self.advance();

        if !(self.current.get_kind() == TokenKind::Fn) {
            panic!("Error: Declare statement must be a C function declaration");
        }

        let parsed_fn_sig = self.parse_fn_signature(ParsingDeclareFn::Yes);

        let (item_type, parsed_fn_sig) = match parsed_fn_sig {
            Some(parsed_fn_sig) => parsed_fn_sig,
            None => {
                self.synchronize();
                return None;
            }
        };

        if ItemType::Normal == item_type {
            panic!("Error: Declare statement must be a C function declaration");
        }

        let fields = parsed_fn_sig.args
            .iter()
            .map(|arg| {
                match *arg {
                    ArgKind::Arg(field) => field,
                    _ => panic!("Error: Only normal arguments are allowed in function declaration"),
                }
            })
            .collect::<Vec<_>>();

        let comp_decl_item = CompDeclItem::CompFnDeclItem(
            self.ast_arena.alloc_expr_or_stmt(
                CompFnDeclItem::new(
                    parsed_fn_sig.ident,
                    self.ast_arena.alloc_vec(fields),
                    parsed_fn_sig.ret_typing,
                    Span::merge(start_span, self.current.get_span()),
                    self.get_ast_node_id()
                )
            )
        );

        Some(Stmt::ItemStmt(ItemStmt::CompDeclItem(comp_decl_item)))
    }

    pub(crate) fn typedef_statement(&mut self) -> Stmt<'a> {
        let start_span = self.current.get_span();
        self.advance();
        let ident_node = self.consume_ident("Expected ident after `typedef`");
        let ty = self.parse_typing().expect("TODO: Error handling, Expected type");

        let typedef_stmt = ItemStmt::TypedefItem(
            self.ast_arena.alloc_expr_or_stmt(
                TypedefItem::new(
                    self.ast_arena.alloc_expr_or_stmt(ident_node),
                    ty,
                    ItemType::Normal,
                    Span::merge(start_span, self.current.get_span()),
                    self.get_ast_node_id()
                )
            )
        );

        Stmt::ItemStmt(typedef_stmt)
    }

    pub(crate) fn enum_item(&mut self) -> Stmt<'a> {
        let start_span = self.current.get_span();
        self.advance();
        let ident_node = self.consume_ident("Expected ident after `enum`");
        let mut variants = Vec::with_capacity(8);

        self.consume(TokenKind::LeftCurly, "Expected `{` before enum variants");

        while !self.is_eof() && !self.is_curr_kind(TokenKind::RightCurly) {
            let start_enum_variant_span = self.current.get_span();
            let variant_name = self.ast_arena.alloc_expr_or_stmt(
                self.consume_ident("Expected ident in enum variant")
            );

            match self.current.get_kind() {
                TokenKind::Comma => {
                    self.advance();
                    variants.push(
                        EnumVariant::new(
                            variant_name,
                            None,
                            Span::merge(start_enum_variant_span, self.current.get_span())
                        )
                    );
                    continue;
                }
                TokenKind::RightCurly => {
                    variants.push(
                        EnumVariant::new(
                            variant_name,
                            None,
                            Span::merge(start_enum_variant_span, self.current.get_span())
                        )
                    );
                    break;
                }
                TokenKind::LeftParen => {
                    self.advance();
                    let mut tys = Vec::with_capacity(8);
                    loop {
                        let ty = self.parse_typing().expect("Expected type in enum variant");
                        tys.push(ty);

                        if self.is_curr_kind(TokenKind::Comma) {
                            self.advance();
                            continue;
                        }

                        break;
                    }
                    variants.push(
                        EnumVariant::new(
                            variant_name,
                            Some(self.ast_arena.alloc_vec(tys)),
                            Span::merge(start_enum_variant_span, self.current.get_span())
                        )
                    );

                    self.consume(TokenKind::RightParen, "Expected `)` after enum variant");

                    if self.is_curr_kind(TokenKind::Comma) {
                        self.advance();
                    }
                    continue;
                }
                _ => panic!("Unexpected token in enum variant"),
            }
        }

        self.consume(TokenKind::RightCurly, "Expected `}` after enum variants");

        let enum_item = EnumItem::new(
            self.ast_arena.alloc_expr_or_stmt(ident_node),
            self.ast_arena.alloc_vec(variants),
            ItemType::Normal,
            Span::merge(start_span, self.current.get_span()),
            self.get_ast_node_id()
        );

        Stmt::ItemStmt(ItemStmt::EnumItem(self.ast_arena.alloc_expr_or_stmt(enum_item)))
    }

    pub(crate) fn struct_item(&mut self) -> Stmt<'a> {
        let start_span = self.current.get_span();
        self.advance();

        let item_type = if self.is_curr_kind(TokenKind::Dot) {
            self.advance();
            let ident_span = self.consume_ident_span("Currently available item types is: `C`");
            assert!(&self.src[ident_span.get_byte_range()] == "C");
            ItemType::C
        } else {
            ItemType::Normal
        };

        let ident_node = self.consume_ident("Expected identifier after struct");
        let mut fields = Vec::with_capacity(8);

        if self.is_curr_kind(TokenKind::LeftCurly) {
            self.consume(TokenKind::LeftCurly, "Expected `{` before struct fields");

            while !self.is_eof() && !self.is_curr_kind(TokenKind::RightCurly) {
                let start_field_span = self.current.get_span();
                let field_name = self.consume_ident("Expected ident in field");
                let ty = self.parse_typing().expect("TODO: Error handling, Expected type");
                let field = Field::new(
                    self.ast_arena.alloc_expr_or_stmt(field_name),
                    ty,
                    Span::merge(start_field_span, self.current.get_span())
                );

                fields.push(self.ast_arena.alloc_expr_or_stmt(field));

                if self.is_curr_kind(TokenKind::Comma) {
                    self.advance();
                    continue;
                } else if self.is_curr_kind(TokenKind::Ident) {
                    todo!();
                }

                match self.current.get_kind() {
                    TokenKind::RightCurly => {}
                    TokenKind::Ident =>
                        todo!("Error: You are probably missing a `,` in struct declaration"),
                    t =>
                        todo!("Error: Unexpected token `{}` in struct declaration. Expected `,` or `}}`", t),
                }
                break;
            }

            self.consume(TokenKind::RightCurly, "Expected `}` after struct");
        }

        let fields = self.ast_arena.alloc_vec(fields);
        let struct_stmt = StructItem::new(
            self.ast_arena.alloc_expr_or_stmt(ident_node),
            fields,
            item_type,
            Span::merge(start_span, self.current.get_span()),
            self.get_ast_node_id()
        );

        Stmt::ItemStmt(ItemStmt::StructItem(self.ast_arena.alloc_expr_or_stmt(struct_stmt)))
    }

    pub(crate) fn parse_typing(&mut self) -> Option<Typing<'a>> {
        fn parse_many_typings<'a>(
            parser: &mut Parser<'a, '_>,
            mut tuple_typing: Vec<Typing<'a>>
        ) -> &'a [Typing<'a>] {
            while !parser.is_eof() && !parser.is_curr_kind(TokenKind::RightParen) {
                let typing = parser.parse_typing().expect("Expected typing");
                tuple_typing.push(typing);
                if parser.is_curr_kind(TokenKind::Comma) {
                    parser.advance();
                    continue;
                }
                break;
            }
            parser.consume(TokenKind::RightParen, "Expected `)` after tuple typing");
            parser.ast_arena.alloc_vec(tuple_typing)
        }

        match self.current.get_kind() {
            TokenKind::Ellipsis => {
                self.advance();
                Some(Typing::VariadicArgs)
            }
            TokenKind::BigSelf => {
                self.advance();
                Some(Typing::SelfType)
            }
            TokenKind::Ident => {
                let ident = self.consume_ident("Expected ident");
                Some(Typing::Ident(self.ast_arena.alloc_expr_or_stmt(ident)))
            }
            TokenKind::Star => {
                self.advance();
                let mutability = if self.is_curr_kind(TokenKind::Mut) {
                    self.advance();
                    Mutability::Mutable
                } else {
                    Mutability::Immutable
                };
                let ty = self.parse_typing().expect("Expected type after `*`");
                Some(Typing::Ptr(self.ast_arena.alloc_expr_or_stmt(ty), mutability))
            }
            TokenKind::Fn => {
                self.advance();
                self.consume(TokenKind::LeftParen, "Expected `(` before function args");
                let args_typing = parse_many_typings(self, vec![]);
                let ret_typing = self.parse_typing().map(|x| self.ast_arena.alloc_expr_or_stmt(x));
                Some(Typing::Fn(args_typing, ret_typing))
            }
            TokenKind::LeftSquare => {
                self.advance();
                if self.is_curr_kind(TokenKind::Star) {
                    self.advance();
                    self.consume(TokenKind::RightSquare, "Expected `]` after `[*`");
                    let ty = self.parse_typing().expect("Expected type after `[*]`");
                    Some(Typing::ManyPtr(self.ast_arena.alloc_expr_or_stmt(ty)))
                } else {
                    todo!("Array typing");
                }
            }
            TokenKind::LeftParen => {
                self.advance();
                let typing = self.parse_typing().expect("Expected typing");

                match self.current.get_kind() {
                    TokenKind::RightParen => {
                        self.advance();
                        Some(typing)
                    }
                    TokenKind::Comma => {
                        self.advance();
                        let tuple_typing = parse_many_typings(self, vec![typing]);
                        Some(Typing::Tuple(tuple_typing))
                    }
                    t => panic!("Unexpected token in typing: {}", t),
                }
            }
            _ => None,
        }
    }

    pub(crate) fn expression_statement(&mut self) -> Option<Stmt<'a>> {
        let mut expr_builder = ExprBuilder::new(self.ast_arena, None);
        self.expression(&mut expr_builder);
        expr_builder.take_stmt()
    }

    pub(crate) fn break_expr(&mut self) -> Stmt<'a> {
        let break_expr = self.ast_arena.alloc_expr_or_stmt(
            BreakExpr::new(None, self.current.get_span(), self.get_ast_node_id())
        );
        let expr = Expr::ExprWithoutBlock(ExprWithoutBlock::BreakExpr(break_expr));
        self.advance();
        Stmt::ExprStmt(expr)
    }

    pub(crate) fn continue_expr(&mut self) -> Stmt<'a> {
        let continue_expr = self.ast_arena.alloc_expr_or_stmt(
            ContinueExpr::new(self.current.get_span(), self.get_ast_node_id())
        );
        let expr = Expr::ExprWithoutBlock(ExprWithoutBlock::ContinueExpr(continue_expr));
        self.advance();
        Stmt::ExprStmt(expr)
    }

    /// Returns whether or not further arguments can be parsed
    fn recover_from_arg_parse_error(&mut self) -> bool {
        let start_span = self.current.get_span();
        let can_parse_more_args = self.synchronize_with_callback(|token_kind|
            matches!(token_kind, TokenKind::RightParen | TokenKind::Comma)
        );
        self.report_error(
            ErrorKind::UnexpectedTokens { expected_str: "Expected function argument or `)`" },
            Span::merge(start_span, self.current.get_span())
        );

        if self.is_curr_kind(TokenKind::Comma) {
            self.advance();
        }

        can_parse_more_args
    }

    /// This will parse a function signature `fn ident(args) -> ret_typing` until the left brace `{`.
    ///
    /// It will also synchronize if there's an error parsing the function signature.
    pub(crate) fn parse_fn_signature(
        &mut self,
        parsing_declare_fn: ParsingDeclareFn
    ) -> Option<(ItemType, ParsedFnSignature<'a>)> {
        self.advance();
        let item_type = if self.is_curr_kind(TokenKind::Dot) {
            self.advance();
            let ident_span = self.consume_ident_span("Currently available item types is: `C`");
            assert!(&self.src[ident_span.get_byte_range()] == "C");
            ItemType::C
        } else {
            ItemType::Normal
        };

        let start_span = self.current.get_span();
        let ident_expr = if let Some(ident) = self.try_consume_ident() {
            ident
        } else {
            self.report_error(
                ErrorKind::ExpectedIdent {
                    additional_info: Some("after keyword `fn`"),
                    found: Symbol::new(self.get_lexeme_of_current()),
                },
                self.current.get_span()
            );
            if self.current.get_kind() != TokenKind::LeftParen {
                self.synchronize();
                return None;
            } else {
                // We return a ident node with the span equal to the text of an empty string, only if the next token is `(`,
                // because we hope to parse the function arguments as well
                let byte_start = self.current.get_span().get_byte_start();
                let line = self.current.get_span().get_line();
                let node_id = self.get_ast_node_id();
                Symbol::new_with_node_id("", node_id);
                IdentNode::new(Span::new(byte_start, byte_start, line, 0), node_id)
            }
        };

        let success = self.consume_or_report_error(
            TokenKind::LeftParen,
            ErrorKind::ExpectedDelimeterAfter {
                expected_delim: Delimeter::LeftParen,
                kind: ItemErrorKind::FnName,
            },
            Span::merge(start_span, self.current.get_span())
        );

        if !success {
            self.synchronize();

            return None;
        }

        let mut args = Vec::with_capacity(8);
        while !self.is_eof() && !self.is_curr_kind(TokenKind::RightParen) {
            let arg = {
                match self.current.get_kind() {
                    TokenKind::SmallSelf => {
                        let self_ident = self.consume_self_as_ident_node("Expected `self`");

                        ArgKind::NormalSelf(self.ast_arena.alloc_expr_or_stmt(self_ident))
                    }
                    TokenKind::Star => {
                        self.advance();
                        match self.current.get_kind() {
                            TokenKind::SmallSelf => {
                                let self_ident = self.consume_self_as_ident_node("Expected `self`");
                                ArgKind::PtrSelf(self.ast_arena.alloc_expr_or_stmt(self_ident))
                            }
                            TokenKind::Mut => {
                                self.advance();
                                let self_ident = self.consume_self_as_ident_node("Expected `self`");
                                ArgKind::MutPtrSelf(self.ast_arena.alloc_expr_or_stmt(self_ident))
                            }
                            _ => panic!("Expected `self` or `mut` after `*` in function argument"),
                        }
                    }
                    TokenKind::Mut => {
                        self.advance();
                        let self_ident = self.consume_self_as_ident_node("Expected `self`");
                        ArgKind::MutSelf(self.ast_arena.alloc_expr_or_stmt(self_ident))
                    }
                    TokenKind::Ident => {
                        let start_field_span = self.current.get_span();
                        let arg_ident = self.consume_ident("Expected ident in function args");
                        let arg_typing = self
                            .parse_typing()
                            .expect("Expected type in function args");

                        if
                            let (ParsingDeclareFn::No, Typing::VariadicArgs) = (
                                parsing_declare_fn,
                                arg_typing,
                            )
                        {
                            panic!("Error: Variadic args not allowed in function declaration");
                        }

                        let arg = Field::new(
                            self.ast_arena.alloc_expr_or_stmt(arg_ident),
                            arg_typing,
                            Span::merge(start_field_span, self.current.get_span())
                        );
                        ArgKind::Arg(self.ast_arena.alloc_expr_or_stmt(arg))
                    }
                    _ => {
                        let can_parse_more_args = self.recover_from_arg_parse_error();
                        if can_parse_more_args {
                            continue;
                        } else {
                            break;
                        }
                    }
                }
            };

            if ParsingDeclareFn::Yes == parsing_declare_fn {
                match arg {
                    ArgKind::Arg(arg) => {}
                    _ => panic!("Error: Only normal arguments are allowed in function declaration"),
                }
            }

            args.push(arg);

            if self.is_curr_kind(TokenKind::Comma) {
                self.advance();

                continue;
            } else if self.is_curr_kind(TokenKind::Ident) {
                let symbol = Symbol::new(self.get_lexeme(self.current.get_span()));
                self.report_error(
                    ErrorKind::MissingComma {
                        missing_comma_place: MissingCommaPlace::FnArgs,
                        arg_symbol_after_missing_comma: symbol,
                    },
                    Span::merge(self.prev.get_span(), self.current.get_span())
                );
                continue;
            }

            if self.is_curr_kind(TokenKind::RightParen) {
                break;
            }

            let can_parse_more_args = self.recover_from_arg_parse_error();

            if !can_parse_more_args {
                return None;
            } else {
                continue;
            }
        }

        let args = self.ast_arena.alloc_vec(args);

        let success = self.consume_or_report_error(
            TokenKind::RightParen,
            ErrorKind::ExpectedDelimeterAfter {
                expected_delim: Delimeter::RightParen,
                kind: ItemErrorKind::FnArgs,
            },
            Span::merge(start_span, self.current.get_span())
        );

        if !success {
            self.synchronize();
            return Some((
                item_type,
                ParsedFnSignature {
                    ident: self.ast_arena.alloc_expr_or_stmt(ident_expr),
                    args,
                    ret_typing: None,
                },
            ));
        }

        let return_ty = self.parse_typing();

        self.parsed_fn_count += 1;

        Some((
            item_type,
            ParsedFnSignature {
                ident: self.ast_arena.alloc_expr_or_stmt(ident_expr),
                args,
                ret_typing: return_ty,
            },
        ))
    }

    pub(crate) fn function_statement(&mut self) -> Option<&'a FnItem<'a>> {
        let start_span = self.current.get_span();
        let parsed_fn_sig = self.parse_fn_signature(ParsingDeclareFn::No);

        let (item_type, parsed_fn_sig) = match parsed_fn_sig {
            Some(parsed_fn_sig) => parsed_fn_sig,
            None => {
                return None;
            }
        };

        let body = if let TokenKind::LeftCurly = self.current.get_kind() {
            let body_start_span = self.current.get_span();
            self.advance();
            let body = self.parse_block_as_stmts(StopToken::Token(TokenKind::RightCurly));

            let success = self.consume_or_report_error(
                TokenKind::RightCurly,
                ErrorKind::ExpectedDelimeterAfter {
                    expected_delim: Delimeter::RightCurly,
                    kind: ItemErrorKind::FnBody,
                },
                Span::merge(body_start_span, self.current.get_span())
            );

            if !success {
                self.advance();
            }

            body
        } else {
            self.report_error(
                ErrorKind::FnWithoutBody {
                    symbol: Symbol::from_node_id(parsed_fn_sig.ident.ast_node_id),
                },
                Span::merge(start_span, self.current.get_span())
            );

            self.ast_arena.alloc_vec(Vec::new())
        };

        let fn_stmt = self.ast_arena.alloc_expr_or_stmt(
            FnItem::new(
                parsed_fn_sig.ident,
                body,
                parsed_fn_sig.args,
                parsed_fn_sig.ret_typing,
                item_type,
                Span::merge(start_span, self.current.get_span()),
                self.get_ast_node_id()
            )
        );

        Some(fn_stmt)
    }

    fn return_expr(&mut self) -> Stmt<'a> {
        let start_span = self.current.get_span();
        self.advance();

        let ret_value_expr = if self.get_parse_rule_of_current().prefix_method.is_some() {
            Some(self.parse_expr_and_take(Precedence::PrecAssign.get_next()))
        } else {
            None
        };

        let return_expr = self.ast_arena.alloc_expr_or_stmt(
            ReturnExpr::new(
                ret_value_expr,
                Span::merge(start_span, self.current.get_span()),
                self.get_ast_node_id()
            )
        );

        let expr = Expr::ExprWithoutBlock(ExprWithoutBlock::ReturnExpr(return_expr));

        Stmt::ExprStmt(expr)
    }

    pub(crate) fn mut_stmt(&mut self) -> Stmt<'a> {
        let mut_span = self.prev.get_span();
        self.advance();
        let mut expr_builder = ExprBuilder::new_with_mut_span(self.ast_arena, None, mut_span);
        self.parse_precedence(expr_builder.get_base_prec(), &mut expr_builder);
        let stmt = expr_builder.take_stmt().expect("TODO: Error handling");
        Self::test_and_set_mutable(&stmt).expect("Misuse of mut");
        stmt
    }

    fn test_and_set_mutable(stmt: &Stmt<'a>) -> Result<(), &'b str> {
        match stmt {
            Stmt::DefineStmt(_) => Ok(()),
            Stmt::AssignStmt(_) => Err("Unexpected token `mut` in assignment"),
            Stmt::ExprStmt(_) => Err("Unexpected token `mut` in expression statement"),
            Stmt::ItemStmt(_) => Err("Unexpected token `mut` before item statement"),
        }
    }

    pub(crate) fn expression(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        self.parse_precedence(expr_builder.get_base_prec(), expr_builder);
    }

    /// Parse rule method: `define`
    pub(crate) fn define(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        expr_builder.set_base_prec(Precedence::PrecAssign.get_next());
        self.expression(expr_builder);
        self.def_count += 1;
        expr_builder.emit_define_stmt(self)
    }

    /// Parse rule method: `assign`
    pub(crate) fn assign(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        expr_builder.set_base_prec(Precedence::PrecAssign.get_next());
        self.expression(expr_builder);
        expr_builder.emit_assign_stmt(self)
    }

    /// Parse rule method: `block`
    pub(crate) fn block_expr(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        let block_expr = self.parse_block();
        self.consume(TokenKind::RightCurly, "Expected `}` after block");
        expr_builder.emit_block_expr(block_expr);
    }

    /// Parse rule method: `grouping`, called by prefix `(`
    pub(crate) fn grouping(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        let mut exprs = Vec::new();
        while !self.is_eof() {
            let expr = self.parse_expr_and_take(expr_builder.get_base_prec());
            exprs.push(expr);

            if self.is_curr_kind(TokenKind::Comma) {
                self.advance();
                continue;
            }

            self.consume(TokenKind::RightParen, "Expected ')' after group");
            break;
        }

        expr_builder.emit_grouping_or_tuple_expr(self, exprs)
    }

    /// Parse rule method: `string`
    pub(crate) fn string(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        let mut string_builder = String::with_capacity(16);
        let mut str_len = 1;

        let start_span = self.prev.get_span();

        while !self.is_eof() && !self.is_curr_kind(TokenKind::DoubleQuote) {
            let char = &self.src[self.current.get_span().get_byte_range()];
            if char == "\\" {
                self.advance();

                match &self.src[self.current.get_span().get_byte_range()] {
                    "n" => {
                        string_builder += "\\0A";
                        self.advance();
                        str_len += 1;
                    }
                    "0" => {
                        string_builder += "\\00";
                        self.advance();
                        str_len += 1;
                    }
                    next_char => {
                        string_builder += char;
                        string_builder += next_char;
                        self.advance();
                    }
                }
            } else {
                string_builder += char;
                self.advance();
                str_len += char.len();
            }
        }
        self.consume(TokenKind::DoubleQuote, "Expected `\"` after string");

        let end_span = self.prev.get_span();

        string_builder += "\\00";

        let node_id = self.get_ast_node_id();
        // Creates symbol (to save it to the node id)
        Symbol::new_with_node_id(string_builder.as_str(), node_id);

        let string_expr = StringExpr::new(Span::merge(start_span, end_span), str_len, node_id);

        expr_builder.emit_string_expr(string_expr);
    }

    /// Parse rule method: `call`
    pub(crate) fn call(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        let mut args = Vec::with_capacity(8);
        while !self.is_eof() && !self.is_curr_kind(TokenKind::RightParen) {
            let arg = self.parse_expr_and_take(Precedence::PrecAssign.get_next());
            args.push(arg);

            if self.is_curr_kind(TokenKind::Comma) {
                self.advance();
                continue;
            }

            break;
        }

        self.consume(TokenKind::RightParen, "Expected ')' after function call");

        expr_builder.emit_call_expr(self, args);
    }

    pub(crate) fn pkg_ident(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        let ident = PkgIdentNode::new(self.prev.get_span(), self.get_ast_node_id());

        expr_builder.emit_pkg_ident_expr(ident);
    }

    /// Parse rule method: `ident`
    pub(crate) fn ident(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        let ident_expr = IdentNode::new(self.prev.get_span(), self.get_ast_node_id());
        Symbol::new_with_node_id(
            &self.src[ident_expr.span.get_byte_range()],
            ident_expr.ast_node_id
        );
        expr_builder.emit_ident_expr(ident_expr);
    }

    /// Parse rule method: `null_lit`
    pub(crate) fn null_lit(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        let null_expr = NullExpr::new(Span::dummy(), self.get_ast_node_id());
        expr_builder.emit_null_expr(null_expr);
    }

    /// Parse rule method: `true_lit`
    pub(crate) fn true_lit(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        let bool_expr = BoolExpr::new(true, self.prev.get_span(), self.get_ast_node_id());
        expr_builder.emit_bool_expr(bool_expr)
    }

    /// Parse rule method: `false_lit`
    pub(crate) fn false_lit(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        let bool_expr = BoolExpr::new(false, self.prev.get_span(), self.get_ast_node_id());
        expr_builder.emit_bool_expr(bool_expr)
    }

    /// Parse rule method: `integer`
    pub(crate) fn integer(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        let integer_expr = self.parse_integer_expr();
        expr_builder.emit_integer_expr(integer_expr);
    }

    pub(crate) fn parse_integer_expr(&mut self) -> IntegerExpr {
        let lexeme = self.get_lexeme_of_prev();
        let val = lexeme.parse::<i64>().expect("TODO: Error handling");
        IntegerExpr::new(val, self.prev.get_span(), self.get_ast_node_id())
    }

    /// Parse rule method: `dot_float`
    pub(crate) fn dot_float(&mut self, _expr_builder: &mut ExprBuilder<'a, 'b>) {
        todo!()
    }

    // fn consume_ident_or_report_and_synchronize(
    //     &mut self,
    //     error_kind: ErrorKind,
    //     recover_tokens: &[TokenKind]
    // ) -> Result<IdentNode, bool> {
    //     match self.try_consume_ident() {
    //         Some(ident) => Ok(ident),
    //         None => {
    //             self.report_error(error_kind, self.current.get_span());
    //             let stopped_by_callback = self.synchronize_with_callback(|token_kind|
    //                 recover_tokens.contains(&token_kind)
    //             );
    //         }
    //     }
    // }

    /// Parse rule method: `struct_expr`
    pub(crate) fn struct_expr(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        let mut initialization_fields = Vec::with_capacity(8);
        while !self.is_eof() && !self.is_curr_kind(TokenKind::RightCurly) {
            let start_field_span = self.current.get_span();
            let field_ident = match self.try_consume_ident() {
                Some(ident) => ident,
                None => {
                    self.report_error(
                        ErrorKind::ExpectedIdent {
                            additional_info: Some("in field declaration"),
                            found: Symbol::new(self.get_lexeme_of_current()),
                        },
                        self.current.get_span()
                    );
                    let can_continue = self.synchronize_with_callback(|token_kind|
                        matches!(token_kind, TokenKind::Comma)
                    );

                    if can_continue {
                        if self.is_curr_kind(TokenKind::Comma) {
                            self.advance();
                        }
                        continue;
                    } else {
                        break;
                    }
                }
            };
            let success = self.consume_or_report_error(
                TokenKind::Colon,
                ErrorKind::ExpectedToken {
                    additional_info: Some("after field identifier"),
                    expected: TokenKind::Colon,
                    found: Symbol::new(self.get_lexeme_of_current()),
                },
                self.current.get_span()
            );

            if !success {
                let can_continue = self.synchronize_with_callback(|token_kind|
                    matches!(token_kind, TokenKind::Comma | TokenKind::RightCurly)
                );

                if can_continue && !self.is_curr_kind(TokenKind::RightCurly) {
                    if self.is_curr_kind(TokenKind::Comma) {
                        self.advance();
                    }
                    continue;
                } else {
                    break;
                }
            }

            let expr = self.parse_expr_and_take(Precedence::PrecAssign.get_next());
            let field_init = self.ast_arena.alloc_expr_or_stmt(
                FieldInitialization::new(
                    self.ast_arena.alloc_expr_or_stmt(field_ident),
                    expr,
                    Span::merge(start_field_span, self.current.get_span())
                )
            );
            initialization_fields.push(field_init);

            if self.is_curr_kind(TokenKind::Comma) {
                self.advance();
                continue;
            } else if self.is_curr_kind(TokenKind::Ident) {
                let symbol = Symbol::new(self.get_lexeme_of_prev());
                println!(
                    "Try adding a comma after `{}` at line {}",
                    symbol.get(),
                    self.prev.get_span().get_line()
                );
            }

            break;
        }

        let success = self.consume_or_report_error(
            TokenKind::RightCurly,
            ErrorKind::ExpectedToken {
                additional_info: Some("after struct expression"),
                expected: TokenKind::RightCurly,
                found: Symbol::new(self.get_lexeme_of_current()),
            },
            self.current.get_span()
        );

        if !success {
            self.synchronize_with_callback(|token_kind|
                matches!(token_kind, TokenKind::RightCurly)
            );

            if self.is_curr_kind(TokenKind::RightCurly) {
                self.advance();
            }
        }

        expr_builder.emit_struct_expr(initialization_fields, self);
    }

    /// Parse rule method: `index_expr`
    pub(crate) fn index_expr(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        let index_expr = self.parse_expr_and_take(Precedence::PrecAssign.get_next());

        let success = self.consume_or_report_error(
            TokenKind::RightSquare,
            ErrorKind::ExpectedToken {
                additional_info: Some("after index expression"),
                expected: TokenKind::RightSquare,
                found: Symbol::new(self.get_lexeme_of_current()),
            },
            self.current.get_span()
        );

        if !success {
            self.synchronize();
        }

        expr_builder.emit_index_expr(index_expr, self)
    }

    /// Parse rule method: `dot_expr`
    pub(crate) fn field_expr(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        match self.current.get_kind() {
            TokenKind::Integer => {
                self.advance();
                let integer_expr = self.parse_integer_expr();
                expr_builder.emit_tuple_field_expr(integer_expr, self);
            }
            TokenKind::Ident => {
                let ident_node = self.consume_ident("Unreachable");
                expr_builder.emit_field_expr(ident_node, self);
            }
            _ => {
                self.report_error(
                    ErrorKind::ExpectedToken {
                        additional_info: Some("after `.`"),
                        expected: TokenKind::Ident,
                        found: Symbol::new(self.get_lexeme_of_current()),
                    },
                    self.current.get_span()
                );
                self.synchronize();
                println!("")
            }
        }
    }

    /// Parse rule method: `float`
    pub(crate) fn float(&mut self, _expr_builder: &mut ExprBuilder<'a, 'b>) {
        todo!("Floats not implemented yet: {:?}", self.get_lexeme_of_prev().parse::<f64>())
    }

    /// Parse rule method: `eq`
    pub(crate) fn eq(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        self.binary(expr_builder, BinaryOp::ComparisonOp(ComparisonOp::Eq));
    }

    /// Parse rule method: `ne`
    pub(crate) fn ne(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        self.binary(expr_builder, BinaryOp::ComparisonOp(ComparisonOp::Ne));
    }

    /// Parse rule method: `ge`
    pub(crate) fn ge(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        self.binary(expr_builder, BinaryOp::ComparisonOp(ComparisonOp::Ge));
    }

    /// Parse rule method: `gt`
    pub(crate) fn gt(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        self.binary(expr_builder, BinaryOp::ComparisonOp(ComparisonOp::Gt));
    }

    /// Parse rule method: `le`
    pub(crate) fn le(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        self.binary(expr_builder, BinaryOp::ComparisonOp(ComparisonOp::Le));
    }

    /// Parse rule method: `lt`
    pub(crate) fn lt(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        self.binary(expr_builder, BinaryOp::ComparisonOp(ComparisonOp::Lt));
    }

    /// Parse rule method: `add`
    pub(crate) fn add(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        self.binary(expr_builder, BinaryOp::ArithmeticOp(ArithmeticOp::Add))
    }

    /// Parse rule method: `sub`
    pub(crate) fn sub(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        self.binary(expr_builder, BinaryOp::ArithmeticOp(ArithmeticOp::Sub))
    }

    /// Parse rule method: `mul`
    pub(crate) fn mul(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        self.binary(expr_builder, BinaryOp::ArithmeticOp(ArithmeticOp::Mul))
    }

    /// Parse rule method: `div`
    pub(crate) fn div(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        self.binary(expr_builder, BinaryOp::ArithmeticOp(ArithmeticOp::Div))
    }

    /// Logic of binary parse rule methods
    pub(crate) fn binary(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>, binary_op: BinaryOp) {
        self.parse_precedence(self.get_parse_rule_of_prev().infix_prec.get_next(), expr_builder);

        expr_builder.emit_binary_expr(binary_op, self)
    }

    pub(crate) fn post_inc(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        todo!("Post inc not implemented yet");
        self.advance();
        expr_builder.emit_post_inc_expr(self)
    }

    pub(crate) fn post_dec(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        todo!()
        // self.advance();
        // expr_builder.emit_post_dec_expr(self)
    }

    pub(crate) fn pre_inc(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        todo!()
        // self.advance();
        // expr_builder.emit_pre_inc_expr(self)
    }

    pub(crate) fn pre_dec(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        todo!()
        // self.advance();
        // expr_builder.emit_pre_dec_expr(self)
    }

    /// Parse rule method: `loop_expr`
    pub(crate) fn loop_expr(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        let start_span = self.current.get_span();
        self.consume(TokenKind::LeftCurly, "Expected `{` before loop");
        let block = self.parse_block();
        self.consume(TokenKind::RightCurly, "Expected `}` after loop");

        let loop_expr = self.ast_arena.alloc_expr_or_stmt(
            LoopExpr::new(
                block,
                Span::merge(start_span, self.current.get_span()),
                self.get_ast_node_id()
            )
        );

        expr_builder.emit_loop_expr(loop_expr);
    }

    /// Parse rule method: `if_expr`
    pub(crate) fn if_expr(&mut self, expr_builder: &mut ExprBuilder<'a, 'b>) {
        let if_expr = self.parse_if_expr();

        expr_builder.emit_if_expr(if_expr);
    }

    pub(crate) fn parse_expr_and_take_with_terminate_infix_token(
        &mut self,
        prec: Precedence,
        terminate_infix_token: Option<TokenKind>
    ) -> Expr<'a> {
        let mut expr_builder = ExprBuilder::new(self.ast_arena, terminate_infix_token);
        self.parse_precedence(prec, &mut expr_builder);
        expr_builder.take_expr().expect("TODO:Error handling")
    }

    pub(crate) fn parse_expr_and_take(&mut self, prec: Precedence) -> Expr<'a> {
        let mut expr_builder = ExprBuilder::new(self.ast_arena, None);
        self.parse_precedence(prec, &mut expr_builder);
        expr_builder.take_expr().expect("TODO:Error handling")
    }

    pub(crate) fn parse_if_expr(&mut self) -> &'a IfExpr<'a> {
        let start_span = self.current.get_span();
        let cond = self.parse_expr_and_take_with_terminate_infix_token(
            Precedence::PrecAssign.get_next(),
            Some(TokenKind::LeftCurly)
        );

        let cond_kind = if self.is_curr_kind(TokenKind::Define) {
            // We have an IfDefExpr
            let pat = self.try_as_pat(cond).expect("Expected pattern");

            self.advance();

            let rhs = self.parse_expr_and_take_with_terminate_infix_token(
                Precedence::PrecAssign.get_next(),
                Some(TokenKind::LeftCurly)
            );

            self.def_count += 1;

            CondKind::CondPat(pat, rhs)
        } else {
            CondKind::CondExpr(cond)
        };

        self.consume(TokenKind::LeftCurly, "Expected `{` after if condition");

        let true_block = self.parse_block_as_stmts(StopToken::Token(TokenKind::RightCurly));

        self.consume(TokenKind::RightCurly, "Expected `}` after if block");

        self.advance_if(matches!(self.current.get_kind(), TokenKind::Else | TokenKind::Elif));

        let false_block = match self.prev.get_kind() {
            TokenKind::Else => {
                self.consume(TokenKind::LeftCurly, "Expected `{` after else");
                let block_expr = self.parse_block();
                self.consume(TokenKind::RightCurly, "Expected `}` after else block");
                Some(IfFalseBranchExpr::ElseExpr(block_expr))
            }
            TokenKind::Elif => Some(IfFalseBranchExpr::ElifExpr(self.parse_if_expr())),
            _ => None,
        };

        let if_expr = self.ast_arena.alloc_expr_or_stmt(
            IfExpr::new(
                cond_kind,
                true_block,
                false_block,
                Span::merge(start_span, self.current.get_span()),
                self.get_ast_node_id()
            )
        );

        if_expr
    }

    fn parse_block_as_stmts(&mut self, stop_token: StopToken) -> &'a [Stmt<'a>] {
        let mut stmts = Vec::with_capacity(32);
        while !self.is_eof() {
            if let StopToken::Token(stop_token) = stop_token {
                if self.is_curr_kind(stop_token) {
                    break;
                }
            }

            if let Some(stmt) = self.statement() {
                stmts.push(stmt);
            }
        }

        self.ast_arena.alloc_vec(stmts.into())
    }

    fn parse_block(&mut self) -> &'a BlockExpr<'a> {
        let start_span = self.current.get_span();
        let stmts = self.parse_block_as_stmts(StopToken::Token(TokenKind::RightCurly));

        let block_expr = self.ast_arena.alloc_expr_or_stmt(
            BlockExpr::new(
                stmts,
                Span::merge(start_span, self.current.get_span()),
                self.get_ast_node_id()
            )
        );

        block_expr
    }

    pub(crate) fn synchronize_with_callback(
        &mut self,
        mut callback: impl FnMut(TokenKind) -> bool
    ) -> bool {
        loop {
            if self.is_eof() {
                break false;
            }

            if callback(self.current.get_kind()) {
                break true;
            }

            match self.current.get_kind() {
                | TokenKind::Fn
                | TokenKind::Struct
                | TokenKind::Enum
                | TokenKind::Typedef
                | TokenKind::Declare
                | TokenKind::Import
                | TokenKind::Loop
                | TokenKind::Impl
                | TokenKind::If
                | TokenKind::Elif
                | TokenKind::LeftCurly
                | TokenKind::RightCurly => {
                    break false;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    pub(crate) fn synchronize(&mut self) {
        self.synchronize_with_callback(|_| false);
    }

    pub(crate) fn parse_precedence(
        &mut self,
        mut prec: Precedence,
        expr_builder: &mut ExprBuilder<'a, 'b>
    ) {
        self.advance();

        let parse_rule = self.get_parse_rule_of_prev();

        if let Some(prefix_method) = parse_rule.prefix_method {
            macro_rules! is_terminate_infix_token {
                (current) => {
                    match expr_builder.terminate_infix_token {
                        Some(token_kind) => self.is_curr_kind(token_kind),
                        None => false,
                    }
                };
                (prev) => {
                    match expr_builder.terminate_infix_token {
                        Some(token_kind) => self.prev.get_kind() == token_kind,
                        None => false,
                    }
                };
            }

            if is_terminate_infix_token!(prev) {
                return;
            }

            prefix_method(self, expr_builder);
            loop {
                if
                    (self.prev.get_kind().eq(&TokenKind::Ident) ||
                        self.prev.get_kind().eq(&TokenKind::BigSelf)) &&
                    self.current.get_kind().eq(&TokenKind::LeftCurly) &&
                    !is_terminate_infix_token!(current)
                {
                    self.advance();
                    self.struct_expr(expr_builder);
                }

                while
                    prec <= self.get_parse_rule_of_current().infix_prec &&
                    !is_terminate_infix_token!(current)
                {
                    // If this is true we don't want to parse another `:=` or `=` in expr without block
                    if self.current.get_kind().has_assign_prec() {
                        prec = Precedence::PrecAssign.get_next();
                    }

                    self.advance();

                    if let Some(infix_rule) = self.get_parse_rule_of_prev().infix_method {
                        infix_rule(self, expr_builder);
                    }
                }

                if let Some(postfix_rule) = self.get_parse_rule_of_current().postfix_method {
                    postfix_rule(self, expr_builder);
                    continue;
                } else {
                    break;
                }
            }
        } else {
            self.report_error(
                ErrorKind::ExpectedExprOrItem {
                    found: Symbol::new(self.get_lexeme_of_prev()),
                },
                self.prev.get_span()
            );
            self.synchronize();
        }
    }

    /* Helper methods */
    pub(crate) fn get_ast_node_id(&mut self) -> NodeId {
        let prev = self.next_ast_node_id;
        self.next_ast_node_id = prev + 1;
        NodeId {
            node_id: prev,
            mod_id: self.mod_id,
        }
    }

    /// Converts e.g. `69` into `0.69` (this is a fast version by chat)
    pub(crate) fn _integer_to_dot_float(int: i64) -> f64 {
        // Compute the number of digits using logarithms
        let num_digits = if int == 0 { 1 } else { (int.abs() as f64).log10().ceil() as u32 };

        // Compute the divisor
        let divisor = (10.0_f64).powi(-(num_digits as i32));

        // Compute the float
        (int as f64) * divisor
    }

    pub(crate) fn get_lexeme_of_prev(&self) -> &str {
        self.get_lexeme(self.prev.get_span())
    }

    pub(crate) fn get_lexeme_of_current(&self) -> &str {
        self.get_lexeme(self.current.get_span())
    }

    pub(crate) fn is_eof(&self) -> bool {
        self.lexer.is_eof() && self.current.get_kind() == TokenKind::Eof
    }

    pub(crate) fn is_curr_kind(&self, kind: TokenKind) -> bool {
        self.current.get_kind() == kind
    }

    pub(crate) fn get_lexeme(&self, span: Span) -> &str {
        &self.src[span.get_byte_start()..span.get_byte_end()]
    }

    pub(crate) fn advance(&mut self) {
        self.prev = self.current;
        self.current = self.lexer.scan_token();
    }

    pub(crate) fn advance_if(&mut self, cond: bool) {
        if cond {
            self.advance();
        }
    }

    pub(crate) fn consume_or_report_error(
        &mut self,
        kind: TokenKind,
        error_kind: ErrorKind,
        span: Span
    ) -> bool {
        if self.current.get_kind() == kind {
            self.advance();
            true
        } else {
            self.report_error(error_kind, span);
            false
        }
    }

    pub(crate) fn consume(&mut self, kind: TokenKind, err_msg: &str) {
        if self.current.get_kind() == kind {
            self.advance();
        } else {
            panic!("Error: {}", err_msg);

            // self.advance();
        }
    }

    pub(crate) fn consume_self_as_ident_node(&mut self, err_msg: &str) -> IdentNode {
        match self.current.get_kind() {
            TokenKind::SmallSelf => {
                let ident_node = self.make_ident_node_from_current();
                self.advance();
                ident_node
            }
            _ => panic!("{}", err_msg),
        }
    }

    pub(crate) fn try_consume_self_as_ident_node(&mut self) -> Option<IdentNode> {
        match self.current.get_kind() {
            TokenKind::SmallSelf => {
                let ident_node = self.make_ident_node_from_current();
                self.advance();
                Some(ident_node)
            }
            _ => None,
        }
    }

    pub(crate) fn consume_ident(&mut self, err_msg: &str) -> IdentNode {
        match self.current.get_kind() {
            TokenKind::Ident => {
                let ident_node = self.make_ident_node_from_current();
                self.advance();
                ident_node
            }
            _ => panic!("{}", err_msg),
        }
    }

    pub(crate) fn try_consume_ident(&mut self) -> Option<IdentNode> {
        match self.current.get_kind() {
            TokenKind::Ident => {
                let ident_node = self.make_ident_node_from_current();
                self.advance();
                Some(ident_node)
            }
            _ => None,
        }
    }

    pub(crate) fn consume_pkg_ident(&mut self, err_msg: &str) -> PkgIdentNode {
        match self.current.get_kind() {
            TokenKind::Pkg => {
                let ident_node = PkgIdentNode::new(self.current.get_span(), self.get_ast_node_id());
                self.advance();
                ident_node
            }
            _ => panic!("{}", err_msg),
        }
    }

    pub(crate) fn consume_ident_span(&mut self, err_msg: &str) -> Span {
        match self.current.get_kind() {
            TokenKind::Ident => {
                let span = self.current.get_span();
                self.advance();
                span
            }
            _ => panic!("{}", err_msg),
        }
    }

    fn make_ident_node_from_current(&mut self) -> IdentNode {
        let ident_node = IdentNode::new(self.current.get_span(), self.get_ast_node_id());
        let lexeme = self.get_lexeme(ident_node.span);
        Symbol::new_with_node_id(lexeme, ident_node.ast_node_id);
        ident_node
    }

    pub(crate) fn get_parse_rule_of_current(&self) -> &ParseRule {
        &PARSE_RULES[self.current.get_kind() as usize]
    }

    pub(crate) fn get_parse_rule_of_prev(&self) -> &ParseRule {
        &PARSE_RULES[self.prev.get_kind() as usize]
    }
}
