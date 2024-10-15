use crate::with_global_session;

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub(crate) struct SymbolId(pub u32);

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct Symbol {
    pub(crate) id: SymbolId,
}

impl Symbol {
    pub fn new(str: &str) -> Self {
        with_global_session(|globals| globals.intern_str(str))
    }

    pub fn get(&self) -> &'static str {
        with_global_session(|globals| globals.get_str(self))
    }

    pub(crate) fn from_id(id: usize) -> Self {
        Self { id: SymbolId(id as u32) }
    }
}
