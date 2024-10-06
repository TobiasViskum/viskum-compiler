define i32 @main() {
    %1 = alloca [4 x i8], align 4
    %2 = alloca [4 x i8], align 4
    %3 = alloca [4 x i8], align 4
    %4 = alloca [4 x i8], align 4
    %5 = alloca [4 x i8], align 4
    %6 = alloca [12 x i8], align 4
    %7 = alloca [1 x i8], align 1
    %8 = alloca [8 x i8], align 4
    %9 = alloca [4 x i8], align 4
    %10 = alloca [4 x i8], align 4
    %11 = alloca [4 x i8], align 4
    %12 = alloca [4 x i8], align 4
    %13 = alloca [4 x i8], align 4
    %14 = alloca [4 x i8], align 4
    %15 = alloca [4 x i8], align 4
    %16 = alloca [4 x i8], align 4
    %17 = alloca [4 x i8], align 4
    %18 = alloca [4 x i8], align 4
    %19 = alloca [12 x i8], align 4
    %20 = alloca [8 x i8], align 4
    %21 = alloca [4 x i8], align 4
    %22 = alloca [4 x i8], align 4
    br label %23
23:
    store i32 0, ptr %2
    store i32 2, ptr %3
    store i32 2, ptr %4
    %24 = load i32, ptr %3
    %25 = load i32, ptr %2
    %26 = add nsw i32 %24, %25
    store i32 %26, ptr %1
    br i1 1, label %27, label %28
27:
    store i32 1, ptr %18
    br label %29
28:
    store i32 0, ptr %18
    br label %29
29:
    %30 = load i32, ptr %18
    store i32 %30, ptr %5
    store i32 3, ptr %19
    %31 = getelementptr inbounds i8, ptr %19, i64 4
    store i32 4, ptr %31
    %32 = getelementptr inbounds i8, ptr %19, i64 8
    store i32 5, ptr %32
    %33 = load [12 x i8], ptr %19
    store [12 x i8] %33, ptr %6
    store i1 1, ptr %7
    store i32 7, ptr %20
    %34 = getelementptr inbounds i8, ptr %20, i64 4
    store i32 8, ptr %34
    %35 = load [8 x i8], ptr %20
    store [8 x i8] %35, ptr %8
    %36 = add nsw i32 2, 3
    %37 = mul nsw i32 %36, 9
    %38 = getelementptr inbounds i8, ptr %8, i64 4
    %39 = load i32, ptr %38
    %40 = add nsw i32 %37, %39
    store i32 %40, ptr %9
    %41 = load i32, ptr %9
    %42 = add nsw i32 1, %41
    %43 = sub nsw i32 6, %42
    store i32 %43, ptr %10
    %44 = load i32, ptr %9
    %45 = icmp eq i32 %44, 2
    br i1 %45, label %46, label %47
46:
    store i32 2, ptr %12
    store i32 2, ptr %21
    br label %50
47:
    %48 = load i32, ptr %9
    %49 = icmp eq i32 %48, 9
    br i1 %49, label %50, label %51
50:
    store i32 3, ptr %13
    store i32 99, ptr %21
    br label %52
51:
    store i32 4, ptr %14
    store i32 7, ptr %21
    br label %52
52:
    %53 = load i32, ptr %21
    store i32 %53, ptr %11
    %54 = load i32, ptr %9
    %55 = load i32, ptr %10
    %56 = add nsw i32 %54, %55
    %57 = load i32, ptr %11
    %58 = add nsw i32 %56, %57
    store i32 %58, ptr %15
    store i32 2, ptr %15
    %59 = icmp eq i1 1, 1
    br i1 %59, label %60, label %61
60:
    store i32 1, ptr %22
    br label %63
61:
    %62 = load i32, ptr %15
    store i32 %62, ptr %22
    br label %63
63:
    %64 = load i32, ptr %22
    store i32 %64, ptr %16
    %65 = load i32, ptr %15
    %66 = add nsw i32 928, %65
    store i32 %66, ptr %15
    store i32 0, ptr %17
    %67 = load i32, ptr %17
    %68 = icmp eq i32 %67, 10
    br i1 %68, label %69, label %72
69:
    %70 = load i32, ptr %17
    %71 = add nsw i32 %70, 1
    store i32 %71, ptr %17
    br label %72
72:
    ret i32 0
}

