declare ptr @realloc(ptr noundef, i32 noundef)
declare ptr @malloc(i32 noundef)
declare i32 @socket(i32 noundef, i32 noundef, i32 noundef)
declare void @exit(i32 noundef)

define i32 @main() {
    %1 = alloca [16 x i8], align 4
    %2 = alloca [1 x i8], align 1
    %3 = alloca [32 x i8], align 4
    %4 = alloca [32 x i8], align 4
    %5 = alloca [4 x i8], align 4
    %6 = alloca [4 x i8], align 4
    %7 = alloca [1 x i8], align 1
    %8 = alloca [4 x i8], align 4
    %9 = alloca [4 x i8], align 4
    %10 = alloca [12 x i8], align 4
    %11 = alloca [4 x i8], align 4
    %12 = alloca [8 x i8], align 8
    %13 = alloca [4 x i8], align 4
    %14 = alloca [4 x i8], align 4
    %15 = alloca [4 x i8], align 4
    %16 = alloca [4 x i8], align 4
    %17 = alloca [4 x i8], align 4
    %18 = alloca [4 x i8], align 4
    %19 = alloca [4 x i8], align 4
    %20 = alloca [34 x i8], align 1
    %21 = alloca [8 x i8], align 8
    %22 = alloca [8 x i8], align 4
    %23 = alloca [4 x i8], align 4
    %24 = alloca [4 x i8], align 4
    %25 = alloca [4 x i8], align 4
    %26 = alloca [4 x i8], align 4
    %27 = alloca [4 x i8], align 4
    %28 = alloca [4 x i8], align 4
    %29 = alloca [8 x i8], align 8
    %30 = alloca [4 x i8], align 4
    %31 = alloca [4 x i8], align 4
    %32 = alloca [32 x i8], align 4
    %33 = alloca [20 x i8], align 4
    %34 = alloca [32 x i8], align 4
    %35 = alloca [12 x i8], align 4
    %36 = alloca [4 x i8], align 4
    %37 = alloca [34 x i8], align 1
    %38 = alloca [14 x i8], align 1
    %39 = alloca [6 x i8], align 1
    %40 = alloca [8 x i8], align 4
    %41 = alloca [4 x i8], align 4
    %42 = alloca [4 x i8], align 4
    br label %43
43:
    %44 = call [16 x i8] () @newVec22()
    store [16 x i8] %44, ptr %1
    call void (ptr, i32) @push33(ptr noundef %1, i32 noundef 2)
    store i8 0, ptr %2
    store i64 0, ptr %32
    store i64 1, ptr %33
    %45 = getelementptr inbounds i8, ptr %33, i64 8
    store i32 2, ptr %45
    %46 = getelementptr inbounds i8, ptr %33, i64 12
    store i8 0, ptr %46
    %47 = getelementptr inbounds i8, ptr %33, i64 13
    store i32 8, ptr %47
    %48 = load [20 x i8], ptr %33
    %49 = getelementptr inbounds i8, ptr %32, i64 8
    store i32 0, ptr %49
    %50 = getelementptr inbounds i8, ptr %32, i64 12
    store [20 x i8] %48, ptr %50
    %51 = load [32 x i8], ptr %32
    store [32 x i8] %51, ptr %3
    store i64 1, ptr %34
    %52 = getelementptr inbounds i8, ptr %34, i64 8
    store i32 4, ptr %52
    %53 = getelementptr inbounds i8, ptr %34, i64 12
    store i32 9, ptr %53
    %54 = load [32 x i8], ptr %34
    store [32 x i8] %54, ptr %4
    %55 = load i64, ptr %3
    %56 = icmp eq i64 %55, 0
    br i1 %56, label %61, label %73
57:
    %58 = getelementptr inbounds i8, ptr %3, i64 12
    %59 = load i64, ptr %58
    %60 = icmp eq i64 %59, 1
    br i1 %60, label %61, label %73
61:
    %62 = getelementptr inbounds i8, ptr %3, i64 8
    %63 = load i32, ptr %62
    store i32 %63, ptr %5
    %64 = getelementptr inbounds i8, ptr %58, i64 8
    %65 = load i32, ptr %64
    store i32 %65, ptr %6
    %66 = getelementptr inbounds i8, ptr %58, i64 12
    %67 = load i8, ptr %66
    store i8 %67, ptr %7
    %68 = getelementptr inbounds i8, ptr %58, i64 13
    %69 = load i32, ptr %68
    store i32 %69, ptr %8
    %70 = load i32, ptr %6
    %71 = load i32, ptr %8
    %72 = add nsw i32 %70, %71
    store i32 %72, ptr %9
    store i8 1, ptr %2
    br label %73
73:
    store i64 0, ptr %35
    %74 = getelementptr inbounds i8, ptr %35, i64 8
    store i32 2, ptr %74
    %75 = load [12 x i8], ptr %35
    store [12 x i8] %75, ptr %10
    %76 = load i64, ptr %10
    %77 = icmp eq i64 %76, 0
    br i1 %77, label %78, label %81
78:
    %79 = getelementptr inbounds i8, ptr %10, i64 8
    %80 = load i32, ptr %79
    store i32 %80, ptr %11
    store ptr %11, ptr %12
    store i8 1, ptr %2
    br label %81
81:
    %82 = call i32 (i32) @fib131(i32 noundef 30)
    store i32 0, ptr %14
    store i32 2, ptr %15
    store i32 2, ptr %16
    %83 = load i32, ptr %15
    %84 = load i32, ptr %14
    %85 = add nsw i32 %83, %84
    store i32 %85, ptr %13
    store i32 0, ptr %17
    br i1 1, label %86, label %87
86:
    store i32 1, ptr %36
    br label %88
87:
    store i32 0, ptr %36
    br label %88
88:
    %89 = load i32, ptr %36
    store i32 %89, ptr %18
    %90 = load i32, ptr %18
    %91 = sub nsw i32 %90, 1
    store i32 %91, ptr %19
    store i32 3, ptr %37
    %92 = getelementptr inbounds i8, ptr %37, i64 4
    store i32 4, ptr %92
    %93 = load i32, ptr %18
    %94 = call ptr () @returnFunction119()
    %95 = call i32 (i32, i32) %94(i32 noundef 2, i32 noundef 3)
    %96 = add nsw i32 %93, %95
    %97 = add nsw i32 5, %96
    %98 = getelementptr inbounds i8, ptr %37, i64 8
    store i32 %97, ptr %98
    store i32 9, ptr %38
    %99 = getelementptr inbounds i8, ptr %38, i64 4
    store i32 4, ptr %99
    store i8 1, ptr %39
    %100 = getelementptr inbounds i8, ptr %39, i64 1
    store i8 0, ptr %100
    %101 = getelementptr inbounds i8, ptr %39, i64 2
    store i32 8, ptr %101
    %102 = load [6 x i8], ptr %39
    %103 = getelementptr inbounds i8, ptr %38, i64 8
    store [6 x i8] %102, ptr %103
    %104 = load [14 x i8], ptr %38
    %105 = getelementptr inbounds i8, ptr %37, i64 12
    store [14 x i8] %104, ptr %105
    %106 = getelementptr inbounds i8, ptr %37, i64 26
    store ptr @add102, ptr %106
    %107 = load [34 x i8], ptr %37
    store [34 x i8] %107, ptr %20
    %108 = getelementptr inbounds i8, ptr %20, i64 12
    %109 = getelementptr inbounds i8, ptr %108, i64 8
    %110 = getelementptr inbounds i8, ptr %109, i64 1
    store ptr %110, ptr %21
    store i32 7, ptr %40
    %111 = getelementptr inbounds i8, ptr %40, i64 4
    store i32 8, ptr %111
    %112 = load [8 x i8], ptr %40
    store [8 x i8] %112, ptr %22
    %113 = add nsw i32 2, 3
    %114 = mul nsw i32 %113, 9
    %115 = getelementptr inbounds i8, ptr %22, i64 4
    %116 = load i32, ptr %115
    %117 = add nsw i32 %114, %116
    %118 = getelementptr inbounds i8, ptr %20, i64 12
    %119 = getelementptr inbounds i8, ptr %118, i64 8
    %120 = getelementptr inbounds i8, ptr %119, i64 2
    %121 = load i32, ptr %120
    %122 = add nsw i32 %117, %121
    %123 = getelementptr inbounds i8, ptr %20, i64 26
    %124 = load ptr, ptr %123
    %125 = call i32 (i32, i32) %124(i32 noundef 22, i32 noundef 33)
    %126 = add nsw i32 %122, %125
    %127 = load [34 x i8], ptr %20
    %128 = call i32 ([34 x i8]) @sum85([34 x i8] noundef %127)
    %129 = add nsw i32 %126, %128
    store i32 %129, ptr %23
    %130 = load i32, ptr %23
    %131 = add nsw i32 1, %130
    %132 = sub nsw i32 6, %131
    store i32 %132, ptr %24
    call void () @iReturnVoid100()
    %133 = load ptr, ptr %21
    %134 = load i8, ptr %133
    %135 = icmp sle i8 %134, 1
    br i1 %135, label %136, label %137
136:
    store i32 2, ptr %26
    store i32 2, ptr %41
    br label %140
137:
    %138 = load i32, ptr %23
    %139 = icmp sge i32 %138, 9
    br i1 %139, label %140, label %141
140:
    store i32 3, ptr %27
    store i32 99, ptr %41
    br label %142
141:
    store i32 4, ptr %28
    store i32 7, ptr %41
    br label %142
142:
    %143 = load i32, ptr %41
    store i32 %143, ptr %25
    %144 = getelementptr inbounds i8, ptr %20, i64 12
    %145 = getelementptr inbounds i8, ptr %144, i64 8
    %146 = getelementptr inbounds i8, ptr %145, i64 2
    store ptr %146, ptr %29
    store i32 2, ptr %29
    %147 = icmp ne i8 1, 1
    br i1 %147, label %148, label %149
148:
    store i32 1, ptr %42
    br label %152
149:
    %150 = load ptr, ptr %29
    %151 = load i32, ptr %150
    store i32 %151, ptr %42
    br label %152
152:
    %153 = load i32, ptr %42
    store i32 %153, ptr %30
    %154 = load ptr, ptr %29
    %155 = load i32, ptr %154
    %156 = add nsw i32 928, %155
    store i32 %156, ptr %29
    store i32 0, ptr %31
    %157 = load i32, ptr %31
    %158 = icmp ne i32 %157, 10
    br i1 %158, label %159, label %162
159:
    %160 = load i32, ptr %31
    %161 = add nsw i32 %160, 1
    store i32 %161, ptr %31
    br label %162
162:
    br label %163
163:
    %164 = load i32, ptr %31
    %165 = add nsw i32 %164, 1
    store i32 %165, ptr %31
    %166 = load i32, ptr %31
    %167 = sub nsw i32 %166, 1
    %168 = icmp eq i32 %167, 100
    br i1 %168, label %169, label %171
169:
    br label %172
170:
    br label %171
171:
    br label %163
172:
    ret i32 0
}

define [16 x i8] @newVec22() {
    %1 = alloca [16 x i8], align 4
    br label %2
2:
    store i32 0, ptr %1
    %3 = getelementptr inbounds i8, ptr %1, i64 4
    store i32 0, ptr %3
    %4 = getelementptr inbounds i8, ptr %1, i64 8
    store ptr null, ptr %4
    %5 = load [16 x i8], ptr %1
    ret [16 x i8] %5
    unreachable
}

define void @push33(ptr noundef %0, i32 noundef %1) {
    %3 = alloca [8 x i8], align 8
    %4 = alloca [4 x i8], align 4
    %5 = alloca [4 x i8], align 4
    br label %6
6:
    store ptr %0, ptr %3
    store i32 %1, ptr %4
    %7 = load ptr, ptr %3
    %8 = load i32, ptr %7
    %9 = load ptr, ptr %3
    %10 = getelementptr inbounds i8, ptr %9, i64 4
    %11 = load i32, ptr %10
    %12 = icmp eq i32 %8, %11
    br i1 %12, label %13, label %35
13:
    %14 = load ptr, ptr %3
    %15 = getelementptr inbounds i8, ptr %14, i64 4
    %16 = load ptr, ptr %3
    %17 = getelementptr inbounds i8, ptr %16, i64 4
    %18 = load i32, ptr %17
    %19 = icmp eq i32 %18, 0
    br i1 %19, label %20, label %21
20:
    store i32 2, ptr %5
    br label %26
21:
    %22 = load ptr, ptr %3
    %23 = getelementptr inbounds i8, ptr %22, i64 4
    %24 = load i32, ptr %23
    %25 = mul nsw i32 %24, 2
    store i32 %25, ptr %5
    br label %26
26:
    %27 = load i32, ptr %5
    store i32 %27, ptr %15
    %28 = load ptr, ptr %3
    %29 = getelementptr inbounds i8, ptr %28, i64 8
    %30 = load ptr, ptr %3
    %31 = getelementptr inbounds i8, ptr %30, i64 4
    %32 = load i32, ptr %31
    %33 = mul nsw i32 %32, 4
    %34 = call ptr (i32) @malloc(i32 noundef %33)
    store ptr %34, ptr %29
    br label %35
35:
    %36 = load ptr, ptr %3
    %37 = getelementptr inbounds i8, ptr %36, i64 4
    %38 = load i32, ptr %37
    call void (i32) @exit(i32 noundef %38)
    ret void
}

define i32 @sum85([34 x i8] noundef %0) {
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

define void @iReturnVoid100() {
    br label %1
1:
    ret void
}

define i32 @doAddition105(i32 noundef %0, i32 noundef %1) {
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

define i32 @add102(i32 noundef %0, i32 noundef %1) {
    %3 = alloca [4 x i8], align 4
    %4 = alloca [4 x i8], align 4
    br label %5
5:
    store i32 %0, ptr %3
    store i32 %1, ptr %4
    %6 = load i32, ptr %3
    %7 = load i32, ptr %4
    %8 = call i32 (i32, i32) @doAddition105(i32 noundef %6, i32 noundef %7)
    ret i32 %8
    unreachable
}

define i32 @addTwo120(i32 noundef %0, i32 noundef %1) {
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

define ptr @returnFunction119() {
    br label %1
1:
    ret ptr @addTwo120
    unreachable
}

define i32 @fib131(i32 noundef %0) {
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
    %13 = call i32 (i32) @fib131(i32 noundef %12)
    %14 = load i32, ptr %2
    %15 = sub nsw i32 %14, 1
    %16 = call i32 (i32) @fib131(i32 noundef %15)
    %17 = add nsw i32 %13, %16
    ret i32 %17
    unreachable
}

define [12 x i8] @getEnum163() {
    %1 = alloca [12 x i8], align 4
    br label %2
2:
    store i64 0, ptr %1
    %3 = getelementptr inbounds i8, ptr %1, i64 8
    store i32 2, ptr %3
    %4 = load [12 x i8], ptr %1
    ret [4 x i8] %4
    unreachable
}

