macro_rules! make_token_kind {
    (
        $(
            [$tkind:ident, doc = $doc:literal] = {
                ($prefix_method:ident $prefix_prec:ident),
                ($infix_method:ident $infix_prec:ident),
                ($postfix_method:ident $postfix_prec:ident)
            }
        ),*
        $(,)?
    ) => {
        const PARSE_RULE_COUNT: usize = make_token_kind::make_token_kind!(@count $($tkind),*);

        #[derive(Eq, PartialEq, Clone, Copy, Debug)]
        pub enum TokenKind {
            $(
                #[doc = $doc]
                $tkind,
            )*
        }

        impl From<TokenKind> for usize {
            fn from(value: TokenKind) -> Self {
                let mut index = 0;
                match value {
                    $(
                        TokenKind::$tkind => {
                            index += 1;
                            index
                        }
                        
                    )+
                }
            }
        }

        impl<'a> Parser<'a> {
            fn create_parse_rules() -> [ParseRule; PARSE_RULE_COUNT] {
                const D: [ParseRule; PARSE_RULE_COUNT] = [
                    $(
                        ParseRule {
                            prefix_method: make_token_kind::make_token_kind!(@construct_rule $prefix_method),
                            prefix_prec: make_token_kind::make_token_kind!(@construct_prec $prefix_prec),
                            infix_method: make_token_kind::make_token_kind!(@construct_rule $infix_method),
                            infix_prec: make_token_kind::make_token_kind!(@construct_prec $infix_prec),
                            postfix_method: make_token_kind::make_token_kind!(@construct_rule $postfix_method),
                            postfix_prec: make_token_kind::make_token_kind!(@construct_prec $postfix_prec),
                        }
                    ),*
                ];
                return D
            }
        }
    };

    (@count $($tkind:ident),*) => {
        [$(stringify!($tkind)),*].len()
    };

    (@construct_rule None) => {
        None
    };

    (@construct_rule $method_name:ident) => {
        Some(|c /*, args*/| c.$method_name(/* args */))
    };

    (@construct_prec None) => {
        Precedence::PrecNone
    };

    (@construct_prec $prec:ident) => {
        Precedence::$prec
    };

    (@index $($tkind:ident),*; $target:ident) => {
        {
            let mut index = 0;
            $(
                if stringify!($tkind) == stringify!($target) {
                    break;
                }
                index += 1;
            )*
            index
        }
    };
}

pub(crate) use make_token_kind;
