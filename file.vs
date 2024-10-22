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


enum AnotherEnum {
    FirstVariant,
    SecondVariant(Int, Bool, Int),
    ThirdVariant(Int, (Int, Int))
}

enum DepthOne {
    VariantOne(Int, AnotherEnum),
    VariantTwo(Int, Int)
}

enum Option {
    Some(Int),
    None
}

fn main() {
    mut matched := false

    anotherEnum1 := DepthOne.VariantOne(0, AnotherEnum.SecondVariant(2, false, 8))

    anotherEnum2 := DepthOne.VariantTwo(4, 9)

    if DepthOne.VariantOne(k, AnotherEnum.SecondVariant(x, y, z)) := anotherEnum1 {
        abcabc := x + z
        matched = true
    }


    myEnum := Option.Some(2)

    if Option.Some(xk) := myEnum {
        abcabc := xk
        matched = true
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

    k := if boolean == true {
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