struct Point {
    x MyInt,
    y Int,
    z Int,
    data (Int, Int, (Bool, Bool, Int)),
}

typedef MyInt Int

fn add(x Int, y Int) Int {
    ret x + y
}

fn main() {
    a := do
        k := 0
        a := 2
        b := 2
        a + k
    end

    abc := if true then 1 else 0 end

    point := Point { z: 5, y: 4, x: 3, data: (9, 4, (true, false, 8)) }

    boolean := point.data.2.1

    tuple := (7, 8)

    a := (2 + 3) * 9 + tuple.1 + point.data.2.2
    b := 6 - (1 + a)

    k := if boolean then
        l := 2
        2
    elif a == 9 then
        ll := 3
        99
    else
        lll := 4
        7
    end

    mut c := a + b + k

    c = 2

    cond := if true == true then 1 else c end

    c = 928 + c

    mut i := 0
    if i == 10 then
        i = i + 1
    end

    result := loop
        i = i + 1
        if i == 10 then
            break
        end
    end
}