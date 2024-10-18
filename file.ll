define i32 @main() {
    %2 = alloca [4 x i8], align 4
    %3 = alloca [4 x i8], align 4
    %4 = alloca [4 x i8], align 4
    %5 = alloca [4 x i8], align 4
    %6 = alloca [4 x i8], align 4
    %7 = alloca [26 x i8], align 1
    %8 = alloca [1 x i8], align 1
    %9 = alloca [8 x i8], align 4
    %10 = alloca [4 x i8], align 4
    %11 = alloca [4 x i8], align 4
    %12 = alloca [4 x i8], align 4
    %13 = alloca [4 x i8], align 4
    %14 = alloca [4 x i8], align 4
    %15 = alloca [4 x i8], align 4
    %16 = alloca [4 x i8], align 4
    %17 = alloca [4 x i8], align 4
    %18 = alloca [4 x i8], align 4
    %19 = alloca [4 x i8], align 4
    %20 = alloca [26 x i8], align 1
    %21 = alloca [14 x i8], align 1
    %22 = alloca [6 x i8], align 1
    %23 = alloca [8 x i8], align 4
    %24 = alloca [4 x i8], align 4
    %25 = alloca [4 x i8], align 4
    br label %26
26:
    store i32 0, ptr %3
    store i32 2, ptr %4
    store i32 2, ptr %5
    %27 = load i32, ptr %4
    %28 = load i32, ptr %3
    %29 = add nsw i32 %27, %28
    store i32 %29, ptr %2
    br i1 1, label %30, label %31
30:
    store i32 1, ptr %19
    br label %32
31:
    store i32 0, ptr %19
    br label %32
32:
    %33 = load i32, ptr %19
    store i32 %33, ptr %6
    store i32 3, ptr %20
    %34 = getelementptr inbounds i8, ptr %20, i64 4
    store i32 4, ptr %34
    %35 = getelementptr inbounds i8, ptr %20, i64 8
    store i32 5, ptr %35
    store i32 9, ptr %21
    %36 = getelementptr inbounds i8, ptr %21, i64 4
    store i32 4, ptr %36
    store i1 1, ptr %22
    %37 = getelementptr inbounds i8, ptr %22, i64 1
    store i1 0, ptr %37
    %38 = getelementptr inbounds i8, ptr %22, i64 2
    store i32 8, ptr %38
    %39 = load [6 x i8], ptr %22
    %40 = getelementptr inbounds i8, ptr %21, i64 8
    store [6 x i8] %39, ptr %40
    %41 = load [14 x i8], ptr %21
    %42 = getelementptr inbounds i8, ptr %20, i64 12
    store [14 x i8] %41, ptr %42
    %43 = load [26 x i8], ptr %20
    store [26 x i8] %43, ptr %7
    %44 = getelementptr inbounds i8, ptr %7, i64 12
    %45 = getelementptr inbounds i8, ptr %44, i64 8
    %46 = getelementptr inbounds i8, ptr %45, i64 1
    %47 = load i1, ptr %46
    store i1 %47, ptr %8
    store i32 7, ptr %23
    %48 = getelementptr inbounds i8, ptr %23, i64 4
    store i32 8, ptr %48
    %49 = load [8 x i8], ptr %23
    store [8 x i8] %49, ptr %9
    %50 = add nsw i32 2, 3
    %51 = mul nsw i32 %50, 9
    %52 = getelementptr inbounds i8, ptr %9, i64 4
    %53 = load i32, ptr %52
    %54 = add nsw i32 %51, %53
    %55 = getelementptr inbounds i8, ptr %7, i64 12
    %56 = getelementptr inbounds i8, ptr %55, i64 8
    %57 = getelementptr inbounds i8, ptr %56, i64 2
    %58 = load i32, ptr %57
    %59 = add nsw i32 %54, %58
    store i32 %59, ptr %10
    %60 = load i32, ptr %10
    %61 = add nsw i32 1, %60
    %62 = sub nsw i32 6, %61
    store i32 %62, ptr %11
    %63 = load i1, ptr %8
    br i1 %63, label %64, label %65
64:
    store i32 2, ptr %13
    store i32 2, ptr %24
    br label %68
65:
    %66 = load i32, ptr %10
    %67 = icmp eq i32 %66, 9
    br i1 %67, label %68, label %69
68:
    store i32 3, ptr %14
    store i32 99, ptr %24
    br label %70
69:
    store i32 4, ptr %15
    store i32 7, ptr %24
    br label %70
70:
    %71 = load i32, ptr %24
    store i32 %71, ptr %12
    %72 = load i32, ptr %10
    %73 = load i32, ptr %11
    %74 = add nsw i32 %72, %73
    %75 = load i32, ptr %12
    %76 = add nsw i32 %74, %75
    store i32 %76, ptr %16
    store i32 2, ptr %16
    %77 = icmp eq i1 1, 1
    br i1 %77, label %78, label %79
78:
    store i32 1, ptr %25
    br label %81
79:
    %80 = load i32, ptr %16
    store i32 %80, ptr %25
    br label %81
81:
    %82 = load i32, ptr %25
    store i32 %82, ptr %17
    %83 = load i32, ptr %16
    %84 = add nsw i32 928, %83
    store i32 %84, ptr %16
    store i32 0, ptr %18
    %85 = load i32, ptr %18
    %86 = icmp eq i32 %85, 10
    br i1 %86, label %87, label %90
87:
    %88 = load i32, ptr %18
    %89 = add nsw i32 %88, 1
    store i32 %89, ptr %18
    br label %90
90:
    ret i32 0
}

define void @addNoReturn8(i32 noundef %1, i32 noundef %2) {
    %4 = alloca [4 x i8], align 4
    %5 = alloca [4 x i8], align 4
    br label %6
6:
    store i32 %1, ptr %4
    store i32 %2, ptr %5
    %7 = load i32, ptr %4
    %8 = load i32, ptr %5
    %9 = add nsw i32 %7, %8
    ret void
}

