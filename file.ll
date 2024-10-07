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
    %41 = getelementptr inbounds i8, ptr %6, i64 8
    %42 = load i32, ptr %41
    %43 = add nsw i32 %40, %42
    store i32 %43, ptr %9
    %44 = load i32, ptr %9
    %45 = add nsw i32 1, %44
    %46 = sub nsw i32 6, %45
    store i32 %46, ptr %10
    %47 = load i32, ptr %9
    %48 = icmp eq i32 %47, 2
    br i1 %48, label %49, label %50
49:
    store i32 2, ptr %12
    store i32 2, ptr %21
    br label %53
50:
    %51 = load i32, ptr %9
    %52 = icmp eq i32 %51, 9
    br i1 %52, label %53, label %54
53:
    store i32 3, ptr %13
    store i32 99, ptr %21
    br label %55
54:
    store i32 4, ptr %14
    store i32 7, ptr %21
    br label %55
55:
    %56 = load i32, ptr %21
    store i32 %56, ptr %11
    %57 = load i32, ptr %9
    %58 = load i32, ptr %10
    %59 = add nsw i32 %57, %58
    %60 = load i32, ptr %11
    %61 = add nsw i32 %59, %60
    store i32 %61, ptr %15
    store i32 2, ptr %15
    %62 = icmp eq i1 1, 1
    br i1 %62, label %63, label %64
63:
    store i32 1, ptr %22
    br label %66
64:
    %65 = load i32, ptr %15
    store i32 %65, ptr %22
    br label %66
66:
    %67 = load i32, ptr %22
    store i32 %67, ptr %16
    %68 = load i32, ptr %15
    %69 = add nsw i32 928, %68
    store i32 %69, ptr %15
    store i32 0, ptr %17
    %70 = load i32, ptr %17
    %71 = icmp eq i32 %70, 10
    br i1 %71, label %72, label %75
72:
    %73 = load i32, ptr %17
    %74 = add nsw i32 %73, 1
    store i32 %74, ptr %17
    br label %75
75:
    ret i32 0
}

