struct Point { 
    x int,
    y int,
    z int,
}

impl Point {
    fn new(x int, y int, z int) Self {
        ret Self {
            x: x,
            y: y,
            z: z
        }
    }

    fn sum(self) int {
        ret self.x + self.y + self.z
    }
}

fn doSomething() int {
    ret 2 * 3
}