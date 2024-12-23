declare fn socket(domain int, type int, protocol int) int

struct HttpServer {

}

fn calcPos(pos mut Pos) {
    pos.x++
    pos.y++
    pos.z++
}

fn displayPos(pos Pos) {
    print("X: {pos.x}, Y: {pos.y}, Z: {pos.Z}")
}

struct Pos {
    x int,
    y int,
    z int
}

impl Pos {
    fn update(mut self) {
        calcPos(self)
    }
}

mut pos := Pos { x: 0, y: 0, z: 0 }

calcPos(mut pos)


export HttpServer