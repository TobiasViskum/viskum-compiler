declare ptr @realloc(ptr noundef, i32 noundef)
declare ptr @malloc(i32 noundef)
declare i32 @socket(i32 noundef, i32 noundef, i32 noundef)
declare void @exit(i32 noundef)
declare i32 @printf(ptr noundef, ...)
declare i32 @asprintf(ptr noundef, ptr noundef, ...)
declare i32 @time(ptr noundef)
declare i32 @sleep(i32 noundef)
declare i32 @clock_gettime(i32 noundef, ptr noundef)

@.str.1.423 = private unnamed_addr constant [2 x i8] c"s\00"
@.str.1.91 = private unnamed_addr constant [38 x i8] c"TimeSpec { tv_sec: %d, tv_nsec: %d }\0A\00"
@.str.1.399 = private unnamed_addr constant [3 x i8] c"ns\00"
@.str.1.289 = private unnamed_addr constant [11 x i8] c"[%d] = %d\0A\00"
@.str.1.268 = private unnamed_addr constant [10 x i8] c"Len = %d\0A\00"
@.str.1.137 = private unnamed_addr constant [18 x i8] c"Len: %d, Cap: %d\0A\00"
@.str.1.417 = private unnamed_addr constant [3 x i8] c"ms\00"
@.str.1.463 = private unnamed_addr constant [30 x i8] c"Hello, value is: %d + 1 = %d\0A\00"
@.str.1.430 = private unnamed_addr constant [17 x i8] c"Elapsed: %ld %s\0A\00"
@.str.1.408 = private unnamed_addr constant [4 x i8] c"µs\00"
@.str.1.614 = private unnamed_addr constant [1 x i8] c"\00"
@.str.1.547 = private unnamed_addr constant [21 x i8] c"\1B[32mTest %d passed\0A\00"
@.str.1.307 = private unnamed_addr constant [2 x i8] c"\0A\00"
@.str.1.274 = private unnamed_addr constant [10 x i8] c"Cap = %d\0A\00"
@.str.1.513 = private unnamed_addr constant [5 x i8] c"\1B[0m\00"
@.str.1.528 = private unnamed_addr constant [34 x i8] c"\1B[31mAssert: %d != %d, Err: '%s'\0A\00"
@.str.1.459 = private unnamed_addr constant [21 x i8] c"Hello, value is: %d\0A\00"

define i32 @main() {
    %1 = alloca [16 x i8], align 8
    br label %2
2:
    call void () @runTests1_444()
    %3 = call [16 x i8] () @new1_333()
    store [16 x i8] %3, ptr %1
    %4 = sext i8 1 to i32
    %5 = call i32 (i32) @sleep(i32 noundef %4)
    %6 = call i64 (ptr) @elapsed1_350(ptr noundef %1)
    ret i32 0
}

define [12 x i8] @new0_9(i32 noundef %0, i32 noundef %1, i32 noundef %2) {
    %4 = alloca [4 x i8], align 4
    %5 = alloca [4 x i8], align 4
    %6 = alloca [4 x i8], align 4
    %7 = alloca [12 x i8], align 4
    br label %8
8:
    store i32 %0, ptr %4
    store i32 %1, ptr %5
    store i32 %2, ptr %6
    %9 = load i32, ptr %4
    store i32 %9, ptr %7
    %10 = load i32, ptr %5
    %11 = getelementptr inbounds i8, ptr %7, i64 4
    store i32 %10, ptr %11
    %12 = load i32, ptr %6
    %13 = getelementptr inbounds i8, ptr %7, i64 8
    store i32 %12, ptr %13
    unreachable
}

define i32 @sum0_25(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = load ptr, ptr %2
    %5 = load i32, ptr %4
    %6 = load ptr, ptr %2
    %7 = getelementptr inbounds i8, ptr %6, i64 4
    %8 = load i32, ptr %7
    %9 = add nsw i32 %5, %8
    %10 = load ptr, ptr %2
    %11 = load i32, ptr %10
    %12 = add nsw i32 %9, %11
    ret i32 %12
    unreachable
}

define [16 x i8] @new1_63() {
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

define i64 @getSec1_72(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = load ptr, ptr %2
    %5 = load i64, ptr %4
    ret i64 %5
    unreachable
}

define i64 @getNsec1_80(ptr noundef %0) {
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

define void @print1_88(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = load ptr, ptr %2
    %5 = load i64, ptr %4
    %6 = load ptr, ptr %2
    %7 = getelementptr inbounds i8, ptr %6, i64 8
    %8 = load i64, ptr %7
    %9 = call i32 (ptr, ...) @printf(ptr noundef @.str.1.91, i64 noundef %5, i64 noundef %8)
    ret void
}

define [16 x i8] @new1_121() {
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

define void @push1_132(ptr noundef %0, i32 noundef %1) {
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
    %14 = call i32 (ptr, ...) @printf(ptr noundef @.str.1.137, i32 noundef %10, i32 noundef %13)
    %15 = load ptr, ptr %3
    %16 = load i32, ptr %15
    %17 = load ptr, ptr %3
    %18 = getelementptr inbounds i8, ptr %17, i64 4
    %19 = load i32, ptr %18
    %20 = icmp eq i32 %16, %19
    br i1 %20, label %21, label %61
21:
    %22 = load ptr, ptr %3
    %23 = getelementptr inbounds i8, ptr %22, i64 4
    %24 = load ptr, ptr %3
    %25 = getelementptr inbounds i8, ptr %24, i64 4
    %26 = load i32, ptr %25
    %27 = sext i8 0 to i32
    %28 = icmp eq i32 %26, %27
    br i1 %28, label %29, label %31
29:
    %30 = sext i8 2 to i32
    store i32 %30, ptr %6
    br label %37
31:
    %32 = load ptr, ptr %3
    %33 = getelementptr inbounds i8, ptr %32, i64 4
    %34 = load i32, ptr %33
    %35 = sext i8 2 to i32
    %36 = mul nsw i32 %34, %35
    store i32 %36, ptr %6
    br label %37
37:
    %38 = load i32, ptr %6
    store i32 %38, ptr %23
    %39 = load ptr, ptr %3
    %40 = getelementptr inbounds i8, ptr %39, i64 4
    %41 = load i32, ptr %40
    %42 = sext i8 4 to i32
    %43 = mul nsw i32 %41, %42
    store i32 %43, ptr %5
    %44 = load ptr, ptr %3
    %45 = getelementptr inbounds i8, ptr %44, i64 8
    %46 = load ptr, ptr %3
    %47 = load i32, ptr %46
    %48 = sext i8 0 to i32
    %49 = icmp eq i32 %47, %48
    br i1 %49, label %50, label %53
50:
    %51 = load i32, ptr %5
    %52 = call ptr (i32) @malloc(i32 noundef %51)
    store ptr %52, ptr %7
    br label %59
53:
    %54 = load ptr, ptr %3
    %55 = getelementptr inbounds i8, ptr %54, i64 8
    %56 = load ptr, ptr %55
    %57 = load i32, ptr %5
    %58 = call ptr (ptr, i32) @realloc(ptr noundef %56, i32 noundef %57)
    store ptr %58, ptr %7
    br label %59
59:
    %60 = load ptr, ptr %7
    store ptr %60, ptr %45
    br label %61
61:
    %62 = load ptr, ptr %3
    %63 = getelementptr inbounds i8, ptr %62, i64 8
    %64 = load ptr, ptr %63
    %65 = load ptr, ptr %3
    %66 = load i32, ptr %65
    %67 = sext i32 %66 to i64
    %68 = getelementptr inbounds i32, ptr %64, i64 %67
    %69 = load i32, ptr %4
    store i32 %69, ptr %68
    %70 = load ptr, ptr %3
    %71 = load ptr, ptr %3
    %72 = load i32, ptr %71
    %73 = sext i8 1 to i32
    %74 = add nsw i32 %72, %73
    store i32 %74, ptr %70
    ret void
}

define ptr @last1_216(ptr noundef %0) {
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

define ptr @lastMut1_230(ptr noundef %0) {
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

define i32 @pop1_244(ptr noundef %0) {
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

define void @debug1_265(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    %3 = alloca [1 x i8], align 1
    br label %4
4:
    store ptr %0, ptr %2
    %5 = load ptr, ptr %2
    %6 = load i32, ptr %5
    %7 = call i32 (ptr, ...) @printf(ptr noundef @.str.1.268, i32 noundef %6)
    %8 = load ptr, ptr %2
    %9 = getelementptr inbounds i8, ptr %8, i64 4
    %10 = load i32, ptr %9
    %11 = call i32 (ptr, ...) @printf(ptr noundef @.str.1.274, i32 noundef %10)
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
    %29 = call i32 (ptr, ...) @printf(ptr noundef @.str.1.289, i8 noundef %21, i32 noundef %28)
    %30 = load i8, ptr %3
    %31 = add nsw i8 %30, 1
    store i8 %31, ptr %3
    br label %32
32:
    br label %12
33:
    %34 = call i32 (ptr) @printf(ptr noundef @.str.1.307)
    ret void
}

define [16 x i8] @newVec1_311() {
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

define [16 x i8] @new1_333() {
    %1 = alloca [16 x i8], align 8
    %2 = alloca [16 x i8], align 8
    br label %3
3:
    %4 = call [16 x i8] () @new1_63()
    store [16 x i8] %4, ptr %1
    %5 = sext i8 0 to i32
    %6 = call i32 (i32, ptr) @clock_gettime(i32 noundef %5, ptr noundef %1)
    %7 = load [16 x i8], ptr %1
    store [16 x i8] %7, ptr %2
    %8 = load [16 x i8], ptr %2
    ret [16 x i8] %8
    unreachable
}

define i64 @elapsed1_350(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    %3 = alloca [16 x i8], align 8
    %4 = alloca [8 x i8], align 8
    %5 = alloca [8 x i8], align 8
    %6 = alloca [8 x i8], align 8
    br label %7
7:
    store ptr %0, ptr %2
    %8 = call [16 x i8] () @new1_63()
    store [16 x i8] %8, ptr %3
    %9 = sext i8 0 to i32
    %10 = call i32 (i32, ptr) @clock_gettime(i32 noundef %9, ptr noundef %3)
    %11 = call i64 (ptr) @getSec1_72(ptr noundef %3)
    %12 = load ptr, ptr %2
    %13 = call i64 (ptr) @getSec1_72(ptr noundef %12)
    %14 = sub nsw i64 %11, %13
    %15 = sext i32 1000000000 to i64
    %16 = mul nsw i64 %14, %15
    %17 = call i64 (ptr) @getNsec1_80(ptr noundef %3)
    %18 = load ptr, ptr %2
    %19 = call i64 (ptr) @getNsec1_80(ptr noundef %18)
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
    store ptr @.str.1.399, ptr %6
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
    store ptr @.str.1.408, ptr %6
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
    store ptr @.str.1.417, ptr %6
    br label %46
42:
    %43 = load i64, ptr %4
    %44 = sext i32 1000000000 to i64
    %45 = sdiv i64 %43, %44
    store i64 %45, ptr %4
    store ptr @.str.1.423, ptr %6
    br label %46
46:
    %47 = load ptr, ptr %6
    store ptr %47, ptr %5
    %48 = load i64, ptr %4
    %49 = load ptr, ptr %5
    %50 = call i32 (ptr, ...) @printf(ptr noundef @.str.1.430, i64 noundef %48, ptr noundef %49)
    %51 = sext i8 0 to i64
    ret i64 %51
    unreachable
}

define void @runTests1_444() {
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
    %16 = call i32 (ptr, ...) @printf(ptr noundef @.str.1.459, i32 noundef %15)
    %17 = load i32, ptr %2
    %18 = load i32, ptr %2
    %19 = sext i8 1 to i32
    %20 = add nsw i32 %18, %19
    %21 = call i32 (ptr, ...) @printf(ptr noundef @.str.1.463, i32 noundef %17, i32 noundef %20)
    store i32 %21, ptr %5
    br label %22
22:
    %23 = call [20 x i8] () @new1_565()
    store [20 x i8] %23, ptr %3
    call void (ptr) @runTests1_582(ptr noundef %3)
    ret void
}

define [0 x i8] @new1_505() {
    %1 = alloca [0 x i8], align 1
    br label %2
2:
    %3 = load [0 x i8], ptr %1
    ret [0 x i8] %3
    unreachable
}

define void @printReset1_510(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = call i32 (ptr) @printf(ptr noundef @.str.1.513)
    ret void
}

define void @assertInt1_516(ptr noundef %0, i32 noundef %1, i32 noundef %2, ptr noundef %3) {
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
    %17 = call i32 (ptr, ...) @printf(ptr noundef @.str.1.528, i32 noundef %14, i32 noundef %15, ptr noundef %16)
    %18 = load ptr, ptr %5
    call void (ptr) @printReset1_510(ptr noundef %18)
    %19 = sext i8 1 to i32
    call void (i32) @exit(i32 noundef %19)
    br label %20
20:
    ret void
}

define void @printTestSucces1_542(ptr noundef %0, i32 noundef %1) {
    %3 = alloca [8 x i8], align 8
    %4 = alloca [4 x i8], align 4
    br label %5
5:
    store ptr %0, ptr %3
    store i32 %1, ptr %4
    %6 = load i32, ptr %4
    %7 = call i32 (ptr, ...) @printf(ptr noundef @.str.1.547, i32 noundef %6)
    %8 = load ptr, ptr %3
    call void (ptr) @printReset1_510(ptr noundef %8)
    ret void
}

define [20 x i8] @new1_565() {
    %1 = alloca [20 x i8], align 1
    br label %2
2:
    %3 = call [16 x i8] () @new1_121()
    store [16 x i8] %3, ptr %1
    %4 = call [0 x i8] () @new1_505()
    %5 = getelementptr inbounds i8, ptr %1, i64 16
    store [0 x i8] %4, ptr %5
    %6 = sext i8 1 to i32
    %7 = getelementptr inbounds i8, ptr %1, i64 16
    store i32 %6, ptr %7
    %8 = load [20 x i8], ptr %1
    ret [20 x i8] %8
    unreachable
}

define void @runTests1_582(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = load ptr, ptr %2
    call void (ptr) @testPush1_593(ptr noundef %4)
    %5 = load ptr, ptr %2
    call void (ptr) @testPop1_627(ptr noundef %5)
    ret void
}

define void @testPush1_593(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = load ptr, ptr %2
    %5 = sext i8 2 to i32
    call void (ptr, i32) @push1_132(ptr noundef %4, i32 noundef %5)
    %6 = load ptr, ptr %2
    %7 = getelementptr inbounds i8, ptr %6, i64 16
    %8 = load ptr, ptr %2
    %9 = call ptr (ptr) @last1_216(ptr noundef %8)
    %10 = load i32, ptr %9
    %11 = sext i8 2 to i32
    call void (ptr, i32, i32, ptr) @assertInt1_516(ptr noundef %7, i32 noundef %10, i32 noundef %11, ptr noundef @.str.1.614)
    %12 = load ptr, ptr %2
    %13 = getelementptr inbounds i8, ptr %12, i64 16
    %14 = load ptr, ptr %2
    %15 = call i32 (ptr) @getTestCount1_661(ptr noundef %14)
    call void (ptr, i32) @printTestSucces1_542(ptr noundef %13, i32 noundef %15)
    ret void
}

define void @testPop1_627(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = load ptr, ptr %2
    %5 = sext i8 94 to i32
    call void (ptr, i32) @push1_132(ptr noundef %4, i32 noundef %5)
    %6 = load ptr, ptr %2
    %7 = getelementptr inbounds i8, ptr %6, i64 16
    %8 = load ptr, ptr %2
    %9 = call i32 (ptr) @pop1_244(ptr noundef %8)
    %10 = sext i8 94 to i32
    call void (ptr, i32, i32, ptr) @assertInt1_516(ptr noundef %7, i32 noundef %9, i32 noundef %10, ptr noundef @.str.1.614)
    %11 = load ptr, ptr %2
    %12 = getelementptr inbounds i8, ptr %11, i64 16
    %13 = load ptr, ptr %2
    %14 = call i32 (ptr) @getTestCount1_661(ptr noundef %13)
    call void (ptr, i32) @printTestSucces1_542(ptr noundef %12, i32 noundef %14)
    ret void
}

define i32 @getTestCount1_661(ptr noundef %0) {
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

