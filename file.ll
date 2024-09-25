define i32 @main() {
    %1 = alloca i32, align 4
    %2 = alloca i32, align 4
    %3 = alloca i32, align 4
    %4 = alloca i32, align 4
    %5 = alloca i32, align 4
    %6 = alloca i32, align 4
    %7 = alloca i32, align 4
    %8 = alloca i32, align 4
    %9 = alloca i32, align 4
    %10 = alloca i32, align 4
    %11 = alloca i32, align 4
    %12 = alloca i32, align 4
    %13 = alloca i32, align 4
    %14 = alloca i32, align 4
    br label %15
15:
    store i32 0, ptr %2
    store i32 2, ptr %3
    store i32 2, ptr %4
    %16 = load i32, ptr %3
    %17 = load i32, ptr %2
    %18 = add nsw i32 %16, %17
    store i32 %18, ptr %1
    %19 = add nsw i32 2, 3
    %20 = mul nsw i32 %19, 9
    store i32 %20, ptr %5
    %21 = load i32, ptr %5
    %22 = add nsw i32 1, %21
    %23 = sub nsw i32 6, %22
    store i32 %23, ptr %6
    %24 = load i32, ptr %5
    %25 = icmp eq i32 %24, 2
    br i1 %25, label %26, label %27
26:
    store i32 2, ptr %8
    store i32 2, ptr %13
    br label %30
27:
    %28 = load i32, ptr %5
    %29 = icmp eq i32 %28, 9
    br i1 %29, label %30, label %31
30:
    store i32 3, ptr %9
    store i32 99, ptr %13
    br label %32
31:
    store i32 4, ptr %10
    store i32 7, ptr %13
    br label %32
32:
    %33 = load i32, ptr %13
    %34 = load i32, ptr %13
    store i32 %34, ptr %7
    %35 = load i32, ptr %5
    %36 = load i32, ptr %7
    %37 = add nsw i32 %35, %36
    %38 = load i32, ptr %6
    %39 = add nsw i32 %37, %38
    store i32 %39, ptr %11
    store i32 2, ptr %11
    %40 = icmp eq i1 1, 1
    br i1 %40, label %41, label %42
41:
    store i32 1, ptr %14
    br label %44
42:
    %43 = load i32, ptr %11
    store i32 %43, ptr %14
    br label %44
44:
    %45 = load i32, ptr %14
    store i32 %45, ptr %12
    %46 = load i32, ptr %11
    %47 = add nsw i32 928, %46
    store i32 %47, ptr %11
    ret i32 0
}

