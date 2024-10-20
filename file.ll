define i32 @main() {
    %1 = alloca [12 x i8], align 4
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
    %19 = alloca [12 x i8], align 4
    %20 = alloca [4 x i8], align 4
    %21 = alloca [34 x i8], align 1
    %22 = alloca [14 x i8], align 1
    %23 = alloca [6 x i8], align 1
    %24 = alloca [8 x i8], align 4
    %25 = alloca [4 x i8], align 4
    %26 = alloca [4 x i8], align 4
    br label %27
27:
    store i64 0, ptr %19
    %28 = getelementptr inbounds i8, ptr %19, i64 8
    store i32 2, ptr %28
    %29 = load [12 x i8], ptr %19
    store [12 x i8] %29, ptr %1
    %30 = call i32 (i32) @fib55(i32 noundef 45)
    store i32 0, ptr %3
    store i32 2, ptr %4
    store i32 2, ptr %5
    %31 = load i32, ptr %4
    %32 = load i32, ptr %3
    %33 = add nsw i32 %31, %32
    store i32 %33, ptr %2
    br i1 1, label %34, label %35
34:
    store i32 1, ptr %20
    br label %36
35:
    store i32 0, ptr %20
    br label %36
36:
    %37 = load i32, ptr %20
    store i32 %37, ptr %6
    store i32 3, ptr %21
    %38 = getelementptr inbounds i8, ptr %21, i64 4
    store i32 4, ptr %38
    %39 = load i32, ptr %6
    %40 = call ptr () @returnFunction43()
    %41 = call i32 (i32, i32) %40(i32 noundef 2, i32 noundef 3)
    %42 = add nsw i32 %39, %41
    %43 = add nsw i32 5, %42
    %44 = getelementptr inbounds i8, ptr %21, i64 8
    store i32 %43, ptr %44
    store i32 9, ptr %22
    %45 = getelementptr inbounds i8, ptr %22, i64 4
    store i32 4, ptr %45
    store i8 1, ptr %23
    %46 = getelementptr inbounds i8, ptr %23, i64 1
    store i8 0, ptr %46
    %47 = getelementptr inbounds i8, ptr %23, i64 2
    store i32 8, ptr %47
    %48 = load [6 x i8], ptr %23
    %49 = getelementptr inbounds i8, ptr %22, i64 8
    store [6 x i8] %48, ptr %49
    %50 = load [14 x i8], ptr %22
    %51 = getelementptr inbounds i8, ptr %21, i64 12
    store [14 x i8] %50, ptr %51
    %52 = getelementptr inbounds i8, ptr %21, i64 26
    store ptr @add26, ptr %52
    %53 = load [34 x i8], ptr %21
    store [34 x i8] %53, ptr %7
    %54 = getelementptr inbounds i8, ptr %7, i64 12
    %55 = getelementptr inbounds i8, ptr %54, i64 8
    %56 = getelementptr inbounds i8, ptr %55, i64 1
    %57 = load i8, ptr %56
    store i8 %57, ptr %8
    store i32 7, ptr %24
    %58 = getelementptr inbounds i8, ptr %24, i64 4
    store i32 8, ptr %58
    %59 = load [8 x i8], ptr %24
    store [8 x i8] %59, ptr %9
    %60 = add nsw i32 2, 3
    %61 = mul nsw i32 %60, 9
    %62 = getelementptr inbounds i8, ptr %9, i64 4
    %63 = load i32, ptr %62
    %64 = add nsw i32 %61, %63
    %65 = getelementptr inbounds i8, ptr %7, i64 12
    %66 = getelementptr inbounds i8, ptr %65, i64 8
    %67 = getelementptr inbounds i8, ptr %66, i64 2
    %68 = load i32, ptr %67
    %69 = add nsw i32 %64, %68
    %70 = getelementptr inbounds i8, ptr %7, i64 26
    %71 = load ptr, ptr %70
    %72 = call i32 (i32, i32) %71(i32 noundef 2, i32 noundef 3)
    %73 = add nsw i32 %69, %72
    %74 = load [34 x i8], ptr %7
    %75 = call i32 ([34 x i8]) @sum9([34 x i8] noundef %74)
    %76 = add nsw i32 %73, %75
    store i32 %76, ptr %10
    %77 = load i32, ptr %10
    %78 = add nsw i32 1, %77
    %79 = sub nsw i32 6, %78
    store i32 %79, ptr %11
    call void () @iReturnVoid24()
    %80 = load i8, ptr %8
    br i1 %80, label %81, label %82
81:
    store i32 2, ptr %13
    store i32 2, ptr %25
    br label %85
82:
    %83 = load i32, ptr %10
    %84 = icmp eq i32 %83, 9
    br i1 %84, label %85, label %86
85:
    store i32 3, ptr %14
    store i32 99, ptr %25
    br label %87
86:
    store i32 4, ptr %15
    store i32 7, ptr %25
    br label %87
87:
    %88 = load i32, ptr %25
    store i32 %88, ptr %12
    %89 = load i32, ptr %10
    %90 = load i32, ptr %11
    %91 = add nsw i32 %89, %90
    %92 = load i32, ptr %12
    %93 = add nsw i32 %91, %92
    store i32 %93, ptr %16
    store i32 2, ptr %16
    %94 = icmp eq i8 1, 1
    br i1 %94, label %95, label %96
95:
    store i32 1, ptr %26
    br label %98
96:
    %97 = load i32, ptr %16
    store i32 %97, ptr %26
    br label %98
98:
    %99 = load i32, ptr %26
    store i32 %99, ptr %17
    %100 = load i32, ptr %16
    %101 = add nsw i32 928, %100
    store i32 %101, ptr %16
    store i32 0, ptr %18
    %102 = load i32, ptr %18
    %103 = icmp eq i32 %102, 10
    br i1 %103, label %104, label %107
104:
    %105 = load i32, ptr %18
    %106 = add nsw i32 %105, 1
    store i32 %106, ptr %18
    br label %107
107:
    br label %108
108:
    %109 = load i32, ptr %18
    %110 = add nsw i32 %109, 1
    store i32 %110, ptr %18
    %111 = load i32, ptr %18
    %112 = sub nsw i32 %111, 1
    %113 = icmp eq i32 %112, 100
    br i1 %113, label %114, label %116
114:
    br label %117
115:
    br label %116
116:
    br label %108
117:
    ret i32 0
}

define i32 @sum9([34 x i8] noundef %0) {
    %2 = alloca [34 x i8], align 1
    br label %3
3:
    store [34 x i8] %0, ptr %2
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

define void @iReturnVoid24() {
    br label %1
1:
    ret void
}

define i32 @doAddition29(i32 noundef %0, i32 noundef %1) {
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

define i32 @add26(i32 noundef %0, i32 noundef %1) {
    %3 = alloca [4 x i8], align 4
    %4 = alloca [4 x i8], align 4
    br label %5
5:
    store i32 %0, ptr %3
    store i32 %1, ptr %4
    %6 = load i32, ptr %3
    %7 = load i32, ptr %4
    %8 = call i32 (i32, i32) @doAddition29(i32 noundef %6, i32 noundef %7)
    ret i32 %8
    unreachable
}

define i32 @addTwo44(i32 noundef %0, i32 noundef %1) {
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

define ptr @returnFunction43() {
    br label %1
1:
    ret ptr @addTwo44
    unreachable
}

define i32 @fib55(i32 noundef %0) {
    %2 = alloca [4 x i8], align 4
    br label %3
3:
    store i32 %0, ptr %2
    %4 = load i32, ptr %2
    %5 = icmp sle i32 %4, 1
    br i1 %5, label %6, label %10
6:
    %7 = load i32, ptr %2
    %8 = add nsw i32 %7, 0
    ret i32 %8
    br label %10
10:
    %11 = load i32, ptr %2
    %12 = sub nsw i32 %11, 2
    %13 = call i32 (i32) @fib55(i32 noundef %12)
    %14 = load i32, ptr %2
    %15 = sub nsw i32 %14, 1
    %16 = call i32 (i32) @fib55(i32 noundef %15)
    %17 = add nsw i32 %13, %16
    ret i32 %17
    unreachable
}

