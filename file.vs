declare fn.C realloc(prevItems *Int, bytesize Int) [*]Int
declare fn.C malloc(bytesize Int) [*]Int
declare fn.C socket(domain Int, type Int, protocol Int) Int
declare fn.C exit(status Int)

typedef Data (Int, Int, (Bool, Bool, Int))

struct.C Vec {
    len Int,
    cap Int,
    items [*]Int,
}

fn.C newVec() Vec {
    ret Vec {
        len: 0,
        cap: 0,
        items: null
    }
}

fn.C push(vec *mut Vec, item Int) {
    if vec.len == vec.cap {
        vec.cap = if vec.cap == 0 { 2 } else { vec.cap * 2 }
        size := vec.cap * 4
        vec.items = if vec.len == 0 { malloc(size) } else { realloc(vec.items, size) }
    }

    vec.items[vec.len] = item
    vec.len = vec.len + 1
}

fn.C pop(vec *mut Vec) Int {
    vec.len = vec.len - 1
    ret vec.items[vec.len]
}

fn.C getLastMut(vec *mut Vec) *mut Int {
    ret vec.items[vec.len - 1]
}

fn.C getLast(vec *Vec) *Int {
    ret vec.items[vec.len - 1]
}

fn fib(n Int) Int {
    if n <= 1 { ret n }
    ret fib(n - 2) + fib(n - 1)
}


fn main() {
    runTest()

    fmtStr := "%d"


    mut vec := newVec()
    push(vec, 0)
    last := getLastMut(vec)
    last = 0
    poppedItem := pop(vec)

    push(vec, 0)
    last := getLast(vec)

    runTests()

    exit(last)
}

fn runTests() {
    {
        mut vec := newVec()
        push(vec, 0)
        pushedItem := vec.items[0]
        if pushedItem != 0 {
            exit(1)
        }
    }
    {

    }
}