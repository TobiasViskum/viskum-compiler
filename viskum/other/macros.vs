from std.macro import *

macro requiresFree(derived mut Derived) {
    fn onFieldDerive(derivedField mut DerivedField) {
        if Type.Ptr(_) := derivedField.getType() {
            ret CompileError("Expected type to be a pointer", derivedField.getType().getSpan())
        }

        structName := derivedField.getStruct().name

        implTokens := @write {
            impl Drop for $[structName] {
                fn onDrop(*self) {
                    free($[derivedField])
                }
            }
        }

        derivedField.getMutStruct().impl(implTokens)
    }
}

macro write(tokens Tokens) Tokens {
    ret tokens
}