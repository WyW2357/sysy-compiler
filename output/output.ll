; ModuleID = 'sysy_module'
source_filename = "sysy_module"
target datalayout = "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128"
target triple = "aarch64-unknown-linux-gnu"

@g_scalar = global i32 7
@g_const = constant i32 11
@g_arr = global [5 x i32] [i32 1, i32 3, i32 5, i32 7, i32 9]
@g_f = global float 1.250000e+00
@g_fc = constant float 7.500000e-01
@msg_begin = global [13 x i32] [i32 66, i32 69, i32 71, i32 73, i32 78, i32 95, i32 84, i32 69, i32 83, i32 84, i32 83, i32 10, i32 0]
@msg_test = global [6 x i32] [i32 84, i32 69, i32 83, i32 84, i32 95, i32 0]
@msg_pass = global [8 x i32] [i32 58, i32 80, i32 65, i32 83, i32 83, i32 33, i32 10, i32 0]
@msg_fail = global [8 x i32] [i32 58, i32 70, i32 65, i32 73, i32 76, i32 33, i32 10, i32 0]

declare i32 @getint()

declare i32 @getch()

declare float @getfloat()

declare i32 @getarray(ptr)

declare i32 @getfarray(ptr)

declare void @putint(i32)

declare void @putch(i32)

declare void @putstr(ptr)

declare void @putfloat(float)

declare void @putarray(i32, ptr)

declare void @putfarray(i32, ptr)

declare void @starttime()

declare void @stoptime()

define i32 @add3(i32 %a, i32 %b, i32 %c) {
entry:
  %a1 = alloca i32, align 4
  store i32 %a, ptr %a1, align 4
  %b2 = alloca i32, align 4
  store i32 %b, ptr %b2, align 4
  %c3 = alloca i32, align 4
  store i32 %c, ptr %c3, align 4
  %t0 = load i32, ptr %a1, align 4
  %t1 = load i32, ptr %b2, align 4
  %iadd = add i32 %t0, %t1
  %t3 = load i32, ptr %c3, align 4
  %iadd4 = add i32 %iadd, %t3
  ret i32 %iadd4
}

define i32 @passthrough_int(i32 %x) {
entry:
  %x1 = alloca i32, align 4
  store i32 %x, ptr %x1, align 4
  %t0 = load i32, ptr %x1, align 4
  ret i32 %t0
}

define float @passthrough_float(float %x) {
entry:
  %x1 = alloca float, align 4
  store float %x, ptr %x1, align 4
  %t0 = load float, ptr %x1, align 4
  ret float %t0
}

define i32 @clamp_nonneg(i32 %x) {
entry:
  %x1 = alloca i32, align 4
  store i32 %x, ptr %x1, align 4
  %t0 = load i32, ptr %x1, align 4
  %icmp = icmp sgt i32 %t0, 0
  %icmpi32 = zext i1 %icmp to i32
  %tobool = icmp ne i32 %icmpi32, 0
  %t2 = load i32, ptr %x1, align 4
  %common.ret.op = select i1 %tobool, i32 %t2, i32 0
  ret i32 %common.ret.op
}

define void @fill_linear(ptr %arr, i32 %n, i32 %start) {
entry:
  %n1 = alloca i32, align 4
  store i32 %n, ptr %n1, align 4
  %start2 = alloca i32, align 4
  store i32 %start, ptr %start2, align 4
  %i = alloca i32, align 4
  store i32 0, ptr %i, align 4
  store i32 0, ptr %i, align 4
  br label %for_cond.0

for_cond.0:                                       ; preds = %for_body.1, %entry
  %t0 = load i32, ptr %i, align 4
  %t1 = load i32, ptr %n1, align 4
  %icmp = icmp slt i32 %t0, %t1
  %icmpi32 = zext i1 %icmp to i32
  %tobool = icmp ne i32 %icmpi32, 0
  br i1 %tobool, label %for_body.1, label %for_end.3

for_body.1:                                       ; preds = %for_cond.0
  %t3 = load i32, ptr %i, align 4
  %t4 = load i32, ptr %start2, align 4
  %imul = mul i32 %t3, 2
  %iadd = add i32 %t4, %imul
  %idxptr = getelementptr i32, ptr %arr, i32 %t3
  store i32 %iadd, ptr %idxptr, align 4
  %t8 = load i32, ptr %i, align 4
  %iadd3 = add i32 %t8, 1
  store i32 %iadd3, ptr %i, align 4
  br label %for_cond.0

for_end.3:                                        ; preds = %for_cond.0
  ret void
}

define void @print_case_result(i32 %case_id, i32 %value) {
entry:
  %case_id1 = alloca i32, align 4
  store i32 %case_id, ptr %case_id1, align 4
  %value2 = alloca i32, align 4
  store i32 %value, ptr %value2, align 4
  call void @putstr(ptr @msg_test)
  %t0 = load i32, ptr %case_id1, align 4
  call void @putint(i32 %t0)
  %t1 = load i32, ptr %value2, align 4
  %tobool = icmp ne i32 %t1, 0
  %msg_pass.msg_fail = select i1 %tobool, ptr @msg_pass, ptr @msg_fail
  call void @putstr(ptr %msg_pass.msg_fail)
  ret void
}

define i32 @test_arith_and_assign() {
entry:
  %flag = alloca i32, align 4
  store i32 0, ptr %flag, align 4
  %x = alloca i32, align 4
  store i32 14, ptr %x, align 4
  %t0 = load i32, ptr %x, align 4
  %icmp = icmp eq i32 %t0, 14
  %icmpi32 = zext i1 %icmp to i32
  %tobool = icmp ne i32 %icmpi32, 0
  br i1 %tobool, label %if_then.0, label %if_end.2

if_then.0:                                        ; preds = %entry
  %t2 = load i32, ptr %flag, align 4
  %iadd = add i32 %t2, 1
  store i32 %iadd, ptr %flag, align 4
  br label %if_end.2

if_end.2:                                         ; preds = %entry, %if_then.0
  %t4 = load i32, ptr %x, align 4
  %isub = sub i32 %t4, 5
  %idiv = sdiv i32 %isub, 3
  store i32 %idiv, ptr %x, align 4
  %t7 = load i32, ptr %x, align 4
  %icmp1 = icmp eq i32 %t7, 3
  %icmpi322 = zext i1 %icmp1 to i32
  %tobool3 = icmp ne i32 %icmpi322, 0
  br i1 %tobool3, label %if_then.3, label %if_end.5

if_then.3:                                        ; preds = %if_end.2
  %t9 = load i32, ptr %flag, align 4
  %iadd4 = add i32 %t9, 1
  store i32 %iadd4, ptr %flag, align 4
  br label %if_end.5

if_end.5:                                         ; preds = %if_end.2, %if_then.3
  %t11 = load i32, ptr %x, align 4
  %irem = srem i32 %t11, 2
  store i32 %irem, ptr %x, align 4
  %t13 = load i32, ptr %x, align 4
  %icmp5 = icmp eq i32 %t13, 1
  %icmpi326 = zext i1 %icmp5 to i32
  %tobool7 = icmp ne i32 %icmpi326, 0
  br i1 %tobool7, label %if_then.6, label %if_end.8

if_then.6:                                        ; preds = %if_end.5
  %t15 = load i32, ptr %flag, align 4
  %iadd8 = add i32 %t15, 1
  store i32 %iadd8, ptr %flag, align 4
  br label %if_end.8

if_end.8:                                         ; preds = %if_end.5, %if_then.6
  %t17 = load i32, ptr %flag, align 4
  %icmp9 = icmp eq i32 %t17, 3
  %icmpi3210 = zext i1 %icmp9 to i32
  ret i32 %icmpi3210
}

define i32 @test_unary_and_empty_stmt() {
entry:
  %a = alloca i32, align 4
  store i32 3, ptr %a, align 4
  %b = alloca i32, align 4
  %t0 = load i32, ptr %a, align 4
  %ineg = sub i32 0, %t0
  store i32 %ineg, ptr %b, align 4
  %c = alloca i32, align 4
  %t2 = load i32, ptr %b, align 4
  store i32 %t2, ptr %c, align 4
  %ok = alloca i32, align 4
  store i32 0, ptr %ok, align 4
  %t4 = load i32, ptr %ok, align 4
  %tobool = icmp ne i32 %t4, 0
  %nottmp = xor i1 %tobool, true
  %booltoi32 = zext i1 %nottmp to i32
  %tobool1 = icmp ne i32 %booltoi32, 0
  br i1 %tobool1, label %if_then.0, label %if_end.2

if_then.0:                                        ; preds = %entry
  %t6 = load i32, ptr %ok, align 4
  %iadd = add i32 %t6, 1
  store i32 %iadd, ptr %ok, align 4
  br label %if_end.2

if_end.2:                                         ; preds = %entry, %if_then.0
  %t8 = load i32, ptr %a, align 4
  %icmp = icmp eq i32 %t8, 3
  %icmpi32 = zext i1 %icmp to i32
  %t10 = load i32, ptr %b, align 4
  %icmp2 = icmp eq i32 %t10, -3
  %icmpi323 = zext i1 %icmp2 to i32
  %tobool4 = icmp ne i32 %icmpi32, 0
  %tobool5 = icmp ne i32 %icmpi323, 0
  %and = and i1 %tobool4, %tobool5
  %andi32 = zext i1 %and to i32
  %t13 = load i32, ptr %c, align 4
  %icmp6 = icmp eq i32 %t13, -3
  %icmpi327 = zext i1 %icmp6 to i32
  %tobool8 = icmp ne i32 %andi32, 0
  %tobool9 = icmp ne i32 %icmpi327, 0
  %and10 = and i1 %tobool8, %tobool9
  %andi3211 = zext i1 %and10 to i32
  %t16 = load i32, ptr %ok, align 4
  %icmp12 = icmp eq i32 %t16, 1
  %icmpi3213 = zext i1 %icmp12 to i32
  %tobool14 = icmp ne i32 %andi3211, 0
  %tobool15 = icmp ne i32 %icmpi3213, 0
  %and16 = and i1 %tobool14, %tobool15
  %andi3217 = zext i1 %and16 to i32
  ret i32 %andi3217
}

define i32 @test_if_else_logic() {
entry:
  %flag = alloca i32, align 4
  store i32 0, ptr %flag, align 4
  %a = alloca i32, align 4
  store i32 -3, ptr %a, align 4
  %b = alloca i32, align 4
  %t0 = load i32, ptr %a, align 4
  %calltmp = call i32 @clamp_nonneg(i32 %t0)
  store i32 %calltmp, ptr %b, align 4
  %t2 = load i32, ptr %a, align 4
  %icmp = icmp slt i32 %t2, 0
  %icmpi32 = zext i1 %icmp to i32
  %t4 = load i32, ptr %b, align 4
  %icmp1 = icmp eq i32 %t4, 0
  %icmpi322 = zext i1 %icmp1 to i32
  %tobool = icmp ne i32 %icmpi32, 0
  %tobool3 = icmp ne i32 %icmpi322, 0
  %and = and i1 %tobool, %tobool3
  %andi32 = zext i1 %and to i32
  %tobool4 = icmp ne i32 %andi32, 0
  br i1 %tobool4, label %if_then.0, label %if_end.2

if_then.0:                                        ; preds = %entry
  %t7 = load i32, ptr %flag, align 4
  %iadd = add i32 %t7, 1
  store i32 %iadd, ptr %flag, align 4
  br label %if_end.2

if_end.2:                                         ; preds = %entry, %if_then.0
  %t9 = load i32, ptr %a, align 4
  %icmp5 = icmp sgt i32 %t9, 0
  %icmpi326 = zext i1 %icmp5 to i32
  %t11 = load i32, ptr %b, align 4
  %icmp7 = icmp eq i32 %t11, 0
  %icmpi328 = zext i1 %icmp7 to i32
  %tobool9 = icmp ne i32 %icmpi326, 0
  %tobool10 = icmp ne i32 %icmpi328, 0
  %or = or i1 %tobool9, %tobool10
  %ori32 = zext i1 %or to i32
  %tobool11 = icmp ne i32 %ori32, 0
  br i1 %tobool11, label %if_then.3, label %if_end.5

if_then.3:                                        ; preds = %if_end.2
  %t14 = load i32, ptr %flag, align 4
  %iadd12 = add i32 %t14, 1
  store i32 %iadd12, ptr %flag, align 4
  br label %if_end.5

if_end.5:                                         ; preds = %if_end.2, %if_then.3
  %t16 = load i32, ptr %a, align 4
  %icmp13 = icmp eq i32 %t16, 0
  %icmpi3214 = zext i1 %icmp13 to i32
  %tobool15 = icmp ne i32 %icmpi3214, 0
  %nottmp = xor i1 %tobool15, true
  %booltoi32 = zext i1 %nottmp to i32
  %tobool16 = icmp ne i32 %booltoi32, 0
  br i1 %tobool16, label %if_then.6, label %if_end.8

if_then.6:                                        ; preds = %if_end.5
  %t19 = load i32, ptr %flag, align 4
  %iadd17 = add i32 %t19, 1
  store i32 %iadd17, ptr %flag, align 4
  br label %if_end.8

if_end.8:                                         ; preds = %if_end.5, %if_then.6
  %t21 = load i32, ptr %flag, align 4
  %icmp18 = icmp eq i32 %t21, 3
  %icmpi3219 = zext i1 %icmp18 to i32
  ret i32 %icmpi3219
}

define i32 @test_nested_branch() {
entry:
  %a = alloca i32, align 4
  store i32 2, ptr %a, align 4
  %b = alloca i32, align 4
  store i32 5, ptr %b, align 4
  %result = alloca i32, align 4
  store i32 0, ptr %result, align 4
  %t0 = load i32, ptr %a, align 4
  %t1 = load i32, ptr %b, align 4
  %icmp = icmp slt i32 %t0, %t1
  %icmpi32 = zext i1 %icmp to i32
  %tobool = icmp ne i32 %icmpi32, 0
  br i1 %tobool, label %if_then.0, label %if_else.1

if_then.0:                                        ; preds = %entry
  %t3 = load i32, ptr %a, align 4
  %t4 = load i32, ptr %b, align 4
  %iadd = add i32 %t3, %t4
  %icmp1 = icmp eq i32 %iadd, 7
  %icmpi322 = zext i1 %icmp1 to i32
  %tobool3 = icmp ne i32 %icmpi322, 0
  br i1 %tobool3, label %if_then.3, label %if_else.4

if_then.3:                                        ; preds = %if_then.0
  store i32 1, ptr %result, align 4
  br label %if_end.2

if_else.4:                                        ; preds = %if_then.0
  store i32 100, ptr %result, align 4
  br label %if_end.2

if_else.1:                                        ; preds = %entry
  store i32 100, ptr %result, align 4
  br label %if_end.2

if_end.2:                                         ; preds = %if_then.3, %if_else.4, %if_else.1
  %t7 = load i32, ptr %result, align 4
  %icmp4 = icmp eq i32 %t7, 1
  %icmpi325 = zext i1 %icmp4 to i32
  ret i32 %icmpi325
}

define i32 @test_while_break_continue() {
entry:
  %i = alloca i32, align 4
  store i32 0, ptr %i, align 4
  %sum = alloca i32, align 4
  store i32 0, ptr %sum, align 4
  br label %while_cond.0

while_cond.0:                                     ; preds = %if_end.5, %while_body.1, %entry
  %t0 = load i32, ptr %i, align 4
  %icmp = icmp slt i32 %t0, 8
  %icmpi32 = zext i1 %icmp to i32
  %tobool = icmp ne i32 %icmpi32, 0
  br i1 %tobool, label %while_body.1, label %while_end.2

while_body.1:                                     ; preds = %while_cond.0
  %t2 = load i32, ptr %i, align 4
  %iadd = add i32 %t2, 1
  store i32 %iadd, ptr %i, align 4
  %t4 = load i32, ptr %i, align 4
  %irem = srem i32 %t4, 2
  %icmp1 = icmp eq i32 %irem, 0
  %icmpi322 = zext i1 %icmp1 to i32
  %tobool3 = icmp ne i32 %icmpi322, 0
  br i1 %tobool3, label %while_cond.0, label %if_end.5

if_end.5:                                         ; preds = %while_body.1
  %t7 = load i32, ptr %sum, align 4
  %t8 = load i32, ptr %i, align 4
  %iadd4 = add i32 %t7, %t8
  store i32 %iadd4, ptr %sum, align 4
  %t10 = load i32, ptr %sum, align 4
  %icmp5 = icmp sgt i32 %t10, 10
  %icmpi326 = zext i1 %icmp5 to i32
  %tobool7 = icmp ne i32 %icmpi326, 0
  br i1 %tobool7, label %while_end.2, label %while_cond.0

while_end.2:                                      ; preds = %if_end.5, %while_cond.0
  %t12 = load i32, ptr %sum, align 4
  %icmp8 = icmp eq i32 %t12, 16
  %icmpi329 = zext i1 %icmp8 to i32
  ret i32 %icmpi329
}

define i32 @test_for_zero_iteration() {
entry:
  %i = alloca i32, align 4
  store i32 0, ptr %i, align 4
  %sum = alloca i32, align 4
  store i32 0, ptr %sum, align 4
  store i32 0, ptr %i, align 4
  br label %for_cond.0

for_cond.0:                                       ; preds = %for_body.1, %entry
  %t0 = load i32, ptr %i, align 4
  %icmp = icmp slt i32 %t0, 0
  %icmpi32 = zext i1 %icmp to i32
  %tobool = icmp ne i32 %icmpi32, 0
  %t2 = load i32, ptr %sum, align 4
  br i1 %tobool, label %for_body.1, label %for_end.3

for_body.1:                                       ; preds = %for_cond.0
  %iadd = add i32 %t2, 1
  store i32 %iadd, ptr %sum, align 4
  %t4 = load i32, ptr %i, align 4
  %iadd1 = add i32 %t4, 1
  store i32 %iadd1, ptr %i, align 4
  br label %for_cond.0

for_end.3:                                        ; preds = %for_cond.0
  %icmp2 = icmp eq i32 %t2, 0
  %icmpi323 = zext i1 %icmp2 to i32
  ret i32 %icmpi323
}

define i32 @test_for_break_continue() {
entry:
  %i = alloca i32, align 4
  store i32 0, ptr %i, align 4
  %acc = alloca i32, align 4
  store i32 0, ptr %acc, align 4
  store i32 0, ptr %i, align 4
  br label %for_cond.0

for_cond.0:                                       ; preds = %for_update.2, %entry
  %t0 = load i32, ptr %i, align 4
  %icmp = icmp slt i32 %t0, 10
  %icmpi32 = zext i1 %icmp to i32
  %tobool = icmp ne i32 %icmpi32, 0
  br i1 %tobool, label %for_body.1, label %for_end.3

for_body.1:                                       ; preds = %for_cond.0
  %t2 = load i32, ptr %i, align 4
  %icmp1 = icmp eq i32 %t2, 2
  %icmpi322 = zext i1 %icmp1 to i32
  %tobool3 = icmp ne i32 %icmpi322, 0
  br i1 %tobool3, label %for_update.2, label %if_end.6

if_end.6:                                         ; preds = %for_body.1
  %t4 = load i32, ptr %i, align 4
  %icmp4 = icmp eq i32 %t4, 7
  %icmpi325 = zext i1 %icmp4 to i32
  %tobool6 = icmp ne i32 %icmpi325, 0
  br i1 %tobool6, label %for_end.3, label %if_end.9

if_end.9:                                         ; preds = %if_end.6
  %t6 = load i32, ptr %acc, align 4
  %t7 = load i32, ptr %i, align 4
  %iadd = add i32 %t6, %t7
  store i32 %iadd, ptr %acc, align 4
  br label %for_update.2

for_update.2:                                     ; preds = %for_body.1, %if_end.9
  %t9 = load i32, ptr %i, align 4
  %iadd7 = add i32 %t9, 1
  store i32 %iadd7, ptr %i, align 4
  br label %for_cond.0

for_end.3:                                        ; preds = %if_end.6, %for_cond.0
  %t11 = load i32, ptr %acc, align 4
  %icmp8 = icmp eq i32 %t11, 19
  %icmpi329 = zext i1 %icmp8 to i32
  ret i32 %icmpi329
}

define i32 @test_array() {
entry:
  %local = alloca [4 x i32], align 4
  %idxptr13 = bitcast ptr %local to ptr
  store i32 0, ptr %idxptr13, align 4
  %idxptr1 = getelementptr [4 x i32], ptr %local, i32 0, i32 1
  store i32 0, ptr %idxptr1, align 4
  %idxptr2 = getelementptr [4 x i32], ptr %local, i32 0, i32 2
  store i32 0, ptr %idxptr2, align 4
  %idxptr3 = getelementptr [4 x i32], ptr %local, i32 0, i32 3
  store i32 0, ptr %idxptr3, align 4
  %i = alloca i32, align 4
  store i32 0, ptr %i, align 4
  %sum = alloca i32, align 4
  store i32 0, ptr %sum, align 4
  %arraydecay14 = bitcast ptr %local to ptr
  call void @fill_linear(ptr %arraydecay14, i32 4, i32 3)
  br label %while_cond.0

while_cond.0:                                     ; preds = %while_body.1, %entry
  %t0 = load i32, ptr %i, align 4
  %icmp = icmp slt i32 %t0, 4
  %icmpi32 = zext i1 %icmp to i32
  %tobool = icmp ne i32 %icmpi32, 0
  %t2 = load i32, ptr %sum, align 4
  br i1 %tobool, label %while_body.1, label %while_end.2

while_body.1:                                     ; preds = %while_cond.0
  %t3 = load i32, ptr %i, align 4
  %idxptr4 = getelementptr [4 x i32], ptr %local, i32 0, i32 %t3
  %t4 = load i32, ptr %idxptr4, align 4
  %iadd = add i32 %t2, %t4
  store i32 %iadd, ptr %sum, align 4
  %t6 = load i32, ptr %i, align 4
  %iadd5 = add i32 %t6, 1
  store i32 %iadd5, ptr %i, align 4
  br label %while_cond.0

while_end.2:                                      ; preds = %while_cond.0
  %icmp6 = icmp eq i32 %t2, 24
  %icmpi327 = zext i1 %icmp6 to i32
  %idxptr8 = getelementptr [4 x i32], ptr %local, i32 0, i32 2
  %t10 = load i32, ptr %idxptr8, align 4
  %icmp9 = icmp eq i32 %t10, 7
  %icmpi3210 = zext i1 %icmp9 to i32
  %tobool11 = icmp ne i32 %icmpi327, 0
  %tobool12 = icmp ne i32 %icmpi3210, 0
  %and = and i1 %tobool11, %tobool12
  %andi32 = zext i1 %and to i32
  ret i32 %andi32
}

define i32 @test_prefix_postfix() {
entry:
  %x = alloca i32, align 4
  store i32 5, ptr %x, align 4
  %y = alloca i32, align 4
  store i32 0, ptr %y, align 4
  %t0 = load i32, ptr %x, align 4
  %iadd = add i32 %t0, 1
  store i32 %iadd, ptr %x, align 4
  store i32 %t0, ptr %y, align 4
  %t2 = load i32, ptr %y, align 4
  %t3 = load i32, ptr %x, align 4
  %iadd1 = add i32 %t3, 1
  store i32 %iadd1, ptr %x, align 4
  %iadd2 = add i32 %t2, %iadd1
  store i32 %iadd2, ptr %y, align 4
  %t6 = load i32, ptr %y, align 4
  %t7 = load i32, ptr %x, align 4
  %isub = sub i32 %t7, 1
  store i32 %isub, ptr %x, align 4
  %iadd3 = add i32 %t6, %t7
  store i32 %iadd3, ptr %y, align 4
  %t10 = load i32, ptr %y, align 4
  %t11 = load i32, ptr %x, align 4
  %isub4 = sub i32 %t11, 1
  store i32 %isub4, ptr %x, align 4
  %iadd5 = add i32 %t10, %isub4
  store i32 %iadd5, ptr %y, align 4
  %t14 = load i32, ptr %x, align 4
  %icmp = icmp eq i32 %t14, 5
  %icmpi32 = zext i1 %icmp to i32
  %t16 = load i32, ptr %y, align 4
  %icmp6 = icmp eq i32 %t16, 24
  %icmpi327 = zext i1 %icmp6 to i32
  %tobool = icmp ne i32 %icmpi32, 0
  %tobool8 = icmp ne i32 %icmpi327, 0
  %and = and i1 %tobool, %tobool8
  %andi32 = zext i1 %and to i32
  ret i32 %andi32
}

define i32 @test_scope_and_call() {
entry:
  %x = alloca i32, align 4
  store i32 10, ptr %x, align 4
  %x1 = alloca i32, align 4
  store i32 3, ptr %x1, align 4
  %t0 = load i32, ptr %x1, align 4
  %iadd = add i32 %t0, 2
  store i32 %iadd, ptr %x1, align 4
  %t2 = load i32, ptr %x1, align 4
  %icmp = icmp ne i32 %t2, 5
  %icmpi32 = zext i1 %icmp to i32
  %tobool = icmp ne i32 %icmpi32, 0
  br i1 %tobool, label %common.ret, label %if_end.2

common.ret:                                       ; preds = %entry, %if_end.2
  %common.ret.op = phi i32 [ %icmpi323, %if_end.2 ], [ 0, %entry ]
  ret i32 %common.ret.op

if_end.2:                                         ; preds = %entry
  %t4 = load i32, ptr %x, align 4
  %t5 = load i32, ptr @g_scalar, align 4
  %calltmp = call i32 @add3(i32 %t4, i32 %t5, i32 11)
  %icmp2 = icmp eq i32 %calltmp, 28
  %icmpi323 = zext i1 %icmp2 to i32
  br label %common.ret
}

define i32 @test_float_expr() {
entry:
  %v = alloca float, align 4
  %t0 = load float, ptr @g_f, align 4
  %fadd = fadd float %t0, 7.500000e-01
  store float %fadd, ptr %v, align 4
  %t2 = load float, ptr %v, align 4
  %fcmp = fcmp ogt float %t2, 0x3FFE666660000000
  %fcmpi32 = zext i1 %fcmp to i32
  %fcmp1 = fcmp olt float %t2, 0x4000CCCCC0000000
  %fcmpi322 = zext i1 %fcmp1 to i32
  %tobool = icmp ne i32 %fcmpi32, 0
  %tobool3 = icmp ne i32 %fcmpi322, 0
  %and = and i1 %tobool, %tobool3
  %andi32 = zext i1 %and to i32
  %tobool4 = icmp ne i32 %andi32, 0
  %spec.select = select i1 %tobool4, i32 1, i32 0
  ret i32 %spec.select
}

define i32 @test_float_arith_extended() {
entry:
  %a = alloca float, align 4
  store float 1.500000e+00, ptr %a, align 4
  %b = alloca float, align 4
  store float 2.000000e+00, ptr %b, align 4
  %c = alloca float, align 4
  %t0 = load float, ptr %a, align 4
  %t1 = load float, ptr %b, align 4
  %fmul = fmul float %t0, %t1
  %fdiv = fdiv float %t1, %t0
  %fadd = fadd float %fmul, %fdiv
  %fsub = fsub float %fadd, 1.000000e+00
  store float %fsub, ptr %c, align 4
  %t8 = load float, ptr %c, align 4
  %fcmp = fcmp ogt float %t8, 0x400A666660000000
  %fcmpi32 = zext i1 %fcmp to i32
  %fcmp1 = fcmp olt float %t8, 0x400B333340000000
  %fcmpi322 = zext i1 %fcmp1 to i32
  %tobool = icmp ne i32 %fcmpi32, 0
  %tobool3 = icmp ne i32 %fcmpi322, 0
  %and = and i1 %tobool, %tobool3
  %andi32 = zext i1 %and to i32
  %tobool4 = icmp ne i32 %andi32, 0
  %spec.select = select i1 %tobool4, i32 1, i32 0
  ret i32 %spec.select
}

define i32 @test_forced_conversion() {
entry:
  %i = alloca i32, align 4
  store i32 5, ptr %i, align 4
  %f = alloca float, align 4
  store float 0x4004CCCCC0000000, ptr %f, align 4
  %to_float = alloca float, align 4
  %t0 = load i32, ptr %i, align 4
  %itof = sitofp i32 %t0 to float
  store float %itof, ptr %to_float, align 4
  %to_int = alloca i32, align 4
  %t1 = load float, ptr %f, align 4
  %ftoi = fptosi float %t1 to i32
  store i32 %ftoi, ptr %to_int, align 4
  %mixed_assign = alloca i32, align 4
  %t2 = load i32, ptr %i, align 4
  %t3 = load float, ptr %f, align 4
  %itof1 = sitofp i32 %t2 to float
  %fadd = fadd float %itof1, %t3
  %ftoi2 = fptosi float %fadd to i32
  store i32 %ftoi2, ptr %mixed_assign, align 4
  %via_call_int = alloca i32, align 4
  %t5 = load float, ptr %f, align 4
  %ftoi3 = fptosi float %t5 to i32
  %calltmp = call i32 @passthrough_int(i32 %ftoi3)
  store i32 %calltmp, ptr %via_call_int, align 4
  %via_call_float = alloca float, align 4
  %t7 = load i32, ptr %i, align 4
  %itof4 = sitofp i32 %t7 to float
  %calltmp5 = call float @passthrough_float(float %itof4)
  store float %calltmp5, ptr %via_call_float, align 4
  %ok = alloca i32, align 4
  store i32 0, ptr %ok, align 4
  %t9 = load float, ptr %to_float, align 4
  %fcmp = fcmp ogt float %t9, 0x40139999A0000000
  %fcmpi32 = zext i1 %fcmp to i32
  %fcmp6 = fcmp olt float %t9, 0x4014666660000000
  %fcmpi327 = zext i1 %fcmp6 to i32
  %tobool = icmp ne i32 %fcmpi32, 0
  %tobool8 = icmp ne i32 %fcmpi327, 0
  %and = and i1 %tobool, %tobool8
  %andi32 = zext i1 %and to i32
  %tobool9 = icmp ne i32 %andi32, 0
  br i1 %tobool9, label %if_then.0, label %if_end.2

if_then.0:                                        ; preds = %entry
  %t14 = load i32, ptr %ok, align 4
  %iadd = add i32 %t14, 1
  store i32 %iadd, ptr %ok, align 4
  br label %if_end.2

if_end.2:                                         ; preds = %entry, %if_then.0
  %t16 = load i32, ptr %to_int, align 4
  %icmp = icmp eq i32 %t16, 2
  %icmpi32 = zext i1 %icmp to i32
  %tobool10 = icmp ne i32 %icmpi32, 0
  br i1 %tobool10, label %if_then.3, label %if_end.5

if_then.3:                                        ; preds = %if_end.2
  %t18 = load i32, ptr %ok, align 4
  %iadd11 = add i32 %t18, 1
  store i32 %iadd11, ptr %ok, align 4
  br label %if_end.5

if_end.5:                                         ; preds = %if_end.2, %if_then.3
  %t20 = load i32, ptr %mixed_assign, align 4
  %icmp12 = icmp eq i32 %t20, 7
  %icmpi3213 = zext i1 %icmp12 to i32
  %tobool14 = icmp ne i32 %icmpi3213, 0
  br i1 %tobool14, label %if_then.6, label %if_end.8

if_then.6:                                        ; preds = %if_end.5
  %t22 = load i32, ptr %ok, align 4
  %iadd15 = add i32 %t22, 1
  store i32 %iadd15, ptr %ok, align 4
  br label %if_end.8

if_end.8:                                         ; preds = %if_end.5, %if_then.6
  %t24 = load i32, ptr %via_call_int, align 4
  %icmp16 = icmp eq i32 %t24, 2
  %icmpi3217 = zext i1 %icmp16 to i32
  %tobool18 = icmp ne i32 %icmpi3217, 0
  br i1 %tobool18, label %if_then.9, label %if_end.11

if_then.9:                                        ; preds = %if_end.8
  %t26 = load i32, ptr %ok, align 4
  %iadd19 = add i32 %t26, 1
  store i32 %iadd19, ptr %ok, align 4
  br label %if_end.11

if_end.11:                                        ; preds = %if_end.8, %if_then.9
  %t28 = load float, ptr %via_call_float, align 4
  %fcmp20 = fcmp ogt float %t28, 0x40139999A0000000
  %fcmpi3221 = zext i1 %fcmp20 to i32
  %fcmp22 = fcmp olt float %t28, 0x4014666660000000
  %fcmpi3223 = zext i1 %fcmp22 to i32
  %tobool24 = icmp ne i32 %fcmpi3221, 0
  %tobool25 = icmp ne i32 %fcmpi3223, 0
  %and26 = and i1 %tobool24, %tobool25
  %andi3227 = zext i1 %and26 to i32
  %tobool28 = icmp ne i32 %andi3227, 0
  br i1 %tobool28, label %if_then.12, label %if_end.14

if_then.12:                                       ; preds = %if_end.11
  %t33 = load i32, ptr %ok, align 4
  %iadd29 = add i32 %t33, 1
  store i32 %iadd29, ptr %ok, align 4
  br label %if_end.14

if_end.14:                                        ; preds = %if_end.11, %if_then.12
  %t35 = load i32, ptr %ok, align 4
  %icmp30 = icmp eq i32 %t35, 5
  %icmpi3231 = zext i1 %icmp30 to i32
  ret i32 %icmpi3231
}

define i32 @test_multidim_row_major() {
entry:
  %m = alloca [2 x [3 x i32]], align 4
  %i = alloca i32, align 4
  store i32 0, ptr %i, align 4
  %j = alloca i32, align 4
  store i32 0, ptr %j, align 4
  %k = alloca i32, align 4
  store i32 1, ptr %k, align 4
  store i32 0, ptr %i, align 4
  br label %for_cond.0

for_cond.0:                                       ; preds = %for_update.2, %entry
  %t0 = load i32, ptr %i, align 4
  %icmp = icmp slt i32 %t0, 2
  %icmpi32 = zext i1 %icmp to i32
  %tobool = icmp ne i32 %icmpi32, 0
  br i1 %tobool, label %for_body.1, label %for_end.3

for_body.1:                                       ; preds = %for_cond.0
  store i32 0, ptr %j, align 4
  br label %for_cond.4

for_cond.4:                                       ; preds = %for_body.5, %for_body.1
  %t2 = load i32, ptr %j, align 4
  %icmp1 = icmp slt i32 %t2, 3
  %icmpi322 = zext i1 %icmp1 to i32
  %tobool3 = icmp ne i32 %icmpi322, 0
  br i1 %tobool3, label %for_body.5, label %for_update.2

for_body.5:                                       ; preds = %for_cond.4
  %t4 = load i32, ptr %j, align 4
  %t5 = load i32, ptr %i, align 4
  %t6 = load i32, ptr %k, align 4
  %idxptr = getelementptr [2 x [3 x i32]], ptr %m, i32 0, i32 %t5, i32 %t4
  store i32 %t6, ptr %idxptr, align 4
  %t7 = load i32, ptr %k, align 4
  %iadd = add i32 %t7, 1
  store i32 %iadd, ptr %k, align 4
  %t9 = load i32, ptr %j, align 4
  %iadd4 = add i32 %t9, 1
  store i32 %iadd4, ptr %j, align 4
  br label %for_cond.4

for_update.2:                                     ; preds = %for_cond.4
  %t11 = load i32, ptr %i, align 4
  %iadd5 = add i32 %t11, 1
  store i32 %iadd5, ptr %i, align 4
  br label %for_cond.0

for_end.3:                                        ; preds = %for_cond.0
  %idxptr642 = bitcast ptr %m to ptr
  %t13 = load i32, ptr %idxptr642, align 4
  %icmp7 = icmp eq i32 %t13, 1
  %icmpi328 = zext i1 %icmp7 to i32
  %idxptr9 = getelementptr [2 x [3 x i32]], ptr %m, i32 0, i32 0, i32 1
  %t15 = load i32, ptr %idxptr9, align 4
  %icmp10 = icmp eq i32 %t15, 2
  %icmpi3211 = zext i1 %icmp10 to i32
  %tobool12 = icmp ne i32 %icmpi328, 0
  %tobool13 = icmp ne i32 %icmpi3211, 0
  %and = and i1 %tobool12, %tobool13
  %andi32 = zext i1 %and to i32
  %idxptr14 = getelementptr [2 x [3 x i32]], ptr %m, i32 0, i32 0, i32 2
  %t18 = load i32, ptr %idxptr14, align 4
  %icmp15 = icmp eq i32 %t18, 3
  %icmpi3216 = zext i1 %icmp15 to i32
  %tobool17 = icmp ne i32 %andi32, 0
  %tobool18 = icmp ne i32 %icmpi3216, 0
  %and19 = and i1 %tobool17, %tobool18
  %andi3220 = zext i1 %and19 to i32
  %idxptr21 = getelementptr [2 x [3 x i32]], ptr %m, i32 0, i32 1, i32 0
  %t21 = load i32, ptr %idxptr21, align 4
  %icmp22 = icmp eq i32 %t21, 4
  %icmpi3223 = zext i1 %icmp22 to i32
  %tobool24 = icmp ne i32 %andi3220, 0
  %tobool25 = icmp ne i32 %icmpi3223, 0
  %and26 = and i1 %tobool24, %tobool25
  %andi3227 = zext i1 %and26 to i32
  %idxptr28 = getelementptr [2 x [3 x i32]], ptr %m, i32 0, i32 1, i32 1
  %t24 = load i32, ptr %idxptr28, align 4
  %icmp29 = icmp eq i32 %t24, 5
  %icmpi3230 = zext i1 %icmp29 to i32
  %tobool31 = icmp ne i32 %andi3227, 0
  %tobool32 = icmp ne i32 %icmpi3230, 0
  %and33 = and i1 %tobool31, %tobool32
  %andi3234 = zext i1 %and33 to i32
  %idxptr35 = getelementptr [2 x [3 x i32]], ptr %m, i32 0, i32 1, i32 2
  %t27 = load i32, ptr %idxptr35, align 4
  %icmp36 = icmp eq i32 %t27, 6
  %icmpi3237 = zext i1 %icmp36 to i32
  %tobool38 = icmp ne i32 %andi3234, 0
  %tobool39 = icmp ne i32 %icmpi3237, 0
  %and40 = and i1 %tobool38, %tobool39
  %andi3241 = zext i1 %and40 to i32
  ret i32 %andi3241
}

define i32 @test_const_and_global_index() {
entry:
  %idx = alloca i32, align 4
  store i32 3, ptr %idx, align 4
  %v = alloca i32, align 4
  %t0 = load i32, ptr getelementptr inbounds ([5 x i32], ptr @g_arr, i32 0, i32 2), align 4
  store i32 %t0, ptr %v, align 4
  %t1 = load i32, ptr %v, align 4
  %icmp = icmp eq i32 %t1, 5
  %icmpi32 = zext i1 %icmp to i32
  %tobool = icmp ne i32 %icmpi32, 0
  %and = and i1 %tobool, true
  %andi32 = zext i1 %and to i32
  ret i32 %andi32
}

define i32 @integrated_test() {
entry:
  %buf = alloca [5 x i32], align 4
  %idxptr22 = bitcast ptr %buf to ptr
  store i32 0, ptr %idxptr22, align 4
  %idxptr1 = getelementptr [5 x i32], ptr %buf, i32 0, i32 1
  store i32 0, ptr %idxptr1, align 4
  %idxptr2 = getelementptr [5 x i32], ptr %buf, i32 0, i32 2
  store i32 0, ptr %idxptr2, align 4
  %idxptr3 = getelementptr [5 x i32], ptr %buf, i32 0, i32 3
  store i32 0, ptr %idxptr3, align 4
  %idxptr4 = getelementptr [5 x i32], ptr %buf, i32 0, i32 4
  store i32 0, ptr %idxptr4, align 4
  %i = alloca i32, align 4
  store i32 0, ptr %i, align 4
  %k = alloca i32, align 4
  store i32 0, ptr %k, align 4
  %total = alloca i32, align 4
  store i32 0, ptr %total, align 4
  %arraydecay23 = bitcast ptr %buf to ptr
  call void @fill_linear(ptr %arraydecay23, i32 5, i32 1)
  store i32 0, ptr %i, align 4
  br label %for_cond.0

for_cond.0:                                       ; preds = %for_body.1, %entry
  %t0 = load i32, ptr %i, align 4
  %icmp = icmp slt i32 %t0, 5
  %icmpi32 = zext i1 %icmp to i32
  %tobool = icmp ne i32 %icmpi32, 0
  %t2 = load i32, ptr %total, align 4
  br i1 %tobool, label %for_body.1, label %for_end.3

for_body.1:                                       ; preds = %for_cond.0
  %t3 = load i32, ptr %i, align 4
  %idxptr5 = getelementptr [5 x i32], ptr %buf, i32 0, i32 %t3
  %t4 = load i32, ptr %idxptr5, align 4
  %isub = sub i32 4, %t3
  %idxptr6 = getelementptr [5 x i32], ptr @g_arr, i32 0, i32 %isub
  %t7 = load i32, ptr %idxptr6, align 4
  %imul = mul i32 %t4, %t7
  %iadd = add i32 %t2, %imul
  store i32 %iadd, ptr %total, align 4
  %t10 = load i32, ptr %i, align 4
  %iadd7 = add i32 %t10, 1
  store i32 %iadd7, ptr %i, align 4
  br label %for_cond.0

for_end.3:                                        ; preds = %for_cond.0
  %icmp8 = icmp eq i32 %t2, 85
  %icmpi329 = zext i1 %icmp8 to i32
  %tobool10 = icmp ne i32 %icmpi329, 0
  %t14 = load i32, ptr %total, align 4
  br i1 %tobool10, label %if_then.4, label %if_else.5

if_then.4:                                        ; preds = %for_end.3
  %calltmp = call i32 @add3(i32 1, i32 2, i32 3)
  %iadd11 = add i32 %t14, %calltmp
  store i32 %iadd11, ptr %total, align 4
  br label %while_cond.7.preheader

if_else.5:                                        ; preds = %for_end.3
  %isub12 = sub i32 %t14, 100
  store i32 %isub12, ptr %total, align 4
  br label %while_cond.7.preheader

while_cond.7.preheader:                           ; preds = %if_else.5, %if_then.4
  br label %while_cond.7

while_cond.7:                                     ; preds = %while_cond.7.preheader, %while_body.8
  %t19 = load i32, ptr %k, align 4
  %icmp13 = icmp slt i32 %t19, 3
  %icmpi3214 = zext i1 %icmp13 to i32
  %tobool15 = icmp ne i32 %icmpi3214, 0
  %t21 = load i32, ptr %total, align 4
  br i1 %tobool15, label %while_body.8, label %while_end.9

while_body.8:                                     ; preds = %while_cond.7
  %t22 = load i32, ptr %k, align 4
  %iadd16 = add i32 %t22, 1
  %idxptr17 = getelementptr [5 x i32], ptr @g_arr, i32 0, i32 %iadd16
  %t24 = load i32, ptr %idxptr17, align 4
  %iadd18 = add i32 %t21, %t24
  store i32 %iadd18, ptr %total, align 4
  %t26 = load i32, ptr %k, align 4
  %iadd19 = add i32 %t26, 1
  store i32 %iadd19, ptr %k, align 4
  br label %while_cond.7

while_end.9:                                      ; preds = %while_cond.7
  %icmp20 = icmp eq i32 %t21, 106
  %icmpi3221 = zext i1 %icmp20 to i32
  ret i32 %icmpi3221
}

define i32 @main() {
entry:
  %failed = alloca i32, align 4
  store i32 16, ptr %failed, align 4
  %result = alloca i32, align 4
  store i32 0, ptr %result, align 4
  call void @putstr(ptr @msg_begin)
  %calltmp = call i32 @test_arith_and_assign()
  store i32 %calltmp, ptr %result, align 4
  %t1 = load i32, ptr %result, align 4
  call void @print_case_result(i32 1, i32 %t1)
  %t2 = load i32, ptr %failed, align 4
  %t3 = load i32, ptr %result, align 4
  %isub = sub i32 %t2, %t3
  store i32 %isub, ptr %failed, align 4
  %calltmp1 = call i32 @test_unary_and_empty_stmt()
  store i32 %calltmp1, ptr %result, align 4
  %t6 = load i32, ptr %result, align 4
  call void @print_case_result(i32 2, i32 %t6)
  %t7 = load i32, ptr %failed, align 4
  %t8 = load i32, ptr %result, align 4
  %isub2 = sub i32 %t7, %t8
  store i32 %isub2, ptr %failed, align 4
  %calltmp3 = call i32 @test_if_else_logic()
  store i32 %calltmp3, ptr %result, align 4
  %t11 = load i32, ptr %result, align 4
  call void @print_case_result(i32 3, i32 %t11)
  %t12 = load i32, ptr %failed, align 4
  %t13 = load i32, ptr %result, align 4
  %isub4 = sub i32 %t12, %t13
  store i32 %isub4, ptr %failed, align 4
  %calltmp5 = call i32 @test_nested_branch()
  store i32 %calltmp5, ptr %result, align 4
  %t16 = load i32, ptr %result, align 4
  call void @print_case_result(i32 4, i32 %t16)
  %t17 = load i32, ptr %failed, align 4
  %t18 = load i32, ptr %result, align 4
  %isub6 = sub i32 %t17, %t18
  store i32 %isub6, ptr %failed, align 4
  %calltmp7 = call i32 @test_while_break_continue()
  store i32 %calltmp7, ptr %result, align 4
  %t21 = load i32, ptr %result, align 4
  call void @print_case_result(i32 5, i32 %t21)
  %t22 = load i32, ptr %failed, align 4
  %t23 = load i32, ptr %result, align 4
  %isub8 = sub i32 %t22, %t23
  store i32 %isub8, ptr %failed, align 4
  %calltmp9 = call i32 @test_for_zero_iteration()
  store i32 %calltmp9, ptr %result, align 4
  %t26 = load i32, ptr %result, align 4
  call void @print_case_result(i32 6, i32 %t26)
  %t27 = load i32, ptr %failed, align 4
  %t28 = load i32, ptr %result, align 4
  %isub10 = sub i32 %t27, %t28
  store i32 %isub10, ptr %failed, align 4
  %calltmp11 = call i32 @test_for_break_continue()
  store i32 %calltmp11, ptr %result, align 4
  %t31 = load i32, ptr %result, align 4
  call void @print_case_result(i32 7, i32 %t31)
  %t32 = load i32, ptr %failed, align 4
  %t33 = load i32, ptr %result, align 4
  %isub12 = sub i32 %t32, %t33
  store i32 %isub12, ptr %failed, align 4
  %calltmp13 = call i32 @test_array()
  store i32 %calltmp13, ptr %result, align 4
  %t36 = load i32, ptr %result, align 4
  call void @print_case_result(i32 8, i32 %t36)
  %t37 = load i32, ptr %failed, align 4
  %t38 = load i32, ptr %result, align 4
  %isub14 = sub i32 %t37, %t38
  store i32 %isub14, ptr %failed, align 4
  %calltmp15 = call i32 @test_prefix_postfix()
  store i32 %calltmp15, ptr %result, align 4
  %t41 = load i32, ptr %result, align 4
  call void @print_case_result(i32 9, i32 %t41)
  %t42 = load i32, ptr %failed, align 4
  %t43 = load i32, ptr %result, align 4
  %isub16 = sub i32 %t42, %t43
  store i32 %isub16, ptr %failed, align 4
  %calltmp17 = call i32 @test_scope_and_call()
  store i32 %calltmp17, ptr %result, align 4
  %t46 = load i32, ptr %result, align 4
  call void @print_case_result(i32 10, i32 %t46)
  %t47 = load i32, ptr %failed, align 4
  %t48 = load i32, ptr %result, align 4
  %isub18 = sub i32 %t47, %t48
  store i32 %isub18, ptr %failed, align 4
  %calltmp19 = call i32 @test_float_expr()
  store i32 %calltmp19, ptr %result, align 4
  %t51 = load i32, ptr %result, align 4
  call void @print_case_result(i32 11, i32 %t51)
  %t52 = load i32, ptr %failed, align 4
  %t53 = load i32, ptr %result, align 4
  %isub20 = sub i32 %t52, %t53
  store i32 %isub20, ptr %failed, align 4
  %calltmp21 = call i32 @test_float_arith_extended()
  store i32 %calltmp21, ptr %result, align 4
  %t56 = load i32, ptr %result, align 4
  call void @print_case_result(i32 12, i32 %t56)
  %t57 = load i32, ptr %failed, align 4
  %t58 = load i32, ptr %result, align 4
  %isub22 = sub i32 %t57, %t58
  store i32 %isub22, ptr %failed, align 4
  %calltmp23 = call i32 @test_forced_conversion()
  store i32 %calltmp23, ptr %result, align 4
  %t61 = load i32, ptr %result, align 4
  call void @print_case_result(i32 13, i32 %t61)
  %t62 = load i32, ptr %failed, align 4
  %t63 = load i32, ptr %result, align 4
  %isub24 = sub i32 %t62, %t63
  store i32 %isub24, ptr %failed, align 4
  %calltmp25 = call i32 @test_multidim_row_major()
  store i32 %calltmp25, ptr %result, align 4
  %t66 = load i32, ptr %result, align 4
  call void @print_case_result(i32 14, i32 %t66)
  %t67 = load i32, ptr %failed, align 4
  %t68 = load i32, ptr %result, align 4
  %isub26 = sub i32 %t67, %t68
  store i32 %isub26, ptr %failed, align 4
  %calltmp27 = call i32 @test_const_and_global_index()
  store i32 %calltmp27, ptr %result, align 4
  %t71 = load i32, ptr %result, align 4
  call void @print_case_result(i32 15, i32 %t71)
  %t72 = load i32, ptr %failed, align 4
  %t73 = load i32, ptr %result, align 4
  %isub28 = sub i32 %t72, %t73
  store i32 %isub28, ptr %failed, align 4
  %calltmp29 = call i32 @integrated_test()
  store i32 %calltmp29, ptr %result, align 4
  %t76 = load i32, ptr %result, align 4
  call void @print_case_result(i32 16, i32 %t76)
  %t77 = load i32, ptr %failed, align 4
  %t78 = load i32, ptr %result, align 4
  %isub30 = sub i32 %t77, %t78
  store i32 %isub30, ptr %failed, align 4
  %t80 = load i32, ptr %failed, align 4
  ret i32 %t80
}

