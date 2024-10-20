typedef Data (Int, Int, (Bool, Bool, Int))

struct Point {
    x Int,
    y Int,
    z Int,
    data Data,
    addFn fn(Int, Int) Int,
}

fn sum(point Point) Int {
    ret point.x + point.y + point.z
}

fn iReturnVoid() Void { }

fn add(x Int, y Int) Int {
    fn doAddition(x Int, y Int) Int {
        ret x + y
    }

    ret doAddition(x, y)
}

fn returnFunction() fn(Int, Int) Int {
    fn addTwo(x Int, y Int) Int {
        ret x + y
    }

    ret addTwo
}

fn fib(n Int) Int {
    if n <= 1 { ret n + 0 }
    ret fib(n - 2) + fib(n - 1)
}


enum Option {
    Some(Int),
    None
}


fn main() {

    myEnum := Option.Some(2)

    if Option.Some(value) := myEnum {
        
    }

    fib(45)

    a := {
        k := 0
        a := 2
        b := 2
        a + k
    }

    abc := if true { 1 } else { 0 }

    point := Point { z: 5 + (abc + returnFunction()(2, 3)), y: 4, x: 3, data: (9, 4, (true, false, 8)), addFn: add }

    boolean := point.data.2.1

    tuple := (7, 8)

    a := (2 + 3) * 9 + tuple.1 + point.data.2.2 + point.addFn(2, 3) + sum(point)
    b := 6 - (1 + a)

    adkg := iReturnVoid()

    k := if boolean {
        l := 2
        2
    } elif a == 9 {
        ll := 3
        99
    } else {
        lll := 4
        7
    }

    mut c := a + b + k

    c = 2

    cond := if true == true { 1 } else { c }

    c = 928 + c

    mut i := 0
    if i == 10 {
        i = i + 1
    }

    result := loop {
        if i++ == 100 { break }
    }
}