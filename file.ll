define i32 @main() {
    %1 = alloca [4 x i8], align 4
    %2 = alloca [4 x i8], align 4
    %3 = alloca [4 x i8], align 4
    %4 = alloca [4 x i8], align 4
    %5 = alloca [4 x i8], align 4
    %6 = alloca [26 x i8], align 1
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
    %19 = alloca [26 x i8], align 1
    %20 = alloca [14 x i8], align 1
    %21 = alloca [6 x i8], align 1
    %22 = alloca [8 x i8], align 4
    %23 = alloca [4 x i8], align 4
    %24 = alloca [4 x i8], align 4
    br label %25
25:
    store i32 0, ptr %2
    store i32 2, ptr %3
    store i32 2, ptr %4
    %26 = load i32, ptr %3
    %27 = load i32, ptr %2
    %28 = add nsw i32 %26, %27
    store i32 %28, ptr %1
    br i1 1, label %29, label %30
29:
    store i32 1, ptr %18
    br label %31
30:
    store i32 0, ptr %18
    br label %31
31:
    %32 = load i32, ptr %18
    store i32 %32, ptr %5
    store i32 3, ptr %19
    %33 = getelementptr inbounds i8, ptr %19, i64 4
    store i32 4, ptr %33
    %34 = getelementptr inbounds i8, ptr %19, i64 8
    store i32 5, ptr %34
    store i32 9, ptr %20
    %35 = getelementptr inbounds i8, ptr %20, i64 4
    store i32 4, ptr %35
    store i1 1, ptr %21
    %36 = getelementptr inbounds i8, ptr %21, i64 1
    store i1 0, ptr %36
    %37 = getelementptr inbounds i8, ptr %21, i64 2
    store i32 8, ptr %37
    %38 = load [6 x i8], ptr %21
    %39 = getelementptr inbounds i8, ptr %20, i64 8
    store [6 x i8] %38, ptr %39
    %40 = load [14 x i8], ptr %20
    %41 = getelementptr inbounds i8, ptr %19, i64 12
    store [14 x i8] %40, ptr %41
    %42 = load [26 x i8], ptr %19
    store [26 x i8] %42, ptr %6
    %43 = getelementptr inbounds i8, ptr %6, i64 12
    %44 = getelementptr inbounds i8, ptr %43, i64 8
    %45 = getelementptr inbounds i8, ptr %44, i64 1
    %46 = load i1, ptr %45
    store i1 %46, ptr %7
    store i32 7, ptr %22
    %47 = getelementptr inbounds i8, ptr %22, i64 4
    store i32 8, ptr %47
    %48 = load [8 x i8], ptr %22
    store [8 x i8] %48, ptr %8
    %49 = add nsw i32 2, 3
    %50 = mul nsw i32 %49, 9
    %51 = getelementptr inbounds i8, ptr %8, i64 4
    %52 = load i32, ptr %51
    %53 = add nsw i32 %50, %52
    %54 = getelementptr inbounds i8, ptr %6, i64 12
    %55 = getelementptr inbounds i8, ptr %54, i64 8
    %56 = getelementptr inbounds i8, ptr %55, i64 2
    %57 = load i32, ptr %56
    %58 = add nsw i32 %53, %57
    store i32 %58, ptr %9
    %59 = load i32, ptr %9
    %60 = add nsw i32 1, %59
    %61 = sub nsw i32 6, %60
    store i32 %61, ptr %10
    %62 = load i1, ptr %7
    br i1 %62, label %63, label %64
63:
    store i32 2, ptr %12
    store i32 2, ptr %23
    br label %67
64:
    %65 = load i32, ptr %9
    %66 = icmp eq i32 %65, 9
    br i1 %66, label %67, label %68
67:
    store i32 3, ptr %13
    store i32 99, ptr %23
    br label %69
68:
    store i32 4, ptr %14
    store i32 7, ptr %23
    br label %69
69:
    %70 = load i32, ptr %23
    store i32 %70, ptr %11
    %71 = load i32, ptr %9
    %72 = load i32, ptr %10
    %73 = add nsw i32 %71, %72
    %74 = load i32, ptr %11
    %75 = add nsw i32 %73, %74
    store i32 %75, ptr %15
    store i32 2, ptr %15
    %76 = icmp eq i1 1, 1
    br i1 %76, label %77, label %78
77:
    store i32 1, ptr %24
    br label %80
78:
    %79 = load i32, ptr %15
    store i32 %79, ptr %24
    br label %80
80:
    %81 = load i32, ptr %24
    store i32 %81, ptr %16
    %82 = load i32, ptr %15
    %83 = add nsw i32 928, %82
    store i32 %83, ptr %15
    store i32 0, ptr %17
    %84 = load i32, ptr %17
    %85 = icmp eq i32 %84, 10
    br i1 %85, label %86, label %89
86:
    %87 = load i32, ptr %17
    %88 = add nsw i32 %87, 1
    store i32 %88, ptr %17
    br label %89
89:
    ret i32 0
}

