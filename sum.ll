; ModuleID = 'my_module'
source_filename = "my_module"

define i32 @sum(i32 noundef %0, i32 noundef %1) {
entry:
  %tmp = add i32 %0, %1
  ret i32 %tmp
}

define i32 @main() {
entry:
  %sum = call i32 @sum(i32 1, i32 2)
  ret i32 %sum
}

declare i32 @printf(ptr, i32, ...)
