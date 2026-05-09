	.text
	.file	"sysy_module"
	.globl	add3
	.p2align	2
	.type	add3,@function
add3:
	.cfi_startproc
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	add	w8, w0, w1
	stp	w1, w0, [sp, #8]
	add	w0, w8, w2
	str	w2, [sp, #4]
	add	sp, sp, #16
	ret
.Lfunc_end0:
	.size	add3, .Lfunc_end0-add3
	.cfi_endproc

	.globl	passthrough_int
	.p2align	2
	.type	passthrough_int,@function
passthrough_int:
	.cfi_startproc
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	str	w0, [sp, #12]
	add	sp, sp, #16
	ret
.Lfunc_end1:
	.size	passthrough_int, .Lfunc_end1-passthrough_int
	.cfi_endproc

	.globl	passthrough_float
	.p2align	2
	.type	passthrough_float,@function
passthrough_float:
	.cfi_startproc
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	str	s0, [sp, #12]
	add	sp, sp, #16
	ret
.Lfunc_end2:
	.size	passthrough_float, .Lfunc_end2-passthrough_float
	.cfi_endproc

	.globl	clamp_nonneg
	.p2align	2
	.type	clamp_nonneg,@function
clamp_nonneg:
	.cfi_startproc
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	mov	w8, w0
	bic	w0, w0, w0, asr #31
	str	w8, [sp, #12]
	add	sp, sp, #16
	ret
.Lfunc_end3:
	.size	clamp_nonneg, .Lfunc_end3-clamp_nonneg
	.cfi_endproc

	.globl	fill_linear
	.p2align	2
	.type	fill_linear,@function
fill_linear:
	.cfi_startproc
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	stp	w2, w1, [sp, #8]
	str	wzr, [sp, #4]
.LBB4_1:
	ldr	w8, [sp, #4]
	ldr	w9, [sp, #12]
	cmp	w8, w9
	b.ge	.LBB4_3
	ldp	w8, w9, [sp, #4]
	sxtw	x8, w8
	add	w9, w9, w8, lsl #1
	add	w10, w8, #1
	str	w9, [x0, x8, lsl #2]
	str	w10, [sp, #4]
	b	.LBB4_1
.LBB4_3:
	add	sp, sp, #16
	ret
.Lfunc_end4:
	.size	fill_linear, .Lfunc_end4-fill_linear
	.cfi_endproc

	.globl	print_case_result
	.p2align	2
	.type	print_case_result,@function
print_case_result:
	.cfi_startproc
	str	x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	.cfi_offset w30, -16
	stp	w1, w0, [sp, #8]
	adrp	x0, :got:msg_test
	ldr	x0, [x0, :got_lo12:msg_test]
	bl	putstr
	ldr	w0, [sp, #12]
	bl	putint
	ldr	w8, [sp, #8]
	adrp	x9, :got:msg_fail
	adrp	x10, :got:msg_pass
	ldr	x9, [x9, :got_lo12:msg_fail]
	ldr	x10, [x10, :got_lo12:msg_pass]
	cmp	w8, #0
	csel	x0, x10, x9, ne
	bl	putstr
	ldr	x30, [sp], #16
	ret
.Lfunc_end5:
	.size	print_case_result, .Lfunc_end5-print_case_result
	.cfi_endproc

	.globl	test_arith_and_assign
	.p2align	2
	.type	test_arith_and_assign,@function
test_arith_and_assign:
	.cfi_startproc
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	mov	w8, #14
	mov	w9, #1
	stp	w8, wzr, [sp, #8]
	cbz	w9, .LBB6_2
	ldr	w8, [sp, #12]
	add	w8, w8, #1
	str	w8, [sp, #12]
.LBB6_2:
	ldr	w8, [sp, #8]
	mov	w9, #21846
	movk	w9, #21845, lsl #16
	sub	w8, w8, #5
	smull	x8, w8, w9
	lsr	x9, x8, #63
	lsr	x8, x8, #32
	add	w8, w8, w9
	cmp	w8, #3
	str	w8, [sp, #8]
	b.ne	.LBB6_4
	ldr	w8, [sp, #12]
	add	w8, w8, #1
	str	w8, [sp, #12]
.LBB6_4:
	ldr	w8, [sp, #8]
	and	w9, w8, #0x1
	cmp	w8, #0
	cneg	w8, w9, lt
	cmp	w8, #1
	str	w8, [sp, #8]
	b.ne	.LBB6_6
	ldr	w8, [sp, #12]
	add	w8, w8, #1
	str	w8, [sp, #12]
.LBB6_6:
	ldr	w8, [sp, #12]
	cmp	w8, #3
	cset	w0, eq
	add	sp, sp, #16
	ret
.Lfunc_end6:
	.size	test_arith_and_assign, .Lfunc_end6-test_arith_and_assign
	.cfi_endproc

	.globl	test_unary_and_empty_stmt
	.p2align	2
	.type	test_unary_and_empty_stmt,@function
test_unary_and_empty_stmt:
	.cfi_startproc
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	mov	w8, #3
	mov	w9, #-3
	stp	w9, w8, [sp, #8]
	stp	wzr, w9, [sp]
	cbnz	wzr, .LBB7_2
	ldr	w8, [sp]
	add	w8, w8, #1
	str	w8, [sp]
.LBB7_2:
	ldp	w9, w8, [sp, #8]
	cmp	w8, #3
	ccmn	w9, #3, #0, eq
	ldp	w9, w8, [sp], #16
	ccmn	w8, #3, #0, eq
	ccmp	w9, #1, #0, eq
	cset	w0, eq
	ret
.Lfunc_end7:
	.size	test_unary_and_empty_stmt, .Lfunc_end7-test_unary_and_empty_stmt
	.cfi_endproc

	.globl	test_if_else_logic
	.p2align	2
	.type	test_if_else_logic,@function
test_if_else_logic:
	.cfi_startproc
	sub	sp, sp, #32
	str	x30, [sp, #16]
	.cfi_def_cfa_offset 32
	.cfi_offset w30, -16
	mov	w8, #-3
	mov	w0, #-3
	stp	w8, wzr, [sp, #24]
	bl	clamp_nonneg
	ldr	w8, [sp, #24]
	str	w0, [sp, #12]
	cmp	w8, #0
	ccmp	w0, #0, #0, lt
	b.ne	.LBB8_2
	ldr	w8, [sp, #28]
	add	w8, w8, #1
	str	w8, [sp, #28]
.LBB8_2:
	ldr	w8, [sp, #24]
	ldr	w9, [sp, #12]
	cmp	w8, #0
	ccmp	w9, #0, #4, le
	b.ne	.LBB8_4
	ldr	w8, [sp, #28]
	add	w8, w8, #1
	str	w8, [sp, #28]
.LBB8_4:
	ldr	w8, [sp, #24]
	cbz	w8, .LBB8_6
	ldr	w8, [sp, #28]
	add	w8, w8, #1
	str	w8, [sp, #28]
.LBB8_6:
	ldr	w8, [sp, #28]
	ldr	x30, [sp, #16]
	cmp	w8, #3
	cset	w0, eq
	add	sp, sp, #32
	ret
.Lfunc_end8:
	.size	test_if_else_logic, .Lfunc_end8-test_if_else_logic
	.cfi_endproc

	.globl	test_nested_branch
	.p2align	2
	.type	test_nested_branch,@function
test_nested_branch:
	.cfi_startproc
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	mov	w8, #2
	mov	w9, #5
	str	wzr, [sp, #4]
	stp	w9, w8, [sp, #8]
	mov	w8, #1
	cbz	w8, .LBB9_3
	ldp	w9, w8, [sp, #8]
	add	w8, w8, w9
	cmp	w8, #7
	b.ne	.LBB9_3
	mov	w8, #1
	b	.LBB9_4
.LBB9_3:
	mov	w8, #100
.LBB9_4:
	str	w8, [sp, #4]
	mov	w8, w8
	cmp	w8, #1
	cset	w0, eq
	add	sp, sp, #16
	ret
.Lfunc_end9:
	.size	test_nested_branch, .Lfunc_end9-test_nested_branch
	.cfi_endproc

	.globl	test_while_break_continue
	.p2align	2
	.type	test_while_break_continue,@function
test_while_break_continue:
	.cfi_startproc
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	str	xzr, [sp, #8]
.LBB10_1:
	ldr	w8, [sp, #12]
	cmp	w8, #7
	b.gt	.LBB10_4
	ldr	w8, [sp, #12]
	add	w8, w8, #1
	and	w9, w8, #0x1
	cmp	w8, #0
	str	w8, [sp, #12]
	cneg	w9, w9, lt
	cbz	w9, .LBB10_1
	ldp	w8, w9, [sp, #8]
	add	w8, w8, w9
	cmp	w8, #11
	str	w8, [sp, #8]
	b.lt	.LBB10_1
.LBB10_4:
	ldr	w8, [sp, #8]
	cmp	w8, #16
	cset	w0, eq
	add	sp, sp, #16
	ret
.Lfunc_end10:
	.size	test_while_break_continue, .Lfunc_end10-test_while_break_continue
	.cfi_endproc

	.globl	test_for_zero_iteration
	.p2align	2
	.type	test_for_zero_iteration,@function
test_for_zero_iteration:
	.cfi_startproc
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	str	xzr, [sp, #8]
	ldp	w8, w9, [sp, #8]
	tbz	w9, #31, .LBB11_2
.LBB11_1:
	ldr	w9, [sp, #12]
	add	w10, w8, #1
	add	w8, w9, #1
	stp	w10, w8, [sp, #8]
	ldp	w8, w9, [sp, #8]
	tbnz	w9, #31, .LBB11_1
.LBB11_2:
	cmp	w8, #0
	cset	w0, eq
	add	sp, sp, #16
	ret
.Lfunc_end11:
	.size	test_for_zero_iteration, .Lfunc_end11-test_for_zero_iteration
	.cfi_endproc

	.globl	test_for_break_continue
	.p2align	2
	.type	test_for_break_continue,@function
test_for_break_continue:
	.cfi_startproc
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	str	xzr, [sp, #8]
	b	.LBB12_2
.LBB12_1:
	ldr	w8, [sp, #12]
	add	w8, w8, #1
	str	w8, [sp, #12]
.LBB12_2:
	ldr	w8, [sp, #12]
	cmp	w8, #9
	b.gt	.LBB12_6
	ldr	w8, [sp, #12]
	cmp	w8, #2
	b.eq	.LBB12_1
	ldr	w8, [sp, #12]
	cmp	w8, #7
	b.eq	.LBB12_6
	ldp	w8, w9, [sp, #8]
	add	w8, w8, w9
	str	w8, [sp, #8]
	b	.LBB12_1
.LBB12_6:
	ldr	w8, [sp, #8]
	cmp	w8, #19
	cset	w0, eq
	add	sp, sp, #16
	ret
.Lfunc_end12:
	.size	test_for_break_continue, .Lfunc_end12-test_for_break_continue
	.cfi_endproc

	.globl	test_array
	.p2align	2
	.type	test_array,@function
test_array:
	.cfi_startproc
	sub	sp, sp, #48
	stp	x30, x19, [sp, #32]
	.cfi_def_cfa_offset 48
	.cfi_offset w19, -8
	.cfi_offset w30, -16
	add	x0, sp, #16
	mov	w1, #4
	mov	w2, #3
	stp	xzr, xzr, [sp, #16]
	add	x19, sp, #16
	str	xzr, [sp, #8]
	bl	fill_linear
.LBB13_1:
	ldp	w8, w9, [sp, #8]
	cmp	w9, #3
	b.gt	.LBB13_3
	ldrsw	x9, [sp, #12]
	ldr	w10, [x19, x9, lsl #2]
	add	w9, w9, #1
	add	w8, w8, w10
	stp	w8, w9, [sp, #8]
	b	.LBB13_1
.LBB13_3:
	ldr	w9, [sp, #24]
	cmp	w8, #24
	ldp	x30, x19, [sp, #32]
	ccmp	w9, #7, #0, eq
	cset	w0, eq
	add	sp, sp, #48
	ret
.Lfunc_end13:
	.size	test_array, .Lfunc_end13-test_array
	.cfi_endproc

	.globl	test_prefix_postfix
	.p2align	2
	.type	test_prefix_postfix,@function
test_prefix_postfix:
	.cfi_startproc
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	mov	w8, #5
	mov	w9, #24
	mov	w0, #1
	stp	w9, w8, [sp, #8]
	add	sp, sp, #16
	ret
.Lfunc_end14:
	.size	test_prefix_postfix, .Lfunc_end14-test_prefix_postfix
	.cfi_endproc

	.globl	test_scope_and_call
	.p2align	2
	.type	test_scope_and_call,@function
test_scope_and_call:
	.cfi_startproc
	str	x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	.cfi_offset w30, -16
	mov	w0, wzr
	mov	w8, #10
	mov	w9, #5
	stp	w9, w8, [sp, #8]
	cbz	wzr, .LBB15_2
	ldr	x30, [sp], #16
	ret
.LBB15_2:
	adrp	x8, :got:g_scalar
	mov	w2, #11
	ldr	x8, [x8, :got_lo12:g_scalar]
	ldr	w0, [sp, #12]
	ldr	w1, [x8]
	bl	add3
	cmp	w0, #28
	cset	w0, eq
	ldr	x30, [sp], #16
	ret
.Lfunc_end15:
	.size	test_scope_and_call, .Lfunc_end15-test_scope_and_call
	.cfi_endproc

	.globl	test_float_expr
	.p2align	2
	.type	test_float_expr,@function
test_float_expr:
	.cfi_startproc
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	adrp	x8, :got:g_f
	fmov	s0, #0.75000000
	ldr	x8, [x8, :got_lo12:g_f]
	ldr	s1, [x8]
	mov	w8, #26214
	movk	w8, #16390, lsl #16
	fadd	s0, s1, s0
	fmov	s1, w8
	mov	w8, #13107
	movk	w8, #16371, lsl #16
	fcmp	s0, s1
	fmov	s1, w8
	str	s0, [sp, #12]
	fccmp	s0, s1, #4, mi
	cset	w0, gt
	add	sp, sp, #16
	ret
.Lfunc_end16:
	.size	test_float_expr, .Lfunc_end16-test_float_expr
	.cfi_endproc

	.globl	test_float_arith_extended
	.p2align	2
	.type	test_float_arith_extended,@function
test_float_arith_extended:
	.cfi_startproc
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	mov	w8, #1069547520
	mov	w9, #1073741824
	mov	w0, #1
	stp	w9, w8, [sp, #8]
	mov	w8, #21846
	movk	w8, #16469, lsl #16
	str	w8, [sp, #4]
	add	sp, sp, #16
	ret
.Lfunc_end17:
	.size	test_float_arith_extended, .Lfunc_end17-test_float_arith_extended
	.cfi_endproc

	.globl	test_forced_conversion
	.p2align	2
	.type	test_forced_conversion,@function
test_forced_conversion:
	.cfi_startproc
	sub	sp, sp, #48
	str	x30, [sp, #32]
	.cfi_def_cfa_offset 48
	.cfi_offset w30, -16
	mov	w8, #5
	mov	w0, #2
	str	w8, [sp, #44]
	mov	w8, #26214
	ldr	s0, [sp, #44]
	movk	w8, #16422, lsl #16
	fmov	s1, w8
	str	w8, [sp, #40]
	mov	w8, #2
	scvtf	s0, s0
	fadd	s1, s0, s1
	str	s0, [sp, #28]
	fcvtzs	w9, s1
	stp	w9, w8, [sp, #20]
	bl	passthrough_int
	ldr	s0, [sp, #44]
	str	w0, [sp, #16]
	scvtf	s0, s0
	bl	passthrough_float
	mov	w8, #13107
	ldr	s1, [sp, #28]
	str	s0, [sp, #12]
	movk	w8, #16547, lsl #16
	str	wzr, [sp, #8]
	fmov	s2, w8
	mov	w8, #52429
	movk	w8, #16540, lsl #16
	fcmp	s1, s2
	fmov	s2, w8
	fccmp	s1, s2, #4, mi
	cset	w8, gt
	cbz	w8, .LBB18_2
	ldr	w8, [sp, #8]
	add	w8, w8, #1
	str	w8, [sp, #8]
.LBB18_2:
	ldr	w8, [sp, #24]
	cmp	w8, #2
	b.ne	.LBB18_4
	ldr	w8, [sp, #8]
	add	w8, w8, #1
	str	w8, [sp, #8]
.LBB18_4:
	ldr	w8, [sp, #20]
	cmp	w8, #7
	b.ne	.LBB18_6
	ldr	w8, [sp, #8]
	add	w8, w8, #1
	str	w8, [sp, #8]
.LBB18_6:
	ldr	w8, [sp, #16]
	cmp	w8, #2
	b.ne	.LBB18_8
	ldr	w8, [sp, #8]
	add	w8, w8, #1
	str	w8, [sp, #8]
.LBB18_8:
	mov	w8, #13107
	ldr	s0, [sp, #12]
	movk	w8, #16547, lsl #16
	fmov	s1, w8
	mov	w8, #52429
	movk	w8, #16540, lsl #16
	fcmp	s0, s1
	fmov	s1, w8
	fccmp	s0, s1, #4, mi
	cset	w8, gt
	cbz	w8, .LBB18_10
	ldr	w8, [sp, #8]
	add	w8, w8, #1
	str	w8, [sp, #8]
.LBB18_10:
	ldr	w8, [sp, #8]
	ldr	x30, [sp, #32]
	cmp	w8, #5
	cset	w0, eq
	add	sp, sp, #48
	ret
.Lfunc_end18:
	.size	test_forced_conversion, .Lfunc_end18-test_forced_conversion
	.cfi_endproc

	.globl	test_multidim_row_major
	.p2align	2
	.type	test_multidim_row_major,@function
test_multidim_row_major:
	.cfi_startproc
	sub	sp, sp, #48
	.cfi_def_cfa_offset 48
	mov	w10, #1
	mov	w8, #12
	add	x9, sp, #24
	str	xzr, [sp, #16]
	str	w10, [sp, #12]
	b	.LBB19_2
.LBB19_1:
	ldr	w10, [sp, #20]
	add	w10, w10, #1
	str	w10, [sp, #20]
.LBB19_2:
	ldr	w10, [sp, #20]
	cmp	w10, #1
	b.gt	.LBB19_6
	str	wzr, [sp, #16]
.LBB19_4:
	ldr	w10, [sp, #16]
	cmp	w10, #2
	b.gt	.LBB19_1
	ldpsw	x12, x10, [sp, #16]
	ldr	w11, [sp, #12]
	smaddl	x10, w10, w8, x9
	add	w14, w11, #1
	add	w13, w12, #1
	stp	w14, w13, [sp, #12]
	str	w11, [x10, x12, lsl #2]
	b	.LBB19_4
.LBB19_6:
	ldp	w8, w9, [sp, #24]
	cmp	w8, #1
	ccmp	w9, #2, #0, eq
	ldp	w8, w9, [sp, #32]
	ccmp	w8, #3, #0, eq
	ccmp	w9, #4, #0, eq
	ldp	w8, w9, [sp, #40]
	ccmp	w8, #5, #0, eq
	ccmp	w9, #6, #0, eq
	cset	w0, eq
	add	sp, sp, #48
	ret
.Lfunc_end19:
	.size	test_multidim_row_major, .Lfunc_end19-test_multidim_row_major
	.cfi_endproc

	.globl	test_const_and_global_index
	.p2align	2
	.type	test_const_and_global_index,@function
test_const_and_global_index:
	.cfi_startproc
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	adrp	x8, :got:g_arr
	mov	w9, #3
	ldr	x8, [x8, :got_lo12:g_arr]
	ldr	w8, [x8, #8]
	cmp	w8, #5
	stp	w8, w9, [sp, #8]
	cset	w0, eq
	add	sp, sp, #16
	ret
.Lfunc_end20:
	.size	test_const_and_global_index, .Lfunc_end20-test_const_and_global_index
	.cfi_endproc

	.globl	integrated_test
	.p2align	2
	.type	integrated_test,@function
integrated_test:
	.cfi_startproc
	sub	sp, sp, #64
	stp	xzr, x30, [sp, #24]
	stp	x20, x19, [sp, #48]
	.cfi_def_cfa_offset 64
	.cfi_offset w19, -8
	.cfi_offset w20, -16
	.cfi_offset w30, -32
	add	x0, sp, #12
	mov	w1, #5
	mov	w2, #1
	stp	xzr, xzr, [sp, #8]
	add	x20, sp, #12
	str	xzr, [sp, #40]
	bl	fill_linear
	adrp	x19, :got:g_arr
	mov	w8, #4
	ldr	x19, [x19, :got_lo12:g_arr]
	str	wzr, [sp, #44]
.LBB21_1:
	ldr	w10, [sp, #44]
	ldr	w9, [sp, #8]
	cmp	w10, #4
	b.gt	.LBB21_3
	ldrsw	x10, [sp, #44]
	sub	w11, w8, w10
	ldr	w12, [x20, x10, lsl #2]
	add	w10, w10, #1
	ldr	w11, [x19, w11, sxtw #2]
	str	w10, [sp, #44]
	madd	w9, w12, w11, w9
	str	w9, [sp, #8]
	b	.LBB21_1
.LBB21_3:
	ldr	w20, [sp, #8]
	cmp	w9, #85
	b.ne	.LBB21_5
	mov	w0, #1
	mov	w1, #2
	mov	w2, #3
	bl	add3
	add	w8, w20, w0
	b	.LBB21_6
.LBB21_5:
	sub	w8, w20, #100
.LBB21_6:
	str	w8, [sp, #8]
.LBB21_7:
	ldr	w9, [sp, #40]
	ldr	w8, [sp, #8]
	cmp	w9, #2
	b.gt	.LBB21_9
	ldr	w9, [sp, #40]
	add	w9, w9, #1
	ldr	w10, [x19, w9, sxtw #2]
	str	w9, [sp, #40]
	add	w8, w8, w10
	str	w8, [sp, #8]
	b	.LBB21_7
.LBB21_9:
	ldp	x20, x19, [sp, #48]
	cmp	w8, #106
	ldr	x30, [sp, #32]
	cset	w0, eq
	add	sp, sp, #64
	ret
.Lfunc_end21:
	.size	integrated_test, .Lfunc_end21-integrated_test
	.cfi_endproc

	.globl	main
	.p2align	2
	.type	main,@function
main:
	.cfi_startproc
	str	x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	.cfi_offset w30, -16
	mov	w8, #16
	adrp	x0, :got:msg_begin
	stp	wzr, w8, [sp, #8]
	ldr	x0, [x0, :got_lo12:msg_begin]
	bl	putstr
	bl	test_arith_and_assign
	mov	w1, w0
	str	w0, [sp, #8]
	mov	w0, #1
	bl	print_case_result
	ldp	w9, w8, [sp, #8]
	sub	w8, w8, w9
	str	w8, [sp, #12]
	bl	test_unary_and_empty_stmt
	mov	w1, w0
	str	w0, [sp, #8]
	mov	w0, #2
	bl	print_case_result
	ldp	w9, w8, [sp, #8]
	sub	w8, w8, w9
	str	w8, [sp, #12]
	bl	test_if_else_logic
	mov	w1, w0
	str	w0, [sp, #8]
	mov	w0, #3
	bl	print_case_result
	ldp	w9, w8, [sp, #8]
	sub	w8, w8, w9
	str	w8, [sp, #12]
	bl	test_nested_branch
	mov	w1, w0
	str	w0, [sp, #8]
	mov	w0, #4
	bl	print_case_result
	ldp	w9, w8, [sp, #8]
	sub	w8, w8, w9
	str	w8, [sp, #12]
	bl	test_while_break_continue
	mov	w1, w0
	str	w0, [sp, #8]
	mov	w0, #5
	bl	print_case_result
	ldp	w9, w8, [sp, #8]
	sub	w8, w8, w9
	str	w8, [sp, #12]
	bl	test_for_zero_iteration
	mov	w1, w0
	str	w0, [sp, #8]
	mov	w0, #6
	bl	print_case_result
	ldp	w9, w8, [sp, #8]
	sub	w8, w8, w9
	str	w8, [sp, #12]
	bl	test_for_break_continue
	mov	w1, w0
	str	w0, [sp, #8]
	mov	w0, #7
	bl	print_case_result
	ldp	w9, w8, [sp, #8]
	sub	w8, w8, w9
	str	w8, [sp, #12]
	bl	test_array
	mov	w1, w0
	str	w0, [sp, #8]
	mov	w0, #8
	bl	print_case_result
	ldp	w9, w8, [sp, #8]
	sub	w8, w8, w9
	str	w8, [sp, #12]
	bl	test_prefix_postfix
	mov	w1, w0
	str	w0, [sp, #8]
	mov	w0, #9
	bl	print_case_result
	ldp	w9, w8, [sp, #8]
	sub	w8, w8, w9
	str	w8, [sp, #12]
	bl	test_scope_and_call
	mov	w1, w0
	str	w0, [sp, #8]
	mov	w0, #10
	bl	print_case_result
	ldp	w9, w8, [sp, #8]
	sub	w8, w8, w9
	str	w8, [sp, #12]
	bl	test_float_expr
	mov	w1, w0
	str	w0, [sp, #8]
	mov	w0, #11
	bl	print_case_result
	ldp	w9, w8, [sp, #8]
	sub	w8, w8, w9
	str	w8, [sp, #12]
	bl	test_float_arith_extended
	mov	w1, w0
	str	w0, [sp, #8]
	mov	w0, #12
	bl	print_case_result
	ldp	w9, w8, [sp, #8]
	sub	w8, w8, w9
	str	w8, [sp, #12]
	bl	test_forced_conversion
	mov	w1, w0
	str	w0, [sp, #8]
	mov	w0, #13
	bl	print_case_result
	ldp	w9, w8, [sp, #8]
	sub	w8, w8, w9
	str	w8, [sp, #12]
	bl	test_multidim_row_major
	mov	w1, w0
	str	w0, [sp, #8]
	mov	w0, #14
	bl	print_case_result
	ldp	w9, w8, [sp, #8]
	sub	w8, w8, w9
	str	w8, [sp, #12]
	bl	test_const_and_global_index
	mov	w1, w0
	str	w0, [sp, #8]
	mov	w0, #15
	bl	print_case_result
	ldp	w9, w8, [sp, #8]
	sub	w8, w8, w9
	str	w8, [sp, #12]
	bl	integrated_test
	mov	w1, w0
	str	w0, [sp, #8]
	mov	w0, #16
	bl	print_case_result
	ldp	w9, w8, [sp, #8]
	sub	w0, w8, w9
	str	w0, [sp, #12]
	ldr	x30, [sp], #16
	ret
.Lfunc_end22:
	.size	main, .Lfunc_end22-main
	.cfi_endproc

	.type	g_scalar,@object
	.data
	.globl	g_scalar
	.p2align	2, 0x0
g_scalar:
	.word	7
	.size	g_scalar, 4

	.type	g_const,@object
	.section	.rodata,"a",@progbits
	.globl	g_const
	.p2align	2, 0x0
g_const:
	.word	11
	.size	g_const, 4

	.type	g_arr,@object
	.data
	.globl	g_arr
	.p2align	4, 0x0
g_arr:
	.word	1
	.word	3
	.word	5
	.word	7
	.word	9
	.size	g_arr, 20

	.type	g_f,@object
	.globl	g_f
	.p2align	2, 0x0
g_f:
	.word	0x3fa00000
	.size	g_f, 4

	.type	g_fc,@object
	.section	.rodata,"a",@progbits
	.globl	g_fc
	.p2align	2, 0x0
g_fc:
	.word	0x3f400000
	.size	g_fc, 4

	.type	msg_begin,@object
	.data
	.globl	msg_begin
	.p2align	4, 0x0
msg_begin:
	.word	66
	.word	69
	.word	71
	.word	73
	.word	78
	.word	95
	.word	84
	.word	69
	.word	83
	.word	84
	.word	83
	.word	10
	.word	0
	.size	msg_begin, 52

	.type	msg_test,@object
	.globl	msg_test
	.p2align	4, 0x0
msg_test:
	.word	84
	.word	69
	.word	83
	.word	84
	.word	95
	.word	0
	.size	msg_test, 24

	.type	msg_pass,@object
	.globl	msg_pass
	.p2align	4, 0x0
msg_pass:
	.word	58
	.word	80
	.word	65
	.word	83
	.word	83
	.word	33
	.word	10
	.word	0
	.size	msg_pass, 32

	.type	msg_fail,@object
	.globl	msg_fail
	.p2align	4, 0x0
msg_fail:
	.word	58
	.word	70
	.word	65
	.word	73
	.word	76
	.word	33
	.word	10
	.word	0
	.size	msg_fail, 32

	.section	".note.GNU-stack","",@progbits
