# Viskum Compiler

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

Doubles a number:
x >> 1

Halfs a number:
x << 1

point := Point(x: 2, y: 3)

point := Point { x: 2, y: 3 }


## Function patter matching

## Example of Low level control


struct.C Vec {
    len Int,
    cap Int,
    items [*]Int,

    fn new() Self {
        Self { len: 0, cap: 0, items: null }
    }

    fn push(*mut self, item Int) {
        if self.len == self.cap {
            if self.cap == 0 { 
                self.cap = 2
                items := match malloc(self.cap * Int.SIZE) {
                    Option.Some(items) -> items,
                    Option.None -> exit(1)
                }
                self.items = items
            } else { 
                self.cap *= 2
                newItems := match realloc(self.cap * Int.SIZE) {
                    Option.Some(newItems) -> newItems,
                    Option.None -> exit(1)
                }
                self.items = newItems
            }
        }

        self.items[len++] = item   
    }

    fn pop(*mut self) Option<Int> {
        self.items[--self.len]
    }
}