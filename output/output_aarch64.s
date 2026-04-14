	.text
	.file	"sysy_module"
	.globl	add
	.p2align	2
	.type	add,@function
add:
	.cfi_startproc
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	stp	w1, w0, [sp, #8]
	add	w0, w0, w1
	add	sp, sp, #16
	ret
.Lfunc_end0:
	.size	add, .Lfunc_end0-add
	.cfi_endproc

	.globl	sum_array
	.p2align	2
	.type	sum_array,@function
sum_array:
	.cfi_startproc
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	stp	wzr, w1, [sp, #8]
	str	wzr, [sp, #4]
.LBB1_1:
	ldp	w9, w10, [sp, #8]
	ldr	w8, [sp, #4]
	cmp	w9, w10
	b.ge	.LBB1_3
	ldrsw	x9, [sp, #8]
	ldr	w10, [x0, x9, lsl #2]
	add	w9, w9, #1
	add	w8, w8, w10
	stp	w8, w9, [sp, #4]
	b	.LBB1_1
.LBB1_3:
	mov	w0, w8
	add	sp, sp, #16
	ret
.Lfunc_end1:
	.size	sum_array, .Lfunc_end1-sum_array
	.cfi_endproc

	.globl	fill
	.p2align	2
	.type	fill,@function
fill:
	.cfi_startproc
	str	x30, [sp, #-32]!
	stp	x20, x19, [sp, #16]
	.cfi_def_cfa_offset 32
	.cfi_offset w19, -8
	.cfi_offset w20, -16
	.cfi_offset w30, -32
	mov	x19, x0
	stp	wzr, w1, [sp, #8]
.LBB2_1:
	ldp	w8, w9, [sp, #8]
	cmp	w8, w9
	b.ge	.LBB2_7
	ldr	w20, [sp, #8]
	and	w8, w20, #0x1
	cmp	w20, #0
	cneg	w8, w8, lt
	cbnz	w8, .LBB2_4
	bl	getint
	b	.LBB2_5
.LBB2_4:
	sub	w8, w20, #1
	mov	w1, #1
	ldr	w0, [x19, w8, sxtw #2]
	bl	add
.LBB2_5:
	ldrsw	x8, [sp, #8]
	str	w0, [x19, w20, sxtw #2]
	ldr	w8, [x19, x8, lsl #2]
	cmp	w8, #100
	b.gt	.LBB2_7
	ldr	w8, [sp, #8]
	add	w8, w8, #1
	str	w8, [sp, #8]
	b	.LBB2_1
.LBB2_7:
	ldp	x20, x19, [sp, #16]
	ldr	x30, [sp], #32
	ret
.Lfunc_end2:
	.size	fill, .Lfunc_end2-fill
	.cfi_endproc

	.globl	main
	.p2align	2
	.type	main,@function
main:
	.cfi_startproc
	sub	sp, sp, #64
	stp	x30, x19, [sp, #48]
	.cfi_def_cfa_offset 64
	.cfi_offset w19, -8
	.cfi_offset w30, -16
	mov	x8, #4294967296
	mov	w10, #3
	mov	w9, #2
	stur	x8, [sp, #36]
	mov	w8, #1
	add	x0, sp, #36
	stur	x8, [sp, #20]
	mov	w8, #-1
	mov	w1, #3
	stp	w8, w10, [sp, #28]
	mov	w8, #4
	add	x19, sp, #36
	str	w9, [sp, #44]
	str	w8, [sp, #16]
	str	xzr, [sp, #8]
	bl	fill
	add	x8, sp, #20
	str	wzr, [sp, #12]
	b	.LBB3_2
.LBB3_1:
	ldr	w9, [sp, #12]
	add	w9, w9, #1
	str	w9, [sp, #12]
.LBB3_2:
	ldr	w9, [sp, #12]
	cmp	w9, #2
	b.gt	.LBB3_5
	ldrsw	x9, [sp, #12]
	ldr	w9, [x19, x9, lsl #2]
	cmp	w9, #10
	b.gt	.LBB3_1
	ldp	w10, w9, [sp, #8]
	sxtw	x9, w9
	lsl	x9, x9, #2
	ldr	w11, [x19, x9]
	ldr	w9, [x8, x9]
	add	w10, w10, w11
	add	w9, w10, w9
	add	w9, w9, #4
	str	w9, [sp, #8]
	b	.LBB3_1
.LBB3_5:
	ldr	w0, [sp, #8]
	bl	putint
	adrp	x8, :got:scale
	fmov	s0, #0.25000000
	ldr	x8, [x8, :got_lo12:scale]
	ldr	s1, [x8]
	fadd	s0, s1, s0
	bl	putfloat
	adrp	x0, :got:numbers
	mov	w1, #3
	ldr	x0, [x0, :got_lo12:numbers]
	bl	sum_array
	adrp	x8, :got:global_count
	ldr	x8, [x8, :got_lo12:global_count]
	ldp	x30, x19, [sp, #48]
	ldr	w8, [x8]
	add	w0, w0, w8
	add	sp, sp, #64
	ret
.Lfunc_end3:
	.size	main, .Lfunc_end3-main
	.cfi_endproc

	.type	global_count,@object
	.data
	.globl	global_count
	.p2align	2, 0x0
global_count:
	.word	3
	.size	global_count, 4

	.type	scale,@object
	.globl	scale
	.p2align	2, 0x0
scale:
	.word	0x3fc00000
	.size	scale, 4

	.type	max_value,@object
	.section	.rodata,"a",@progbits
	.globl	max_value
	.p2align	2, 0x0
max_value:
	.word	100
	.size	max_value, 4

	.type	default_bias,@object
	.globl	default_bias
	.p2align	2, 0x0
default_bias:
	.word	0x3e800000
	.size	default_bias, 4

	.type	offsets,@object
	.globl	offsets
	.p2align	2, 0x0
offsets:
	.word	2
	.word	4
	.word	6
	.size	offsets, 12

	.type	numbers,@object
	.data
	.globl	numbers
	.p2align	4, 0x0
numbers:
	.word	1
	.word	2
	.word	3
	.word	4
	.word	5
	.size	numbers, 20

	.section	".note.GNU-stack","",@progbits
