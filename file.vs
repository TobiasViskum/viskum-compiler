declare fn.C realloc(prevItems [*]int, bytesize int) [*]int
declare fn.C malloc(bytesize int) [*]int
declare fn.C socket(domain int, type int, protocol int) int
declare fn.C exit(status int)
declare fn.C printf(fmt str, rest ...) int
declare fn.C asprintf(buffer *str, fmt str, rest ...) int
declare fn.C time(time *int64) int
declare fn.C sleep(time int) int
declare fn.C clock_gettime(realtime int, timespec *mut TimeSpec) int

struct TimeSpec {
    tv_sec int64,
    tv_nsec int64
}

impl TimeSpec {
    fn new() Self {
        ret Self {
            tv_sec: 0,
            tv_nsec: 0
        }
    }

    fn getSec(self) int64 {
        ret self.tv_sec
    }

    fn getNsec(self) int64 {
        ret self.tv_nsec
    }

    fn print(*self) {
        printf("TimeSpec { tv_sec: %d, tv_nsec: %d }\n", self.tv_sec, self.tv_nsec)
    }
}

typedef Data (int, int, (bool, bool, int))

struct.C Vec {
    len uint,
    cap uint,
    items [*]int,
}

impl Vec {
    fn.C new() Self {
        ret Self {
            len: 0,
            cap: 0,
            items: null
        }
    }

    fn.C push(*mut self, item int) {
        if self.len == self.cap {
            self.cap = if self.cap == 0 { 2 } else { self.cap * 2 }
            size := self.cap * 4
            self.items = if self.len == 0 { malloc(size) } else { realloc(self.items, size) }
        }
        self.items[self.len] = item
        self.len = self.len + 1
    }

    fn.C last(*self) *int {
        ret self.items[self.len - 1]
    }

    fn.C lastMut(*mut self) *mut int {
        ret self.items[self.len - 1]
    }

    fn.C pop(*mut self) int {
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

enum Option {
    Some(int),
    None
}

struct Instant {
    start TimeSpec
}

impl Instant {
    fn new() Self {
        mut start := TimeSpec.new()
        clock_gettime(0, start)

        ret Self {
            start: start
        }
    }

    fn elapsed(self) int64 {
        mut end := TimeSpec.new()
        clock_gettime(0, end)

        mut elapsedNs := (end.getSec() - self.start.getSec()) * 1000000000 + (end.getNsec() - self.start.getNsec())

        suffix := if elapsedNs < 1000 {
            elapsedNs = elapsedNs
            "ns"
        } elif elapsedNs < 10000000 {
            elapsedNs = elapsedNs / 1000
            "µs"
        } elif elapsedNs < 10000000000 {
            elapsedNs = elapsedNs / 1000000
            "ms"
        } else {
            elapsedNs = elapsedNs / 1000000000
            "s"
        }

        printf("Elapsed: %ld %s\n", elapsedNs, suffix)
        ret 0
    }
}

struct.C StrVec {
    len uint,
    cap uint,

}

fn.C main(argc int, args [*]str) {

    
    maybe := Option.Some(2)

    if Option.Some(x) := maybe {
        printf("Hello, value is: %d\n", x)
    }

    runTests()

    mut now := Instant.new()
    sleep(1)
    now.elapsed()
}

struct TestUtils
impl TestUtils {
    fn new() Self {
        ret Self {}
    }

    fn printReset(self) {
        printf("\1B[0m")
    }

    fn assertInt(self, x int, y int, err str) {
        if x != y {
            printf("\1B[31mAssert: %d != %d, Err: '%s'\n", x, y, err)
            self.printReset()
            exit(1)
        }
    }

    fn printTestSucces(self, num int) {
        printf("\1B[32mTest %d passed\n", num)
        self.printReset()
    }
}

struct VecTester {
    vec Vec,
    testUtils TestUtils,
    testCount int
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
        self.testPop()
    }

    fn testPush(mut self) {
        self.vec.push(2)
        self.testUtils.assertInt(self.vec.last(), 2, "")
        self.testUtils.printTestSucces(self.getTestCount())
    }

    fn testPop(mut self) {
        self.vec.push(94)
        self.testUtils.assertInt(self.vec.pop(), 94, "")
        self.testUtils.printTestSucces(self.getTestCount())
    }

    fn getTestCount(mut self) int {
        self.testCount = self.testCount + 1
        ret self.testCount - 1
    }
}

fn runTests() {
    mut vecTester := VecTester.new()
    vecTester.runTests()
}