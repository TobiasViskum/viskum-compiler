# Viskum Compiler

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

## Notes

Doubles a number:
x >> 1

Halfs a number:
x << 1

point := Point(x: 2, y: 3)

point := Point { x: 2, y: 3 }
