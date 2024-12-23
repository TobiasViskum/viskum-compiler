/// The only purpose of AstState is to expose different methods on the
/// Ast and in the Visitor based on which state it's in
///
/// All structs that implements AstState is zero-sized, which is ensured
/// by making AstState depend on a private bound
#[allow(private_bounds)]
pub trait AstState: PrivateAstState {
    /// Only exists because it's easier to match the current state by
    /// ThisState, compared to matching it by the next state
    type ThisState;

    /// Used only the next_state function in Ast
    type NextState;

    /// Gets the next state object
    fn get_next_state() -> Self::NextState;
}

make_ast_states! {
    #[doc = "Ast state 1: Unvalidated. This pass will resolve all forward declarations"];
    0: AstUnvalidated -> AstPartlyResolved;

    #[doc = "Ast state 2: Partly Resolved. This pass will resolve all top level names"];
    1: AstPartlyResolved -> AstResolved;

    #[doc = "Ast state 3: Resolved. This pass will type check the Ast"];
    2: AstResolved -> AstTypeChecked;

    #[doc = "Ast state 4: Type checking is done"];
    3: AstTypeChecked -> AstValidated;

    #[doc = "Ast state 5: Fully validated"];
    4: AstValidated -> AstValidated;
}

macro_rules! make_ast_states {
    ($(#[doc = $doc:literal]; $idx:literal: $state:ident -> $next_state:ident;)* $(,)?) => {
        /// Makes sure, that only this file can implement AstState for structs
        trait PrivateAstState: Sized {}

        $(
            paste::paste! {
                /// This is just a type that makes it easier to target a specific
                /// AstState by number instead of by name
                /// 
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
