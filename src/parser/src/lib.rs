use ast_arena::AstArena;
use lexer::Lexer;
use make_parse_rule::make_parse_rule;
use op::BinaryOp;
use token::TokenKind;
mod make_parse_rule;

pub type ParseRuleMethod = fn(&mut Parser);

const PARSE_RULE_COUNT: usize = enum_iterator::cardinality::<TokenKind>();

pub struct ParseRule {
    pub prefix_method: Option<ParseRuleMethod>,
    pub prefix_prec: Precedence,
    pub infix_method: Option<ParseRuleMethod>,
    pub infix_prec: Precedence,
    pub postfix_method: Option<ParseRuleMethod>,
    pub postfix_prec: Precedence,
}

pub enum Precedence {
    PrecGroup,
    PrecCall,
    PrecNone,
}

pub struct Parser<'a, 'b> where 'a: 'b {
    parse_rules: [ParseRule; PARSE_RULE_COUNT],
    lexer: Lexer<'b>,
    ast_arena: &'b AstArena<'a>,
}

impl<'a, 'b> Parser<'a, 'b> where 'a: 'b {
    pub fn new(file_content: &'a str, ast_arena: &'b AstArena<'a>) -> Self {
        Self {
            parse_rules: Self::create_parse_rules(),
            lexer: Lexer::new(file_content),
            ast_arena,
        }
    }

    pub fn parse(&mut self) {
        self.expression_statement()
    }

    pub(crate) fn expression_statement(&mut self) {
        self.parse_precedence()
    }

    pub(crate) fn parse_precedence(&mut self) {}

    pub(crate) fn grouping(&mut self) {}

    pub(crate) fn integer(&mut self) {}

    pub(crate) fn add(&mut self) {}

    pub(crate) fn binary(&mut self, binary_op: BinaryOp) {}

    pub(crate) fn get_parse_rule(&self, tkind: TokenKind) -> &ParseRule {
        &self.parse_rules[tkind as usize]
    }

    pub(crate) fn create_parse_rules() -> [ParseRule; PARSE_RULE_COUNT] {
        array_init::array_init(|i| {
            let kind = enum_iterator::all::<TokenKind>().nth(i).unwrap();
            Self::create_parse_rule(kind)
        })
    }

    pub(crate) fn create_parse_rule(kind: TokenKind) -> ParseRule {
        make_parse_rule!(kind,
            LeftParen   = { (grouping None),    (None None),    (None None) },
            RightParen  = { (None None),        (None None),    (None None) },
            Eq          = { (None None),        (None None),    (None None) },
            Ne          = { (None None),        (None None),    (None None) },
            Ge          = { (None None),        (None None),    (None None) },
            Gt          = { (None None),        (None None),    (None None) },
            Le          = { (None None),        (None None),    (None None) },
            Lt          = { (None None),        (None None),    (None None) },
            Plus        = { (None None),        (add None),    (None None) },
            Minus       = { (None None),        (None None),    (None None) },
            Star        = { (None None),        (None None),    (None None) },
            Slash       = { (None None),        (None None),    (None None) },
            Colon       = { (None None),        (None None),    (None None) },
            Define      = { (None None),        (None None),    (None None) },
            
            Integer     = { (integer None),        (None None),    (None None) },
            Float       = { (None None),        (None None),    (None None) },
            
            Ident       = { (None None),        (None None),    (None None) },
            
            Def         = { (None None),        (None None),    (None None) },
            Class       = { (None None),        (None None),    (None None) },
            While       = { (None None),        (None None),    (None None) },
            Loop        = { (None None),        (None None),    (None None) },
            Do          = { (None None),        (None None),    (None None) },
            End         = { (None None),        (None None),    (None None) },
            Eof         = { (None None),        (None None),    (None None) }
        )
    }
}
