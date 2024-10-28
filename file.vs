declare fn.C realloc(prevItems *Int, bytesize Int) [*]Int
declare fn.C malloc(bytesize Int) [*]Int
declare fn.C socket(domain Int, type Int, protocol Int) Int
declare fn.C exit(status Int)
declare fn.C printf(fmt Str, rest ...) Int

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

fn.C printInt(integer Int) {
    printf("Value is: %i\n\0", integer)
}


fn main() {
    printInt(2)

    runTests()
    
    mut vec := newVec()
    push(vec, 0)
    last := getLastMut(vec)
    last = 0
    poppedItem := pop(vec)

    push(vec, 0)
    last := getLast(vec)

    exit(last)
}

fn.C printStr(s Str) {
    printf("%s\n\0", s)
}

fn assertInt(x Int, y Int) {
    if x != y {
        printf("Assert failed. Info:\n%d != %d\n\0", x, y)
        exit(1)
    }
}

fn runTests() {
    assertInt(1, 2)
    mut vec := newVec()
    push(vec, 2)
    pushedItem := vec.items[0]
    if pushedItem != 2 {
        printf("%d != %d\n\0", pushedItem + 0, 0)
        exit(1)
    }

    printVec(vec)

    push(vec, 4)
    
    printVec(vec)

    pop(vec)

    printVec(vec)
}

fn.C printVec(vec *Vec) {
    printf("Items in vec:\n\0")
    mut i := 0
    loop {
        printf("vec[%d] = %d\n\0", i, vec.items[i] + 0)
        if i + 1 == vec.len { break }
        i = i + 1
    }
}