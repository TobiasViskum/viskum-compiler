# Viskum Compiler

## How the compilation works

### Multiple packages compilation

- Multiple packages are not yet supported

### Single package compilation

- First it parses the entry-package provided in the command line (e.g. in dir/file.vs, then all files in dir is part of the package)
    - This stage is multihreaded

- AST: Now it does a name resolution of all items that can be forward declared e.g. structs, enums, functions etc.
    - This stage is multithreaded per AST

- Syncs the result of the previous phase into lookup tables, owned by the Resolver, which is all items accessible in the package

- AST: Next phase is full name resolution of all top-level items like variables, and ensures that they are accessible in their scopes and/or contexts. It also figures out the types of all forward declared items
    - This stage is multithreaded per AST

- Syncs the result (namebindings of forward declared items, implementations of a type) into lookup tables like before

- AST: Last phase is a full type checking
    - This stage is multithreaded per AST

- Syncs the result of typechecking (def ids, types and namebindings to all node ids) into lookup tables in the Resolver

- Before compiling into the ICFG the resolver is deallocated, but neccesary lookup tables are saved into a single struct, which is used during the construction of the ICFG

- Constructs the icfg (which is mostly a list of CFGs alongside some global data), where a CFG is made from each function in the program (no matter the scoping or context)
    - This stage is multithreaded per function

- There's no analysis of the ICFG for now, but it's coming in the future

- Lastly codegen which is pretty much just each cfg that get's converted into LLVM IR alongside some global variables
    - This stage is multithreaded per CFG

## Clang and llvm
- https://mcyoung.xyz/2023/08/01/llvm-ir/
- llvm switch instruction: https://llvm.org/docs/LangRef.html#switch-instruction

## Compiler IRs

Parser produces an AST. This Ast is used for name resolution and type checking
Ast is produced into an ICFG (inter-procedual control flow graph). In the ICFG all further analysis is done
- (drop analysis, pointer analysis, lifetime analysis, dead code analysis)

## Todo: Intern types
- This will make comparison between types MUCH faster.
- Current implementation example:
`
typedef MyTuple (MyInt, MyInt)
typeDef MyInt Int

struct Data {
    data MyTuple
}

Data { data: (2, 3) }
`
This will first have lookup the type of each user defined type (MyTuple and MyInt), which eventually resolves to (Int, Int)
However if I instead is able to type interning, then the type of MyTuple will just be a TyId, which is equal to the TyId of (Int, Int). Then comparisons is much faster, right now the comparison is a recursive function. Instead it could be a simple number comparison. There benefits of this will become much more noticeable as the program gets bigger. 

## Todo

### Fix memory issues
- Heap allocated data (like vecs) is allocated in bump arenas (in TyCtx::intern_many) which never frees the original memory even though it was allocated in the arena
- Usage of TyCtx::intern_many with non-types (e.g. namebindings) doesn't live for the entire program, but the heap allocated data is never dropped because TyCtx::intern_many returns a static reference (this happens in the resolver)
- In the resolver: Identical Namebindings are right now being stored in both pkg_def_id_to_name_binding and in the AstTypeChecker (created in AstResolver). The same thing applies to pkg_symbol_to_def_id

### Other todos
- Remove most of the automatic type coercion (buggy and right now a bit unpredictable). Reimplement once ICFG analysis is in place and there's a difference between .vs and .vsc files

## Types

### Primary types

#### Number types

Num can be either `Int`, `Uint` or `Float`

Num = Num of size 32 bits
Num.(8 | 16 | 32 | 64) = Num of given size in bits

#### Strings

String = The only string type for now

### Array types
[T] = array of unkown static size (allocated in an arena setup during compilation)
[T; N] = fixed size array
T[] = dynamic array

### Hashmaps
T->K = T is the key type and K is the value type
std.Hashmap< T, K > can also be used

## Notes

Doubles a number: x >> 1

Halfs a number: x << 1

## Function pattern matching
