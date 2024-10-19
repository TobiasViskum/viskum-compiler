define i32 @main() {
    %2 = alloca [4 x i8], align 4
    %3 = alloca [4 x i8], align 4
    %4 = alloca [4 x i8], align 4
    %5 = alloca [4 x i8], align 4
    %6 = alloca [4 x i8], align 4
    %7 = alloca [34 x i8], align 1
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
    %20 = alloca [34 x i8], align 1
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
    %35 = load i32, ptr %6
    %36 = call ptr () @returnFunction24()
    %37 = call i32 (i32, i32) %36(i32 noundef 2, i32 noundef 3)
    %38 = add nsw i32 %35, %37
    %39 = add nsw i32 5, %38
    %40 = getelementptr inbounds i8, ptr %20, i64 8
    store i32 %39, ptr %40
    store i32 9, ptr %21
    %41 = getelementptr inbounds i8, ptr %21, i64 4
    store i32 4, ptr %41
    store i1 1, ptr %22
    %42 = getelementptr inbounds i8, ptr %22, i64 1
    store i1 0, ptr %42
    %43 = getelementptr inbounds i8, ptr %22, i64 2
    store i32 8, ptr %43
    %44 = load [6 x i8], ptr %22
    %45 = getelementptr inbounds i8, ptr %21, i64 8
    store [6 x i8] %44, ptr %45
    %46 = load [14 x i8], ptr %21
    %47 = getelementptr inbounds i8, ptr %20, i64 12
    store [14 x i8] %46, ptr %47
    %48 = getelementptr inbounds i8, ptr %20, i64 26
    store ptr @addNoReturn16, ptr %48
    %49 = load [34 x i8], ptr %20
    store [34 x i8] %49, ptr %7
    %50 = getelementptr inbounds i8, ptr %7, i64 12
    %51 = getelementptr inbounds i8, ptr %50, i64 8
    %52 = getelementptr inbounds i8, ptr %51, i64 1
    %53 = load i1, ptr %52
    store i1 %53, ptr %8
    store i32 7, ptr %23
    %54 = getelementptr inbounds i8, ptr %23, i64 4
    store i32 8, ptr %54
    %55 = load [8 x i8], ptr %23
    store [8 x i8] %55, ptr %9
    %56 = add nsw i32 2, 3
    %57 = mul nsw i32 %56, 9
    %58 = getelementptr inbounds i8, ptr %9, i64 4
    %59 = load i32, ptr %58
    %60 = add nsw i32 %57, %59
    %61 = getelementptr inbounds i8, ptr %7, i64 12
    %62 = getelementptr inbounds i8, ptr %61, i64 8
    %63 = getelementptr inbounds i8, ptr %62, i64 2
    %64 = load i32, ptr %63
    %65 = add nsw i32 %60, %64
    %66 = getelementptr inbounds i8, ptr %7, i64 26
    %67 = load ptr, ptr %66
    %68 = call i32 (i32, i32) %67(i32 noundef 2, i32 noundef 3)
    %69 = add nsw i32 %65, %68
    store i32 %69, ptr %10
    %70 = load i32, ptr %10
    %71 = add nsw i32 1, %70
    %72 = sub nsw i32 6, %71
    store i32 %72, ptr %11
    %73 = load i1, ptr %8
    br i1 %73, label %74, label %75
74:
    store i32 2, ptr %13
    store i32 2, ptr %24
    br label %78
75:
    %76 = load i32, ptr %10
    %77 = icmp eq i32 %76, 9
    br i1 %77, label %78, label %79
78:
    store i32 3, ptr %14
    store i32 99, ptr %24
    br label %80
79:
    store i32 4, ptr %15
    store i32 7, ptr %24
    br label %80
80:
    %81 = load i32, ptr %24
    store i32 %81, ptr %12
    %82 = load i32, ptr %10
    %83 = load i32, ptr %11
    %84 = add nsw i32 %82, %83
    %85 = load i32, ptr %12
    %86 = add nsw i32 %84, %85
    store i32 %86, ptr %16
    store i32 2, ptr %16
    %87 = icmp eq i1 1, 1
    br i1 %87, label %88, label %89
88:
    store i32 1, ptr %25
    br label %91
89:
    %90 = load i32, ptr %16
    store i32 %90, ptr %25
    br label %91
91:
    %92 = load i32, ptr %25
    store i32 %92, ptr %17
    %93 = load i32, ptr %16
    %94 = add nsw i32 928, %93
    store i32 %94, ptr %16
    store i32 0, ptr %18
    %95 = load i32, ptr %18
    %96 = icmp eq i32 %95, 10
    br i1 %96, label %97, label %100
97:
    %98 = load i32, ptr %18
    %99 = add nsw i32 %98, 1
    store i32 %99, ptr %18
    br label %100
100:
    ret i32 0
}

define void @iReturnVoid9() {
    br label %2
2:
    ret void
}

define i32 @addNoReturn16(i32 noundef %1, i32 noundef %2) {
    %4 = alloca [4 x i8], align 4
    %5 = alloca [4 x i8], align 4
    br label %6
6:
    store i32 %1, ptr %4
    store i32 %2, ptr %5
    %7 = load i32, ptr %4
    %8 = load i32, ptr %5
    %9 = add nsw i32 %7, %8
    ret i32 %9
    unreachable
}

define ptr @returnFunction24() {
    br label %2
2:
    ret ptr @addNoReturn16
    unreachable
}

