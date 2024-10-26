declare void @exit(i32 noundef)
declare i32 @printf(ptr noundef)

define i32 @main() {
    %1 = alloca [4 x i8], align 4
    %2 = alloca [4 x i8], align 4
    %3 = alloca [4 x i8], align 4
    %4 = alloca [28 x i8], align 4
    %5 = alloca [8 x i8], align 4
    %6 = alloca [4 x i8], align 4
    %7 = alloca [28 x i8], align 4
    %8 = alloca [16 x i8], align 4
    %9 = alloca [8 x i8], align 4
    %10 = alloca [8 x i8], align 4
    br label %11
11:
    store i32 2, ptr %1
    store i32 2, ptr %2
    %12 = load i32, ptr %1
    %13 = load i32, ptr %2
    %14 = add nsw i32 %12, %13
    store i32 %14, ptr %3
    store i32 1, ptr %7
    %15 = getelementptr inbounds i8, ptr %7, i64 4
    store i32 2, ptr %15
    %16 = getelementptr inbounds i8, ptr %7, i64 8
    store i32 3, ptr %16
    store i32 1, ptr %8
    %17 = getelementptr inbounds i8, ptr %8, i64 4
    store i32 2, ptr %17
    store i32 3, ptr %9
    %18 = getelementptr inbounds i8, ptr %9, i64 4
    store i32 4, ptr %18
    %19 = load [8 x i8], ptr %9
    %20 = getelementptr inbounds i8, ptr %8, i64 8
    store [8 x i8] %19, ptr %20
    %21 = load [16 x i8], ptr %8
    %22 = getelementptr inbounds i8, ptr %7, i64 12
    store [16 x i8] %21, ptr %22
    %23 = load [28 x i8], ptr %7
    store [28 x i8] %23, ptr %4
    store i32 2, ptr %10
    %24 = getelementptr inbounds i8, ptr %10, i64 4
    store i32 3, ptr %24
    %25 = load [8 x i8], ptr %10
    store [8 x i8] %25, ptr %5
    %26 = getelementptr inbounds i8, ptr %4, i64 8
    %27 = load i32, ptr %26
    %28 = add nsw i32 2, %27
    %29 = load i32, ptr %5
    %30 = mul nsw i32 %29, 2
    %31 = add nsw i32 %28, %30
    %32 = getelementptr inbounds i8, ptr %4, i64 12
    %33 = getelementptr inbounds i8, ptr %32, i64 8
    %34 = getelementptr inbounds i8, ptr %33, i64 4
    %35 = load i32, ptr %34
    %36 = add nsw i32 %31, %35
    store i32 %36, ptr %3
    %37 = load i32, ptr %3
    %38 = add nsw i32 %37, 2
    %39 = call i32 () @givemeint14()
    %40 = add nsw i32 %38, %39
    %41 = call i32 (i32, i32) @add18(i32 noundef 2, i32 noundef 3)
    %42 = add nsw i32 %40, %41
    %43 = load [28 x i8], ptr %4
    %44 = call i32 ([28 x i8]) @sum47([28 x i8] noundef %43)
    %45 = add nsw i32 %42, %44
    %46 = call ptr () @returnFunction35()
    %47 = call i32 (i32, i32) %46(i32 noundef 2, i32 noundef 3)
    %48 = add nsw i32 %45, %47
    store i32 %48, ptr %6
    ret i32 0
}

define i32 @givemeint14() {
    br label %1
1:
    ret i32 2
    unreachable
}

define i32 @doAddition21(i32 noundef %0, i32 noundef %1) {
    %3 = alloca [4 x i8], align 4
    %4 = alloca [4 x i8], align 4
    br label %5
5:
    store i32 %0, ptr %3
    store i32 %1, ptr %4
    %6 = load i32, ptr %3
    %7 = load i32, ptr %4
    %8 = add nsw i32 %6, %7
    ret i32 %8
    unreachable
}

define i32 @add18(i32 noundef %0, i32 noundef %1) {
    %3 = alloca [4 x i8], align 4
    %4 = alloca [4 x i8], align 4
    br label %5
5:
    store i32 %0, ptr %3
    store i32 %1, ptr %4
    %6 = load i32, ptr %3
    %7 = load i32, ptr %4
    %8 = call i32 (i32, i32) @doAddition21(i32 noundef %6, i32 noundef %7)
    ret i32 %8
    unreachable
}

define i32 @addTwo36(i32 noundef %0, i32 noundef %1) {
    %3 = alloca [4 x i8], align 4
    %4 = alloca [4 x i8], align 4
    br label %5
5:
    store i32 %0, ptr %3
    store i32 %1, ptr %4
    %6 = load i32, ptr %3
    %7 = load i32, ptr %4
    %8 = add nsw i32 %6, %7
    ret i32 %8
    unreachable
}

define ptr @returnFunction35() {
    br label %1
1:
    ret ptr @addTwo36
    unreachable
}

define i32 @sum47([28 x i8] noundef %0) {
    %2 = alloca [28 x i8], align 4
    br label %3
3:
    store [28 x i8] %0, ptr %2
    %4 = load i32, ptr %2
    %5 = getelementptr inbounds i8, ptr %2, i64 4
    %6 = load i32, ptr %5
    %7 = add nsw i32 %4, %6
    %8 = getelementptr inbounds i8, ptr %2, i64 8
    %9 = load i32, ptr %8
    %10 = add nsw i32 %7, %9
    ret i32 %10
    unreachable
}

