#[allow(private_bounds)]
pub trait AstState: PrivateAstState {
    type ThisState;
    type NextState;

    fn get_next_state() -> Self::NextState;
}

make_ast_states! {

    #[doc = "Ast state 0: Unvalidated"];
    0: AstBuildingQuerySystem -> AstUnvalidated;


    #[doc = "Ast state 1: Unvalidated"];
    1: AstUnvalidated -> AstResolved;

    #[doc = "Ast state 2: Name resolution is done"];
    2: AstResolved -> AstTypeChecked;


    #[doc = "Ast state 3: Type checking is done"];
    3: AstTypeChecked -> AstValidated;


    #[doc = "Ast state 4: Fully validated"];
    4: AstValidated -> AstValidated;
}

macro_rules! make_ast_states {
    ($(#[doc = $doc:literal]; $idx:literal: $state:ident -> $next_state:ident;)* $(,)?) => {
        /// Makes sure, that only this file can implement AstState for structs
        trait PrivateAstState: Sized {}


        $(
            paste::paste! {
                #[doc = $doc]
                pub type [<AstState $idx>] = $state;
            }

            #[derive(Debug, Clone, Copy)]
            #[doc = $doc]
            pub struct $state;

            impl AstState for $state {
                type ThisState = $state;
                type NextState = $next_state;


                fn get_next_state() -> Self::NextState {
                    assert_eq!(std::mem::size_of::<$state>(), 0);
                    assert_eq!(std::mem::size_of::<$next_state>(), 0);
                    $next_state
                }
            }
            impl PrivateAstState for $state {}
        )*
    };
}

pub(self) use make_ast_states;
