#[allow(private_bounds)]
pub trait AstState: PrivateAstState {}
trait PrivateAstState: Sized {}

/// Ast state 1: Unvalidated
#[derive(Debug, Clone, Copy)]
pub struct AstUnvalidated;

/// Ast state 2: Validated
#[derive(Debug, Clone, Copy)]
pub struct AstValidated;

impl AstState for AstUnvalidated {}
impl PrivateAstState for AstUnvalidated {}

impl AstState for AstValidated {}
impl PrivateAstState for AstValidated {}

/// The private generic used in the arena
pub struct AstArenaState;

impl AstState for AstArenaState {}
impl PrivateAstState for AstArenaState {}
