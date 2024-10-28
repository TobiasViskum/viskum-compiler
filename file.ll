declare ptr @realloc(ptr noundef, i32 noundef)
declare ptr @malloc(i32 noundef)
declare i32 @socket(i32 noundef, i32 noundef, i32 noundef)
declare void @exit(i32 noundef)

define i32 @main() {
    %1 = alloca [16 x i8], align 4
    %2 = alloca [8 x i8], align 8
    %3 = alloca [4 x i8], align 4
    %4 = alloca [8 x i8], align 8
    br label %5
5:
    %6 = call [16 x i8] () @newVec22()
    store [16 x i8] %6, ptr %1
    call void (ptr, i32) @push33(ptr noundef %1, i32 noundef 0)
    %7 = call ptr (ptr) @getLastMut127(ptr noundef %1)
    store ptr %7, ptr %2
    %8 = load ptr, ptr %2
    %9 = load ptr, ptr %2
    store i32 0, ptr %9
    %10 = call i32 (ptr) @pop107(ptr noundef %1)
    store i32 %10, ptr %3
    call void (ptr, i32) @push33(ptr noundef %1, i32 noundef 0)
    %11 = call ptr (ptr) @getLast140(ptr noundef %1)
    store ptr %11, ptr %4
    %12 = load ptr, ptr %4
    %13 = load i32, ptr %12
    call void (i32) @exit(i32 noundef %13)
    ret i32 0
}

define [16 x i8] @newVec22() {
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

define void @push33(ptr noundef %0, i32 noundef %1) {
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
    %20 = load ptr, ptr %3
    %21 = getelementptr inbounds i8, ptr %20, i64 4
    %22 = load i32, ptr %21
    %23 = icmp eq i32 %22, 0
    br i1 %23, label %24, label %25
24:
    store i32 2, ptr %6
    br label %30
25:
    %26 = load ptr, ptr %3
    %27 = getelementptr inbounds i8, ptr %26, i64 4
    %28 = load i32, ptr %27
    %29 = mul nsw i32 %28, 2
    store i32 %29, ptr %6
    br label %30
30:
    %31 = load i32, ptr %6
    store i32 %31, ptr %19
    %32 = load ptr, ptr %3
    %33 = getelementptr inbounds i8, ptr %32, i64 4
    %34 = load i32, ptr %33
    %35 = mul nsw i32 %34, 4
    store i32 %35, ptr %5
    %36 = load ptr, ptr %3
    %37 = getelementptr inbounds i8, ptr %36, i64 8
    %38 = load ptr, ptr %37
    %39 = load ptr, ptr %3
    %40 = getelementptr inbounds i8, ptr %39, i64 8
    %41 = load ptr, ptr %3
    %42 = load i32, ptr %41
    %43 = icmp eq i32 %42, 0
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
    store ptr %54, ptr %40
    br label %55
55:
    %56 = load ptr, ptr %3
    %57 = getelementptr inbounds i8, ptr %56, i64 8
    %58 = load ptr, ptr %57
    %59 = load ptr, ptr %3
    %60 = load i32, ptr %59
    %61 = getelementptr inbounds i32, ptr %58, i32 %60
    %62 = load ptr, ptr %3
    %63 = getelementptr inbounds i8, ptr %62, i64 8
    %64 = load ptr, ptr %63
    %65 = load ptr, ptr %3
    %66 = load i32, ptr %65
    %67 = getelementptr inbounds i32, ptr %64, i32 %66
    %68 = load i32, ptr %4
    store i32 %68, ptr %67
    %69 = load ptr, ptr %3
    %70 = load ptr, ptr %3
    %71 = load ptr, ptr %3
    %72 = load i32, ptr %71
    %73 = add nsw i32 %72, 1
    store i32 %73, ptr %70
    ret void
}

define i32 @pop107(ptr noundef %0) {
    %2 = alloca [8 x i8], align 8
    br label %3
3:
    store ptr %0, ptr %2
    %4 = load ptr, ptr %2
    %5 = load ptr, ptr %2
    %6 = load ptr, ptr %2
    %7 = load i32, ptr %6
    %8 = sub nsw i32 %7, 1
    store i32 %8, ptr %5
    %9 = load ptr, ptr %2
    %10 = getelementptr inbounds i8, ptr %9, i64 8
    %11 = load ptr, ptr %10
    %12 = load ptr, ptr %2
    %13 = load i32, ptr %12
    %14 = getelementptr inbounds i32, ptr %11, i32 %13
    %15 = load i32, ptr %14
    ret i32 %15
    unreachable
}

define ptr @getLastMut127(ptr noundef %0) {
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

define ptr @getLast140(ptr noundef %0) {
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

define i32 @fib153(i32 noundef %0) {
    %2 = alloca [4 x i8], align 4
    br label %3
3:
    store i32 %0, ptr %2
    %4 = load i32, ptr %2
    %5 = icmp sle i32 %4, 1
    br i1 %5, label %6, label %9
6:
    %7 = load i32, ptr %2
    ret i32 %7
    br label %9
9:
    %10 = load i32, ptr %2
    %11 = sub nsw i32 %10, 2
    %12 = call i32 (i32) @fib153(i32 noundef %11)
    %13 = load i32, ptr %2
    %14 = sub nsw i32 %13, 1
    %15 = call i32 (i32) @fib153(i32 noundef %14)
    %16 = add nsw i32 %12, %15
    ret i32 %16
    unreachable
}

