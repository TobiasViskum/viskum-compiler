struct Point
    x Int,
    y Int,
    z Int,
end

a := do
    k := 0
    a := 2
    b := 2
    a + k
end

abc := if true then 1 else 0 end

point := Point { z: 5, y: 4, x: 3 }

boolean := true

tuple := (7, 8)

a := (2 + 3) * 9 + tuple.1 + point.z
b := 6 - (1 + a)

k := if a == 2 then
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