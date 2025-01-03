use std::sync::LazyLock;

use crate::{ with_global_session, NodeId };

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

    pub fn can_be_constant(&self) -> bool {
        self.get()
            .chars()
            .all(|c| (c.is_uppercase() || c == '_'))
    }

    pub fn new_with_node_id(str: &str, node_id: NodeId) -> Self {
        with_global_session(|globals| {
            let symbol = globals.intern_str(str);
            globals.insert_symbol_to_node_id(node_id, symbol);
            symbol
        })
    }

    pub fn get(&self) -> &'static str {
        with_global_session(|globals| globals.get_str(self))
    }

    pub fn from_node_id(node_id: NodeId) -> Self {
        with_global_session(|globals| globals.get_symbol_from_node_id(node_id))
    }

    pub(crate) fn from_id(id: usize) -> Self {
        Self { id: SymbolId(id as u32) }
    }
}

pub static BIG_SELF_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("Self"))
});
pub static SMALL_SELF_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("self"))
});
pub static INT_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("int"))
});
pub static INT8_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("int8"))
});
pub static INT16_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("int16"))
});
pub static INT32_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("int32"))
});
pub static INT64_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("int64"))
});
pub static UINT_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("uint"))
});
pub static UINT8_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("uint8"))
});
pub static UINT16_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("uint16"))
});
pub static UINT32_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("uint32"))
});
pub static UINT64_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("uint64"))
});
pub static FLOAT_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("float"))
});
pub static FLOAT_32_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("float32"))
});
pub static FLOAT_64_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("float64"))
});
pub static BOOL_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("bool"))
});
pub static CHAR_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("char"))
});
pub static VOID_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("void"))
});
pub static IMPORT_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("import"))
});
pub static FROM_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("from"))
});
pub static EXPORT_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("export"))
});
pub static FN_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("fn"))
});
pub static DECLARE_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("declare"))
});
pub static MUT_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("mut"))
});
pub static IMPL_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("impl"))
});
pub static STRUCT_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("struct"))
});
pub static ENUM_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("enum"))
});
pub static TYPEDEF_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("typedef"))
});
pub static NULL_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("null"))
});
pub static TRUE_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("true"))
});
pub static FALSE_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("false"))
});
pub static IF_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("if"))
});
pub static ELSE_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("else"))
});
pub static ELIF_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("elif"))
});
pub static WHILE_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("while"))
});
pub static LOOP_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("loop"))
});
pub static BREAK_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("break"))
});
pub static CONTINUE_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("continue"))
});
pub static RETURN_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("return"))
});
pub static STR_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("str"))
});
pub static MAIN_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("main"))
});
pub static PKG_SYMBOL: LazyLock<Symbol> = LazyLock::new(|| {
    with_global_session(|globals| globals.intern_str("pkg"))
});
