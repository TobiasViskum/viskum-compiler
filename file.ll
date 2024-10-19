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
    %27 = call i32 (i32) @fib55(i32 noundef 45)
    store i32 0, ptr %3
    store i32 2, ptr %4
    store i32 2, ptr %5
    %28 = load i32, ptr %4
    %29 = load i32, ptr %3
    %30 = add nsw i32 %28, %29
    store i32 %30, ptr %2
    br i1 1, label %31, label %32
31:
    store i32 1, ptr %19
    br label %33
32:
    store i32 0, ptr %19
    br label %33
33:
    %34 = load i32, ptr %19
    store i32 %34, ptr %6
    store i32 3, ptr %20
    %35 = getelementptr inbounds i8, ptr %20, i64 4
    store i32 4, ptr %35
    %36 = load i32, ptr %6
    %37 = call ptr () @returnFunction43()
    %38 = call i32 (i32, i32) %37(i32 noundef 2, i32 noundef 3)
    %39 = add nsw i32 %36, %38
    %40 = add nsw i32 5, %39
    %41 = getelementptr inbounds i8, ptr %20, i64 8
    store i32 %40, ptr %41
    store i32 9, ptr %21
    %42 = getelementptr inbounds i8, ptr %21, i64 4
    store i32 4, ptr %42
    store i1 1, ptr %22
    %43 = getelementptr inbounds i8, ptr %22, i64 1
    store i1 0, ptr %43
    %44 = getelementptr inbounds i8, ptr %22, i64 2
    store i32 8, ptr %44
    %45 = load [6 x i8], ptr %22
    %46 = getelementptr inbounds i8, ptr %21, i64 8
    store [6 x i8] %45, ptr %46
    %47 = load [14 x i8], ptr %21
    %48 = getelementptr inbounds i8, ptr %20, i64 12
    store [14 x i8] %47, ptr %48
    %49 = getelementptr inbounds i8, ptr %20, i64 26
    store ptr @add26, ptr %49
    %50 = load [34 x i8], ptr %20
    store [34 x i8] %50, ptr %7
    %51 = getelementptr inbounds i8, ptr %7, i64 12
    %52 = getelementptr inbounds i8, ptr %51, i64 8
    %53 = getelementptr inbounds i8, ptr %52, i64 1
    %54 = load i1, ptr %53
    store i1 %54, ptr %8
    store i32 7, ptr %23
    %55 = getelementptr inbounds i8, ptr %23, i64 4
    store i32 8, ptr %55
    %56 = load [8 x i8], ptr %23
    store [8 x i8] %56, ptr %9
    %57 = add nsw i32 2, 3
    %58 = mul nsw i32 %57, 9
    %59 = getelementptr inbounds i8, ptr %9, i64 4
    %60 = load i32, ptr %59
    %61 = add nsw i32 %58, %60
    %62 = getelementptr inbounds i8, ptr %7, i64 12
    %63 = getelementptr inbounds i8, ptr %62, i64 8
    %64 = getelementptr inbounds i8, ptr %63, i64 2
    %65 = load i32, ptr %64
    %66 = add nsw i32 %61, %65
    %67 = getelementptr inbounds i8, ptr %7, i64 26
    %68 = load ptr, ptr %67
    %69 = call i32 (i32, i32) %68(i32 noundef 2, i32 noundef 3)
    %70 = add nsw i32 %66, %69
    %71 = load [34 x i8], ptr %7
    %72 = call i32 ([34 x i8]) @sum9([34 x i8] noundef %71)
    %73 = add nsw i32 %70, %72
    store i32 %73, ptr %10
    %74 = load i32, ptr %10
    %75 = add nsw i32 1, %74
    %76 = sub nsw i32 6, %75
    store i32 %76, ptr %11
    call void () @iReturnVoid24()
    %77 = load i1, ptr %8
    br i1 %77, label %78, label %79
78:
    store i32 2, ptr %13
    store i32 2, ptr %24
    br label %82
79:
    %80 = load i32, ptr %10
    %81 = icmp eq i32 %80, 9
    br i1 %81, label %82, label %83
82:
    store i32 3, ptr %14
    store i32 99, ptr %24
    br label %84
83:
    store i32 4, ptr %15
    store i32 7, ptr %24
    br label %84
84:
    %85 = load i32, ptr %24
    store i32 %85, ptr %12
    %86 = load i32, ptr %10
    %87 = load i32, ptr %11
    %88 = add nsw i32 %86, %87
    %89 = load i32, ptr %12
    %90 = add nsw i32 %88, %89
    store i32 %90, ptr %16
    store i32 2, ptr %16
    %91 = icmp eq i1 1, 1
    br i1 %91, label %92, label %93
92:
    store i32 1, ptr %25
    br label %95
93:
    %94 = load i32, ptr %16
    store i32 %94, ptr %25
    br label %95
95:
    %96 = load i32, ptr %25
    store i32 %96, ptr %17
    %97 = load i32, ptr %16
    %98 = add nsw i32 928, %97
    store i32 %98, ptr %16
    store i32 0, ptr %18
    %99 = load i32, ptr %18
    %100 = icmp eq i32 %99, 10
    br i1 %100, label %101, label %104
101:
    %102 = load i32, ptr %18
    %103 = add nsw i32 %102, 1
    store i32 %103, ptr %18
    br label %104
104:
    br label %105
105:
    %106 = load i32, ptr %18
    %107 = add nsw i32 %106, 1
    store i32 %107, ptr %18
    %108 = load i32, ptr %18
    %109 = sub nsw i32 %108, 1
    %110 = icmp eq i32 %109, 100
    br i1 %110, label %111, label %113
111:
    br label %114
112:
    br label %113
113:
    br label %105
114:
    ret i32 0
}

define i32 @sum9([34 x i8] noundef %1) {
    %3 = alloca [34 x i8], align 1
    br label %4
4:
    store [34 x i8] %1, ptr %3
    %5 = load i32, ptr %3
    %6 = getelementptr inbounds i8, ptr %3, i64 4
    %7 = load i32, ptr %6
    %8 = add nsw i32 %5, %7
    %9 = getelementptr inbounds i8, ptr %3, i64 8
    %10 = load i32, ptr %9
    %11 = add nsw i32 %8, %10
    ret i32 %11
    unreachable
}

define void @iReturnVoid24() {
    br label %2
2:
    ret void
}

define i32 @doAddition29(i32 noundef %1, i32 noundef %2) {
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

define i32 @add26(i32 noundef %1, i32 noundef %2) {
    %4 = alloca [4 x i8], align 4
    %5 = alloca [4 x i8], align 4
    br label %6
6:
    store i32 %1, ptr %4
    store i32 %2, ptr %5
    %7 = load i32, ptr %4
    %8 = load i32, ptr %5
    %9 = call i32 (i32, i32) @doAddition29(i32 noundef %7, i32 noundef %8)
    ret i32 %9
    unreachable
}

define i32 @addTwo44(i32 noundef %1, i32 noundef %2) {
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

define ptr @returnFunction43() {
    br label %2
2:
    ret ptr @addTwo44
    unreachable
}

define i32 @fib55(i32 noundef %1) {
    %3 = alloca [4 x i8], align 4
    br label %4
4:
    store i32 %1, ptr %3
    %5 = load i32, ptr %3
    %6 = icmp sle i32 %5, 1
    br i1 %6, label %7, label %11
7:
    %8 = load i32, ptr %3
    %9 = add nsw i32 %8, 0
    ret i32 %9
    br label %11
11:
    %12 = load i32, ptr %3
    %13 = sub nsw i32 %12, 2
    %14 = call i32 (i32) @fib55(i32 noundef %13)
    %15 = load i32, ptr %3
    %16 = sub nsw i32 %15, 1
    %17 = call i32 (i32) @fib55(i32 noundef %16)
    %18 = add nsw i32 %14, %17
    ret i32 %18
    unreachable
}

