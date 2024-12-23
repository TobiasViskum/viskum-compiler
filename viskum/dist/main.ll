declare ptr @realloc(ptr noundef, i32 noundef)
declare ptr @malloc(i32 noundef)
declare i32 @socket(i32 noundef, i32 noundef, i32 noundef)
declare void @exit(i32 noundef)
declare i32 @printf(ptr noundef, ...)
declare i32 @asprintf(ptr noundef, ptr noundef, ...)
declare i32 @time(ptr noundef)
declare i32 @sleep(i32 noundef)
declare i32 @clock_gettime(i32 noundef, ptr noundef)

@.str.0.454 = private unnamed_addr constant [30 x i8] c"Hello, value is: %d + 1 = %d\0A\00"
@.str.0.421 = private unnamed_addr constant [17 x i8] c"Elapsed: %ld %s\0A\00"
@.str.0.399 = private unnamed_addr constant [4 x i8] c"µs\00"
@.str.0.298 = private unnamed_addr constant [2 x i8] c"\0A\00"
@.str.0.265 = private unnamed_addr constant [10 x i8] c"Cap = %d\0A\00"
@.str.0.504 = private unnamed_addr constant [5 x i8] c"\1B[0m\00"
@.str.0.450 = private unnamed_addr constant [21 x i8] c"Hello, value is: %d\0A\00"
@.str.0.414 = private unnamed_addr constant [2 x i8] c"s\00"
@.str.0.390 = private unnamed_addr constant [3 x i8] c"ns\00"
@.str.0.91 = private unnamed_addr constant [38 x i8] c"TimeSpec { tv_sec: %d, tv_nsec: %d }\0A\00"
@.str.0.280 = private unnamed_addr constant [11 x i8] c"[%d] = %d\0A\00"
@.str.0.259 = private unnamed_addr constant [10 x i8] c"Len = %d\0A\00"
@.str.0.408 = private unnamed_addr constant [3 x i8] c"ms\00"
@.str.0.605 = private unnamed_addr constant [1 x i8] c"\00"
@.str.0.538 = private unnamed_addr constant [21 x i8] c"\1B[32mTest %d passed\0A\00"
@.str.0.519 = private unnamed_addr constant [34 x i8] c"\1B[31mAssert: %d != %d, Err: '%s'\0A\00"

define i32 @main() {
    %1 = alloca [16 x i8], align 8
    br label %2
2:
    call void () @runTests0_435()
    %3 = call [16 x i8] () @new0_324()
    store [16 x i8] %3, ptr %1
    %4 = sext i8 1 to i32
    %5 = call i32 (i32) @sleep(i32 noundef %4)
    %6 = call i64 (ptr) @elapsed0_341(ptr noundef %1)
    ret i32 0
}

define [16 x i8] @new0_63() {
    %1 = alloca [16 x i8], align 8
    br label %2
2:
    %3 = sext i8 0 to i64
    store i64 %3, ptr %1
    %4 = sext i8 0 to i64
    %5 = getelementptr inbounds i8, ptr %1, i64 8
    store i64 %4, ptr %5
    %6 = load [16 x i8], ptr %1
    ret [16 x i8] %6
    unreachable
}

define i64 @getSec0_72(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = load ptr, ptr %2
    %5 = load i64, ptr %4
    ret i64 %5
    unreachable
}

define i64 @getNsec0_80(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = load ptr, ptr %2
    %5 = getelementptr inbounds i8, ptr %4, i64 8
    %6 = load i64, ptr %5
    ret i64 %6
    unreachable
}

define void @print0_88(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = load ptr, ptr %2
    %5 = load i64, ptr %4
    %6 = load ptr, ptr %2
    %7 = getelementptr inbounds i8, ptr %6, i64 8
    %8 = load i64, ptr %7
    %9 = call i32 (ptr, ...) @printf(ptr noundef @.str.0.91, i64 noundef %5, i64 noundef %8)
    ret void
}

define [16 x i8] @new0_121() {
    %1 = alloca [16 x i8], align 4
    br label %2
2:
    %3 = sext i8 0 to i32
    store i32 %3, ptr %1
    %4 = sext i8 0 to i32
    %5 = getelementptr inbounds i8, ptr %1, i64 4
    store i32 %4, ptr %5
    %6 = getelementptr inbounds i8, ptr %1, i64 8
    store ptr null, ptr %6
    %7 = load [16 x i8], ptr %1
    ret [16 x i8] %7
    unreachable
}

define void @push0_132(ptr noundef %0, i32 noundef %1) {
    %3 = alloca [8 x i8], align 8
    %4 = alloca [4 x i8], align 4
    %5 = alloca [4 x i8], align 4
    %6 = alloca [4 x i8], align 4
    %7 = alloca [8 x i8], align 8
    br label %8
8:
    store ptr %0, ptr %3
    store i32 %1, ptr %4
    %9 = load ptr, ptr %3
    %10 = load i32, ptr %9
    %11 = load ptr, ptr %3
    %12 = getelementptr inbounds i8, ptr %11, i64 4
    %13 = load i32, ptr %12
    %14 = icmp eq i32 %10, %13
    br i1 %14, label %15, label %55
15:
    %16 = load ptr, ptr %3
    %17 = getelementptr inbounds i8, ptr %16, i64 4
    %18 = load ptr, ptr %3
    %19 = getelementptr inbounds i8, ptr %18, i64 4
    %20 = load i32, ptr %19
    %21 = sext i8 0 to i32
    %22 = icmp eq i32 %20, %21
    br i1 %22, label %23, label %25
23:
    %24 = sext i8 2 to i32
    store i32 %24, ptr %6
    br label %31
25:
    %26 = load ptr, ptr %3
    %27 = getelementptr inbounds i8, ptr %26, i64 4
    %28 = load i32, ptr %27
    %29 = sext i8 2 to i32
    %30 = mul nsw i32 %28, %29
    store i32 %30, ptr %6
    br label %31
31:
    %32 = load i32, ptr %6
    store i32 %32, ptr %17
    %33 = load ptr, ptr %3
    %34 = getelementptr inbounds i8, ptr %33, i64 4
    %35 = load i32, ptr %34
    %36 = sext i8 4 to i32
    %37 = mul nsw i32 %35, %36
    store i32 %37, ptr %5
    %38 = load ptr, ptr %3
    %39 = getelementptr inbounds i8, ptr %38, i64 8
    %40 = load ptr, ptr %3
    %41 = load i32, ptr %40
    %42 = sext i8 0 to i32
    %43 = icmp eq i32 %41, %42
    br i1 %43, label %44, label %47
44:
    %45 = load i32, ptr %5
    %46 = call ptr (i32) @malloc(i32 noundef %45)
    store ptr %46, ptr %7
    br label %53
47:
    %48 = load ptr, ptr %3
    %49 = getelementptr inbounds i8, ptr %48, i64 8
    %50 = load ptr, ptr %49
    %51 = load i32, ptr %5
    %52 = call ptr (ptr, i32) @realloc(ptr noundef %50, i32 noundef %51)
    store ptr %52, ptr %7
    br label %53
53:
    %54 = load ptr, ptr %7
    store ptr %54, ptr %39
    br label %55
55:
    %56 = load ptr, ptr %3
    %57 = getelementptr inbounds i8, ptr %56, i64 8
    %58 = load ptr, ptr %57
    %59 = load ptr, ptr %3
    %60 = load i32, ptr %59
    %61 = sext i32 %60 to i64
    %62 = getelementptr inbounds i32, ptr %58, i64 %61
    %63 = load i32, ptr %4
    store i32 %63, ptr %62
    %64 = load ptr, ptr %3
    %65 = load ptr, ptr %3
    %66 = load i32, ptr %65
    %67 = sext i8 1 to i32
    %68 = add nsw i32 %66, %67
    store i32 %68, ptr %64
    ret void
}

define ptr @last0_207(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = load ptr, ptr %2
    %5 = getelementptr inbounds i8, ptr %4, i64 8
    %6 = load ptr, ptr %5
    %7 = load ptr, ptr %2
    %8 = load i32, ptr %7
    %9 = sext i8 1 to i32
    %10 = sub nsw i32 %8, %9
    %11 = sext i32 %10 to i64
    %12 = getelementptr inbounds i32, ptr %6, i64 %11
    ret ptr %12
    unreachable
}

define ptr @lastMut0_221(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = load ptr, ptr %2
    %5 = getelementptr inbounds i8, ptr %4, i64 8
    %6 = load ptr, ptr %5
    %7 = load ptr, ptr %2
    %8 = load i32, ptr %7
    %9 = sext i8 1 to i32
    %10 = sub nsw i32 %8, %9
    %11 = sext i32 %10 to i64
    %12 = getelementptr inbounds i32, ptr %6, i64 %11
    ret ptr %12
    unreachable
}

define i32 @pop0_235(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = load ptr, ptr %2
    %5 = load ptr, ptr %2
    %6 = load i32, ptr %5
    %7 = sext i8 1 to i32
    %8 = sub nsw i32 %6, %7
    store i32 %8, ptr %4
    %9 = load ptr, ptr %2
    %10 = getelementptr inbounds i8, ptr %9, i64 8
    %11 = load ptr, ptr %10
    %12 = load ptr, ptr %2
    %13 = load i32, ptr %12
    %14 = sext i32 %13 to i64
    %15 = getelementptr inbounds i32, ptr %11, i64 %14
    %16 = load i32, ptr %15
    ret i32 %16
    unreachable
}

define void @debug0_256(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    %3 = alloca [1 x i8], align 1
    br label %4
4:
    store ptr %0, ptr %2
    %5 = load ptr, ptr %2
    %6 = load i32, ptr %5
    %7 = call i32 (ptr, ...) @printf(ptr noundef @.str.0.259, i32 noundef %6)
    %8 = load ptr, ptr %2
    %9 = getelementptr inbounds i8, ptr %8, i64 4
    %10 = load i32, ptr %9
    %11 = call i32 (ptr, ...) @printf(ptr noundef @.str.0.265, i32 noundef %10)
    store i8 0, ptr %3
    br label %12
12:
    %13 = load i8, ptr %3
    %14 = sext i8 %13 to i32
    %15 = load ptr, ptr %2
    %16 = load i32, ptr %15
    %17 = icmp eq i32 %14, %16
    br i1 %17, label %18, label %20
18:
    br label %33
19:
    br label %32
20:
    %21 = load i8, ptr %3
    %22 = load ptr, ptr %2
    %23 = getelementptr inbounds i8, ptr %22, i64 8
    %24 = load ptr, ptr %23
    %25 = load i8, ptr %3
    %26 = sext i8 %25 to i64
    %27 = getelementptr inbounds i32, ptr %24, i64 %26
    %28 = load i32, ptr %27
    %29 = call i32 (ptr, ...) @printf(ptr noundef @.str.0.280, i8 noundef %21, i32 noundef %28)
    %30 = load i8, ptr %3
    %31 = add nsw i8 %30, 1
    store i8 %31, ptr %3
    br label %32
32:
    br label %12
33:
    %34 = call i32 (ptr) @printf(ptr noundef @.str.0.298)
    ret void
}

define [16 x i8] @newVec0_302() {
    %1 = alloca [16 x i8], align 4
    br label %2
2:
    %3 = sext i8 0 to i32
    store i32 %3, ptr %1
    %4 = sext i8 0 to i32
    %5 = getelementptr inbounds i8, ptr %1, i64 4
    store i32 %4, ptr %5
    %6 = getelementptr inbounds i8, ptr %1, i64 8
    store ptr null, ptr %6
    %7 = load [16 x i8], ptr %1
    ret [16 x i8] %7
    unreachable
}

define [16 x i8] @new0_324() {
    %1 = alloca [16 x i8], align 8
    %2 = alloca [16 x i8], align 8
    br label %3
3:
    %4 = call [16 x i8] () @new0_63()
    store [16 x i8] %4, ptr %1
    %5 = sext i8 0 to i32
    %6 = call i32 (i32, ptr) @clock_gettime(i32 noundef %5, ptr noundef %1)
    %7 = load [16 x i8], ptr %1
    store [16 x i8] %7, ptr %2
    %8 = load [16 x i8], ptr %2
    ret [16 x i8] %8
    unreachable
}

define i64 @elapsed0_341(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    %3 = alloca [16 x i8], align 8
    %4 = alloca [8 x i8], align 8
    %5 = alloca [8 x i8], align 8
    %6 = alloca [8 x i8], align 8
    br label %7
7:
    store ptr %0, ptr %2
    %8 = call [16 x i8] () @new0_63()
    store [16 x i8] %8, ptr %3
    %9 = sext i8 0 to i32
    %10 = call i32 (i32, ptr) @clock_gettime(i32 noundef %9, ptr noundef %3)
    %11 = call i64 (ptr) @getSec0_72(ptr noundef %3)
    %12 = load ptr, ptr %2
    %13 = call i64 (ptr) @getSec0_72(ptr noundef %12)
    %14 = sub nsw i64 %11, %13
    %15 = sext i32 1000000000 to i64
    %16 = mul nsw i64 %14, %15
    %17 = call i64 (ptr) @getNsec0_80(ptr noundef %3)
    %18 = load ptr, ptr %2
    %19 = call i64 (ptr) @getNsec0_80(ptr noundef %18)
    %20 = sub nsw i64 %17, %19
    %21 = add nsw i64 %16, %20
    store i64 %21, ptr %4
    %22 = load i64, ptr %4
    %23 = sext i16 1000 to i64
    %24 = icmp slt i64 %22, %23
    br i1 %24, label %25, label %27
25:
    %26 = load i64, ptr %4
    store i64 %26, ptr %4
    store ptr @.str.0.390, ptr %6
    br label %31
27:
    %28 = load i64, ptr %4
    %29 = sext i32 10000000 to i64
    %30 = icmp slt i64 %28, %29
    br i1 %30, label %31, label %35
31:
    %32 = load i64, ptr %4
    %33 = sext i16 1000 to i64
    %34 = sdiv i64 %32, %33
    store i64 %34, ptr %4
    store ptr @.str.0.399, ptr %6
    br label %38
35:
    %36 = load i64, ptr %4
    %37 = icmp slt i64 %36, 10000000000
    br i1 %37, label %38, label %42
38:
    %39 = load i64, ptr %4
    %40 = sext i32 1000000 to i64
    %41 = sdiv i64 %39, %40
    store i64 %41, ptr %4
    store ptr @.str.0.408, ptr %6
    br label %46
42:
    %43 = load i64, ptr %4
    %44 = sext i32 1000000000 to i64
    %45 = sdiv i64 %43, %44
    store i64 %45, ptr %4
    store ptr @.str.0.414, ptr %6
    br label %46
46:
    %47 = load ptr, ptr %6
    store ptr %47, ptr %5
    %48 = load i64, ptr %4
    %49 = load ptr, ptr %5
    %50 = call i32 (ptr, ...) @printf(ptr noundef @.str.0.421, i64 noundef %48, ptr noundef %49)
    %51 = sext i8 0 to i64
    ret i64 %51
    unreachable
}

define void @runTests0_435() {
    %1 = alloca [12 x i8], align 4
    %2 = alloca [4 x i8], align 4
    %3 = alloca [20 x i8], align 1
    %4 = alloca [12 x i8], align 4
    %5 = alloca [4 x i8], align 4
    br label %6
6:
    store i64 0, ptr %4
    %7 = sext i8 2 to i32
    %8 = getelementptr inbounds i8, ptr %4, i64 8
    store i32 %7, ptr %8
    %9 = load [12 x i8], ptr %4
    store [12 x i8] %9, ptr %1
    %10 = load i64, ptr %1
    %11 = icmp eq i64 %10, 0
    br i1 %11, label %12, label %22
12:
    %13 = getelementptr inbounds i8, ptr %1, i64 8
    %14 = load i32, ptr %13
    store i32 %14, ptr %2
    %15 = load i32, ptr %2
    %16 = call i32 (ptr, ...) @printf(ptr noundef @.str.0.450, i32 noundef %15)
    %17 = load i32, ptr %2
    %18 = load i32, ptr %2
    %19 = sext i8 1 to i32
    %20 = add nsw i32 %18, %19
    %21 = call i32 (ptr, ...) @printf(ptr noundef @.str.0.454, i32 noundef %17, i32 noundef %20)
    store i32 %21, ptr %5
    br label %22
22:
    %23 = call [20 x i8] () @new0_556()
    store [20 x i8] %23, ptr %3
    call void (ptr) @runTests0_573(ptr noundef %3)
    ret void
}

define [0 x i8] @new0_496() {
    %1 = alloca [0 x i8], align 1
    br label %2
2:
    %3 = load [0 x i8], ptr %1
    ret [0 x i8] %3
    unreachable
}

define void @printReset0_501(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = call i32 (ptr) @printf(ptr noundef @.str.0.504)
    ret void
}

define void @assertInt0_507(ptr noundef %0, i32 noundef %1, i32 noundef %2, ptr noundef %3) {
    %5 = alloca [8 x i8], align 8
    %6 = alloca [4 x i8], align 4
    %7 = alloca [4 x i8], align 4
    %8 = alloca [8 x i8], align 8
    br label %9
9:
    store ptr %0, ptr %5
    store i32 %1, ptr %6
    store i32 %2, ptr %7
    store ptr %3, ptr %8
    %10 = load i32, ptr %6
    %11 = load i32, ptr %7
    %12 = icmp ne i32 %10, %11
    br i1 %12, label %13, label %20
13:
    %14 = load i32, ptr %6
    %15 = load i32, ptr %7
    %16 = load ptr, ptr %8
    %17 = call i32 (ptr, ...) @printf(ptr noundef @.str.0.519, i32 noundef %14, i32 noundef %15, ptr noundef %16)
    %18 = load ptr, ptr %5
    call void (ptr) @printReset0_501(ptr noundef %18)
    %19 = sext i8 1 to i32
    call void (i32) @exit(i32 noundef %19)
    br label %20
20:
    ret void
}

define void @printTestSucces0_533(ptr noundef %0, i32 noundef %1) {
    %3 = alloca [8 x i8], align 8
    %4 = alloca [4 x i8], align 4
    br label %5
5:
    store ptr %0, ptr %3
    store i32 %1, ptr %4
    %6 = load i32, ptr %4
    %7 = call i32 (ptr, ...) @printf(ptr noundef @.str.0.538, i32 noundef %6)
    %8 = load ptr, ptr %3
    call void (ptr) @printReset0_501(ptr noundef %8)
    ret void
}

define [20 x i8] @new0_556() {
    %1 = alloca [20 x i8], align 1
    br label %2
2:
    %3 = call [16 x i8] () @new0_121()
    store [16 x i8] %3, ptr %1
    %4 = call [0 x i8] () @new0_496()
    %5 = getelementptr inbounds i8, ptr %1, i64 16
    store [0 x i8] %4, ptr %5
    %6 = sext i8 1 to i32
    %7 = getelementptr inbounds i8, ptr %1, i64 16
    store i32 %6, ptr %7
    %8 = load [20 x i8], ptr %1
    ret [20 x i8] %8
    unreachable
}

define void @runTests0_573(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = load ptr, ptr %2
    call void (ptr) @testPush0_584(ptr noundef %4)
    %5 = load ptr, ptr %2
    call void (ptr) @testPop0_618(ptr noundef %5)
    ret void
}

define void @testPush0_584(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = load ptr, ptr %2
    %5 = sext i8 2 to i32
    call void (ptr, i32) @push0_132(ptr noundef %4, i32 noundef %5)
    %6 = load ptr, ptr %2
    %7 = getelementptr inbounds i8, ptr %6, i64 16
    %8 = load ptr, ptr %2
    %9 = call ptr (ptr) @last0_207(ptr noundef %8)
    %10 = load i32, ptr %9
    %11 = sext i8 2 to i32
    call void (ptr, i32, i32, ptr) @assertInt0_507(ptr noundef %7, i32 noundef %10, i32 noundef %11, ptr noundef @.str.0.605)
    %12 = load ptr, ptr %2
    %13 = getelementptr inbounds i8, ptr %12, i64 16
    %14 = load ptr, ptr %2
    %15 = call i32 (ptr) @getTestCount0_652(ptr noundef %14)
    call void (ptr, i32) @printTestSucces0_533(ptr noundef %13, i32 noundef %15)
    ret void
}

define void @testPop0_618(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = load ptr, ptr %2
    %5 = sext i8 94 to i32
    call void (ptr, i32) @push0_132(ptr noundef %4, i32 noundef %5)
    %6 = load ptr, ptr %2
    %7 = getelementptr inbounds i8, ptr %6, i64 16
    %8 = load ptr, ptr %2
    %9 = call i32 (ptr) @pop0_235(ptr noundef %8)
    %10 = sext i8 94 to i32
    call void (ptr, i32, i32, ptr) @assertInt0_507(ptr noundef %7, i32 noundef %9, i32 noundef %10, ptr noundef @.str.0.605)
    %11 = load ptr, ptr %2
    %12 = getelementptr inbounds i8, ptr %11, i64 16
    %13 = load ptr, ptr %2
    %14 = call i32 (ptr) @getTestCount0_652(ptr noundef %13)
    call void (ptr, i32) @printTestSucces0_533(ptr noundef %12, i32 noundef %14)
    ret void
}

define i32 @getTestCount0_652(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = load ptr, ptr %2
    %5 = getelementptr inbounds i8, ptr %4, i64 16
    %6 = load ptr, ptr %2
    %7 = getelementptr inbounds i8, ptr %6, i64 16
    %8 = load i32, ptr %7
    %9 = sext i8 1 to i32
    %10 = add nsw i32 %8, %9
    store i32 %10, ptr %5
    %11 = load ptr, ptr %2
    %12 = getelementptr inbounds i8, ptr %11, i64 16
    %13 = load i32, ptr %12
    %14 = sext i8 1 to i32
    %15 = sub nsw i32 %13, %14
    ret i32 %15
    unreachable
}

