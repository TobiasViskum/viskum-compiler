typedef Data (Int, Int, (Bool, Bool, Int))

struct Point {
    x Int,
    y Int,
    z Int,
    data Data,
    addFn fn(Int, Int) Int,
}

fn iReturnVoid() Void { }

a := 2 + 1

fn addNoReturn(x Int, y Int) Int {
    ret x + y
}

fn returnFunction() fn(Int, Int) Int {
    ret addNoReturn
}

fn main() {
    a := do
        k := 0
        a := 2
        b := 2
        a + k
    end

    abc := if true then 1 else 0 end


    point := Point { z: 5 + (abc + returnFunction()(2, 3)), y: 4, x: 3, data: (9, 4, (true, false, 8)), addFn: addNoReturn }

    boolean := point.data.2.1

    tuple := (7, 8)

    a := (2 + 3) * 9 + tuple.1 + point.data.2.2 + point.addFn(2, 3)
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


