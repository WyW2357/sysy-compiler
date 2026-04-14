; ModuleID = 'sysy_module'
source_filename = "sysy_module"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@global_count = global i32 3
@scale = global float 1.500000e+00
@max_value = constant i32 100
@default_bias = constant float 2.500000e-01
@offsets = constant [3 x i32] [i32 2, i32 4, i32 6]
@numbers = global [5 x i32] [i32 1, i32 2, i32 3, i32 4, i32 5]

declare i32 @getint()

declare i32 @getch()

declare float @getfloat()

declare i32 @getarray(ptr)

declare i32 @getfarray(ptr)

declare void @putint(i32)

declare void @putch(i32)

declare void @putfloat(float)

declare void @putarray(i32, ptr)

declare void @putfarray(i32, ptr)

declare void @starttime()

declare void @stoptime()

define i32 @add(i32 %lhs, i32 %rhs) {
entry:
  %lhs1 = alloca i32, align 4
  store i32 %lhs, ptr %lhs1, align 4
  %rhs2 = alloca i32, align 4
  store i32 %rhs, ptr %rhs2, align 4
  %t0 = load i32, ptr %lhs1, align 4
  %t1 = load i32, ptr %rhs2, align 4
  %iadd = add i32 %t0, %t1
  ret i32 %iadd
}

define i32 @sum_array(ptr %arr, i32 %n) {
entry:
  %n1 = alloca i32, align 4
  store i32 %n, ptr %n1, align 4
  %i = alloca i32, align 4
  store i32 0, ptr %i, align 4
  %total = alloca i32, align 4
  store i32 0, ptr %total, align 4
  br label %while_cond.0

while_cond.0:                                     ; preds = %while_body.1, %entry
  %t0 = load i32, ptr %i, align 4
  %t1 = load i32, ptr %n1, align 4
  %icmp = icmp slt i32 %t0, %t1
  %icmpi32 = zext i1 %icmp to i32
  %tobool = icmp ne i32 %icmpi32, 0
  %t3 = load i32, ptr %total, align 4
  br i1 %tobool, label %while_body.1, label %while_end.2

while_body.1:                                     ; preds = %while_cond.0
  %t4 = load i32, ptr %i, align 4
  %idxptr = getelementptr i32, ptr %arr, i32 %t4
  %t5 = load i32, ptr %idxptr, align 4
  %iadd = add i32 %t3, %t5
  store i32 %iadd, ptr %total, align 4
  %t7 = load i32, ptr %i, align 4
  %iadd2 = add i32 %t7, 1
  store i32 %iadd2, ptr %i, align 4
  br label %while_cond.0

while_end.2:                                      ; preds = %while_cond.0
  ret i32 %t3
}

define void @fill(ptr %arr, i32 %n) {
entry:
  %n1 = alloca i32, align 4
  store i32 %n, ptr %n1, align 4
  %i = alloca i32, align 4
  store i32 0, ptr %i, align 4
  br label %for_cond.0

for_cond.0:                                       ; preds = %for_update.2, %entry
  %t0 = load i32, ptr %i, align 4
  %t1 = load i32, ptr %n1, align 4
  %icmp = icmp slt i32 %t0, %t1
  %icmpi32 = zext i1 %icmp to i32
  %tobool = icmp ne i32 %icmpi32, 0
  br i1 %tobool, label %for_body.1, label %for_end.3

for_body.1:                                       ; preds = %for_cond.0
  %t3 = load i32, ptr %i, align 4
  %irem = srem i32 %t3, 2
  %icmp2 = icmp eq i32 %irem, 0
  %icmpi323 = zext i1 %icmp2 to i32
  %tobool4 = icmp ne i32 %icmpi323, 0
  %t6 = load i32, ptr %i, align 4
  br i1 %tobool4, label %if_then.4, label %if_else.5

if_then.4:                                        ; preds = %for_body.1
  %calltmp = call i32 @getint()
  br label %if_end.6

if_else.5:                                        ; preds = %for_body.1
  %isub = sub i32 %t6, 1
  %idxptr5 = getelementptr i32, ptr %arr, i32 %isub
  %t11 = load i32, ptr %idxptr5, align 4
  %calltmp6 = call i32 @add(i32 %t11, i32 1)
  br label %if_end.6

if_end.6:                                         ; preds = %if_else.5, %if_then.4
  %calltmp6.sink = phi i32 [ %calltmp6, %if_else.5 ], [ %calltmp, %if_then.4 ]
  %idxptr7 = getelementptr i32, ptr %arr, i32 %t6
  store i32 %calltmp6.sink, ptr %idxptr7, align 4
  %t13 = load i32, ptr %i, align 4
  %idxptr8 = getelementptr i32, ptr %arr, i32 %t13
  %t14 = load i32, ptr %idxptr8, align 4
  %icmp9 = icmp sgt i32 %t14, 100
  %icmpi3210 = zext i1 %icmp9 to i32
  %tobool11 = icmp ne i32 %icmpi3210, 0
  br i1 %tobool11, label %for_end.3, label %for_update.2

for_update.2:                                     ; preds = %if_end.6
  %t16 = load i32, ptr %i, align 4
  %iadd = add i32 %t16, 1
  store i32 %iadd, ptr %i, align 4
  br label %for_cond.0

for_end.3:                                        ; preds = %if_end.6, %for_cond.0
  ret void
}

define i32 @main() {
entry:
  %local = alloca [3 x i32], align 4
  %idxptr16 = bitcast ptr %local to ptr
  store i32 0, ptr %idxptr16, align 4
  %idxptr1 = getelementptr [3 x i32], ptr %local, i32 0, i32 1
  store i32 1, ptr %idxptr1, align 4
  %idxptr2 = getelementptr [3 x i32], ptr %local, i32 0, i32 2
  store i32 2, ptr %idxptr2, align 4
  %local_limit = alloca i32, align 4
  store i32 3, ptr %local_limit, align 4
  %adjust = alloca [3 x i32], align 4
  %idxptr317 = bitcast ptr %adjust to ptr
  store i32 1, ptr %idxptr317, align 4
  %idxptr4 = getelementptr [3 x i32], ptr %adjust, i32 0, i32 1
  store i32 0, ptr %idxptr4, align 4
  %idxptr5 = getelementptr [3 x i32], ptr %adjust, i32 0, i32 2
  store i32 -1, ptr %idxptr5, align 4
  %base = alloca i32, align 4
  store i32 4, ptr %base, align 4
  %i = alloca i32, align 4
  store i32 0, ptr %i, align 4
  %total = alloca i32, align 4
  store i32 0, ptr %total, align 4
  %arraydecay18 = bitcast ptr %local to ptr
  call void @fill(ptr %arraydecay18, i32 3)
  store i32 0, ptr %i, align 4
  br label %for_cond.0

for_cond.0:                                       ; preds = %for_update.2, %entry
  %t2 = load i32, ptr %i, align 4
  %icmp = icmp slt i32 %t2, 3
  %icmpi32 = zext i1 %icmp to i32
  %tobool = icmp ne i32 %icmpi32, 0
  br i1 %tobool, label %for_body.1, label %for_end.3

for_body.1:                                       ; preds = %for_cond.0
  %t4 = load i32, ptr %i, align 4
  %idxptr6 = getelementptr [3 x i32], ptr %local, i32 0, i32 %t4
  %t5 = load i32, ptr %idxptr6, align 4
  %icmp7 = icmp sgt i32 %t5, 10
  %icmpi328 = zext i1 %icmp7 to i32
  %tobool9 = icmp ne i32 %icmpi328, 0
  br i1 %tobool9, label %for_update.2, label %if_else.5

if_else.5:                                        ; preds = %for_body.1
  %t7 = load i32, ptr %total, align 4
  %t8 = load i32, ptr %i, align 4
  %idxptr10 = getelementptr [3 x i32], ptr %local, i32 0, i32 %t8
  %t9 = load i32, ptr %idxptr10, align 4
  %iadd = add i32 %t7, %t9
  %idxptr11 = getelementptr [3 x i32], ptr %adjust, i32 0, i32 %t8
  %t12 = load i32, ptr %idxptr11, align 4
  %iadd12 = add i32 %iadd, %t12
  %iadd13 = add i32 %iadd12, 4
  store i32 %iadd13, ptr %total, align 4
  br label %for_update.2

for_update.2:                                     ; preds = %for_body.1, %if_else.5
  %t15 = load i32, ptr %i, align 4
  %iadd14 = add i32 %t15, 1
  store i32 %iadd14, ptr %i, align 4
  br label %for_cond.0

for_end.3:                                        ; preds = %for_cond.0
  %t17 = load i32, ptr %total, align 4
  call void @putint(i32 %t17)
  %t18 = load float, ptr @scale, align 4
  %fadd = fadd float %t18, 2.500000e-01
  call void @putfloat(float %fadd)
  %calltmp = call i32 @sum_array(ptr @numbers, i32 3)
  %t21 = load i32, ptr @global_count, align 4
  %iadd15 = add i32 %calltmp, %t21
  ret i32 %iadd15
}

