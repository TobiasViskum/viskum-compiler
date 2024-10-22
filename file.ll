define i32 @main() {
    %1 = alloca [1 x i8], align 1
    %2 = alloca [32 x i8], align 4
    %3 = alloca [32 x i8], align 4
    %4 = alloca [4 x i8], align 4
    %5 = alloca [4 x i8], align 4
    %6 = alloca [1 x i8], align 1
    %7 = alloca [4 x i8], align 4
    %8 = alloca [4 x i8], align 4
    %9 = alloca [12 x i8], align 4
    %10 = alloca [4 x i8], align 4
    %11 = alloca [8 x i8], align 8
    %12 = alloca [4 x i8], align 4
    %13 = alloca [4 x i8], align 4
    %14 = alloca [4 x i8], align 4
    %15 = alloca [4 x i8], align 4
    %16 = alloca [4 x i8], align 4
    %17 = alloca [34 x i8], align 1
    %18 = alloca [1 x i8], align 1
    %19 = alloca [8 x i8], align 4
    %20 = alloca [4 x i8], align 4
    %21 = alloca [4 x i8], align 4
    %22 = alloca [4 x i8], align 4
    %23 = alloca [4 x i8], align 4
    %24 = alloca [4 x i8], align 4
    %25 = alloca [4 x i8], align 4
    %26 = alloca [4 x i8], align 4
    %27 = alloca [4 x i8], align 4
    %28 = alloca [4 x i8], align 4
    %29 = alloca [32 x i8], align 4
    %30 = alloca [20 x i8], align 4
    %31 = alloca [32 x i8], align 4
    %32 = alloca [12 x i8], align 4
    %33 = alloca [4 x i8], align 4
    %34 = alloca [34 x i8], align 1
    %35 = alloca [14 x i8], align 1
    %36 = alloca [6 x i8], align 1
    %37 = alloca [8 x i8], align 4
    %38 = alloca [4 x i8], align 4
    %39 = alloca [4 x i8], align 4
    br label %40
40:
    store i8 0, ptr %1
    store i64 0, ptr %29
    store i64 1, ptr %30
    %41 = getelementptr inbounds i8, ptr %30, i64 8
    store i32 2, ptr %41
    %42 = getelementptr inbounds i8, ptr %30, i64 12
    store i8 0, ptr %42
    %43 = getelementptr inbounds i8, ptr %30, i64 13
    store i32 8, ptr %43
    %44 = load [20 x i8], ptr %30
    %45 = getelementptr inbounds i8, ptr %29, i64 8
    store i32 0, ptr %45
    %46 = getelementptr inbounds i8, ptr %29, i64 12
    store [20 x i8] %44, ptr %46
    %47 = load [32 x i8], ptr %29
    store [32 x i8] %47, ptr %2
    store i64 1, ptr %31
    %48 = getelementptr inbounds i8, ptr %31, i64 8
    store i32 4, ptr %48
    %49 = getelementptr inbounds i8, ptr %31, i64 12
    store i32 9, ptr %49
    %50 = load [32 x i8], ptr %31
    store [32 x i8] %50, ptr %3
    %51 = load i64, ptr %2
    %52 = icmp eq i64 %51, 0
    br i1 %52, label %57, label %69
53:
    %54 = getelementptr inbounds i8, ptr %2, i64 12
    %55 = load i64, ptr %54
    %56 = icmp eq i64 %55, 1
    br i1 %56, label %57, label %69
57:
    %58 = getelementptr inbounds i8, ptr %2, i64 8
    %59 = load i32, ptr %58
    store i32 %59, ptr %4
    %60 = getelementptr inbounds i8, ptr %54, i64 8
    %61 = load i32, ptr %60
    store i32 %61, ptr %5
    %62 = getelementptr inbounds i8, ptr %54, i64 12
    %63 = load i8, ptr %62
    store i8 %63, ptr %6
    %64 = getelementptr inbounds i8, ptr %54, i64 13
    %65 = load i32, ptr %64
    store i32 %65, ptr %7
    %66 = load i32, ptr %5
    %67 = load i32, ptr %7
    %68 = add nsw i32 %66, %67
    store i32 %68, ptr %8
    store i8 1, ptr %1
    br label %69
69:
    store i64 0, ptr %32
    %70 = getelementptr inbounds i8, ptr %32, i64 8
    store i32 2, ptr %70
    %71 = load [12 x i8], ptr %32
    store [12 x i8] %71, ptr %9
    %72 = load i64, ptr %9
    %73 = icmp eq i64 %72, 0
    br i1 %73, label %74, label %78
74:
    %75 = getelementptr inbounds i8, ptr %9, i64 8
    %76 = load i32, ptr %75
    store i32 %76, ptr %10
    %77 = load i32, ptr %10
    store i32 %77, ptr %11
    store i8 1, ptr %1
    br label %78
78:
    %79 = call i32 (i32) @fib55(i32 noundef 45)
    store i32 0, ptr %13
    store i32 2, ptr %14
    store i32 2, ptr %15
    %80 = load i32, ptr %14
    %81 = load i32, ptr %13
    %82 = add nsw i32 %80, %81
    store i32 %82, ptr %12
    br i1 1, label %83, label %84
83:
    store i32 1, ptr %33
    br label %85
84:
    store i32 0, ptr %33
    br label %85
85:
    %86 = load i32, ptr %33
    store i32 %86, ptr %16
    store i32 3, ptr %34
    %87 = getelementptr inbounds i8, ptr %34, i64 4
    store i32 4, ptr %87
    %88 = load i32, ptr %16
    %89 = call ptr () @returnFunction43()
    %90 = call i32 (i32, i32) %89(i32 noundef 2, i32 noundef 3)
    %91 = add nsw i32 %88, %90
    %92 = add nsw i32 5, %91
    %93 = getelementptr inbounds i8, ptr %34, i64 8
    store i32 %92, ptr %93
    store i32 9, ptr %35
    %94 = getelementptr inbounds i8, ptr %35, i64 4
    store i32 4, ptr %94
    store i8 1, ptr %36
    %95 = getelementptr inbounds i8, ptr %36, i64 1
    store i8 0, ptr %95
    %96 = getelementptr inbounds i8, ptr %36, i64 2
    store i32 8, ptr %96
    %97 = load [6 x i8], ptr %36
    %98 = getelementptr inbounds i8, ptr %35, i64 8
    store [6 x i8] %97, ptr %98
    %99 = load [14 x i8], ptr %35
    %100 = getelementptr inbounds i8, ptr %34, i64 12
    store [14 x i8] %99, ptr %100
    %101 = getelementptr inbounds i8, ptr %34, i64 26
    store ptr @add26, ptr %101
    %102 = load [34 x i8], ptr %34
    store [34 x i8] %102, ptr %17
    %103 = getelementptr inbounds i8, ptr %17, i64 12
    %104 = getelementptr inbounds i8, ptr %103, i64 8
    %105 = getelementptr inbounds i8, ptr %104, i64 1
    %106 = load i8, ptr %105
    store i8 %106, ptr %18
    store i32 7, ptr %37
    %107 = getelementptr inbounds i8, ptr %37, i64 4
    store i32 8, ptr %107
    %108 = load [8 x i8], ptr %37
    store [8 x i8] %108, ptr %19
    %109 = add nsw i32 2, 3
    %110 = mul nsw i32 %109, 9
    %111 = getelementptr inbounds i8, ptr %19, i64 4
    %112 = load i32, ptr %111
    %113 = add nsw i32 %110, %112
    %114 = getelementptr inbounds i8, ptr %17, i64 12
    %115 = getelementptr inbounds i8, ptr %114, i64 8
    %116 = getelementptr inbounds i8, ptr %115, i64 2
    %117 = load i32, ptr %116
    %118 = add nsw i32 %113, %117
    %119 = getelementptr inbounds i8, ptr %17, i64 26
    %120 = load ptr, ptr %119
    %121 = call i32 (i32, i32) %120(i32 noundef 2, i32 noundef 3)
    %122 = add nsw i32 %118, %121
    %123 = load [34 x i8], ptr %17
    %124 = call i32 ([34 x i8]) @sum9([34 x i8] noundef %123)
    %125 = add nsw i32 %122, %124
    store i32 %125, ptr %20
    %126 = load i32, ptr %20
    %127 = add nsw i32 1, %126
    %128 = sub nsw i32 6, %127
    store i32 %128, ptr %21
    call void () @iReturnVoid24()
    %129 = load i8, ptr %18
    %130 = icmp eq i8 %129, 1
    br i1 %130, label %131, label %132
131:
    store i32 2, ptr %23
    store i32 2, ptr %38
    br label %135
132:
    %133 = load i32, ptr %20
    %134 = icmp eq i32 %133, 9
    br i1 %134, label %135, label %136
135:
    store i32 3, ptr %24
    store i32 99, ptr %38
    br label %137
136:
    store i32 4, ptr %25
    store i32 7, ptr %38
    br label %137
137:
    %138 = load i32, ptr %38
    store i32 %138, ptr %22
    %139 = load i32, ptr %20
    %140 = load i32, ptr %21
    %141 = add nsw i32 %139, %140
    %142 = load i32, ptr %22
    %143 = add nsw i32 %141, %142
    store i32 %143, ptr %26
    store i32 2, ptr %26
    %144 = icmp eq i8 1, 1
    br i1 %144, label %145, label %146
145:
    store i32 1, ptr %39
    br label %148
146:
    %147 = load i32, ptr %26
    store i32 %147, ptr %39
    br label %148
148:
    %149 = load i32, ptr %39
    store i32 %149, ptr %27
    %150 = load i32, ptr %26
    %151 = add nsw i32 928, %150
    store i32 %151, ptr %26
    store i32 0, ptr %28
    %152 = load i32, ptr %28
    %153 = icmp eq i32 %152, 10
    br i1 %153, label %154, label %157
154:
    %155 = load i32, ptr %28
    %156 = add nsw i32 %155, 1
    store i32 %156, ptr %28
    br label %157
157:
    br label %158
158:
    %159 = load i32, ptr %28
    %160 = add nsw i32 %159, 1
    store i32 %160, ptr %28
    %161 = load i32, ptr %28
    %162 = sub nsw i32 %161, 1
    %163 = icmp eq i32 %162, 100
    br i1 %163, label %164, label %166
164:
    br label %167
165:
    br label %166
166:
    br label %158
167:
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

