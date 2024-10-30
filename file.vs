declare fn.C realloc(prevItems *mut Int, bytesize Int) [*]Int
declare fn.C malloc(bytesize Int) [*]Int
declare fn.C socket(domain Int, type Int, protocol Int) Int
declare fn.C exit(status Int)
declare fn.C printf(fmt Str, rest ...) Int
declare fn.C asprintf(buffer *Str, fmt Str, rest ...) Int

typedef Data (Int, Int, (Bool, Bool, Int))

struct.C Vec {
    len Int,
    cap Int,
    items [*]Int,
}

impl Vec {
    fn.C new() Self {
        ret Self {
            len: 0,
            cap: 0,
            items: null
        }
    }

    fn.C push(*mut self, item Int) {
        if self.len == self.cap {
            self.cap = if self.cap == 0 { 2 } else { self.cap * 2 }
            size := self.cap * 4
            self.items = if self.len == 0 { malloc(size) } else { realloc(self.items, size) }
        }
        self.items[self.len] = item
        self.len = self.len + 1
    }

    fn.C last(*self) *Int {
        ret self.items[self.len - 1]
    }

    fn.C lastMut(*mut self) *mut Int {
        ret self.items[self.len - 1]
    }

    fn.C pop(*mut self) Int {
        self.len = self.len - 1
        ret self.items[self.len]
    }

    fn.C debug(*self) {
        printf("Len = %d\n", self.len)
        printf("Cap = %d\n", self.cap)

        mut i := 0
        loop {
            if i == self.len { break } else { 
                printf("[%d] = %d\n", i, self.items[i])
                i = i + 1
            }
        }
        printf("\n")
    }
}

fn.C newVec() Vec {
    ret Vec {
        len: 0,
        cap: 0,
        items: null
    }
}

fn main() {
    runTests()
    mut vec := Vec.new()
    vec.push(0)
    vec.push(1)
    vec.push(2)
    vec.debug()
    vec.pop()
    vec.debug()
    vec.push(10)
    vec.push(20)
    vec.push(30)
    vec.debug()
    vec.lastMut() = 2
    vec.debug()
}

struct TestUtils
impl TestUtils {
    fn new() Self {
        ret Self {}
    }

    fn printReset(self) {
        printf("\1B[0m")
    }

    fn assertInt(self, x Int, y Int, err Str) {
        if x != y {
            printf("\1B[31mAssert: %d != %d, Err: '%s'\n", x, y, err)
            self.printReset()
            exit(1)
        }
    }

    fn printTestSucces(self, num Int) {
        printf("\1B[32mTest %d passed\n", num)
        self.printReset()
    }
}

struct VecTester {
    vec Vec,
    testUtils TestUtils,
    testCount Int
}

impl VecTester {
    fn new() Self {
        ret Self {
            vec: Vec.new(),
            testUtils: TestUtils.new(),
            testCount: 1
        }
    }

    fn runTests(mut self) {
        self.testPush()
    }

    fn testPush(mut self) {
        self.vec.push(2)
        self.testUtils.assertInt(self.vec.cap, 2, "")
        self.testUtils.printTestSucces(self.getTestCount())
    }

    fn getTestCount(mut self) Int {
        self.testCount = self.testCount + 1
        ret self.testCount - 1
    }
}

fn runTests() {
    mut vecTester := VecTester.new()
    vecTester.runTests()

    tester := TestUtils {}

    vecTester := VecTester.new()

    mut vec := newVec()
    tester.assertInt(vec.len, vec.cap, "vec.len != vec.cap")
    tester.printTestSucces(1)

    vec.push(5)
    tester.assertInt(vec.len, 1, "vec.len != 1")
    tester.assertInt(vec.cap, 2, "vec.cap != 2")
    tester.assertInt(vec.items[0], 5, "vec.items[0] != 2")
    tester.printTestSucces(2)

}