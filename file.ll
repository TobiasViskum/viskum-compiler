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
    call void (ptr, i32) @push33(ptr noundef %1, i32 noundef 0)
    call void (ptr, i32) @push33(ptr noundef %1, i32 noundef 1)
    call void (ptr, i32) @push33(ptr noundef %1, i32 noundef 0)
    call void (ptr, i32) @push33(ptr noundef %1, i32 noundef 2)
    call void (ptr, i32) @push33(ptr noundef %1, i32 noundef 0)
    %45 = getelementptr inbounds i8, ptr %1, i64 8
    %46 = load ptr, ptr %45
    %47 = getelementptr inbounds ptr, ptr %46, i32 4
    %48 = load i32, ptr %47
    call void (i32) @exit(i32 noundef %48)
    store i8 0, ptr %2
    store i64 0, ptr %32
    store i64 1, ptr %33
    %49 = getelementptr inbounds i8, ptr %33, i64 8
    store i32 2, ptr %49
    %50 = getelementptr inbounds i8, ptr %33, i64 12
    store i8 0, ptr %50
    %51 = getelementptr inbounds i8, ptr %33, i64 13
    store i32 8, ptr %51
    %52 = load [20 x i8], ptr %33
    %53 = getelementptr inbounds i8, ptr %32, i64 8
    store i32 0, ptr %53
    %54 = getelementptr inbounds i8, ptr %32, i64 12
    store [20 x i8] %52, ptr %54
    %55 = load [32 x i8], ptr %32
    store [32 x i8] %55, ptr %3
    store i64 1, ptr %34
    %56 = getelementptr inbounds i8, ptr %34, i64 8
    store i32 4, ptr %56
    %57 = getelementptr inbounds i8, ptr %34, i64 12
    store i32 9, ptr %57
    %58 = load [32 x i8], ptr %34
    store [32 x i8] %58, ptr %4
    %59 = load i64, ptr %3
    %60 = icmp eq i64 %59, 0
    br i1 %60, label %65, label %77
61:
    %62 = getelementptr inbounds i8, ptr %3, i64 12
    %63 = load i64, ptr %62
    %64 = icmp eq i64 %63, 1
    br i1 %64, label %65, label %77
65:
    %66 = getelementptr inbounds i8, ptr %3, i64 8
    %67 = load i32, ptr %66
    store i32 %67, ptr %5
    %68 = getelementptr inbounds i8, ptr %62, i64 8
    %69 = load i32, ptr %68
    store i32 %69, ptr %6
    %70 = getelementptr inbounds i8, ptr %62, i64 12
    %71 = load i8, ptr %70
    store i8 %71, ptr %7
    %72 = getelementptr inbounds i8, ptr %62, i64 13
    %73 = load i32, ptr %72
    store i32 %73, ptr %8
    %74 = load i32, ptr %6
    %75 = load i32, ptr %8
    %76 = add nsw i32 %74, %75
    store i32 %76, ptr %9
    store i8 1, ptr %2
    br label %77
77:
    store i64 0, ptr %35
    %78 = getelementptr inbounds i8, ptr %35, i64 8
    store i32 2, ptr %78
    %79 = load [12 x i8], ptr %35
    store [12 x i8] %79, ptr %10
    %80 = load i64, ptr %10
    %81 = icmp eq i64 %80, 0
    br i1 %81, label %82, label %85
82:
    %83 = getelementptr inbounds i8, ptr %10, i64 8
    %84 = load i32, ptr %83
    store i32 %84, ptr %11
    store ptr %11, ptr %12
    store i8 1, ptr %2
    br label %85
85:
    %86 = call i32 (i32) @fib157(i32 noundef 30)
    store i32 0, ptr %14
    store i32 2, ptr %15
    store i32 2, ptr %16
    %87 = load i32, ptr %15
    %88 = load i32, ptr %14
    %89 = add nsw i32 %87, %88
    store i32 %89, ptr %13
    store i32 0, ptr %17
    br i1 1, label %90, label %91
90:
    store i32 1, ptr %36
    br label %92
91:
    store i32 0, ptr %36
    br label %92
92:
    %93 = load i32, ptr %36
    store i32 %93, ptr %18
    %94 = load i32, ptr %18
    %95 = sub nsw i32 %94, 1
    store i32 %95, ptr %19
    store i32 3, ptr %37
    %96 = getelementptr inbounds i8, ptr %37, i64 4
    store i32 4, ptr %96
    %97 = load i32, ptr %18
    %98 = call ptr () @returnFunction145()
    %99 = call i32 (i32, i32) %98(i32 noundef 2, i32 noundef 3)
    %100 = add nsw i32 %97, %99
    %101 = add nsw i32 5, %100
    %102 = getelementptr inbounds i8, ptr %37, i64 8
    store i32 %101, ptr %102
    store i32 9, ptr %38
    %103 = getelementptr inbounds i8, ptr %38, i64 4
    store i32 4, ptr %103
    store i8 1, ptr %39
    %104 = getelementptr inbounds i8, ptr %39, i64 1
    store i8 0, ptr %104
    %105 = getelementptr inbounds i8, ptr %39, i64 2
    store i32 8, ptr %105
    %106 = load [6 x i8], ptr %39
    %107 = getelementptr inbounds i8, ptr %38, i64 8
    store [6 x i8] %106, ptr %107
    %108 = load [14 x i8], ptr %38
    %109 = getelementptr inbounds i8, ptr %37, i64 12
    store [14 x i8] %108, ptr %109
    %110 = getelementptr inbounds i8, ptr %37, i64 26
    store ptr @add128, ptr %110
    %111 = load [34 x i8], ptr %37
    store [34 x i8] %111, ptr %20
    %112 = getelementptr inbounds i8, ptr %20, i64 12
    %113 = getelementptr inbounds i8, ptr %112, i64 8
    %114 = getelementptr inbounds i8, ptr %113, i64 1
    store ptr %114, ptr %21
    store i32 7, ptr %40
    %115 = getelementptr inbounds i8, ptr %40, i64 4
    store i32 8, ptr %115
    %116 = load [8 x i8], ptr %40
    store [8 x i8] %116, ptr %22
    %117 = add nsw i32 2, 3
    %118 = mul nsw i32 %117, 9
    %119 = getelementptr inbounds i8, ptr %22, i64 4
    %120 = load i32, ptr %119
    %121 = add nsw i32 %118, %120
    %122 = getelementptr inbounds i8, ptr %20, i64 12
    %123 = getelementptr inbounds i8, ptr %122, i64 8
    %124 = getelementptr inbounds i8, ptr %123, i64 2
    %125 = load i32, ptr %124
    %126 = add nsw i32 %121, %125
    %127 = getelementptr inbounds i8, ptr %20, i64 26
    %128 = load ptr, ptr %127
    %129 = call i32 (i32, i32) %128(i32 noundef 22, i32 noundef 33)
    %130 = add nsw i32 %126, %129
    %131 = load [34 x i8], ptr %20
    %132 = call i32 ([34 x i8]) @sum111([34 x i8] noundef %131)
    %133 = add nsw i32 %130, %132
    store i32 %133, ptr %23
    %134 = load i32, ptr %23
    %135 = add nsw i32 1, %134
    %136 = sub nsw i32 6, %135
    store i32 %136, ptr %24
    call void () @iReturnVoid126()
    %137 = load ptr, ptr %21
    %138 = load i8, ptr %137
    %139 = icmp sle i8 %138, 1
    br i1 %139, label %140, label %141
140:
    store i32 2, ptr %26
    store i32 2, ptr %41
    br label %144
141:
    %142 = load i32, ptr %23
    %143 = icmp sge i32 %142, 9
    br i1 %143, label %144, label %145
144:
    store i32 3, ptr %27
    store i32 99, ptr %41
    br label %146
145:
    store i32 4, ptr %28
    store i32 7, ptr %41
    br label %146
146:
    %147 = load i32, ptr %41
    store i32 %147, ptr %25
    %148 = getelementptr inbounds i8, ptr %20, i64 12
    %149 = getelementptr inbounds i8, ptr %148, i64 8
    %150 = getelementptr inbounds i8, ptr %149, i64 2
    store ptr %150, ptr %29
    store i32 2, ptr %29
    %151 = icmp ne i8 1, 1
    br i1 %151, label %152, label %153
152:
    store i32 1, ptr %42
    br label %156
153:
    %154 = load ptr, ptr %29
    %155 = load i32, ptr %154
    store i32 %155, ptr %42
    br label %156
156:
    %157 = load i32, ptr %42
    store i32 %157, ptr %30
    %158 = load ptr, ptr %29
    %159 = load i32, ptr %158
    %160 = add nsw i32 928, %159
    store i32 %160, ptr %29
    store i32 0, ptr %31
    %161 = load i32, ptr %31
    %162 = icmp ne i32 %161, 10
    br i1 %162, label %163, label %166
163:
    %164 = load i32, ptr %31
    %165 = add nsw i32 %164, 1
    store i32 %165, ptr %31
    br label %166
166:
    br label %167
167:
    %168 = load i32, ptr %31
    %169 = add nsw i32 %168, 1
    store i32 %169, ptr %31
    %170 = load i32, ptr %31
    %171 = sub nsw i32 %170, 1
    %172 = icmp eq i32 %171, 100
    br i1 %172, label %173, label %175
173:
    br label %176
174:
    br label %175
175:
    br label %167
176:
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
    %6 = alloca [4 x i8], align 4
    %7 = alloca [8 x i8], align 8
    br label %8
8:
    store ptr %0, ptr %3
    store i32 %1, ptr %4
    %9 = load ptr, ptr %3
    %10 = load i32, ptr %9
    %11 = load ptr, ptr %3
    %12 = getelementptr inbounds i8, ptr %11, i64 4
    %13 = load i32, ptr %12
    %14 = icmp eq i32 %10, %13
    br i1 %14, label %15, label %50
15:
    %16 = load ptr, ptr %3
    %17 = getelementptr inbounds i8, ptr %16, i64 4
    %18 = load ptr, ptr %3
    %19 = getelementptr inbounds i8, ptr %18, i64 4
    %20 = load i32, ptr %19
    %21 = icmp eq i32 %20, 0
    br i1 %21, label %22, label %23
22:
    store i32 2, ptr %6
    br label %28
23:
    %24 = load ptr, ptr %3
    %25 = getelementptr inbounds i8, ptr %24, i64 4
    %26 = load i32, ptr %25
    %27 = mul nsw i32 %26, 2
    store i32 %27, ptr %6
    br label %28
28:
    %29 = load i32, ptr %6
    store i32 %29, ptr %17
    %30 = load ptr, ptr %3
    %31 = getelementptr inbounds i8, ptr %30, i64 4
    %32 = load i32, ptr %31
    %33 = mul nsw i32 %32, 4
    store i32 %33, ptr %5
    %34 = load ptr, ptr %3
    %35 = getelementptr inbounds i8, ptr %34, i64 8
    %36 = load ptr, ptr %3
    %37 = load i32, ptr %36
    %38 = icmp eq i32 %37, 0
    br i1 %38, label %39, label %42
39:
    %40 = load i32, ptr %5
    %41 = call ptr (i32) @malloc(i32 noundef %40)
    store ptr %41, ptr %7
    br label %48
42:
    %43 = load ptr, ptr %3
    %44 = getelementptr inbounds i8, ptr %43, i64 8
    %45 = load ptr, ptr %44
    %46 = load i32, ptr %5
    %47 = call ptr (ptr, i32) @realloc(ptr noundef %45, i32 noundef %46)
    store ptr %47, ptr %7
    br label %48
48:
    %49 = load ptr, ptr %7
    store ptr %49, ptr %35
    br label %50
50:
    %51 = load ptr, ptr %3
    %52 = getelementptr inbounds i8, ptr %51, i64 8
    %53 = load ptr, ptr %52
    %54 = load ptr, ptr %3
    %55 = load ptr, ptr %3
    %56 = load i32, ptr %55
    %57 = add nsw i32 %56, 1
    store i32 %57, ptr %54
    %58 = load ptr, ptr %3
    %59 = load i32, ptr %58
    %60 = sub nsw i32 %59, 1
    %61 = getelementptr inbounds ptr, ptr %53, i32 %60
    %62 = load i32, ptr %4
    store i32 %62, ptr %61
    ret void
}

define i32 @sum111([34 x i8] noundef %0) {
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

define void @iReturnVoid126() {
    br label %1
1:
    ret void
}

define i32 @doAddition131(i32 noundef %0, i32 noundef %1) {
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

define i32 @add128(i32 noundef %0, i32 noundef %1) {
    %3 = alloca [4 x i8], align 4
    %4 = alloca [4 x i8], align 4
    br label %5
5:
    store i32 %0, ptr %3
    store i32 %1, ptr %4
    %6 = load i32, ptr %3
    %7 = load i32, ptr %4
    %8 = call i32 (i32, i32) @doAddition131(i32 noundef %6, i32 noundef %7)
    ret i32 %8
    unreachable
}

define i32 @addTwo146(i32 noundef %0, i32 noundef %1) {
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

define ptr @returnFunction145() {
    br label %1
1:
    ret ptr @addTwo146
    unreachable
}

define i32 @fib157(i32 noundef %0) {
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
    %13 = call i32 (i32) @fib157(i32 noundef %12)
    %14 = load i32, ptr %2
    %15 = sub nsw i32 %14, 1
    %16 = call i32 (i32) @fib157(i32 noundef %15)
    %17 = add nsw i32 %13, %16
    ret i32 %17
    unreachable
}

