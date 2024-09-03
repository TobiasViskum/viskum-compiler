use lexer::Lexer;
use make_parse_rule::make_parse_rule;
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

pub struct Parser<'a> {
    parse_rules: [ParseRule; PARSE_RULE_COUNT],
    lexer: Lexer<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(file_content: &'a str) -> Self {
        Self {
            parse_rules: Self::create_parse_rules(),
            lexer: Lexer::new(file_content),
        }
    }

    pub fn parse(&mut self) {}

    pub(crate) fn grouping(&mut self) {}

    pub(crate) fn call(&mut self) {}

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
            Plus        = { (None None),        (None None),    (None None) },
            Minus       = { (None None),        (None None),    (None None) },
            Star        = { (None None),        (None None),    (None None) },
            Slash       = { (None None),        (None None),    (None None) },
            Colon       = { (None None),        (None None),    (None None) },
            Define      = { (None None),        (None None),    (None None) },
            
            Integer     = { (None None),        (None None),    (None None) },
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
