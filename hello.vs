declare fn.C exit(status Int)
declare fn.C printf(num *Int) Int

typedef Data (Int, Int, (Int, Int))

struct Point {
    x Int,
    y Int,
    z Int,
    data Data
}

fn givemeint() Int {
    ret 2
}

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

fn sum(point Point) Int {
    ret point.x + point.y + point.z
}

fn main() {
    a := 2
    b := 2
    mut c := a + b

    point := Point { x: 1, y: 2, z: 3, data: (1, 2, (3, 4)) }

    tuple := (2, 3)

    c = 2 + point.z + tuple.0 * 2 + point.data.2.1

    k := c + 2 + givemeint() + add(2, 3) + sum(point) + returnFunction()(2, 3)
}