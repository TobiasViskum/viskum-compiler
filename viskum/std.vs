struct Point { x int, y int, z int }

impl Point {
    fn sum(self) int {
        ret self.x + self.y + self.x
    }
}

 maybe := Option.Some(2)

    if Option.Some(x) := maybe {
        printf("Hello, value is: %d\n", x)

        printf("Hello, value is: %d + 1 = %d\n", x, x + 1)
    }
