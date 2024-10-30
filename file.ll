declare ptr @realloc(ptr noundef, i32 noundef)
declare ptr @malloc(i32 noundef)
declare i32 @socket(i32 noundef, i32 noundef, i32 noundef)
declare void @exit(i32 noundef)
declare i32 @printf(ptr noundef, ...)
declare i32 @asprintf(ptr noundef, ptr noundef, ...)

@.str.166 = private unnamed_addr constant [10 x i8] c"Len = %d\0A\00"
@.str.480 = private unnamed_addr constant [13 x i8] c"vec.cap != 2\00"
@.str.386 = private unnamed_addr constant [1 x i8] c"\00"
@.str.297 = private unnamed_addr constant [5 x i8] c"\1B[0m\00"
@.str.172 = private unnamed_addr constant [10 x i8] c"Cap = %d\0A\00"
@.str.491 = private unnamed_addr constant [18 x i8] c"vec.items[0] != 2\00"
@.str.452 = private unnamed_addr constant [19 x i8] c"vec.len != vec.cap\00"
@.str.309 = private unnamed_addr constant [34 x i8] c"\1B[31mAssert: %d != %d, Err: '%s'\0A\00"
@.str.187 = private unnamed_addr constant [11 x i8] c"[%d] = %d\0A\00"
@.str.471 = private unnamed_addr constant [13 x i8] c"vec.len != 1\00"
@.str.327 = private unnamed_addr constant [21 x i8] c"\1B[32mTest %d passed\0A\00"
@.str.205 = private unnamed_addr constant [2 x i8] c"\0A\00"

define i32 @main() {
    %1 = alloca [16 x i8], align 4
    br label %2
2:
    call void () @runTests418()
    %3 = call [16 x i8] () @new32()
    store [16 x i8] %3, ptr %1
    call void (ptr, i32) @push43(ptr noundef %1, i32 noundef 0)
    call void (ptr, i32) @push43(ptr noundef %1, i32 noundef 1)
    call void (ptr, i32) @push43(ptr noundef %1, i32 noundef 2)
    call void (ptr) @debug163(ptr noundef %1)
    %4 = call i32 (ptr) @pop143(ptr noundef %1)
    call void (ptr) @debug163(ptr noundef %1)
    call void (ptr, i32) @push43(ptr noundef %1, i32 noundef 10)
    call void (ptr, i32) @push43(ptr noundef %1, i32 noundef 20)
    call void (ptr, i32) @push43(ptr noundef %1, i32 noundef 30)
    call void (ptr) @debug163(ptr noundef %1)
    %5 = call ptr (ptr) @lastMut130(ptr noundef %1)
    store i32 2, ptr %5
    call void (ptr) @debug163(ptr noundef %1)
    ret i32 0
}

define [16 x i8] @new32() {
    %1 = alloca [16 x i8], align 4
    br label %2
2:
    store i32 0, ptr %1
    %3 = getelementptr inbounds i8, ptr %1, i64 4
    store i32 0, ptr %3
    %4 = getelementptr inbounds i8, ptr %1, i64 8
    store ptr null, ptr %4
    %5 = load [16 x i8], ptr %1
    ret [16 x i8] %5
    unreachable
}

define void @push43(ptr noundef %0, i32 noundef %1) {
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
    br i1 %14, label %15, label %50
15:
    %16 = load ptr, ptr %3
    %17 = getelementptr inbounds i8, ptr %16, i64 4
    %18 = load ptr, ptr %3
    %19 = getelementptr inbounds i8, ptr %18, i64 4
    %20 = load i32, ptr %19
    %21 = icmp eq i32 %20, 0
    br i1 %21, label %22, label %23
22:
    store i32 2, ptr %6
    br label %28
23:
    %24 = load ptr, ptr %3
    %25 = getelementptr inbounds i8, ptr %24, i64 4
    %26 = load i32, ptr %25
    %27 = mul nsw i32 %26, 2
    store i32 %27, ptr %6
    br label %28
28:
    %29 = load i32, ptr %6
    store i32 %29, ptr %17
    %30 = load ptr, ptr %3
    %31 = getelementptr inbounds i8, ptr %30, i64 4
    %32 = load i32, ptr %31
    %33 = mul nsw i32 %32, 4
    store i32 %33, ptr %5
    %34 = load ptr, ptr %3
    %35 = getelementptr inbounds i8, ptr %34, i64 8
    %36 = load ptr, ptr %3
    %37 = load i32, ptr %36
    %38 = icmp eq i32 %37, 0
    br i1 %38, label %39, label %42
39:
    %40 = load i32, ptr %5
    %41 = call ptr (i32) @malloc(i32 noundef %40)
    store ptr %41, ptr %7
    br label %48
42:
    %43 = load ptr, ptr %3
    %44 = getelementptr inbounds i8, ptr %43, i64 8
    %45 = load ptr, ptr %44
    %46 = load i32, ptr %5
    %47 = call ptr (ptr, i32) @realloc(ptr noundef %45, i32 noundef %46)
    store ptr %47, ptr %7
    br label %48
48:
    %49 = load ptr, ptr %7
    store ptr %49, ptr %35
    br label %50
50:
    %51 = load ptr, ptr %3
    %52 = getelementptr inbounds i8, ptr %51, i64 8
    %53 = load ptr, ptr %52
    %54 = load ptr, ptr %3
    %55 = load i32, ptr %54
    %56 = getelementptr inbounds i32, ptr %53, i32 %55
    %57 = load i32, ptr %4
    store i32 %57, ptr %56
    %58 = load ptr, ptr %3
    %59 = load ptr, ptr %3
    %60 = load i32, ptr %59
    %61 = add nsw i32 %60, 1
    store i32 %61, ptr %58
    ret void
}

define ptr @last117(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = load ptr, ptr %2
    %5 = getelementptr inbounds i8, ptr %4, i64 8
    %6 = load ptr, ptr %5
    %7 = load ptr, ptr %2
    %8 = load i32, ptr %7
    %9 = sub nsw i32 %8, 1
    %10 = getelementptr inbounds i32, ptr %6, i32 %9
    ret ptr %10
    unreachable
}

define ptr @lastMut130(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = load ptr, ptr %2
    %5 = getelementptr inbounds i8, ptr %4, i64 8
    %6 = load ptr, ptr %5
    %7 = load ptr, ptr %2
    %8 = load i32, ptr %7
    %9 = sub nsw i32 %8, 1
    %10 = getelementptr inbounds i32, ptr %6, i32 %9
    ret ptr %10
    unreachable
}

define i32 @pop143(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = load ptr, ptr %2
    %5 = load ptr, ptr %2
    %6 = load i32, ptr %5
    %7 = sub nsw i32 %6, 1
    store i32 %7, ptr %4
    %8 = load ptr, ptr %2
    %9 = getelementptr inbounds i8, ptr %8, i64 8
    %10 = load ptr, ptr %9
    %11 = load ptr, ptr %2
    %12 = load i32, ptr %11
    %13 = getelementptr inbounds i32, ptr %10, i32 %12
    %14 = load i32, ptr %13
    ret i32 %14
    unreachable
}

define void @debug163(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    %3 = alloca [4 x i8], align 4
    br label %4
4:
    store ptr %0, ptr %2
    %5 = load ptr, ptr %2
    %6 = load i32, ptr %5
    %7 = call i32 (ptr, ...) @printf(ptr noundef @.str.166, i32 noundef %6)
    %8 = load ptr, ptr %2
    %9 = getelementptr inbounds i8, ptr %8, i64 4
    %10 = load i32, ptr %9
    %11 = call i32 (ptr, ...) @printf(ptr noundef @.str.172, i32 noundef %10)
    store i32 0, ptr %3
    br label %12
12:
    %13 = load i32, ptr %3
    %14 = load ptr, ptr %2
    %15 = load i32, ptr %14
    %16 = icmp eq i32 %13, %15
    br i1 %16, label %17, label %19
17:
    br label %31
18:
    br label %30
19:
    %20 = load i32, ptr %3
    %21 = load ptr, ptr %2
    %22 = getelementptr inbounds i8, ptr %21, i64 8
    %23 = load ptr, ptr %22
    %24 = load i32, ptr %3
    %25 = getelementptr inbounds i32, ptr %23, i32 %24
    %26 = load i32, ptr %25
    %27 = call i32 (ptr, ...) @printf(ptr noundef @.str.187, i32 noundef %20, i32 noundef %26)
    %28 = load i32, ptr %3
    %29 = add nsw i32 %28, 1
    store i32 %29, ptr %3
    br label %30
30:
    br label %12
31:
    %32 = call i32 (ptr) @printf(ptr noundef @.str.205)
    ret void
}

define [16 x i8] @newVec209() {
    %1 = alloca [16 x i8], align 4
    br label %2
2:
    store i32 0, ptr %1
    %3 = getelementptr inbounds i8, ptr %1, i64 4
    store i32 0, ptr %3
    %4 = getelementptr inbounds i8, ptr %1, i64 8
    store ptr null, ptr %4
    %5 = load [16 x i8], ptr %1
    ret [16 x i8] %5
    unreachable
}

define [0 x i8] @new289() {
    %1 = alloca [0 x i8], align 1
    br label %2
2:
    %3 = load [0 x i8], ptr %1
    ret [0 x i8] %3
    unreachable
}

define void @printReset294([0 x i8] noundef %0) {
    %2 = alloca [0 x i8], align 1
    br label %3
3:
    store [0 x i8] %0, ptr %2
    %4 = call i32 (ptr) @printf(ptr noundef @.str.297)
    ret void
}

define void @assertInt300([0 x i8] noundef %0, i32 noundef %1, i32 noundef %2, ptr noundef %3) {
    %5 = alloca [0 x i8], align 1
    %6 = alloca [4 x i8], align 4
    %7 = alloca [4 x i8], align 4
    %8 = alloca [8 x i8], align 8
    br label %9
9:
    store [0 x i8] %0, ptr %5
    store i32 %1, ptr %6
    store i32 %2, ptr %7
    store ptr %3, ptr %8
    %10 = load i32, ptr %6
    %11 = load i32, ptr %7
    %12 = icmp ne i32 %10, %11
    br i1 %12, label %13, label %19
13:
    %14 = load i32, ptr %6
    %15 = load i32, ptr %7
    %16 = load ptr, ptr %8
    %17 = call i32 (ptr, ...) @printf(ptr noundef @.str.309, i32 noundef %14, i32 noundef %15, ptr noundef %16)
    %18 = load [0 x i8], ptr %5
    call void ([0 x i8]) @printReset294([0 x i8] noundef %18)
    call void (i32) @exit(i32 noundef 1)
    br label %19
19:
    ret void
}

define void @printTestSucces323([0 x i8] noundef %0, i32 noundef %1) {
    %3 = alloca [0 x i8], align 1
    %4 = alloca [4 x i8], align 4
    br label %5
5:
    store [0 x i8] %0, ptr %3
    store i32 %1, ptr %4
    %6 = load i32, ptr %4
    %7 = call i32 (ptr, ...) @printf(ptr noundef @.str.327, i32 noundef %6)
    %8 = load [0 x i8], ptr %3
    call void ([0 x i8]) @printReset294([0 x i8] noundef %8)
    ret void
}

define [20 x i8] @new342() {
    %1 = alloca [20 x i8], align 1
    br label %2
2:
    %3 = call [16 x i8] () @new32()
    store [16 x i8] %3, ptr %1
    %4 = call [0 x i8] () @new289()
    %5 = getelementptr inbounds i8, ptr %1, i64 16
    store [0 x i8] %4, ptr %5
    %6 = getelementptr inbounds i8, ptr %1, i64 16
    store i32 1, ptr %6
    %7 = load [20 x i8], ptr %1
    ret [20 x i8] %7
    unreachable
}

define void @runTests359(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = load ptr, ptr %2
    call void (ptr) @testPush366(ptr noundef %4)
    ret void
}

define void @testPush366(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = load ptr, ptr %2
    call void (ptr, i32) @push43(ptr noundef %4, i32 noundef 2)
    %5 = load ptr, ptr %2
    %6 = getelementptr inbounds i8, ptr %5, i64 16
    %7 = load [0 x i8], ptr %6
    %8 = load ptr, ptr %2
    %9 = getelementptr inbounds i8, ptr %8, i64 4
    %10 = load i32, ptr %9
    call void ([0 x i8], i32, i32, ptr) @assertInt300([0 x i8] noundef %7, i32 noundef %10, i32 noundef 2, ptr noundef @.str.386)
    %11 = load ptr, ptr %2
    %12 = getelementptr inbounds i8, ptr %11, i64 16
    %13 = load [0 x i8], ptr %12
    %14 = load ptr, ptr %2
    %15 = call i32 (ptr) @getTestCount399(ptr noundef %14)
    call void ([0 x i8], i32) @printTestSucces323([0 x i8] noundef %13, i32 noundef %15)
    ret void
}

define i32 @getTestCount399(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = load ptr, ptr %2
    %5 = getelementptr inbounds i8, ptr %4, i64 16
    %6 = load ptr, ptr %2
    %7 = getelementptr inbounds i8, ptr %6, i64 16
    %8 = load i32, ptr %7
    %9 = add nsw i32 %8, 1
    store i32 %9, ptr %5
    %10 = load ptr, ptr %2
    %11 = getelementptr inbounds i8, ptr %10, i64 16
    %12 = load i32, ptr %11
    %13 = sub nsw i32 %12, 1
    ret i32 %13
    unreachable
}

define void @runTests418() {
    %1 = alloca [20 x i8], align 1
    %2 = alloca [0 x i8], align 1
    %3 = alloca [20 x i8], align 1
    %4 = alloca [16 x i8], align 4
    %5 = alloca [0 x i8], align 1
    br label %6
6:
    %7 = call [20 x i8] () @new342()
    store [20 x i8] %7, ptr %1
    call void (ptr) @runTests359(ptr noundef %1)
    %8 = load [0 x i8], ptr %5
    store [0 x i8] %8, ptr %2
    %9 = call [20 x i8] () @new342()
    store [20 x i8] %9, ptr %3
    %10 = call [16 x i8] () @newVec209()
    store [16 x i8] %10, ptr %4
    %11 = load [0 x i8], ptr %2
    %12 = load i32, ptr %4
    %13 = getelementptr inbounds i8, ptr %4, i64 4
    %14 = load i32, ptr %13
    call void ([0 x i8], i32, i32, ptr) @assertInt300([0 x i8] noundef %11, i32 noundef %12, i32 noundef %14, ptr noundef @.str.452)
    %15 = load [0 x i8], ptr %2
    call void ([0 x i8], i32) @printTestSucces323([0 x i8] noundef %15, i32 noundef 1)
    call void (ptr, i32) @push43(ptr noundef %4, i32 noundef 5)
    %16 = load [0 x i8], ptr %2
    %17 = load i32, ptr %4
    call void ([0 x i8], i32, i32, ptr) @assertInt300([0 x i8] noundef %16, i32 noundef %17, i32 noundef 1, ptr noundef @.str.471)
    %18 = load [0 x i8], ptr %2
    %19 = getelementptr inbounds i8, ptr %4, i64 4
    %20 = load i32, ptr %19
    call void ([0 x i8], i32, i32, ptr) @assertInt300([0 x i8] noundef %18, i32 noundef %20, i32 noundef 2, ptr noundef @.str.480)
    %21 = load [0 x i8], ptr %2
    %22 = getelementptr inbounds i8, ptr %4, i64 8
    %23 = load ptr, ptr %22
    %24 = getelementptr inbounds i32, ptr %23, i32 0
    %25 = load i32, ptr %24
    call void ([0 x i8], i32, i32, ptr) @assertInt300([0 x i8] noundef %21, i32 noundef %25, i32 noundef 5, ptr noundef @.str.491)
    %26 = load [0 x i8], ptr %2
    call void ([0 x i8], i32) @printTestSucces323([0 x i8] noundef %26, i32 noundef 2)
    ret void
}

