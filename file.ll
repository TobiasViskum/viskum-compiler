define i32 @main() {
    %1 = alloca i32, align 4
    %2 = alloca i32, align 4
    %3 = alloca i32, align 4
    %4 = alloca i32, align 4
    %5 = alloca i1, align 4
    %6 = alloca i32, align 4
    %7 = alloca i32, align 4
    %8 = alloca i32, align 4
    %9 = alloca i32, align 4
    %10 = alloca i32, align 4
    %11 = alloca i32, align 4
    %12 = alloca i32, align 4
    %13 = alloca i32, align 4
    %14 = alloca i32, align 4
    %15 = alloca i32, align 4
    %16 = alloca i32, align 4
    br label %17
17:
    store i32 0, ptr %2
    store i32 2, ptr %3
    store i32 2, ptr %4
    %18 = load i32, ptr %3
    %19 = load i32, ptr %2
    %20 = add nsw i32 %18, %19
    store i32 %20, ptr %1
    store i1 1, ptr %5
    %21 = add nsw i32 2, 3
    %22 = mul nsw i32 %21, 9
    store i32 %22, ptr %6
    %23 = load i32, ptr %6
    %24 = add nsw i32 1, %23
    %25 = sub nsw i32 6, %24
    store i32 %25, ptr %7
    %26 = load i32, ptr %6
    %27 = icmp eq i32 %26, 2
    br i1 %27, label %28, label %29
28:
    store i32 2, ptr %9
    store i32 2, ptr %15
    br label %32
29:
    %30 = load i32, ptr %6
    %31 = icmp eq i32 %30, 9
    br i1 %31, label %32, label %33
32:
    store i32 3, ptr %10
    store i32 99, ptr %15
    br label %34
33:
    store i32 4, ptr %11
    store i32 7, ptr %15
    br label %34
34:
    %35 = load i32, ptr %15
    %36 = load i32, ptr %15
    store i32 %36, ptr %8
    %37 = load i32, ptr %6
    %38 = load i32, ptr %7
    %39 = add nsw i32 %37, %38
    %40 = load i32, ptr %8
    %41 = add nsw i32 %39, %40
    store i32 %41, ptr %12
    store i32 2, ptr %12
    %42 = icmp eq i1 1, 1
    br i1 %42, label %43, label %44
43:
    store i32 1, ptr %16
    br label %46
44:
    %45 = load i32, ptr %12
    store i32 %45, ptr %16
    br label %46
46:
    %47 = load i32, ptr %16
    store i32 %47, ptr %13
    %48 = load i32, ptr %12
    %49 = add nsw i32 928, %48
    store i32 %49, ptr %12
    store i32 0, ptr %14
    %50 = load i32, ptr %14
    %51 = icmp eq i32 %50, 10
    br i1 %51, label %52, label %55
52:
    %53 = load i32, ptr %14
    %54 = add nsw i32 %53, 1
    store i32 %54, ptr %14
    br label %55
55:
    ret i32 0
}

