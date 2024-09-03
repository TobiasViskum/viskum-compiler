macro_rules! make_parse_rule {
    (
        $kind:ident,
        $(
            $tkind:ident = {
                ($prefix_method:ident $prefix_prec:ident),
                ($infix_method:ident $infix_prec:ident),
                ($postfix_method:ident $postfix_prec:ident)
            }
        ),+
    ) => {
        match $kind {
            $(
                TokenKind::$tkind => ParseRule {
                    prefix_method: make_parse_rule::make_parse_rule!(@construct_rule $prefix_method),
                    prefix_prec: make_parse_rule::make_parse_rule!(@construct_prec $prefix_prec),
                    infix_method: make_parse_rule::make_parse_rule!(@construct_rule $infix_method),
                    infix_prec: make_parse_rule::make_parse_rule!(@construct_prec $infix_prec),
                    postfix_method: make_parse_rule::make_parse_rule!(@construct_rule $postfix_method),
                    postfix_prec: make_parse_rule::make_parse_rule!(@construct_prec $postfix_prec),
                }
            ),+
        }
    };

    (@construct_rule None) => {
        None
    };

    (@construct_rule $method_name:ident) => {
        Some(|c, expr_builder| c.$method_name(expr_builder))
    };

    (@construct_prec None) => {
        Precedence::PrecNone
    };

    (@construct_prec $prec:ident) => {
        Precedence::$prec
    };
}

pub(crate) use make_parse_rule;
