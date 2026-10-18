	.build_version macos, 11, 0
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math10remainderf10remainderf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math10remainderf10remainderf:
	.cfi_startproc
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math7remquof7remquof
	.cfi_endproc

	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math14rem_pio2_large14rem_pio2_large:
	.cfi_startproc
	stp	x28, x27, [sp, #-80]!
	.cfi_def_cfa_offset 80
	stp	x26, x25, [sp, #16]
	stp	x24, x23, [sp, #32]
	stp	x22, x21, [sp, #48]
	stp	x20, x19, [sp, #64]
	.cfi_offset w19, -8
	.cfi_offset w20, -16
	.cfi_offset w21, -24
	.cfi_offset w22, -32
	.cfi_offset w23, -40
	.cfi_offset w24, -48
	.cfi_offset w25, -56
	.cfi_offset w26, -64
	.cfi_offset w27, -72
	.cfi_offset w28, -80
	sub	sp, sp, #560
	.cfi_def_cfa_offset 640
	mov	x9, #0
	movi.2d	v0, #0000000000000000
	stp	q0, q0, [sp, #128]
	stp	q0, q0, [sp, #96]
	stp	q0, q0, [sp, #64]
	stp	q0, q0, [sp, #32]
	stp	q0, q0, [sp]
	stp	q0, q0, [sp, #288]
	stp	q0, q0, [sp, #256]
	stp	q0, q0, [sp, #224]
	stp	q0, q0, [sp, #192]
	stp	q0, q0, [sp, #160]
	stp	q0, q0, [sp, #448]
	stp	q0, q0, [sp, #416]
	stp	q0, q0, [sp, #384]
	stp	q0, q0, [sp, #352]
Lloh0:
	adrp	x8, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.13@PAGE
Lloh1:
	add	x8, x8, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.13@PAGEOFF
	ldr	x8, [x8, x4, lsl #3]
	mov	x22, x8
	sub	x12, x1, #1
	sub	w10, w3, #3
	stp	q0, q0, [sp, #320]
	sxth	w10, w10
	mov	w11, #10923
	stp	q0, q0, [sp, #528]
	mul	w10, w10, w11
	asr	w11, w10, #18
	add	w13, w11, w10, lsr #31
	stp	q0, q0, [sp, #496]
	sub	w10, w13, w12
	add	x11, x8, x12
	mov	x15, sp
Lloh2:
	adrp	x14, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.14@PAGE
Lloh3:
	add	x14, x14, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.14@PAGEOFF
	str	q0, [sp, #480]
LBB1_1:
	cmp	x9, x11
	cinc	x16, x9, lo
	tbnz	w10, #31, LBB1_3
	ldr	s0, [x14, w10, uxtw #2]
	sshll.2d	v0, v0, #0
	scvtf	d0, d0
	str	d0, [x15, x9, lsl #3]
	cmp	x9, x11
	b.lo	LBB1_4
	b	LBB1_5
LBB1_3:
	movi.2d	v0, #0000000000000000
	str	d0, [x15, x9, lsl #3]
	cmp	x9, x11
	b.hs	LBB1_5
LBB1_4:
	add	w10, w10, #1
	mov	x9, x16
	cmp	x16, x11
	b.ls	LBB1_1
LBB1_5:
	mov	x11, #0
	mov	x9, sp
	add	x10, sp, #320
LBB1_6:
	mov	x16, #0
	add	x15, x11, x12
	movi.2d	v0, #0000000000000000
LBB1_7:
	sub	x17, x15, x16
	ldr	d1, [x0, x16, lsl #3]
	ldr	d2, [x9, x17, lsl #3]
	fmul	d1, d1, d2
	fadd	d0, d0, d1
	cmp	x16, x12
	cinc	x17, x16, lo
	b.hs	LBB1_9
	mov	x16, x17
	cmp	x17, x12
	b.ls	LBB1_7
LBB1_9:
	str	d0, [x10, x11, lsl #3]
	cmp	x11, x8
	cinc	x15, x11, lo
	b.hs	LBB1_11
	mov	x11, x15
	cmp	x15, x8
	b.ls	LBB1_6
LBB1_11:
	lsl	w9, w13, #3
	sub	w16, w9, w13, lsl #5
	mov	w9, #48
	add	w10, w3, w16
	sub	w9, w9, w10
	sub	w15, w10, #24
	and	w17, w9, #0x1f
	mov	w9, #47
	sub	w9, w9, w10
	and	w1, w9, #0x1f
	add	w9, w10, #999
	lsl	x9, x9, #52
	fmov	d0, x9
	add	x5, sp, #480
	add	x9, x5, x8, lsl #2
	sub	x6, x9, #4
	add	x7, sp, #320
	fmov	d1, #0.12500000
	fmov	d2, #-8.00000000
	fmov	d3, #1.00000000
	mov	x19, sp
	fmov	d4, #0.50000000
	mov	x20, #4503599627370495
	mov	x21, #-4503599627370496
	movi.2d	v5, #0000000000000000
	fmov	d6, #-1.00000000
	mov	x9, #4499096027743125504
	fmov	d7, x9
	mov	x9, #-4508103226997866496
	fmov	d16, x9
LBB1_12:
	mov	x9, x22
	ldr	d17, [x7, x22, lsl #3]
	cbz	x22, LBB1_16
	mov	w10, #0
	mov	x11, x9
LBB1_14:
	fmul	d18, d17, d7
	fcvtzs	w22, d18
	scvtf	d18, w22
	fmul	d19, d18, d16
	fadd	d17, d17, d19
	fcvtzs	w22, d17
	str	w22, [x5, w10, sxtw #2]
	add	x22, x7, x11, lsl #3
	ldur	d17, [x22, #-8]
	fadd	d17, d17, d18
	subs	x11, x11, #1
	csinc	x11, x11, xzr, ne
	b.eq	LBB1_16
	add	w10, w10, #1
	cbnz	x11, LBB1_14
LBB1_16:
	fmul	d17, d17, d0
	fmul	d18, d17, d1
	fmov	x10, d18
	ubfx	x11, x10, #52, #11
	cmp	w11, #1074
	b.hi	LBB1_21
	cmp	w11, #1022
	b.ls	LBB1_20
	sub	w11, w11, #1023
	lsr	x22, x20, x11
	tst	x22, x10
	b.eq	LBB1_21
	and	x22, x22, x10, asr #63
	asr	x11, x21, x11
	add	x10, x22, x10
	and	x10, x10, x11
	fmov	d18, x10
	b	LBB1_21
LBB1_20:
	fcmp	d18, #0.0
	fcsel	d18, d18, d6, eq
	tst	x10, #0x8000000000000000
	fcsel	d18, d5, d18, eq
LBB1_21:
	fmul	d18, d18, d2
	fadd	d17, d17, d18
	fcvtzs	w10, d17
	scvtf	d18, w10
	fsub	d17, d17, d18
	cmp	w15, #0
	b.le	LBB1_31
	add	x11, x5, x9, lsl #2
	ldur	w22, [x11, #-4]
	asr	w23, w22, w17
	add	w10, w23, w10
	lsl	w23, w23, w17
	sub	w22, w22, w23
	stur	w22, [x11, #-4]
	asr	w11, w22, w1
	cmp	w11, #1
	b.lt	LBB1_45
LBB1_23:
	cbz	x9, LBB1_36
LBB1_24:
	mov	w25, #0
	add	x23, sp, #480
	mov	x24, x9
	b	LBB1_27
LBB1_25:
	mov	w25, #16777215
LBB1_26:
	mov	w22, #0
	sub	w25, w25, w26
	str	w25, [x23], #4
	mov	w25, #1
	subs	x24, x24, #1
	b.eq	LBB1_37
LBB1_27:
	ldr	w26, [x23]
	cbnz	w25, LBB1_25
	cbz	w26, LBB1_30
	mov	w25, #16777216
	b	LBB1_26
LBB1_30:
	mov	w25, #0
	mov	w22, #1
	add	x23, x23, #4
	subs	x24, x24, #1
	b.ne	LBB1_27
	b	LBB1_37
LBB1_31:
	cbz	w15, LBB1_34
	fcmp	d17, d4
	b.ge	LBB1_35
	mov	w11, #0
	fcmp	d17, #0.0
	b.eq	LBB1_47
	b	LBB1_60
LBB1_34:
	add	x11, x5, x9, lsl #2
	ldur	w11, [x11, #-4]
	asr	w11, w11, #23
	cmp	w11, #1
	b.ge	LBB1_23
	b	LBB1_45
LBB1_35:
	mov	w11, #2
	cbnz	x9, LBB1_24
LBB1_36:
	mov	w22, #1
LBB1_37:
	cmp	w15, #0
	b.le	LBB1_43
	cmp	w15, #2
	b.eq	LBB1_41
	cmp	w15, #1
	b.ne	LBB1_43
	mov	w23, #8388607
	b	LBB1_42
LBB1_41:
	mov	w23, #4194303
LBB1_42:
	add	x24, x5, x9, lsl #2
	ldur	w25, [x24, #-4]
	and	w23, w25, w23
	stur	w23, [x24, #-4]
LBB1_43:
	add	w10, w10, #1
	cmp	w11, #2
	b.ne	LBB1_46
	fsub	d17, d3, d17
	fsub	d18, d17, d0
	cmp	w22, #0
	fcsel	d17, d17, d18, ne
LBB1_45:
	fcmp	d17, #0.0
	b.eq	LBB1_47
	b	LBB1_60
LBB1_46:
	mov	w11, #2
	fcmp	d17, #0.0
	b.ne	LBB1_60
LBB1_47:
	sub	x24, x9, #1
	mov	x25, x6
	mov	x22, x9
	cmp	x8, x24
	b.hi	LBB1_52
	mov	w23, #0
LBB1_49:
	ldr	w22, [x5, x24, lsl #2]
	orr	w23, w22, w23
	cmp	x8, x24
	sbc	x22, x24, xzr
	b.hs	LBB1_51
	mov	x24, x22
	cmp	x8, x22
	b.ls	LBB1_49
LBB1_51:
	mov	x25, x6
	mov	x22, x9
	cbnz	w23, LBB1_63
LBB1_52:
	ldr	w10, [x25], #-4
	add	x22, x22, #1
	cbz	w10, LBB1_52
	cmp	x9, x22
	b.hs	LBB1_12
	add	x9, x9, #1
LBB1_55:
	mov	x11, #0
	add	x10, x9, x13
	ldr	s17, [x14, x10, lsl #2]
	add	x10, x9, x12
	sshll.2d	v17, v17, #0
	scvtf	d17, d17
	str	d17, [x19, x10, lsl #3]
	movi.2d	v17, #0000000000000000
LBB1_56:
	sub	x23, x10, x11
	ldr	d18, [x0, x11, lsl #3]
	ldr	d19, [x19, x23, lsl #3]
	fmul	d18, d18, d19
	fadd	d17, d17, d18
	cmp	x11, x12
	cinc	x23, x11, lo
	b.hs	LBB1_58
	mov	x11, x23
	cmp	x23, x12
	b.ls	LBB1_56
LBB1_58:
	str	d17, [x7, x9, lsl #3]
	cmp	x9, x22
	cinc	x10, x9, lo
	b.hs	LBB1_12
	mov	x9, x10
	cmp	x10, x22
	b.ls	LBB1_55
	b	LBB1_12
LBB1_60:
	mov	w12, #1023
	sub	w12, w12, w15
	lsl	x12, x12, #52
	fmov	d0, x12
	fmul	d0, d17, d0
	mov	x12, #4715268809856909312
	fmov	d1, x12
	fcmp	d0, d1
	b.lt	LBB1_62
	mov	x12, #4499096027743125504
	fmov	d1, x12
	fmul	d1, d0, d1
	fcvtzs	w12, d1
	scvtf	d1, w12
	mov	x12, #-4508103226997866496
	fmov	d2, x12
	fmul	d2, d1, d2
	fadd	d0, d0, d2
	fcvtzs	w12, d0
	add	x13, sp, #480
	str	w12, [x13, x9, lsl #2]
	add	x9, x9, #1
	add	w15, w16, w3
	mov.16b	v0, v1
LBB1_62:
	fcvtzs	w12, d0
	add	x13, sp, #480
	str	w12, [x13, x9, lsl #2]
	b	LBB1_65
LBB1_63:
	add	x12, sp, #480
	sub	x12, x12, #4
LBB1_64:
	sub	w15, w15, #24
	ldr	w13, [x12, x9, lsl #2]
	sub	x9, x9, #1
	cbz	w13, LBB1_64
LBB1_65:
	sub	w12, w15, #1023
	mov	x13, #9214364837600034816
	fmov	d0, x13
	mov	w13, #3069
	cmp	w15, #3069
	csel	w13, w15, w13, lo
	sub	w13, w13, #2046
	mov	x14, #9218868437227405312
	fmov	d1, x14
	cmp	w15, #2046
	csel	w12, w13, w12, hi
	fcsel	d0, d1, d0, hi
	fmov	d1, #1.00000000
	add	w13, w15, #969
	mov	x14, #243194379878006784
	fmov	d2, x14
	cmn	w15, #2960
	mov	w14, #-2960
	csel	w14, w15, w14, hi
	add	w14, w14, #1938
	movi.2d	v3, #0000000000000000
	cmn	w15, #1992
	fcsel	d2, d2, d3, hi
	csel	w13, w13, w14, hi
	cmn	w15, #1022
	fcsel	d1, d2, d1, lt
	csel	w13, w13, w15, lt
	cmp	w15, #1023
	csel	w12, w12, w13, gt
	fcsel	d0, d0, d1, gt
	add	w12, w12, #1023
	lsl	x12, x12, #52
	fmov	d1, x12
	fmul	d0, d0, d1
	add	x12, sp, #480
	add	x13, sp, #320
	mov	x14, #4499096027743125504
	fmov	d1, x14
	mov	x14, x9
LBB1_66:
	ldr	s2, [x12, x14, lsl #2]
	sshll.2d	v2, v2, #0
	scvtf	d2, d2
	fmul	d2, d0, d2
	str	d2, [x13, x14, lsl #3]
	sub	x14, x14, #1
	fmul	d0, d0, d1
	cmn	x14, #1
	b.ne	LBB1_66
	add	x12, sp, #320
	add	x12, x12, x9, lsl #3
	add	x13, x12, #32
Lloh4:
	adrp	x14, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.15@PAGE
Lloh5:
	add	x14, x14, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.15@PAGEOFF
	add	x15, sp, #160
Lloh6:
	adrp	x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.15@PAGE+32
Lloh7:
	add	x16, x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.15@PAGEOFF+32
	mov	x1, x9
	b	LBB1_69
LBB1_68:
	sub	x1, x17, #1
	str	d0, [x15, x0, lsl #3]
	sub	x13, x13, #8
	sub	x12, x12, #8
	cbz	x17, LBB1_75
LBB1_69:
	mov	x17, x1
	sub	x0, x9, x1
	cmp	x8, x0
	csel	x1, x8, x0, lo
	add	x1, x1, #1
	cmp	x1, #7
	b.hi	LBB1_71
	mov	x3, #0
	movi.2d	v0, #0000000000000000
	b	LBB1_73
LBB1_71:
	and	x3, x1, #0xfffffffffffffff8
	movi.2d	v0, #0000000000000000
	mov	x5, x13
	mov	x6, x16
	and	x7, x1, #0xfffffffffffffff8
LBB1_72:
	ldp	q1, q2, [x6, #-32]
	ldp	q3, q4, [x6], #64
	ldp	q5, q6, [x5, #-32]
	ldp	q7, q16, [x5], #64
	fmul.2d	v1, v1, v5
	mov	d5, v1[1]
	fmul.2d	v2, v2, v6
	mov	d6, v2[1]
	fmul.2d	v3, v3, v7
	mov	d7, v3[1]
	fmul.2d	v4, v4, v16
	mov	d16, v4[1]
	fadd	d0, d0, d1
	fadd	d0, d0, d5
	fadd	d0, d0, d2
	fadd	d0, d0, d6
	fadd	d0, d0, d3
	fadd	d0, d0, d7
	fadd	d0, d0, d4
	fadd	d0, d0, d16
	subs	x7, x7, #8
	b.ne	LBB1_72
	b	LBB1_74
LBB1_73:
	ldr	d1, [x14, x3, lsl #3]
	ldr	d2, [x12, x3, lsl #3]
	fmul	d1, d1, d2
	fadd	d0, d0, d1
	add	x3, x3, #1
LBB1_74:
	cmp	x1, x3
	b.ne	LBB1_73
	b	LBB1_68
LBB1_75:
	add	x8, x9, #1
	movi.2d	v0, #0000000000000000
	cbz	x4, LBB1_87
	mov	x12, x9
	cmp	x8, #3
	b.ls	LBB1_80
	and	x13, x8, #0xfffffffffffffffc
	sub	x12, x9, x13
	add	x14, sp, #160
	add	x14, x14, x9, lsl #3
	sub	x14, x14, #8
	and	x15, x8, #0xfffffffffffffffc
LBB1_78:
	ldp	d2, d1, [x14]
	ldp	d4, d3, [x14, #-16]
	fadd	d0, d0, d1
	fadd	d0, d0, d2
	fadd	d0, d0, d3
	fadd	d0, d0, d4
	sub	x14, x14, #32
	sub	x15, x15, #4
	cbnz	x15, LBB1_78
	cmp	x8, x13
	b.eq	LBB1_82
LBB1_80:
	add	x8, sp, #160
LBB1_81:
	ldr	d1, [x8, x12, lsl #3]
	sub	x12, x12, #1
	fadd	d0, d0, d1
	cmn	x12, #1
	b.ne	LBB1_81
LBB1_82:
	fneg	d1, d0
	cmp	w11, #0
	fcsel	d1, d0, d1, eq
	str	d1, [x2]
	ldr	d1, [sp, #160]
	fsub	d0, d1, d0
	cbz	x9, LBB1_86
	mov	w12, #1
	add	x8, sp, #160
LBB1_84:
	ldr	d1, [x8, x12, lsl #3]
	fadd	d0, d0, d1
	cmp	x12, x9
	cinc	x13, x12, lo
	b.hs	LBB1_86
	mov	x12, x13
	cmp	x13, x9
	b.ls	LBB1_84
LBB1_86:
	fneg	d1, d0
	cmp	w11, #0
	fcsel	d0, d0, d1, eq
	str	d0, [x2, #8]
	b	LBB1_95
LBB1_87:
	cmp	x8, #3
	b.hi	LBB1_89
	mov	x12, x9
	b	LBB1_92
LBB1_89:
	and	x13, x8, #0xfffffffffffffffc
	sub	x12, x9, x13
	add	x14, sp, #160
	add	x9, x14, x9, lsl #3
	sub	x9, x9, #8
	and	x14, x8, #0xfffffffffffffffc
LBB1_90:
	ldp	d2, d1, [x9]
	ldp	d4, d3, [x9, #-16]
	fadd	d0, d0, d1
	fadd	d0, d0, d2
	fadd	d0, d0, d3
	fadd	d0, d0, d4
	sub	x9, x9, #32
	sub	x14, x14, #4
	cbnz	x14, LBB1_90
	cmp	x8, x13
	b.eq	LBB1_94
LBB1_92:
	add	x8, sp, #160
LBB1_93:
	ldr	d1, [x8, x12, lsl #3]
	sub	x12, x12, #1
	fadd	d0, d0, d1
	cmn	x12, #1
	b.ne	LBB1_93
LBB1_94:
	fneg	d1, d0
	cmp	w11, #0
	fcsel	d0, d0, d1, eq
	str	d0, [x2]
LBB1_95:
	and	w0, w10, #0x7
	add	sp, sp, #560
	.cfi_def_cfa_offset 80
	ldp	x20, x19, [sp, #64]
	ldp	x22, x21, [sp, #48]
	ldp	x24, x23, [sp, #32]
	ldp	x26, x25, [sp, #16]
	ldp	x28, x27, [sp], #80
	.cfi_def_cfa_offset 0
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	.cfi_restore w23
	.cfi_restore w24
	.cfi_restore w25
	.cfi_restore w26
	.cfi_restore w27
	.cfi_restore w28
	ret
	.loh AdrpAdd	Lloh2, Lloh3
	.loh AdrpAdd	Lloh0, Lloh1
	.loh AdrpAdd	Lloh6, Lloh7
	.loh AdrpAdd	Lloh4, Lloh5
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math17fminimum_fmaximum8fmaximum
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math17fminimum_fmaximum8fmaximum:
	.cfi_startproc
	fcmp	d0, d0
	b.vs	LBB2_5
	fcmp	d1, d1
	b.vs	LBB2_6
	fcmp	d0, d1
	b.gt	LBB2_5
	fmov	x8, d1
	mov	x9, #-9223372036854775808
	cmp	x8, x9
	b.ne	LBB2_6
	fmov	x8, d0
	tbnz	x8, #63, LBB2_6
LBB2_5:
	mov.16b	v1, v0
LBB2_6:
	mov.16b	v0, v1
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math17fminimum_fmaximum8fminimum
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math17fminimum_fmaximum8fminimum:
	.cfi_startproc
	fcmp	d0, d0
	b.vs	LBB3_5
	fcmp	d1, d1
	b.vs	LBB3_6
	fcmp	d0, d1
	b.mi	LBB3_5
	fmov	x8, d0
	mov	x9, #-9223372036854775808
	cmp	x8, x9
	b.ne	LBB3_6
	fmov	x8, d1
	tbnz	x8, #63, LBB3_6
LBB3_5:
	mov.16b	v1, v0
LBB3_6:
	mov.16b	v0, v1
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math17fminimum_fmaximum9fmaximumf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math17fminimum_fmaximum9fmaximumf:
	.cfi_startproc
	fcmp	s0, s0
	b.vs	LBB4_5
	fcmp	s1, s1
	b.vs	LBB4_6
	fcmp	s0, s1
	b.gt	LBB4_5
	fmov	w8, s1
	mov	w9, #-2147483648
	cmp	w8, w9
	b.ne	LBB4_6
	fmov	w8, s0
	tbnz	w8, #31, LBB4_6
LBB4_5:
	mov.16b	v1, v0
LBB4_6:
	mov.16b	v0, v1
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math17fminimum_fmaximum9fminimumf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math17fminimum_fmaximum9fminimumf:
	.cfi_startproc
	fcmp	s0, s0
	b.vs	LBB5_5
	fcmp	s1, s1
	b.vs	LBB5_6
	fcmp	s0, s1
	b.mi	LBB5_5
	fmov	w8, s0
	mov	w9, #-2147483648
	cmp	w8, w9
	b.ne	LBB5_6
	fmov	w8, s1
	tbnz	w8, #31, LBB5_6
LBB5_5:
	mov.16b	v1, v0
LBB5_6:
	mov.16b	v0, v1
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math21fminimum_fmaximum_num12fmaximum_num
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math21fminimum_fmaximum_num12fmaximum_num:
	.cfi_startproc
	fcmp	d0, d1
	fccmp	d1, d1, #1, le
	b.vs	LBB6_4
	fcmp	d0, d0
	fccmp	d1, d0, #0, vc
	b.le	LBB6_3
	mov.16b	v0, v1
	ret
LBB6_3:
	fmov	x8, d0
	cmp	x8, #0
	fcsel	d0, d1, d0, mi
LBB6_4:
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math21fminimum_fmaximum_num12fminimum_num
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math21fminimum_fmaximum_num12fminimum_num:
	.cfi_startproc
	fcmp	d0, d0
	fccmp	d0, d1, #0, vc
	b.gt	LBB7_4
	fcmp	d1, d0
	fccmp	d1, d1, #1, le
	b.vc	LBB7_3
	ret
LBB7_3:
	fmov	x8, d0
	cmp	x8, #0
	fcsel	d1, d0, d1, mi
LBB7_4:
	mov.16b	v0, v1
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math21fminimum_fmaximum_num13fmaximum_numf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math21fminimum_fmaximum_num13fmaximum_numf:
	.cfi_startproc
	fcmp	s0, s1
	fccmp	s1, s1, #1, le
	b.vs	LBB8_4
	fcmp	s0, s0
	fccmp	s1, s0, #0, vc
	b.le	LBB8_3
	mov.16b	v0, v1
	ret
LBB8_3:
	fmov	w8, s0
	cmp	w8, #0
	fcsel	s0, s1, s0, mi
LBB8_4:
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math21fminimum_fmaximum_num13fminimum_numf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math21fminimum_fmaximum_num13fminimum_numf:
	.cfi_startproc
	fcmp	s0, s0
	fccmp	s0, s1, #0, vc
	b.gt	LBB9_4
	fcmp	s1, s0
	fccmp	s1, s1, #1, le
	b.vc	LBB9_3
	ret
LBB9_3:
	fmov	w8, s0
	cmp	w8, #0
	fcsel	s1, s0, s1, mi
LBB9_4:
	mov.16b	v0, v1
	ret
	.cfi_endproc

	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0
lCPI10_0:
	.quad	0xbf28e6a5b61ac6e9
	.quad	0x3f8ffce882c8c2a4
lCPI10_1:
	.quad	0x3f8ffffffffffffd
	.quad	0x3ff0000000000000
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math2j02j0
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math2j02j0:
	.cfi_startproc
	fmov	x8, d0
	lsr	x9, x8, #32
	ubfx	w9, w9, #20, #11
	cmp	w9, #2046
	b.ls	LBB10_2
	fmul	d0, d0, d0
	fmov	d1, #1.00000000
	fdiv	d0, d1, d0
	ret
LBB10_2:
	ubfx	x0, x8, #32, #31
	mov.16b	v1, v0
	fabs	d0, d0
	lsr	w8, w0, #30
	cbz	w8, LBB10_4
	mov	w1, #0
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math2j06common
LBB10_4:
	lsr	w8, w0, #21
	cmp	w8, #504
	b.ls	LBB10_6
	fmul	d1, d1, d1
	mov	x8, #23951
	movk	x8, #62580, lsl #16
	movk	x8, #2236, lsl #32
	movk	x8, #15892, lsl #48
	fmov	d2, x8
	fmul	d2, d1, d2
	mov	x8, #54697
	movk	x8, #52868, lsl #16
	movk	x8, #15188, lsl #32
	movk	x8, #16033, lsl #48
	fmov	d3, x8
	fadd	d2, d2, d3
	mov	x8, #16334
	movk	x8, #29654, lsl #16
	movk	x8, #54759, lsl #32
	movk	x8, #15923, lsl #48
	fmov	d3, x8
	dup.2d	v3, v3[0]
	mov.d	v3[1], v2[0]
	fmul.2d	v2, v3, v1[0]
	mov	x8, #14617
	movk	x8, #3152, lsl #16
	movk	x8, #45521, lsl #32
	movk	x8, #16062, lsl #48
	fmov	d3, x8
	fsub.2d	v3, v3, v2
	mov	x8, #56308
	movk	x8, #56663, lsl #16
	movk	x8, #42706, lsl #32
	movk	x8, #16158, lsl #48
	dup.2d	v4, x8
	fadd.2d	v2, v2, v4
	mov.d	v3[1], v2[1]
	fmul.2d	v2, v3, v1[0]
Lloh8:
	adrp	x8, lCPI10_0@PAGE
Lloh9:
	ldr	q3, [x8, lCPI10_0@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul.2d	v2, v2, v1[0]
Lloh10:
	adrp	x8, lCPI10_1@PAGE
Lloh11:
	ldr	q3, [x8, lCPI10_1@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul	d3, d1, d2
	fmov	d4, #0.50000000
	fmul	d0, d0, d4
	fmov	d4, #1.00000000
	fadd	d5, d0, d4
	fsub	d0, d4, d0
	fmul	d0, d5, d0
	mov	d2, v2[1]
	fdiv	d2, d3, d2
	fmul	d1, d1, d2
	fadd	d0, d0, d1
	ret
LBB10_6:
	fmov	d1, #0.25000000
	fmul	d1, d0, d1
	fmul	d1, d0, d1
	lsr	w8, w0, #27
	cmp	w8, #6
	fcsel	d0, d1, d0, hi
	fmov	d1, #1.00000000
	fsub	d0, d1, d0
	ret
	.loh AdrpLdr	Lloh10, Lloh11
	.loh AdrpAdrp	Lloh8, Lloh10
	.loh AdrpLdr	Lloh8, Lloh9
	.cfi_endproc

	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0
lCPI11_0:
	.quad	0xbf28e6a5b61ac6e9
	.quad	0x3f8ffce882c8c2a4
lCPI11_1:
	.quad	0x3f8ffffffffffffd
	.quad	0x3ff0000000000000
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math2j02y0
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math2j02y0:
	.cfi_startproc
	stp	d9, d8, [sp, #-32]!
	.cfi_def_cfa_offset 32
	stp	x29, x30, [sp, #16]
	add	x29, sp, #16
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset b8, -24
	.cfi_offset b9, -32
	.cfi_remember_state
	fmov	x8, d0
	lsr	x0, x8, #32
	orr	w9, w8, w0, lsl #1
	cbz	w9, LBB11_4
	tbnz	x8, #63, LBB11_5
	lsr	x9, x8, #52
	cmp	x9, #2046
	b.ls	LBB11_8
	fmov	d1, #1.00000000
	fdiv	d0, d1, d0
	b	LBB11_7
LBB11_4:
	mov	x8, #-4503599627370496
	b	LBB11_6
LBB11_5:
	mov	x8, #9221120237041090560
LBB11_6:
	fmov	d0, x8
LBB11_7:
	.cfi_def_cfa wsp, 32
	ldp	x29, x30, [sp, #16]
	ldp	d9, d8, [sp], #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore b8
	.cfi_restore b9
	ret
LBB11_8:
	.cfi_restore_state
	.cfi_remember_state
	lsr	x9, x8, #62
	cbz	x9, LBB11_10
	mov	w1, #1
	.cfi_def_cfa wsp, 32
	ldp	x29, x30, [sp, #16]
	ldp	d9, d8, [sp], #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore b8
	.cfi_restore b9
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math2j06common
LBB11_10:
	.cfi_restore_state
	lsr	x9, x8, #54
	cmp	x9, #248
	b.ls	LBB11_13
	fmul	d1, d0, d0
	mov	x9, #46024
	movk	x9, #26943, lsl #16
	movk	x9, #58429, lsl #32
	movk	x9, #48581, lsl #48
	fmov	d2, x9
	fmul	d2, d1, d2
	mov	x9, #43988
	movk	x9, #15182, lsl #16
	movk	x9, #87, lsl #32
	movk	x9, #15957, lsl #48
	fmov	d3, x9
	fadd	d2, d2, d3
	fmul	d2, d1, d2
	mov	x9, #23725
	movk	x9, #29650, lsl #16
	movk	x9, #65191, lsl #32
	movk	x9, #48847, lsl #48
	fmov	d3, x9
	fadd	d2, d2, d3
	fmul	d2, d1, d2
	mov	x9, #39787
	movk	x9, #8370, lsl #16
	movk	x9, #50509, lsl #32
	movk	x9, #16182, lsl #48
	fmov	d3, x9
	fadd	d2, d2, d3
	fmul	d2, d1, d2
	mov	x9, #64151
	movk	x9, #45420, lsl #16
	movk	x9, #19688, lsl #32
	movk	x9, #49036, lsl #48
	fmov	d3, x9
	fadd	d2, d2, d3
	fmul	d2, d1, d2
	mov	x9, #58364
	movk	x9, #40425, lsl #16
	movk	x9, #40193, lsl #32
	movk	x9, #16326, lsl #48
	fmov	d3, x9
	fadd	d2, d2, d3
	fmul	d2, d1, d2
	mov	x9, #53279
	movk	x9, #39371, lsl #16
	movk	x9, #58582, lsl #32
	movk	x9, #49074, lsl #48
	fmov	d3, x9
	fadd	d3, d2, d3
	mov	x9, #55791
	movk	x9, #15318, lsl #16
	movk	x9, #20504, lsl #32
	movk	x9, #15870, lsl #48
	fmov	d2, x9
	fmul	d2, d1, d2
	mov	x9, #765
	movk	x9, #32754, lsl #16
	movk	x9, #25645, lsl #32
	movk	x9, #16017, lsl #48
	fmov	d4, x9
	fadd	d2, d2, d4
	fmul	d2, d1, d2
	mov	x9, #50881
	movk	x9, #62840, lsl #16
	movk	x9, #60603, lsl #32
	movk	x9, #16147, lsl #48
	fmov	d4, x9
	fadd	d2, d2, d4
	fmul	d2, d1, d2
	mov	x9, #50970
	movk	x9, #37321, lsl #16
	movk	x9, #4720, lsl #32
	movk	x9, #16266, lsl #48
	fmov	d4, x9
	fadd	d2, d2, d4
	fmul	d4, d1, d2
	fmov	d2, #1.00000000
	fadd	d4, d4, d2
	fdiv	d8, d3, d4
	lsr	x8, x8, #53
	cmp	x8, #504
	b.ls	LBB11_14
	mov	x8, #23951
	movk	x8, #62580, lsl #16
	movk	x8, #2236, lsl #32
	movk	x8, #15892, lsl #48
	fmov	d3, x8
	fmul	d3, d1, d3
	mov	x8, #54697
	movk	x8, #52868, lsl #16
	movk	x8, #15188, lsl #32
	movk	x8, #16033, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	mov	x8, #16334
	movk	x8, #29654, lsl #16
	movk	x8, #54759, lsl #32
	movk	x8, #15923, lsl #48
	fmov	d4, x8
	dup.2d	v4, v4[0]
	mov.d	v4[1], v3[0]
	fmul.2d	v3, v4, v1[0]
	mov	x8, #14617
	movk	x8, #3152, lsl #16
	movk	x8, #45521, lsl #32
	movk	x8, #16062, lsl #48
	fmov	d4, x8
	fsub.2d	v4, v4, v3
	mov	x8, #56308
	movk	x8, #56663, lsl #16
	movk	x8, #42706, lsl #32
	movk	x8, #16158, lsl #48
	dup.2d	v5, x8
	fadd.2d	v3, v3, v5
	mov.d	v4[1], v3[1]
	fmul.2d	v3, v4, v1[0]
Lloh12:
	adrp	x8, lCPI11_0@PAGE
Lloh13:
	ldr	q4, [x8, lCPI11_0@PAGEOFF]
	fadd.2d	v3, v3, v4
	fmul.2d	v3, v3, v1[0]
Lloh14:
	adrp	x8, lCPI11_1@PAGE
Lloh15:
	ldr	q4, [x8, lCPI11_1@PAGEOFF]
	fadd.2d	v3, v3, v4
	fmul	d4, d1, d3
	fmov	d5, #0.50000000
	fmul	d5, d0, d5
	fadd	d6, d5, d2
	fsub	d2, d2, d5
	fmul	d2, d6, d2
	mov	d3, v3[1]
	fdiv	d3, d4, d3
	fmul	d1, d1, d3
	fadd	d9, d2, d1
	b	LBB11_15
LBB11_13:
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3log3log
	mov	x8, #51331
	movk	x8, #28105, lsl #16
	movk	x8, #24368, lsl #32
	movk	x8, #16356, lsl #48
	fmov	d1, x8
	fmul	d0, d0, d1
	mov	x8, #53279
	movk	x8, #39371, lsl #16
	movk	x8, #58582, lsl #32
	movk	x8, #49074, lsl #48
	fmov	d1, x8
	fadd	d0, d0, d1
	b	LBB11_7
LBB11_14:
	fmov	d1, #-0.25000000
	fmul	d1, d0, d1
	fmul	d1, d0, d1
	fadd	d9, d1, d2
LBB11_15:
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3log3log
	fmul	d0, d9, d0
	mov	x8, #51331
	movk	x8, #28105, lsl #16
	movk	x8, #24368, lsl #32
	movk	x8, #16356, lsl #48
	fmov	d1, x8
	fmul	d0, d0, d1
	fadd	d0, d8, d0
	b	LBB11_7
	.loh AdrpLdr	Lloh14, Lloh15
	.loh AdrpAdrp	Lloh12, Lloh14
	.loh AdrpLdr	Lloh12, Lloh13
	.cfi_endproc

	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math2j06common:
	.cfi_startproc
	stp	d11, d10, [sp, #-64]!
	.cfi_def_cfa_offset 64
	stp	d9, d8, [sp, #16]
	stp	x20, x19, [sp, #32]
	stp	x29, x30, [sp, #48]
	add	x29, sp, #48
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset b8, -40
	.cfi_offset b9, -48
	.cfi_offset b10, -56
	.cfi_offset b11, -64
	mov	x19, x1
	mov.16b	v8, v0
	mov	x20, x0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3sin3sin
	mov.16b	v9, v0
	mov.16b	v0, v8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3cos3cos
	fneg	d1, d0
	cmp	w19, #0
	fcsel	d11, d1, d0, ne
	fadd	d10, d9, d11
	lsr	w8, w20, #21
	cmp	w8, #1023
	b.hs	LBB12_5
	fadd	d0, d8, d8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3cos3cos
	fneg	d1, d0
	fmul	d0, d9, d11
	fcmp	d0, #0.0
	b.pl	LBB12_3
	fsub	d0, d9, d11
	fdiv	d10, d1, d0
	lsr	w8, w20, #27
	cmp	w8, #8
	b.ls	LBB12_4
	b	LBB12_5
LBB12_3:
	fdiv	d0, d1, d10
	lsr	w8, w20, #27
	cmp	w8, #8
	b.hi	LBB12_5
LBB12_4:
	mov	w8, #11914
	movk	w8, #16402, lsl #16
	fneg	d1, d0
	cmp	w19, #0
	fcsel	d0, d1, d0, ne
	fmov	x9, d8
	lsr	x10, x9, #32
Lloh16:
	adrp	x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.22@PAGE
Lloh17:
	add	x11, x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.22@PAGEOFF
Lloh18:
	adrp	x12, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.23@PAGE
Lloh19:
	add	x12, x12, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.23@PAGEOFF
	ubfx	x9, x9, #32, #31
	ubfx	w10, w10, #21, #10
Lloh20:
	adrp	x13, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.20@PAGE
Lloh21:
	add	x13, x13, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.20@PAGEOFF
Lloh22:
	adrp	x14, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.21@PAGE
Lloh23:
	add	x14, x14, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.21@PAGEOFF
Lloh24:
	adrp	x15, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.17@PAGE
Lloh25:
	add	x15, x15, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.17@PAGEOFF
Lloh26:
	adrp	x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.19@PAGE
Lloh27:
	add	x16, x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.19@PAGEOFF
	mov	w17, #56172
	movk	w17, #16390, lsl #16
	cmp	w9, w17
	csel	x15, x16, x15, hi
Lloh28:
	adrp	x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.16@PAGE
Lloh29:
	add	x16, x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.16@PAGEOFF
Lloh30:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.18@PAGE
Lloh31:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.18@PAGEOFF
	csel	x16, x0, x16, hi
	cmp	w9, w8
	csel	x14, x14, x15, hi
	csel	x13, x13, x16, hi
	cmp	w10, #512
	csel	x10, x12, x14, hi
	csel	x11, x11, x13, hi
	fmul	d2, d8, d8
	fmov	d1, #1.00000000
	fdiv	d2, d1, d2
	ldp	d3, d4, [x11]
	ldp	d5, d6, [x11, #16]
	ldp	d7, d16, [x11, #32]
	fmul	d16, d2, d16
	fadd	d7, d7, d16
	fmul	d7, d2, d7
	fadd	d6, d6, d7
	fmul	d6, d2, d6
	fadd	d5, d5, d6
	fmul	d5, d2, d5
	fadd	d4, d4, d5
	fmul	d4, d2, d4
	fadd	d3, d3, d4
	ldp	d4, d5, [x10]
	ldp	d6, d7, [x10, #16]
	ldr	d16, [x10, #32]
	fmul	d16, d2, d16
	fadd	d7, d7, d16
	fmul	d7, d2, d7
	fadd	d6, d6, d7
	fmul	d6, d2, d6
	fadd	d5, d5, d6
	fmul	d5, d2, d5
	fadd	d4, d4, d5
	fmul	d4, d2, d4
	fadd	d4, d4, d1
	fdiv	d3, d3, d4
	fadd	d3, d3, d1
	fmul	d3, d10, d3
Lloh32:
	adrp	x10, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.30@PAGE
Lloh33:
	add	x10, x10, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.30@PAGEOFF
Lloh34:
	adrp	x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.31@PAGE
Lloh35:
	add	x11, x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.31@PAGEOFF
	lsr	w12, w9, #21
Lloh36:
	adrp	x13, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.28@PAGE
Lloh37:
	add	x13, x13, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.28@PAGEOFF
Lloh38:
	adrp	x14, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.29@PAGE
Lloh39:
	add	x14, x14, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.29@PAGEOFF
Lloh40:
	adrp	x15, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.25@PAGE
Lloh41:
	add	x15, x15, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.25@PAGEOFF
Lloh42:
	adrp	x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.27@PAGE
Lloh43:
	add	x16, x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.27@PAGEOFF
	cmp	w9, w17
	csel	x15, x16, x15, hi
Lloh44:
	adrp	x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.24@PAGE
Lloh45:
	add	x16, x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.24@PAGEOFF
Lloh46:
	adrp	x17, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.26@PAGE
Lloh47:
	add	x17, x17, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.26@PAGEOFF
	csel	x16, x17, x16, hi
	cmp	w9, w8
	csel	x8, x14, x15, hi
	csel	x9, x13, x16, hi
	cmp	w12, #512
	csel	x8, x11, x8, hi
	csel	x9, x10, x9, hi
	ldp	d4, d5, [x8, #32]
	fmul	d5, d2, d5
	fadd	d4, d4, d5
	ldp	q5, q6, [x9]
	ldr	q7, [x9, #32]
	ldp	q16, q17, [x8]
	dup.2d	v18, v7[1]
	mov.d	v18[1], v4[0]
	mov.d	v7[1], v17[1]
	fmul.2d	v4, v18, v2[0]
	fadd.2d	v4, v7, v4
	fmul.2d	v4, v4, v2[0]
	ext.16b	v7, v6, v17, #8
	fadd.2d	v4, v7, v4
	fmul.2d	v4, v4, v2[0]
	mov.d	v6[1], v16[1]
	fadd.2d	v4, v6, v4
	fmul.2d	v4, v4, v2[0]
	ext.16b	v6, v5, v16, #8
	fadd.2d	v4, v6, v4
	fmul.2d	v2, v4, v2[0]
	mov.d	v5[1], v1[0]
	fadd.2d	v1, v5, v2
	dup.2d	v2, v1[1]
	fdiv.2d	v1, v1, v2
	fmov	d2, #-0.12500000
	fadd	d1, d1, d2
	fdiv	d1, d1, d8
	fmul	d0, d0, d1
	fsub	d10, d3, d0
LBB12_5:
	mov	x8, #39789
	movk	x8, #20546, lsl #16
	movk	x8, #3543, lsl #32
	movk	x8, #16354, lsl #48
	fmov	d0, x8
	fmul	d0, d10, d0
	; InlineAsm Start
	fsqrt	d8, d8
	; InlineAsm End
	fdiv	d0, d0, d8
	.cfi_def_cfa wsp, 64
	ldp	x29, x30, [sp, #48]
	ldp	x20, x19, [sp, #32]
	ldp	d9, d8, [sp, #16]
	ldp	d11, d10, [sp], #64
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore b8
	.cfi_restore b9
	.cfi_restore b10
	.cfi_restore b11
	ret
	.loh AdrpAdd	Lloh46, Lloh47
	.loh AdrpAdd	Lloh44, Lloh45
	.loh AdrpAdd	Lloh42, Lloh43
	.loh AdrpAdd	Lloh40, Lloh41
	.loh AdrpAdd	Lloh38, Lloh39
	.loh AdrpAdd	Lloh36, Lloh37
	.loh AdrpAdd	Lloh34, Lloh35
	.loh AdrpAdd	Lloh32, Lloh33
	.loh AdrpAdd	Lloh30, Lloh31
	.loh AdrpAdd	Lloh28, Lloh29
	.loh AdrpAdd	Lloh26, Lloh27
	.loh AdrpAdd	Lloh24, Lloh25
	.loh AdrpAdd	Lloh22, Lloh23
	.loh AdrpAdd	Lloh20, Lloh21
	.loh AdrpAdd	Lloh18, Lloh19
	.loh AdrpAdd	Lloh16, Lloh17
	.cfi_endproc

	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0
lCPI13_0:
	.quad	0xbef0c5c6ba169668
	.quad	0x3f285f56b9cdf664
lCPI13_1:
	.quad	0x3f570d9f98472c61
	.quad	0x3f939d0b12637e53
lCPI13_2:
	.quad	0xbfb0000000000000
	.quad	0x3ff0000000000000
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math2j12j1
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math2j12j1:
	.cfi_startproc
	fmov	x8, d0
	lsr	x9, x8, #32
	ubfx	w9, w9, #20, #11
	cmp	w9, #2046
	b.ls	LBB13_2
	fmul	d0, d0, d0
	fmov	d1, #1.00000000
	fdiv	d0, d1, d0
	ret
LBB13_2:
	ubfx	x0, x8, #32, #31
	lsr	w9, w0, #30
	cbz	w9, LBB13_4
	fabs	d0, d0
	lsr	x2, x8, #63
	mov	w1, #0
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math2j16common
LBB13_4:
	lsr	w8, w0, #27
	mov.16b	v1, v0
	cmp	w8, #6
	b.ls	LBB13_6
	fmul	d1, d0, d0
	mov	x8, #32472
	movk	x8, #53177, lsl #16
	movk	x8, #10959, lsl #32
	movk	x8, #15787, lsl #48
	fmov	d2, x8
	fmul	d2, d1, d2
	mov	x8, #65324
	movk	x8, #51581, lsl #16
	movk	x8, #44168, lsl #32
	movk	x8, #15925, lsl #48
	fmov	d3, x8
	fadd	d2, d2, d3
	fmul	d2, d1, d2
	mov	x8, #33944
	movk	x8, #13119, lsl #16
	movk	x8, #49144, lsl #32
	movk	x8, #16051, lsl #48
	fmov	d3, x8
	mov	x8, #3033
	movk	x8, #18122, lsl #16
	movk	x8, #43770, lsl #32
	movk	x8, #15978, lsl #48
	fmov	d4, x8
	dup.2d	v4, v4[0]
	fadd	d2, d2, d3
	mov.d	v4[1], v2[0]
	fmul.2d	v2, v4, v1[0]
Lloh48:
	adrp	x8, lCPI13_0@PAGE
Lloh49:
	ldr	q3, [x8, lCPI13_0@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul.2d	v2, v2, v1[0]
Lloh50:
	adrp	x8, lCPI13_1@PAGE
Lloh51:
	ldr	q3, [x8, lCPI13_1@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul.2d	v2, v2, v1[0]
Lloh52:
	adrp	x8, lCPI13_2@PAGE
Lloh53:
	ldr	q3, [x8, lCPI13_2@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul	d1, d1, d2
	mov	d2, v2[1]
	fdiv	d1, d1, d2
LBB13_6:
	fmov	d2, #0.50000000
	fadd	d1, d1, d2
	fmul	d0, d0, d1
	ret
	.loh AdrpLdr	Lloh52, Lloh53
	.loh AdrpAdrp	Lloh50, Lloh52
	.loh AdrpLdr	Lloh50, Lloh51
	.loh AdrpAdrp	Lloh48, Lloh50
	.loh AdrpLdr	Lloh48, Lloh49
	.cfi_endproc

	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0
lCPI14_0:
	.quad	0xbf5f55e54844f50f
	.quad	0x3f2a8c896c257764
lCPI14_1:
	.quad	0x3fa9d3c776292cd1
	.quad	0x3f94650d3f4da9f0
lCPI14_2:
	.quad	0xbfc91866143cbc8a
	.quad	0x3ff0000000000000
lCPI14_3:
	.quad	0xbef0c5c6ba169668
	.quad	0x3f285f56b9cdf664
lCPI14_4:
	.quad	0x3f570d9f98472c61
	.quad	0x3f939d0b12637e53
lCPI14_5:
	.quad	0xbfb0000000000000
	.quad	0x3ff0000000000000
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math2j12y1
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math2j12y1:
	.cfi_startproc
	fmov	x8, d0
	lsr	x0, x8, #32
	orr	w9, w8, w0, lsl #1
	cbz	w9, LBB14_4
	tbnz	x8, #63, LBB14_5
	lsr	x9, x8, #52
	cmp	x9, #2046
	b.ls	LBB14_6
	fmov	d1, #1.00000000
	fdiv	d0, d1, d0
	ret
LBB14_4:
	mov	x8, #-4503599627370496
	fmov	d0, x8
	ret
LBB14_5:
	mov	x8, #9221120237041090560
	fmov	d0, x8
	ret
LBB14_6:
	lsr	x8, x8, #62
	cbz	x8, LBB14_8
	mov	w1, #1
	mov	w2, #0
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math2j16common
LBB14_8:
	cmp	x9, #969
	b.hs	LBB14_10
	mov	x8, #51331
	movk	x8, #28105, lsl #16
	movk	x8, #24368, lsl #32
	movk	x8, #49124, lsl #48
	fmov	d1, x8
	fdiv	d0, d1, d0
	ret
LBB14_10:
	stp	d11, d10, [sp, #-48]!
	.cfi_def_cfa_offset 48
	stp	d9, d8, [sp, #16]
	stp	x29, x30, [sp, #32]
	add	x29, sp, #32
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset b8, -24
	.cfi_offset b9, -32
	.cfi_offset b10, -40
	.cfi_offset b11, -48
	fmul	d1, d0, d0
	mov	x8, #30506
	movk	x8, #56010, lsl #16
	movk	x8, #20537, lsl #32
	movk	x8, #15794, lsl #48
	fmov	d2, x8
	fmul	d2, d1, d2
	mov	x8, #39558
	movk	x8, #23462, lsl #16
	movk	x8, #48925, lsl #32
	movk	x8, #15930, lsl #48
	fmov	d3, x8
	fadd	d2, d2, d3
	mov	x8, #1464
	movk	x8, #22161, lsl #16
	movk	x8, #44032, lsl #32
	movk	x8, #15992, lsl #48
	fmov	d3, x8
	dup.2d	v3, v3[0]
	mov.d	v3[1], v2[0]
	fmul.2d	v2, v3, v1[0]
	mov	x8, #47246
	movk	x8, #36774, lsl #16
	movk	x8, #43779, lsl #32
	movk	x8, #16120, lsl #48
	fmov	d3, x8
	mov	x8, #36006
	movk	x8, #35150, lsl #16
	movk	x8, #49242, lsl #32
	movk	x8, #16054, lsl #48
	dup.2d	v4, x8
	fsub.2d	v3, v3, v2
	fadd.2d	v2, v2, v4
	mov.d	v3[1], v2[1]
	fmul.2d	v2, v3, v1[0]
Lloh54:
	adrp	x8, lCPI14_0@PAGE
Lloh55:
	ldr	q3, [x8, lCPI14_0@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul.2d	v2, v2, v1[0]
Lloh56:
	adrp	x8, lCPI14_1@PAGE
Lloh57:
	ldr	q3, [x8, lCPI14_1@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul.2d	v2, v2, v1[0]
Lloh58:
	adrp	x8, lCPI14_2@PAGE
Lloh59:
	ldr	q3, [x8, lCPI14_2@PAGEOFF]
	fadd.2d	v2, v2, v3
	dup.2d	v3, v2[1]
	fdiv.2d	v2, v2, v3
	fmul	d9, d0, d2
	mov	x8, #32472
	movk	x8, #53177, lsl #16
	movk	x8, #10959, lsl #32
	movk	x8, #15787, lsl #48
	fmov	d2, x8
	fmul	d2, d1, d2
	mov	x8, #65324
	movk	x8, #51581, lsl #16
	movk	x8, #44168, lsl #32
	movk	x8, #15925, lsl #48
	fmov	d3, x8
	fadd	d2, d2, d3
	fmul	d2, d1, d2
	mov	x8, #33944
	movk	x8, #13119, lsl #16
	movk	x8, #49144, lsl #32
	movk	x8, #16051, lsl #48
	fmov	d3, x8
	fadd	d2, d2, d3
	mov	x8, #3033
	movk	x8, #18122, lsl #16
	movk	x8, #43770, lsl #32
	movk	x8, #15978, lsl #48
	fmov	d3, x8
	dup.2d	v3, v3[0]
	mov.d	v3[1], v2[0]
	fmul.2d	v2, v3, v1[0]
Lloh60:
	adrp	x8, lCPI14_3@PAGE
Lloh61:
	ldr	q3, [x8, lCPI14_3@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul.2d	v2, v2, v1[0]
Lloh62:
	adrp	x8, lCPI14_4@PAGE
Lloh63:
	ldr	q3, [x8, lCPI14_4@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul.2d	v2, v2, v1[0]
Lloh64:
	adrp	x8, lCPI14_5@PAGE
Lloh65:
	ldr	q3, [x8, lCPI14_5@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul	d1, d1, d2
	mov	d2, v2[1]
	fdiv	d1, d1, d2
	fmov	d2, #0.50000000
	fadd	d1, d1, d2
	fmul	d10, d0, d1
	mov.16b	v8, v0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3log3log
	fmul	d0, d10, d0
	fmov	d1, #-1.00000000
	fdiv	d1, d1, d8
	fadd	d0, d0, d1
	mov	x8, #51331
	movk	x8, #28105, lsl #16
	movk	x8, #24368, lsl #32
	movk	x8, #16356, lsl #48
	fmov	d1, x8
	fmul	d0, d0, d1
	fadd	d0, d9, d0
	.cfi_def_cfa wsp, 48
	ldp	x29, x30, [sp, #32]
	ldp	d9, d8, [sp, #16]
	ldp	d11, d10, [sp], #48
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore b8
	.cfi_restore b9
	.cfi_restore b10
	.cfi_restore b11
	ret
	.loh AdrpLdr	Lloh64, Lloh65
	.loh AdrpAdrp	Lloh62, Lloh64
	.loh AdrpLdr	Lloh62, Lloh63
	.loh AdrpAdrp	Lloh60, Lloh62
	.loh AdrpLdr	Lloh60, Lloh61
	.loh AdrpLdr	Lloh58, Lloh59
	.loh AdrpAdrp	Lloh56, Lloh58
	.loh AdrpLdr	Lloh56, Lloh57
	.loh AdrpAdrp	Lloh54, Lloh56
	.loh AdrpLdr	Lloh54, Lloh55
	.cfi_endproc

	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math2j16common:
	.cfi_startproc
	stp	d11, d10, [sp, #-80]!
	.cfi_def_cfa_offset 80
	stp	d9, d8, [sp, #16]
	stp	x22, x21, [sp, #32]
	stp	x20, x19, [sp, #48]
	stp	x29, x30, [sp, #64]
	add	x29, sp, #64
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset w21, -40
	.cfi_offset w22, -48
	.cfi_offset b8, -56
	.cfi_offset b9, -64
	.cfi_offset b10, -72
	.cfi_offset b11, -80
	mov	x19, x2
	mov	x20, x1
	mov.16b	v8, v0
	mov	x21, x0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3sin3sin
	fneg	d1, d0
	cmp	w20, #0
	fcsel	d11, d1, d0, ne
	mov.16b	v0, v8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3cos3cos
	fsub	d10, d11, d0
	lsr	w8, w21, #21
	cmp	w8, #1023
	b.hs	LBB15_5
	mov.16b	v9, v0
	fadd	d0, d8, d8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3cos3cos
	fmul	d1, d11, d9
	fcmp	d1, #0.0
	b.le	LBB15_3
	fneg	d1, d11
	fsub	d1, d1, d9
	fdiv	d10, d0, d1
	lsr	w8, w21, #27
	cmp	w8, #8
	b.ls	LBB15_4
	b	LBB15_5
LBB15_3:
	fdiv	d1, d0, d10
	lsr	w8, w21, #27
	cmp	w8, #8
	b.hi	LBB15_5
LBB15_4:
	mov	w8, #11914
	movk	w8, #16402, lsl #16
	fneg	d0, d1
	cmp	w20, #0
	fcsel	d0, d0, d1, ne
	fmov	x9, d8
	lsr	x10, x9, #32
Lloh66:
	adrp	x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.38@PAGE
Lloh67:
	add	x11, x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.38@PAGEOFF
Lloh68:
	adrp	x12, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.39@PAGE
Lloh69:
	add	x12, x12, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.39@PAGEOFF
	ubfx	x9, x9, #32, #31
	ubfx	w10, w10, #21, #10
Lloh70:
	adrp	x13, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.36@PAGE
Lloh71:
	add	x13, x13, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.36@PAGEOFF
Lloh72:
	adrp	x14, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.37@PAGE
Lloh73:
	add	x14, x14, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.37@PAGEOFF
Lloh74:
	adrp	x15, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.33@PAGE
Lloh75:
	add	x15, x15, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.33@PAGEOFF
Lloh76:
	adrp	x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.35@PAGE
Lloh77:
	add	x16, x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.35@PAGEOFF
	mov	w17, #56172
	movk	w17, #16390, lsl #16
	cmp	w9, w17
	csel	x15, x16, x15, hi
Lloh78:
	adrp	x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.32@PAGE
Lloh79:
	add	x16, x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.32@PAGEOFF
Lloh80:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.34@PAGE
Lloh81:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.34@PAGEOFF
	csel	x16, x0, x16, hi
	cmp	w9, w8
	csel	x14, x14, x15, hi
	csel	x13, x13, x16, hi
	cmp	w10, #512
	csel	x10, x12, x14, hi
	csel	x11, x11, x13, hi
	fmul	d2, d8, d8
	fmov	d1, #1.00000000
	fdiv	d2, d1, d2
	ldp	d3, d4, [x11]
	ldp	d5, d6, [x11, #16]
	ldp	d7, d16, [x11, #32]
	fmul	d16, d2, d16
	fadd	d7, d7, d16
	fmul	d7, d2, d7
	fadd	d6, d6, d7
	fmul	d6, d2, d6
	fadd	d5, d5, d6
	fmul	d5, d2, d5
	fadd	d4, d4, d5
	fmul	d4, d2, d4
	fadd	d3, d3, d4
	ldp	d4, d5, [x10]
	ldp	d6, d7, [x10, #16]
	ldr	d16, [x10, #32]
	fmul	d16, d2, d16
	fadd	d7, d7, d16
	fmul	d7, d2, d7
	fadd	d6, d6, d7
	fmul	d6, d2, d6
	fadd	d5, d5, d6
	fmul	d5, d2, d5
	fadd	d4, d4, d5
	fmul	d4, d2, d4
	fadd	d4, d4, d1
	fdiv	d3, d3, d4
	fadd	d3, d3, d1
	fmul	d3, d10, d3
Lloh82:
	adrp	x10, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.46@PAGE
Lloh83:
	add	x10, x10, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.46@PAGEOFF
Lloh84:
	adrp	x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.47@PAGE
Lloh85:
	add	x11, x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.47@PAGEOFF
	lsr	w12, w9, #21
Lloh86:
	adrp	x13, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.44@PAGE
Lloh87:
	add	x13, x13, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.44@PAGEOFF
Lloh88:
	adrp	x14, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.45@PAGE
Lloh89:
	add	x14, x14, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.45@PAGEOFF
Lloh90:
	adrp	x15, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.41@PAGE
Lloh91:
	add	x15, x15, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.41@PAGEOFF
Lloh92:
	adrp	x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.43@PAGE
Lloh93:
	add	x16, x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.43@PAGEOFF
	cmp	w9, w17
	csel	x15, x16, x15, hi
Lloh94:
	adrp	x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.40@PAGE
Lloh95:
	add	x16, x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.40@PAGEOFF
Lloh96:
	adrp	x17, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.42@PAGE
Lloh97:
	add	x17, x17, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.42@PAGEOFF
	csel	x16, x17, x16, hi
	cmp	w9, w8
	csel	x8, x14, x15, hi
	csel	x9, x13, x16, hi
	cmp	w12, #512
	csel	x8, x11, x8, hi
	csel	x9, x10, x9, hi
	ldp	d4, d5, [x8, #32]
	fmul	d5, d2, d5
	fadd	d4, d4, d5
	ldp	q5, q6, [x9]
	ldr	q7, [x9, #32]
	ldp	q16, q17, [x8]
	dup.2d	v18, v7[1]
	mov.d	v18[1], v4[0]
	mov.d	v7[1], v17[1]
	fmul.2d	v4, v18, v2[0]
	fadd.2d	v4, v7, v4
	fmul.2d	v4, v4, v2[0]
	ext.16b	v7, v6, v17, #8
	fadd.2d	v4, v7, v4
	fmul.2d	v4, v4, v2[0]
	mov.d	v6[1], v16[1]
	fadd.2d	v4, v6, v4
	fmul.2d	v4, v4, v2[0]
	ext.16b	v6, v5, v16, #8
	fadd.2d	v4, v6, v4
	fmul.2d	v2, v4, v2[0]
	mov.d	v5[1], v1[0]
	fadd.2d	v1, v5, v2
	dup.2d	v2, v1[1]
	fdiv.2d	v1, v1, v2
	fmov	d2, #0.37500000
	fadd	d1, d1, d2
	fdiv	d1, d1, d8
	fmul	d0, d0, d1
	fsub	d10, d3, d0
LBB15_5:
	fneg	d0, d10
	cmp	w19, #0
	fcsel	d0, d0, d10, ne
	mov	x8, #39789
	movk	x8, #20546, lsl #16
	movk	x8, #3543, lsl #32
	movk	x8, #16354, lsl #48
	fmov	d1, x8
	fmul	d0, d0, d1
	; InlineAsm Start
	fsqrt	d8, d8
	; InlineAsm End
	fdiv	d0, d0, d8
	.cfi_def_cfa wsp, 80
	ldp	x29, x30, [sp, #64]
	ldp	x20, x19, [sp, #48]
	ldp	x22, x21, [sp, #32]
	ldp	d9, d8, [sp, #16]
	ldp	d11, d10, [sp], #80
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	.cfi_restore b8
	.cfi_restore b9
	.cfi_restore b10
	.cfi_restore b11
	ret
	.loh AdrpAdd	Lloh96, Lloh97
	.loh AdrpAdd	Lloh94, Lloh95
	.loh AdrpAdd	Lloh92, Lloh93
	.loh AdrpAdd	Lloh90, Lloh91
	.loh AdrpAdd	Lloh88, Lloh89
	.loh AdrpAdd	Lloh86, Lloh87
	.loh AdrpAdd	Lloh84, Lloh85
	.loh AdrpAdd	Lloh82, Lloh83
	.loh AdrpAdd	Lloh80, Lloh81
	.loh AdrpAdd	Lloh78, Lloh79
	.loh AdrpAdd	Lloh76, Lloh77
	.loh AdrpAdd	Lloh74, Lloh75
	.loh AdrpAdd	Lloh72, Lloh73
	.loh AdrpAdd	Lloh70, Lloh71
	.loh AdrpAdd	Lloh68, Lloh69
	.loh AdrpAdd	Lloh66, Lloh67
	.cfi_endproc

	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0
lCPI16_0:
	.quad	0xbf28e6a5b61ac6e9
	.quad	0x3f8ffce882c8c2a4
lCPI16_1:
	.quad	0x3f8ffffffffffffd
	.quad	0x3ff0000000000000
lCPI16_2:
	.quad	0xbef0c5c6ba169668
	.quad	0x3f285f56b9cdf664
lCPI16_3:
	.quad	0x3f570d9f98472c61
	.quad	0x3f939d0b12637e53
lCPI16_4:
	.quad	0xbfb0000000000000
	.quad	0x3ff0000000000000
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math2jn2jn
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math2jn2jn:
	.cfi_startproc
	stp	d13, d12, [sp, #-96]!
	.cfi_def_cfa_offset 96
	stp	d11, d10, [sp, #16]
	stp	d9, d8, [sp, #32]
	stp	x22, x21, [sp, #48]
	stp	x20, x19, [sp, #64]
	stp	x29, x30, [sp, #80]
	add	x29, sp, #80
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset w21, -40
	.cfi_offset w22, -48
	.cfi_offset b8, -56
	.cfi_offset b9, -64
	.cfi_offset b10, -72
	.cfi_offset b11, -80
	.cfi_offset b12, -88
	.cfi_offset b13, -96
	.cfi_remember_state
	mov	x8, x0
	fmov	x9, d0
	ubfx	x0, x9, #32, #31
	cmp	w9, #0
	cset	w10, ne
	orr	w10, w0, w10
	mov	w11, #2146435072
	cmp	w10, w11
	b.hi	LBB16_75
	cbz	w8, LBB16_10
	lsr	x10, x9, #63
	eor	w11, w10, #0x1
	fmov	x12, d0
	eor	x12, x12, #0x8000000000000000
	sub	w13, w8, #1
	tst	w8, #0x80000000
	csel	w10, w11, w10, ne
	csinv	w21, w13, w8, eq
	csel	x19, x12, x9, ne
	cbz	w21, LBB16_13
	and	w20, w8, w10
	movi.2d	v0, #0000000000000000
	mov	w8, #2146435072
	cmp	w0, w8
	b.eq	LBB16_74
	orr	w8, w0, w9
	cbz	w8, LBB16_74
	and	x8, x19, #0x7fffffffffffffff
	ucvtf	d2, w21
	fmov	d8, x8
	fcmp	d2, d8
	lsr	w9, w0, #20
	b.pl	LBB16_17
	cmp	w9, #1324
	b.ls	LBB16_27
	and	w19, w21, #0x3
	mov.16b	v0, v8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3cos3cos
	mov.16b	v9, v0
	cmp	w19, #1
	b.gt	LBB16_41
	cbnz	w19, LBB16_51
	mov.16b	v0, v8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3sin3sin
	fsub	d0, d0, d9
	b	LBB16_53
LBB16_10:
	lsr	w8, w0, #20
	cmp	w8, #2046
	b.hi	LBB16_14
	fabs	d1, d0
	lsr	w8, w0, #30
	cbz	w8, LBB16_22
	mov.16b	v0, v1
	mov	w1, #0
	.cfi_def_cfa wsp, 96
	ldp	x29, x30, [sp, #80]
	ldp	x20, x19, [sp, #64]
	ldp	x22, x21, [sp, #48]
	ldp	d9, d8, [sp, #32]
	ldp	d11, d10, [sp, #16]
	ldp	d13, d12, [sp], #96
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	.cfi_restore b8
	.cfi_restore b9
	.cfi_restore b10
	.cfi_restore b11
	.cfi_restore b12
	.cfi_restore b13
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math2j06common
LBB16_13:
	.cfi_restore_state
	.cfi_remember_state
	fmov	d0, x19
	lsr	x8, x19, #32
	ubfx	w8, w8, #20, #11
	cmp	w8, #2046
	b.ls	LBB16_15
LBB16_14:
	fmul	d0, d0, d0
	fmov	d1, #1.00000000
	fdiv	d0, d1, d0
	b	LBB16_75
LBB16_15:
	ubfx	x0, x19, #32, #31
	lsr	w8, w0, #30
	cbz	w8, LBB16_24
	fabs	d0, d0
	lsr	x2, x19, #63
	mov	w1, #0
	.cfi_def_cfa wsp, 96
	ldp	x29, x30, [sp, #80]
	ldp	x20, x19, [sp, #64]
	ldp	x22, x21, [sp, #48]
	ldp	d9, d8, [sp, #32]
	ldp	d11, d10, [sp, #16]
	ldp	d13, d12, [sp], #96
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	.cfi_restore b8
	.cfi_restore b9
	.cfi_restore b10
	.cfi_restore b11
	.cfi_restore b12
	.cfi_restore b13
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math2j16common
LBB16_17:
	.cfi_restore_state
	cmp	w9, #993
	b.hs	LBB16_29
	cmp	w21, #32
	b.hi	LBB16_74
	fmov	d0, #0.50000000
	fmul	d0, d8, d0
	fmov	d1, #1.00000000
	fmov	d2, #2.00000000
	mov.16b	v3, v0
	fmov	d4, #1.00000000
LBB16_20:
	fmul	d4, d4, d2
	fmul	d3, d0, d3
	fadd	d2, d2, d1
	subs	w21, w21, #1
	b.ne	LBB16_20
	fdiv	d0, d3, d4
	b	LBB16_74
LBB16_22:
	lsr	w8, w0, #21
	cmp	w8, #504
	b.ls	LBB16_40
	fmul	d0, d0, d0
	mov	x8, #23951
	movk	x8, #62580, lsl #16
	movk	x8, #2236, lsl #32
	movk	x8, #15892, lsl #48
	fmov	d2, x8
	fmul	d2, d0, d2
	mov	x8, #54697
	movk	x8, #52868, lsl #16
	movk	x8, #15188, lsl #32
	movk	x8, #16033, lsl #48
	fmov	d3, x8
	fadd	d2, d2, d3
	mov	x8, #16334
	movk	x8, #29654, lsl #16
	movk	x8, #54759, lsl #32
	movk	x8, #15923, lsl #48
	fmov	d3, x8
	dup.2d	v3, v3[0]
	mov.d	v3[1], v2[0]
	fmul.2d	v2, v3, v0[0]
	mov	x8, #14617
	movk	x8, #3152, lsl #16
	movk	x8, #45521, lsl #32
	movk	x8, #16062, lsl #48
	fmov	d3, x8
	fsub.2d	v3, v3, v2
	mov	x8, #56308
	movk	x8, #56663, lsl #16
	movk	x8, #42706, lsl #32
	movk	x8, #16158, lsl #48
	dup.2d	v4, x8
	fadd.2d	v2, v2, v4
	mov.d	v3[1], v2[1]
	fmul.2d	v2, v3, v0[0]
Lloh98:
	adrp	x8, lCPI16_0@PAGE
Lloh99:
	ldr	q3, [x8, lCPI16_0@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul.2d	v2, v2, v0[0]
Lloh100:
	adrp	x8, lCPI16_1@PAGE
Lloh101:
	ldr	q3, [x8, lCPI16_1@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul	d3, d0, d2
	fmov	d4, #0.50000000
	fmul	d1, d1, d4
	fmov	d4, #1.00000000
	fadd	d5, d1, d4
	fsub	d1, d4, d1
	fmul	d1, d5, d1
	mov	d2, v2[1]
	fdiv	d2, d3, d2
	fmul	d0, d0, d2
	fadd	d0, d1, d0
	b	LBB16_75
LBB16_24:
	lsr	w8, w0, #27
	mov.16b	v1, v0
	cmp	w8, #6
	b.ls	LBB16_26
	fmul	d1, d0, d0
	mov	x8, #32472
	movk	x8, #53177, lsl #16
	movk	x8, #10959, lsl #32
	movk	x8, #15787, lsl #48
	fmov	d2, x8
	fmul	d2, d1, d2
	mov	x8, #65324
	movk	x8, #51581, lsl #16
	movk	x8, #44168, lsl #32
	movk	x8, #15925, lsl #48
	fmov	d3, x8
	fadd	d2, d2, d3
	fmul	d2, d1, d2
	mov	x8, #33944
	movk	x8, #13119, lsl #16
	movk	x8, #49144, lsl #32
	movk	x8, #16051, lsl #48
	fmov	d3, x8
	mov	x8, #3033
	movk	x8, #18122, lsl #16
	movk	x8, #43770, lsl #32
	movk	x8, #15978, lsl #48
	fmov	d4, x8
	dup.2d	v4, v4[0]
	fadd	d2, d2, d3
	mov.d	v4[1], v2[0]
	fmul.2d	v2, v4, v1[0]
Lloh102:
	adrp	x8, lCPI16_2@PAGE
Lloh103:
	ldr	q3, [x8, lCPI16_2@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul.2d	v2, v2, v1[0]
Lloh104:
	adrp	x8, lCPI16_3@PAGE
Lloh105:
	ldr	q3, [x8, lCPI16_3@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul.2d	v2, v2, v1[0]
Lloh106:
	adrp	x8, lCPI16_4@PAGE
Lloh107:
	ldr	q3, [x8, lCPI16_4@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul	d1, d1, d2
	mov	d2, v2[1]
	fdiv	d1, d1, d2
LBB16_26:
	fmov	d2, #0.50000000
	fadd	d1, d1, d2
	fmul	d0, d1, d0
	b	LBB16_75
LBB16_27:
	lsr	x9, x8, #52
	cmp	x9, #2046
	b.ls	LBB16_43
	fmul	d0, d8, d8
	fmov	d1, #1.00000000
	fdiv	d0, d1, d0
	mov.16b	v9, v0
	b	LBB16_67
LBB16_29:
	fmov	d1, #1.00000000
	fadd	d9, d2, d1
	fadd	d0, d9, d9
	fdiv	d0, d0, d8
	fmov	d2, #2.00000000
	fdiv	d2, d2, d8
	fadd	d3, d0, d2
	fmul	d4, d0, d3
	fmov	d5, #-1.00000000
	fadd	d4, d4, d5
	mov	x8, #225833675390976
	movk	x8, #16845, lsl #48
	fmov	d5, x8
	mov	w8, #1
	fcmp	d4, d5
	b.pl	LBB16_33
	mov	x9, #225833675390976
	movk	x9, #16845, lsl #48
	fmov	d5, x9
	mov.16b	v6, v0
LBB16_31:
	mov.16b	v7, v6
	mov.16b	v6, v4
	add	w8, w8, #1
	fadd	d3, d2, d3
	fmul	d4, d3, d4
	fsub	d4, d4, d7
	fcmp	d4, d5
	b.mi	LBB16_31
	tbnz	w8, #31, LBB16_58
LBB16_33:
	movi.2d	v10, #0000000000000000
LBB16_34:
	ucvtf	d2, w8
	fadd	d2, d9, d2
	fadd	d2, d2, d2
	fdiv	d2, d2, d8
	fsub	d2, d2, d10
	fdiv	d10, d1, d2
	sub	w8, w8, #1
	cmn	w8, #1
	b.ne	LBB16_34
LBB16_35:
	fabs	d0, d0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3log3log
	fmul	d0, d9, d0
	mov	x8, #14831
	movk	x8, #65274, lsl #16
	movk	x8, #11842, lsl #32
	movk	x8, #16518, lsl #48
	fmov	d1, x8
	add	w8, w21, #1
	lsl	w9, w21, #1
	fcmp	d0, d1
	b.pl	LBB16_45
	fmov	d11, #1.00000000
	mov.16b	v12, v10
LBB16_37:
	mov.16b	v0, v12
	mov.16b	v12, v11
	ucvtf	d1, w9
	fmul	d1, d11, d1
	fdiv	d1, d1, d8
	fsub	d11, d1, d0
	sub	w8, w8, #1
	sub	w9, w9, #2
	cmp	w8, #1
	b.hi	LBB16_37
LBB16_38:
	lsr	x8, x19, #32
	ubfx	w8, w8, #20, #11
	cmp	w8, #2046
	b.ls	LBB16_49
	fmul	d0, d8, d8
	fmov	d1, #1.00000000
	fdiv	d9, d1, d0
	mov.16b	v0, v9
	b	LBB16_71
LBB16_40:
	fmov	d0, #0.25000000
	fmul	d0, d1, d0
	fmul	d0, d1, d0
	lsr	w8, w0, #27
	cmp	w8, #6
	fcsel	d0, d0, d1, hi
	fmov	d1, #1.00000000
	fsub	d0, d1, d0
	b	LBB16_75
LBB16_41:
	cmp	w19, #2
	b.eq	LBB16_52
	mov.16b	v0, v8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3sin3sin
	fadd	d0, d9, d0
	b	LBB16_53
LBB16_43:
	lsr	x9, x8, #62
	cbz	x9, LBB16_54
	lsr	x19, x8, #32
	mov	x0, x19
	mov.16b	v0, v8
	mov	w1, #0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math2j06common
	mov.16b	v9, v0
	mov	x0, x19
	mov.16b	v0, v8
	mov	w1, #0
	mov	w2, #0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math2j16common
	b	LBB16_67
LBB16_45:
	fmov	d1, #1.00000000
	mov	x10, #6858982232485265408
	fmov	d0, x10
	mov.16b	v12, v10
	b	LBB16_47
LBB16_46:
	mov.16b	v12, v1
	sub	w8, w8, #1
	sub	w9, w9, #2
	mov.16b	v1, v11
	cmp	w8, #1
	b.le	LBB16_38
LBB16_47:
	ucvtf	d2, w9
	fmul	d2, d1, d2
	fdiv	d2, d2, d8
	fsub	d11, d2, d12
	fcmp	d11, d0
	b.le	LBB16_46
	fdiv	d1, d1, d11
	fdiv	d10, d10, d11
	fmov	d11, #1.00000000
	b	LBB16_46
LBB16_49:
	ubfx	x19, x19, #32, #31
	lsr	w8, w19, #30
	cbz	w8, LBB16_56
	mov	x0, x19
	mov.16b	v0, v8
	mov	w1, #0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math2j06common
	mov.16b	v9, v0
	mov	x0, x19
	mov.16b	v0, v8
	mov	w1, #0
	mov	w2, #0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math2j16common
	b	LBB16_71
LBB16_51:
	fneg	d9, d9
LBB16_52:
	mov.16b	v0, v8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3sin3sin
	fsub	d0, d9, d0
LBB16_53:
	mov	x8, #39789
	movk	x8, #20546, lsl #16
	movk	x8, #3543, lsl #32
	movk	x8, #16354, lsl #48
	fmov	d1, x8
	fmul	d0, d0, d1
	; InlineAsm Start
	fsqrt	d8, d8
	; InlineAsm End
	fdiv	d0, d0, d8
	b	LBB16_74
LBB16_54:
	lsr	x9, x8, #53
	cmp	x9, #505
	b.lo	LBB16_59
	fmul	d0, d8, d8
	mov	x8, #23951
	movk	x8, #62580, lsl #16
	movk	x8, #2236, lsl #32
	movk	x8, #15892, lsl #48
	fmov	d1, x8
	fmul	d1, d0, d1
	mov	x8, #54697
	movk	x8, #52868, lsl #16
	movk	x8, #15188, lsl #32
	movk	x8, #16033, lsl #48
	fmov	d2, x8
	fadd	d1, d1, d2
	mov	x8, #16334
	movk	x8, #29654, lsl #16
	movk	x8, #54759, lsl #32
	movk	x8, #15923, lsl #48
	fmov	d2, x8
	dup.2d	v2, v2[0]
	mov.d	v2[1], v1[0]
	fmul.2d	v1, v2, v0[0]
	mov	x8, #14617
	movk	x8, #3152, lsl #16
	movk	x8, #45521, lsl #32
	movk	x8, #16062, lsl #48
	fmov	d2, x8
	fsub.2d	v2, v2, v1
	mov	x8, #56308
	movk	x8, #56663, lsl #16
	movk	x8, #42706, lsl #32
	movk	x8, #16158, lsl #48
	dup.2d	v3, x8
	fadd.2d	v1, v1, v3
	mov.d	v2[1], v1[1]
	fmul.2d	v1, v2, v0[0]
Lloh108:
	adrp	x8, lCPI16_0@PAGE
Lloh109:
	ldr	q2, [x8, lCPI16_0@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul.2d	v1, v1, v0[0]
Lloh110:
	adrp	x8, lCPI16_1@PAGE
Lloh111:
	ldr	q2, [x8, lCPI16_1@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul	d2, d0, d1
	fmov	d3, #0.50000000
	fmul	d3, d8, d3
	fmov	d4, #1.00000000
	fadd	d5, d3, d4
	fsub	d3, d4, d3
	fmul	d3, d5, d3
	mov	d1, v1[1]
	fdiv	d1, d2, d1
	fmul	d1, d0, d1
	fadd	d9, d3, d1
	b	LBB16_61
LBB16_56:
	lsr	w8, w19, #21
	cmp	w8, #505
	b.lo	LBB16_62
	fmul	d0, d8, d8
	mov	x8, #23951
	movk	x8, #62580, lsl #16
	movk	x8, #2236, lsl #32
	movk	x8, #15892, lsl #48
	fmov	d1, x8
	fmul	d1, d0, d1
	mov	x8, #54697
	movk	x8, #52868, lsl #16
	movk	x8, #15188, lsl #32
	movk	x8, #16033, lsl #48
	fmov	d2, x8
	fadd	d1, d1, d2
	mov	x8, #16334
	movk	x8, #29654, lsl #16
	movk	x8, #54759, lsl #32
	movk	x8, #15923, lsl #48
	fmov	d2, x8
	dup.2d	v2, v2[0]
	mov.d	v2[1], v1[0]
	fmul.2d	v1, v2, v0[0]
	mov	x8, #14617
	movk	x8, #3152, lsl #16
	movk	x8, #45521, lsl #32
	movk	x8, #16062, lsl #48
	fmov	d2, x8
	fsub.2d	v2, v2, v1
	mov	x8, #56308
	movk	x8, #56663, lsl #16
	movk	x8, #42706, lsl #32
	movk	x8, #16158, lsl #48
	dup.2d	v3, x8
	fadd.2d	v1, v1, v3
	mov.d	v2[1], v1[1]
	fmul.2d	v1, v2, v0[0]
Lloh112:
	adrp	x8, lCPI16_0@PAGE
Lloh113:
	ldr	q2, [x8, lCPI16_0@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul.2d	v1, v1, v0[0]
Lloh114:
	adrp	x8, lCPI16_1@PAGE
Lloh115:
	ldr	q2, [x8, lCPI16_1@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul	d2, d0, d1
	fmov	d3, #0.50000000
	fmul	d3, d8, d3
	fmov	d4, #1.00000000
	fadd	d5, d3, d4
	fsub	d3, d4, d3
	fmul	d3, d5, d3
	mov	d1, v1[1]
	fdiv	d1, d2, d1
	fmul	d1, d0, d1
	fadd	d9, d3, d1
	b	LBB16_64
LBB16_58:
	movi.2d	v10, #0000000000000000
	b	LBB16_35
LBB16_59:
	fmov	d0, #0.25000000
	fmul	d0, d8, d0
	lsr	x8, x8, #59
	cmp	x8, #6
	fmov	d1, #1.00000000
	fcsel	d0, d0, d1, hi
	fmul	d0, d0, d8
	fsub	d9, d1, d0
	cmp	x8, #7
	b.lo	LBB16_65
	fmul	d0, d8, d8
LBB16_61:
	mov	x8, #32472
	movk	x8, #53177, lsl #16
	movk	x8, #10959, lsl #32
	movk	x8, #15787, lsl #48
	fmov	d1, x8
	fmul	d1, d0, d1
	mov	x8, #65324
	movk	x8, #51581, lsl #16
	movk	x8, #44168, lsl #32
	movk	x8, #15925, lsl #48
	fmov	d2, x8
	fadd	d1, d1, d2
	fmul	d1, d0, d1
	mov	x8, #33944
	movk	x8, #13119, lsl #16
	movk	x8, #49144, lsl #32
	movk	x8, #16051, lsl #48
	fmov	d2, x8
	mov	x8, #3033
	movk	x8, #18122, lsl #16
	movk	x8, #43770, lsl #32
	movk	x8, #15978, lsl #48
	fmov	d3, x8
	dup.2d	v3, v3[0]
	fadd	d1, d1, d2
	mov.d	v3[1], v1[0]
	fmul.2d	v1, v3, v0[0]
Lloh116:
	adrp	x8, lCPI16_2@PAGE
Lloh117:
	ldr	q2, [x8, lCPI16_2@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul.2d	v1, v1, v0[0]
Lloh118:
	adrp	x8, lCPI16_3@PAGE
Lloh119:
	ldr	q2, [x8, lCPI16_3@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul.2d	v1, v1, v0[0]
Lloh120:
	adrp	x8, lCPI16_4@PAGE
Lloh121:
	ldr	q2, [x8, lCPI16_4@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul	d0, d0, d1
	mov	d1, v1[1]
	fdiv	d0, d0, d1
	b	LBB16_66
LBB16_62:
	fmov	d0, #0.25000000
	fmul	d0, d8, d0
	fmul	d0, d0, d8
	lsr	w8, w19, #27
	cmp	w8, #6
	fcsel	d0, d0, d8, hi
	fmov	d1, #1.00000000
	fsub	d9, d1, d0
	cmp	w8, #7
	b.lo	LBB16_69
	fmul	d0, d8, d8
LBB16_64:
	mov	x8, #32472
	movk	x8, #53177, lsl #16
	movk	x8, #10959, lsl #32
	movk	x8, #15787, lsl #48
	fmov	d1, x8
	fmul	d1, d0, d1
	mov	x8, #65324
	movk	x8, #51581, lsl #16
	movk	x8, #44168, lsl #32
	movk	x8, #15925, lsl #48
	fmov	d2, x8
	fadd	d1, d1, d2
	fmul	d1, d0, d1
	mov	x8, #33944
	movk	x8, #13119, lsl #16
	movk	x8, #49144, lsl #32
	movk	x8, #16051, lsl #48
	fmov	d2, x8
	mov	x8, #3033
	movk	x8, #18122, lsl #16
	movk	x8, #43770, lsl #32
	movk	x8, #15978, lsl #48
	fmov	d3, x8
	dup.2d	v3, v3[0]
	fadd	d1, d1, d2
	mov.d	v3[1], v1[0]
	fmul.2d	v1, v3, v0[0]
Lloh122:
	adrp	x8, lCPI16_2@PAGE
Lloh123:
	ldr	q2, [x8, lCPI16_2@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul.2d	v1, v1, v0[0]
Lloh124:
	adrp	x8, lCPI16_3@PAGE
Lloh125:
	ldr	q2, [x8, lCPI16_3@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul.2d	v1, v1, v0[0]
Lloh126:
	adrp	x8, lCPI16_4@PAGE
Lloh127:
	ldr	q2, [x8, lCPI16_4@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul	d0, d0, d1
	mov	d1, v1[1]
	fdiv	d0, d0, d1
	b	LBB16_70
LBB16_65:
	mov.16b	v0, v8
LBB16_66:
	fmov	d1, #0.50000000
	fadd	d0, d0, d1
	fmul	d0, d0, d8
LBB16_67:
	mov	w8, #2
LBB16_68:
	mov.16b	v1, v9
	ucvtf	d2, w8
	mov.16b	v9, v0
	fdiv	d0, d2, d8
	fmul	d0, d9, d0
	fsub	d0, d0, d1
	add	w8, w8, #2
	subs	w21, w21, #1
	b.ne	LBB16_68
	b	LBB16_74
LBB16_69:
	mov.16b	v0, v8
LBB16_70:
	fmov	d1, #0.50000000
	fadd	d0, d0, d1
	fmul	d0, d0, d8
LBB16_71:
	fabs	d1, d9
	fabs	d2, d0
	fcmp	d1, d2
	b.ge	LBB16_73
	fmul	d0, d10, d0
	fdiv	d0, d0, d12
	b	LBB16_74
LBB16_73:
	fmul	d0, d10, d9
	fdiv	d0, d0, d11
LBB16_74:
	fneg	d1, d0
	cmp	w20, #0
	fcsel	d0, d1, d0, ne
LBB16_75:
	.cfi_def_cfa wsp, 96
	ldp	x29, x30, [sp, #80]
	ldp	x20, x19, [sp, #64]
	ldp	x22, x21, [sp, #48]
	ldp	d9, d8, [sp, #32]
	ldp	d11, d10, [sp, #16]
	ldp	d13, d12, [sp], #96
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	.cfi_restore b8
	.cfi_restore b9
	.cfi_restore b10
	.cfi_restore b11
	.cfi_restore b12
	.cfi_restore b13
	ret
	.loh AdrpLdr	Lloh100, Lloh101
	.loh AdrpAdrp	Lloh98, Lloh100
	.loh AdrpLdr	Lloh98, Lloh99
	.loh AdrpLdr	Lloh106, Lloh107
	.loh AdrpAdrp	Lloh104, Lloh106
	.loh AdrpLdr	Lloh104, Lloh105
	.loh AdrpAdrp	Lloh102, Lloh104
	.loh AdrpLdr	Lloh102, Lloh103
	.loh AdrpLdr	Lloh110, Lloh111
	.loh AdrpAdrp	Lloh108, Lloh110
	.loh AdrpLdr	Lloh108, Lloh109
	.loh AdrpLdr	Lloh114, Lloh115
	.loh AdrpAdrp	Lloh112, Lloh114
	.loh AdrpLdr	Lloh112, Lloh113
	.loh AdrpLdr	Lloh120, Lloh121
	.loh AdrpAdrp	Lloh118, Lloh120
	.loh AdrpLdr	Lloh118, Lloh119
	.loh AdrpAdrp	Lloh116, Lloh118
	.loh AdrpLdr	Lloh116, Lloh117
	.loh AdrpLdr	Lloh126, Lloh127
	.loh AdrpAdrp	Lloh124, Lloh126
	.loh AdrpLdr	Lloh124, Lloh125
	.loh AdrpAdrp	Lloh122, Lloh124
	.loh AdrpLdr	Lloh122, Lloh123
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math2jn2yn
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math2jn2yn:
	.cfi_startproc
	stp	d11, d10, [sp, #-64]!
	.cfi_def_cfa_offset 64
	stp	d9, d8, [sp, #16]
	stp	x20, x19, [sp, #32]
	stp	x29, x30, [sp, #48]
	add	x29, sp, #48
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset b8, -40
	.cfi_offset b9, -48
	.cfi_offset b10, -56
	.cfi_offset b11, -64
	.cfi_remember_state
	fmov	x9, d0
	ubfx	x8, x9, #32, #31
	cmp	w9, #0
	cset	w10, ne
	orr	w10, w8, w10
	mov	w11, #2146435072
	cmp	w10, w11
	b.ls	LBB17_2
	mov.16b	v1, v0
	b	LBB17_30
LBB17_2:
	tbz	x9, #63, LBB17_5
	orr	w9, w8, w9
	cbz	w9, LBB17_5
	mov	x8, #9221120237041090560
	fmov	d1, x8
	b	LBB17_30
LBB17_5:
	movi.2d	v1, #0000000000000000
	mov	w9, #2146435072
	cmp	w8, w9
	b.eq	LBB17_30
	cbz	w0, LBB17_13
	tbnz	w0, #31, LBB17_14
	subs	w20, w0, #1
	b.eq	LBB17_21
	mov	w0, #0
	mov	x19, x0
	lsr	w8, w8, #20
	cmp	w8, #1324
	b.ls	LBB17_16
LBB17_10:
	and	w20, w20, #0x3
	mov.16b	v8, v0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3sin3sin
	mov.16b	v9, v0
	cmp	w20, #1
	b.gt	LBB17_23
	cbnz	w20, LBB17_27
	fneg	d9, d9
	b	LBB17_25
LBB17_13:
	.cfi_def_cfa wsp, 64
	ldp	x29, x30, [sp, #48]
	ldp	x20, x19, [sp, #32]
	ldp	d9, d8, [sp, #16]
	ldp	d11, d10, [sp], #64
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore b8
	.cfi_restore b9
	.cfi_restore b10
	.cfi_restore b11
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math2j02y0
LBB17_14:
	.cfi_restore_state
	.cfi_remember_state
	cmn	w0, #1
	b.eq	LBB17_26
	mvn	w20, w0
	mov	x19, x0
	lsr	w8, w8, #20
	cmp	w8, #1324
	b.hi	LBB17_10
LBB17_16:
	mov.16b	v9, v0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math2j02y0
	mov.16b	v8, v0
	mov.16b	v10, v9
	mov.16b	v0, v9
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math2j12y1
	cbz	w20, LBB17_22
	fmov	x8, d0
	and	x9, x8, #0xffffffff00000000
	mov	x8, #-4503599627370496
	cmp	x9, x8
	b.eq	LBB17_22
	mov	w9, #1
	mov	w10, #2
	mov.16b	v2, v10
LBB17_19:
	ucvtf	d1, w10
	fdiv	d1, d1, d2
	fmul	d1, d0, d1
	fsub	d1, d1, d8
	cmp	w9, w20
	b.ge	LBB17_29
	fmov	x11, d1
	and	x11, x11, #0xffffffff00000000
	add	w9, w9, #1
	add	w10, w10, #2
	mov.16b	v8, v0
	mov.16b	v0, v1
	cmp	x11, x8
	b.ne	LBB17_19
	b	LBB17_29
LBB17_21:
	.cfi_def_cfa wsp, 64
	ldp	x29, x30, [sp, #48]
	ldp	x20, x19, [sp, #32]
	ldp	d9, d8, [sp, #16]
	ldp	d11, d10, [sp], #64
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore b8
	.cfi_restore b9
	.cfi_restore b10
	.cfi_restore b11
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math2j12y1
LBB17_22:
	.cfi_restore_state
	mov.16b	v1, v0
	b	LBB17_29
LBB17_23:
	cmp	w20, #2
	b.ne	LBB17_25
	mov.16b	v0, v8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3cos3cos
	fadd	d0, d9, d0
	b	LBB17_28
LBB17_25:
	mov.16b	v0, v8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3cos3cos
	fsub	d0, d9, d0
	b	LBB17_28
LBB17_26:
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math2j12y1
	fneg	d1, d0
	b	LBB17_30
LBB17_27:
	mov.16b	v0, v8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3cos3cos
	fsub	d0, d0, d9
LBB17_28:
	mov	x8, #39789
	movk	x8, #20546, lsl #16
	movk	x8, #3543, lsl #32
	movk	x8, #16354, lsl #48
	fmov	d1, x8
	fmul	d0, d0, d1
	; InlineAsm Start
	fsqrt	d8, d8
	; InlineAsm End
	fdiv	d1, d0, d8
LBB17_29:
	fneg	d0, d1
	tst	w19, #0x1
	fcsel	d1, d0, d1, ne
LBB17_30:
	mov.16b	v0, v1
	.cfi_def_cfa wsp, 64
	ldp	x29, x30, [sp, #48]
	ldp	x20, x19, [sp, #32]
	ldp	d9, d8, [sp, #16]
	ldp	d11, d10, [sp], #64
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore b8
	.cfi_restore b9
	.cfi_restore b10
	.cfi_restore b11
	ret
	.cfi_endproc

	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0
lCPI18_0:
	.quad	0x3efa01a019cb1590
	.quad	0x3da8fae9be8838d4
lCPI18_1:
	.quad	0x3fa555555555554c
	.quad	0xbe927e4f809c52ad
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3cos3cos
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math3cos3cos:
	.cfi_startproc
	fmov	x8, d0
	ubfx	x8, x8, #32, #31
	mov	w9, #8700
	movk	w9, #16361, lsl #16
	cmp	w8, w9
	b.hs	LBB18_3
	fcvtzs	w9, d0
	fmov	d1, #1.00000000
	cmp	w9, #0
	mov	w9, #41118
	movk	w9, #15942, lsl #16
	ccmp	w8, w9, #2, eq
	b.lo	LBB18_12
	fmul	d1, d0, d0
	fmul	d2, d1, d1
	mov	x8, #5520
	movk	x8, #6603, lsl #16
	movk	x8, #416, lsl #32
	movk	x8, #16122, lsl #48
	fmov	d3, x8
	fmul	d3, d1, d3
	mov	x8, #20855
	movk	x8, #5825, lsl #16
	movk	x8, #49516, lsl #32
	movk	x8, #48982, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d3, d1, d3
	mov	x8, #6148914691236517205
	movk	x8, #21836
	movk	x8, #16293, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d3, d1, d3
	fmul	d2, d2, d2
	mov	x8, #14548
	movk	x8, #48776, lsl #16
	movk	x8, #64233, lsl #32
	movk	x8, #48552, lsl #48
	fmov	d4, x8
	fmul	d4, d1, d4
	mov	x8, #45508
	movk	x8, #48564, lsl #16
	movk	x8, #61086, lsl #32
	movk	x8, #15905, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d4, d1, d4
	mov	x8, #21165
	movk	x8, #32924, lsl #16
	movk	x8, #32335, lsl #32
	movk	x8, #48786, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d2, d2, d4
	fadd	d2, d3, d2
	fmov	d3, #0.50000000
	fmul	d3, d1, d3
	fmov	d4, #1.00000000
	fsub	d5, d4, d3
	fsub	d4, d4, d5
	fsub	d3, d4, d3
	fmul	d1, d1, d2
	mov	x8, #-9223372036854775808
	fmov	d2, x8
	fmul	d0, d0, d2
	fadd	d0, d1, d0
	fadd	d0, d3, d0
	fadd	d0, d5, d0
	ret
LBB18_3:
	lsr	w8, w8, #20
	cmp	w8, #2046
	b.ls	LBB18_5
	fsub	d0, d0, d0
	ret
LBB18_5:
	sub	sp, sp, #48
	.cfi_def_cfa_offset 48
	stp	x29, x30, [sp, #32]
	add	x29, sp, #32
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	add	x0, sp, #8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math8rem_pio28rem_pio2
	ldr	w8, [sp, #16]
	ldr	d0, [sp, #8]
	ldr	d1, [sp, #24]
	and	w8, w8, #0x3
	fmul	d2, d0, d0
	fmul	d3, d2, d2
	cmp	w8, #1
	.cfi_def_cfa wsp, 48
	ldp	x29, x30, [sp, #32]
	add	sp, sp, #48
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	b.gt	LBB18_8
	cbnz	w8, LBB18_10
	dup.2d	v4, v2[0]
Lloh128:
	adrp	x8, lCPI18_0@PAGE
Lloh129:
	ldr	q5, [x8, lCPI18_0@PAGEOFF]
	fmul.2d	v5, v5, v2[0]
	fmul	d3, d3, d3
	mov	x8, #20855
	movk	x8, #5825, lsl #16
	movk	x8, #49516, lsl #32
	movk	x8, #48982, lsl #48
	fmov	d6, x8
	fadd.2d	v6, v5, v6
	mov	x8, #45508
	movk	x8, #48564, lsl #16
	movk	x8, #61086, lsl #32
	movk	x8, #15905, lsl #48
	dup.2d	v7, x8
	fsub.2d	v5, v7, v5
	mov.d	v6[1], v5[1]
	fmul.2d	v5, v6, v2[0]
Lloh130:
	adrp	x8, lCPI18_1@PAGE
Lloh131:
	ldr	q6, [x8, lCPI18_1@PAGEOFF]
	fadd.2d	v5, v5, v6
	mov.d	v4[1], v3[0]
	fmul.2d	v3, v4, v5
	faddp.2d	d3, v3
	fmov	d4, #0.50000000
	fmul	d4, d2, d4
	fmov	d5, #1.00000000
	fsub	d6, d5, d4
	fsub	d5, d5, d6
	fsub	d4, d5, d4
	fmul	d2, d2, d3
	fmul	d0, d0, d1
	fsub	d0, d2, d0
	fadd	d0, d4, d0
	fadd	d0, d6, d0
	ret
LBB18_8:
	cmp	w8, #2
	b.ne	LBB18_13
	dup.2d	v4, v2[0]
Lloh132:
	adrp	x8, lCPI18_0@PAGE
Lloh133:
	ldr	q5, [x8, lCPI18_0@PAGEOFF]
	fmul.2d	v5, v5, v2[0]
	fmul	d3, d3, d3
	mov	x8, #20855
	movk	x8, #5825, lsl #16
	movk	x8, #49516, lsl #32
	movk	x8, #48982, lsl #48
	fmov	d6, x8
	mov	x8, #45508
	movk	x8, #48564, lsl #16
	movk	x8, #61086, lsl #32
	movk	x8, #15905, lsl #48
	dup.2d	v7, x8
	fadd.2d	v6, v5, v6
	fsub.2d	v5, v7, v5
	mov.d	v6[1], v5[1]
	fmul.2d	v5, v6, v2[0]
Lloh134:
	adrp	x8, lCPI18_1@PAGE
Lloh135:
	ldr	q6, [x8, lCPI18_1@PAGEOFF]
	fadd.2d	v5, v5, v6
	mov.d	v4[1], v3[0]
	fmul.2d	v3, v4, v5
	faddp.2d	d3, v3
	fmov	d4, #0.50000000
	fmul	d4, d2, d4
	fmov	d5, #1.00000000
	fsub	d6, d5, d4
	fsub	d5, d5, d6
	fsub	d4, d5, d4
	fmul	d2, d2, d3
	fmul	d0, d0, d1
	fsub	d0, d2, d0
	fadd	d0, d4, d0
	fadd	d0, d6, d0
	b	LBB18_11
LBB18_10:
	mov	x8, #65149
	movk	x8, #22449, lsl #16
	movk	x8, #7651, lsl #32
	movk	x8, #16071, lsl #48
	fmov	d4, x8
	fmul	d4, d2, d4
	mov	x8, #25045
	movk	x8, #6593, lsl #16
	movk	x8, #416, lsl #32
	movk	x8, #48938, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d4, d2, d4
	mov	x8, #63654
	movk	x8, #4368, lsl #16
	movk	x8, #4369, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d3, d2, d3
	mov	x8, #54652
	movk	x8, #23247, lsl #16
	movk	x8, #55610, lsl #32
	movk	x8, #15845, lsl #48
	fmov	d5, x8
	fmul	d5, d2, d5
	mov	x8, #40171
	movk	x8, #35371, lsl #16
	movk	x8, #58854, lsl #32
	movk	x8, #48730, lsl #48
	fmov	d6, x8
	fadd	d5, d5, d6
	fmul	d3, d3, d5
	fadd	d3, d3, d4
	fmul	d4, d0, d2
	fmov	d5, #0.50000000
	fmul	d5, d1, d5
	fmul	d3, d4, d3
	fsub	d3, d5, d3
	fmul	d2, d2, d3
	fsub	d1, d2, d1
	mov	x8, #6148914691236517205
	movk	x8, #21833
	movk	x8, #16325, lsl #48
	fmov	d2, x8
	fmul	d2, d4, d2
	fadd	d1, d2, d1
	fsub	d0, d0, d1
LBB18_11:
	fneg	d1, d0
LBB18_12:
	mov.16b	v0, v1
	ret
LBB18_13:
	mov	x8, #65149
	movk	x8, #22449, lsl #16
	movk	x8, #7651, lsl #32
	movk	x8, #16071, lsl #48
	fmov	d4, x8
	fmul	d4, d2, d4
	mov	x8, #25045
	movk	x8, #6593, lsl #16
	movk	x8, #416, lsl #32
	movk	x8, #48938, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d4, d2, d4
	mov	x8, #63654
	movk	x8, #4368, lsl #16
	movk	x8, #4369, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d3, d2, d3
	mov	x8, #54652
	movk	x8, #23247, lsl #16
	movk	x8, #55610, lsl #32
	movk	x8, #15845, lsl #48
	fmov	d5, x8
	fmul	d5, d2, d5
	mov	x8, #40171
	movk	x8, #35371, lsl #16
	movk	x8, #58854, lsl #32
	movk	x8, #48730, lsl #48
	fmov	d6, x8
	fadd	d5, d5, d6
	fmul	d3, d3, d5
	fadd	d3, d3, d4
	fmul	d4, d0, d2
	fmov	d5, #0.50000000
	fmul	d5, d1, d5
	fmul	d3, d4, d3
	fsub	d3, d5, d3
	fmul	d2, d2, d3
	fsub	d1, d2, d1
	mov	x8, #6148914691236517205
	movk	x8, #21833
	movk	x8, #16325, lsl #48
	fmov	d2, x8
	fmul	d2, d4, d2
	fadd	d1, d2, d1
	fsub	d0, d0, d1
	ret
	.loh AdrpLdr	Lloh130, Lloh131
	.loh AdrpLdr	Lloh128, Lloh129
	.loh AdrpLdr	Lloh134, Lloh135
	.loh AdrpLdr	Lloh132, Lloh133
	.cfi_endproc

	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0
lCPI19_0:
	.quad	0xbf9d2a51dbd7194f
	.quad	0x3fb0a54c5536ceba
lCPI19_1:
	.quad	0xbfd4cd7d691cb913
	.quad	0x3fd97779cddadc09
lCPI19_2:
	.quad	0x3fc06eba8214db68
	.quad	0x3ff0000000000000
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3erf3erf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math3erf3erf:
	.cfi_startproc
	stp	x20, x19, [sp, #-32]!
	.cfi_def_cfa_offset 32
	stp	x29, x30, [sp, #16]
	add	x29, sp, #16
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_remember_state
	fmov	x19, d0
	lsr	x8, x19, #32
	ubfx	w8, w8, #20, #11
	cmp	w8, #2046
	b.ls	LBB19_3
	lsr	x8, x19, #62
	and	x8, x8, #0x2
	ucvtf	d1, x8
	fmov	d2, #1.00000000
	fsub	d1, d2, d1
	fdiv	d0, d2, d0
LBB19_2:
	fadd	d0, d0, d1
	b	LBB19_9
LBB19_3:
	ubfx	x0, x19, #32, #31
	mov	w8, #1072365568
	cmp	w0, w8
	b.hs	LBB19_6
	lsr	w8, w0, #20
	cmp	w8, #995
	b.hs	LBB19_10
	fmov	d1, #8.00000000
	fmul	d1, d0, d1
	mov	x8, #56169
	movk	x8, #33300, lsl #16
	movk	x8, #28346, lsl #32
	movk	x8, #16368, lsl #48
	fmov	d2, x8
	fmul	d0, d0, d2
	fadd	d0, d1, d0
	fmov	d1, #0.12500000
	fmul	d0, d0, d1
	b	LBB19_9
LBB19_6:
	fmov	d1, #1.00000000
	lsr	w8, w0, #19
	cmp	w8, #2051
	b.hs	LBB19_8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3erf5erfc2
	fmov	d1, #1.00000000
	fsub	d1, d1, d0
LBB19_8:
	fneg	d0, d1
	cmp	x19, #0
	fcsel	d0, d0, d1, mi
LBB19_9:
	.cfi_def_cfa wsp, 32
	ldp	x29, x30, [sp, #16]
	ldp	x20, x19, [sp], #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	ret
LBB19_10:
	.cfi_restore_state
	fmul	d1, d0, d0
	mov	x8, #24864
	movk	x8, #17058, lsl #16
	movk	x8, #40003, lsl #32
	movk	x8, #48848, lsl #48
	fmov	d2, x8
	fmul	d2, d1, d2
	mov	x8, #6672
	movk	x8, #8732, lsl #16
	movk	x8, #24009, lsl #32
	movk	x8, #16161, lsl #48
	fmov	d3, x8
	fadd	d2, d2, d3
	mov	x8, #5804
	movk	x8, #4608, lsl #16
	movk	x8, #60118, lsl #32
	movk	x8, #16120, lsl #48
	fmov	d3, x8
	dup.2d	v3, v3[0]
	mov.d	v3[1], v2[0]
	fmul.2d	v2, v3, v1[0]
	mov	x8, #26852
	movk	x8, #9062, lsl #16
	movk	x8, #41617, lsl #32
	movk	x8, #49015, lsl #48
	fmov	d3, x8
	fsub.2d	v3, v3, v2
	mov	x8, #27407
	movk	x8, #50387, lsl #16
	movk	x8, #53282, lsl #32
	movk	x8, #16244, lsl #48
	dup.2d	v4, x8
	fadd.2d	v2, v2, v4
	mov.d	v3[1], v2[1]
	fmul.2d	v2, v3, v1[0]
Lloh136:
	adrp	x8, lCPI19_0@PAGE
Lloh137:
	ldr	q3, [x8, lCPI19_0@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul.2d	v2, v2, v1[0]
Lloh138:
	adrp	x8, lCPI19_1@PAGE
Lloh139:
	ldr	q3, [x8, lCPI19_1@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul.2d	v1, v2, v1[0]
Lloh140:
	adrp	x8, lCPI19_2@PAGE
Lloh141:
	ldr	q2, [x8, lCPI19_2@PAGEOFF]
	fadd.2d	v1, v1, v2
	dup.2d	v2, v1[1]
	fdiv.2d	v1, v1, v2
	fmul	d1, d0, d1
	b	LBB19_2
	.loh AdrpLdr	Lloh140, Lloh141
	.loh AdrpAdrp	Lloh138, Lloh140
	.loh AdrpLdr	Lloh138, Lloh139
	.loh AdrpAdrp	Lloh136, Lloh138
	.loh AdrpLdr	Lloh136, Lloh137
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3erf4erfc
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math3erf4erfc:
	.cfi_startproc
	stp	x20, x19, [sp, #-32]!
	.cfi_def_cfa_offset 32
	stp	x29, x30, [sp, #16]
	add	x29, sp, #16
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_remember_state
	fmov	x19, d0
	lsr	x8, x19, #32
	ubfx	w8, w8, #20, #11
	cmp	w8, #2046
	b.ls	LBB20_3
	lsr	x8, x19, #62
	and	x8, x8, #0x2
	ucvtf	d1, x8
	fmov	d2, #1.00000000
	fdiv	d0, d2, d0
	fadd	d0, d0, d1
LBB20_2:
	.cfi_def_cfa wsp, 32
	ldp	x29, x30, [sp, #16]
	ldp	x20, x19, [sp], #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	ret
LBB20_3:
	.cfi_restore_state
	ubfx	x0, x19, #32, #31
	mov	w8, #1072365568
	cmp	w0, w8
	b.hs	LBB20_6
	lsr	w8, w0, #20
	cmp	w8, #967
	b.hs	LBB20_8
	fmov	d1, #1.00000000
	fsub	d0, d1, d0
	b	LBB20_2
LBB20_6:
	mov	w8, #1077673984
	cmp	w0, w8
	b.hs	LBB20_11
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3erf5erfc2
	fmov	d1, #2.00000000
	fsub	d1, d1, d0
	cmn	x19, #1
	fcsel	d0, d0, d1, gt
	b	LBB20_2
LBB20_8:
	fmul	d1, d0, d0
	mov	x9, #5804
	movk	x9, #4608, lsl #16
	movk	x9, #60118, lsl #32
	movk	x9, #48888, lsl #48
	fmov	d2, x9
	fmul	d2, d1, d2
	mov	x9, #26852
	movk	x9, #9062, lsl #16
	movk	x9, #41617, lsl #32
	movk	x9, #49015, lsl #48
	fmov	d3, x9
	fadd	d2, d2, d3
	fmul	d2, d1, d2
	mov	x9, #6479
	movk	x9, #56279, lsl #16
	movk	x9, #10833, lsl #32
	movk	x9, #49053, lsl #48
	fmov	d3, x9
	fadd	d2, d2, d3
	fmul	d2, d1, d2
	mov	x9, #47379
	movk	x9, #26908, lsl #16
	movk	x9, #52605, lsl #32
	movk	x9, #49108, lsl #48
	fmov	d3, x9
	fadd	d2, d2, d3
	fmul	d2, d1, d2
	mov	x9, #56168
	movk	x9, #33300, lsl #16
	movk	x9, #28346, lsl #32
	movk	x9, #16320, lsl #48
	fmov	d3, x9
	fadd	d2, d2, d3
	mov	x9, #24864
	movk	x9, #17058, lsl #16
	movk	x9, #40003, lsl #32
	movk	x9, #48848, lsl #48
	fmov	d3, x9
	fmul	d3, d1, d3
	mov	x9, #6672
	movk	x9, #8732, lsl #16
	movk	x9, #24009, lsl #32
	movk	x9, #16161, lsl #48
	fmov	d4, x9
	fadd	d3, d3, d4
	fmul	d3, d1, d3
	mov	x9, #27407
	movk	x9, #50387, lsl #16
	movk	x9, #53282, lsl #32
	movk	x9, #16244, lsl #48
	fmov	d4, x9
	fadd	d3, d3, d4
	fmul	d3, d1, d3
	mov	x9, #52922
	movk	x9, #21814, lsl #16
	movk	x9, #42316, lsl #32
	movk	x9, #16304, lsl #48
	fmov	d4, x9
	fadd	d3, d3, d4
	fmul	d3, d1, d3
	mov	x9, #56329
	movk	x9, #52698, lsl #16
	movk	x9, #30585, lsl #32
	movk	x9, #16345, lsl #48
	fmov	d4, x9
	fadd	d3, d3, d4
	fmul	d3, d1, d3
	fmov	d1, #1.00000000
	fadd	d3, d3, d1
	fdiv	d2, d2, d3
	tbnz	x19, #63, LBB20_12
	cmp	w8, #1021
	b.lo	LBB20_12
	fmov	d1, #-0.50000000
	fadd	d1, d0, d1
	fmul	d0, d0, d2
	fadd	d0, d1, d0
	fmov	d1, #0.50000000
	fsub	d0, d1, d0
	b	LBB20_2
LBB20_11:
	cmn	x19, #1
	fmov	d0, #2.00000000
	movi.2d	v1, #0000000000000000
	fcsel	d0, d1, d0, gt
	b	LBB20_2
LBB20_12:
	fmul	d2, d0, d2
	fadd	d0, d0, d2
	fsub	d0, d1, d0
	b	LBB20_2
	.cfi_endproc

	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0
lCPI21_0:
	.quad	0xbf843412600d6435
	.quad	0x405b28a3ee48ae2c
lCPI21_1:
	.quad	0x3f61bf380a96073f
	.quad	0x3f888b545735151d
lCPI21_2:
	.quad	0xbfbc63983d3e28ec
	.quad	0x3fc02660e763351f
lCPI21_3:
	.quad	0x3fd45fca805120e4
	.quad	0x3fb2635cd99fe9a7
lCPI21_4:
	.quad	0xbfd7d240fbb8c3f1
	.quad	0x3fe14af092eb6f33
lCPI21_5:
	.quad	0x3fda8d00ad92b34d
	.quad	0x3fbb3e6618eee323
lCPI21_6:
	.quad	0xbf6359b8bef77538
	.quad	0x3ff0000000000000
	.section	__TEXT,__text,regular,pure_instructions
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math3erf5erfc2:
	.cfi_startproc
	sub	sp, sp, #64
	.cfi_def_cfa_offset 64
	stp	d11, d10, [sp, #16]
	stp	d9, d8, [sp, #32]
	stp	x29, x30, [sp, #48]
	add	x29, sp, #48
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset b8, -24
	.cfi_offset b9, -32
	.cfi_offset b10, -40
	.cfi_offset b11, -48
	fabs	d9, d0
	lsr	w8, w0, #18
	cmp	w8, #4093
	b.hs	LBB21_2
	fmov	d0, #-1.00000000
	fadd	d0, d9, d0
Lloh142:
	adrp	x8, lCPI21_1@PAGE
Lloh143:
	ldr	q1, [x8, lCPI21_1@PAGEOFF]
	fmul.2d	v1, v1, v0[0]
	mov	x8, #38379
	movk	x8, #22935, lsl #16
	movk	x8, #10806, lsl #32
	movk	x8, #16290, lsl #48
	fmov	d2, x8
	mov	x8, #56604
	movk	x8, #27473, lsl #16
	movk	x8, #60866, lsl #32
	movk	x8, #16267, lsl #48
	dup.2d	v3, x8
	fsub.2d	v2, v2, v1
	fadd.2d	v1, v1, v3
	mov.d	v2[1], v1[1]
	fmul.2d	v1, v2, v0[0]
Lloh144:
	adrp	x8, lCPI21_2@PAGE
Lloh145:
	ldr	q2, [x8, lCPI21_2@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul.2d	v1, v1, v0[0]
Lloh146:
	adrp	x8, lCPI21_3@PAGE
Lloh147:
	ldr	q2, [x8, lCPI21_3@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul.2d	v1, v1, v0[0]
Lloh148:
	adrp	x8, lCPI21_4@PAGE
Lloh149:
	ldr	q2, [x8, lCPI21_4@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul.2d	v1, v1, v0[0]
Lloh150:
	adrp	x8, lCPI21_5@PAGE
Lloh151:
	ldr	q2, [x8, lCPI21_5@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul.2d	v0, v1, v0[0]
Lloh152:
	adrp	x8, lCPI21_6@PAGE
Lloh153:
	ldr	q1, [x8, lCPI21_6@PAGEOFF]
	fadd.2d	v0, v0, v1
	dup.2d	v1, v0[1]
	fdiv.2d	v0, v0, v1
	mov	x8, #2147483648
	movk	x8, #54522, lsl #32
	movk	x8, #16323, lsl #48
	fmov	d1, x8
	fsub	d0, d1, d0
	b	LBB21_6
LBB21_2:
	fmov	x8, d9
	fmul	d0, d0, d0
	fmov	d1, #1.00000000
	mov	w9, #56173
	movk	w9, #16390, lsl #16
	fdiv	d0, d1, d0
	cmp	w0, w9
	b.hs	LBB21_4
	mov	x9, #49756
	movk	x9, #50842, lsl #16
	movk	x9, #41199, lsl #32
	movk	x9, #49187, lsl #48
	fmov	d1, x9
	fmul	d1, d0, d1
	mov	x9, #54002
	movk	x9, #22500, lsl #16
	movk	x9, #21093, lsl #32
	movk	x9, #49236, lsl #48
	fmov	d2, x9
	fadd	d1, d1, d2
	fmul	d1, d0, d1
	mov	x9, #43954
	movk	x9, #60364, lsl #16
	movk	x9, #4956, lsl #32
	movk	x9, #49255, lsl #48
	fmov	d2, x9
	fadd	d1, d1, d2
	fmul	d1, d0, d1
	mov	x9, #8806
	movk	x9, #33832, lsl #16
	movk	x9, #19633, lsl #32
	movk	x9, #49252, lsl #48
	fmov	d2, x9
	fadd	d1, d1, d2
	fmul	d1, d0, d1
	mov	x9, #41869
	movk	x9, #58571, lsl #16
	movk	x9, #12298, lsl #32
	movk	x9, #49231, lsl #48
	fmov	d2, x9
	fadd	d1, d1, d2
	fmul	d1, d0, d1
	mov	x9, #59174
	movk	x9, #16816, lsl #16
	movk	x9, #7684, lsl #32
	movk	x9, #49189, lsl #48
	fmov	d2, x9
	fadd	d1, d1, d2
	mov	x9, #39522
	movk	x9, #61044, lsl #16
	movk	x9, #61426, lsl #32
	movk	x9, #16302, lsl #48
	fmov	d2, x9
	dup.2d	v2, v2[0]
	mov.d	v2[0], v1[0]
	fmul.2d	v1, v2, v0[0]
	mov	x9, #29536
	movk	x9, #58554, lsl #16
	movk	x9, #13334, lsl #32
	movk	x9, #49126, lsl #48
	fmov	d2, x9
	mov	x9, #19091
	movk	x9, #36424, lsl #16
	movk	x9, #18415, lsl #32
	movk	x9, #16410, lsl #48
	dup.2d	v3, x9
	fadd.2d	v2, v1, v2
	fsub.2d	v1, v3, v1
	mov.d	v2[1], v1[1]
	fmul.2d	v2, v2, v0[0]
Lloh154:
	adrp	x9, lCPI21_0@PAGE
Lloh155:
	ldr	q3, [x9, lCPI21_0@PAGEOFF]
	mov	x9, #30343
	movk	x9, #48496, lsl #16
	movk	x9, #42681, lsl #32
	movk	x9, #16435, lsl #48
	fmov	d1, x9
	fadd.2d	v6, v2, v3
	mov	x9, #59169
	movk	x9, #21098, lsl #16
	movk	x9, #13580, lsl #32
	movk	x9, #16481, lsl #48
	fmov	d2, x9
	mov	x9, #6769
	movk	x9, #54666, lsl #16
	movk	x9, #10509, lsl #32
	movk	x9, #16507, lsl #48
	fmov	d3, x9
	mov	x9, #10344
	movk	x9, #8684, lsl #16
	movk	x9, #11033, lsl #32
	movk	x9, #16516, lsl #48
	fmov	d4, x9
	mov	x9, #788
	movk	x9, #22384, lsl #16
	movk	x9, #53281, lsl #32
	movk	x9, #16506, lsl #48
	b	LBB21_5
LBB21_4:
	mov	x9, #14399
	movk	x9, #39900, lsl #16
	movk	x9, #14414, lsl #32
	movk	x9, #49278, lsl #48
	fmov	d1, x9
	fmul	d1, d0, d1
	mov	x9, #22930
	movk	x9, #27182, lsl #16
	movk	x9, #1121, lsl #32
	movk	x9, #49296, lsl #48
	fmov	d2, x9
	fadd	d1, d1, d2
	fmul	d1, d0, d1
	mov	x9, #61992
	movk	x9, #4981, lsl #16
	movk	x9, #60552, lsl #32
	movk	x9, #49283, lsl #48
	fmov	d2, x9
	fadd	d1, d1, d2
	fmul	d1, d0, d1
	mov	x9, #60824
	movk	x9, #17349, lsl #16
	movk	x9, #5213, lsl #32
	movk	x9, #49252, lsl #48
	fmov	d2, x9
	fadd	d1, d1, d2
	fmul	d1, d0, d1
	mov	x9, #39258
	movk	x9, #21855, lsl #16
	movk	x9, #49673, lsl #32
	movk	x9, #49201, lsl #48
	fmov	d2, x9
	fadd	d1, d1, d2
	fmul	d1, d0, d1
	mov	x9, #34270
	movk	x9, #28866, lsl #16
	movk	x9, #37818, lsl #32
	movk	x9, #49129, lsl #48
	fmov	d2, x9
	fadd	d1, d1, d2
	mov	x9, #11618
	movk	x9, #17009, lsl #16
	movk	x9, #28898, lsl #32
	movk	x9, #16438, lsl #48
	fmov	d2, x9
	dup.2d	v2, v2[0]
	mov.d	v2[0], v1[0]
	fmul.2d	v1, v2, v0[0]
	mov	x9, #28490
	movk	x9, #14824, lsl #16
	movk	x9, #13330, lsl #32
	movk	x9, #49028, lsl #48
	fmov	d2, x9
	fadd.2d	v6, v1, v2
	mov	x9, #59235
	movk	x9, #59295, lsl #16
	movk	x9, #43124, lsl #32
	movk	x9, #16509, lsl #48
	dup.2d	v2, x9
	fsub.2d	v2, v2, v1
	mov	x9, #20880
	movk	x9, #9757, lsl #16
	movk	x9, #22155, lsl #32
	movk	x9, #16446, lsl #48
	fmov	d1, x9
	mov.d	v6[1], v2[1]
	mov	x9, #40714
	movk	x9, #8731, lsl #16
	movk	x9, #23726, lsl #32
	movk	x9, #16500, lsl #48
	fmov	d2, x9
	mov	x9, #20760
	movk	x9, #6301, lsl #16
	movk	x9, #747, lsl #32
	movk	x9, #16536, lsl #48
	fmov	d3, x9
	mov	x9, #9322
	movk	x9, #26764, lsl #16
	movk	x9, #65463, lsl #32
	movk	x9, #16552, lsl #48
	fmov	d4, x9
	mov	x9, #15334
	movk	x9, #52959, lsl #16
	movk	x9, #61977, lsl #32
	movk	x9, #16547, lsl #48
LBB21_5:
	fmov	d5, x9
	str	q6, [sp]
	fmul.d	d6, d0, v6[1]
	fadd	d5, d6, d5
	fmul	d5, d0, d5
	fadd	d4, d5, d4
	fmul	d4, d0, d4
	fadd	d3, d4, d3
	fmul	d3, d0, d3
	fadd	d2, d3, d2
	fmul	d2, d0, d2
	fadd	d1, d2, d1
	fmul	d0, d0, d1
	fmov	d1, #1.00000000
	fadd	d10, d0, d1
	and	x8, x8, #0x7fffffff00000000
	fmov	d11, x8
	fmul	d0, d11, d11
	fmov	d1, #-0.56250000
	fsub	d0, d1, d0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3exp3exp
	mov.16b	v8, v0
	fsub	d0, d11, d9
	fadd	d1, d9, d11
	fmul	d0, d0, d1
	ldr	q1, [sp]
	fdiv	d1, d1, d10
	fadd	d0, d0, d1
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3exp3exp
	fmul	d0, d8, d0
	fdiv	d0, d0, d9
LBB21_6:
	.cfi_def_cfa wsp, 64
	ldp	x29, x30, [sp, #48]
	ldp	d9, d8, [sp, #32]
	ldp	d11, d10, [sp, #16]
	add	sp, sp, #64
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore b8
	.cfi_restore b9
	.cfi_restore b10
	.cfi_restore b11
	ret
	.loh AdrpLdr	Lloh152, Lloh153
	.loh AdrpAdrp	Lloh150, Lloh152
	.loh AdrpLdr	Lloh150, Lloh151
	.loh AdrpAdrp	Lloh148, Lloh150
	.loh AdrpLdr	Lloh148, Lloh149
	.loh AdrpAdrp	Lloh146, Lloh148
	.loh AdrpLdr	Lloh146, Lloh147
	.loh AdrpAdrp	Lloh144, Lloh146
	.loh AdrpLdr	Lloh144, Lloh145
	.loh AdrpLdr	Lloh142, Lloh143
	.loh AdrpLdr	Lloh154, Lloh155
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3exp3exp
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math3exp3exp:
	.cfi_startproc
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	.cfi_remember_state
	fmov	x9, d0
	lsr	x8, x9, #63
	ubfx	x9, x9, #32, #31
	mov	w10, #9003
	movk	w10, #16518, lsl #16
	cmp	w9, w10
	b.lo	LBB22_4
	fcmp	d0, d0
	b.vs	LBB22_25
	mov	x9, #14831
	movk	x9, #65274, lsl #16
	movk	x9, #11842, lsl #32
	movk	x9, #16518, lsl #48
	fmov	d1, x9
	fcmp	d0, d1
	b.le	LBB22_9
	mov	x8, #9214364837600034816
	fmov	d1, x8
	fmul	d1, d0, d1
	b	LBB22_24
LBB22_4:
	mov	w10, #11842
	movk	w10, #16342, lsl #16
	cmp	w9, w10
	b.ls	LBB22_7
	mov	w10, #41649
	movk	w10, #16368, lsl #16
	cmp	w9, w10
	b.hi	LBB22_11
	eor	w9, w8, #0x1
	sub	w8, w9, w8
	b	LBB22_12
LBB22_7:
	mov	w8, #1043333120
	cmp	w9, w8
	b.ls	LBB22_20
	mov	w8, #0
	movi.2d	v1, #0000000000000000
	mov.16b	v2, v0
	b	LBB22_13
LBB22_9:
	mov	x9, #48338
	movk	x9, #56698, lsl #16
	movk	x9, #9003, lsl #32
	movk	x9, #49286, lsl #48
	fmov	d1, x9
	fcmp	d0, d1
	b.pl	LBB22_11
	mov	x9, #-5287225962532962304
	fmov	d1, x9
	fdiv	d1, d1, d0
	fcvt	s1, d1
	str	s1, [sp, #4]
	ldr	s1, [sp, #4]
	movi.2d	v1, #0000000000000000
	mov	x9, #12369
	movk	x9, #54573, lsl #16
	movk	x9, #18704, lsl #32
	movk	x9, #49287, lsl #48
	fmov	d2, x9
	fcmp	d0, d2
	b.mi	LBB22_24
LBB22_11:
	mov	x9, #33534
	movk	x9, #25899, lsl #16
	movk	x9, #5447, lsl #32
	movk	x9, #16375, lsl #48
	fmov	d1, x9
Lloh156:
	adrp	x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.48@PAGE
Lloh157:
	add	x9, x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.48@PAGEOFF
	fmul	d1, d0, d1
	ldr	d2, [x9, w8, uxtw #3]
	fadd	d1, d1, d2
	fcvtzs	w8, d1
LBB22_12:
	scvtf	d1, w8
	mov	x9, #4276092928
	movk	x9, #11842, lsl #32
	movk	x9, #49126, lsl #48
	fmov	d2, x9
	fmul	d2, d1, d2
	fadd	d0, d0, d2
	mov	x9, #15478
	movk	x9, #13689, lsl #16
	movk	x9, #14831, lsl #32
	movk	x9, #15850, lsl #48
	fmov	d2, x9
	fmul	d1, d1, d2
	fsub	d2, d0, d1
LBB22_13:
	fmul	d3, d2, d2
	mov	x9, #42192
	movk	x9, #29374, lsl #16
	movk	x9, #14185, lsl #32
	movk	x9, #15974, lsl #48
	fmov	d4, x9
	fmul	d4, d3, d4
	mov	x9, #27633
	movk	x9, #50642, lsl #16
	movk	x9, #48449, lsl #32
	movk	x9, #48827, lsl #48
	fmov	d5, x9
	fadd	d4, d4, d5
	fmul	d4, d3, d4
	mov	x9, #56876
	movk	x9, #44837, lsl #16
	movk	x9, #22122, lsl #32
	movk	x9, #16145, lsl #48
	fmov	d5, x9
	fadd	d4, d4, d5
	fmul	d4, d3, d4
	mov	x9, #48531
	movk	x9, #5822, lsl #16
	movk	x9, #49516, lsl #32
	movk	x9, #48998, lsl #48
	fmov	d5, x9
	fadd	d4, d4, d5
	fmul	d4, d3, d4
	mov	x9, #6148914691236517205
	movk	x9, #21822
	movk	x9, #16325, lsl #48
	fmov	d5, x9
	fadd	d4, d4, d5
	fmul	d3, d3, d4
	fsub	d3, d2, d3
	fmul	d2, d2, d3
	fmov	d4, #2.00000000
	fsub	d3, d4, d3
	fdiv	d2, d2, d3
	fsub	d1, d2, d1
	fadd	d0, d0, d1
	fmov	d1, #1.00000000
	fadd	d1, d0, d1
	cbz	w8, LBB22_24
	subs	w9, w8, #1023
	b.le	LBB22_17
	mov	x10, #9214364837600034816
	fmov	d0, x10
	fmul	d1, d1, d0
	cmp	w8, #2046
	b.ls	LBB22_21
	mov	x9, #9214364837600034816
	fmov	d0, x9
	fmul	d1, d1, d0
	mov	w9, #3069
	cmp	w8, #3069
	csel	w8, w8, w9, lo
	sub	w8, w8, #2046
	b	LBB22_23
LBB22_17:
	cmn	w8, #1022
	b.ge	LBB22_23
	mov	x9, #243194379878006784
	fmov	d0, x9
	fmul	d1, d1, d0
	cmn	w8, #1992
	b.hi	LBB22_22
	fmov	d0, x9
	fmul	d1, d1, d0
	cmn	w8, #2960
	mov	w9, #-2960
	csel	w8, w8, w9, hi
	add	w8, w8, #1938
	b	LBB22_23
LBB22_20:
	mov	x8, #9214364837600034816
	fmov	d1, x8
	fadd	d1, d0, d1
	str	d1, [sp, #8]
	ldr	d1, [sp, #8]
	fmov	d1, #1.00000000
	fadd	d1, d0, d1
	b	LBB22_24
LBB22_21:
	mov	x8, x9
	b	LBB22_23
LBB22_22:
	add	w8, w8, #969
LBB22_23:
	add	w8, w8, #1023
	lsl	x8, x8, #52
	fmov	d0, x8
	fmul	d1, d1, d0
LBB22_24:
	mov.16b	v0, v1
	add	sp, sp, #16
	.cfi_def_cfa_offset 0
	ret
LBB22_25:
	.cfi_restore_state
	mov.16b	v1, v0
	b	LBB22_24
	.loh AdrpAdd	Lloh156, Lloh157
	.cfi_endproc

	.section	__TEXT,__literal8,8byte_literals
	.p2align	3, 0x0
lCPI23_0:
	.long	0xb947352e
	.long	0x3c7fe744
lCPI23_1:
	.long	0x3c800000
	.long	0x3f800000
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3j0f3j0f
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math3j0f3j0f:
	.cfi_startproc
	mov.16b	v1, v0
	fabs	s0, s0
	fmov	w0, s0
	lsr	w8, w0, #23
	cmp	w8, #254
	b.ls	LBB23_2
	fmul	s0, s1, s1
	fmov	s1, #1.00000000
	fdiv	s0, s1, s0
	ret
LBB23_2:
	lsr	w9, w0, #30
	cbz	w9, LBB23_4
	mov	w1, #0
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math3j0f6common
LBB23_4:
	lsr	w9, w0, #25
	cmp	w9, #28
	b.ls	LBB23_6
	fmul	s1, s1, s1
	mov	w8, #17896
	movk	w8, #12448, lsl #16
	fmov	s2, w8
	fmul	s2, s1, s2
	mov	w8, #55974
	movk	w8, #13577, lsl #16
	fmov	s3, w8
	fadd	s2, s2, s3
	mov	w8, #44860
	movk	w8, #12702, lsl #16
	fmov	s3, w8
	dup.2s	v3, v3[0]
	mov.s	v3[1], v2[0]
	fmul.2s	v2, v3, v1[0]
	mov	w8, #36488
	movk	w8, #13813, lsl #16
	fmov	s3, w8
	fsub.2s	v3, v3, v2
	mov	w8, #13975
	movk	w8, #14581, lsl #16
	dup.2s	v4, w8
	fadd.2s	v2, v2, v4
	mov.s	v3[1], v2[1]
	fmul.2s	v2, v3, v1[0]
Lloh158:
	adrp	x8, lCPI23_0@PAGE
Lloh159:
	ldr	d3, [x8, lCPI23_0@PAGEOFF]
	fadd.2s	v2, v2, v3
	fmul.2s	v2, v2, v1[0]
Lloh160:
	adrp	x8, lCPI23_1@PAGE
Lloh161:
	ldr	d3, [x8, lCPI23_1@PAGEOFF]
	fadd.2s	v2, v2, v3
	fmul	s3, s1, s2
	fmov	s4, #0.50000000
	fmul	s0, s0, s4
	fmov	s4, #1.00000000
	fadd	s5, s0, s4
	fsub	s0, s4, s0
	fmul	s0, s5, s0
	mov	s2, v2[1]
	fdiv	s2, s3, s2
	fmul	s1, s1, s2
	fadd	s0, s0, s1
	ret
LBB23_6:
	fmov	s1, #0.25000000
	fmul	s1, s0, s1
	fmul	s1, s0, s1
	cmp	w8, #66
	fcsel	s0, s1, s0, hi
	fmov	s1, #1.00000000
	fsub	s0, s1, s0
	ret
	.loh AdrpLdr	Lloh160, Lloh161
	.loh AdrpAdrp	Lloh158, Lloh160
	.loh AdrpLdr	Lloh158, Lloh159
	.cfi_endproc

	.section	__TEXT,__literal8,8byte_literals
	.p2align	3, 0x0
lCPI24_0:
	.long	0x3e91e9ee
	.long	0x3e789e26
lCPI24_1:
	.long	0x3f2aaaaa
	.long	0x3eccce13
lCPI24_2:
	.long	0xb947352e
	.long	0x3c7fe744
lCPI24_3:
	.long	0x3c800000
	.long	0x3f800000
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3j0f3y0f
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math3j0f3y0f:
	.cfi_startproc
	fcmp	s0, #0.0
	b.ne	LBB24_2
	mov	w8, #-8388608
	fmov	s0, w8
	ret
LBB24_2:
	fmov	w0, s0
	tbnz	w0, #31, LBB24_5
	lsr	w8, w0, #23
	cmp	w8, #254
	b.ls	LBB24_6
	fmov	s1, #1.00000000
	fdiv	s0, s1, s0
	ret
LBB24_5:
	mov	w8, #2143289344
	fmov	s0, w8
	ret
LBB24_6:
	lsr	w9, w0, #30
	cbz	w9, LBB24_8
	mov	w1, #1
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math3j0f6common
LBB24_8:
	lsr	w9, w0, #24
	cmp	w9, #56
	b.ls	LBB24_11
	fmul	s2, s0, s0
	mov	w8, #8683
	movk	w8, #44591, lsl #16
	fmov	s1, w8
	fmul	s1, s2, s1
	mov	w8, #698
	movk	w8, #12968, lsl #16
	fmov	s3, w8
	fadd	s1, s1, s3
	fmul	s1, s2, s1
	mov	w8, #62780
	movk	w8, #46719, lsl #16
	fmov	s3, w8
	fadd	s1, s1, s3
	fmul	s1, s2, s1
	mov	w8, #10857
	movk	w8, #14774, lsl #16
	fmov	s3, w8
	fadd	s1, s1, s3
	fmul	s1, s2, s1
	mov	w8, #26438
	movk	w8, #48226, lsl #16
	fmov	s3, w8
	fadd	s1, s1, s3
	fmul	s1, s2, s1
	mov	w8, #59405
	movk	w8, #15924, lsl #16
	fmov	s3, w8
	fadd	s1, s1, s3
	fmul	s1, s2, s1
	mov	w8, #9909
	movk	w8, #48535, lsl #16
	fmov	s3, w8
	fadd	s1, s1, s3
	mov	w8, #32962
	movk	w8, #12274, lsl #16
	fmov	s3, w8
	fmul	s3, s2, s3
	mov	w8, #8556
	movk	w8, #13451, lsl #16
	fmov	s4, w8
	fadd	s3, s3, s4
	fmul	s3, s2, s3
	mov	w8, #26080
	movk	w8, #14495, lsl #16
	fmov	s4, w8
	fadd	s3, s3, s4
	fmul	s3, s2, s3
	mov	w8, #37765
	movk	w8, #15440, lsl #16
	fmov	s4, w8
	fadd	s3, s3, s4
	fmul	s4, s2, s3
	fmov	s3, #1.00000000
	fadd	s4, s4, s3
	fdiv	s1, s1, s4
	lsr	w8, w0, #25
	cmp	w8, #28
	b.ls	LBB24_12
	mov	w8, #17896
	movk	w8, #12448, lsl #16
	fmov	s4, w8
	fmul	s4, s2, s4
	mov	w8, #55974
	movk	w8, #13577, lsl #16
	fmov	s5, w8
	fadd	s4, s4, s5
	mov	w8, #44860
	movk	w8, #12702, lsl #16
	fmov	s5, w8
	dup.2s	v5, v5[0]
	mov.s	v5[1], v4[0]
	fmul.2s	v4, v5, v2[0]
	mov	w8, #36488
	movk	w8, #13813, lsl #16
	fmov	s5, w8
	fsub.2s	v5, v5, v4
	mov	w8, #13975
	movk	w8, #14581, lsl #16
	dup.2s	v6, w8
	fadd.2s	v4, v4, v6
	mov.s	v5[1], v4[1]
	fmul.2s	v4, v5, v2[0]
Lloh162:
	adrp	x8, lCPI24_2@PAGE
Lloh163:
	ldr	d5, [x8, lCPI24_2@PAGEOFF]
	fadd.2s	v4, v4, v5
	fmul.2s	v4, v4, v2[0]
Lloh164:
	adrp	x8, lCPI24_3@PAGE
Lloh165:
	ldr	d5, [x8, lCPI24_3@PAGEOFF]
	fadd.2s	v4, v4, v5
	fmul	s5, s2, s4
	fmov	s6, #0.50000000
	fmul	s0, s0, s6
	fadd	s6, s0, s3
	fsub	s0, s3, s0
	fmul	s0, s6, s0
	mov	s3, v4[1]
	fdiv	s3, s5, s3
	fmul	s2, s2, s3
	fadd	s0, s0, s2
	movi.2d	v2, #0000000000000000
	mov	w8, #1065353216
	cmp	w0, w8
	b.ne	LBB24_13
	b	LBB24_14
LBB24_11:
	movi.2s	v1, #76, lsl #24
	fmul	s0, s0, s1
	fmov	w9, s0
	cmp	w8, #0
	csel	w8, w0, w9, ne
	mov	w9, #-152
	mov	w10, #-127
	csel	w9, w10, w9, ne
	add	w8, w8, #1199, lsl #12
	add	w8, w8, #2829
	add	w9, w9, w8, lsr #23
	and	w8, w8, #0x7fffff
	mov	w10, #1267
	movk	w10, #16181, lsl #16
	add	w8, w8, w10
	fmov	s0, w8
	fmov	s1, #-1.00000000
	fadd	s0, s0, s1
	fmov	s1, #2.00000000
	fadd	s1, s0, s1
	fdiv	s1, s0, s1
	fmul	s2, s1, s1
	fmul	s3, s2, s2
	dup.2s	v4, v3[0]
Lloh166:
	adrp	x8, lCPI24_0@PAGE
Lloh167:
	ldr	d5, [x8, lCPI24_0@PAGEOFF]
	fmul.2s	v3, v5, v3[0]
Lloh168:
	adrp	x8, lCPI24_1@PAGE
Lloh169:
	ldr	d5, [x8, lCPI24_1@PAGEOFF]
	fadd.2s	v3, v3, v5
	mov.s	v4[0], v2[0]
	fmul.2s	v2, v4, v3
	faddp.2s	s2, v2
	fmov	s3, #0.50000000
	fmul	s3, s0, s3
	fmul	s3, s0, s3
	scvtf	s4, w9
	fadd	s2, s3, s2
	fmul	s1, s1, s2
	mov	w8, #63441
	movk	w8, #14103, lsl #16
	fmov	s2, w8
	fmul	s2, s4, s2
	fadd	s1, s2, s1
	fsub	s1, s1, s3
	fadd	s0, s0, s1
	mov	w8, #29056
	movk	w8, #16177, lsl #16
	fmov	s1, w8
	fmul	s1, s4, s1
	fadd	s0, s1, s0
	mov	w8, #63875
	movk	w8, #16162, lsl #16
	fmov	s1, w8
	fmul	s0, s0, s1
	mov	w8, #9909
	movk	w8, #48535, lsl #16
	fmov	s1, w8
	fadd	s0, s0, s1
	ret
LBB24_12:
	fmov	s2, #-0.25000000
	fmul	s2, s0, s2
	fmul	s0, s0, s2
	fadd	s0, s0, s3
LBB24_13:
	add	w8, w0, #1199, lsl #12
	add	w8, w8, #2829
	lsr	w9, w8, #23
	sub	w9, w9, #127
	and	w8, w8, #0x7fffff
	mov	w10, #1267
	movk	w10, #16181, lsl #16
	add	w8, w8, w10
	fmov	s2, w8
	fmov	s3, #-1.00000000
	fadd	s2, s2, s3
	fmov	s3, #2.00000000
	fadd	s3, s2, s3
	fdiv	s3, s2, s3
	fmul	s4, s3, s3
	fmul	s5, s4, s4
	dup.2s	v6, v5[0]
Lloh170:
	adrp	x8, lCPI24_0@PAGE
Lloh171:
	ldr	d7, [x8, lCPI24_0@PAGEOFF]
	fmul.2s	v5, v7, v5[0]
Lloh172:
	adrp	x8, lCPI24_1@PAGE
Lloh173:
	ldr	d7, [x8, lCPI24_1@PAGEOFF]
	fadd.2s	v5, v5, v7
	mov.s	v6[0], v4[0]
	fmul.2s	v4, v6, v5
	faddp.2s	s4, v4
	fmov	s5, #0.50000000
	fmul	s5, s2, s5
	fmul	s5, s2, s5
	scvtf	s6, w9
	fadd	s4, s5, s4
	fmul	s3, s3, s4
	mov	w8, #63441
	movk	w8, #14103, lsl #16
	fmov	s4, w8
	fmul	s4, s6, s4
	fadd	s3, s4, s3
	fsub	s3, s3, s5
	fadd	s2, s2, s3
	mov	w8, #29056
	movk	w8, #16177, lsl #16
	fmov	s3, w8
	fmul	s3, s6, s3
	fadd	s2, s3, s2
LBB24_14:
	fmul	s0, s0, s2
	mov	w8, #63875
	movk	w8, #16162, lsl #16
	fmov	s2, w8
	fmul	s0, s0, s2
	fadd	s0, s1, s0
	ret
	.loh AdrpLdr	Lloh164, Lloh165
	.loh AdrpAdrp	Lloh162, Lloh164
	.loh AdrpLdr	Lloh162, Lloh163
	.loh AdrpLdr	Lloh168, Lloh169
	.loh AdrpAdrp	Lloh166, Lloh168
	.loh AdrpLdr	Lloh166, Lloh167
	.loh AdrpLdr	Lloh172, Lloh173
	.loh AdrpAdrp	Lloh170, Lloh172
	.loh AdrpLdr	Lloh170, Lloh171
	.cfi_endproc

	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math3j0f6common:
	.cfi_startproc
	stp	d11, d10, [sp, #-64]!
	.cfi_def_cfa_offset 64
	stp	d9, d8, [sp, #16]
	stp	x20, x19, [sp, #32]
	stp	x29, x30, [sp, #48]
	add	x29, sp, #48
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset b8, -40
	.cfi_offset b9, -48
	.cfi_offset b10, -56
	.cfi_offset b11, -64
	mov	x19, x1
	mov.16b	v8, v0
	mov	x20, x0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4sinf4sinf
	mov.16b	v9, v0
	mov.16b	v0, v8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4cosf4cosf
	fneg	s1, s0
	cmp	w19, #0
	fcsel	s11, s1, s0, ne
	fadd	s10, s9, s11
	lsr	w8, w20, #24
	cmp	w8, #127
	b.hs	LBB25_5
	fadd	s0, s8, s8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4cosf4cosf
	fneg	s1, s0
	fmul	s0, s9, s11
	fcmp	s0, #0.0
	b.pl	LBB25_3
	fsub	s0, s9, s11
	fdiv	s10, s1, s0
	lsr	w8, w20, #23
	cmp	w8, #176
	b.ls	LBB25_4
	b	LBB25_5
LBB25_3:
	fdiv	s0, s1, s10
	lsr	w8, w20, #23
	cmp	w8, #176
	b.hi	LBB25_5
LBB25_4:
	mov	w8, #29674
	movk	w8, #16529, lsl #16
	fneg	s1, s0
	cmp	w19, #0
	fcsel	s0, s1, s0, ne
	fmov	w10, s8
Lloh174:
	adrp	x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.55@PAGE
Lloh175:
	add	x11, x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.55@PAGEOFF
Lloh176:
	adrp	x12, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.56@PAGE
Lloh177:
	add	x12, x12, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.56@PAGEOFF
	and	w9, w10, #0x7fffffff
	ubfx	w10, w10, #24, #7
Lloh178:
	adrp	x13, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.53@PAGE
Lloh179:
	add	x13, x13, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.53@PAGEOFF
Lloh180:
	adrp	x14, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.54@PAGE
Lloh181:
	add	x14, x14, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.54@PAGEOFF
Lloh182:
	adrp	x15, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.50@PAGE
Lloh183:
	add	x15, x15, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.50@PAGEOFF
Lloh184:
	adrp	x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.52@PAGE
Lloh185:
	add	x16, x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.52@PAGEOFF
	mov	w17, #55574
	movk	w17, #16438, lsl #16
	cmp	w9, w17
	csel	x15, x16, x15, hi
Lloh186:
	adrp	x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.49@PAGE
Lloh187:
	add	x16, x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.49@PAGEOFF
Lloh188:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.51@PAGE
Lloh189:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.51@PAGEOFF
	csel	x16, x0, x16, hi
	cmp	w9, w8
	csel	x14, x14, x15, hi
	csel	x13, x13, x16, hi
	cmp	w10, #64
	csel	x10, x12, x14, hi
	csel	x11, x11, x13, hi
	fmul	s2, s8, s8
	fmov	s1, #1.00000000
	fdiv	s2, s1, s2
	ldp	s3, s4, [x11]
	ldp	s5, s6, [x11, #8]
	ldp	s7, s16, [x11, #16]
	fmul	s16, s2, s16
	fadd	s7, s7, s16
	fmul	s7, s2, s7
	fadd	s6, s6, s7
	fmul	s6, s2, s6
	fadd	s5, s5, s6
	fmul	s5, s2, s5
	fadd	s4, s4, s5
	fmul	s4, s2, s4
	fadd	s3, s3, s4
	ldp	s4, s5, [x10]
	ldp	s6, s7, [x10, #8]
	ldr	s16, [x10, #16]
	fmul	s16, s2, s16
	fadd	s7, s7, s16
	fmul	s7, s2, s7
	fadd	s6, s6, s7
	fmul	s6, s2, s6
	fadd	s5, s5, s6
	fmul	s5, s2, s5
	fadd	s4, s4, s5
	fmul	s4, s2, s4
	fadd	s4, s4, s1
	fdiv	s3, s3, s4
	fadd	s3, s3, s1
	fmul	s3, s10, s3
Lloh190:
	adrp	x10, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.63@PAGE
Lloh191:
	add	x10, x10, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.63@PAGEOFF
Lloh192:
	adrp	x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.64@PAGE
Lloh193:
	add	x11, x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.64@PAGEOFF
	lsr	w12, w9, #24
Lloh194:
	adrp	x13, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.61@PAGE
Lloh195:
	add	x13, x13, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.61@PAGEOFF
Lloh196:
	adrp	x14, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.62@PAGE
Lloh197:
	add	x14, x14, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.62@PAGEOFF
Lloh198:
	adrp	x15, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.58@PAGE
Lloh199:
	add	x15, x15, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.58@PAGEOFF
Lloh200:
	adrp	x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.60@PAGE
Lloh201:
	add	x16, x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.60@PAGEOFF
	cmp	w9, w17
	csel	x15, x16, x15, hi
Lloh202:
	adrp	x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.57@PAGE
Lloh203:
	add	x16, x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.57@PAGEOFF
Lloh204:
	adrp	x17, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.59@PAGE
Lloh205:
	add	x17, x17, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.59@PAGEOFF
	csel	x16, x17, x16, hi
	cmp	w9, w8
	csel	x8, x14, x15, hi
	csel	x9, x13, x16, hi
	cmp	w12, #64
	csel	x8, x11, x8, hi
	csel	x9, x10, x9, hi
	ldp	s4, s5, [x8, #16]
	fmul	s5, s2, s5
	fadd	s4, s4, s5
	ldp	d5, d6, [x9]
	ldr	d7, [x9, #16]
	ldp	d16, d17, [x8]
	dup.2s	v18, v7[1]
	mov.s	v18[1], v4[0]
	mov.s	v7[1], v17[1]
	fmul.2s	v4, v18, v2[0]
	fadd.2s	v4, v7, v4
	fmul.2s	v4, v4, v2[0]
	ext.8b	v7, v6, v17, #4
	fadd.2s	v4, v7, v4
	fmul.2s	v4, v4, v2[0]
	mov.s	v6[1], v16[1]
	fadd.2s	v4, v6, v4
	fmul.2s	v4, v4, v2[0]
	ext.8b	v6, v5, v16, #4
	fadd.2s	v4, v6, v4
	fmul.2s	v2, v4, v2[0]
	mov.s	v5[1], v1[0]
	fadd.2s	v1, v5, v2
	dup.2s	v2, v1[1]
	fdiv.2s	v1, v1, v2
	fmov	s2, #-0.12500000
	fadd	s1, s1, s2
	fdiv	s1, s1, s8
	fmul	s0, s0, s1
	fsub	s10, s3, s0
LBB25_5:
	mov	w8, #28347
	movk	w8, #16144, lsl #16
	fmov	s0, w8
	fmul	s0, s10, s0
	; InlineAsm Start
	fsqrt	s8, s8
	; InlineAsm End
	fdiv	s0, s0, s8
	.cfi_def_cfa wsp, 64
	ldp	x29, x30, [sp, #48]
	ldp	x20, x19, [sp, #32]
	ldp	d9, d8, [sp, #16]
	ldp	d11, d10, [sp], #64
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore b8
	.cfi_restore b9
	.cfi_restore b10
	.cfi_restore b11
	ret
	.loh AdrpAdd	Lloh204, Lloh205
	.loh AdrpAdd	Lloh202, Lloh203
	.loh AdrpAdd	Lloh200, Lloh201
	.loh AdrpAdd	Lloh198, Lloh199
	.loh AdrpAdd	Lloh196, Lloh197
	.loh AdrpAdd	Lloh194, Lloh195
	.loh AdrpAdd	Lloh192, Lloh193
	.loh AdrpAdd	Lloh190, Lloh191
	.loh AdrpAdd	Lloh188, Lloh189
	.loh AdrpAdd	Lloh186, Lloh187
	.loh AdrpAdd	Lloh184, Lloh185
	.loh AdrpAdd	Lloh182, Lloh183
	.loh AdrpAdd	Lloh180, Lloh181
	.loh AdrpAdd	Lloh178, Lloh179
	.loh AdrpAdd	Lloh176, Lloh177
	.loh AdrpAdd	Lloh174, Lloh175
	.cfi_endproc

	.section	__TEXT,__literal8,8byte_literals
	.p2align	3, 0x0
lCPI26_0:
	.long	0xb7862e36
	.long	0x3942fab6
lCPI26_1:
	.long	0x3ab86cfd
	.long	0x3c9ce859
lCPI26_2:
	.long	0xbd800000
	.long	0x3f800000
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3j1f3j1f
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math3j1f3j1f:
	.cfi_startproc
	fabs	s1, s0
	fmov	w0, s1
	lsr	w8, w0, #23
	cmp	w8, #254
	b.ls	LBB26_2
	fmul	s0, s0, s0
	fmov	s1, #1.00000000
	fdiv	s0, s1, s0
	ret
LBB26_2:
	lsr	w8, w0, #30
	cbz	w8, LBB26_4
	fmov	w8, s0
	lsr	w2, w8, #31
	mov.16b	v0, v1
	mov	w1, #0
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math3j1f6common
LBB26_4:
	fmov	s1, #0.50000000
	lsr	w8, w0, #24
	cmp	w8, #56
	b.ls	LBB26_6
	fmul	s1, s0, s0
	mov	w8, #22142
	movk	w8, #11609, lsl #16
	fmov	s2, w8
	fmul	s2, s1, s2
	mov	w8, #25670
	movk	w8, #12717, lsl #16
	fmov	s3, w8
	fadd	s2, s2, s3
	fmul	s2, s1, s2
	mov	w8, #65474
	movk	w8, #13725, lsl #16
	fmov	s3, w8
	fadd	s2, s2, s3
	mov	w8, #22482
	movk	w8, #13141, lsl #16
	fmov	s3, w8
	dup.2s	v3, v3[0]
	mov.s	v3[1], v2[0]
	fmul.2s	v2, v3, v1[0]
Lloh206:
	adrp	x8, lCPI26_0@PAGE
Lloh207:
	ldr	d3, [x8, lCPI26_0@PAGEOFF]
	fadd.2s	v2, v2, v3
	fmul.2s	v2, v2, v1[0]
Lloh208:
	adrp	x8, lCPI26_1@PAGE
Lloh209:
	ldr	d3, [x8, lCPI26_1@PAGEOFF]
	fadd.2s	v2, v2, v3
	fmul.2s	v2, v2, v1[0]
Lloh210:
	adrp	x8, lCPI26_2@PAGE
Lloh211:
	ldr	d3, [x8, lCPI26_2@PAGEOFF]
	fadd.2s	v2, v2, v3
	fmul	s1, s1, s2
	mov	s2, v2[1]
	fdiv	s1, s1, s2
	fmov	s2, #0.50000000
	fadd	s1, s1, s2
LBB26_6:
	fmul	s0, s0, s1
	ret
	.loh AdrpLdr	Lloh210, Lloh211
	.loh AdrpAdrp	Lloh208, Lloh210
	.loh AdrpLdr	Lloh208, Lloh209
	.loh AdrpAdrp	Lloh206, Lloh208
	.loh AdrpLdr	Lloh206, Lloh207
	.cfi_endproc

	.section	__TEXT,__literal8,8byte_literals
	.p2align	3, 0x0
lCPI27_0:
	.long	0xb7862e36
	.long	0x3942fab6
lCPI27_1:
	.long	0x3ab86cfd
	.long	0x3c9ce859
lCPI27_2:
	.long	0xbd800000
	.long	0x3f800000
lCPI27_3:
	.long	0x3e91e9ee
	.long	0x3e789e26
lCPI27_4:
	.long	0x3f2aaaaa
	.long	0x3eccce13
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3j1f3y1f
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math3j1f3y1f:
	.cfi_startproc
	fcmp	s0, #0.0
	b.ne	LBB27_2
	mov	w8, #-8388608
	fmov	s0, w8
	ret
LBB27_2:
	fmov	w0, s0
	tbnz	w0, #31, LBB27_5
	lsr	w8, w0, #23
	cmp	w8, #254
	b.ls	LBB27_6
	fmov	s1, #1.00000000
	fdiv	s0, s1, s0
	ret
LBB27_5:
	mov	w8, #2143289344
	fmov	s0, w8
	ret
LBB27_6:
	lsr	w8, w0, #30
	cbz	w8, LBB27_8
	mov	w1, #1
	mov	w2, #0
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math3j1f6common
LBB27_8:
	lsr	w8, w0, #24
	cmp	w8, #51
	b.hs	LBB27_10
	mov	w8, #63875
	movk	w8, #48930, lsl #16
	fmov	s1, w8
	fdiv	s0, s1, s0
	ret
LBB27_10:
	fmul	s2, s0, s0
	mov	w9, #24579
	movk	w9, #46021, lsl #16
	fmov	s1, w9
	fmul	s1, s2, s1
	mov	w9, #22556
	movk	w9, #14277, lsl #16
	fmov	s3, w9
	fadd	s1, s1, s3
	fmul	s1, s2, s1
	mov	w9, #44842
	movk	w9, #47866, lsl #16
	fmov	s3, w9
	fadd	s1, s1, s3
	fmul	s1, s2, s1
	mov	w9, #40508
	movk	w9, #15694, lsl #16
	fmov	s3, w9
	fadd	s1, s1, s3
	fmul	s1, s2, s1
	mov	w9, #49969
	movk	w9, #48712, lsl #16
	fmov	s3, w9
	fadd	s1, s1, s3
	mov	w9, #33231
	movk	w9, #11666, lsl #16
	fmov	s3, w9
	fmul	s3, s2, s3
	mov	w9, #63723
	movk	w9, #12757, lsl #16
	fmov	s4, w9
	fadd	s3, s3, s4
	fmul	s3, s2, s3
	mov	w9, #724
	movk	w9, #13750, lsl #16
	fmov	s4, w9
	fadd	s3, s3, s4
	fmul	s3, s2, s3
	mov	w9, #25675
	movk	w9, #14676, lsl #16
	fmov	s4, w9
	fadd	s3, s3, s4
	fmul	s3, s2, s3
	mov	w9, #10346
	movk	w9, #15523, lsl #16
	fmov	s4, w9
	fadd	s3, s3, s4
	fmul	s3, s2, s3
	fmov	s4, #1.00000000
	fadd	s3, s3, s4
	fdiv	s1, s1, s3
	fmul	s1, s0, s1
	cmp	w8, #56
	b.ls	LBB27_12
	mov	w8, #22142
	movk	w8, #11609, lsl #16
	fmov	s3, w8
	fmul	s3, s2, s3
	mov	w8, #25670
	movk	w8, #12717, lsl #16
	fmov	s4, w8
	fadd	s3, s3, s4
	fmul	s3, s2, s3
	mov	w8, #65474
	movk	w8, #13725, lsl #16
	fmov	s4, w8
	mov	w8, #22482
	movk	w8, #13141, lsl #16
	fmov	s5, w8
	dup.2s	v5, v5[0]
	fadd	s3, s3, s4
	mov.s	v5[1], v3[0]
	fmul.2s	v3, v5, v2[0]
Lloh212:
	adrp	x8, lCPI27_0@PAGE
Lloh213:
	ldr	d4, [x8, lCPI27_0@PAGEOFF]
	fadd.2s	v3, v3, v4
	fmul.2s	v3, v3, v2[0]
Lloh214:
	adrp	x8, lCPI27_1@PAGE
Lloh215:
	ldr	d4, [x8, lCPI27_1@PAGEOFF]
	fadd.2s	v3, v3, v4
	fmul.2s	v3, v3, v2[0]
Lloh216:
	adrp	x8, lCPI27_2@PAGE
Lloh217:
	ldr	d4, [x8, lCPI27_2@PAGEOFF]
	fadd.2s	v3, v3, v4
	fmul	s2, s2, s3
	mov	s3, v3[1]
	fdiv	s2, s2, s3
	fmov	s3, #0.50000000
	fadd	s2, s2, s3
	fmul	s2, s0, s2
	movi.2d	v3, #0000000000000000
	mov	w8, #1065353216
	cmp	w0, w8
	b.ne	LBB27_13
	b	LBB27_14
LBB27_12:
	fmov	s2, #0.50000000
	fmul	s2, s0, s2
LBB27_13:
	add	w8, w0, #1199, lsl #12
	add	w8, w8, #2829
	lsr	w9, w8, #23
	sub	w9, w9, #127
	and	w8, w8, #0x7fffff
	mov	w10, #1267
	movk	w10, #16181, lsl #16
	add	w8, w8, w10
	fmov	s3, w8
	fmov	s4, #-1.00000000
	fadd	s3, s3, s4
	fmov	s4, #2.00000000
	fadd	s4, s3, s4
	fdiv	s4, s3, s4
	fmul	s5, s4, s4
	fmul	s6, s5, s5
	dup.2s	v7, v6[0]
Lloh218:
	adrp	x8, lCPI27_3@PAGE
Lloh219:
	ldr	d16, [x8, lCPI27_3@PAGEOFF]
	fmul.2s	v6, v16, v6[0]
Lloh220:
	adrp	x8, lCPI27_4@PAGE
Lloh221:
	ldr	d16, [x8, lCPI27_4@PAGEOFF]
	fadd.2s	v6, v6, v16
	mov.s	v7[0], v5[0]
	fmul.2s	v5, v7, v6
	faddp.2s	s5, v5
	fmov	s6, #0.50000000
	fmul	s6, s3, s6
	fmul	s6, s3, s6
	scvtf	s7, w9
	fadd	s5, s6, s5
	fmul	s4, s4, s5
	mov	w8, #63441
	movk	w8, #14103, lsl #16
	fmov	s5, w8
	fmul	s5, s7, s5
	fadd	s4, s5, s4
	fsub	s4, s4, s6
	fadd	s3, s3, s4
	mov	w8, #29056
	movk	w8, #16177, lsl #16
	fmov	s4, w8
	fmul	s4, s7, s4
	fadd	s3, s4, s3
LBB27_14:
	fmul	s2, s2, s3
	fmov	s3, #-1.00000000
	fdiv	s0, s3, s0
	fadd	s0, s2, s0
	mov	w8, #63875
	movk	w8, #16162, lsl #16
	fmov	s2, w8
	fmul	s0, s0, s2
	fadd	s0, s1, s0
	ret
	.loh AdrpLdr	Lloh216, Lloh217
	.loh AdrpAdrp	Lloh214, Lloh216
	.loh AdrpLdr	Lloh214, Lloh215
	.loh AdrpAdrp	Lloh212, Lloh214
	.loh AdrpLdr	Lloh212, Lloh213
	.loh AdrpLdr	Lloh220, Lloh221
	.loh AdrpAdrp	Lloh218, Lloh220
	.loh AdrpLdr	Lloh218, Lloh219
	.cfi_endproc

	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math3j1f6common:
	.cfi_startproc
	stp	d11, d10, [sp, #-80]!
	.cfi_def_cfa_offset 80
	stp	d9, d8, [sp, #16]
	stp	x22, x21, [sp, #32]
	stp	x20, x19, [sp, #48]
	stp	x29, x30, [sp, #64]
	add	x29, sp, #64
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset w21, -40
	.cfi_offset w22, -48
	.cfi_offset b8, -56
	.cfi_offset b9, -64
	.cfi_offset b10, -72
	.cfi_offset b11, -80
	mov	x19, x2
	mov	x20, x1
	mov.16b	v8, v0
	mov	x21, x0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4sinf4sinf
	fcvt	d0, s0
	fneg	d1, d0
	cmp	w20, #0
	fcsel	d10, d1, d0, ne
	mov.16b	v0, v8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4cosf4cosf
	fcvt	d11, s0
	fsub	d9, d10, d11
	lsr	w8, w21, #24
	cmp	w8, #127
	b.hs	LBB28_5
	fadd	s0, s8, s8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4cosf4cosf
	fcvt	d1, s0
	fmul	d0, d10, d11
	fcmp	d0, #0.0
	b.le	LBB28_3
	fneg	d0, d10
	fsub	d0, d0, d11
	fdiv	d9, d1, d0
	lsr	w8, w21, #23
	cmp	w8, #176
	b.ls	LBB28_4
	b	LBB28_5
LBB28_3:
	fdiv	d0, d1, d9
	lsr	w8, w21, #23
	cmp	w8, #176
	b.hi	LBB28_5
LBB28_4:
	mov	w8, #29674
	movk	w8, #16529, lsl #16
	fneg	d1, d0
	cmp	w20, #0
	fcsel	d0, d1, d0, ne
	fmov	w10, s8
	and	w9, w10, #0x7fffffff
Lloh222:
	adrp	x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.71@PAGE
Lloh223:
	add	x11, x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.71@PAGEOFF
Lloh224:
	adrp	x12, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.72@PAGE
Lloh225:
	add	x12, x12, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.72@PAGEOFF
	ubfx	w13, w10, #24, #7
Lloh226:
	adrp	x14, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.69@PAGE
Lloh227:
	add	x14, x14, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.69@PAGEOFF
Lloh228:
	adrp	x15, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.70@PAGE
Lloh229:
	add	x15, x15, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.70@PAGEOFF
Lloh230:
	adrp	x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.66@PAGE
Lloh231:
	add	x16, x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.66@PAGEOFF
Lloh232:
	adrp	x17, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.68@PAGE
Lloh233:
	add	x17, x17, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.68@PAGEOFF
	mov	w10, #55574
	movk	w10, #16438, lsl #16
	cmp	w9, w10
Lloh234:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.65@PAGE
Lloh235:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.65@PAGEOFF
	csel	x16, x17, x16, hi
Lloh236:
	adrp	x17, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.67@PAGE
Lloh237:
	add	x17, x17, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.67@PAGEOFF
	csel	x17, x17, x0, hi
	cmp	w9, w8
	csel	x15, x15, x16, hi
	csel	x14, x14, x17, hi
	cmp	w13, #64
	csel	x12, x12, x15, hi
	csel	x11, x11, x14, hi
	fmul	s2, s8, s8
	fmov	s1, #1.00000000
	fdiv	s2, s1, s2
	ldp	s3, s4, [x11]
	ldp	s5, s6, [x11, #8]
	ldp	s7, s16, [x11, #16]
	fmul	s16, s2, s16
	fadd	s7, s7, s16
	fmul	s7, s2, s7
	fadd	s6, s6, s7
	fmul	s6, s2, s6
	fadd	s5, s5, s6
	fmul	s5, s2, s5
	fadd	s4, s4, s5
	fmul	s4, s2, s4
	fadd	s3, s3, s4
	ldp	s4, s5, [x12]
	ldp	s6, s7, [x12, #8]
	ldr	s16, [x12, #16]
	fmul	s16, s2, s16
	fadd	s7, s7, s16
	fmul	s7, s2, s7
	fadd	s6, s6, s7
	fmul	s6, s2, s6
	fadd	s5, s5, s6
	fmul	s5, s2, s5
	fadd	s4, s4, s5
	fmul	s4, s2, s4
	fadd	s4, s4, s1
	fdiv	s3, s3, s4
	fadd	s3, s3, s1
	fcvt	d3, s3
	fmul	d3, d9, d3
Lloh238:
	adrp	x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.79@PAGE
Lloh239:
	add	x11, x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.79@PAGEOFF
Lloh240:
	adrp	x12, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.80@PAGE
Lloh241:
	add	x12, x12, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.80@PAGEOFF
Lloh242:
	adrp	x13, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.77@PAGE
Lloh243:
	add	x13, x13, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.77@PAGEOFF
Lloh244:
	adrp	x14, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.78@PAGE
Lloh245:
	add	x14, x14, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.78@PAGEOFF
Lloh246:
	adrp	x15, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.74@PAGE
Lloh247:
	add	x15, x15, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.74@PAGEOFF
Lloh248:
	adrp	x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.76@PAGE
Lloh249:
	add	x16, x16, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.76@PAGEOFF
	lsr	w17, w9, #24
	cmp	w9, w10
Lloh250:
	adrp	x10, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.73@PAGE
Lloh251:
	add	x10, x10, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.73@PAGEOFF
Lloh252:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.75@PAGE
Lloh253:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.75@PAGEOFF
	csel	x15, x16, x15, hi
	csel	x10, x0, x10, hi
	cmp	w9, w8
	csel	x8, x14, x15, hi
	csel	x9, x13, x10, hi
	cmp	w17, #64
	csel	x8, x12, x8, hi
	csel	x9, x11, x9, hi
	ldp	s4, s5, [x8, #16]
	fmul	s5, s2, s5
	fadd	s4, s4, s5
	ldp	d5, d6, [x9]
	ldr	d7, [x9, #16]
	dup.2s	v16, v7[1]
	ldp	d17, d18, [x8]
	mov.s	v16[1], v4[0]
	fmul.2s	v4, v16, v2[0]
	mov.s	v7[1], v18[1]
	fadd.2s	v4, v7, v4
	fmul.2s	v4, v4, v2[0]
	ext.8b	v7, v6, v18, #4
	fadd.2s	v4, v7, v4
	mov.s	v6[1], v17[1]
	fmul.2s	v4, v4, v2[0]
	fadd.2s	v4, v6, v4
	fmul.2s	v4, v4, v2[0]
	ext.8b	v6, v5, v17, #4
	fadd.2s	v4, v6, v4
	fmul.2s	v2, v4, v2[0]
	mov.s	v5[1], v1[0]
	fadd.2s	v1, v5, v2
	dup.2s	v2, v1[1]
	fdiv.2s	v1, v1, v2
	fmov	s2, #0.37500000
	fadd	s1, s1, s2
	fdiv	s1, s1, s8
	fcvt	d1, s1
	fmul	d0, d0, d1
	fsub	d9, d3, d0
LBB28_5:
	fneg	d0, d9
	cmp	w19, #0
	fcsel	d0, d0, d9, ne
	mov	x8, #1610612736
	movk	x8, #3543, lsl #32
	movk	x8, #16354, lsl #48
	fmov	d1, x8
	fmul	d0, d0, d1
	; InlineAsm Start
	fsqrt	s8, s8
	; InlineAsm End
	fcvt	d1, s8
	fdiv	d0, d0, d1
	fcvt	s0, d0
	.cfi_def_cfa wsp, 80
	ldp	x29, x30, [sp, #64]
	ldp	x20, x19, [sp, #48]
	ldp	x22, x21, [sp, #32]
	ldp	d9, d8, [sp, #16]
	ldp	d11, d10, [sp], #80
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	.cfi_restore b8
	.cfi_restore b9
	.cfi_restore b10
	.cfi_restore b11
	ret
	.loh AdrpAdd	Lloh252, Lloh253
	.loh AdrpAdd	Lloh250, Lloh251
	.loh AdrpAdd	Lloh248, Lloh249
	.loh AdrpAdd	Lloh246, Lloh247
	.loh AdrpAdd	Lloh244, Lloh245
	.loh AdrpAdd	Lloh242, Lloh243
	.loh AdrpAdd	Lloh240, Lloh241
	.loh AdrpAdd	Lloh238, Lloh239
	.loh AdrpAdd	Lloh236, Lloh237
	.loh AdrpAdd	Lloh234, Lloh235
	.loh AdrpAdd	Lloh232, Lloh233
	.loh AdrpAdd	Lloh230, Lloh231
	.loh AdrpAdd	Lloh228, Lloh229
	.loh AdrpAdd	Lloh226, Lloh227
	.loh AdrpAdd	Lloh224, Lloh225
	.loh AdrpAdd	Lloh222, Lloh223
	.cfi_endproc

	.section	__TEXT,__literal8,8byte_literals
	.p2align	3, 0x0
lCPI29_0:
	.long	0xb947352e
	.long	0x3c7fe744
lCPI29_1:
	.long	0x3c800000
	.long	0x3f800000
lCPI29_2:
	.long	0xb7862e36
	.long	0x3942fab6
lCPI29_3:
	.long	0x3ab86cfd
	.long	0x3c9ce859
lCPI29_4:
	.long	0xbd800000
	.long	0x3f800000
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3jnf3jnf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math3jnf3jnf:
	.cfi_startproc
	stp	d13, d12, [sp, #-96]!
	.cfi_def_cfa_offset 96
	stp	d11, d10, [sp, #16]
	stp	d9, d8, [sp, #32]
	stp	x22, x21, [sp, #48]
	stp	x20, x19, [sp, #64]
	stp	x29, x30, [sp, #80]
	add	x29, sp, #80
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset w21, -40
	.cfi_offset w22, -48
	.cfi_offset b8, -56
	.cfi_offset b9, -64
	.cfi_offset b10, -72
	.cfi_offset b11, -80
	.cfi_offset b12, -88
	.cfi_offset b13, -96
	.cfi_remember_state
	mov.16b	v1, v0
	mov	x8, x0
	fabs	s0, s0
	fmov	w0, s0
	mov	w9, #2139095040
	cmp	w0, w9
	b.hi	LBB29_53
	cbz	w8, LBB29_8
	fmov	w9, s1
	mvn	w10, w9
	lsr	w10, w10, #31
	eor	w11, w9, #0x80000000
	sub	w12, w8, #1
	lsr	w13, w9, #31
	tst	w8, #0x80000000
	csel	w10, w10, w13, ne
	csinv	w21, w12, w8, eq
	csel	w9, w11, w9, ne
	cbz	w21, LBB29_11
	and	w20, w8, w10
	movi.2d	v0, #0000000000000000
	cbz	w0, LBB29_52
	mov	w8, #2139095040
	cmp	w0, w8
	b.eq	LBB29_52
	and	w19, w9, #0x7fffffff
	ucvtf	s0, w21
	fmov	s8, w19
	fcmp	s0, s8
	b.pl	LBB29_17
	lsr	w8, w19, #23
	cmp	w8, #254
	b.ls	LBB29_26
	fmul	s0, s8, s8
	fmov	s1, #1.00000000
	fdiv	s0, s1, s0
	mov.16b	v9, v0
	b	LBB29_50
LBB29_8:
	mov	w8, #2139095040
	cmp	w0, w8
	b.eq	LBB29_12
	lsr	w8, w0, #30
	cbz	w8, LBB29_15
	mov	w1, #0
	.cfi_def_cfa wsp, 96
	ldp	x29, x30, [sp, #80]
	ldp	x20, x19, [sp, #64]
	ldp	x22, x21, [sp, #48]
	ldp	d9, d8, [sp, #32]
	ldp	d11, d10, [sp, #16]
	ldp	d13, d12, [sp], #96
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	.cfi_restore b8
	.cfi_restore b9
	.cfi_restore b10
	.cfi_restore b11
	.cfi_restore b12
	.cfi_restore b13
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math3j0f6common
LBB29_11:
	.cfi_restore_state
	.cfi_remember_state
	fmov	s1, w9
	fabs	s0, s1
	fmov	w0, s0
	lsr	w8, w0, #23
	cmp	w8, #254
	b.ls	LBB29_13
LBB29_12:
	fmul	s0, s1, s1
	fmov	s1, #1.00000000
	fdiv	s1, s1, s0
	b	LBB29_53
LBB29_13:
	lsr	w8, w0, #30
	cbz	w8, LBB29_22
	lsr	w2, w9, #31
	mov	w1, #0
	.cfi_def_cfa wsp, 96
	ldp	x29, x30, [sp, #80]
	ldp	x20, x19, [sp, #64]
	ldp	x22, x21, [sp, #48]
	ldp	d9, d8, [sp, #32]
	ldp	d11, d10, [sp, #16]
	ldp	d13, d12, [sp], #96
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	.cfi_restore b8
	.cfi_restore b9
	.cfi_restore b10
	.cfi_restore b11
	.cfi_restore b12
	.cfi_restore b13
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math3j1f6common
LBB29_15:
	.cfi_restore_state
	.cfi_remember_state
	lsr	w8, w0, #25
	cmp	w8, #28
	b.ls	LBB29_25
	fmul	s1, s1, s1
	mov	w8, #17896
	movk	w8, #12448, lsl #16
	fmov	s2, w8
	fmul	s2, s1, s2
	mov	w8, #55974
	movk	w8, #13577, lsl #16
	fmov	s3, w8
	fadd	s2, s2, s3
	mov	w8, #44860
	movk	w8, #12702, lsl #16
	fmov	s3, w8
	dup.2s	v3, v3[0]
	mov.s	v3[1], v2[0]
	fmul.2s	v2, v3, v1[0]
	mov	w8, #36488
	movk	w8, #13813, lsl #16
	fmov	s3, w8
	fsub.2s	v3, v3, v2
	mov	w8, #13975
	movk	w8, #14581, lsl #16
	dup.2s	v4, w8
	fadd.2s	v2, v2, v4
	mov.s	v3[1], v2[1]
	fmul.2s	v2, v3, v1[0]
Lloh254:
	adrp	x8, lCPI29_0@PAGE
Lloh255:
	ldr	d3, [x8, lCPI29_0@PAGEOFF]
	fadd.2s	v2, v2, v3
	fmul.2s	v2, v2, v1[0]
Lloh256:
	adrp	x8, lCPI29_1@PAGE
Lloh257:
	ldr	d3, [x8, lCPI29_1@PAGEOFF]
	fadd.2s	v2, v2, v3
	fmul	s3, s1, s2
	fmov	s4, #0.50000000
	fmul	s0, s0, s4
	fmov	s4, #1.00000000
	fadd	s5, s0, s4
	fsub	s0, s4, s0
	fmul	s0, s5, s0
	mov	s2, v2[1]
	fdiv	s2, s3, s2
	fmul	s1, s1, s2
	fadd	s1, s0, s1
	b	LBB29_53
LBB29_17:
	lsr	w8, w0, #23
	cmp	w8, #107
	b.hs	LBB29_28
	fmov	s0, #0.50000000
	fmul	s2, s8, s0
	mov	w8, #8
	cmp	w21, #8
	csel	w8, w21, w8, lo
	add	w8, w8, #1
	fmul	s0, s2, s2
	fmov	s1, #2.00000000
	cmp	w8, #2
	b.eq	LBB29_21
	fmul	s0, s2, s0
	fmov	s1, #6.00000000
	cmp	w8, #3
	b.eq	LBB29_21
	fmul	s0, s2, s0
	fmov	s1, #24.00000000
	cmp	w8, #4
	b.ne	LBB29_55
LBB29_21:
	fdiv	s0, s0, s1
	b	LBB29_52
LBB29_22:
	fmov	s0, #0.50000000
	lsr	w8, w0, #24
	cmp	w8, #56
	b.ls	LBB29_24
	fmul	s0, s1, s1
	mov	w8, #22142
	movk	w8, #11609, lsl #16
	fmov	s2, w8
	fmul	s2, s0, s2
	mov	w8, #25670
	movk	w8, #12717, lsl #16
	fmov	s3, w8
	fadd	s2, s2, s3
	fmul	s2, s0, s2
	mov	w8, #65474
	movk	w8, #13725, lsl #16
	fmov	s3, w8
	fadd	s2, s2, s3
	mov	w8, #22482
	movk	w8, #13141, lsl #16
	fmov	s3, w8
	dup.2s	v3, v3[0]
	mov.s	v3[1], v2[0]
	fmul.2s	v2, v3, v0[0]
Lloh258:
	adrp	x8, lCPI29_2@PAGE
Lloh259:
	ldr	d3, [x8, lCPI29_2@PAGEOFF]
	fadd.2s	v2, v2, v3
	fmul.2s	v2, v2, v0[0]
Lloh260:
	adrp	x8, lCPI29_3@PAGE
Lloh261:
	ldr	d3, [x8, lCPI29_3@PAGEOFF]
	fadd.2s	v2, v2, v3
	fmul.2s	v2, v2, v0[0]
Lloh262:
	adrp	x8, lCPI29_4@PAGE
Lloh263:
	ldr	d3, [x8, lCPI29_4@PAGEOFF]
	fadd.2s	v2, v2, v3
	fmul	s0, s0, s2
	mov	s2, v2[1]
	fdiv	s0, s0, s2
	fmov	s2, #0.50000000
	fadd	s0, s0, s2
LBB29_24:
	fmul	s1, s0, s1
	b	LBB29_53
LBB29_25:
	fmov	s1, #0.25000000
	fmul	s1, s0, s1
	fmul	s1, s0, s1
	lsr	w8, w0, #23
	cmp	w8, #66
	fcsel	s0, s1, s0, hi
	fmov	s1, #1.00000000
	fsub	s1, s1, s0
	b	LBB29_53
LBB29_26:
	lsr	w9, w19, #30
	cbz	w9, LBB29_39
	mov	x0, x19
	mov.16b	v0, v8
	mov	w1, #0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3j0f6common
	mov.16b	v9, v0
	mov	x0, x19
	mov.16b	v0, v8
	mov	w1, #0
	mov	w2, #0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3j1f6common
	b	LBB29_50
LBB29_28:
	fmov	s1, #1.00000000
	fadd	s9, s0, s1
	fadd	s0, s9, s9
	fdiv	s0, s0, s8
	fmov	s2, #2.00000000
	fdiv	s2, s2, s8
	fadd	s3, s0, s2
	fmul	s4, s0, s3
	fmov	s5, #-1.00000000
	fadd	s4, s4, s5
	mov	w8, #16384
	movk	w8, #17948, lsl #16
	fmov	s5, w8
	mov	w8, #1
	fcmp	s4, s5
	b.pl	LBB29_32
	mov	w9, #16384
	movk	w9, #17948, lsl #16
	fmov	s5, w9
	mov.16b	v6, v0
LBB29_30:
	mov.16b	v7, v6
	mov.16b	v6, v4
	add	w8, w8, #1
	fadd	s3, s2, s3
	fmul	s4, s3, s4
	fsub	s4, s4, s7
	fcmp	s4, s5
	b.mi	LBB29_30
	tbnz	w8, #31, LBB29_54
LBB29_32:
	movi.2d	v10, #0000000000000000
LBB29_33:
	ucvtf	s2, w8
	fadd	s2, s9, s2
	fadd	s2, s2, s2
	fdiv	s2, s2, s8
	fsub	s2, s2, s10
	fdiv	s10, s1, s2
	sub	w8, w8, #1
	cmn	w8, #1
	b.ne	LBB29_33
LBB29_34:
	fabs	s0, s0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4logf4logf
	fmul	s0, s9, s0
	mov	w8, #29056
	movk	w8, #17073, lsl #16
	fmov	s1, w8
	fcmp	s0, s1
	b.pl	LBB29_41
	fmov	s11, #1.00000000
	mov.16b	v12, v10
LBB29_36:
	mov.16b	v0, v12
	ucvtf	s1, w21
	mov.16b	v12, v11
	fadd	s1, s1, s1
	fmul	s1, s11, s1
	fdiv	s1, s1, s8
	fsub	s11, s1, s0
	subs	w21, w21, #1
	b.hi	LBB29_36
LBB29_37:
	mov.16b	v0, v8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3j0f3j0f
	mov.16b	v9, v0
	mov.16b	v0, v8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3j1f3j1f
	fabs	s1, s9
	fabs	s2, s0
	fcmp	s1, s2
	b.ge	LBB29_45
	fmul	s0, s10, s0
	fdiv	s0, s0, s12
	b	LBB29_52
LBB29_39:
	lsr	w9, w19, #25
	cmp	w9, #29
	b.lo	LBB29_46
	fmul	s0, s8, s8
	mov	w8, #17896
	movk	w8, #12448, lsl #16
	fmov	s1, w8
	fmul	s1, s0, s1
	mov	w8, #55974
	movk	w8, #13577, lsl #16
	fmov	s2, w8
	fadd	s1, s1, s2
	mov	w8, #44860
	movk	w8, #12702, lsl #16
	fmov	s2, w8
	dup.2s	v2, v2[0]
	mov.s	v2[1], v1[0]
	fmul.2s	v1, v2, v0[0]
	mov	w8, #36488
	movk	w8, #13813, lsl #16
	fmov	s2, w8
	fsub.2s	v2, v2, v1
	mov	w8, #13975
	movk	w8, #14581, lsl #16
	dup.2s	v3, w8
	fadd.2s	v1, v1, v3
	mov.s	v2[1], v1[1]
	fmul.2s	v1, v2, v0[0]
Lloh264:
	adrp	x8, lCPI29_0@PAGE
Lloh265:
	ldr	d2, [x8, lCPI29_0@PAGEOFF]
	fadd.2s	v1, v1, v2
	fmul.2s	v1, v1, v0[0]
Lloh266:
	adrp	x8, lCPI29_1@PAGE
Lloh267:
	ldr	d2, [x8, lCPI29_1@PAGEOFF]
	fadd.2s	v1, v1, v2
	fmul	s2, s0, s1
	fmov	s3, #0.50000000
	fmul	s3, s8, s3
	fmov	s4, #1.00000000
	fadd	s5, s3, s4
	fsub	s3, s4, s3
	fmul	s3, s5, s3
	mov	s1, v1[1]
	fdiv	s1, s2, s1
	fmul	s1, s0, s1
	fadd	s9, s3, s1
	b	LBB29_48
LBB29_41:
	fmov	s1, #1.00000000
	mov	w8, #1568669696
	fmov	s0, w8
	mov.16b	v12, v10
	b	LBB29_43
LBB29_42:
	mov.16b	v12, v1
	mov.16b	v1, v11
	subs	w21, w21, #1
	b.le	LBB29_37
LBB29_43:
	ucvtf	s2, w21
	fadd	s2, s2, s2
	fmul	s2, s2, s1
	fdiv	s2, s2, s8
	fsub	s11, s2, s12
	fcmp	s11, s0
	b.le	LBB29_42
	fdiv	s1, s1, s11
	fdiv	s10, s10, s11
	fmov	s11, #1.00000000
	b	LBB29_42
LBB29_45:
	fmul	s0, s10, s9
	fdiv	s0, s0, s11
	b	LBB29_52
LBB29_46:
	fmov	s0, #0.25000000
	fmul	s0, s8, s0
	cmp	w8, #66
	fmov	s1, #1.00000000
	fcsel	s0, s0, s1, hi
	fmul	s0, s0, s8
	fsub	s9, s1, s0
	fmov	s0, #0.50000000
	lsr	w8, w19, #24
	cmp	w8, #57
	b.lo	LBB29_49
	fmul	s0, s8, s8
LBB29_48:
	mov	w8, #22142
	movk	w8, #11609, lsl #16
	fmov	s1, w8
	fmul	s1, s0, s1
	mov	w8, #25670
	movk	w8, #12717, lsl #16
	fmov	s2, w8
	fadd	s1, s1, s2
	fmul	s1, s0, s1
	mov	w8, #65474
	movk	w8, #13725, lsl #16
	fmov	s2, w8
	fadd	s1, s1, s2
	mov	w8, #22482
	movk	w8, #13141, lsl #16
	fmov	s2, w8
	dup.2s	v2, v2[0]
	mov.s	v2[1], v1[0]
	fmul.2s	v1, v2, v0[0]
Lloh268:
	adrp	x8, lCPI29_2@PAGE
Lloh269:
	ldr	d2, [x8, lCPI29_2@PAGEOFF]
	fadd.2s	v1, v1, v2
	fmul.2s	v1, v1, v0[0]
Lloh270:
	adrp	x8, lCPI29_3@PAGE
Lloh271:
	ldr	d2, [x8, lCPI29_3@PAGEOFF]
	fadd.2s	v1, v1, v2
	fmul.2s	v1, v1, v0[0]
Lloh272:
	adrp	x8, lCPI29_4@PAGE
Lloh273:
	ldr	d2, [x8, lCPI29_4@PAGEOFF]
	fadd.2s	v1, v1, v2
	fmul	s0, s0, s1
	mov	s1, v1[1]
	fdiv	s0, s0, s1
	fmov	s1, #0.50000000
	fadd	s0, s0, s1
LBB29_49:
	fmul	s0, s0, s8
LBB29_50:
	mov	w8, #1
LBB29_51:
	mov.16b	v1, v9
	mov.16b	v9, v0
	ucvtf	s0, w8
	fadd	s0, s0, s0
	fdiv	s0, s0, s8
	fmul	s0, s9, s0
	fsub	s0, s0, s1
	add	w8, w8, #1
	subs	w21, w21, #1
	b.ne	LBB29_51
LBB29_52:
	fneg	s1, s0
	cmp	w20, #0
	fcsel	s1, s1, s0, ne
LBB29_53:
	mov.16b	v0, v1
	.cfi_def_cfa wsp, 96
	ldp	x29, x30, [sp, #80]
	ldp	x20, x19, [sp, #64]
	ldp	x22, x21, [sp, #48]
	ldp	d9, d8, [sp, #32]
	ldp	d11, d10, [sp, #16]
	ldp	d13, d12, [sp], #96
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	.cfi_restore b8
	.cfi_restore b9
	.cfi_restore b10
	.cfi_restore b11
	.cfi_restore b12
	.cfi_restore b13
	ret
LBB29_54:
	.cfi_restore_state
	movi.2d	v10, #0000000000000000
	b	LBB29_34
LBB29_55:
	fmul	s0, s2, s0
	cmp	w8, #5
	b.ne	LBB29_57
	mov	w8, #1123024896
	fmov	s1, w8
	fdiv	s0, s0, s1
	b	LBB29_52
LBB29_57:
	fmul	s0, s2, s0
	cmp	w8, #6
	b.ne	LBB29_59
	mov	w8, #1144258560
	fmov	s1, w8
	fdiv	s0, s0, s1
	b	LBB29_52
LBB29_59:
	fmul	s0, s2, s0
	cmp	w8, #7
	b.ne	LBB29_61
	mov	w8, #32768
	movk	w8, #17821, lsl #16
	fmov	s1, w8
	fdiv	s0, s0, s1
	b	LBB29_52
LBB29_61:
	fmul	s0, s2, s0
	fmul	s2, s2, s0
	cmp	w8, #8
	mov	w8, #12288
	movk	w8, #18609, lsl #16
	fmov	s1, w8
	mov	w8, #32768
	movk	w8, #18205, lsl #16
	fmov	s3, w8
	fcsel	s1, s3, s1, eq
	fcsel	s0, s0, s2, eq
	b	LBB29_21
	.loh AdrpLdr	Lloh256, Lloh257
	.loh AdrpAdrp	Lloh254, Lloh256
	.loh AdrpLdr	Lloh254, Lloh255
	.loh AdrpLdr	Lloh262, Lloh263
	.loh AdrpAdrp	Lloh260, Lloh262
	.loh AdrpLdr	Lloh260, Lloh261
	.loh AdrpAdrp	Lloh258, Lloh260
	.loh AdrpLdr	Lloh258, Lloh259
	.loh AdrpLdr	Lloh266, Lloh267
	.loh AdrpAdrp	Lloh264, Lloh266
	.loh AdrpLdr	Lloh264, Lloh265
	.loh AdrpLdr	Lloh272, Lloh273
	.loh AdrpAdrp	Lloh270, Lloh272
	.loh AdrpLdr	Lloh270, Lloh271
	.loh AdrpAdrp	Lloh268, Lloh270
	.loh AdrpLdr	Lloh268, Lloh269
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3jnf3ynf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math3jnf3ynf:
	.cfi_startproc
	stp	d11, d10, [sp, #-64]!
	.cfi_def_cfa_offset 64
	stp	d9, d8, [sp, #16]
	stp	x20, x19, [sp, #32]
	stp	x29, x30, [sp, #48]
	add	x29, sp, #48
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset b8, -40
	.cfi_offset b9, -48
	.cfi_offset b10, -56
	.cfi_offset b11, -64
	.cfi_remember_state
	fmov	w8, s0
	and	w8, w8, #0x7fffffff
	mov	w9, #2139095040
	cmp	w8, w9
	b.ls	LBB30_2
	mov.16b	v1, v0
	b	LBB30_20
LBB30_2:
	fmov	w9, s0
	tbz	w9, #31, LBB30_5
	cbz	w8, LBB30_5
	mov	w8, #2143289344
	fmov	s1, w8
	b	LBB30_20
LBB30_5:
	movi.2d	v1, #0000000000000000
	mov	w9, #2139095040
	cmp	w8, w9
	b.eq	LBB30_20
	cbz	w0, LBB30_10
	tbnz	w0, #31, LBB30_11
	subs	w19, w0, #1
	b.eq	LBB30_21
	mov	w20, #0
	b	LBB30_13
LBB30_10:
	.cfi_def_cfa wsp, 64
	ldp	x29, x30, [sp, #48]
	ldp	x20, x19, [sp, #32]
	ldp	d9, d8, [sp, #16]
	ldp	d11, d10, [sp], #64
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore b8
	.cfi_restore b9
	.cfi_restore b10
	.cfi_restore b11
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math3j0f3y0f
LBB30_11:
	.cfi_restore_state
	.cfi_remember_state
	cmn	w0, #1
	b.eq	LBB30_22
	mov	x20, x0
	mvn	w19, w0
LBB30_13:
	mov.16b	v9, v0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3j0f3y0f
	mov.16b	v8, v0
	mov.16b	v10, v9
	mov.16b	v0, v9
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3j1f3y1f
	cbz	w19, LBB30_18
	mov	w8, #-8388608
	fmov	s1, w8
	fcmp	s0, s1
	b.eq	LBB30_18
	mov	w8, #1
	mov	w9, #-8388608
	fmov	s1, w9
	mov.16b	v3, v10
LBB30_16:
	ucvtf	s2, w8
	fadd	s2, s2, s2
	fdiv	s2, s2, s3
	fmul	s2, s0, s2
	fsub	s2, s2, s8
	cmp	w8, w19
	b.ge	LBB30_19
	add	w8, w8, #1
	fcmp	s2, s1
	mov.16b	v8, v0
	mov.16b	v0, v2
	b.ne	LBB30_16
	b	LBB30_19
LBB30_18:
	mov.16b	v2, v0
LBB30_19:
	fneg	s0, s2
	tst	w20, #0x1
	fcsel	s1, s0, s2, ne
LBB30_20:
	mov.16b	v0, v1
	.cfi_def_cfa wsp, 64
	ldp	x29, x30, [sp, #48]
	ldp	x20, x19, [sp, #32]
	ldp	d9, d8, [sp, #16]
	ldp	d11, d10, [sp], #64
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore b8
	.cfi_restore b9
	.cfi_restore b10
	.cfi_restore b11
	ret
LBB30_21:
	.cfi_restore_state
	.cfi_remember_state
	.cfi_def_cfa wsp, 64
	ldp	x29, x30, [sp, #48]
	ldp	x20, x19, [sp, #32]
	ldp	d9, d8, [sp, #16]
	ldp	d11, d10, [sp], #64
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore b8
	.cfi_restore b9
	.cfi_restore b10
	.cfi_restore b11
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math3j1f3y1f
LBB30_22:
	.cfi_restore_state
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3j1f3y1f
	fneg	s1, s0
	b	LBB30_20
	.cfi_endproc

	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0
lCPI31_0:
	.quad	0x3fcc71c51d8e78af
	.quad	0x3fd2492494229359
lCPI31_1:
	.quad	0x3fd999999997fa04
	.quad	0x3fe5555555555593
	.section	__TEXT,__text,regular,pure_instructions
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math3log3log:
	.cfi_startproc
	fmov	x8, d0
	mov	x9, #4503599627370496
	cmp	x8, x9
	b.lt	LBB31_6
	lsr	x9, x8, #52
	cmp	x9, #2046
	b.hi	LBB31_12
	lsr	x9, x8, #32
	mov	w10, #1072693248
	cmp	x9, x10
	b.ne	LBB31_5
	cbz	w8, LBB31_13
	mov	w9, #1072693248
LBB31_5:
	mov	w10, #-1023
	b	LBB31_10
LBB31_6:
	fcmp	d0, #0.0
	b.ne	LBB31_8
	fmul	d0, d0, d0
	fmov	d1, #-1.00000000
	fdiv	d0, d1, d0
	ret
LBB31_8:
	tbnz	x8, #63, LBB31_11
	mov	x8, #4850376798678024192
	fmov	d1, x8
	fmul	d0, d0, d1
	fmov	x8, d0
	lsr	x9, x8, #32
	mov	w10, #-1077
LBB31_10:
	add	w9, w9, #149, lsl #12
	add	w9, w9, #3938
	add	w10, w10, w9, lsr #20
	and	w9, w9, #0xfffff
	mov	w11, #41118
	movk	w11, #16358, lsl #16
	add	w9, w9, w11
	mov	w8, w8
	orr	x8, x8, x9, lsl #32
	fmov	d0, x8
	fmov	d1, #-1.00000000
	fadd	d0, d0, d1
	fmov	d1, #0.50000000
	fmul	d1, d0, d1
	fmul	d1, d0, d1
	fmov	d2, #2.00000000
	fadd	d2, d0, d2
	fdiv	d2, d0, d2
	fmul	d3, d2, d2
	fmul	d4, d3, d3
	mov	x8, #21060
	movk	x8, #57150, lsl #16
	movk	x8, #61714, lsl #32
	movk	x8, #16322, lsl #48
	fmov	d5, x8
	fmul	d5, d4, d5
	mov	x8, #990
	movk	x8, #38603, lsl #16
	movk	x8, #18020, lsl #32
	movk	x8, #16327, lsl #48
	fmov	d6, x8
	fadd	d5, d5, d6
	dup.2d	v6, v4[0]
	mov	x8, #50847
	movk	x8, #53368, lsl #16
	movk	x8, #39433, lsl #32
	movk	x8, #16323, lsl #48
	fmov	d7, x8
	dup.2d	v7, v7[0]
	mov.d	v7[1], v5[0]
	fmul.2d	v5, v7, v4[0]
Lloh274:
	adrp	x8, lCPI31_0@PAGE
Lloh275:
	ldr	q7, [x8, lCPI31_0@PAGEOFF]
	fadd.2d	v5, v5, v7
	fmul.2d	v4, v5, v4[0]
Lloh276:
	adrp	x8, lCPI31_1@PAGE
Lloh277:
	ldr	q5, [x8, lCPI31_1@PAGEOFF]
	fadd.2d	v4, v4, v5
	mov.d	v6[1], v3[0]
	fmul.2d	v3, v6, v4
	faddp.2d	d3, v3
	scvtf	d4, w10
	fadd	d3, d1, d3
	fmul	d2, d2, d3
	mov	x8, #15478
	movk	x8, #13689, lsl #16
	movk	x8, #14831, lsl #32
	movk	x8, #15850, lsl #48
	fmov	d3, x8
	fmul	d3, d4, d3
	fadd	d2, d3, d2
	fsub	d1, d2, d1
	fadd	d0, d0, d1
	mov	x8, #4276092928
	movk	x8, #11842, lsl #32
	movk	x8, #16358, lsl #48
	fmov	d1, x8
	fmul	d1, d4, d1
	fadd	d0, d1, d0
	ret
LBB31_11:
	fsub	d0, d0, d0
	movi.2d	v1, #0000000000000000
	fdiv	d0, d0, d1
LBB31_12:
	ret
LBB31_13:
	movi.2d	v0, #0000000000000000
	ret
	.loh AdrpLdr	Lloh276, Lloh277
	.loh AdrpAdrp	Lloh274, Lloh276
	.loh AdrpLdr	Lloh274, Lloh275
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3pow3pow
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math3pow3pow:
	.cfi_startproc
	stp	d9, d8, [sp, #-32]!
	.cfi_def_cfa_offset 32
	stp	x29, x30, [sp, #16]
	add	x29, sp, #16
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset b8, -24
	.cfi_offset b9, -32
	.cfi_remember_state
	fmov	x8, d1
	ubfx	x10, x8, #32, #31
	fmov	d2, #1.00000000
	orr	w9, w10, w8
	cbz	w9, LBB32_10
	fmov	x11, d0
	lsr	x13, x11, #32
	mov	w9, #1072693248
	cmp	x13, x9
	b.ne	LBB32_3
	cbz	w11, LBB32_10
LBB32_3:
	ubfx	x9, x11, #32, #31
	mov	w12, #2146435072
	cmp	w9, w12
	b.hi	LBB32_9
	b.ne	LBB32_6
	cbnz	w11, LBB32_9
LBB32_6:
	cmp	w10, w12
	b.hi	LBB32_9
	cbz	w8, LBB32_12
	cmp	w10, w12
	b.ne	LBB32_12
LBB32_9:
	fadd	d2, d0, d1
LBB32_10:
	mov.16b	v0, v2
	.cfi_def_cfa wsp, 32
LBB32_11:
	ldp	x29, x30, [sp, #16]
	ldp	d9, d8, [sp], #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore b8
	.cfi_restore b9
	ret
LBB32_12:
	.cfi_restore_state
	.cfi_remember_state
	lsr	x12, x8, #32
	tbnz	x11, #63, LBB32_14
LBB32_13:
	mov	w14, #0
	cbnz	w8, LBB32_35
	b	LBB32_16
LBB32_14:
	lsr	w15, w10, #22
	cmp	w15, #268
	b.ls	LBB32_21
	mov	w14, #2
	cbnz	w8, LBB32_35
LBB32_16:
	mov	w15, #1072693248
	cmp	w10, w15
	b.eq	LBB32_25
	mov	w15, #2146435072
	cmp	w10, w15
	b.ne	LBB32_27
	mov	w10, #-1072693248
	add	w10, w9, w10
	orr	w10, w10, w11
	cbz	w10, LBB32_10
	lsr	w9, w9, #20
	cmp	w9, #1022
	b.ls	LBB32_77
	cmn	x8, #1
	movi.2d	v0, #0000000000000000
	fcsel	d2, d1, d0, gt
	b	LBB32_10
LBB32_21:
	lsr	w14, w10, #20
	cmp	w14, #1023
	b.lo	LBB32_13
	cmp	w15, #260
	b.ls	LBB32_31
	mov	w15, #1075
	sub	w15, w15, w14
	lsr	w14, w8, w15
	lsl	w15, w14, w15
	cmp	w15, w8
	b.ne	LBB32_13
	and	w14, w14, #0x1
	mov	w15, #2
	sub	w14, w15, w14
	cbnz	w8, LBB32_35
	b	LBB32_16
LBB32_25:
	tbnz	x8, #63, LBB32_30
	mov.16b	v2, v0
	b	LBB32_10
LBB32_27:
	mov	w15, #1071644672
	cmp	w12, w15
	b.eq	LBB32_33
	mov	w15, #1073741824
	cmp	w12, w15
	b.ne	LBB32_35
	fmul	d2, d0, d0
	b	LBB32_10
LBB32_30:
	fmov	d1, #1.00000000
	fdiv	d2, d1, d0
	b	LBB32_10
LBB32_31:
	cbz	w8, LBB32_80
	mov	w14, #0
	b	LBB32_35
LBB32_33:
	tbnz	x11, #63, LBB32_35
	; InlineAsm Start
	fsqrt	d0, d0
	; InlineAsm End
	.cfi_def_cfa wsp, 32
	b	LBB32_11
LBB32_35:
	.cfi_restore_state
	fabs	d2, d0
	cbz	w11, LBB32_41
LBB32_36:
	fmov	d8, #1.00000000
	tbnz	x11, #63, LBB32_49
LBB32_37:
	mov	w11, #1105199104
	cmp	w10, w11
	b.ls	LBB32_45
	mov	w11, #1139802112
	cmp	w10, w11
	b.ls	LBB32_63
	lsr	w9, w9, #20
	cmp	w9, #1022
	b.hi	LBB32_67
	cmp	x8, #0
	mov	x8, #9218868437227405312
	fmov	d0, x8
	movi.2d	v1, #0000000000000000
	fcsel	d2, d0, d1, mi
	b	LBB32_10
LBB32_41:
	tbz	w13, #31, LBB32_52
	mov	w15, #-2147483648
	cmp	w13, w15
	b.eq	LBB32_55
	mov	w15, #-1074790400
	cmp	w13, w15
	b.eq	LBB32_55
	cmn	w13, #256, lsl #12
	b.eq	LBB32_55
	b	LBB32_36
LBB32_45:
	fmov	x10, d2
	mov	x11, #4845873199050653696
	fmov	d0, x11
	fmul	d0, d2, d0
	fmov	x11, d0
	lsr	x12, x11, #32
	cmp	w9, #256, lsl #12
	csel	w9, w12, w9, lo
	csel	x11, x11, x10, lo
	mov	w10, #-1023
	mov	w12, #-1076
	csel	w10, w12, w10, lo
	add	w10, w10, w9, asr #20
	and	w12, w9, #0xfffff
	mov	w13, #1072693248
	bfxil	w13, w9, #0, #20
	mov	w9, #39055
	movk	w9, #3, lsl #16
	mov	w14, #46714
	movk	w14, #11, lsl #16
	orr	w15, w12, #0x3fe00000
	cmp	w12, w14
	csel	w14, w13, w15, lo
	cset	w15, lo
	cinc	w16, w10, hs
	cmp	w12, w9
	csel	w9, wzr, w15, lo
	csel	w10, w10, w16, lo
	csel	w12, w13, w14, lo
	mov	w11, w11
	orr	x11, x11, x12, lsl #32
	fmov	d0, x11
Lloh278:
	adrp	x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.81@PAGE
Lloh279:
	add	x11, x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.81@PAGEOFF
	ldr	d2, [x11, w9, uxtw #3]
	fsub	d3, d0, d2
	fadd	d4, d2, d0
	fmov	d5, #1.00000000
	fdiv	d4, d5, d4
	lsl	w11, w9, #18
	add	w11, w11, w12, lsr #1
	mov	w12, #537395200
	add	w11, w11, w12
	lsl	x11, x11, #32
	fmov	d5, x11
	fsub	d2, d2, d5
	fadd	d0, d2, d0
	fmul	d2, d3, d4
	fmov	x11, d2
	and	x11, x11, #0xffffffff00000000
	fmov	d6, x11
	fmul	d5, d5, d6
	fsub	d3, d3, d5
	fmul	d0, d0, d6
	fmul	d5, d2, d2
	fsub	d0, d3, d0
	mov.d	v4[1], v5[0]
	mov.16b	v3, v4
	mov.d	v3[0], v0[0]
	fmul.2d	v0, v4, v3
	mov	x11, #20207
	movk	x11, #19013, lsl #16
	movk	x11, #32296, lsl #32
	movk	x11, #16330, lsl #48
	fmov	d3, x11
	fmul	d3, d5, d3
	mov	x11, #56165
	movk	x11, #37833, lsl #16
	movk	x11, #34378, lsl #32
	movk	x11, #16333, lsl #48
	fmov	d4, x11
	fadd	d3, d3, d4
	fmul	d3, d5, d3
	mov	x11, #16641
	movk	x11, #43293, lsl #16
	movk	x11, #29792, lsl #32
	movk	x11, #16337, lsl #48
	fmov	d4, x11
	fadd	d3, d3, d4
	fmul	d3, d5, d3
	mov	x11, #9805
	movk	x11, #20879, lsl #16
	movk	x11, #21845, lsl #32
	movk	x11, #16341, lsl #48
	fmov	d4, x11
	fadd	d3, d3, d4
	fmul	d3, d5, d3
	mov	x11, #44031
	movk	x11, #56175, lsl #16
	movk	x11, #28086, lsl #32
	movk	x11, #16347, lsl #48
	fmov	d4, x11
	fadd	d3, d3, d4
	fmul	d3, d5, d3
	mov.16b	v4, v2
	mov.d	v4[1], v3[0]
	mov	x11, #3689348814741910323
	movk	x11, #13059
	movk	x11, #16355, lsl #48
	fmov	d3, x11
	dup.2d	v3, v3[0]
	mov.d	v3[0], v6[0]
	fadd.2d	v3, v4, v3
	fmul.2d	v3, v3, v0
	faddp.2d	d3, v3
	fmul	d4, d6, d6
	fmov	d5, #3.00000000
	fadd	d5, d4, d5
	fadd	d5, d5, d3
	fmov	x11, d5
	and	x11, x11, #0xffffffff00000000
	fmov	d5, x11
	fmov	d7, #-3.00000000
	fadd	d7, d5, d7
	fsub	d4, d7, d4
	fsub	d3, d3, d4
	fmul	d4, d6, d5
	fmul	d0, d5, d0
	fmul	d2, d2, d3
	fadd	d0, d0, d2
	fadd	d2, d4, d0
	fmov	x11, d2
	and	x11, x11, #0xffffffff00000000
	fmov	d2, x11
	fsub	d3, d2, d4
	fsub	d0, d0, d3
	mov	x11, #3758096384
	movk	x11, #50953, lsl #32
	movk	x11, #16366, lsl #48
	fmov	d3, x11
	fmul	d3, d2, d3
	mov	x11, #501
	movk	x11, #5211, lsl #16
	movk	x11, #12256, lsl #32
	movk	x11, #48702, lsl #48
	fmov	d4, x11
	fmul	d2, d2, d4
	mov	x11, #1021
	movk	x11, #56378, lsl #16
	movk	x11, #50953, lsl #32
	movk	x11, #16366, lsl #48
	fmov	d4, x11
	fmul	d0, d0, d4
Lloh280:
	adrp	x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.82@PAGE
Lloh281:
	add	x11, x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.82@PAGEOFF
	ldr	d4, [x11, w9, uxtw #3]
	fadd	d0, d0, d2
	fadd	d0, d4, d0
	scvtf	d4, w10
Lloh282:
	adrp	x10, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.83@PAGE
Lloh283:
	add	x10, x10, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.83@PAGEOFF
	fadd	d2, d3, d0
	ldr	d5, [x10, w9, uxtw #3]
	fadd	d2, d5, d2
	fadd	d2, d2, d4
	fmov	x9, d2
	and	x9, x9, #0xffffffff00000000
	fmov	d2, x9
	fsub	d4, d2, d4
	fsub	d4, d4, d5
	fsub	d3, d4, d3
LBB32_46:
	fsub	d0, d0, d3
	and	x8, x8, #0xffffffff00000000
	fmov	d3, x8
	fsub	d4, d1, d3
	fmul	d4, d4, d2
	fmul	d0, d1, d0
	fadd	d0, d4, d0
	fmul	d1, d2, d3
	fadd	d2, d1, d0
	fmov	x8, d2
	lsr	x9, x8, #32
	mov	w10, #1072693247
	add	w10, w10, #2560, lsl #12
	cmp	w9, w10
	b.le	LBB32_58
	mov	w10, #-1072693248
	add	w10, w9, w10
	sub	w10, w10, #2560, lsl #12
	orr	w10, w10, w8
	cbnz	w10, LBB32_70
	mov	x10, #33534
	movk	x10, #25899, lsl #16
	movk	x10, #5447, lsl #32
	movk	x10, #15511, lsl #48
	fmov	d3, x10
	fadd	d3, d0, d3
	fsub	d2, d2, d1
	fcmp	d3, d2
	b.le	LBB32_61
	b	LBB32_70
LBB32_49:
	cmp	w14, #1
	b.eq	LBB32_66
	cbnz	w14, LBB32_37
	fsub	d0, d0, d0
	fdiv	d2, d0, d0
	b	LBB32_10
LBB32_52:
	cbz	w13, LBB32_55
	mov	w15, #1072693248
	cmp	w13, w15
	b.eq	LBB32_55
	mov	w15, #2146435072
	cmp	w13, w15
	b.ne	LBB32_36
LBB32_55:
	fmov	d0, #1.00000000
	fdiv	d0, d0, d2
	cmp	x8, #0
	fcsel	d2, d0, d2, mi
	tbz	x11, #63, LBB32_10
	mov	w8, #-1072693248
	add	w8, w9, w8
	orr	w8, w14, w8
	cbz	w8, LBB32_78
	fneg	d0, d2
	cmp	w14, #1
	fcsel	d2, d0, d2, eq
	b	LBB32_10
LBB32_58:
	and	w10, w9, #0x7ffffc00
	mov	w11, #52223
	movk	w11, #16528, lsl #16
	cmp	w10, w11
	b.ls	LBB32_61
	mov	w10, #13312
	movk	w10, #16239, lsl #16
	add	w10, w9, w10
	orr	w10, w10, w8
	cbnz	w10, LBB32_65
	fsub	d2, d2, d1
	fcmp	d0, d2
	b.ls	LBB32_65
LBB32_61:
	and	w10, w9, #0x7fffffff
	mov	w11, #1071644672
	cmp	w10, w11
	b.ls	LBB32_72
	lsr	w10, w9, #20
	add	w10, w10, #2
	mov	w11, #1048576
	lsr	w10, w11, w10
	add	w9, w10, w9
	lsr	w10, w9, #20
	add	w12, w10, #1
	mov	w13, #-1048576
	asr	w12, w13, w12
	and	w12, w12, w9
	lsl	x12, x12, #32
	fmov	d2, x12
	bfxil	w11, w9, #0, #20
	mov	w9, #19
	sub	w9, w9, w10
	lsr	w9, w11, w9
	cmp	x8, #0
	cneg	w0, w9, mi
	fsub	d1, d1, d2
	fadd	d2, d0, d1
	fmov	x8, d2
	b	LBB32_73
LBB32_63:
	mov	w10, #1072693247
	cmp	w9, w10
	b.hs	LBB32_68
	tbnz	x8, #63, LBB32_70
LBB32_65:
	mov	x8, #62297
	movk	x8, #49912, lsl #16
	movk	x8, #28191, lsl #32
	movk	x8, #421, lsl #48
	b	LBB32_71
LBB32_66:
	fmov	d8, #-1.00000000
	b	LBB32_37
LBB32_67:
	cmp	w12, #0
	mov	x8, #9218868437227405312
	fmov	d0, x8
	movi.2d	v1, #0000000000000000
	fcsel	d2, d0, d1, gt
	b	LBB32_10
LBB32_68:
	mov	w10, #1072693248
	cmp	w9, w10
	b.ls	LBB32_79
	cmp	w12, #0
	b.le	LBB32_65
LBB32_70:
	mov	x8, #30108
	movk	x8, #34816, lsl #16
	movk	x8, #58428, lsl #32
	movk	x8, #32311, lsl #48
LBB32_71:
	fmov	d0, x8
	fmul	d1, d8, d0
	fmul	d2, d1, d0
	b	LBB32_10
LBB32_72:
	mov	w0, #0
LBB32_73:
	and	x8, x8, #0xffffffff00000000
	fmov	d2, x8
	mov	x8, #50865297686528
	movk	x8, #16358, lsl #48
	fmov	d3, x8
	fmul	d3, d2, d3
	fsub	d1, d2, d1
	fsub	d0, d0, d1
	mov	x8, #14831
	movk	x8, #65274, lsl #16
	movk	x8, #11842, lsl #32
	movk	x8, #16358, lsl #48
	fmov	d1, x8
	fmul	d0, d0, d1
	mov	x8, #27705
	movk	x8, #3240, lsl #16
	movk	x8, #23649, lsl #32
	movk	x8, #48672, lsl #48
	fmov	d1, x8
	fmul	d1, d2, d1
	fadd	d0, d0, d1
	fadd	d1, d3, d0
	fsub	d2, d1, d3
	fsub	d0, d0, d2
	fmul	d2, d1, d1
	mov	x8, #42192
	movk	x8, #29374, lsl #16
	movk	x8, #14185, lsl #32
	movk	x8, #15974, lsl #48
	fmov	d3, x8
	fmul	d3, d2, d3
	mov	x8, #27633
	movk	x8, #50642, lsl #16
	movk	x8, #48449, lsl #32
	movk	x8, #48827, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d3, d2, d3
	mov	x8, #56876
	movk	x8, #44837, lsl #16
	movk	x8, #22122, lsl #32
	movk	x8, #16145, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d3, d2, d3
	mov	x8, #48531
	movk	x8, #5822, lsl #16
	movk	x8, #49516, lsl #32
	movk	x8, #48998, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d3, d2, d3
	mov	x8, #6148914691236517205
	movk	x8, #21822
	movk	x8, #16325, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d2, d2, d3
	fsub	d2, d1, d2
	fmul	d3, d1, d2
	fmov	d4, #-2.00000000
	fadd	d2, d2, d4
	fdiv	d2, d3, d2
	fmul	d3, d1, d0
	fadd	d0, d0, d3
	fsub	d0, d2, d0
	fsub	d0, d1, d0
	fmov	d1, #1.00000000
	fadd	d0, d0, d1
	fmov	x8, d0
	lsl	w9, w0, #20
	add	x9, x9, x8, lsr #32
	cmp	w9, #256, lsl #12
	b.ge	LBB32_75
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5ldexp5ldexp
	b	LBB32_76
LBB32_75:
	bfi	x8, x9, #32, #32
	fmov	d0, x8
LBB32_76:
	fmul	d2, d8, d0
	b	LBB32_10
LBB32_77:
	fneg	d0, d1
	cmn	x8, #1
	movi.2d	v1, #0000000000000000
	fcsel	d2, d1, d0, gt
	b	LBB32_10
LBB32_78:
	fsub	d0, d2, d2
	fdiv	d2, d0, d0
	b	LBB32_10
LBB32_79:
	fmov	d0, #-1.00000000
	fadd	d0, d2, d0
	fmul	d2, d0, d0
	fmov	d3, #-0.25000000
	fmul	d3, d0, d3
	mov	x9, #6148914691236517205
	movk	x9, #16341, lsl #48
	fmov	d4, x9
	fadd	d3, d3, d4
	fmul	d3, d0, d3
	fmov	d4, #0.50000000
	fsub	d3, d4, d3
	fmul	d2, d2, d3
	mov	x9, #1610612736
	movk	x9, #5447, lsl #32
	movk	x9, #16375, lsl #48
	fmov	d3, x9
	fmul	d3, d0, d3
	mov	x9, #57156
	movk	x9, #63581, lsl #16
	movk	x9, #44555, lsl #32
	movk	x9, #15956, lsl #48
	fmov	d4, x9
	fmul	d0, d0, d4
	mov	x9, #33534
	movk	x9, #25899, lsl #16
	movk	x9, #5447, lsl #32
	movk	x9, #49143, lsl #48
	fmov	d4, x9
	fmul	d2, d2, d4
	fadd	d0, d0, d2
	fadd	d2, d3, d0
	fmov	x9, d2
	and	x9, x9, #0xffffffff00000000
	fmov	d2, x9
	fsub	d3, d2, d3
	b	LBB32_46
LBB32_80:
	mov	w15, #1043
	sub	w15, w15, w14
	lsr	w14, w10, w15
	lsl	w15, w14, w15
	cmp	w15, w10
	b.ne	LBB32_82
	and	w14, w14, #0x1
	mov	w15, #2
	sub	w14, w15, w14
	b	LBB32_16
LBB32_82:
	mov	w14, #0
	b	LBB32_16
	.loh AdrpAdd	Lloh282, Lloh283
	.loh AdrpAdd	Lloh280, Lloh281
	.loh AdrpAdd	Lloh278, Lloh279
	.cfi_endproc

	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0
lCPI33_0:
	.quad	0x3efa01a019cb1590
	.quad	0x3da8fae9be8838d4
lCPI33_1:
	.quad	0x3fa555555555554c
	.quad	0xbe927e4f809c52ad
lCPI33_2:
	.quad	0x3de5d93a5acfd57c
	.quad	0x3ec71de357b1fe7d
lCPI33_3:
	.quad	0xbe5ae5e68a2b9ceb
	.quad	0xbf2a01a019c161d5
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3sin3sin
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math3sin3sin:
	.cfi_startproc
	sub	sp, sp, #48
	.cfi_def_cfa_offset 48
	stp	x29, x30, [sp, #32]
	add	x29, sp, #32
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	fmov	x8, d0
	ubfx	x8, x8, #32, #31
	mov	w9, #8700
	movk	w9, #16361, lsl #16
	cmp	w8, w9
	b.hs	LBB33_4
	lsr	w9, w8, #20
	cmp	w9, #997
	b.hs	LBB33_6
	cmp	w8, #256, lsl #12
	b.hs	LBB33_10
	mov	x8, #4066750463515557888
	fmov	d1, x8
	fmul	d1, d0, d1
	b	LBB33_11
LBB33_4:
	lsr	w8, w8, #20
	cmp	w8, #2046
	b.ls	LBB33_7
	fsub	d0, d0, d0
	b	LBB33_17
LBB33_6:
	fmul	d1, d0, d0
	fmul	d2, d1, d1
	fmul	d2, d1, d2
	dup.2d	v3, v1[0]
Lloh284:
	adrp	x8, lCPI33_2@PAGE
Lloh285:
	ldr	q4, [x8, lCPI33_2@PAGEOFF]
	fmul.2d	v4, v4, v1[0]
Lloh286:
	adrp	x8, lCPI33_3@PAGE
Lloh287:
	ldr	q5, [x8, lCPI33_3@PAGEOFF]
	fadd.2d	v4, v4, v5
	mov.d	v3[0], v2[0]
	fmul.2d	v2, v3, v4
	mov	d3, v2[1]
	mov	x8, #63654
	movk	x8, #4368, lsl #16
	movk	x8, #4369, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fadd	d2, d2, d3
	fmul	d3, d0, d1
	fmul	d1, d1, d2
	mov	x8, #6148914691236517205
	movk	x8, #21833
	movk	x8, #49093, lsl #48
	fmov	d2, x8
	fadd	d1, d1, d2
	fmul	d1, d3, d1
	fadd	d0, d0, d1
	b	LBB33_17
LBB33_7:
	add	x0, sp, #8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math8rem_pio28rem_pio2
	ldr	w8, [sp, #16]
	ldr	d0, [sp, #8]
	ldr	d1, [sp, #24]
	and	w8, w8, #0x3
	fmul	d2, d0, d0
	fmul	d3, d2, d2
	cmp	w8, #1
	b.gt	LBB33_12
	cbnz	w8, LBB33_14
	mov	x8, #65149
	movk	x8, #22449, lsl #16
	movk	x8, #7651, lsl #32
	movk	x8, #16071, lsl #48
	fmov	d4, x8
	fmul	d4, d2, d4
	mov	x8, #25045
	movk	x8, #6593, lsl #16
	movk	x8, #416, lsl #32
	movk	x8, #48938, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d4, d2, d4
	mov	x8, #63654
	movk	x8, #4368, lsl #16
	movk	x8, #4369, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d3, d2, d3
	mov	x8, #54652
	movk	x8, #23247, lsl #16
	movk	x8, #55610, lsl #32
	movk	x8, #15845, lsl #48
	fmov	d5, x8
	fmul	d5, d2, d5
	mov	x8, #40171
	movk	x8, #35371, lsl #16
	movk	x8, #58854, lsl #32
	movk	x8, #48730, lsl #48
	fmov	d6, x8
	fadd	d5, d5, d6
	fmul	d3, d3, d5
	fadd	d3, d3, d4
	fmul	d4, d0, d2
	fmov	d5, #0.50000000
	fmul	d5, d1, d5
	fmul	d3, d4, d3
	fsub	d3, d5, d3
	fmul	d2, d2, d3
	fsub	d1, d2, d1
	mov	x8, #6148914691236517205
	movk	x8, #21833
	movk	x8, #16325, lsl #48
	fmov	d2, x8
	fmul	d2, d4, d2
	fadd	d1, d2, d1
	fsub	d0, d0, d1
	b	LBB33_17
LBB33_10:
	mov	x8, #5147614374084476928
	fmov	d1, x8
	fadd	d1, d0, d1
LBB33_11:
	str	d1, [sp, #8]
	ldr	d1, [sp, #8]
	b	LBB33_17
LBB33_12:
	cmp	w8, #2
	b.ne	LBB33_15
	mov	x8, #65149
	movk	x8, #22449, lsl #16
	movk	x8, #7651, lsl #32
	movk	x8, #16071, lsl #48
	fmov	d4, x8
	fmul	d4, d2, d4
	mov	x8, #25045
	movk	x8, #6593, lsl #16
	movk	x8, #416, lsl #32
	movk	x8, #48938, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d4, d2, d4
	mov	x8, #63654
	movk	x8, #4368, lsl #16
	movk	x8, #4369, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d3, d2, d3
	mov	x8, #54652
	movk	x8, #23247, lsl #16
	movk	x8, #55610, lsl #32
	movk	x8, #15845, lsl #48
	fmov	d5, x8
	fmul	d5, d2, d5
	mov	x8, #40171
	movk	x8, #35371, lsl #16
	movk	x8, #58854, lsl #32
	movk	x8, #48730, lsl #48
	fmov	d6, x8
	fadd	d5, d5, d6
	fmul	d3, d3, d5
	fadd	d3, d3, d4
	fmul	d4, d0, d2
	fmov	d5, #0.50000000
	fmul	d5, d1, d5
	fmul	d3, d4, d3
	fsub	d3, d5, d3
	fmul	d2, d2, d3
	fsub	d1, d2, d1
	mov	x8, #6148914691236517205
	movk	x8, #21833
	movk	x8, #16325, lsl #48
	fmov	d2, x8
	fmul	d2, d4, d2
	fadd	d1, d2, d1
	fsub	d0, d0, d1
	b	LBB33_16
LBB33_14:
	dup.2d	v4, v2[0]
Lloh288:
	adrp	x8, lCPI33_0@PAGE
Lloh289:
	ldr	q5, [x8, lCPI33_0@PAGEOFF]
	fmul.2d	v5, v5, v2[0]
	fmul	d3, d3, d3
	mov	x8, #20855
	movk	x8, #5825, lsl #16
	movk	x8, #49516, lsl #32
	movk	x8, #48982, lsl #48
	fmov	d6, x8
	fadd.2d	v6, v5, v6
	mov	x8, #45508
	movk	x8, #48564, lsl #16
	movk	x8, #61086, lsl #32
	movk	x8, #15905, lsl #48
	dup.2d	v7, x8
	fsub.2d	v5, v7, v5
	mov.d	v6[1], v5[1]
	fmul.2d	v5, v6, v2[0]
Lloh290:
	adrp	x8, lCPI33_1@PAGE
Lloh291:
	ldr	q6, [x8, lCPI33_1@PAGEOFF]
	fadd.2d	v5, v5, v6
	mov.d	v4[1], v3[0]
	fmul.2d	v3, v4, v5
	faddp.2d	d3, v3
	fmov	d4, #0.50000000
	fmul	d4, d2, d4
	fmov	d5, #1.00000000
	fsub	d6, d5, d4
	fsub	d5, d5, d6
	fsub	d4, d5, d4
	fmul	d2, d2, d3
	fmul	d0, d0, d1
	fsub	d0, d2, d0
	fadd	d0, d4, d0
	fadd	d0, d6, d0
	b	LBB33_17
LBB33_15:
	dup.2d	v4, v2[0]
Lloh292:
	adrp	x8, lCPI33_0@PAGE
Lloh293:
	ldr	q5, [x8, lCPI33_0@PAGEOFF]
	fmul.2d	v5, v5, v2[0]
	fmul	d3, d3, d3
	mov	x8, #20855
	movk	x8, #5825, lsl #16
	movk	x8, #49516, lsl #32
	movk	x8, #48982, lsl #48
	fmov	d6, x8
	mov	x8, #45508
	movk	x8, #48564, lsl #16
	movk	x8, #61086, lsl #32
	movk	x8, #15905, lsl #48
	dup.2d	v7, x8
	fadd.2d	v6, v5, v6
	fsub.2d	v5, v7, v5
	mov.d	v6[1], v5[1]
	fmul.2d	v5, v6, v2[0]
Lloh294:
	adrp	x8, lCPI33_1@PAGE
Lloh295:
	ldr	q6, [x8, lCPI33_1@PAGEOFF]
	fadd.2d	v5, v5, v6
	mov.d	v4[1], v3[0]
	fmul.2d	v3, v4, v5
	faddp.2d	d3, v3
	fmov	d4, #0.50000000
	fmul	d4, d2, d4
	fmov	d5, #1.00000000
	fsub	d6, d5, d4
	fsub	d5, d5, d6
	fsub	d4, d5, d4
	fmul	d2, d2, d3
	fmul	d0, d0, d1
	fsub	d0, d2, d0
	fadd	d0, d4, d0
	fadd	d0, d6, d0
LBB33_16:
	fneg	d0, d0
LBB33_17:
	.cfi_def_cfa wsp, 48
	ldp	x29, x30, [sp, #32]
	add	sp, sp, #48
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
	.loh AdrpLdr	Lloh286, Lloh287
	.loh AdrpAdrp	Lloh284, Lloh286
	.loh AdrpLdr	Lloh284, Lloh285
	.loh AdrpLdr	Lloh290, Lloh291
	.loh AdrpLdr	Lloh288, Lloh289
	.loh AdrpLdr	Lloh294, Lloh295
	.loh AdrpLdr	Lloh292, Lloh293
	.cfi_endproc

	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0
lCPI34_0:
	.quad	0x3ef375cbdb605373
	.quad	0x3efb2a7074bf7ad4
lCPI34_1:
	.quad	0x3f4344d8f2f26501
	.quad	0x3f3026f71a8d1068
lCPI34_2:
	.quad	0x3f6d6d22c9560328
	.quad	0x3f57dbc8fee08315
lCPI34_3:
	.quad	0x3f9664f48406d637
	.quad	0x3f8226e3e96e8493
lCPI34_4:
	.quad	0x3fc111111110fe7a
	.quad	0x3faba1ba1bb341fe
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3tan3tan
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math3tan3tan:
	.cfi_startproc
	sub	sp, sp, #48
	.cfi_def_cfa_offset 48
	stp	x29, x30, [sp, #32]
	add	x29, sp, #32
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_remember_state
	fmov	x8, d0
	ubfx	x9, x8, #32, #31
	mov	w10, #8700
	movk	w10, #16361, lsl #16
	cmp	w9, w10
	b.hs	LBB34_3
	lsr	w10, w9, #22
	cmp	w10, #249
	b.hs	LBB34_5
	mov	x8, #5147614374084476928
	fmov	d1, x8
	fadd	d1, d0, d1
	mov	x8, #4066750463515557888
	fmov	d2, x8
	fmul	d2, d0, d2
	cmp	w9, #256, lsl #12
	fcsel	d1, d2, d1, lo
	str	d1, [sp, #8]
	ldr	d1, [sp, #8]
	b	LBB34_11
LBB34_3:
	lsr	w8, w9, #20
	cmp	w8, #2046
	b.ls	LBB34_7
	fsub	d0, d0, d0
	b	LBB34_11
LBB34_5:
	and	x9, x8, #0x7ffffff800000000
	mov	x10, #162895224635392
	movk	x10, #16357, lsl #48
	cmp	x9, x10
	b.ls	LBB34_8
	movi.2d	v1, #0000000000000000
	movi.2d	v2, #0xffffffffffffffff
	fneg.2d	v2, v2
	bif.16b	v1, v0, v2
	fabs	d0, d0
	mov	x11, #11544
	movk	x11, #21572, lsl #16
	movk	x11, #8699, lsl #32
	movk	x11, #16361, lsl #48
	fmov	d2, x11
	fsub	d0, d2, d0
	mov	x11, #23559
	movk	x11, #13076, lsl #16
	movk	x11, #42534, lsl #32
	movk	x11, #15489, lsl #48
	fmov	d2, x11
	fsub	d1, d2, d1
	fadd	d1, d0, d1
	b	LBB34_9
LBB34_7:
	add	x0, sp, #8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math8rem_pio28rem_pio2
	ldr	w8, [sp, #16]
	ldr	d0, [sp, #8]
	ldr	d1, [sp, #24]
	and	w0, w8, #0x1
	.cfi_def_cfa wsp, 48
	ldp	x29, x30, [sp, #32]
	add	sp, sp, #48
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math5k_tan5k_tan
LBB34_8:
	.cfi_restore_state
	mov.16b	v1, v0
LBB34_9:
	fmul	d0, d1, d1
	fmul	d2, d0, d0
Lloh296:
	adrp	x11, lCPI34_0@PAGE
Lloh297:
	ldr	q3, [x11, lCPI34_0@PAGEOFF]
	fmul.2d	v3, v3, v2[0]
	mov	x11, #37542
	movk	x11, #41015, lsl #16
	movk	x11, #32392, lsl #32
	movk	x11, #16148, lsl #48
	fmov	d4, x11
	fsub.2d	v4, v4, v3
	mov	x11, #42985
	movk	x11, #13040, lsl #16
	movk	x11, #47119, lsl #32
	movk	x11, #16146, lsl #48
	dup.2d	v5, x11
	fadd.2d	v3, v3, v5
	mov.d	v4[1], v3[1]
	fmul.2d	v3, v4, v2[0]
Lloh298:
	adrp	x11, lCPI34_1@PAGE
Lloh299:
	ldr	q4, [x11, lCPI34_1@PAGEOFF]
	fadd.2d	v3, v3, v4
	fmul.2d	v3, v3, v2[0]
Lloh300:
	adrp	x11, lCPI34_2@PAGE
Lloh301:
	ldr	q4, [x11, lCPI34_2@PAGEOFF]
	fadd.2d	v3, v3, v4
	fmul.2d	v3, v3, v2[0]
Lloh302:
	adrp	x11, lCPI34_3@PAGE
Lloh303:
	ldr	q4, [x11, lCPI34_3@PAGEOFF]
	fadd.2d	v3, v3, v4
	fmul.2d	v2, v3, v2[0]
Lloh304:
	adrp	x11, lCPI34_4@PAGE
Lloh305:
	ldr	q3, [x11, lCPI34_4@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul.d	d3, d0, v2[1]
	fmul	d4, d1, d0
	fadd	d2, d2, d3
	fmul	d2, d4, d2
	movi.2d	v3, #0000000000000000
	fadd	d2, d2, d3
	fmul	d0, d0, d2
	fadd	d0, d0, d3
	mov	x11, #6148914691236517205
	movk	x11, #21859
	movk	x11, #16341, lsl #48
	fmov	d2, x11
	fmul	d2, d4, d2
	fadd	d2, d2, d0
	fadd	d0, d1, d2
	cmp	x9, x10
	b.ls	LBB34_11
	fmul	d3, d0, d0
	fmov	d4, #1.00000000
	fadd	d0, d0, d4
	fdiv	d0, d3, d0
	fsub	d0, d2, d0
	fadd	d0, d1, d0
	fadd	d0, d0, d0
	fsub	d0, d4, d0
	fneg	d1, d0
	cmp	x8, #0
	fcsel	d0, d1, d0, mi
LBB34_11:
	.cfi_def_cfa wsp, 48
	ldp	x29, x30, [sp, #32]
	add	sp, sp, #48
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
	.loh AdrpLdr	Lloh304, Lloh305
	.loh AdrpAdrp	Lloh302, Lloh304
	.loh AdrpLdr	Lloh302, Lloh303
	.loh AdrpAdrp	Lloh300, Lloh302
	.loh AdrpLdr	Lloh300, Lloh301
	.loh AdrpAdrp	Lloh298, Lloh300
	.loh AdrpLdr	Lloh298, Lloh299
	.loh AdrpLdr	Lloh296, Lloh297
	.cfi_endproc

	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0
lCPI35_0:
	.quad	0xbfa48228b5688f3b
	.quad	0xbfe6066c1b8d0159
lCPI35_1:
	.quad	0x3fc9c1550e884455
	.quad	0x40002ae59c598ac8
lCPI35_2:
	.quad	0xbfd4d61203eb6f7d
	.quad	0xc0033a271c8a2d4b
lCPI35_3:
	.quad	0x3fc5555555555555
	.quad	0x3ff0000000000000
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4acos4acos
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math4acos4acos:
	.cfi_startproc
	fmov	x8, d0
	lsr	x10, x8, #32
	ubfx	x9, x8, #32, #31
	ubfx	w10, w10, #20, #11
	cmp	w10, #1022
	b.ls	LBB35_3
	mov	w10, #-1072693248
	add	w9, w9, w10
	orr	w9, w9, w8
	cbz	w9, LBB35_8
	fsub	d0, d0, d0
	movi.2d	v1, #0000000000000000
	fdiv	d0, d1, d0
	ret
LBB35_3:
	lsr	w10, w9, #21
	cmp	w10, #511
	b.hs	LBB35_6
	mov	w8, #1012924416
	cmp	w9, w8
	b.hi	LBB35_9
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #16377, lsl #48
	fmov	d0, x8
	ret
LBB35_6:
	fmov	d1, #1.00000000
	tbnz	x8, #63, LBB35_10
	fsub	d0, d1, d0
	fmov	d1, #0.50000000
	fmul	d0, d0, d1
	mov.16b	v1, v0
	; InlineAsm Start
	fsqrt	d1, d1
	; InlineAsm End
	fmov	x8, d1
	and	x8, x8, #0xffffffff00000000
	fmov	d2, x8
	fmul	d3, d2, d2
	fsub	d3, d0, d3
	mov	x8, #63241
	movk	x8, #3581, lsl #16
	movk	x8, #15841, lsl #32
	movk	x8, #16130, lsl #48
	fmov	d4, x8
	fmul	d4, d0, d4
	mov	x8, #45704
	movk	x8, #29953, lsl #16
	movk	x8, #61408, lsl #32
	movk	x8, #16201, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	mov	x8, #37506
	movk	x8, #45358, lsl #16
	movk	x8, #47301, lsl #32
	movk	x8, #16307, lsl #48
	fmov	d5, x8
	dup.2d	v5, v5[0]
	mov.d	v5[0], v4[0]
	fadd	d4, d1, d2
	fdiv	d3, d3, d4
	fmul.2d	v4, v5, v0[0]
Lloh306:
	adrp	x8, lCPI35_0@PAGE
Lloh307:
	ldr	q5, [x8, lCPI35_0@PAGEOFF]
	fadd.2d	v4, v4, v5
	fmul.2d	v4, v4, v0[0]
Lloh308:
	adrp	x8, lCPI35_1@PAGE
Lloh309:
	ldr	q5, [x8, lCPI35_1@PAGEOFF]
	fadd.2d	v4, v4, v5
	fmul.2d	v4, v4, v0[0]
Lloh310:
	adrp	x8, lCPI35_2@PAGE
Lloh311:
	ldr	q5, [x8, lCPI35_2@PAGEOFF]
	fadd.2d	v4, v4, v5
	fmul.2d	v4, v4, v0[0]
Lloh312:
	adrp	x8, lCPI35_3@PAGE
Lloh313:
	ldr	q5, [x8, lCPI35_3@PAGEOFF]
	fadd.2d	v4, v4, v5
	fmul	d0, d0, d4
	mov	d4, v4[1]
	fdiv	d0, d0, d4
	fmul	d0, d0, d1
	fadd	d0, d0, d3
	fadd	d0, d0, d2
	fadd	d0, d0, d0
	ret
LBB35_8:
	cmn	x8, #1
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #16393, lsl #48
	fmov	d0, x8
	movi.2d	v1, #0000000000000000
	fcsel	d0, d1, d0, gt
	ret
LBB35_9:
	fmul	d1, d0, d0
	mov	x8, #63241
	movk	x8, #3581, lsl #16
	movk	x8, #15841, lsl #32
	movk	x8, #16130, lsl #48
	fmov	d2, x8
	fmul	d2, d1, d2
	mov	x8, #45704
	movk	x8, #29953, lsl #16
	movk	x8, #61408, lsl #32
	movk	x8, #16201, lsl #48
	fmov	d3, x8
	mov	x8, #37506
	movk	x8, #45358, lsl #16
	movk	x8, #47301, lsl #32
	movk	x8, #16307, lsl #48
	fmov	d4, x8
	dup.2d	v4, v4[0]
	fadd	d2, d2, d3
	mov.d	v4[0], v2[0]
	fmul.2d	v2, v4, v1[0]
Lloh314:
	adrp	x8, lCPI35_0@PAGE
Lloh315:
	ldr	q3, [x8, lCPI35_0@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul.2d	v2, v2, v1[0]
Lloh316:
	adrp	x8, lCPI35_1@PAGE
Lloh317:
	ldr	q3, [x8, lCPI35_1@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul.2d	v2, v2, v1[0]
Lloh318:
	adrp	x8, lCPI35_2@PAGE
Lloh319:
	ldr	q3, [x8, lCPI35_2@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul.2d	v2, v2, v1[0]
Lloh320:
	adrp	x8, lCPI35_3@PAGE
Lloh321:
	ldr	q3, [x8, lCPI35_3@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul	d1, d1, d2
	mov	d2, v2[1]
	fdiv	d1, d1, d2
	fmul	d1, d0, d1
	mov	x8, #23559
	movk	x8, #13076, lsl #16
	movk	x8, #42534, lsl #32
	movk	x8, #15505, lsl #48
	fmov	d2, x8
	fsub	d1, d2, d1
	fsub	d0, d1, d0
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #16377, lsl #48
	fmov	d1, x8
	fadd	d0, d0, d1
	ret
LBB35_10:
	fadd	d0, d0, d1
	fmov	d1, #0.50000000
	fmul	d0, d0, d1
	mov	x8, #63241
	movk	x8, #3581, lsl #16
	movk	x8, #15841, lsl #32
	movk	x8, #16130, lsl #48
	fmov	d1, x8
	fmul	d1, d0, d1
	mov	x8, #45704
	movk	x8, #29953, lsl #16
	movk	x8, #61408, lsl #32
	movk	x8, #16201, lsl #48
	fmov	d2, x8
	fadd	d1, d1, d2
	mov	x8, #37506
	movk	x8, #45358, lsl #16
	movk	x8, #47301, lsl #32
	movk	x8, #16307, lsl #48
	fmov	d2, x8
	dup.2d	v2, v2[0]
	mov.d	v2[0], v1[0]
	fmul.2d	v1, v2, v0[0]
Lloh322:
	adrp	x8, lCPI35_0@PAGE
Lloh323:
	ldr	q2, [x8, lCPI35_0@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul.2d	v1, v1, v0[0]
Lloh324:
	adrp	x8, lCPI35_1@PAGE
Lloh325:
	ldr	q2, [x8, lCPI35_1@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul.2d	v1, v1, v0[0]
Lloh326:
	adrp	x8, lCPI35_2@PAGE
Lloh327:
	ldr	q2, [x8, lCPI35_2@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul.2d	v1, v1, v0[0]
Lloh328:
	adrp	x8, lCPI35_3@PAGE
Lloh329:
	ldr	q2, [x8, lCPI35_3@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul	d2, d0, d1
	; InlineAsm Start
	fsqrt	d0, d0
	; InlineAsm End
	mov	d1, v1[1]
	fdiv	d1, d2, d1
	fmul	d1, d1, d0
	mov	x8, #23559
	movk	x8, #13076, lsl #16
	movk	x8, #42534, lsl #32
	movk	x8, #48273, lsl #48
	fmov	d2, x8
	fadd	d1, d1, d2
	fadd	d0, d0, d1
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #16377, lsl #48
	fmov	d1, x8
	fsub	d0, d1, d0
	fadd	d0, d0, d0
	ret
	.loh AdrpLdr	Lloh312, Lloh313
	.loh AdrpAdrp	Lloh310, Lloh312
	.loh AdrpLdr	Lloh310, Lloh311
	.loh AdrpAdrp	Lloh308, Lloh310
	.loh AdrpLdr	Lloh308, Lloh309
	.loh AdrpAdrp	Lloh306, Lloh308
	.loh AdrpLdr	Lloh306, Lloh307
	.loh AdrpLdr	Lloh320, Lloh321
	.loh AdrpAdrp	Lloh318, Lloh320
	.loh AdrpLdr	Lloh318, Lloh319
	.loh AdrpAdrp	Lloh316, Lloh318
	.loh AdrpLdr	Lloh316, Lloh317
	.loh AdrpAdrp	Lloh314, Lloh316
	.loh AdrpLdr	Lloh314, Lloh315
	.loh AdrpLdr	Lloh328, Lloh329
	.loh AdrpAdrp	Lloh326, Lloh328
	.loh AdrpLdr	Lloh326, Lloh327
	.loh AdrpAdrp	Lloh324, Lloh326
	.loh AdrpLdr	Lloh324, Lloh325
	.loh AdrpAdrp	Lloh322, Lloh324
	.loh AdrpLdr	Lloh322, Lloh323
	.cfi_endproc

	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0
lCPI36_0:
	.quad	0xbfa48228b5688f3b
	.quad	0xbfe6066c1b8d0159
lCPI36_1:
	.quad	0x3fc9c1550e884455
	.quad	0x40002ae59c598ac8
lCPI36_2:
	.quad	0xbfd4d61203eb6f7d
	.quad	0xc0033a271c8a2d4b
lCPI36_3:
	.quad	0x3fc5555555555555
	.quad	0x3ff0000000000000
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4asin4asin
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math4asin4asin:
	.cfi_startproc
	fmov	x8, d0
	lsr	x10, x8, #32
	ubfx	x9, x8, #32, #31
	ubfx	w10, w10, #20, #11
	cmp	w10, #1022
	b.ls	LBB36_3
	mov	w10, #-1072693248
	add	w9, w9, w10
	orr	w8, w9, w8
	cbz	w8, LBB36_7
	fsub	d0, d0, d0
	movi.2d	v1, #0000000000000000
	fdiv	d0, d1, d0
	ret
LBB36_3:
	lsr	w10, w9, #21
	cmp	w10, #511
	b.hs	LBB36_6
	sub	w8, w9, #256, lsl #12
	lsr	w8, w8, #22
	cmp	w8, #249
	b.hs	LBB36_8
	ret
LBB36_6:
	fabs	d0, d0
	fmov	d1, #1.00000000
	fsub	d0, d1, d0
	fmov	d2, #0.50000000
	fmul	d0, d0, d2
	mov.16b	v2, v0
	; InlineAsm Start
	fsqrt	d2, d2
	; InlineAsm End
	mov	x10, #63241
	movk	x10, #3581, lsl #16
	movk	x10, #15841, lsl #32
	movk	x10, #16130, lsl #48
	fmov	d3, x10
	fmul	d3, d0, d3
	mov	x10, #45704
	movk	x10, #29953, lsl #16
	movk	x10, #61408, lsl #32
	movk	x10, #16201, lsl #48
	fmov	d4, x10
	fadd	d3, d3, d4
	fmul	d3, d0, d3
	mov	x10, #36667
	movk	x10, #46440, lsl #16
	movk	x10, #33320, lsl #32
	movk	x10, #49060, lsl #48
	fmov	d4, x10
	fadd	d3, d3, d4
	fmul	d3, d0, d3
	mov	x10, #17493
	movk	x10, #3720, lsl #16
	movk	x10, #49493, lsl #32
	movk	x10, #16329, lsl #48
	fmov	d4, x10
	fadd	d3, d3, d4
	fmul	d3, d0, d3
	mov	x10, #28541
	movk	x10, #1003, lsl #16
	movk	x10, #54802, lsl #32
	movk	x10, #49108, lsl #48
	fmov	d4, x10
	fadd	d3, d3, d4
	fmul	d3, d0, d3
	mov	x10, #6148914691236517205
	movk	x10, #16325, lsl #48
	fmov	d4, x10
	fadd	d3, d3, d4
	fmul	d3, d0, d3
	mov	x10, #37506
	movk	x10, #45358, lsl #16
	movk	x10, #47301, lsl #32
	movk	x10, #16307, lsl #48
	fmov	d4, x10
	fmul	d4, d0, d4
	mov	x10, #345
	movk	x10, #7053, lsl #16
	movk	x10, #1644, lsl #32
	movk	x10, #49126, lsl #48
	fmov	d5, x10
	fadd	d4, d4, d5
	fmul	d4, d0, d4
	mov	x10, #35528
	movk	x10, #40025, lsl #16
	movk	x10, #10981, lsl #32
	movk	x10, #16384, lsl #48
	fmov	d5, x10
	fadd	d4, d4, d5
	fmul	d4, d0, d4
	mov	x10, #11595
	movk	x10, #7306, lsl #16
	movk	x10, #14887, lsl #32
	movk	x10, #49155, lsl #48
	fmov	d5, x10
	fadd	d4, d4, d5
	fmul	d4, d0, d4
	fadd	d1, d4, d1
	fdiv	d1, d3, d1
	mov	w10, #13106
	movk	w10, #16367, lsl #16
	cmp	w9, w10
	fmul	d3, d1, d2
	fadd	d3, d2, d3
	fadd	d3, d3, d3
	mov	x9, #23559
	movk	x9, #13076, lsl #16
	movk	x9, #42534, lsl #32
	movk	x9, #48273, lsl #48
	fmov	d4, x9
	fadd	d3, d3, d4
	mov	x9, #11544
	movk	x9, #21572, lsl #16
	movk	x9, #8699, lsl #32
	movk	x9, #16377, lsl #48
	fmov	d4, x9
	fsub	d3, d4, d3
	fmov	x9, d2
	and	x9, x9, #0xffffffff00000000
	fmov	d4, x9
	fmul	d5, d4, d4
	fsub	d0, d0, d5
	fadd	d5, d2, d4
	fdiv	d0, d0, d5
	fadd	d2, d2, d2
	fmul	d1, d1, d2
	fadd	d0, d0, d0
	mov	x9, #23559
	movk	x9, #13076, lsl #16
	movk	x9, #42534, lsl #32
	movk	x9, #15505, lsl #48
	fmov	d2, x9
	fsub	d0, d2, d0
	fadd	d2, d4, d4
	mov	x9, #11544
	movk	x9, #21572, lsl #16
	movk	x9, #8699, lsl #32
	movk	x9, #16361, lsl #48
	fmov	d4, x9
	fsub	d2, d4, d2
	fsub	d0, d0, d1
	fadd	d0, d2, d0
	fadd	d0, d0, d4
	fcsel	d0, d3, d0, hi
	fneg	d1, d0
	cmp	x8, #0
	fcsel	d0, d1, d0, mi
	ret
LBB36_7:
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #16377, lsl #48
	fmov	d1, x8
	fmul	d0, d0, d1
	mov	x8, #4066750463515557888
	fmov	d1, x8
	fadd	d0, d0, d1
	ret
LBB36_8:
	fmul	d1, d0, d0
	mov	x8, #63241
	movk	x8, #3581, lsl #16
	movk	x8, #15841, lsl #32
	movk	x8, #16130, lsl #48
	fmov	d2, x8
	fmul	d2, d1, d2
	mov	x8, #45704
	movk	x8, #29953, lsl #16
	movk	x8, #61408, lsl #32
	movk	x8, #16201, lsl #48
	fmov	d3, x8
	mov	x8, #37506
	movk	x8, #45358, lsl #16
	movk	x8, #47301, lsl #32
	movk	x8, #16307, lsl #48
	fmov	d4, x8
	dup.2d	v4, v4[0]
	fadd	d2, d2, d3
	mov.d	v4[0], v2[0]
	fmul.2d	v2, v4, v1[0]
Lloh330:
	adrp	x8, lCPI36_0@PAGE
Lloh331:
	ldr	q3, [x8, lCPI36_0@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul.2d	v2, v2, v1[0]
Lloh332:
	adrp	x8, lCPI36_1@PAGE
Lloh333:
	ldr	q3, [x8, lCPI36_1@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul.2d	v2, v2, v1[0]
Lloh334:
	adrp	x8, lCPI36_2@PAGE
Lloh335:
	ldr	q3, [x8, lCPI36_2@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul.2d	v2, v2, v1[0]
Lloh336:
	adrp	x8, lCPI36_3@PAGE
Lloh337:
	ldr	q3, [x8, lCPI36_3@PAGEOFF]
	fadd.2d	v2, v2, v3
	fmul	d1, d1, d2
	mov	d2, v2[1]
	fdiv	d1, d1, d2
	fmul	d1, d0, d1
	fadd	d0, d0, d1
	ret
	.loh AdrpLdr	Lloh336, Lloh337
	.loh AdrpAdrp	Lloh334, Lloh336
	.loh AdrpLdr	Lloh334, Lloh335
	.loh AdrpAdrp	Lloh332, Lloh334
	.loh AdrpLdr	Lloh332, Lloh333
	.loh AdrpAdrp	Lloh330, Lloh332
	.loh AdrpLdr	Lloh330, Lloh331
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4atan4atan
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math4atan4atan:
	.cfi_startproc
	fmov	x8, d0
	lsr	x9, x8, #32
	ubfx	w9, w9, #20, #11
	cmp	w9, #1088
	b.ls	LBB37_3
	fcmp	d0, d0
	b.vs	LBB37_18
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #16377, lsl #48
	fmov	d1, x8
	movi.2d	v2, #0xffffffffffffffff
	fneg.2d	v2, v2
	bit.16b	v0, v1, v2
	ret
LBB37_3:
	ubfx	x10, x8, #32, #31
	lsr	w9, w10, #18
	cmp	w9, #4087
	b.hs	LBB37_7
	lsr	w11, w10, #22
	cmp	w11, #249
	b.hs	LBB37_10
	cmp	w10, #256, lsl #12
	b.hs	LBB37_18
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	fcvt	s1, d0
	str	s1, [sp, #12]
	ldr	s1, [sp, #12]
	add	sp, sp, #16
	.cfi_def_cfa_offset 0
	ret
LBB37_7:
	fabs	d0, d0
	mov	w11, #1072889856
	cmp	w10, w11
	b.hs	LBB37_11
	mov	w11, #1072037888
	cmp	w10, w11
	b.hs	LBB37_13
	mov	x10, #0
	fadd	d1, d0, d0
	fmov	d2, #-1.00000000
	fadd	d1, d1, d2
	fmov	d2, #2.00000000
	fadd	d0, d0, d2
	b	LBB37_15
LBB37_10:
	mov	w10, #-1
	b	LBB37_16
LBB37_11:
	mov	w11, #32768
	movk	w11, #16387, lsl #16
	cmp	w10, w11
	b.hs	LBB37_14
	fmov	d1, #-1.50000000
	fadd	d1, d0, d1
	fmov	d2, #1.50000000
	fmul	d0, d0, d2
	fmov	d2, #1.00000000
	fadd	d0, d0, d2
	fdiv	d0, d1, d0
	mov	w10, #2
	b	LBB37_16
LBB37_13:
	fmov	d1, #-1.00000000
	fadd	d1, d0, d1
	fmov	d2, #1.00000000
	fadd	d0, d0, d2
	mov	w10, #1
	b	LBB37_15
LBB37_14:
	fmov	d1, #-1.00000000
	mov	w10, #3
LBB37_15:
	fdiv	d0, d1, d0
LBB37_16:
	fmul	d1, d0, d0
	fmul	d2, d1, d1
	mov	x11, #55825
	movk	x11, #58146, lsl #16
	movk	x11, #44346, lsl #32
	movk	x11, #16272, lsl #48
	fmov	d3, x11
	fmul	d3, d2, d3
	mov	x11, #3563
	movk	x11, #9334, lsl #16
	movk	x11, #31563, lsl #32
	movk	x11, #16297, lsl #48
	fmov	d4, x11
	fadd	d3, d3, d4
	fmul	d3, d2, d3
	mov	x11, #15697
	movk	x11, #41168, lsl #16
	movk	x11, #3430, lsl #32
	movk	x11, #16305, lsl #48
	fmov	d4, x11
	fadd	d3, d3, d4
	fmul	d3, d2, d3
	mov	x11, #8302
	movk	x11, #50508, lsl #16
	movk	x11, #17869, lsl #32
	movk	x11, #16311, lsl #48
	fmov	d4, x11
	fadd	d3, d3, d4
	fmul	d3, d2, d3
	mov	x11, #33791
	movk	x11, #37376, lsl #16
	movk	x11, #18724, lsl #32
	movk	x11, #16322, lsl #48
	fmov	d4, x11
	fadd	d3, d3, d4
	fmul	d3, d2, d3
	mov	x11, #6148914691236517205
	movk	x11, #21773
	movk	x11, #16341, lsl #48
	fmov	d4, x11
	fadd	d3, d3, d4
	fmul	d1, d1, d3
	mov	x11, #27695
	movk	x11, #11370, lsl #16
	movk	x11, #46148, lsl #32
	movk	x11, #49058, lsl #48
	fmov	d3, x11
	fmul	d3, d2, d3
	mov	x11, #64922
	movk	x11, #21214, lsl #16
	movk	x11, #56877, lsl #32
	movk	x11, #49069, lsl #48
	fmov	d4, x11
	fadd	d3, d3, d4
	fmul	d3, d2, d3
	mov	x11, #39533
	movk	x11, #44916, lsl #16
	movk	x11, #45298, lsl #32
	movk	x11, #49075, lsl #48
	fmov	d4, x11
	fadd	d3, d3, d4
	fmul	d3, d2, d3
	mov	x11, #5745
	movk	x11, #65059, lsl #16
	movk	x11, #29126, lsl #32
	movk	x11, #49084, lsl #48
	fmov	d4, x11
	fadd	d3, d3, d4
	fmul	d3, d2, d3
	mov	x11, #60356
	movk	x11, #39320, lsl #16
	movk	x11, #39321, lsl #32
	movk	x11, #49097, lsl #48
	fmov	d4, x11
	fadd	d3, d3, d4
	fmul	d2, d2, d3
	cmp	w9, #4087
	b.hs	LBB37_19
	fadd	d1, d2, d1
	fmul	d1, d0, d1
	fsub	d0, d0, d1
LBB37_18:
	ret
LBB37_19:
Lloh338:
	adrp	x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.84@PAGE
Lloh339:
	add	x9, x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.84@PAGEOFF
	ldr	d3, [x9, x10, lsl #3]
	fadd	d1, d2, d1
	fmul	d1, d0, d1
Lloh340:
	adrp	x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.85@PAGE
Lloh341:
	add	x9, x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.85@PAGEOFF
	ldr	d2, [x9, x10, lsl #3]
	fsub	d1, d1, d2
	fsub	d0, d1, d0
	fsub	d0, d3, d0
	fneg	d1, d0
	cmp	x8, #0
	fcsel	d0, d1, d0, mi
	ret
	.loh AdrpAdd	Lloh340, Lloh341
	.loh AdrpAdd	Lloh338, Lloh339
	.cfi_endproc

	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0
lCPI38_0:
	.quad	0x3ff0000000000000
	.quad	0xbff0000000000000
lCPI38_1:
	.quad	0x3fe0000000000000
	.quad	0xbfe0000000000000
lCPI38_2:
	.quad	0x3fd0000000000000
	.quad	0xbfd0000000000000
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4cbrt4cbrt
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math4cbrt4cbrt:
	.cfi_startproc
Lloh342:
	adrp	x8, lCPI38_0@PAGE
Lloh343:
	ldr	q1, [x8, lCPI38_0@PAGEOFF]
Lloh344:
	adrp	x8, lCPI38_1@PAGE
Lloh345:
	ldr	q2, [x8, lCPI38_1@PAGEOFF]
	stp	q1, q2, [sp, #-48]!
	.cfi_def_cfa_offset 48
	.cfi_remember_state
Lloh346:
	adrp	x8, lCPI38_2@PAGE
Lloh347:
	ldr	q1, [x8, lCPI38_2@PAGEOFF]
	str	q1, [sp, #32]
	fmov	x9, d0
	lsr	x10, x9, #52
	ubfx	x8, x9, #52, #11
	add	w10, w10, #1
	tst	w10, #0x7fe
	b.eq	LBB38_2
	mov	x10, x9
	b	LBB38_5
LBB38_2:
	fmov	x10, d0
	ands	x10, x10, #0x7fffffffffffffff
	b.eq	LBB38_15
	cmp	w8, #2047
	b.eq	LBB38_15
	clz	x11, x10
	add	w10, w11, #53
	lsl	x10, x9, x10
	sub	w8, w8, w11
	add	w8, w8, #12
LBB38_5:
	mov	x11, #4607182418800017408
	bfxil	x11, x10, #0, #52
	add	w10, w8, #3072
	and	w8, w10, #0xffff
	mov	w12, #43691
	mul	w8, w8, w12
	lsr	w8, w8, #17
	add	w12, w8, w8, lsl #1
	sub	w10, w10, w12
	and	x12, x10, #0xffff
	add	x10, x11, x10, lsl #52
	fmov	d1, x10
	movi.2d	v2, #0xffffffffffffffff
	fneg.2d	v2, v2
	bsl.16b	v2, v1, v0
	extr	x10, x12, x9, #63
	fmov	d4, x11
	fmov	d3, #1.00000000
Lloh348:
	adrp	x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.86@PAGE
Lloh349:
	add	x11, x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.86@PAGEOFF
	fdiv	d5, d3, d4
	ldr	x11, [x11, x12, lsl #3]
	and	x9, x9, #0x8000000000000000
	orr	x9, x11, x9
	mov	x11, sp
	ldr	d3, [x11, x10, lsl #3]
	fmul	d3, d3, d5
	fmul	d6, d4, d4
	mov	x10, #7589
	movk	x10, #59725, lsl #16
	movk	x10, #51619, lsl #32
	movk	x10, #16354, lsl #48
	fmov	d7, x10
	fmul	d7, d4, d7
	mov	x10, #61340
	movk	x10, #48335, lsl #16
	movk	x10, #45242, lsl #32
	movk	x10, #16353, lsl #48
	fmov	d16, x10
	fadd	d7, d7, d16
	mov	x10, #39687
	movk	x10, #58604, lsl #16
	movk	x10, #43219, lsl #32
	movk	x10, #16279, lsl #48
	fmov	d16, x10
	fmul	d4, d4, d16
	mov	x10, #56762
	movk	x10, #45473, lsl #16
	movk	x10, #56368, lsl #32
	movk	x10, #49092, lsl #48
	fmov	d16, x10
	fadd	d4, d4, d16
	fmul	d4, d6, d4
	fadd	d4, d7, d4
	fmul	d6, d4, d4
	fmul	d5, d5, d4
	fmul	d5, d6, d5
	fmov	d6, #-1.00000000
	fadd	d5, d5, d6
	fmul	d6, d4, d5
	mov	x10, #50972
	movk	x10, #7281, lsl #16
	movk	x10, #29127, lsl #32
	movk	x10, #49100, lsl #48
	fmov	d7, x10
	fmul	d5, d5, d7
	mov	x10, #6148914691236517205
	movk	x10, #16341, lsl #48
	fmov	d7, x10
	fadd	d5, d5, d7
	fmul	d5, d6, d5
	fsub	d4, d4, d5
	fmov	d5, x9
	fmul	d6, d4, d5
	fmul	d4, d6, d6
	fnmul	d5, d6, d6
	mov.16b	v16, v6
	; InlineAsm Start
	fmadd	d16, d16, d6, d5
	; InlineAsm End
	fmul	d5, d6, d4
	fnmul	d17, d6, d4
	mov.16b	v18, v6
	; InlineAsm Start
	fmadd	d18, d18, d4, d17
	; InlineAsm End
	fmul	d4, d16, d6
	fadd	d4, d18, d4
	fsub	d5, d5, d2
	fadd	d4, d4, d5
	fmul	d4, d3, d4
	fmul	d5, d6, d7
	fmul	d5, d5, d4
	fsub	d4, d6, d5
	fsub	d6, d6, d4
	fabd	d7, d6, d5
	mov	x9, #-4854880398305394688
	fmov	d16, x9
	fadd	d16, d7, d16
	fabs	d16, d16
	mov	x9, #-4848124998864338944
	fmov	d17, x9
	fadd	d7, d7, d17
	fabs	d7, d7
	mov	x9, #4269412446747230208
	fmov	d17, x9
	fcmp	d16, d17
	fmov	d16, x9
	fccmp	d7, d16, #8, pl
	b.pl	LBB38_8
	fmul	d5, d4, d4
	fnmul	d6, d4, d4
	mov.16b	v7, v4
	; InlineAsm Start
	fmadd	d7, d7, d4, d6
	; InlineAsm End
	fmul	d6, d4, d5
	fnmul	d16, d4, d5
	mov.16b	v17, v4
	; InlineAsm Start
	fmadd	d17, d17, d5, d16
	; InlineAsm End
	fmul	d5, d4, d7
	fadd	d5, d17, d5
	fsub	d2, d6, d2
	fadd	d2, d2, d5
	fmul	d2, d3, d2
	mov	x9, #6148914691236517205
	movk	x9, #16341, lsl #48
	fmov	d3, x9
	fmul	d3, d4, d3
	fmul	d5, d3, d2
	fsub	d3, d4, d5
	fsub	d4, d4, d3
	fsub	d2, d4, d5
	fabd	d4, d4, d5
	mov	x9, #-4854880398305394688
	fmov	d5, x9
	fadd	d5, d4, d5
	fabs	d5, d5
	mov	x9, #-4848124998864338944
	fmov	d6, x9
	fadd	d4, d4, d6
	fabs	d4, d4
	mov	x9, #4165829655317708800
	fmov	d6, x9
	fcmp	d5, d6
	fmov	d5, x9
	fccmp	d4, d5, #8, pl
	b.pl	LBB38_9
	mov	x9, #54684
	movk	x9, #53480, lsl #16
	movk	x9, #40213, lsl #32
	movk	x9, #16375, lsl #48
	fmov	d4, x9
	movi.2d	v5, #0xffffffffffffffff
	fneg.2d	v5, v5
	bif.16b	v4, v0, v5
	mov	x9, #12412
	movk	x9, #9130, lsl #16
	movk	x9, #46978, lsl #32
	movk	x9, #16393, lsl #48
	fmov	d6, x9
	fcmp	d1, d6
	fcsel	d3, d4, d3, eq
	mov	x9, #33295
	movk	x9, #43063, lsl #16
	movk	x9, #59514, lsl #32
	movk	x9, #16381, lsl #48
	fmov	d4, x9
	bit.16b	v0, v4, v5
	mov	x9, #56831
	movk	x9, #64649, lsl #16
	movk	x9, #8235, lsl #32
	movk	x9, #16410, lsl #48
	fmov	d4, x9
	fcmp	d1, d4
	fcsel	d4, d0, d3, eq
	b	LBB38_10
LBB38_8:
	fsub	d2, d6, d5
	b	LBB38_10
LBB38_9:
	mov.16b	v4, v3
LBB38_10:
	fmov	x9, d4
	add	w8, w8, #2731
	add	x8, x9, x8, lsl #52
	lsl	x10, x9, #30
	orr	x10, x10, x10, lsr #63
	mov	w11, #1073741824
	cmp	x10, x11
	b.hi	LBB38_13
	and	x9, x9, #0xffffffffffff0000
	add	x9, x9, #1312, lsl #12
	fmov	d0, x9
	fsub	d0, d0, d4
	fabd	d0, d0, d2
	mov	x9, #4336966441157787648
	fmov	d2, x9
	fcmp	d0, d2
	fmov	d0, #1.00000000
	fccmp	d1, d0, #4, pl
	b.ne	LBB38_13
	add	x8, x8, #8, lsl #12
	and	x8, x8, #0xffffffffffff0000
LBB38_13:
	fmov	d0, x8
LBB38_14:
	add	sp, sp, #48
	.cfi_def_cfa_offset 0
	ret
LBB38_15:
	.cfi_restore_state
	fadd	d0, d0, d0
	b	LBB38_14
	.loh AdrpLdr	Lloh346, Lloh347
	.loh AdrpAdrp	Lloh344, Lloh346
	.loh AdrpLdr	Lloh344, Lloh345
	.loh AdrpAdrp	Lloh342, Lloh344
	.loh AdrpLdr	Lloh342, Lloh343
	.loh AdrpAdd	Lloh348, Lloh349
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4ceil4ceil
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math4ceil4ceil:
	.cfi_startproc
	fmov	x8, d0
	ubfx	x9, x8, #52, #11
	cmp	w9, #1074
	b.hi	LBB39_4
	cmp	w9, #1022
	b.ls	LBB39_5
	sub	w9, w9, #1023
	mov	x11, #4503599627370495
	lsr	x10, x11, x9
	tst	x10, x8
	b.eq	LBB39_4
	cmp	x8, #0
	csel	x10, xzr, x10, mi
	mov	x11, #-4503599627370496
	asr	x9, x11, x9
	add	x8, x10, x8
	and	x8, x8, x9
	fmov	d0, x8
LBB39_4:
	ret
LBB39_5:
	mov	x9, #-9223372036854775808
	fmov	d1, x9
	cmp	x8, #0
	fmov	d2, #1.00000000
	fcsel	d0, d0, d2, eq
	tst	x8, #0x8000000000000000
	fcsel	d0, d1, d0, ne
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4ceil5ceilf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math4ceil5ceilf:
	.cfi_startproc
	fmov	w8, s0
	ubfx	w9, w8, #23, #8
	cmp	w9, #149
	b.hi	LBB40_4
	cmp	w9, #126
	b.ls	LBB40_5
	sub	w9, w9, #127
	mov	w10, #8388607
	lsr	w10, w10, w9
	tst	w10, w8
	b.eq	LBB40_4
	cmp	w8, #0
	csel	w10, wzr, w10, mi
	mov	w11, #-8388608
	asr	w9, w11, w9
	add	w8, w10, w8
	and	w8, w8, w9
	fmov	s0, w8
LBB40_4:
	ret
LBB40_5:
	movi.2s	v1, #128, lsl #24
	cmp	w8, #0
	fmov	s2, #1.00000000
	fcsel	s0, s0, s2, eq
	tst	w8, #0x80000000
	fcsel	s0, s1, s0, ne
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4cosf4cosf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math4cosf4cosf:
	.cfi_startproc
	sub	sp, sp, #48
	.cfi_def_cfa_offset 48
	stp	x20, x19, [sp, #16]
	stp	x29, x30, [sp, #32]
	add	x29, sp, #32
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_remember_state
	fcvt	d1, s0
	fmov	w19, s0
	and	w8, w19, #0x7fffffff
	mov	w9, #4059
	movk	w9, #16201, lsl #16
	cmp	w8, w9
	b.hs	LBB41_3
	lsr	w8, w8, #23
	cmp	w8, #115
	b.hs	LBB41_7
	mov	w8, #2071986176
	fmov	s1, w8
	fadd	s0, s0, s1
	str	s0, [sp, #8]
	ldr	s0, [sp, #8]
	fmov	s0, #1.00000000
	b	LBB41_22
LBB41_3:
	mov	w9, #21458
	movk	w9, #16507, lsl #16
	cmp	w8, w9
	b.hs	LBB41_8
	mov	w9, #52195
	movk	w9, #16406, lsl #16
	cmp	w8, w9
	b.ls	LBB41_13
	cmn	w19, #1
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #16393, lsl #48
	fmov	d0, x8
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #49161, lsl #48
	fmov	d2, x8
	fcsel	d0, d2, d0, gt
	fadd	d0, d0, d1
LBB41_6:
	fmul	d0, d0, d0
	fmul	d1, d0, d0
	mov	x8, #20585
	movk	x8, #57582, lsl #16
	movk	x8, #37698, lsl #32
	movk	x8, #16121, lsl #48
	fmov	d2, x8
	fmul	d2, d0, d2
	mov	x8, #7719
	movk	x8, #59407, lsl #16
	movk	x8, #49287, lsl #32
	movk	x8, #48982, lsl #48
	fmov	d3, x8
	fadd	d2, d2, d3
	mov	x8, #-41343
	movk	x8, #64780, lsl #16
	movk	x8, #49119, lsl #48
	fmov	d3, x8
	fmul	d3, d0, d3
	fmov	d4, #1.00000000
	fadd	d3, d3, d4
	mov	x8, #14914
	movk	x8, #57605, lsl #16
	movk	x8, #21843, lsl #32
	movk	x8, #16293, lsl #48
	fmov	d4, x8
	fmul	d4, d1, d4
	fadd	d3, d3, d4
	fmul	d0, d0, d1
	fmul	d0, d0, d2
	fadd	d0, d3, d0
	fcvt	s0, d0
	fneg	s0, s0
	b	LBB41_22
LBB41_7:
	fmul	d0, d1, d1
	b	LBB41_12
LBB41_8:
	mov	w9, #12758
	movk	w9, #16610, lsl #16
	cmp	w8, w9
	b.hs	LBB41_15
	mov	w9, #60895
	movk	w9, #16559, lsl #16
	cmp	w8, w9
	b.ls	LBB41_17
	cmn	w19, #1
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #16409, lsl #48
	fmov	d0, x8
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #49177, lsl #48
	fmov	d2, x8
	fcsel	d0, d2, d0, gt
	fadd	d0, d0, d1
LBB41_11:
	fmul	d0, d0, d0
LBB41_12:
	fmul	d1, d0, d0
	mov	x8, #20585
	movk	x8, #57582, lsl #16
	movk	x8, #37698, lsl #32
	movk	x8, #16121, lsl #48
	fmov	d2, x8
	fmul	d2, d0, d2
	mov	x8, #7719
	movk	x8, #59407, lsl #16
	movk	x8, #49287, lsl #32
	movk	x8, #48982, lsl #48
	fmov	d3, x8
	fadd	d2, d2, d3
	mov	x8, #-41343
	movk	x8, #64780, lsl #16
	movk	x8, #49119, lsl #48
	fmov	d3, x8
	fmul	d3, d0, d3
	fmov	d4, #1.00000000
	fadd	d3, d3, d4
	mov	x8, #14914
	movk	x8, #57605, lsl #16
	movk	x8, #21843, lsl #32
	movk	x8, #16293, lsl #48
	fmov	d4, x8
	fmul	d4, d1, d4
	fadd	d3, d3, d4
	fmul	d0, d0, d1
	fmul	d0, d0, d2
	fadd	d0, d3, d0
	b	LBB41_21
LBB41_13:
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #16377, lsl #48
	tbnz	w19, #31, LBB41_18
LBB41_14:
	fmov	d0, x8
	fsub	d0, d0, d1
	b	LBB41_19
LBB41_15:
	lsr	w9, w8, #23
	cmp	w9, #254
	b.ls	LBB41_23
	fsub	s0, s0, s0
	b	LBB41_22
LBB41_17:
	mov	x8, #8658
	movk	x8, #32563, lsl #16
	movk	x8, #55676, lsl #32
	movk	x8, #49170, lsl #48
	tbnz	w19, #31, LBB41_14
LBB41_18:
	fmov	d0, x8
	fadd	d0, d1, d0
LBB41_19:
	fmul	d1, d0, d0
	fmul	d2, d1, d1
	mov	x8, #18087
	movk	x8, #35899, lsl #16
	movk	x8, #52615, lsl #32
	movk	x8, #16070, lsl #48
	fmov	d3, x8
	fmul	d3, d1, d3
	mov	x8, #59252
	movk	x8, #58058, lsl #16
	movk	x8, #249, lsl #32
	movk	x8, #48938, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d4, d0, d1
	mov	x8, #64434
	movk	x8, #35182, lsl #16
	movk	x8, #4368, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d5, x8
	fmul	d1, d1, d5
	mov	x8, #44151
	movk	x8, #21707, lsl #16
	movk	x8, #21845, lsl #32
	movk	x8, #49093, lsl #48
	fmov	d5, x8
	fadd	d1, d1, d5
	fmul	d1, d4, d1
	fadd	d0, d0, d1
LBB41_20:
	fmul	d1, d4, d2
	fmul	d1, d1, d3
	fadd	d0, d1, d0
LBB41_21:
	fcvt	s0, d0
LBB41_22:
	.cfi_def_cfa wsp, 48
	ldp	x29, x30, [sp, #32]
	ldp	x20, x19, [sp, #16]
	add	sp, sp, #48
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	ret
LBB41_23:
	.cfi_restore_state
	str	xzr, [sp, #8]
	mov	w10, #4058
	movk	w10, #19913, lsl #16
	cmp	w8, w10
	b.hi	LBB41_25
	mov	x8, #51331
	movk	x8, #28105, lsl #16
	movk	x8, #24368, lsl #32
	movk	x8, #16356, lsl #48
	fmov	d0, x8
	fmul	d0, d1, d0
	mov	x8, #4843621399236968448
	fmov	d2, x8
	fadd	d0, d0, d2
	mov	x8, #-4379750637617807360
	fmov	d2, x8
	fadd	d0, d0, d2
	fcvtzs	w0, d0
	mov	x8, #1342177280
	movk	x8, #8699, lsl #32
	movk	x8, #49145, lsl #48
	fmov	d2, x8
	fmul	d2, d0, d2
	fadd	d1, d1, d2
	mov	x8, #25187
	movk	x8, #24858, lsl #16
	movk	x8, #4276, lsl #32
	movk	x8, #48721, lsl #48
	fmov	d2, x8
	fmul	d0, d0, d2
	fadd	d0, d1, d0
	b	LBB41_28
LBB41_25:
	sub	w3, w9, #150
	sub	w8, w8, w3, lsl #23
	fmov	s0, w8
	fcvt	d0, s0
	str	d0, [sp]
	mov	x0, sp
	add	x2, sp, #8
	mov	w1, #1
	mov	x4, #0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math14rem_pio2_large14rem_pio2_large
	tbnz	w19, #31, LBB41_27
	ldr	d0, [sp, #8]
	b	LBB41_28
LBB41_27:
	neg	w0, w0
	ldr	d0, [sp, #8]
	fneg	d0, d0
LBB41_28:
	and	w8, w0, #0x3
	cmp	w8, #1
	b.gt	LBB41_31
	cbz	w8, LBB41_11
	fmul	d1, d0, d0
	fmul	d2, d1, d1
	mov	x8, #18087
	movk	x8, #35899, lsl #16
	movk	x8, #52615, lsl #32
	movk	x8, #16070, lsl #48
	fmov	d3, x8
	fmul	d3, d1, d3
	mov	x8, #59252
	movk	x8, #58058, lsl #16
	movk	x8, #249, lsl #32
	movk	x8, #48938, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fnmul	d4, d0, d1
	mov	x8, #64434
	movk	x8, #35182, lsl #16
	movk	x8, #4368, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d5, x8
	fmul	d1, d1, d5
	mov	x8, #44151
	movk	x8, #21707, lsl #16
	movk	x8, #21845, lsl #32
	movk	x8, #49093, lsl #48
	fmov	d5, x8
	fadd	d1, d1, d5
	fmul	d1, d4, d1
	fsub	d0, d1, d0
	b	LBB41_20
LBB41_31:
	cmp	w8, #2
	b.eq	LBB41_6
	b	LBB41_19
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4cosh4cosh
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math4cosh4cosh:
	.cfi_startproc
	sub	sp, sp, #32
	.cfi_def_cfa_offset 32
	stp	x29, x30, [sp, #16]
	add	x29, sp, #16
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	fabs	d0, d0
	fmov	x8, d0
	mov	x9, #50861002719232
	movk	x9, #16358, lsl #48
	cmp	x8, x9
	b.hs	LBB42_3
	lsr	x8, x8, #52
	cmp	x8, #997
	b.hs	LBB42_5
	mov	x8, #5147614374084476928
	fmov	d1, x8
	fadd	d0, d0, d1
	str	d0, [sp, #8]
	ldr	d0, [sp, #8]
	fmov	d0, #1.00000000
	b	LBB42_8
LBB42_3:
	mov	x9, #50861002719232
	movk	x9, #16518, lsl #48
	cmp	x8, x9
	b.hs	LBB42_6
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3exp3exp
	fmov	d1, #1.00000000
	fdiv	d1, d1, d0
	fadd	d0, d0, d1
	fmov	d1, #0.50000000
	b	LBB42_7
LBB42_5:
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5expm15expm1
	fmul	d1, d0, d0
	fmov	d2, #1.00000000
	fadd	d0, d0, d2
	fadd	d0, d0, d0
	fdiv	d0, d1, d0
	fadd	d0, d0, d2
	b	LBB42_8
LBB42_6:
	mov	x8, #56715
	movk	x8, #5402, lsl #16
	movk	x8, #8294, lsl #32
	movk	x8, #49302, lsl #48
	fmov	d1, x8
	fadd	d0, d0, d1
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3exp3exp
	mov	x8, #9205357638345293824
	fmov	d1, x8
	fmul	d0, d0, d1
LBB42_7:
	fmul	d0, d0, d1
LBB42_8:
	.cfi_def_cfa wsp, 32
	ldp	x29, x30, [sp, #16]
	add	sp, sp, #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
	.cfi_endproc

	.section	__TEXT,__literal8,8byte_literals
	.p2align	3, 0x0
lCPI43_0:
	.long	0xbce9528f
	.long	0x3d852a63
lCPI43_1:
	.long	0xbea66beb
	.long	0x3ecbbbce
lCPI43_2:
	.long	0x3e0375d4
	.long	0x3f800000
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4erff4erff
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math4erff4erff:
	.cfi_startproc
	stp	x20, x19, [sp, #-32]!
	.cfi_def_cfa_offset 32
	stp	x29, x30, [sp, #16]
	add	x29, sp, #16
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_remember_state
	fmov	w19, s0
	ubfx	w8, w19, #23, #8
	cmp	w8, #254
	b.ls	LBB43_3
	lsr	w8, w19, #30
	and	w8, w8, #0x2
	ucvtf	s1, w8
	fmov	s2, #1.00000000
	fsub	s1, s2, s1
	fdiv	s0, s2, s0
LBB43_2:
	fadd	s0, s0, s1
	b	LBB43_9
LBB43_3:
	and	w8, w19, #0x7fffffff
	lsr	w9, w8, #19
	cmp	w9, #2027
	b.hs	LBB43_6
	lsr	w8, w8, #23
	cmp	w8, #99
	b.hs	LBB43_10
	fmov	s1, #8.00000000
	fmul	s1, s0, s1
	mov	w8, #30164
	movk	w8, #16259, lsl #16
	fmov	s2, w8
	fmul	s0, s0, s2
	fadd	s0, s1, s0
	fmov	s1, #0.12500000
	fmul	s0, s0, s1
	b	LBB43_9
LBB43_6:
	fmov	s1, #1.00000000
	lsr	w8, w8, #22
	cmp	w8, #259
	b.hs	LBB43_8
	and	w0, w19, #0x7fffffff
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4erff5erfc2
	fmov	s1, #1.00000000
	fsub	s1, s1, s0
LBB43_8:
	fneg	s0, s1
	cmp	w19, #0
	fcsel	s0, s0, s1, mi
LBB43_9:
	.cfi_def_cfa wsp, 32
	ldp	x29, x30, [sp, #16]
	ldp	x20, x19, [sp], #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	ret
LBB43_10:
	.cfi_restore_state
	fmul	s1, s0, s0
	mov	w8, #57882
	movk	w8, #46724, lsl #16
	fmov	s2, w8
	fmul	s2, s1, s2
	mov	w8, #61001
	movk	w8, #14602, lsl #16
	fmov	s3, w8
	fadd	s2, s2, s3
	mov	w8, #22193
	movk	w8, #14279, lsl #16
	fmov	s3, w8
	dup.2s	v3, v3[0]
	mov.s	v3[1], v2[0]
	fmul.2s	v2, v3, v1[0]
	mov	w8, #5257
	movk	w8, #48061, lsl #16
	fmov	s3, w8
	fsub.2s	v3, v3, v2
	mov	w8, #33046
	movk	w8, #15270, lsl #16
	dup.2s	v4, w8
	fadd.2s	v2, v2, v4
	mov.s	v3[1], v2[1]
	fmul.2s	v2, v3, v1[0]
Lloh350:
	adrp	x8, lCPI43_0@PAGE
Lloh351:
	ldr	d3, [x8, lCPI43_0@PAGEOFF]
	fadd.2s	v2, v2, v3
	fmul.2s	v2, v2, v1[0]
Lloh352:
	adrp	x8, lCPI43_1@PAGE
Lloh353:
	ldr	d3, [x8, lCPI43_1@PAGEOFF]
	fadd.2s	v2, v2, v3
	fmul.2s	v1, v2, v1[0]
Lloh354:
	adrp	x8, lCPI43_2@PAGE
Lloh355:
	ldr	d2, [x8, lCPI43_2@PAGEOFF]
	fadd.2s	v1, v1, v2
	dup.2s	v2, v1[1]
	fdiv.2s	v1, v1, v2
	fmul	s1, s0, s1
	b	LBB43_2
	.loh AdrpLdr	Lloh354, Lloh355
	.loh AdrpAdrp	Lloh352, Lloh354
	.loh AdrpLdr	Lloh352, Lloh353
	.loh AdrpAdrp	Lloh350, Lloh352
	.loh AdrpLdr	Lloh350, Lloh351
	.cfi_endproc

	.section	__TEXT,__literal8,8byte_literals
	.p2align	3, 0x0
lCPI44_0:
	.long	0xbc21a093
	.long	0x42d9451f
lCPI44_1:
	.long	0x3b0df9c0
	.long	0x3c445aa3
lCPI44_2:
	.long	0xbde31cc2
	.long	0x3e013307
lCPI44_3:
	.long	0x3ea2fe54
	.long	0x3d931ae7
lCPI44_4:
	.long	0xbebe9208
	.long	0x3f0a5785
lCPI44_5:
	.long	0x3ed46805
	.long	0x3dd9f331
lCPI44_6:
	.long	0xbb1acdc6
	.long	0x3f800000
	.section	__TEXT,__text,regular,pure_instructions
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math4erff5erfc2:
	.cfi_startproc
	sub	sp, sp, #64
	.cfi_def_cfa_offset 64
	stp	d11, d10, [sp, #16]
	stp	d9, d8, [sp, #32]
	stp	x29, x30, [sp, #48]
	add	x29, sp, #48
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset b8, -24
	.cfi_offset b9, -32
	.cfi_offset b10, -40
	.cfi_offset b11, -48
	fabs	s9, s0
	lsr	w8, w0, #21
	cmp	w8, #509
	b.hs	LBB44_2
	fmov	s0, #-1.00000000
	fadd	s0, s9, s0
Lloh356:
	adrp	x8, lCPI44_1@PAGE
Lloh357:
	ldr	d1, [x8, lCPI44_1@PAGEOFF]
	fmul.2s	v1, v1, v0[0]
	mov	w8, #20915
	movk	w8, #15633, lsl #16
	fmov	s2, w8
	mov	w8, #28179
	movk	w8, #15455, lsl #16
	dup.2s	v3, w8
	fsub.2s	v2, v2, v1
	fadd.2s	v1, v1, v3
	mov.s	v2[1], v1[1]
	fmul.2s	v1, v2, v0[0]
Lloh358:
	adrp	x8, lCPI44_2@PAGE
Lloh359:
	ldr	d2, [x8, lCPI44_2@PAGEOFF]
	fadd.2s	v1, v1, v2
	fmul.2s	v1, v1, v0[0]
Lloh360:
	adrp	x8, lCPI44_3@PAGE
Lloh361:
	ldr	d2, [x8, lCPI44_3@PAGEOFF]
	fadd.2s	v1, v1, v2
	fmul.2s	v1, v1, v0[0]
Lloh362:
	adrp	x8, lCPI44_4@PAGE
Lloh363:
	ldr	d2, [x8, lCPI44_4@PAGEOFF]
	fadd.2s	v1, v1, v2
	fmul.2s	v1, v1, v0[0]
Lloh364:
	adrp	x8, lCPI44_5@PAGE
Lloh365:
	ldr	d2, [x8, lCPI44_5@PAGEOFF]
	fadd.2s	v1, v1, v2
	fmul.2s	v0, v1, v0[0]
Lloh366:
	adrp	x8, lCPI44_6@PAGE
Lloh367:
	ldr	d1, [x8, lCPI44_6@PAGEOFF]
	fadd.2s	v0, v0, v1
	dup.2s	v1, v0[1]
	fdiv.2s	v0, v0, v1
	mov	w8, #42964
	movk	w8, #15902, lsl #16
	fmov	s1, w8
	fsub	s0, s1, s0
	b	LBB44_6
LBB44_2:
	fmov	w8, s9
	fmul	s0, s0, s0
	fmov	s1, #1.00000000
	mov	w9, #56173
	movk	w9, #16438, lsl #16
	fdiv	s0, s1, s0
	cmp	w0, w9
	b.hs	LBB44_4
	mov	w9, #1918
	movk	w9, #49437, lsl #16
	fmov	s1, w9
	fmul	s1, s0, s1
	mov	w9, #37675
	movk	w9, #49826, lsl #16
	fmov	s2, w9
	fadd	s1, s1, s2
	fmul	s1, s0, s1
	mov	w9, #39655
	movk	w9, #49976, lsl #16
	fmov	s2, w9
	fadd	s1, s1, s2
	fmul	s1, s0, s1
	mov	w9, #25996
	movk	w9, #49954, lsl #16
	fmov	s2, w9
	fadd	s1, s1, s2
	fmul	s1, s0, s1
	mov	w9, #32855
	movk	w9, #49785, lsl #16
	fmov	s2, w9
	fadd	s1, s1, s2
	fmul	s1, s0, s1
	mov	w9, #61474
	movk	w9, #49448, lsl #16
	fmov	s2, w9
	fadd	s1, s1, s2
	mov	w9, #32663
	movk	w9, #15735, lsl #16
	fmov	s2, w9
	dup.2s	v2, v2[0]
	mov.s	v2[0], v1[0]
	fmul.2s	v1, v2, v0[0]
	mov	w9, #41143
	movk	w9, #48945, lsl #16
	fmov	s2, w9
	mov	w9, #16252
	movk	w9, #16594, lsl #16
	dup.2s	v3, w9
	fadd.2s	v2, v1, v2
	fsub.2s	v1, v3, v1
	mov.s	v2[1], v1[1]
	fmul.2s	v2, v2, v0[0]
Lloh368:
	adrp	x9, lCPI44_0@PAGE
Lloh369:
	ldr	d3, [x9, lCPI44_0@PAGEOFF]
	mov	w9, #13774
	movk	w9, #16797, lsl #16
	fmov	s1, w9
	fadd.2s	v6, v2, v3
	mov	w9, #43107
	movk	w9, #17161, lsl #16
	fmov	s2, w9
	mov	w9, #18543
	movk	w9, #17369, lsl #16
	fmov	s3, w9
	mov	w9, #22729
	movk	w9, #17441, lsl #16
	fmov	s4, w9
	mov	w9, #33035
	movk	w9, #17366, lsl #16
	b	LBB44_5
LBB44_4:
	mov	w9, #49781
	movk	w9, #50161, lsl #16
	fmov	s1, w9
	fmul	s1, s0, s1
	mov	w9, #8971
	movk	w9, #50304, lsl #16
	fmov	s2, w9
	fadd	s1, s1, s2
	fmul	s1, s0, s1
	mov	w9, #25665
	movk	w9, #50207, lsl #16
	fmov	s2, w9
	fadd	s1, s1, s2
	fmul	s1, s0, s1
	mov	w9, #41706
	movk	w9, #49952, lsl #16
	fmov	s2, w9
	fadd	s1, s1, s2
	fmul	s1, s0, s1
	mov	w9, #4171
	movk	w9, #49550, lsl #16
	fmov	s2, w9
	fadd	s1, s1, s2
	fmul	s1, s0, s1
	mov	w9, #40404
	movk	w9, #48972, lsl #16
	fmov	s2, w9
	fadd	s1, s1, s2
	mov	w9, #34578
	movk	w9, #16819, lsl #16
	fmov	s2, w9
	dup.2s	v2, v2[0]
	mov.s	v2[0], v1[0]
	fmul.2s	v1, v2, v0[0]
	mov	w9, #41106
	movk	w9, #48161, lsl #16
	fmov	s2, w9
	fadd.2s	v6, v1, v2
	mov	w9, #17319
	movk	w9, #17389, lsl #16
	dup.2s	v2, w9
	fsub.2s	v2, v2, v1
	mov	w9, #46169
	movk	w9, #16882, lsl #16
	fmov	s1, w9
	mov.s	v6[1], v2[1]
	mov	w9, #58737
	movk	w9, #17314, lsl #16
	fmov	s2, w9
	mov	w9, #5977
	movk	w9, #17600, lsl #16
	fmov	s3, w9
	mov	w9, #64955
	movk	w9, #17735, lsl #16
	fmov	s4, w9
	mov	w9, #37070
	movk	w9, #17695, lsl #16
LBB44_5:
	fmov	s5, w9
	str	q6, [sp]
	fmul.s	s6, s0, v6[1]
	fadd	s5, s6, s5
	fmul	s5, s0, s5
	fadd	s4, s5, s4
	fmul	s4, s0, s4
	fadd	s3, s4, s3
	fmul	s3, s0, s3
	fadd	s2, s3, s2
	fmul	s2, s0, s2
	fadd	s1, s2, s1
	fmul	s0, s0, s1
	fmov	s1, #1.00000000
	fadd	s10, s0, s1
	and	w8, w8, #0x7fffe000
	fmov	s11, w8
	fmul	s0, s11, s11
	fmov	s1, #-0.56250000
	fsub	s0, s1, s0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4expf4expf
	mov.16b	v8, v0
	fsub	s0, s11, s9
	fadd	s1, s9, s11
	fmul	s0, s0, s1
	ldr	q1, [sp]
	fdiv	s1, s1, s10
	fadd	s0, s0, s1
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4expf4expf
	fmul	s0, s8, s0
	fdiv	s0, s0, s9
LBB44_6:
	.cfi_def_cfa wsp, 64
	ldp	x29, x30, [sp, #48]
	ldp	d9, d8, [sp, #32]
	ldp	d11, d10, [sp, #16]
	add	sp, sp, #64
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore b8
	.cfi_restore b9
	.cfi_restore b10
	.cfi_restore b11
	ret
	.loh AdrpLdr	Lloh366, Lloh367
	.loh AdrpAdrp	Lloh364, Lloh366
	.loh AdrpLdr	Lloh364, Lloh365
	.loh AdrpAdrp	Lloh362, Lloh364
	.loh AdrpLdr	Lloh362, Lloh363
	.loh AdrpAdrp	Lloh360, Lloh362
	.loh AdrpLdr	Lloh360, Lloh361
	.loh AdrpAdrp	Lloh358, Lloh360
	.loh AdrpLdr	Lloh358, Lloh359
	.loh AdrpLdr	Lloh356, Lloh357
	.loh AdrpLdr	Lloh368, Lloh369
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4erff5erfcf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math4erff5erfcf:
	.cfi_startproc
	stp	x20, x19, [sp, #-32]!
	.cfi_def_cfa_offset 32
	stp	x29, x30, [sp, #16]
	add	x29, sp, #16
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_remember_state
	fmov	w19, s0
	ubfx	w8, w19, #23, #8
	cmp	w8, #254
	b.ls	LBB45_3
	lsr	w8, w19, #30
	and	w8, w8, #0x2
	ucvtf	s1, w8
	fmov	s2, #1.00000000
	fdiv	s0, s2, s0
	fadd	s0, s0, s1
LBB45_2:
	.cfi_def_cfa wsp, 32
	ldp	x29, x30, [sp, #16]
	ldp	x20, x19, [sp], #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	ret
LBB45_3:
	.cfi_restore_state
	and	w8, w19, #0x7fffffff
	lsr	w9, w8, #19
	cmp	w9, #2027
	b.hs	LBB45_6
	lsr	w8, w8, #23
	cmp	w8, #71
	b.hs	LBB45_8
	fmov	s1, #1.00000000
	fsub	s0, s1, s0
	b	LBB45_2
LBB45_6:
	lsr	w8, w8, #21
	cmp	w8, #527
	b.hs	LBB45_11
	and	w0, w19, #0x7fffffff
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4erff5erfc2
	fmov	s1, #2.00000000
	fsub	s1, s1, s0
	cmn	w19, #1
	fcsel	s0, s0, s1, gt
	b	LBB45_2
LBB45_8:
	fmul	s1, s0, s0
	mov	w9, #22193
	movk	w9, #47047, lsl #16
	fmov	s2, w9
	fmul	s2, s1, s2
	mov	w9, #5257
	movk	w9, #48061, lsl #16
	fmov	s3, w9
	fadd	s2, s2, s3
	fmul	s2, s1, s2
	mov	w9, #21135
	movk	w9, #48361, lsl #16
	fmov	s3, w9
	fadd	s2, s2, s3
	fmul	s2, s1, s2
	mov	w9, #27627
	movk	w9, #48806, lsl #16
	fmov	s3, w9
	fadd	s2, s2, s3
	fmul	s2, s1, s2
	mov	w9, #30164
	movk	w9, #15875, lsl #16
	fmov	s3, w9
	fadd	s2, s2, s3
	mov	w9, #57882
	movk	w9, #46724, lsl #16
	fmov	s3, w9
	fmul	s3, s1, s3
	mov	w9, #61001
	movk	w9, #14602, lsl #16
	fmov	s4, w9
	fadd	s3, s3, s4
	fmul	s3, s1, s3
	mov	w9, #33046
	movk	w9, #15270, lsl #16
	fmov	s4, w9
	fadd	s3, s3, s4
	fmul	s3, s1, s3
	mov	w9, #10851
	movk	w9, #15749, lsl #16
	fmov	s4, w9
	fadd	s3, s3, s4
	fmul	s3, s1, s3
	mov	w9, #48078
	movk	w9, #16075, lsl #16
	fmov	s4, w9
	fadd	s3, s3, s4
	fmul	s3, s1, s3
	fmov	s1, #1.00000000
	fadd	s3, s3, s1
	fdiv	s2, s2, s3
	tbnz	w19, #31, LBB45_12
	cmp	w8, #125
	b.lo	LBB45_12
	fmov	s1, #-0.50000000
	fadd	s1, s0, s1
	fmul	s0, s0, s2
	fadd	s0, s1, s0
	fmov	s1, #0.50000000
	fsub	s0, s1, s0
	b	LBB45_2
LBB45_11:
	cmn	w19, #1
	fmov	s0, #2.00000000
	movi.2d	v1, #0000000000000000
	fcsel	s0, s1, s0, gt
	b	LBB45_2
LBB45_12:
	fmul	s2, s0, s2
	fadd	s0, s0, s2
	fsub	s0, s1, s0
	b	LBB45_2
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4exp24exp2
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math4exp24exp2:
	.cfi_startproc
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	.cfi_remember_state
	mov	w10, #61439
	movk	w10, #16527, lsl #16
	fmov	x8, d0
	ubfx	x9, x8, #32, #31
	cmp	x9, x10
	b.ls	LBB46_4
	tbnz	x8, #63, LBB46_6
	add	x10, x10, #1, lsl #12
	cmp	x9, x10
	b.ls	LBB46_6
	mov	x8, #9214364837600034816
	b	LBB46_21
LBB46_4:
	lsr	x8, x9, #20
	cmp	x8, #969
	b.hs	LBB46_9
	fmov	d1, #1.00000000
	fadd	d0, d0, d1
	b	LBB46_22
LBB46_6:
	lsr	x9, x9, #20
	cmp	x9, #2046
	b.ls	LBB46_8
	fmov	d1, #-1.00000000
	fdiv	d0, d1, d0
	b	LBB46_22
LBB46_8:
	tbnz	x8, #63, LBB46_16
LBB46_9:
	mov	x8, #4807592602218004480
	fmov	d1, x8
	fadd	d1, d0, d1
	mov	x8, #-4415779434636771328
	fmov	d2, x8
	fadd	d2, d1, d2
	fmov	x8, d1
	add	w9, w8, #128
	asr	w8, w9, #8
	fsub	d0, d0, d2
	ubfiz	w9, w9, #1, #8
Lloh370:
	adrp	x10, __RNvNtNtCsdtCTLUYU3dx_4libm4math4exp23TBL@PAGE
Lloh371:
	add	x10, x10, __RNvNtNtCsdtCTLUYU3dx_4libm4math4exp23TBL@PAGEOFF
	add	x9, x10, w9, uxtw #3
	ldp	d1, d2, [x9]
	fsub	d0, d0, d2
	fmul	d2, d1, d0
	mov	x9, #23668
	movk	x9, #903, lsl #16
	movk	x9, #55424, lsl #32
	movk	x9, #16213, lsl #48
	fmov	d3, x9
	fmul	d3, d0, d3
	mov	x9, #1024
	movk	x9, #35063, lsl #16
	movk	x9, #45739, lsl #32
	movk	x9, #16259, lsl #48
	fmov	d4, x9
	fadd	d3, d3, d4
	fmul	d3, d0, d3
	mov	x9, #41126
	movk	x9, #55044, lsl #16
	movk	x9, #27400, lsl #32
	movk	x9, #16300, lsl #48
	fmov	d4, x9
	fadd	d3, d3, d4
	fmul	d3, d0, d3
	mov	x9, #50549
	movk	x9, #65410, lsl #16
	movk	x9, #49085, lsl #32
	movk	x9, #16334, lsl #48
	fmov	d4, x9
	fadd	d3, d3, d4
	fmul	d0, d0, d3
	mov	x9, #14831
	movk	x9, #65274, lsl #16
	movk	x9, #11842, lsl #32
	movk	x9, #16358, lsl #48
	fmov	d3, x9
	fadd	d0, d0, d3
	fmul	d0, d2, d0
	fadd	d0, d1, d0
	subs	w9, w8, #1023
	b.le	LBB46_12
	mov	x10, #9214364837600034816
	fmov	d1, x10
	fmul	d0, d0, d1
	cmp	w8, #2046
	b.ls	LBB46_15
	mov	x9, #9214364837600034816
	fmov	d1, x9
	fmul	d0, d0, d1
	mov	w9, #3069
	cmp	w8, #3069
	csel	w8, w8, w9, lo
	sub	w8, w8, #2046
	b	LBB46_20
LBB46_12:
	cmn	w8, #1022
	b.ge	LBB46_20
	mov	x9, #243194379878006784
	fmov	d1, x9
	fmul	d0, d0, d1
	cmn	w8, #1992
	b.hi	LBB46_19
	fmov	d1, x9
	fmul	d0, d0, d1
	cmn	w8, #2960
	mov	w9, #-2960
	csel	w8, w8, w9, hi
	add	w8, w8, #1938
	b	LBB46_20
LBB46_15:
	mov	x8, x9
	b	LBB46_20
LBB46_16:
	mov	x8, #224300372066304
	movk	x8, #49296, lsl #48
	fmov	d1, x8
	fcmp	d0, d1
	b.ls	LBB46_23
	mov	x8, #-4382002437431492608
	fmov	d1, x8
	fadd	d1, d0, d1
	mov	x8, #4841369599423283200
	fmov	d2, x8
	fadd	d1, d1, d2
	fcmp	d1, d0
	b.eq	LBB46_9
	mov	x8, #-5287225962532962304
	fmov	d1, x8
	fdiv	d1, d1, d0
	fcvt	s1, d1
	str	s1, [sp, #12]
	ldr	s1, [sp, #12]
	b	LBB46_9
LBB46_19:
	add	w8, w8, #969
LBB46_20:
	add	w8, w8, #1023
	lsl	x8, x8, #52
LBB46_21:
	fmov	d1, x8
	fmul	d0, d0, d1
LBB46_22:
	add	sp, sp, #16
	.cfi_def_cfa_offset 0
	ret
LBB46_23:
	.cfi_restore_state
	mov	x8, #-5287225962532962304
	fmov	d1, x8
	fdiv	d0, d1, d0
	fcvt	s0, d0
	str	s0, [sp, #12]
	ldr	s0, [sp, #12]
	movi.2d	v0, #0000000000000000
	b	LBB46_22
	.loh AdrpAdd	Lloh370, Lloh371
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4expf4expf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math4expf4expf:
	.cfi_startproc
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	fmov	w10, s0
	lsr	w8, w10, #31
	and	w9, w10, #0x7fffffff
	mov	w11, #44112
	movk	w11, #17070, lsl #16
	cmp	w9, w11
	b.lo	LBB47_3
	mov	w11, #2139095040
	cmp	w9, w11
	b.ls	LBB47_6
	mov.16b	v1, v0
	b	LBB47_27
LBB47_3:
	mov	w10, #29208
	movk	w10, #16049, lsl #16
	cmp	w9, w10
	b.ls	LBB47_9
	mov	w10, #5522
	movk	w10, #16261, lsl #16
	cmp	w9, w10
	b.hi	LBB47_15
	eor	w9, w8, #0x1
	sub	w8, w9, w8
	b	LBB47_16
LBB47_6:
	mov	w11, #29207
	movk	w11, #17073, lsl #16
	cmp	w9, w11
	b.ls	LBB47_11
	tbnz	w10, #31, LBB47_14
	movi.2s	v1, #127, lsl #24
	fmul	s1, s0, s1
	b	LBB47_27
LBB47_9:
	mov	w8, #956301312
	cmp	w9, w8
	b.ls	LBB47_13
	mov	w8, #0
	movi.2d	v1, #0000000000000000
	mov.16b	v2, v0
	b	LBB47_17
LBB47_11:
	tbz	w10, #31, LBB47_15
	mov	w9, #-2139095040
	fmov	s1, w9
	fdiv	s1, s1, s0
	str	s1, [sp, #8]
	ldr	s1, [sp, #8]
	b	LBB47_15
LBB47_13:
	movi.2s	v1, #127, lsl #24
	fadd	s1, s0, s1
	str	s1, [sp, #12]
	ldr	s1, [sp, #12]
	fmov	s1, #1.00000000
	fadd	s1, s0, s1
	b	LBB47_27
LBB47_14:
	mov	w10, #-2139095040
	fmov	s1, w10
	fdiv	s1, s1, s0
	str	s1, [sp, #8]
	ldr	s1, [sp, #8]
	movi.2d	v1, #0000000000000000
	mov	w10, #61876
	movk	w10, #17103, lsl #16
	cmp	w9, w10
	b.hi	LBB47_27
LBB47_15:
	mov	w9, #43579
	movk	w9, #16312, lsl #16
	fmov	s1, w9
Lloh372:
	adrp	x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.87@PAGE
Lloh373:
	add	x9, x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.87@PAGEOFF
	fmul	s1, s0, s1
	ldr	s2, [x9, w8, uxtw #2]
	fadd	s1, s1, s2
	fcvtzs	w8, s1
LBB47_16:
	scvtf	s1, w8
	mov	w9, #29184
	movk	w9, #48945, lsl #16
	fmov	s2, w9
	fmul	s2, s1, s2
	fadd	s0, s0, s2
	mov	w9, #48782
	movk	w9, #13759, lsl #16
	fmov	s2, w9
	fmul	s1, s1, s2
	fsub	s2, s0, s1
LBB47_17:
	fmul	s3, s2, s2
	mov	w9, #21013
	movk	w9, #47925, lsl #16
	fmov	s4, w9
	fmul	s4, s3, s4
	mov	w9, #43663
	movk	w9, #15914, lsl #16
	fmov	s5, w9
	fadd	s4, s4, s5
	fmul	s3, s3, s4
	fsub	s3, s2, s3
	fmul	s2, s2, s3
	fmov	s4, #2.00000000
	fsub	s3, s4, s3
	fdiv	s2, s2, s3
	fsub	s1, s2, s1
	fadd	s0, s0, s1
	fmov	s1, #1.00000000
	fadd	s1, s0, s1
	cbz	w8, LBB47_27
	subs	w9, w8, #127
	b.le	LBB47_21
	movi.2s	v0, #127, lsl #24
	fmul	s1, s1, s0
	cmp	w8, #254
	b.ls	LBB47_24
	fmul	s1, s1, s0
	mov	w9, #381
	cmp	w8, #381
	csel	w8, w8, w9, lo
	sub	w8, w8, #254
	b	LBB47_26
LBB47_21:
	cmn	w8, #126
	b.ge	LBB47_26
	mov	w9, #209715200
	fmov	s0, w9
	fmul	s1, s1, s0
	cmn	w8, #229
	b.hi	LBB47_25
	fmov	s0, w9
	fmul	s1, s1, s0
	cmn	w8, #330
	mov	w9, #-330
	csel	w8, w8, w9, hi
	add	w8, w8, #204
	b	LBB47_26
LBB47_24:
	mov	x8, x9
	b	LBB47_26
LBB47_25:
	add	w8, w8, #102
LBB47_26:
	mov	w9, #1065353216
	add	w8, w9, w8, lsl #23
	and	w8, w8, #0x7f800000
	fmov	s0, w8
	fmul	s1, s1, s0
LBB47_27:
	mov.16b	v0, v1
	add	sp, sp, #16
	.cfi_def_cfa_offset 0
	ret
	.loh AdrpAdd	Lloh372, Lloh373
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4fmod4fmod
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math4fmod4fmod:
	.cfi_startproc
	stp	x26, x25, [sp, #-80]!
	.cfi_def_cfa_offset 80
	stp	x24, x23, [sp, #16]
	stp	x22, x21, [sp, #32]
	stp	x20, x19, [sp, #48]
	stp	x29, x30, [sp, #64]
	add	x29, sp, #64
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset w21, -40
	.cfi_offset w22, -48
	.cfi_offset w23, -56
	.cfi_offset w24, -64
	.cfi_offset w25, -72
	.cfi_offset w26, -80
	.cfi_remember_state
	fmov	x8, d0
	mov	x9, #9218868437227405312
	bic	x9, x9, x8
	cbz	x9, LBB48_10
	fmov	x9, d1
	neg	x10, x9
	and	x10, x10, #0x7ff0000000000000
	cbz	x10, LBB48_10
	and	x13, x8, #0x7fffffffffffffff
	and	x11, x9, #0x7fffffffffffffff
	cmp	x13, x11
	b.lo	LBB48_29
	mov	x9, #4503599627370496
	subs	x10, x13, x9
	csel	x10, xzr, x10, lo
	and	x15, x10, #0x7ff0000000000000
	subs	x9, x11, x9
	csel	x12, xzr, x9, lo
	and	x14, x12, #0x7ff0000000000000
	sub	x2, x11, x14
	lsl	x9, x2, #1
	sub	x23, x13, x15
	cmp	x23, x9
	b.lo	LBB48_6
	cmp	x11, x14
	b.eq	LBB48_35
	udiv	x11, x23, x2
	msub	x23, x11, x2, x23
LBB48_6:
	and	x21, x8, #0x8000000000000000
	lsr	x10, x10, #52
	lsr	x22, x12, #52
	sub	w8, w10, w22
	cmp	w8, #31
	b.hi	LBB48_11
	cmp	x10, x22
	b.eq	LBB48_9
LBB48_8:
	cmp	x23, x2
	csel	x9, xzr, x2, lo
	sub	x9, x23, x9
	lsl	x23, x9, #1
	subs	w8, w8, #1
	b.ne	LBB48_8
LBB48_9:
	cmp	x23, x2
	csel	x8, xzr, x2, lo
	sub	x0, x23, x8
	b	LBB48_26
LBB48_10:
	fmul	d0, d0, d1
	fdiv	d0, d0, d0
	b	LBB48_29
LBB48_11:
	cmp	w8, #64
	b.hs	LBB48_14
	mvn	w10, w8
	lsr	x11, x23, #1
	lsr	x10, x11, x10
	lsl	x11, x23, x8
	tst	x8, #0x40
	csel	x1, x11, x10, ne
	csel	x0, xzr, x11, ne
	cmp	x2, x1
	b.ls	LBB48_14
	mov	x3, #0
	bl	___umodti3
	b	LBB48_26
LBB48_14:
	lsr	x10, x2, #62
	cbnz	x10, LBB48_30
	cmp	x23, x9
	b.hs	LBB48_31
	sub	x9, x2, #1
	tst	x2, x9
	b.eq	LBB48_22
	clz	x9, x2
	sub	w24, w9, #2
	lsl	x25, x2, x24
	mov	x9, #2305843009213693952
	cmp	x25, x9
	b.ls	LBB48_32
	lsr	x9, x25, #62
	cbnz	x9, LBB48_33
	lsl	x19, x25, #1
	cmp	x23, x19
	b.hs	LBB48_34
	add	w26, w24, w8
	mov	x8, #-9223372036854775808
	sub	x20, x8, x19
	mov	x0, #0
	mov	x1, x20
	mov	x2, x19
	mov	x3, #0
	bl	___udivti3
	umulh	x8, x0, x19
	madd	x9, x1, x19, x8
	mul	x8, x0, x19
	negs	x8, x8
	sbc	x9, x20, x9
	lsl	x11, x23, #1
	umulh	x10, x0, x11
	madd	x12, x11, x1, x11
	add	x10, x10, x12
	mul	x12, x0, x11
	cmp	w26, #63
	b.lo	LBB48_24
LBB48_21:
	umulh	x11, x10, x8
	madd	x13, x10, x9, x11
	mul	x11, x10, x8
	add	x10, x13, x12, lsr #1
	sub	w26, w26, #63
	mov	x12, x11
	cmp	w26, #62
	b.hi	LBB48_21
	b	LBB48_25
LBB48_22:
	cmp	w8, #64
	b.hs	LBB48_28
	lsl	x8, x23, x8
	and	x0, x8, x9
	b	LBB48_26
LBB48_24:
	mov	x11, x12
LBB48_25:
	eor	w12, w26, #0x3f
	lsr	x12, x10, x12
	mvn	w13, w26
	lsr	x14, x11, #1
	lsr	x13, x14, x13
	mov	w14, w26
	lsl	x10, x10, x14
	orr	x10, x10, x13
	lsl	x11, x11, x14
	tst	x14, #0x40
	csel	x10, x11, x10, ne
	csel	x11, xzr, x11, ne
	and	x10, x10, #0x7fffffffffffffff
	umulh	x13, x8, x12
	madd	x9, x9, x12, x13
	mul	x8, x8, x12
	cmn	x8, x11
	adc	x8, x9, x10
	add	x8, x8, #2
	umulh	x8, x8, x19
	cmp	x25, x8
	csel	x9, xzr, x25, hi
	sub	x8, x8, x9
	lsr	x0, x8, x24
LBB48_26:
	cbz	x0, LBB48_28
	clz	x8, x0
	eor	w8, w8, #0x3f
	mov	w9, #52
	sub	w8, w9, w8
	cmp	w8, w22
	csel	w8, w8, w22, lo
	lsl	x9, x0, x8
	sub	w8, w22, w8
	add	x9, x9, x21
	add	x8, x9, x8, lsl #52
	fmov	d0, x8
	b	LBB48_29
LBB48_28:
	fmov	d0, x21
LBB48_29:
	.cfi_def_cfa wsp, 80
	ldp	x29, x30, [sp, #64]
	ldp	x20, x19, [sp, #48]
	ldp	x22, x21, [sp, #32]
	ldp	x24, x23, [sp, #16]
	ldp	x26, x25, [sp], #80
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	.cfi_restore w23
	.cfi_restore w24
	.cfi_restore w25
	.cfi_restore w26
	ret
LBB48_30:
	.cfi_restore_state
Lloh374:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.3@PAGE
Lloh375:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.3@PAGEOFF
Lloh376:
	adrp	x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.4@PAGE
Lloh377:
	add	x2, x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.4@PAGEOFF
	mov	w1, #34
	bl	__RNvNtCsgtPOCBgevO_4core9panicking5panic
LBB48_31:
Lloh378:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.5@PAGE
Lloh379:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.5@PAGEOFF
Lloh380:
	adrp	x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.6@PAGE
Lloh381:
	add	x2, x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.6@PAGEOFF
	mov	w1, #30
	bl	__RNvNtCsgtPOCBgevO_4core9panicking5panic
LBB48_32:
Lloh382:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.7@PAGE
Lloh383:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.7@PAGEOFF
Lloh384:
	adrp	x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.8@PAGE
Lloh385:
	add	x2, x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.8@PAGEOFF
	mov	w1, #43
	bl	__RNvNtCsgtPOCBgevO_4core9panicking5panic
LBB48_33:
Lloh386:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.9@PAGE
Lloh387:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.9@PAGEOFF
Lloh388:
	adrp	x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.10@PAGE
Lloh389:
	add	x2, x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.10@PAGEOFF
	mov	w1, #43
	bl	__RNvNtCsgtPOCBgevO_4core9panicking5panic
LBB48_34:
Lloh390:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.11@PAGE
Lloh391:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.11@PAGEOFF
Lloh392:
	adrp	x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.12@PAGE
Lloh393:
	add	x2, x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.12@PAGEOFF
	mov	w1, #23
	bl	__RNvNtCsgtPOCBgevO_4core9panicking5panic
LBB48_35:
Lloh394:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.1@PAGE
Lloh395:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.1@PAGEOFF
	bl	__RNvNtNtCsgtPOCBgevO_4core9panicking11panic_const23panic_const_rem_by_zero
	.loh AdrpAdd	Lloh376, Lloh377
	.loh AdrpAdd	Lloh374, Lloh375
	.loh AdrpAdd	Lloh380, Lloh381
	.loh AdrpAdd	Lloh378, Lloh379
	.loh AdrpAdd	Lloh384, Lloh385
	.loh AdrpAdd	Lloh382, Lloh383
	.loh AdrpAdd	Lloh388, Lloh389
	.loh AdrpAdd	Lloh386, Lloh387
	.loh AdrpAdd	Lloh392, Lloh393
	.loh AdrpAdd	Lloh390, Lloh391
	.loh AdrpAdd	Lloh394, Lloh395
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4fmod5fmodf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math4fmod5fmodf:
	.cfi_startproc
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_remember_state
	fmov	w8, s0
	mov	w9, #2139095040
	bic	w9, w9, w8
	cbz	w9, LBB49_9
	fmov	w9, s1
	neg	w10, w9
	and	w10, w10, #0x7f800000
	cbz	w10, LBB49_9
	and	w10, w8, #0x7fffffff
	and	w13, w9, #0x7fffffff
	cmp	w10, w13
	b.lo	LBB49_23
	subs	w9, w10, #2048, lsl #12
	csel	w9, wzr, w9, lo
	and	w16, w9, #0x7f800000
	subs	w11, w13, #2048, lsl #12
	csel	w14, wzr, w11, lo
	and	w15, w14, #0x7f800000
	sub	w12, w13, w15
	lsl	w11, w12, #1
	sub	w10, w10, w16
	cmp	w10, w11
	b.lo	LBB49_6
	cmp	w13, w15
	b.eq	LBB49_29
	udiv	w13, w10, w12
	msub	w10, w13, w12, w10
LBB49_6:
	and	w8, w8, #0x80000000
	lsr	w13, w9, #23
	lsr	w9, w14, #23
	sub	w13, w13, w9
	cmp	w13, #32
	b.hs	LBB49_10
	mov	w14, w10
	lsl	x14, x14, x13
	lsr	x15, x14, #32
	cmp	w12, w15
	b.ls	LBB49_10
	udiv	x10, x14, x12
	msub	w10, w10, w12, w14
	cbnz	w10, LBB49_21
	b	LBB49_22
LBB49_9:
	fmul	s0, s0, s1
	fdiv	s0, s0, s0
	b	LBB49_23
LBB49_10:
	lsr	w14, w12, #30
	cbnz	w14, LBB49_24
	cmp	w10, w11
	b.hs	LBB49_25
	sub	w11, w12, #1
	tst	w12, w11
	b.eq	LBB49_19
	clz	w11, w12
	sub	w11, w11, #2
	lsl	w12, w12, w11
	mov	w14, #536870912
	cmp	w12, w14
	b.ls	LBB49_26
	lsr	w14, w12, #30
	cbnz	w14, LBB49_27
	lsl	w14, w12, #1
	cmp	w10, w14
	b.hs	LBB49_28
	add	w13, w11, w13
	mov	w15, #-2147483648
	sub	w15, w15, w14
	lsl	x15, x15, #32
	mov	w14, w14
	udiv	x16, x15, x14
	msub	x15, x16, x14, x15
	lsl	w10, w10, #1
	mov	x17, #4294967296
	add	x16, x16, x17
	mul	x10, x16, x10
	cmp	w13, #31
	b.lo	LBB49_18
LBB49_17:
	lsr	x16, x10, #32
	lsl	x10, x10, #31
	and	x10, x10, #0x7fffffff00000000
	madd	x10, x16, x15, x10
	sub	w13, w13, #31
	cmp	w13, #30
	b.hi	LBB49_17
LBB49_18:
	lsr	x16, x10, #32
	eor	w17, w13, #0x1f
	lsr	w16, w16, w17
	lsl	x10, x10, x13
	and	x10, x10, #0x7fffffffffffffff
	madd	x10, x15, x16, x10
	lsr	x10, x10, #32
	add	x10, x10, #2
	mul	x10, x10, x14
	lsr	x10, x10, #32
	cmp	w12, w10
	csel	w12, wzr, w12, hi
	sub	w10, w10, w12
	lsr	w10, w10, w11
	cbnz	w10, LBB49_21
	b	LBB49_22
LBB49_19:
	cmp	w13, #32
	b.hs	LBB49_22
	lsl	w10, w10, w13
	and	w10, w10, w11
	cbz	w10, LBB49_22
LBB49_21:
	clz	w11, w10
	eor	w11, w11, #0x1f
	mov	w12, #23
	sub	w11, w12, w11
	cmp	w11, w9
	csel	w11, w11, w9, lo
	lsl	w10, w10, w11
	sub	w9, w9, w11
	add	w8, w10, w8
	add	w8, w8, w9, lsl #23
LBB49_22:
	fmov	s0, w8
LBB49_23:
	.cfi_def_cfa wsp, 16
	ldp	x29, x30, [sp], #16
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
LBB49_24:
	.cfi_restore_state
Lloh396:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.3@PAGE
Lloh397:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.3@PAGEOFF
Lloh398:
	adrp	x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.4@PAGE
Lloh399:
	add	x2, x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.4@PAGEOFF
	mov	w1, #34
	bl	__RNvNtCsgtPOCBgevO_4core9panicking5panic
LBB49_25:
Lloh400:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.5@PAGE
Lloh401:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.5@PAGEOFF
Lloh402:
	adrp	x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.6@PAGE
Lloh403:
	add	x2, x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.6@PAGEOFF
	mov	w1, #30
	bl	__RNvNtCsgtPOCBgevO_4core9panicking5panic
LBB49_26:
Lloh404:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.7@PAGE
Lloh405:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.7@PAGEOFF
Lloh406:
	adrp	x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.8@PAGE
Lloh407:
	add	x2, x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.8@PAGEOFF
	mov	w1, #43
	bl	__RNvNtCsgtPOCBgevO_4core9panicking5panic
LBB49_27:
Lloh408:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.9@PAGE
Lloh409:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.9@PAGEOFF
Lloh410:
	adrp	x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.10@PAGE
Lloh411:
	add	x2, x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.10@PAGEOFF
	mov	w1, #43
	bl	__RNvNtCsgtPOCBgevO_4core9panicking5panic
LBB49_28:
Lloh412:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.11@PAGE
Lloh413:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.11@PAGEOFF
Lloh414:
	adrp	x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.12@PAGE
Lloh415:
	add	x2, x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.12@PAGEOFF
	mov	w1, #23
	bl	__RNvNtCsgtPOCBgevO_4core9panicking5panic
LBB49_29:
Lloh416:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.1@PAGE
Lloh417:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.1@PAGEOFF
	bl	__RNvNtNtCsgtPOCBgevO_4core9panicking11panic_const23panic_const_rem_by_zero
	.loh AdrpAdd	Lloh398, Lloh399
	.loh AdrpAdd	Lloh396, Lloh397
	.loh AdrpAdd	Lloh402, Lloh403
	.loh AdrpAdd	Lloh400, Lloh401
	.loh AdrpAdd	Lloh406, Lloh407
	.loh AdrpAdd	Lloh404, Lloh405
	.loh AdrpAdd	Lloh410, Lloh411
	.loh AdrpAdd	Lloh408, Lloh409
	.loh AdrpAdd	Lloh414, Lloh415
	.loh AdrpAdd	Lloh412, Lloh413
	.loh AdrpAdd	Lloh416, Lloh417
	.cfi_endproc

	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0
lCPI50_0:
	.quad	0x3fcc71c51d8e78af
	.quad	0x3fd2492494229359
lCPI50_1:
	.quad	0x3fd999999997fa04
	.quad	0x3fe5555555555593
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4log24log2
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math4log24log2:
	.cfi_startproc
	fmov	x9, d0
	mov	x8, #4503599627370496
	cmp	x9, x8
	b.lt	LBB50_5
	lsr	x8, x9, #52
	cmp	x8, #2046
	b.hi	LBB50_12
	lsr	x8, x9, #32
	mov	w10, #1072693248
	cmp	x8, x10
	b.ne	LBB50_9
	cbz	w9, LBB50_13
	mov	w10, #-1023
	mov	w8, #1072693248
	b	LBB50_10
LBB50_5:
	fcmp	d0, #0.0
	b.ne	LBB50_7
	fmul	d0, d0, d0
	fmov	d1, #-1.00000000
	fdiv	d0, d1, d0
	ret
LBB50_7:
	tbnz	x9, #63, LBB50_11
	mov	x8, #4850376798678024192
	fmov	d1, x8
	fmul	d0, d0, d1
	fmov	x9, d0
	lsr	x8, x9, #32
	mov	w10, #-1077
	b	LBB50_10
LBB50_9:
	mov	w10, #-1023
LBB50_10:
	add	w8, w8, #149, lsl #12
	add	w11, w8, #3938
	add	w8, w10, w11, lsr #20
	and	w10, w11, #0xfffff
	mov	w11, #41118
	movk	w11, #16358, lsl #16
	add	w10, w10, w11
	mov	w9, w9
	orr	x9, x9, x10, lsl #32
	fmov	d0, x9
	fmov	d1, #-1.00000000
	fadd	d0, d0, d1
	fmov	d1, #0.50000000
	fmul	d1, d0, d1
	fmul	d1, d0, d1
	fmov	d2, #2.00000000
	fadd	d2, d0, d2
	fdiv	d2, d0, d2
	fmul	d3, d2, d2
	fmul	d4, d3, d3
	mov	x9, #21060
	movk	x9, #57150, lsl #16
	movk	x9, #61714, lsl #32
	movk	x9, #16322, lsl #48
	fmov	d5, x9
	fmul	d5, d4, d5
	mov	x9, #990
	movk	x9, #38603, lsl #16
	movk	x9, #18020, lsl #32
	movk	x9, #16327, lsl #48
	fmov	d6, x9
	fadd	d5, d5, d6
	dup.2d	v6, v4[0]
	mov	x9, #50847
	movk	x9, #53368, lsl #16
	movk	x9, #39433, lsl #32
	movk	x9, #16323, lsl #48
	fmov	d7, x9
	dup.2d	v7, v7[0]
	mov.d	v7[1], v5[0]
	fmul.2d	v5, v7, v4[0]
Lloh418:
	adrp	x9, lCPI50_0@PAGE
Lloh419:
	ldr	q7, [x9, lCPI50_0@PAGEOFF]
	fadd.2d	v5, v5, v7
	fmul.2d	v4, v5, v4[0]
Lloh420:
	adrp	x9, lCPI50_1@PAGE
Lloh421:
	ldr	q5, [x9, lCPI50_1@PAGEOFF]
	fadd.2d	v4, v4, v5
	mov.d	v6[1], v3[0]
	fmul.2d	v3, v6, v4
	faddp.2d	d3, v3
	fsub	d4, d0, d1
	fmov	x9, d4
	and	x9, x9, #0xffffffff00000000
	fmov	d4, x9
	fsub	d0, d0, d4
	fsub	d0, d0, d1
	fadd	d1, d1, d3
	fmul	d1, d2, d1
	fadd	d0, d0, d1
	mov	x9, #1696595968
	movk	x9, #5447, lsl #32
	movk	x9, #16375, lsl #48
	fmov	d1, x9
	fmul	d2, d4, d1
	fadd	d3, d0, d4
	mov	x9, #41472
	movk	x9, #12015, lsl #16
	movk	x9, #1532, lsl #32
	movk	x9, #15847, lsl #48
	fmov	d4, x9
	fmul	d3, d3, d4
	fmul	d0, d0, d1
	scvtf	d1, w8
	fadd	d0, d0, d3
	fadd	d3, d2, d1
	fsub	d1, d1, d3
	fadd	d1, d2, d1
	fadd	d0, d1, d0
	fadd	d0, d3, d0
	ret
LBB50_11:
	fsub	d0, d0, d0
	movi.2d	v1, #0000000000000000
	fdiv	d0, d0, d1
LBB50_12:
	ret
LBB50_13:
	movi.2d	v0, #0000000000000000
	ret
	.loh AdrpLdr	Lloh420, Lloh421
	.loh AdrpAdrp	Lloh418, Lloh420
	.loh AdrpLdr	Lloh418, Lloh419
	.cfi_endproc

	.section	__TEXT,__literal8,8byte_literals
	.p2align	3, 0x0
lCPI51_0:
	.long	0x3e91e9ee
	.long	0x3e789e26
lCPI51_1:
	.long	0x3f2aaaaa
	.long	0x3eccce13
	.section	__TEXT,__text,regular,pure_instructions
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math4logf4logf:
	.cfi_startproc
	fmov	w8, s0
	cmp	w8, #2048, lsl #12
	b.lt	LBB51_4
	lsr	w9, w8, #23
	cmp	w9, #254
	b.hi	LBB51_10
	movi.2d	v0, #0000000000000000
	mov	w9, #1065353216
	cmp	w8, w9
	b.eq	LBB51_10
	mov	w9, #-127
	b	LBB51_8
LBB51_4:
	fcmp	s0, #0.0
	b.ne	LBB51_6
	fmul	s0, s0, s0
	fmov	s1, #-1.00000000
	fdiv	s0, s1, s0
	ret
LBB51_6:
	tbnz	w8, #31, LBB51_9
	movi.2s	v1, #76, lsl #24
	fmul	s0, s0, s1
	fmov	w8, s0
	mov	w9, #-152
LBB51_8:
	add	w8, w8, #1199, lsl #12
	add	w8, w8, #2829
	add	w9, w9, w8, lsr #23
	and	w8, w8, #0x7fffff
	mov	w10, #1267
	movk	w10, #16181, lsl #16
	add	w8, w8, w10
	fmov	s0, w8
	fmov	s1, #-1.00000000
	fadd	s0, s0, s1
	fmov	s1, #2.00000000
	fadd	s1, s0, s1
	fdiv	s1, s0, s1
	fmul	s2, s1, s1
	fmul	s3, s2, s2
	dup.2s	v4, v3[0]
Lloh422:
	adrp	x8, lCPI51_0@PAGE
Lloh423:
	ldr	d5, [x8, lCPI51_0@PAGEOFF]
	fmul.2s	v3, v5, v3[0]
Lloh424:
	adrp	x8, lCPI51_1@PAGE
Lloh425:
	ldr	d5, [x8, lCPI51_1@PAGEOFF]
	fadd.2s	v3, v3, v5
	mov.s	v4[0], v2[0]
	fmul.2s	v2, v4, v3
	faddp.2s	s2, v2
	fmov	s3, #0.50000000
	fmul	s3, s0, s3
	fmul	s3, s0, s3
	scvtf	s4, w9
	fadd	s2, s3, s2
	fmul	s1, s1, s2
	mov	w8, #63441
	movk	w8, #14103, lsl #16
	fmov	s2, w8
	fmul	s2, s4, s2
	fadd	s1, s2, s1
	fsub	s1, s1, s3
	fadd	s0, s0, s1
	mov	w8, #29056
	movk	w8, #16177, lsl #16
	fmov	s1, w8
	fmul	s1, s4, s1
	fadd	s0, s1, s0
	ret
LBB51_9:
	fsub	s0, s0, s0
	movi.2d	v1, #0000000000000000
	fdiv	s0, s0, s1
LBB51_10:
	ret
	.loh AdrpLdr	Lloh424, Lloh425
	.loh AdrpAdrp	Lloh422, Lloh424
	.loh AdrpLdr	Lloh422, Lloh423
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4powf4powf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math4powf4powf:
	.cfi_startproc
	stp	d9, d8, [sp, #-32]!
	.cfi_def_cfa_offset 32
	stp	x29, x30, [sp, #16]
	add	x29, sp, #16
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset b8, -24
	.cfi_offset b9, -32
	.cfi_remember_state
	mov.16b	v2, v0
	fmov	w11, s2
	fmov	w8, s1
	and	w10, w8, #0x7fffffff
	fmov	s0, #1.00000000
	mov	w9, #1065353216
	cmp	w11, w9
	ccmp	w10, #0, #4, ne
	b.eq	LBB52_10
	fabs	s3, s2
	fmov	w9, s3
	mov	w12, #2139095040
	cmp	w9, w12
	b.hi	LBB52_9
	cmp	w10, w12
	b.hi	LBB52_9
	tbnz	w11, #31, LBB52_12
	mov	w12, #0
	mov	w13, #1065353216
	cmp	w10, w13
	b.eq	LBB52_13
LBB52_5:
	mov	w13, #2139095040
	cmp	w10, w13
	b.ne	LBB52_15
	mov	w10, #1065353216
	cmp	w9, w10
	cset	w9, gt
	csinv	w9, w9, wzr, ge
	ands	w9, w9, #0xff
	b.eq	LBB52_10
	cmp	w9, #1
	b.ne	LBB52_19
	cmn	w8, #1
	movi.2d	v0, #0000000000000000
	fcsel	s0, s1, s0, gt
	b	LBB52_10
LBB52_9:
	fadd	s0, s2, s1
LBB52_10:
	.cfi_def_cfa wsp, 32
LBB52_11:
	ldp	x29, x30, [sp, #16]
	ldp	d9, d8, [sp], #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore b8
	.cfi_restore b9
	ret
LBB52_12:
	.cfi_restore_state
	.cfi_remember_state
	mov	w12, #2
	lsr	w13, w10, #23
	mov	w14, #65528
	movk	w14, #16255, lsl #16
	orr	w14, w14, #0x7
	mov	w15, #150
	sub	w15, w15, w10, lsr #23
	lsr	w16, w10, w15
	lsl	w15, w16, w15
	and	w16, w16, #0x1
	sub	w16, w12, w16
	cmp	w15, w10
	csel	w15, wzr, w16, ne
	cmp	w10, w14
	csel	w14, wzr, w15, ls
	cmp	w13, #150
	csel	w12, w12, w14, hi
	mov	w13, #1065353216
	cmp	w10, w13
	b.ne	LBB52_5
LBB52_13:
	tbnz	w8, #31, LBB52_18
	mov.16b	v0, v2
	b	LBB52_10
LBB52_15:
	mov	w13, #1056964608
	cmp	w8, w13
	b.eq	LBB52_20
	mov	w13, #1073741824
	cmp	w8, w13
	b.ne	LBB52_22
	fmul	s0, s2, s2
	b	LBB52_10
LBB52_18:
	fdiv	s0, s0, s2
	b	LBB52_10
LBB52_19:
	fneg	s0, s1
	cmn	w8, #1
	movi.2d	v1, #0000000000000000
	fcsel	s0, s1, s0, gt
	b	LBB52_10
LBB52_20:
	tbnz	w11, #31, LBB52_26
	; InlineAsm Start
	fsqrt	s2, s2
	; InlineAsm End
	mov.16b	v0, v2
	.cfi_def_cfa wsp, 32
	b	LBB52_11
LBB52_22:
	.cfi_restore_state
	cbz	w9, LBB52_25
	and	w13, w9, #0x3fffffff
	mov	w14, #1065353216
	cmp	w13, w14
	b.eq	LBB52_25
	fmov	s8, #1.00000000
	tbz	w11, #31, LBB52_35
	b	LBB52_28
LBB52_25:
	fdiv	s0, s0, s3
	cmp	w8, #0
	fcsel	s3, s0, s3, mi
	mov.16b	v0, v3
	tbz	w11, #31, LBB52_10
	b	LBB52_31
LBB52_26:
	cbz	w9, LBB52_31
	and	w11, w9, #0x3fffffff
	mov	w13, #1065353216
	cmp	w11, w13
	b.eq	LBB52_31
LBB52_28:
	cmp	w12, #1
	b.eq	LBB52_34
	fmov	s8, #1.00000000
	cbnz	w12, LBB52_35
	fsub	s0, s2, s2
	fdiv	s0, s0, s0
	b	LBB52_10
LBB52_31:
	mov	w8, #-1065353216
	add	w8, w9, w8
	orr	w8, w12, w8
	cbz	w8, LBB52_33
	fneg	s0, s3
	cmp	w12, #1
	fcsel	s0, s0, s3, eq
	b	LBB52_10
LBB52_33:
	fsub	s0, s3, s3
	fdiv	s0, s0, s0
	b	LBB52_10
LBB52_34:
	fmov	s8, #-1.00000000
LBB52_35:
	mov	w11, #1291845632
	cmp	w10, w11
	b.ls	LBB52_38
	mov	w10, #65528
	movk	w10, #16255, lsl #16
	cmp	w9, w10
	b.hs	LBB52_44
	tbz	w8, #31, LBB52_50
	b	LBB52_40
LBB52_38:
	mov	w10, #1266679808
	fmov	s0, w10
	fmul	s0, s3, s0
	fmov	w10, s0
	cmp	w9, #2048, lsl #12
	csel	w9, w10, w9, lo
	mov	w10, #-127
	mov	w11, #-151
	csel	w10, w11, w10, lo
	add	w10, w10, w9, asr #23
	and	w11, w9, #0x7fffff
	mov	w12, #1065353216
	bfxil	w12, w9, #0, #23
	mov	w9, #50290
	movk	w9, #28, lsl #16
	mov	w13, #46039
	movk	w13, #93, lsl #16
	orr	w14, w11, #0x3f000000
	cmp	w11, w13
	csel	w13, w12, w14, lo
	cinc	w14, w10, hs
	cset	w15, lo
	cmp	w11, w9
	csel	w11, w12, w13, lo
	csel	w10, w10, w14, lo
	csel	w9, wzr, w15, lo
	fmov	s0, w11
Lloh426:
	adrp	x12, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.88@PAGE
Lloh427:
	add	x12, x12, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.88@PAGEOFF
	ldr	s2, [x12, w9, uxtw #2]
	fsub	s3, s0, s2
	fadd	s4, s2, s0
	fmov	s5, #1.00000000
	fdiv	s4, s5, s4
	lsr	w11, w11, #1
	and	w11, w11, #0x1ffff000
	add	w11, w11, w9, lsl #21
	mov	w12, #541065216
	add	w11, w11, w12
	fmov	s5, w11
	fsub	s2, s2, s5
	fadd	s6, s2, s0
	fmul	s0, s3, s4
	fmov	w11, s0
	and	w11, w11, #0xfffff000
	fmov	s2, w11
	fmul	s5, s5, s2
	fsub	s3, s3, s5
	fmul	s5, s6, s2
	fmul	s6, s0, s0
	fsub	s3, s3, s5
	mov.s	v4[1], v6[0]
	mov.16b	v5, v4
	mov.s	v4[0], v3[0]
	fmul.2s	v3, v5, v4
	mov	w11, #61762
	movk	w11, #15955, lsl #16
	fmov	s4, w11
	fmul	s4, s6, s4
	mov	w11, #12885
	movk	w11, #15980, lsl #16
	fmov	s5, w11
	fadd	s4, s4, s5
	fmul	s4, s6, s4
	mov	w11, #41733
	movk	w11, #16011, lsl #16
	fmov	s5, w11
	fadd	s4, s4, s5
	fmul	s4, s6, s4
	mov	w11, #43691
	movk	w11, #16042, lsl #16
	fmov	s5, w11
	fadd	s4, s4, s5
	fmul	s4, s6, s4
	mov	w11, #28087
	movk	w11, #16091, lsl #16
	fmov	s5, w11
	fadd	s4, s4, s5
	fmul	s4, s6, s4
	mov.16b	v5, v0
	mov.s	v5[1], v4[0]
	mov	w11, #39322
	movk	w11, #16153, lsl #16
	fmov	s4, w11
	dup.2s	v4, v4[0]
	mov.s	v4[0], v2[0]
	fadd.2s	v4, v5, v4
	fmul.2s	v4, v4, v3
	faddp.2s	s4, v4
	fmul	s5, s2, s2
	fmov	s6, #3.00000000
	fadd	s6, s5, s6
	fadd	s6, s6, s4
	fmov	w11, s6
	and	w11, w11, #0xfffff000
	fmov	s6, w11
	fmov	s7, #-3.00000000
	fadd	s7, s6, s7
	fsub	s5, s7, s5
	fsub	s4, s4, s5
	fmul	s2, s2, s6
	fmul	s3, s6, s3
	fmul	s0, s0, s4
	fadd	s0, s3, s0
	fadd	s3, s2, s0
	fmov	w11, s3
	and	w11, w11, #0xfffff000
	fmov	s3, w11
	fsub	s2, s3, s2
	fsub	s0, s0, s2
	mov	w11, #16384
	movk	w11, #16246, lsl #16
	fmov	s2, w11
	fmul	s4, s3, s2
	mov	w11, #9158
	movk	w11, #47350, lsl #16
	fmov	s2, w11
	fmul	s2, s3, s2
	mov	w11, #14415
	movk	w11, #16246, lsl #16
	fmov	s3, w11
	fmul	s0, s0, s3
Lloh428:
	adrp	x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.89@PAGE
Lloh429:
	add	x11, x11, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.89@PAGEOFF
	ldr	s3, [x11, w9, uxtw #2]
	fadd	s0, s0, s2
	fadd	s0, s3, s0
	scvtf	s3, w10
Lloh430:
	adrp	x10, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.90@PAGE
Lloh431:
	add	x10, x10, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.90@PAGEOFF
	fadd	s2, s4, s0
	ldr	s5, [x10, w9, uxtw #2]
	fadd	s2, s5, s2
	fadd	s2, s2, s3
	fmov	w9, s2
	and	w9, w9, #0xfffff000
	fmov	s2, w9
	fsub	s3, s2, s3
	fsub	s3, s3, s5
	fsub	s3, s3, s4
LBB52_39:
	fsub	s0, s0, s3
	and	w8, w8, #0xfffff000
	fmov	s3, w8
	fsub	s4, s1, s3
	fmul	s4, s4, s2
	fmul	s0, s1, s0
	fadd	s0, s4, s0
	fmul	s1, s2, s3
	fadd	s2, s1, s0
	fmov	w8, s2
	mov	w9, #1124073472
	cmp	w8, w9
	b.le	LBB52_41
LBB52_40:
	mov	w8, #62154
	movk	w8, #29001, lsl #16
	b	LBB52_51
LBB52_41:
	b.ne	LBB52_46
	mov	w9, #43580
	movk	w9, #13112, lsl #16
	fmov	s3, w9
	fadd	s3, s0, s3
	fsub	s4, s2, s1
	fcmp	s3, s4
	b.gt	LBB52_40
	fmov	w9, s2
	and	w9, w9, #0x7fffffff
	b	LBB52_48
LBB52_44:
	add	w10, w10, #15
	cmp	w9, w10
	b.ls	LBB52_52
	cmp	w8, #0
	b.gt	LBB52_40
	b	LBB52_50
LBB52_46:
	fmov	w9, s2
	and	w9, w9, #0x7fffffff
	mov	w10, #1125515264
	cmp	w9, w10
	b.hi	LBB52_50
	fsub	s2, s2, s1
	mov	w10, #-1021968384
	cmp	w8, w10
	fccmp	s0, s2, #2, eq
	b.ls	LBB52_50
LBB52_48:
	mov	w10, #1056964608
	cmp	w9, w10
	b.ls	LBB52_53
	lsr	w9, w9, #23
	add	w9, w9, #2
	mov	w10, #8388608
	lsr	w9, w10, w9
	add	w9, w9, w8
	lsr	w11, w9, #23
	add	w12, w11, #1
	mov	w13, #-8388608
	asr	w12, w13, w12
	and	w12, w12, w9
	bfxil	w10, w9, #0, #23
	mov	w9, #22
	sub	w9, w9, w11
	lsr	w9, w10, w9
	cmp	w8, #0
	cneg	w0, w9, mi
	fmov	s2, w12
	fsub	s1, s1, s2
	fadd	s2, s0, s1
	fmov	w8, s2
	b	LBB52_54
LBB52_50:
	mov	w8, #16992
	movk	w8, #3490, lsl #16
LBB52_51:
	fmov	s0, w8
	fmul	s1, s8, s0
	fmul	s0, s1, s0
	b	LBB52_10
LBB52_52:
	fmov	s0, #-1.00000000
	fadd	s0, s3, s0
	fmul	s2, s0, s0
	fmov	s3, #-0.25000000
	fmul	s3, s0, s3
	mov	w9, #43691
	movk	w9, #16042, lsl #16
	fmov	s4, w9
	fadd	s3, s3, s4
	fmul	s3, s0, s3
	fmov	s4, #0.50000000
	fsub	s3, s4, s3
	fmul	s2, s2, s3
	mov	w9, #43520
	movk	w9, #16312, lsl #16
	fmov	s3, w9
	fmul	s3, s0, s3
	mov	w9, #42352
	movk	w9, #14060, lsl #16
	fmov	s4, w9
	fmul	s0, s0, s4
	mov	w9, #43579
	movk	w9, #49080, lsl #16
	fmov	s4, w9
	fmul	s2, s2, s4
	fadd	s0, s0, s2
	fadd	s2, s3, s0
	fmov	w9, s2
	and	w9, w9, #0xfffff000
	fmov	s2, w9
	fsub	s3, s2, s3
	b	LBB52_39
LBB52_53:
	mov	w0, #0
LBB52_54:
	and	w8, w8, #0xffff8000
	fmov	s2, w8
	mov	w8, #29184
	movk	w8, #16177, lsl #16
	fmov	s3, w8
	fmul	s3, s2, s3
	fsub	s1, s2, s1
	fsub	s0, s0, s1
	mov	w8, #29208
	movk	w8, #16177, lsl #16
	fmov	s1, w8
	fmul	s0, s0, s1
	mov	w8, #48780
	movk	w8, #13759, lsl #16
	fmov	s1, w8
	fmul	s1, s2, s1
	fadd	s0, s1, s0
	fadd	s1, s3, s0
	fsub	s2, s1, s3
	fsub	s0, s0, s2
	fmul	s2, s1, s1
	mov	w8, #47948
	movk	w8, #13105, lsl #16
	fmov	s3, w8
	fmul	s3, s2, s3
	mov	w8, #59918
	movk	w8, #46557, lsl #16
	fmov	s4, w8
	fadd	s3, s3, s4
	fmul	s3, s2, s3
	mov	w8, #45909
	movk	w8, #14474, lsl #16
	fmov	s4, w8
	fadd	s3, s3, s4
	fmul	s3, s2, s3
	mov	w8, #2913
	movk	w8, #47926, lsl #16
	fmov	s4, w8
	fadd	s3, s3, s4
	fmul	s3, s2, s3
	mov	w8, #43691
	movk	w8, #15914, lsl #16
	fmov	s4, w8
	fadd	s3, s3, s4
	fmul	s2, s2, s3
	fsub	s2, s1, s2
	fmul	s3, s1, s2
	fmov	s4, #-2.00000000
	fadd	s2, s2, s4
	fdiv	s2, s3, s2
	fmul	s3, s1, s0
	fadd	s0, s0, s3
	fsub	s0, s2, s0
	fsub	s0, s1, s0
	fmov	s1, #1.00000000
	fadd	s0, s0, s1
	fmov	w8, s0
	add	w8, w8, w0, lsl #23
	cmp	w8, #2048, lsl #12
	b.ge	LBB52_56
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5ldexp6ldexpf
	b	LBB52_57
LBB52_56:
	fmov	s0, w8
LBB52_57:
	fmul	s0, s8, s0
	b	LBB52_10
	.loh AdrpAdd	Lloh430, Lloh431
	.loh AdrpAdd	Lloh428, Lloh429
	.loh AdrpAdd	Lloh426, Lloh427
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4sinf4sinf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math4sinf4sinf:
	.cfi_startproc
	sub	sp, sp, #48
	.cfi_def_cfa_offset 48
	stp	x20, x19, [sp, #16]
	stp	x29, x30, [sp, #32]
	add	x29, sp, #32
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_remember_state
	fcvt	d1, s0
	fmov	w19, s0
	and	w8, w19, #0x7fffffff
	mov	w9, #4059
	movk	w9, #16201, lsl #16
	cmp	w8, w9
	b.hs	LBB53_3
	lsr	w9, w8, #23
	cmp	w9, #115
	b.hs	LBB53_6
	mov	w9, #2071986176
	fmov	s1, w9
	fadd	s1, s0, s1
	mov	w9, #58720256
	fmov	s2, w9
	fmul	s2, s0, s2
	cmp	w8, #2048, lsl #12
	fcsel	s1, s2, s1, lo
	str	s1, [sp, #8]
	ldr	s1, [sp, #8]
	b	LBB53_20
LBB53_3:
	mov	w9, #21458
	movk	w9, #16507, lsl #16
	cmp	w8, w9
	b.hs	LBB53_7
	mov	w9, #52196
	movk	w9, #16406, lsl #16
	cmp	w8, w9
	b.hs	LBB53_11
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #16377, lsl #48
	fmov	d0, x8
	fadd	d0, d1, d0
	fmul	d0, d0, d0
	fmul	d2, d0, d0
	mov	x8, #20585
	movk	x8, #57582, lsl #16
	movk	x8, #37698, lsl #32
	movk	x8, #16121, lsl #48
	fmov	d3, x8
	fmul	d3, d0, d3
	mov	x8, #7719
	movk	x8, #59407, lsl #16
	movk	x8, #49287, lsl #32
	movk	x8, #48982, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	mov	x8, #-41343
	movk	x8, #64780, lsl #16
	movk	x8, #49119, lsl #48
	fmov	d4, x8
	fmul	d4, d0, d4
	fmov	d5, #1.00000000
	fadd	d4, d4, d5
	mov	x8, #14914
	movk	x8, #57605, lsl #16
	movk	x8, #21843, lsl #32
	movk	x8, #16293, lsl #48
	fmov	d6, x8
	fmul	d6, d2, d6
	fadd	d4, d4, d6
	fmul	d0, d0, d2
	fmul	d0, d0, d3
	fadd	d0, d4, d0
	fcvt	s0, d0
	fneg	s0, s0
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #49145, lsl #48
	fmov	d2, x8
	fadd	d1, d1, d2
	fmul	d1, d1, d1
	fmul	d2, d1, d1
	mov	x8, #20585
	movk	x8, #57582, lsl #16
	movk	x8, #37698, lsl #32
	movk	x8, #16121, lsl #48
	fmov	d3, x8
	fmul	d3, d1, d3
	mov	x8, #7719
	movk	x8, #59407, lsl #16
	movk	x8, #49287, lsl #32
	movk	x8, #48982, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	mov	x8, #-41343
	movk	x8, #64780, lsl #16
	movk	x8, #49119, lsl #48
	fmov	d4, x8
	fmul	d4, d1, d4
	fadd	d4, d4, d5
	mov	x8, #14914
	movk	x8, #57605, lsl #16
	movk	x8, #21843, lsl #32
	movk	x8, #16293, lsl #48
	fmov	d5, x8
	fmul	d5, d2, d5
	fadd	d4, d4, d5
	fmul	d1, d1, d2
	fmul	d1, d1, d3
	fadd	d1, d4, d1
	fcvt	s1, d1
	b	LBB53_10
LBB53_6:
	fmul	d0, d1, d1
	fmul	d2, d0, d0
	mov	x8, #18087
	movk	x8, #35899, lsl #16
	movk	x8, #52615, lsl #32
	movk	x8, #16070, lsl #48
	fmov	d3, x8
	fmul	d3, d0, d3
	mov	x8, #59252
	movk	x8, #58058, lsl #16
	movk	x8, #249, lsl #32
	movk	x8, #48938, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d4, d0, d1
	mov	x8, #64434
	movk	x8, #35182, lsl #16
	movk	x8, #4368, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d5, x8
	fmul	d0, d0, d5
	mov	x8, #44151
	movk	x8, #21707, lsl #16
	movk	x8, #21845, lsl #32
	movk	x8, #49093, lsl #48
	fmov	d5, x8
	fadd	d0, d0, d5
	fmul	d0, d4, d0
	b	LBB53_17
LBB53_7:
	mov	w9, #12758
	movk	w9, #16610, lsl #16
	cmp	w8, w9
	b.hs	LBB53_13
	mov	w9, #60896
	movk	w9, #16559, lsl #16
	cmp	w8, w9
	b.hs	LBB53_15
	mov	x8, #8658
	movk	x8, #32563, lsl #16
	movk	x8, #55676, lsl #32
	movk	x8, #16402, lsl #48
	fmov	d0, x8
	fadd	d0, d1, d0
	fmul	d0, d0, d0
	fmul	d2, d0, d0
	mov	x8, #20585
	movk	x8, #57582, lsl #16
	movk	x8, #37698, lsl #32
	movk	x8, #16121, lsl #48
	fmov	d3, x8
	fmul	d3, d0, d3
	mov	x8, #7719
	movk	x8, #59407, lsl #16
	movk	x8, #49287, lsl #32
	movk	x8, #48982, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	mov	x8, #-41343
	movk	x8, #64780, lsl #16
	movk	x8, #49119, lsl #48
	fmov	d4, x8
	fmul	d4, d0, d4
	fmov	d5, #1.00000000
	fadd	d4, d4, d5
	mov	x8, #14914
	movk	x8, #57605, lsl #16
	movk	x8, #21843, lsl #32
	movk	x8, #16293, lsl #48
	fmov	d6, x8
	fmul	d6, d2, d6
	fadd	d4, d4, d6
	fmul	d0, d0, d2
	fmul	d0, d0, d3
	fadd	d0, d4, d0
	fcvt	s0, d0
	mov	x8, #8658
	movk	x8, #32563, lsl #16
	movk	x8, #55676, lsl #32
	movk	x8, #49170, lsl #48
	fmov	d2, x8
	fadd	d1, d1, d2
	fmul	d1, d1, d1
	fmul	d2, d1, d1
	mov	x8, #20585
	movk	x8, #57582, lsl #16
	movk	x8, #37698, lsl #32
	movk	x8, #16121, lsl #48
	fmov	d3, x8
	fmul	d3, d1, d3
	mov	x8, #7719
	movk	x8, #59407, lsl #16
	movk	x8, #49287, lsl #32
	movk	x8, #48982, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	mov	x8, #-41343
	movk	x8, #64780, lsl #16
	movk	x8, #49119, lsl #48
	fmov	d4, x8
	fmul	d4, d1, d4
	fadd	d4, d4, d5
	mov	x8, #14914
	movk	x8, #57605, lsl #16
	movk	x8, #21843, lsl #32
	movk	x8, #16293, lsl #48
	fmov	d5, x8
	fmul	d5, d2, d5
	fadd	d4, d4, d5
	fmul	d1, d1, d2
	fmul	d1, d1, d3
	fadd	d1, d4, d1
	fcvt	s1, d1
	fneg	s1, s1
LBB53_10:
	tst	w19, #0x80000000
	fcsel	s0, s0, s1, ne
	b	LBB53_20
LBB53_11:
	cmn	w19, #1
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #16393, lsl #48
	fmov	d0, x8
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #49161, lsl #48
	fmov	d2, x8
	fcsel	d0, d2, d0, gt
	fadd	d0, d0, d1
LBB53_12:
	fmul	d1, d0, d0
	fmul	d2, d1, d1
	mov	x8, #18087
	movk	x8, #35899, lsl #16
	movk	x8, #52615, lsl #32
	movk	x8, #16070, lsl #48
	fmov	d3, x8
	fmul	d3, d1, d3
	mov	x8, #59252
	movk	x8, #58058, lsl #16
	movk	x8, #249, lsl #32
	movk	x8, #48938, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fnmul	d4, d0, d1
	mov	x8, #64434
	movk	x8, #35182, lsl #16
	movk	x8, #4368, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d5, x8
	fmul	d1, d1, d5
	mov	x8, #44151
	movk	x8, #21707, lsl #16
	movk	x8, #21845, lsl #32
	movk	x8, #49093, lsl #48
	fmov	d5, x8
	fadd	d1, d1, d5
	fmul	d1, d4, d1
	fsub	d0, d1, d0
	b	LBB53_18
LBB53_13:
	lsr	w9, w8, #23
	cmp	w9, #254
	b.ls	LBB53_21
	fsub	s0, s0, s0
	b	LBB53_20
LBB53_15:
	cmn	w19, #1
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #16409, lsl #48
	fmov	d0, x8
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #49177, lsl #48
	fmov	d2, x8
	fcsel	d0, d2, d0, gt
	fadd	d0, d0, d1
LBB53_16:
	fmul	d1, d0, d0
	fmul	d2, d1, d1
	mov	x8, #18087
	movk	x8, #35899, lsl #16
	movk	x8, #52615, lsl #32
	movk	x8, #16070, lsl #48
	fmov	d3, x8
	fmul	d3, d1, d3
	mov	x8, #59252
	movk	x8, #58058, lsl #16
	movk	x8, #249, lsl #32
	movk	x8, #48938, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d4, d0, d1
	mov	x8, #64434
	movk	x8, #35182, lsl #16
	movk	x8, #4368, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d5, x8
	fmul	d1, d1, d5
	mov	x8, #44151
	movk	x8, #21707, lsl #16
	movk	x8, #21845, lsl #32
	movk	x8, #49093, lsl #48
	fmov	d5, x8
	fadd	d1, d1, d5
	fmul	d1, d4, d1
LBB53_17:
	fadd	d0, d0, d1
LBB53_18:
	fmul	d1, d4, d2
	fmul	d1, d1, d3
	fadd	d0, d1, d0
LBB53_19:
	fcvt	s0, d0
LBB53_20:
	.cfi_def_cfa wsp, 48
	ldp	x29, x30, [sp, #32]
	ldp	x20, x19, [sp, #16]
	add	sp, sp, #48
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	ret
LBB53_21:
	.cfi_restore_state
	str	xzr, [sp, #8]
	mov	w10, #4058
	movk	w10, #19913, lsl #16
	cmp	w8, w10
	b.hi	LBB53_23
	mov	x8, #51331
	movk	x8, #28105, lsl #16
	movk	x8, #24368, lsl #32
	movk	x8, #16356, lsl #48
	fmov	d0, x8
	fmul	d0, d1, d0
	mov	x8, #4843621399236968448
	fmov	d2, x8
	fadd	d0, d0, d2
	mov	x8, #-4379750637617807360
	fmov	d2, x8
	fadd	d0, d0, d2
	fcvtzs	w0, d0
	mov	x8, #1342177280
	movk	x8, #8699, lsl #32
	movk	x8, #49145, lsl #48
	fmov	d2, x8
	fmul	d2, d0, d2
	fadd	d1, d1, d2
	mov	x8, #25187
	movk	x8, #24858, lsl #16
	movk	x8, #4276, lsl #32
	movk	x8, #48721, lsl #48
	fmov	d2, x8
	fmul	d0, d0, d2
	fadd	d0, d1, d0
	b	LBB53_26
LBB53_23:
	sub	w3, w9, #150
	sub	w8, w8, w3, lsl #23
	fmov	s0, w8
	fcvt	d0, s0
	str	d0, [sp]
	mov	x0, sp
	add	x2, sp, #8
	mov	w1, #1
	mov	x4, #0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math14rem_pio2_large14rem_pio2_large
	tbnz	w19, #31, LBB53_25
	ldr	d0, [sp, #8]
	b	LBB53_26
LBB53_25:
	neg	w0, w0
	ldr	d0, [sp, #8]
	fneg	d0, d0
LBB53_26:
	and	w8, w0, #0x3
	cmp	w8, #1
	b.gt	LBB53_29
	cbz	w8, LBB53_16
	fmul	d0, d0, d0
	fmul	d1, d0, d0
	mov	x8, #20585
	movk	x8, #57582, lsl #16
	movk	x8, #37698, lsl #32
	movk	x8, #16121, lsl #48
	fmov	d2, x8
	fmul	d2, d0, d2
	mov	x8, #7719
	movk	x8, #59407, lsl #16
	movk	x8, #49287, lsl #32
	movk	x8, #48982, lsl #48
	fmov	d3, x8
	fadd	d2, d2, d3
	mov	x8, #-41343
	movk	x8, #64780, lsl #16
	movk	x8, #49119, lsl #48
	fmov	d3, x8
	fmul	d3, d0, d3
	fmov	d4, #1.00000000
	fadd	d3, d3, d4
	mov	x8, #14914
	movk	x8, #57605, lsl #16
	movk	x8, #21843, lsl #32
	movk	x8, #16293, lsl #48
	fmov	d4, x8
	fmul	d4, d1, d4
	fadd	d3, d3, d4
	fmul	d0, d0, d1
	fmul	d0, d0, d2
	fadd	d0, d3, d0
	b	LBB53_19
LBB53_29:
	cmp	w8, #2
	b.eq	LBB53_12
	fmul	d0, d0, d0
	fmul	d1, d0, d0
	mov	x8, #20585
	movk	x8, #57582, lsl #16
	movk	x8, #37698, lsl #32
	movk	x8, #16121, lsl #48
	fmov	d2, x8
	fmul	d2, d0, d2
	mov	x8, #7719
	movk	x8, #59407, lsl #16
	movk	x8, #49287, lsl #32
	movk	x8, #48982, lsl #48
	fmov	d3, x8
	fadd	d2, d2, d3
	mov	x8, #-41343
	movk	x8, #64780, lsl #16
	movk	x8, #49119, lsl #48
	fmov	d3, x8
	fmul	d3, d0, d3
	fmov	d4, #1.00000000
	fadd	d3, d3, d4
	mov	x8, #14914
	movk	x8, #57605, lsl #16
	movk	x8, #21843, lsl #32
	movk	x8, #16293, lsl #48
	fmov	d4, x8
	fmul	d4, d1, d4
	fadd	d3, d3, d4
	fmul	d0, d0, d1
	fmul	d0, d0, d2
	fadd	d0, d3, d0
	fcvt	s0, d0
	fneg	s0, s0
	b	LBB53_20
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4sinh4sinh
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math4sinh4sinh:
	.cfi_startproc
	sub	sp, sp, #80
	.cfi_def_cfa_offset 80
	stp	d9, d8, [sp, #32]
	stp	x20, x19, [sp, #48]
	stp	x29, x30, [sp, #64]
	add	x29, sp, #64
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset b8, -40
	.cfi_offset b9, -48
	mov.16b	v2, v0
	fmov	d0, #0.50000000
	movi.2d	v1, #0xffffffffffffffff
	fneg.2d	v1, v1
	bsl.16b	v1, v0, v2
	fabs	d0, d2
	fmov	x19, d0
	mov	x8, #50861002719232
	movk	x8, #16518, lsl #48
	cmp	x19, x8
	b.hs	LBB54_4
	stp	q2, q1, [sp]
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5expm15expm1
	lsr	x8, x19, #52
	cmp	x8, #1023
	b.hs	LBB54_5
	cmp	x8, #997
	ldr	q1, [sp]
	b.lo	LBB54_7
	fadd	d1, d0, d0
	fmul	d2, d0, d0
	fmov	d3, #1.00000000
	fadd	d0, d0, d3
	fdiv	d0, d2, d0
	fsub	d0, d1, d0
	b	LBB54_6
LBB54_4:
	fadd	d8, d1, d1
	mov	x8, #56715
	movk	x8, #5402, lsl #16
	movk	x8, #8294, lsl #32
	movk	x8, #49302, lsl #48
	fmov	d1, x8
	fadd	d0, d0, d1
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3exp3exp
	mov	x8, #9205357638345293824
	fmov	d1, x8
	fmul	d0, d0, d1
	fmul	d0, d0, d1
	fmul	d1, d8, d0
	b	LBB54_7
LBB54_5:
	fmov	d1, #1.00000000
	fadd	d1, d0, d1
	fdiv	d1, d0, d1
	fadd	d0, d0, d1
LBB54_6:
	ldr	q1, [sp, #16]
	fmul	d1, d1, d0
LBB54_7:
	mov.16b	v0, v1
	.cfi_def_cfa wsp, 80
	ldp	x29, x30, [sp, #64]
	ldp	x20, x19, [sp, #48]
	ldp	d9, d8, [sp, #32]
	add	sp, sp, #80
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore b8
	.cfi_restore b9
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4tanf4tanf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math4tanf4tanf:
	.cfi_startproc
	sub	sp, sp, #48
	.cfi_def_cfa_offset 48
	stp	x20, x19, [sp, #16]
	stp	x29, x30, [sp, #32]
	add	x29, sp, #32
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_remember_state
	fcvt	d1, s0
	fmov	w19, s0
	and	w8, w19, #0x7fffffff
	mov	w9, #4059
	movk	w9, #16201, lsl #16
	cmp	w8, w9
	b.hs	LBB55_3
	lsr	w9, w8, #23
	cmp	w9, #115
	b.hs	LBB55_6
	mov	w9, #2071986176
	fmov	s1, w9
	fadd	s1, s0, s1
	mov	w9, #58720256
	fmov	s2, w9
	fmul	s2, s0, s2
	cmp	w8, #2048, lsl #12
	fcsel	s1, s2, s1, lo
	str	s1, [sp, #8]
	ldr	s1, [sp, #8]
	b	LBB55_18
LBB55_3:
	mov	w9, #21458
	movk	w9, #16507, lsl #16
	cmp	w8, w9
	b.hs	LBB55_7
	mov	w9, #52196
	movk	w9, #16406, lsl #16
	cmp	w8, w9
	b.hs	LBB55_11
	cmn	w19, #1
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #16377, lsl #48
	fmov	d0, x8
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #49145, lsl #48
	b	LBB55_10
LBB55_6:
	fmul	d0, d1, d1
	mov	x8, #7117
	movk	x8, #49047, lsl #16
	movk	x8, #25273, lsl #32
	movk	x8, #16259, lsl #48
	fmov	d2, x8
	fmul	d2, d0, d2
	mov	x8, #62542
	movk	x8, #64748, lsl #16
	movk	x8, #23981, lsl #32
	movk	x8, #16232, lsl #48
	fmov	d3, x8
	fadd	d2, d2, d3
	mov	x8, #13262
	movk	x8, #37004, lsl #16
	movk	x8, #7667, lsl #32
	movk	x8, #16281, lsl #48
	fmov	d3, x8
	fmul	d3, d0, d3
	mov	x8, #23294
	movk	x8, #7558, lsl #16
	movk	x8, #21705, lsl #32
	movk	x8, #16299, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d4, d0, d0
	fmul	d5, d0, d1
	mov	x8, #40818
	movk	x8, #14489, lsl #16
	movk	x8, #4861, lsl #32
	movk	x8, #16321, lsl #48
	fmov	d6, x8
	fmul	d0, d0, d6
	mov	x8, #51615
	movk	x8, #13336, lsl #16
	movk	x8, #21837, lsl #32
	movk	x8, #16341, lsl #48
	fmov	d6, x8
	fadd	d0, d0, d6
	fmul	d0, d5, d0
	b	LBB55_16
LBB55_7:
	mov	w9, #12758
	movk	w9, #16610, lsl #16
	cmp	w8, w9
	b.hs	LBB55_12
	mov	w9, #60896
	movk	w9, #16559, lsl #16
	cmp	w8, w9
	b.hs	LBB55_14
	cmn	w19, #1
	mov	x8, #8658
	movk	x8, #32563, lsl #16
	movk	x8, #55676, lsl #32
	movk	x8, #16402, lsl #48
	fmov	d0, x8
	mov	x8, #8658
	movk	x8, #32563, lsl #16
	movk	x8, #55676, lsl #32
	movk	x8, #49170, lsl #48
LBB55_10:
	fmov	d2, x8
	fcsel	d0, d2, d0, gt
	fadd	d0, d0, d1
	fmul	d1, d0, d0
	mov	x8, #7117
	movk	x8, #49047, lsl #16
	movk	x8, #25273, lsl #32
	movk	x8, #16259, lsl #48
	fmov	d2, x8
	fmul	d2, d1, d2
	mov	x8, #62542
	movk	x8, #64748, lsl #16
	movk	x8, #23981, lsl #32
	movk	x8, #16232, lsl #48
	fmov	d3, x8
	fadd	d2, d2, d3
	mov	x8, #13262
	movk	x8, #37004, lsl #16
	movk	x8, #7667, lsl #32
	movk	x8, #16281, lsl #48
	fmov	d3, x8
	fmul	d3, d1, d3
	mov	x8, #23294
	movk	x8, #7558, lsl #16
	movk	x8, #21705, lsl #32
	movk	x8, #16299, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d4, d1, d1
	fmul	d5, d0, d1
	mov	x8, #40818
	movk	x8, #14489, lsl #16
	movk	x8, #4861, lsl #32
	movk	x8, #16321, lsl #48
	fmov	d6, x8
	fmul	d1, d1, d6
	mov	x8, #51615
	movk	x8, #13336, lsl #16
	movk	x8, #21837, lsl #32
	movk	x8, #16341, lsl #48
	fmov	d6, x8
	fadd	d1, d1, d6
	fmul	d1, d5, d1
	fadd	d0, d0, d1
	fmul	d1, d5, d4
	fmul	d2, d4, d2
	fadd	d2, d3, d2
	fmul	d1, d1, d2
	fadd	d0, d0, d1
	fmov	d1, #-1.00000000
	fdiv	d0, d1, d0
	b	LBB55_17
LBB55_11:
	cmn	w19, #1
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #16393, lsl #48
	fmov	d0, x8
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #49161, lsl #48
	b	LBB55_15
LBB55_12:
	lsr	w9, w8, #23
	cmp	w9, #254
	b.ls	LBB55_19
	fsub	s0, s0, s0
	b	LBB55_18
LBB55_14:
	cmn	w19, #1
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #16409, lsl #48
	fmov	d0, x8
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #49177, lsl #48
LBB55_15:
	fmov	d2, x8
	fcsel	d0, d2, d0, gt
	fadd	d0, d0, d1
	fmul	d1, d0, d0
	mov	x8, #7117
	movk	x8, #49047, lsl #16
	movk	x8, #25273, lsl #32
	movk	x8, #16259, lsl #48
	fmov	d2, x8
	fmul	d2, d1, d2
	mov	x8, #62542
	movk	x8, #64748, lsl #16
	movk	x8, #23981, lsl #32
	movk	x8, #16232, lsl #48
	fmov	d3, x8
	fadd	d2, d2, d3
	mov	x8, #13262
	movk	x8, #37004, lsl #16
	movk	x8, #7667, lsl #32
	movk	x8, #16281, lsl #48
	fmov	d3, x8
	fmul	d3, d1, d3
	mov	x8, #23294
	movk	x8, #7558, lsl #16
	movk	x8, #21705, lsl #32
	movk	x8, #16299, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d4, d1, d1
	fmul	d5, d0, d1
	mov	x8, #40818
	movk	x8, #14489, lsl #16
	movk	x8, #4861, lsl #32
	movk	x8, #16321, lsl #48
	fmov	d6, x8
	fmul	d1, d1, d6
	mov	x8, #51615
	movk	x8, #13336, lsl #16
	movk	x8, #21837, lsl #32
	movk	x8, #16341, lsl #48
	fmov	d6, x8
	fadd	d1, d1, d6
	fmul	d1, d5, d1
LBB55_16:
	fadd	d0, d0, d1
	fmul	d1, d5, d4
	fmul	d2, d4, d2
	fadd	d2, d3, d2
	fmul	d1, d1, d2
	fadd	d0, d0, d1
LBB55_17:
	fcvt	s0, d0
LBB55_18:
	.cfi_def_cfa wsp, 48
	ldp	x29, x30, [sp, #32]
	ldp	x20, x19, [sp, #16]
	add	sp, sp, #48
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	ret
LBB55_19:
	.cfi_restore_state
	str	xzr, [sp, #8]
	mov	w10, #4058
	movk	w10, #19913, lsl #16
	cmp	w8, w10
	b.hi	LBB55_21
	mov	x8, #51331
	movk	x8, #28105, lsl #16
	movk	x8, #24368, lsl #32
	movk	x8, #16356, lsl #48
	fmov	d0, x8
	fmul	d0, d1, d0
	mov	x8, #4843621399236968448
	fmov	d2, x8
	fadd	d0, d0, d2
	mov	x8, #-4379750637617807360
	fmov	d2, x8
	fadd	d0, d0, d2
	fcvtzs	w0, d0
	mov	x8, #1342177280
	movk	x8, #8699, lsl #32
	movk	x8, #49145, lsl #48
	fmov	d2, x8
	fmul	d2, d0, d2
	fadd	d1, d1, d2
	mov	x8, #25187
	movk	x8, #24858, lsl #16
	movk	x8, #4276, lsl #32
	movk	x8, #48721, lsl #48
	fmov	d2, x8
	fmul	d0, d0, d2
	fadd	d0, d1, d0
	b	LBB55_24
LBB55_21:
	sub	w3, w9, #150
	sub	w8, w8, w3, lsl #23
	fmov	s0, w8
	fcvt	d0, s0
	str	d0, [sp]
	mov	x0, sp
	add	x2, sp, #8
	mov	w1, #1
	mov	x4, #0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math14rem_pio2_large14rem_pio2_large
	tbnz	w19, #31, LBB55_23
	ldr	d0, [sp, #8]
	b	LBB55_24
LBB55_23:
	neg	w0, w0
	ldr	d0, [sp, #8]
	fneg	d0, d0
LBB55_24:
	fmul	d1, d0, d0
	mov	x8, #7117
	movk	x8, #49047, lsl #16
	movk	x8, #25273, lsl #32
	movk	x8, #16259, lsl #48
	fmov	d2, x8
	fmul	d2, d1, d2
	mov	x8, #62542
	movk	x8, #64748, lsl #16
	movk	x8, #23981, lsl #32
	movk	x8, #16232, lsl #48
	fmov	d3, x8
	fadd	d2, d2, d3
	mov	x8, #13262
	movk	x8, #37004, lsl #16
	movk	x8, #7667, lsl #32
	movk	x8, #16281, lsl #48
	fmov	d3, x8
	fmul	d3, d1, d3
	mov	x8, #23294
	movk	x8, #7558, lsl #16
	movk	x8, #21705, lsl #32
	movk	x8, #16299, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d4, d1, d1
	fmul	d5, d0, d1
	mov	x8, #40818
	movk	x8, #14489, lsl #16
	movk	x8, #4861, lsl #32
	movk	x8, #16321, lsl #48
	fmov	d6, x8
	fmul	d1, d1, d6
	mov	x8, #51615
	movk	x8, #13336, lsl #16
	movk	x8, #21837, lsl #32
	movk	x8, #16341, lsl #48
	fmov	d6, x8
	fadd	d1, d1, d6
	fmul	d1, d5, d1
	fadd	d0, d0, d1
	fmul	d1, d5, d4
	fmul	d2, d4, d2
	fadd	d2, d3, d2
	fmul	d1, d1, d2
	fadd	d0, d0, d1
	fmov	d1, #-1.00000000
	fdiv	d1, d1, d0
	tst	w0, #0x1
	fcsel	d0, d0, d1, eq
	b	LBB55_17
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4tanh4tanh
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math4tanh4tanh:
	.cfi_startproc
	sub	sp, sp, #48
	.cfi_def_cfa_offset 48
	stp	d9, d8, [sp, #16]
	stp	x29, x30, [sp, #32]
	add	x29, sp, #32
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset b8, -24
	.cfi_offset b9, -32
	mov.16b	v8, v0
	fabs	d0, d0
	fmov	x8, d0
	mov	x9, #-118837450113025
	movk	x9, #16353, lsl #48
	cmp	x8, x9
	b.ls	LBB56_3
	mov	x9, #-281470681743361
	movk	x9, #16436, lsl #48
	cmp	x8, x9
	b.ls	LBB56_5
	mov	x8, #-9223372036854775808
	fmov	d1, x8
	fdiv	d0, d1, d0
	fmov	d1, #1.00000000
	fadd	d0, d0, d1
	b	LBB56_9
LBB56_3:
	mov	x9, #-183966334189569
	movk	x9, #16336, lsl #48
	cmp	x8, x9
	b.ls	LBB56_6
	fadd	d0, d0, d0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5expm15expm1
	fmov	d1, #2.00000000
	fadd	d1, d0, d1
	fdiv	d0, d0, d1
	b	LBB56_9
LBB56_5:
	fadd	d0, d0, d0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5expm15expm1
	fmov	d1, #2.00000000
	fadd	d0, d0, d1
	fdiv	d0, d1, d0
	fmov	d1, #1.00000000
	fsub	d0, d1, d0
	b	LBB56_9
LBB56_6:
	lsr	x8, x8, #52
	cbz	x8, LBB56_8
	fmov	d1, #-2.00000000
	fmul	d0, d0, d1
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5expm15expm1
	fneg	d1, d0
	fmov	d2, #2.00000000
	fadd	d0, d0, d2
	fdiv	d0, d1, d0
	b	LBB56_9
LBB56_8:
	fcvt	s1, d0
	str	s1, [sp, #12]
	ldr	s1, [sp, #12]
LBB56_9:
	fmov	x8, d8
	fneg	d1, d0
	cmp	x8, #0
	fcsel	d0, d1, d0, mi
	.cfi_def_cfa wsp, 48
	ldp	x29, x30, [sp, #32]
	ldp	d9, d8, [sp, #16]
	add	sp, sp, #48
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore b8
	.cfi_restore b9
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5acosh5acosh
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5acosh5acosh:
	.cfi_startproc
	fmov	x8, d0
	ubfx	x8, x8, #52, #11
	cmp	x8, #1024
	b.hs	LBB57_2
	fmov	d1, #-1.00000000
	fadd	d0, d0, d1
	fmul	d1, d0, d0
	fadd	d2, d0, d0
	fadd	d1, d1, d2
	; InlineAsm Start
	fsqrt	d1, d1
	; InlineAsm End
	fadd	d0, d0, d1
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math5log1p5log1p
LBB57_2:
	cmp	x8, #1049
	b.hs	LBB57_4
	fadd	d1, d0, d0
	fmul	d2, d0, d0
	fmov	d3, #-1.00000000
	fadd	d2, d2, d3
	; InlineAsm Start
	fsqrt	d2, d2
	; InlineAsm End
	fadd	d0, d0, d2
	fdiv	d0, d3, d0
	fadd	d0, d1, d0
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math3log3log
LBB57_4:
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3log3log
	mov	x8, #14831
	movk	x8, #65274, lsl #16
	movk	x8, #11842, lsl #32
	movk	x8, #16358, lsl #48
	fmov	d1, x8
	fadd	d0, d0, d1
	.cfi_def_cfa wsp, 16
	ldp	x29, x30, [sp], #16
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5asinf5asinf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5asinf5asinf:
	.cfi_startproc
	fabs	s1, s0
	fmov	w8, s1
	lsr	w9, w8, #23
	cmp	w9, #126
	b.ls	LBB58_3
	mov	w9, #1065353216
	cmp	w8, w9
	fcvt	d1, s0
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #16377, lsl #48
	fmov	d2, x8
	fmul	d1, d1, d2
	mov	x8, #4066750463515557888
	fmov	d2, x8
	fadd	d1, d1, d2
	fcvt	s1, d1
	fsub	s0, s0, s0
	movi.2d	v2, #0000000000000000
	fdiv	s0, s2, s0
	fcsel	s0, s1, s0, eq
LBB58_2:
	ret
LBB58_3:
	lsr	w9, w8, #24
	cmp	w9, #63
	b.hs	LBB58_6
	sub	w8, w8, #2048, lsl #12
	lsr	w8, w8, #24
	cmp	w8, #56
	b.ls	LBB58_2
	fmul	s1, s0, s0
	mov	w8, #54123
	movk	w8, #48141, lsl #16
	fmov	s2, w8
	fmul	s2, s1, s2
	mov	w8, #5050
	movk	w8, #48431, lsl #16
	fmov	s3, w8
	fadd	s2, s2, s3
	fmul	s2, s1, s2
	mov	w8, #43637
	movk	w8, #15914, lsl #16
	fmov	s3, w8
	fadd	s2, s2, s3
	fmul	s2, s1, s2
	mov	w8, #58798
	movk	w8, #48948, lsl #16
	fmov	s3, w8
	fmul	s1, s1, s3
	fmov	s3, #1.00000000
	fadd	s1, s1, s3
	fdiv	s1, s2, s1
	fmul	s1, s0, s1
	fadd	s0, s0, s1
	ret
LBB58_6:
	fmov	w8, s0
	fmov	s0, #1.00000000
	fsub	s1, s0, s1
	fmov	s2, #0.50000000
	fmul	s1, s1, s2
	fcvt	d2, s1
	; InlineAsm Start
	fsqrt	d2, d2
	; InlineAsm End
	mov	w9, #54123
	movk	w9, #48141, lsl #16
	fmov	s3, w9
	fmul	s3, s1, s3
	mov	w9, #5050
	movk	w9, #48431, lsl #16
	fmov	s4, w9
	fadd	s3, s3, s4
	fmul	s3, s1, s3
	mov	w9, #43637
	movk	w9, #15914, lsl #16
	fmov	s4, w9
	fadd	s3, s3, s4
	fmul	s3, s1, s3
	mov	w9, #58798
	movk	w9, #48948, lsl #16
	fmov	s4, w9
	fmul	s1, s1, s4
	fadd	s0, s1, s0
	fdiv	s0, s3, s0
	fcvt	d0, s0
	fmul	d0, d2, d0
	fadd	d0, d2, d0
	fadd	d0, d0, d0
	mov	x9, #11544
	movk	x9, #21572, lsl #16
	movk	x9, #8699, lsl #32
	movk	x9, #16377, lsl #48
	fmov	d1, x9
	fsub	d0, d1, d0
	fcvt	s0, d0
	fneg	s1, s0
	cmp	w8, #0
	fcsel	s0, s1, s0, mi
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5asinh5asinh
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5asinh5asinh:
	.cfi_startproc
	sub	sp, sp, #48
	.cfi_def_cfa_offset 48
	stp	x20, x19, [sp, #16]
	stp	x29, x30, [sp, #32]
	add	x29, sp, #32
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	mov.16b	v1, v0
	fmov	x19, d1
	ubfx	x8, x19, #52, #11
	fabs	d0, d0
	cmp	x8, #1048
	b.ls	LBB59_2
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3log3log
	mov	x8, #14831
	movk	x8, #65274, lsl #16
	movk	x8, #11842, lsl #32
	movk	x8, #16358, lsl #48
	fmov	d1, x8
	fadd	d0, d0, d1
	b	LBB59_7
LBB59_2:
	cmp	x8, #1023
	b.ls	LBB59_4
	fadd	d2, d0, d0
	fmul	d1, d1, d1
	fmov	d3, #1.00000000
	fadd	d1, d1, d3
	; InlineAsm Start
	fsqrt	d1, d1
	; InlineAsm End
	fadd	d0, d0, d1
	fdiv	d0, d3, d0
	fadd	d0, d2, d0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3log3log
	b	LBB59_7
LBB59_4:
	cmp	x8, #996
	b.ls	LBB59_6
	fmul	d1, d1, d1
	fmov	d2, #1.00000000
	fadd	d3, d1, d2
	; InlineAsm Start
	fsqrt	d3, d3
	; InlineAsm End
	fadd	d2, d3, d2
	fdiv	d1, d1, d2
	fadd	d0, d0, d1
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5log1p5log1p
	b	LBB59_7
LBB59_6:
	mov	x8, #5147614374084476928
	fmov	d1, x8
	fadd	d1, d0, d1
	str	d1, [sp, #8]
	ldr	d1, [sp, #8]
LBB59_7:
	fneg	d1, d0
	cmp	x19, #0
	fcsel	d0, d1, d0, mi
	.cfi_def_cfa wsp, 48
	ldp	x29, x30, [sp, #32]
	ldp	x20, x19, [sp, #16]
	add	sp, sp, #48
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5atan25atan2
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5atan25atan2:
	.cfi_startproc
	stp	x20, x19, [sp, #-32]!
	.cfi_def_cfa_offset 32
	stp	x29, x30, [sp, #16]
	add	x29, sp, #16
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_remember_state
	fcmp	d1, d0
	b.vs	LBB60_24
	fmov	x11, d1
	lsr	x10, x11, #32
	mov	w8, #-1072693248
	add	w8, w10, w8
	orr	w8, w8, w11
	cbz	w8, LBB60_7
	fmov	x12, d0
	lsr	x8, x12, #63
	lsr	w9, w10, #30
	and	w9, w9, #0x2
	orr	w19, w9, w8
	ubfx	x8, x12, #32, #31
	orr	w12, w8, w12
	cbz	w12, LBB60_8
	and	w10, w10, #0x7fffffff
	orr	w11, w10, w11
	cbz	w11, LBB60_16
	mov	w11, #2146435072
	cmp	w10, w11
	b.ne	LBB60_9
	cmp	w8, w11
	b.ne	LBB60_17
Lloh432:
	adrp	x8, l_switch.table._RNvNtNtCsdtCTLUYU3dx_4libm4math5atan25atan2@PAGE
Lloh433:
	add	x8, x8, l_switch.table._RNvNtNtCsdtCTLUYU3dx_4libm4math5atan25atan2@PAGEOFF
	b	LBB60_18
LBB60_7:
	.cfi_def_cfa wsp, 32
	ldp	x29, x30, [sp, #16]
	ldp	x20, x19, [sp], #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math4atan4atan
LBB60_8:
	.cfi_restore_state
	.cfi_remember_state
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #49161, lsl #48
	fmov	d1, x8
	cmp	w19, #2
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #16393, lsl #48
	fmov	d2, x8
	fcsel	d1, d2, d1, eq
	fcsel	d0, d0, d1, lo
	b	LBB60_19
LBB60_9:
	cmp	w8, w11
	b.eq	LBB60_16
	mov	w11, #67108864
	add	w11, w10, w11
	cmp	w11, w8
	b.lo	LBB60_16
	cbz	w9, LBB60_13
	mov	w9, #67108864
	movi.2d	v2, #0000000000000000
	add	w8, w8, w9
	cmp	w8, w10
	b.lo	LBB60_14
LBB60_13:
	fdiv	d0, d0, d1
	fabs	d0, d0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4atan4atan
	mov.16b	v2, v0
LBB60_14:
	cmp	w19, #1
	b.gt	LBB60_21
	fneg	d0, d2
	cmp	w19, #0
	fcsel	d0, d2, d0, eq
	b	LBB60_19
LBB60_16:
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #16377, lsl #48
	fmov	d1, x8
	movi.2d	v2, #0xffffffffffffffff
	fneg.2d	v2, v2
	bit.16b	v0, v1, v2
	.cfi_def_cfa wsp, 32
	b	LBB60_20
LBB60_17:
	.cfi_restore_state
	.cfi_remember_state
Lloh434:
	adrp	x8, l_switch.table._RNvNtNtCsdtCTLUYU3dx_4libm4math5atan25atan2.49@PAGE
Lloh435:
	add	x8, x8, l_switch.table._RNvNtNtCsdtCTLUYU3dx_4libm4math5atan25atan2.49@PAGEOFF
LBB60_18:
	ldr	d0, [x8, w19, uxtw #3]
LBB60_19:
	.cfi_def_cfa wsp, 32
LBB60_20:
	ldp	x29, x30, [sp, #16]
	ldp	x20, x19, [sp], #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	ret
LBB60_21:
	.cfi_restore_state
	mov	x8, #23559
	movk	x8, #13076, lsl #16
	movk	x8, #42534, lsl #32
	movk	x8, #48289, lsl #48
	fmov	d0, x8
	fadd	d0, d2, d0
	cmp	w19, #2
	b.ne	LBB60_23
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #16393, lsl #48
	fmov	d1, x8
	fsub	d0, d1, d0
	b	LBB60_19
LBB60_23:
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #49161, lsl #48
	fmov	d1, x8
	fadd	d0, d0, d1
	b	LBB60_19
LBB60_24:
	fadd	d0, d0, d1
	b	LBB60_19
	.loh AdrpAdd	Lloh432, Lloh433
	.loh AdrpAdd	Lloh434, Lloh435
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5atanf5atanf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5atanf5atanf:
	.cfi_startproc
	fmov	w8, s0
	fabs	s1, s0
	fmov	w10, s1
	lsr	w11, w10, #23
	cmp	w11, #152
	b.ls	LBB61_3
	fcmp	s0, s0
	b.vs	LBB61_18
	cmn	w8, #1
	mov	w8, #4058
	movk	w8, #49097, lsl #16
	fmov	s0, w8
	mov	w8, #4058
	movk	w8, #16329, lsl #16
	fmov	s1, w8
	fcsel	s0, s1, s0, gt
	ret
LBB61_3:
	lsr	w9, w10, #21
	cmp	w9, #503
	b.hs	LBB61_7
	cmp	w11, #115
	b.hs	LBB61_10
	cmp	w10, #2048, lsl #12
	b.hs	LBB61_18
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	fmul	s1, s0, s0
	str	s1, [sp, #12]
	ldr	s1, [sp, #12]
	add	sp, sp, #16
	.cfi_def_cfa_offset 0
	ret
LBB61_7:
	lsr	w11, w10, #19
	cmp	w11, #2035
	b.hs	LBB61_11
	lsr	w10, w10, #20
	cmp	w10, #1011
	b.hs	LBB61_13
	mov	x10, #0
	fadd	s0, s1, s1
	fmov	s2, #-1.00000000
	fadd	s0, s0, s2
	fmov	s2, #2.00000000
	fadd	s1, s1, s2
	b	LBB61_15
LBB61_10:
	mov	w10, #-1
	b	LBB61_16
LBB61_11:
	mov	w11, #1075576832
	cmp	w10, w11
	b.hs	LBB61_14
	fmov	s0, #-1.50000000
	fadd	s0, s1, s0
	fmov	s2, #1.50000000
	fmul	s1, s1, s2
	fmov	s2, #1.00000000
	fadd	s1, s1, s2
	fdiv	s0, s0, s1
	mov	w10, #2
	b	LBB61_16
LBB61_13:
	fmov	s0, #-1.00000000
	fadd	s0, s1, s0
	fmov	s2, #1.00000000
	fadd	s1, s1, s2
	mov	w10, #1
	b	LBB61_15
LBB61_14:
	fmov	s0, #-1.00000000
	mov	w10, #3
LBB61_15:
	fdiv	s0, s0, s1
LBB61_16:
	fmul	s1, s0, s0
	fmul	s2, s1, s1
	mov	w11, #44069
	movk	w11, #15740, lsl #16
	fmov	s3, w11
	fmul	s3, s2, s3
	mov	w11, #62733
	movk	w11, #15889, lsl #16
	fmov	s4, w11
	fadd	s3, s3, s4
	fmul	s3, s2, s3
	mov	w11, #43689
	movk	w11, #16042, lsl #16
	fmov	s4, w11
	fadd	s3, s3, s4
	fmul	s1, s1, s3
	mov	w11, #4679
	movk	w11, #48602, lsl #16
	fmov	s3, w11
	fmul	s3, s2, s3
	mov	w11, #51864
	movk	w11, #48716, lsl #16
	fmov	s4, w11
	fadd	s3, s3, s4
	fmul	s2, s2, s3
	cmp	w9, #503
	b.hs	LBB61_19
	fadd	s1, s2, s1
	fmul	s1, s0, s1
	fsub	s0, s0, s1
LBB61_18:
	ret
LBB61_19:
Lloh436:
	adrp	x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.91@PAGE
Lloh437:
	add	x9, x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.91@PAGEOFF
	ldr	s3, [x9, x10, lsl #2]
	fadd	s1, s2, s1
	fmul	s1, s0, s1
Lloh438:
	adrp	x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.92@PAGE
Lloh439:
	add	x9, x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.92@PAGEOFF
	ldr	s2, [x9, x10, lsl #2]
	fsub	s1, s1, s2
	fsub	s0, s1, s0
	fsub	s0, s3, s0
	fneg	s1, s0
	cmn	w8, #1
	fcsel	s0, s0, s1, gt
	ret
	.loh AdrpAdd	Lloh438, Lloh439
	.loh AdrpAdd	Lloh436, Lloh437
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5atanh5atanh
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5atanh5atanh:
	.cfi_startproc
	sub	sp, sp, #48
	.cfi_def_cfa_offset 48
	stp	x20, x19, [sp, #16]
	stp	x29, x30, [sp, #32]
	add	x29, sp, #32
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	fabs	d1, d0
	fmov	x19, d0
	ubfx	x8, x19, #52, #11
	cmp	x8, #1022
	b.hs	LBB62_4
	cmp	x8, #991
	b.hs	LBB62_5
	cbnz	x8, LBB62_7
	fcvt	s0, d1
	str	s0, [sp, #12]
	ldr	s0, [sp, #12]
	b	LBB62_7
LBB62_4:
	fmov	d0, #1.00000000
	fsub	d0, d0, d1
	fdiv	d0, d1, d0
	fadd	d0, d0, d0
	b	LBB62_6
LBB62_5:
	fadd	d0, d1, d1
	fmul	d2, d1, d0
	fmov	d3, #1.00000000
	fsub	d1, d3, d1
	fdiv	d1, d2, d1
	fadd	d0, d0, d1
LBB62_6:
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5log1p5log1p
	fmov	d1, #0.50000000
	fmul	d1, d0, d1
LBB62_7:
	fneg	d0, d1
	cmp	x19, #0
	fcsel	d0, d0, d1, mi
	.cfi_def_cfa wsp, 48
	ldp	x29, x30, [sp, #32]
	ldp	x20, x19, [sp, #16]
	add	sp, sp, #48
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5coshf5coshf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5coshf5coshf:
	.cfi_startproc
	sub	sp, sp, #32
	.cfi_def_cfa_offset 32
	stp	x29, x30, [sp, #16]
	add	x29, sp, #16
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	fabs	s0, s0
	fmov	w8, s0
	mov	w9, #29207
	movk	w9, #16177, lsl #16
	cmp	w8, w9
	b.hs	LBB63_3
	lsr	w8, w8, #23
	cmp	w8, #115
	b.hs	LBB63_5
	mov	w8, #2071986176
	fmov	s1, w8
	fadd	s0, s0, s1
	stur	s0, [x29, #-4]
	ldur	s0, [x29, #-4]
	fmov	s0, #1.00000000
	b	LBB63_7
LBB63_3:
	mov	w9, #29207
	movk	w9, #17073, lsl #16
	cmp	w8, w9
	b.hs	LBB63_6
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4expf4expf
	fmov	s1, #1.00000000
	fdiv	s1, s1, s0
	fadd	s0, s0, s1
	fmov	s1, #0.50000000
	fmul	s0, s0, s1
	b	LBB63_7
LBB63_5:
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math6expm1f6expm1f
	fmul	s1, s0, s0
	fmov	s2, #1.00000000
	fadd	s0, s0, s2
	fadd	s0, s0, s0
	fdiv	s0, s1, s0
	fadd	s0, s0, s2
	b	LBB63_7
LBB63_6:
	mov	w8, #58300
	movk	w8, #49954, lsl #16
	fmov	s1, w8
	fadd	s0, s0, s1
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4expf4expf
	movi.2s	v1, #122, lsl #24
	fmul	s0, s0, s1
	fmul	s0, s0, s1
LBB63_7:
	.cfi_def_cfa wsp, 32
	ldp	x29, x30, [sp, #16]
	add	sp, sp, #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5exp105exp10
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5exp105exp10:
	.cfi_startproc
	stp	d9, d8, [sp, #-32]!
	.cfi_def_cfa_offset 32
	stp	x29, x30, [sp, #16]
	add	x29, sp, #16
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset b8, -24
	.cfi_offset b9, -32
	.cfi_remember_state
	mov.16b	v1, v0
	fmov	x8, d1
	ubfx	x10, x8, #52, #11
	sub	w9, w10, #1023
	cmp	w10, #1074
	b.ls	LBB64_3
	tst	x8, #0xfffffffffffff
	b.eq	LBB64_10
	mov.16b	v8, v1
	mov.16b	v0, v1
	cmp	w9, #1024
	b.eq	LBB64_7
	b	LBB64_10
LBB64_3:
	cmp	w10, #1023
	b.hs	LBB64_5
	and	x8, x8, #0x8000000000000000
	fmov	d8, x8
	mov.16b	v0, v1
	fmov	x8, d8
	ubfx	x8, x8, #52, #11
	cmp	x8, #1027
	b.lo	LBB64_8
	b	LBB64_11
LBB64_5:
	lsl	x10, x8, x9
	tst	x10, #0xfffffffffffff
	b.eq	LBB64_10
	mov	x10, #-4503599627370496
	asr	x9, x10, x9
	and	x8, x9, x8
	fmov	d8, x8
	fsub	d0, d1, d8
LBB64_7:
	fmov	x8, d8
	ubfx	x8, x8, #52, #11
	cmp	x8, #1027
	b.hs	LBB64_11
LBB64_8:
	fcmp	d0, #0.0
	b.ne	LBB64_12
	fcvtzs	x8, d8
Lloh440:
	adrp	x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.93@PAGE
Lloh441:
	add	x9, x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.93@PAGEOFF
	add	x8, x9, x8, lsl #3
	ldr	d0, [x8, #120]
	b	LBB64_13
LBB64_10:
	and	x8, x8, #0x8000000000000000
	fmov	d0, x8
	mov.16b	v8, v1
	fmov	x8, d8
	ubfx	x8, x8, #52, #11
	cmp	x8, #1027
	b.lo	LBB64_8
LBB64_11:
	fmov	d0, #10.00000000
	.cfi_def_cfa wsp, 32
	ldp	x29, x30, [sp, #16]
	ldp	d9, d8, [sp], #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore b8
	.cfi_restore b9
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math3pow3pow
LBB64_12:
	.cfi_restore_state
	mov	x8, #41841
	movk	x8, #2425, lsl #16
	movk	x8, #37711, lsl #32
	movk	x8, #16394, lsl #48
	fmov	d1, x8
	fmul	d0, d0, d1
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4exp24exp2
	fcvtzs	x8, d8
Lloh442:
	adrp	x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.93@PAGE
Lloh443:
	add	x9, x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.93@PAGEOFF
	add	x8, x9, x8, lsl #3
	ldr	d1, [x8, #120]
	fmul	d0, d0, d1
LBB64_13:
	.cfi_def_cfa wsp, 32
	ldp	x29, x30, [sp, #16]
	ldp	d9, d8, [sp], #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore b8
	.cfi_restore b9
	ret
	.loh AdrpAdd	Lloh440, Lloh441
	.loh AdrpAdd	Lloh442, Lloh443
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5exp2f5exp2f
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5exp2f5exp2f:
	.cfi_startproc
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	.cfi_remember_state
	fmov	w8, s0
	and	w9, w8, #0x7fffffff
	mov	w10, #1123811328
	cmp	w9, w10
	b.ls	LBB65_4
	mov	w10, #2139095040
	cmp	w9, w10
	b.hi	LBB65_8
	mov	w9, #1124073471
	cmp	w8, w9
	b.le	LBB65_6
	movi.2s	v1, #127, lsl #24
	fmul	s0, s0, s1
	b	LBB65_8
LBB65_4:
	mov	w8, #855638016
	cmp	w9, w8
	b.hi	LBB65_7
	fmov	s1, #1.00000000
	fadd	s0, s0, s1
	b	LBB65_8
LBB65_6:
	tbnz	w8, #31, LBB65_9
LBB65_7:
	mov	w8, #1228931072
	fmov	s1, w8
	fadd	s1, s0, s1
	mov	w8, #-918552576
	fmov	s2, w8
	fadd	s2, s1, s2
	fmov	w8, s1
	add	w8, w8, #8
	lsr	w9, w8, #4
	add	w9, w9, #1023
	lsl	x9, x9, #52
	fmov	d1, x9
	and	w8, w8, #0xf
	fsub	s0, s0, s2
Lloh444:
	adrp	x9, __RNvNtNtCsdtCTLUYU3dx_4libm4math5exp2f6EXP2FT@PAGE
Lloh445:
	add	x9, x9, __RNvNtNtCsdtCTLUYU3dx_4libm4math5exp2f6EXP2FT@PAGEOFF
	fcvt	d0, s0
	ldr	d2, [x9, w8, uxtw #3]
	fmul	d3, d2, d0
	mov	x8, #210822764691456
	movk	x8, #16334, lsl #48
	fmov	d4, x8
	fmul	d4, d0, d4
	mov	x8, #50865297686528
	movk	x8, #16358, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d4, d4, d3
	fadd	d2, d2, d4
	fmul	d4, d0, d0
	fmul	d3, d4, d3
	mov	x8, #3221225472
	movk	x8, #45769, lsl #32
	movk	x8, #16259, lsl #48
	fmov	d4, x8
	fmul	d0, d0, d4
	mov	x8, #2147483648
	movk	x8, #27444, lsl #32
	movk	x8, #16300, lsl #48
	fmov	d4, x8
	fadd	d0, d0, d4
	fmul	d0, d0, d3
	fadd	d0, d2, d0
	fmul	d0, d0, d1
	fcvt	s0, d0
LBB65_8:
	add	sp, sp, #16
	.cfi_def_cfa_offset 0
	ret
LBB65_9:
	.cfi_restore_state
	mov	w9, #-1021968385
	cmp	w8, w9
	b.ls	LBB65_11
	mov	w8, #-2147483647
	fmov	s1, w8
	fdiv	s0, s1, s0
	str	s0, [sp, #12]
	ldr	s0, [sp, #12]
	movi.2d	v0, #0000000000000000
	b	LBB65_8
LBB65_11:
	tst	w8, #0xffff
	b.eq	LBB65_7
	mov	w8, #-2147483647
	fmov	s1, w8
	fdiv	s1, s1, s0
	str	s1, [sp, #12]
	ldr	s1, [sp, #12]
	b	LBB65_7
	.loh AdrpAdd	Lloh444, Lloh445
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5expm15expm1
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5expm15expm1:
	.cfi_startproc
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	str	d0, [sp, #8]
	fmov	x8, d0
	ubfx	x9, x8, #32, #31
	mov	w10, #26745
	movk	w10, #16451, lsl #16
	cmp	w9, w10
	b.ls	LBB66_5
	fcmp	d0, d0
	b.vs	LBB66_27
	tbnz	x8, #63, LBB66_13
	mov	x8, #14831
	movk	x8, #65274, lsl #16
	movk	x8, #11842, lsl #32
	movk	x8, #16518, lsl #48
	fmov	d1, x8
	fcmp	d0, d1
	b.le	LBB66_8
	mov	x8, #9214364837600034816
	fmov	d1, x8
	fmul	d0, d0, d1
	b	LBB66_27
LBB66_5:
	mov	w10, #11842
	movk	w10, #16342, lsl #16
	cmp	w9, w10
	b.ls	LBB66_10
	mov	w10, #41650
	movk	w10, #16368, lsl #16
	cmp	w9, w10
	b.hs	LBB66_8
	mov	x9, #4276092928
	movk	x9, #11842, lsl #32
	movk	x9, #49126, lsl #48
	fmov	d1, x9
	fadd	d1, d0, d1
	mov	x9, #15478
	movk	x9, #13689, lsl #16
	movk	x9, #14831, lsl #32
	movk	x9, #15850, lsl #48
	fmov	d2, x9
	mov	x9, #4276092928
	movk	x9, #11842, lsl #32
	movk	x9, #16358, lsl #48
	fmov	d3, x9
	fadd	d0, d0, d3
	mov	w9, #-1
	mov	x10, #15478
	movk	x10, #13689, lsl #16
	movk	x10, #14831, lsl #32
	movk	x10, #48618, lsl #48
	fmov	d3, x10
	tst	x8, #0x8000000000000000
	csinc	w8, w9, wzr, ne
	fcsel	d2, d2, d3, eq
	fcsel	d1, d1, d0, eq
	b	LBB66_9
LBB66_8:
	mov	x8, #33534
	movk	x8, #25899, lsl #16
	movk	x8, #5447, lsl #32
	movk	x8, #16375, lsl #48
	fmov	d1, x8
	fmul	d1, d0, d1
	fmov	d2, #0.50000000
	movi.2d	v3, #0xffffffffffffffff
	fneg.2d	v3, v3
	bif.16b	v2, v0, v3
	fadd	d1, d1, d2
	fcvtzs	w8, d1
	scvtf	d2, w8
	mov	x9, #4276092928
	movk	x9, #11842, lsl #32
	movk	x9, #49126, lsl #48
	fmov	d1, x9
	fmul	d1, d2, d1
	fadd	d1, d0, d1
	mov	x9, #15478
	movk	x9, #13689, lsl #16
	movk	x9, #14831, lsl #32
	movk	x9, #15850, lsl #48
	fmov	d0, x9
	fmul	d2, d2, d0
LBB66_9:
	fsub	d0, d1, d2
	fsub	d1, d1, d0
	fsub	d1, d1, d2
	b	LBB66_15
LBB66_10:
	lsr	w8, w9, #20
	cmp	w8, #969
	b.hs	LBB66_14
	cmp	w9, #256, lsl #12
	b.hs	LBB66_27
	ldr	d1, [sp, #8]
	b	LBB66_27
LBB66_13:
	fmov	d0, #-1.00000000
	b	LBB66_27
LBB66_14:
	mov	w8, #0
	movi.2d	v1, #0000000000000000
LBB66_15:
	fmov	d2, #0.50000000
	fmul	d3, d0, d2
	fmul	d2, d0, d3
	mov	x9, #49965
	movk	x9, #28169, lsl #16
	movk	x9, #64951, lsl #32
	movk	x9, #48778, lsl #48
	fmov	d4, x9
	fmul	d4, d2, d4
	mov	x9, #21049
	movk	x9, #34534, lsl #16
	movk	x9, #53194, lsl #32
	movk	x9, #16080, lsl #48
	fmov	d5, x9
	fadd	d4, d4, d5
	fmul	d4, d2, d4
	mov	x9, #56247
	movk	x9, #40618, lsl #16
	movk	x9, #52761, lsl #32
	movk	x9, #48916, lsl #48
	fmov	d5, x9
	fadd	d4, d4, d5
	fmul	d4, d2, d4
	mov	x9, #21893
	movk	x9, #6654, lsl #16
	movk	x9, #416, lsl #32
	movk	x9, #16218, lsl #48
	fmov	d5, x9
	fadd	d4, d4, d5
	fmul	d4, d2, d4
	mov	x9, #1229782938247303441
	movk	x9, #4340
	movk	x9, #49057, lsl #48
	fmov	d5, x9
	fadd	d4, d4, d5
	fmul	d4, d2, d4
	fmov	d5, #1.00000000
	fadd	d4, d4, d5
	fmul	d3, d3, d4
	fmov	d5, #3.00000000
	fsub	d3, d5, d3
	fsub	d4, d4, d3
	fmul	d3, d0, d3
	fmov	d5, #6.00000000
	fsub	d3, d5, d3
	fdiv	d3, d4, d3
	fmul	d3, d2, d3
	cbz	w8, LBB66_19
	fsub	d3, d3, d1
	fmul	d3, d0, d3
	fsub	d1, d3, d1
	fsub	d1, d1, d2
	cmp	w8, #1
	b.eq	LBB66_20
	cmn	w8, #1
	b.ne	LBB66_22
	fsub	d0, d0, d1
	fmov	d1, #0.50000000
	fmul	d0, d0, d1
	fmov	d1, #-0.50000000
	b	LBB66_26
LBB66_19:
	fmul	d1, d0, d3
	fsub	d1, d1, d2
	fsub	d0, d0, d1
	b	LBB66_27
LBB66_20:
	fmov	d2, #-0.25000000
	fcmp	d0, d2
	b.pl	LBB66_24
	fmov	d2, #0.50000000
	fadd	d0, d0, d2
	fsub	d0, d1, d0
	fmov	d1, #-2.00000000
	fmul	d0, d0, d1
	b	LBB66_27
LBB66_22:
	add	w9, w8, #1023
	lsl	x9, x9, #52
	fmov	d2, x9
	cmp	w8, #57
	b.hs	LBB66_25
	mov	w9, #1023
	sub	w9, w9, w8
	lsl	x9, x9, #52
	fmov	d3, x9
	cmp	w8, #20
	fsub	d4, d0, d1
	fmov	d5, #1.00000000
	fsub	d6, d5, d3
	fadd	d4, d6, d4
	fadd	d1, d1, d3
	fsub	d0, d0, d1
	fadd	d0, d0, d5
	fcsel	d0, d4, d0, lo
	fmul	d0, d0, d2
	b	LBB66_27
LBB66_24:
	fsub	d0, d0, d1
	fadd	d0, d0, d0
	fmov	d1, #1.00000000
	b	LBB66_26
LBB66_25:
	fsub	d0, d0, d1
	fmov	d1, #1.00000000
	fadd	d0, d0, d1
	fadd	d1, d0, d0
	mov	x9, #9214364837600034816
	fmov	d3, x9
	fmul	d1, d1, d3
	fmul	d0, d0, d2
	cmp	w8, #1024
	fcsel	d0, d1, d0, eq
	fmov	d1, #-1.00000000
LBB66_26:
	fadd	d0, d0, d1
LBB66_27:
	add	sp, sp, #16
	.cfi_def_cfa_offset 0
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5floor5floor
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5floor5floor:
	.cfi_startproc
	fmov	x8, d0
	ubfx	x9, x8, #52, #11
	cmp	w9, #1074
	b.hi	LBB67_4
	cmp	w9, #1022
	b.ls	LBB67_5
	sub	w9, w9, #1023
	mov	x11, #4503599627370495
	lsr	x10, x11, x9
	tst	x10, x8
	b.eq	LBB67_4
	and	x10, x10, x8, asr #63
	mov	x11, #-4503599627370496
	asr	x9, x11, x9
	add	x8, x10, x8
	and	x8, x8, x9
	fmov	d0, x8
LBB67_4:
	ret
LBB67_5:
	movi.2d	v1, #0000000000000000
	fcmp	d0, #0.0
	fmov	d2, #-1.00000000
	fcsel	d0, d0, d2, eq
	tst	x8, #0x8000000000000000
	fcsel	d0, d1, d0, eq
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5floor6floorf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5floor6floorf:
	.cfi_startproc
	fmov	w8, s0
	ubfx	w9, w8, #23, #8
	cmp	w9, #149
	b.hi	LBB68_4
	cmp	w9, #126
	b.ls	LBB68_5
	sub	w9, w9, #127
	mov	w10, #8388607
	lsr	w10, w10, w9
	tst	w10, w8
	b.eq	LBB68_4
	and	w10, w10, w8, asr #31
	mov	w11, #-8388608
	asr	w9, w11, w9
	add	w8, w10, w8
	and	w8, w8, w9
	fmov	s0, w8
LBB68_4:
	ret
LBB68_5:
	movi.2d	v1, #0000000000000000
	fcmp	s0, #0.0
	fmov	s2, #-1.00000000
	fcsel	s0, s0, s2, eq
	tst	w8, #0x80000000
	fcsel	s0, s1, s0, eq
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5frexp5frexp
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5frexp5frexp:
	.cfi_startproc
	fmov	x8, d0
	ubfx	x9, x8, #52, #11
	cmp	w9, #2047
	b.eq	LBB69_4
	cbnz	w9, LBB69_5
	fcmp	d0, #0.0
	b.eq	LBB69_4
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	mov	x8, #4895412794951729152
	fmov	d1, x8
	fmul	d0, d0, d1
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5frexp5frexp
	sub	w0, w0, #64
	.cfi_def_cfa wsp, 16
	ldp	x29, x30, [sp], #16
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
LBB69_4:
	mov	w0, #0
	ret
LBB69_5:
	sub	w0, w9, #1022
	and	x8, x8, #0x800fffffffffffff
	orr	x8, x8, #0x3fe0000000000000
	fmov	d0, x8
	ret
	.cfi_endproc

	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5k_tan5k_tan:
	.cfi_startproc
	fmov	x8, d0
	and	x9, x8, #0x7ffffff800000000
	mov	x10, #162895224635392
	movk	x10, #16357, lsl #48
	cmp	x9, x10
	b.ls	LBB70_2
	fneg	d2, d1
	cmp	x8, #0
	fcsel	d1, d2, d1, mi
	fabs	d0, d0
	mov	x11, #11544
	movk	x11, #21572, lsl #16
	movk	x11, #8699, lsl #32
	movk	x11, #16361, lsl #48
	fmov	d2, x11
	fsub	d0, d2, d0
	mov	x11, #23559
	movk	x11, #13076, lsl #16
	movk	x11, #42534, lsl #32
	movk	x11, #15489, lsl #48
	fmov	d2, x11
	fsub	d1, d2, d1
	fadd	d0, d0, d1
	movi.2d	v1, #0000000000000000
LBB70_2:
	fmul	d2, d0, d0
	fmul	d3, d2, d2
	mov	x11, #21363
	movk	x11, #56160, lsl #16
	movk	x11, #30155, lsl #32
	movk	x11, #48883, lsl #48
	fmov	d4, x11
	fmul	d4, d3, d4
	mov	x11, #37542
	movk	x11, #41015, lsl #16
	movk	x11, #32392, lsl #32
	movk	x11, #16148, lsl #48
	fmov	d5, x11
	fadd	d4, d4, d5
	fmul	d4, d3, d4
	mov	x11, #25857
	movk	x11, #62194, lsl #16
	movk	x11, #17624, lsl #32
	movk	x11, #16195, lsl #48
	fmov	d5, x11
	fadd	d4, d4, d5
	fmul	d4, d3, d4
	mov	x11, #808
	movk	x11, #51542, lsl #16
	movk	x11, #27938, lsl #32
	movk	x11, #16237, lsl #48
	fmov	d5, x11
	fadd	d4, d4, d5
	fmul	d4, d3, d4
	mov	x11, #54839
	movk	x11, #33798, lsl #16
	movk	x11, #25844, lsl #32
	movk	x11, #16278, lsl #48
	fmov	d5, x11
	fadd	d4, d4, d5
	fmul	d4, d3, d4
	mov	x11, #65146
	movk	x11, #4368, lsl #16
	movk	x11, #4369, lsl #32
	movk	x11, #16321, lsl #48
	fmov	d5, x11
	fadd	d4, d4, d5
	mov	x11, #31444
	movk	x11, #29887, lsl #16
	movk	x11, #10864, lsl #32
	movk	x11, #16123, lsl #48
	fmov	d5, x11
	fmul	d5, d3, d5
	mov	x11, #42985
	movk	x11, #13040, lsl #16
	movk	x11, #47119, lsl #32
	movk	x11, #16146, lsl #48
	fmov	d6, x11
	fadd	d5, d5, d6
	fmul	d5, d3, d5
	mov	x11, #4200
	movk	x11, #6797, lsl #16
	movk	x11, #9975, lsl #32
	movk	x11, #16176, lsl #48
	fmov	d6, x11
	fadd	d5, d5, d6
	fmul	d5, d3, d5
	mov	x11, #33557
	movk	x11, #65248, lsl #16
	movk	x11, #56264, lsl #32
	movk	x11, #16215, lsl #48
	fmov	d6, x11
	fadd	d5, d5, d6
	fmul	d5, d3, d5
	mov	x11, #33939
	movk	x11, #59758, lsl #16
	movk	x11, #9955, lsl #32
	movk	x11, #16258, lsl #48
	fmov	d6, x11
	fadd	d5, d5, d6
	fmul	d3, d3, d5
	mov	x11, #16894
	movk	x11, #7091, lsl #16
	movk	x11, #41402, lsl #32
	movk	x11, #16299, lsl #48
	fmov	d5, x11
	fadd	d3, d3, d5
	fmul	d3, d2, d3
	fmul	d5, d0, d2
	fadd	d3, d4, d3
	fmul	d3, d5, d3
	fadd	d3, d1, d3
	fmul	d2, d2, d3
	fadd	d1, d1, d2
	mov	x11, #6148914691236517205
	movk	x11, #21859
	movk	x11, #16341, lsl #48
	fmov	d2, x11
	fmul	d2, d5, d2
	fadd	d2, d2, d1
	fadd	d1, d0, d2
	cmp	x9, x10
	b.ls	LBB70_4
	lsl	w9, w0, #1
	ucvtf	d3, w9
	fmov	d4, #1.00000000
	fsub	d3, d4, d3
	fmul	d4, d1, d1
	fadd	d1, d3, d1
	fdiv	d1, d4, d1
	fsub	d1, d2, d1
	fadd	d0, d0, d1
	fadd	d0, d0, d0
	fsub	d0, d3, d0
	fneg	d1, d0
	cmp	x8, #0
	fcsel	d1, d1, d0, mi
	mov.16b	v0, v1
	ret
LBB70_4:
	cbz	w0, LBB70_6
	fmov	d3, #-1.00000000
	fdiv	d3, d3, d1
	mov.d	v1[1], v3[0]
	movi.2d	v4, #0xffffffff00000000
	and.16b	v1, v1, v4
	fsub	d0, d1, d0
	fsub	d0, d2, d0
	mov	d2, v1[1]
	fmul.d	d4, d1, v1[1]
	fmov	d5, #1.00000000
	fadd	d4, d4, d5
	fmul.d	d0, d0, v1[1]
	fadd	d0, d4, d0
	fmul	d0, d3, d0
	fadd	d1, d0, d2
LBB70_6:
	mov.16b	v0, v1
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5ldexp5ldexp
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5ldexp5ldexp:
	.cfi_startproc
	subs	w8, w0, #1023
	b.le	LBB71_4
	mov	x9, #9214364837600034816
	fmov	d1, x9
	fmul	d0, d0, d1
	cmp	w0, #2046
	b.ls	LBB71_7
	mov	x8, #9214364837600034816
	fmov	d1, x8
	fmul	d0, d0, d1
	mov	w8, #3069
	cmp	w0, #3069
	csel	w8, w0, w8, lo
	sub	w0, w8, #2046
LBB71_3:
	add	w8, w0, #1023
	lsl	x8, x8, #52
	fmov	d1, x8
	fmul	d0, d0, d1
	ret
LBB71_4:
	cmn	w0, #1022
	b.ge	LBB71_3
	mov	x8, #243194379878006784
	fmov	d1, x8
	fmul	d0, d0, d1
	cmn	w0, #1992
	b.hi	LBB71_8
	fmov	d1, x8
	fmul	d0, d0, d1
	cmn	w0, #2960
	mov	w8, #-2960
	csel	w8, w0, w8, hi
	add	w0, w8, #1938
	add	w8, w0, #1023
	lsl	x8, x8, #52
	fmov	d1, x8
	fmul	d0, d0, d1
	ret
LBB71_7:
	mov	x0, x8
	add	w8, w0, #1023
	lsl	x8, x8, #52
	fmov	d1, x8
	fmul	d0, d0, d1
	ret
LBB71_8:
	add	w0, w0, #969
	add	w8, w0, #1023
	lsl	x8, x8, #52
	fmov	d1, x8
	fmul	d0, d0, d1
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5ldexp6ldexpf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5ldexp6ldexpf:
	.cfi_startproc
	subs	w8, w0, #127
	b.le	LBB72_3
	movi.2s	v1, #127, lsl #24
	fmul	s0, s0, s1
	fmul	s1, s0, s1
	mov	w9, #381
	cmp	w0, #381
	csel	w9, w0, w9, lo
	sub	w9, w9, #254
	cmp	w0, #254
	csel	w0, w9, w8, hi
	fcsel	s0, s1, s0, hi
LBB72_2:
	mov	w8, #1065353216
	add	w8, w8, w0, lsl #23
	and	w8, w8, #0x7f800000
	fmov	s1, w8
	fmul	s0, s0, s1
	ret
LBB72_3:
	cmn	w0, #126
	b.ge	LBB72_2
	mov	w8, #209715200
	fmov	s1, w8
	fmul	s0, s0, s1
	add	w8, w0, #102
	mov	w9, #209715200
	fmov	s1, w9
	fmul	s1, s0, s1
	cmn	w0, #330
	mov	w9, #-330
	csel	w9, w0, w9, hi
	add	w9, w9, #204
	cmn	w0, #229
	fcsel	s0, s0, s1, hi
	csel	w0, w8, w9, hi
	mov	w8, #1065353216
	add	w8, w8, w0, lsl #23
	and	w8, w8, #0x7f800000
	fmov	s1, w8
	fmul	s0, s0, s1
	ret
	.cfi_endproc

	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0
lCPI73_0:
	.quad	0x3fcc71c51d8e78af
	.quad	0x3fd2492494229359
lCPI73_1:
	.quad	0x3fd999999997fa04
	.quad	0x3fe5555555555593
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5log105log10
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5log105log10:
	.cfi_startproc
	fmov	x8, d0
	mov	x9, #4503599627370496
	cmp	x8, x9
	b.lt	LBB73_5
	lsr	x9, x8, #52
	cmp	x9, #2046
	b.hi	LBB73_12
	lsr	x9, x8, #32
	mov	w10, #1072693248
	cmp	x9, x10
	b.ne	LBB73_9
	cbz	w8, LBB73_13
	mov	w10, #-1023
	mov	w9, #1072693248
	b	LBB73_10
LBB73_5:
	fcmp	d0, #0.0
	b.ne	LBB73_7
	fmul	d0, d0, d0
	fmov	d1, #-1.00000000
	fdiv	d0, d1, d0
	ret
LBB73_7:
	tbnz	x8, #63, LBB73_11
	mov	x8, #4850376798678024192
	fmov	d1, x8
	fmul	d0, d0, d1
	fmov	x8, d0
	lsr	x9, x8, #32
	mov	w10, #-1077
	b	LBB73_10
LBB73_9:
	mov	w10, #-1023
LBB73_10:
	add	w9, w9, #149, lsl #12
	add	w11, w9, #3938
	add	w9, w10, w11, lsr #20
	and	w10, w11, #0xfffff
	mov	w11, #41118
	movk	w11, #16358, lsl #16
	add	w10, w10, w11
	mov	w8, w8
	orr	x8, x8, x10, lsl #32
	fmov	d0, x8
	fmov	d1, #-1.00000000
	fadd	d0, d0, d1
	fmov	d1, #0.50000000
	fmul	d1, d0, d1
	fmul	d1, d0, d1
	fmov	d2, #2.00000000
	fadd	d2, d0, d2
	fdiv	d2, d0, d2
	fmul	d3, d2, d2
	fmul	d4, d3, d3
	mov	x8, #21060
	movk	x8, #57150, lsl #16
	movk	x8, #61714, lsl #32
	movk	x8, #16322, lsl #48
	fmov	d5, x8
	fmul	d5, d4, d5
	mov	x8, #990
	movk	x8, #38603, lsl #16
	movk	x8, #18020, lsl #32
	movk	x8, #16327, lsl #48
	fmov	d6, x8
	fadd	d5, d5, d6
	dup.2d	v6, v4[0]
	mov	x8, #50847
	movk	x8, #53368, lsl #16
	movk	x8, #39433, lsl #32
	movk	x8, #16323, lsl #48
	fmov	d7, x8
	dup.2d	v7, v7[0]
	mov.d	v7[1], v5[0]
	fmul.2d	v5, v7, v4[0]
Lloh446:
	adrp	x8, lCPI73_0@PAGE
Lloh447:
	ldr	q7, [x8, lCPI73_0@PAGEOFF]
	fadd.2d	v5, v5, v7
	fmul.2d	v4, v5, v4[0]
Lloh448:
	adrp	x8, lCPI73_1@PAGE
Lloh449:
	ldr	q5, [x8, lCPI73_1@PAGEOFF]
	fadd.2d	v4, v4, v5
	mov.d	v6[1], v3[0]
	fmul.2d	v3, v6, v4
	faddp.2d	d3, v3
	fsub	d4, d0, d1
	fmov	x8, d4
	and	x8, x8, #0xffffffff00000000
	fmov	d4, x8
	fsub	d0, d0, d4
	fsub	d0, d0, d1
	fadd	d1, d1, d3
	fmul	d1, d2, d1
	fadd	d0, d0, d1
	mov	x8, #354418688
	movk	x8, #52091, lsl #32
	movk	x8, #16347, lsl #48
	fmov	d1, x8
	fmul	d2, d4, d1
	scvtf	d3, w9
	mov	x8, #24576
	movk	x8, #20639, lsl #16
	movk	x8, #17427, lsl #32
	movk	x8, #16339, lsl #48
	fmov	d5, x8
	fmul	d5, d3, d5
	mov	x8, #11062
	movk	x8, #4593, lsl #16
	movk	x8, #65267, lsl #32
	movk	x8, #15705, lsl #48
	fmov	d6, x8
	fmul	d3, d3, d6
	fadd	d4, d0, d4
	mov	x8, #44501
	movk	x8, #51866, lsl #16
	movk	x8, #37944, lsl #32
	movk	x8, #15803, lsl #48
	fmov	d6, x8
	fmul	d4, d4, d6
	fadd	d3, d3, d4
	fmul	d0, d0, d1
	fadd	d0, d0, d3
	fadd	d1, d5, d2
	fsub	d3, d5, d1
	fadd	d2, d2, d3
	fadd	d0, d2, d0
	fadd	d0, d1, d0
	ret
LBB73_11:
	fsub	d0, d0, d0
	movi.2d	v1, #0000000000000000
	fdiv	d0, d0, d1
LBB73_12:
	ret
LBB73_13:
	movi.2d	v0, #0000000000000000
	ret
	.loh AdrpLdr	Lloh448, Lloh449
	.loh AdrpAdrp	Lloh446, Lloh448
	.loh AdrpLdr	Lloh446, Lloh447
	.cfi_endproc

	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0
lCPI74_0:
	.quad	0x3fcc71c51d8e78af
	.quad	0x3fd2492494229359
lCPI74_1:
	.quad	0x3fd999999997fa04
	.quad	0x3fe5555555555593
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5log1p5log1p
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5log1p5log1p:
	.cfi_startproc
	fmov	x8, d0
	mov	x9, #-138014479089665
	movk	x9, #16346, lsl #48
	cmp	x8, x9
	b.gt	LBB74_4
	lsr	x9, x8, #52
	cmp	x9, #3070
	b.ls	LBB74_7
	fmov	d1, #-1.00000000
	fcmp	d0, d1
	b.ne	LBB74_12
	mov	x8, #-4503599627370496
	fmov	d0, x8
	ret
LBB74_4:
	lsr	x8, x8, #52
	cmp	x8, #2046
	b.hi	LBB74_9
LBB74_5:
	fmov	d1, #1.00000000
	fadd	d3, d0, d1
	fmov	x8, d3
	mov	w9, #24418
	movk	w9, #9, lsl #16
	add	x10, x9, x8, lsr #32
	lsr	w11, w10, #20
	sub	w9, w11, #1023
	movi.2d	v4, #0000000000000000
	fsub	d2, d0, d3
	fadd	d1, d2, d1
	fmov	d2, #-1.00000000
	fadd	d5, d3, d2
	fsub	d0, d0, d5
	cmp	w11, #1024
	fcsel	d0, d1, d0, hi
	fdiv	d0, d0, d3
	cmp	w11, #1077
	fcsel	d1, d0, d4, lo
	and	w10, w10, #0xfffff
	mov	w11, #41118
	movk	w11, #16358, lsl #16
	add	w10, w10, w11
	mov	w8, w8
	orr	x8, x8, x10, lsl #32
	fmov	d0, x8
	fadd	d0, d0, d2
	scvtf	d2, w9
LBB74_6:
	fmov	d3, #0.50000000
	fmul	d3, d0, d3
	fmul	d3, d0, d3
	fmov	d4, #2.00000000
	fadd	d4, d0, d4
	fdiv	d4, d0, d4
	fmul	d5, d4, d4
	fmul	d6, d5, d5
	mov	x8, #21060
	movk	x8, #57150, lsl #16
	movk	x8, #61714, lsl #32
	movk	x8, #16322, lsl #48
	fmov	d7, x8
	fmul	d7, d6, d7
	mov	x8, #990
	movk	x8, #38603, lsl #16
	movk	x8, #18020, lsl #32
	movk	x8, #16327, lsl #48
	fmov	d16, x8
	fadd	d7, d7, d16
	dup.2d	v16, v6[0]
	mov	x8, #50847
	movk	x8, #53368, lsl #16
	movk	x8, #39433, lsl #32
	movk	x8, #16323, lsl #48
	fmov	d17, x8
	dup.2d	v17, v17[0]
	mov.d	v17[1], v7[0]
	fmul.2d	v7, v17, v6[0]
Lloh450:
	adrp	x8, lCPI74_0@PAGE
Lloh451:
	ldr	q17, [x8, lCPI74_0@PAGEOFF]
	fadd.2d	v7, v7, v17
	fmul.2d	v6, v7, v6[0]
Lloh452:
	adrp	x8, lCPI74_1@PAGE
Lloh453:
	ldr	q7, [x8, lCPI74_1@PAGEOFF]
	fadd.2d	v6, v6, v7
	mov.d	v16[1], v5[0]
	fmul.2d	v5, v16, v6
	faddp.2d	d5, v5
	fadd	d5, d3, d5
	fmul	d4, d4, d5
	mov	x8, #15478
	movk	x8, #13689, lsl #16
	movk	x8, #14831, lsl #32
	movk	x8, #15850, lsl #48
	fmov	d5, x8
	fmul	d5, d2, d5
	fadd	d1, d1, d5
	fadd	d1, d1, d4
	fsub	d1, d1, d3
	fadd	d0, d0, d1
	mov	x8, #4276092928
	movk	x8, #11842, lsl #32
	movk	x8, #16358, lsl #48
	fmov	d1, x8
	fmul	d1, d2, d1
	fadd	d0, d1, d0
	ret
LBB74_7:
	lsr	x9, x8, #32
	ubfx	w10, w9, #21, #10
	cmp	w10, #485
	b.hs	LBB74_10
	tst	w9, #0x7ff00000
	b.eq	LBB74_13
LBB74_9:
	ret
LBB74_10:
	movi.2d	v1, #0000000000000000
	mov	x9, #-71721658875905
	movk	x9, #49106, lsl #48
	cmp	x8, x9
	b.hi	LBB74_5
	movi.2d	v2, #0000000000000000
	b	LBB74_6
LBB74_12:
	fsub	d0, d0, d0
	movi.2d	v1, #0000000000000000
	fdiv	d0, d0, d1
	ret
LBB74_13:
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	fcvt	s1, d0
	str	s1, [sp, #12]
	ldr	s1, [sp, #12]
	add	sp, sp, #16
	.cfi_def_cfa_offset 0
	ret
	.loh AdrpLdr	Lloh452, Lloh453
	.loh AdrpAdrp	Lloh450, Lloh452
	.loh AdrpLdr	Lloh450, Lloh451
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5round5round
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5round5round:
	.cfi_startproc
	mov	x8, #4602678819172646911
	fmov	d1, x8
	movi.2d	v2, #0xffffffffffffffff
	fneg.2d	v2, v2
	bif.16b	v1, v0, v2
	fadd	d0, d0, d1
	fmov	x8, d0
	ubfx	x9, x8, #52, #11
	cmp	w9, #1074
	b.hi	LBB75_3
	subs	w9, w9, #1023
	mov	x10, #-4503599627370496
	asr	x9, x10, x9
	mov	x10, #-9223372036854775808
	csel	x9, x10, x9, lo
	bics	xzr, x8, x9
	b.eq	LBB75_3
	and	x8, x9, x8
	fmov	d0, x8
LBB75_3:
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5round6roundf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5round6roundf:
	.cfi_startproc
	mov	w8, #1056964607
	fmov	s1, w8
	mvni.4s	v2, #128, lsl #24
	bif.16b	v1, v0, v2
	fadd	s0, s0, s1
	fmov	w8, s0
	ubfx	w9, w8, #23, #8
	cmp	w9, #149
	b.hi	LBB76_3
	subs	w9, w9, #127
	mov	w10, #-8388608
	asr	w9, w10, w9
	mov	w10, #-2147483648
	csel	w9, w10, w9, lo
	bics	wzr, w8, w9
	b.eq	LBB76_3
	and	w8, w9, w8
	fmov	s0, w8
LBB76_3:
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5sinhf5sinhf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5sinhf5sinhf:
	.cfi_startproc
	sub	sp, sp, #80
	.cfi_def_cfa_offset 80
	stp	d9, d8, [sp, #32]
	stp	x20, x19, [sp, #48]
	stp	x29, x30, [sp, #64]
	add	x29, sp, #64
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset b8, -40
	.cfi_offset b9, -48
	mov.16b	v2, v0
	fmov	s0, #0.50000000
	mvni.4s	v1, #128, lsl #24
	bsl.16b	v1, v0, v2
	fabs	s0, s2
	fmov	w19, s0
	mov	w8, #29207
	movk	w8, #17073, lsl #16
	cmp	w19, w8
	b.hs	LBB77_4
	stp	q2, q1, [sp]
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math6expm1f6expm1f
	lsr	w8, w19, #23
	cmp	w8, #127
	b.hs	LBB77_5
	cmp	w8, #115
	ldr	q1, [sp]
	b.lo	LBB77_7
	fadd	s1, s0, s0
	fmul	s2, s0, s0
	fmov	s3, #1.00000000
	fadd	s0, s0, s3
	fdiv	s0, s2, s0
	fsub	s0, s1, s0
	b	LBB77_6
LBB77_4:
	fadd	s8, s1, s1
	mov	w8, #58300
	movk	w8, #49954, lsl #16
	fmov	s1, w8
	fadd	s0, s0, s1
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4expf4expf
	movi.2s	v1, #122, lsl #24
	fmul	s0, s0, s1
	fmul	s0, s0, s1
	fmul	s1, s8, s0
	b	LBB77_7
LBB77_5:
	fmov	s1, #1.00000000
	fadd	s1, s0, s1
	fdiv	s1, s0, s1
	fadd	s0, s0, s1
LBB77_6:
	ldr	q1, [sp, #16]
	fmul	s1, s1, s0
LBB77_7:
	mov.16b	v0, v1
	.cfi_def_cfa wsp, 80
	ldp	x29, x30, [sp, #64]
	ldp	x20, x19, [sp, #48]
	ldp	d9, d8, [sp, #32]
	add	sp, sp, #80
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore b8
	.cfi_restore b9
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5tanhf5tanhf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5tanhf5tanhf:
	.cfi_startproc
	sub	sp, sp, #48
	.cfi_def_cfa_offset 48
	stp	d9, d8, [sp, #16]
	stp	x29, x30, [sp, #32]
	add	x29, sp, #32
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset b8, -24
	.cfi_offset b9, -32
	mov.16b	v8, v0
	fabs	s0, s0
	fmov	w8, s0
	mov	w9, #40788
	movk	w9, #16140, lsl #16
	cmp	w8, w9
	b.ls	LBB78_3
	mov	w9, #1092616192
	cmp	w8, w9
	b.ls	LBB78_5
	movi.2d	v1, #0000000000000000
	fdiv	s0, s1, s0
	fmov	s1, #1.00000000
	fadd	s0, s0, s1
	b	LBB78_9
LBB78_3:
	mov	w9, #50552
	movk	w9, #16002, lsl #16
	cmp	w8, w9
	b.ls	LBB78_6
	fadd	s0, s0, s0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math6expm1f6expm1f
	fmov	s1, #2.00000000
	fadd	s1, s0, s1
	fdiv	s0, s0, s1
	b	LBB78_9
LBB78_5:
	fadd	s0, s0, s0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math6expm1f6expm1f
	fmov	s1, #2.00000000
	fadd	s0, s0, s1
	fdiv	s0, s1, s0
	fmov	s1, #1.00000000
	fsub	s0, s1, s0
	b	LBB78_9
LBB78_6:
	lsr	w8, w8, #23
	cbz	w8, LBB78_8
	fmov	s1, #-2.00000000
	fmul	s0, s0, s1
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math6expm1f6expm1f
	fneg	s1, s0
	fmov	s2, #2.00000000
	fadd	s0, s0, s2
	fdiv	s0, s1, s0
	b	LBB78_9
LBB78_8:
	fmul	s1, s8, s8
	str	s1, [sp, #12]
	ldr	s1, [sp, #12]
LBB78_9:
	fmov	w8, s8
	fneg	s1, s0
	cmp	w8, #0
	fcsel	s0, s1, s0, mi
	.cfi_def_cfa wsp, 48
	ldp	x29, x30, [sp, #32]
	ldp	d9, d8, [sp, #16]
	add	sp, sp, #48
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore b8
	.cfi_restore b9
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5trunc5trunc
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5trunc5trunc:
	.cfi_startproc
	fmov	x8, d0
	ubfx	x9, x8, #52, #11
	cmp	w9, #1074
	b.hi	LBB79_3
	subs	w9, w9, #1023
	mov	x10, #-4503599627370496
	asr	x9, x10, x9
	mov	x10, #-9223372036854775808
	csel	x9, x10, x9, lo
	bics	xzr, x8, x9
	b.eq	LBB79_3
	and	x8, x9, x8
	fmov	d0, x8
LBB79_3:
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5trunc6truncf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math5trunc6truncf:
	.cfi_startproc
	fmov	w8, s0
	ubfx	w9, w8, #23, #8
	cmp	w9, #149
	b.hi	LBB80_3
	subs	w9, w9, #127
	mov	w10, #-8388608
	asr	w9, w10, w9
	mov	w10, #-2147483648
	csel	w9, w10, w9, lo
	bics	wzr, w8, w9
	b.eq	LBB80_3
	and	w8, w9, w8
	fmov	s0, w8
LBB80_3:
	ret
	.cfi_endproc

	.section	__TEXT,__literal8,8byte_literals
	.p2align	3, 0x0
lCPI81_0:
	.long	0x3e91e9ee
	.long	0x3e789e26
lCPI81_1:
	.long	0x3f2aaaaa
	.long	0x3eccce13
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math6acoshf6acoshf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math6acoshf6acoshf:
	.cfi_startproc
	fmov	w8, s0
	ubfx	w9, w8, #30, #1
	cbnz	w9, LBB81_2
	fmov	s1, #-1.00000000
	fadd	s0, s0, s1
	fmul	s1, s0, s0
	fadd	s2, s0, s0
	fadd	s1, s1, s2
	; InlineAsm Start
	fsqrt	s1, s1
	; InlineAsm End
	fadd	s0, s0, s1
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math6log1pf6log1pf
LBB81_2:
	and	w8, w8, #0x7fffffff
	lsr	w8, w8, #23
	cmp	w8, #139
	b.hs	LBB81_7
	fadd	s1, s0, s0
	fmul	s2, s0, s0
	fmov	s3, #-1.00000000
	fadd	s2, s2, s3
	; InlineAsm Start
	fsqrt	s2, s2
	; InlineAsm End
	fadd	s0, s0, s2
	fdiv	s0, s3, s0
	fadd	s0, s1, s0
	fmov	w8, s0
	cmp	w8, #2048, lsl #12
	b.lt	LBB81_11
	lsr	w9, w8, #23
	cmp	w9, #254
	b.hi	LBB81_23
	movi.2d	v0, #0000000000000000
	mov	w9, #1065353216
	cmp	w8, w9
	b.eq	LBB81_23
	mov	w9, #-127
	b	LBB81_18
LBB81_7:
	fmov	w8, s0
	cmp	w8, #2048, lsl #12
	b.lt	LBB81_13
	lsr	w9, w8, #23
	cmp	w9, #254
	b.hi	LBB81_15
	movi.2d	v0, #0000000000000000
	mov	w9, #1065353216
	cmp	w8, w9
	b.eq	LBB81_15
	mov	w9, #-127
	b	LBB81_21
LBB81_11:
	fcmp	s0, #0.0
	b.ne	LBB81_16
	fmul	s0, s0, s0
	fmov	s1, #-1.00000000
	fdiv	s0, s1, s0
	ret
LBB81_13:
	fcmp	s0, #0.0
	b.ne	LBB81_19
	fmul	s0, s0, s0
	fmov	s1, #-1.00000000
	fdiv	s0, s1, s0
LBB81_15:
	mov	w8, #29208
	movk	w8, #16177, lsl #16
	fmov	s1, w8
	fadd	s0, s0, s1
	ret
LBB81_16:
	tbnz	w8, #31, LBB81_22
	movi.2s	v1, #76, lsl #24
	fmul	s0, s0, s1
	fmov	w8, s0
	mov	w9, #-152
LBB81_18:
	add	w8, w8, #1199, lsl #12
	add	w8, w8, #2829
	add	w9, w9, w8, lsr #23
	and	w8, w8, #0x7fffff
	mov	w10, #1267
	movk	w10, #16181, lsl #16
	add	w8, w8, w10
	fmov	s0, w8
	fmov	s1, #-1.00000000
	fadd	s0, s0, s1
	fmov	s1, #2.00000000
	fadd	s1, s0, s1
	fdiv	s1, s0, s1
	fmul	s2, s1, s1
	fmul	s3, s2, s2
	dup.2s	v4, v3[0]
Lloh454:
	adrp	x8, lCPI81_0@PAGE
Lloh455:
	ldr	d5, [x8, lCPI81_0@PAGEOFF]
	fmul.2s	v3, v5, v3[0]
Lloh456:
	adrp	x8, lCPI81_1@PAGE
Lloh457:
	ldr	d5, [x8, lCPI81_1@PAGEOFF]
	fadd.2s	v3, v3, v5
	mov.s	v4[0], v2[0]
	fmul.2s	v2, v4, v3
	faddp.2s	s2, v2
	fmov	s3, #0.50000000
	fmul	s3, s0, s3
	fmul	s3, s0, s3
	scvtf	s4, w9
	fadd	s2, s3, s2
	fmul	s1, s1, s2
	mov	w8, #63441
	movk	w8, #14103, lsl #16
	fmov	s2, w8
	fmul	s2, s4, s2
	fadd	s1, s2, s1
	fsub	s1, s1, s3
	fadd	s0, s0, s1
	mov	w8, #29056
	movk	w8, #16177, lsl #16
	fmov	s1, w8
	fmul	s1, s4, s1
	fadd	s0, s1, s0
	ret
LBB81_19:
	tbnz	w8, #31, LBB81_24
	movi.2s	v1, #76, lsl #24
	fmul	s0, s0, s1
	fmov	w8, s0
	mov	w9, #-152
LBB81_21:
	add	w8, w8, #1199, lsl #12
	add	w8, w8, #2829
	add	w9, w9, w8, lsr #23
	and	w8, w8, #0x7fffff
	mov	w10, #1267
	movk	w10, #16181, lsl #16
	add	w8, w8, w10
	fmov	s0, w8
	fmov	s1, #-1.00000000
	fadd	s0, s0, s1
	fmov	s1, #2.00000000
	fadd	s1, s0, s1
	fdiv	s1, s0, s1
	fmul	s2, s1, s1
	fmul	s3, s2, s2
	dup.2s	v4, v3[0]
Lloh458:
	adrp	x8, lCPI81_0@PAGE
Lloh459:
	ldr	d5, [x8, lCPI81_0@PAGEOFF]
	fmul.2s	v3, v5, v3[0]
Lloh460:
	adrp	x8, lCPI81_1@PAGE
Lloh461:
	ldr	d5, [x8, lCPI81_1@PAGEOFF]
	fadd.2s	v3, v3, v5
	mov.s	v4[0], v2[0]
	fmul.2s	v2, v4, v3
	faddp.2s	s2, v2
	fmov	s3, #0.50000000
	fmul	s3, s0, s3
	fmul	s3, s0, s3
	scvtf	s4, w9
	fadd	s2, s3, s2
	fmul	s1, s1, s2
	mov	w8, #63441
	movk	w8, #14103, lsl #16
	fmov	s2, w8
	fmul	s2, s4, s2
	fadd	s1, s2, s1
	fsub	s1, s1, s3
	fadd	s0, s0, s1
	mov	w8, #29056
	movk	w8, #16177, lsl #16
	fmov	s1, w8
	fmul	s1, s4, s1
	fadd	s0, s1, s0
	mov	w8, #29208
	movk	w8, #16177, lsl #16
	fmov	s1, w8
	fadd	s0, s0, s1
	ret
LBB81_22:
	fsub	s0, s0, s0
	movi.2d	v1, #0000000000000000
	fdiv	s0, s0, s1
LBB81_23:
	ret
LBB81_24:
	fsub	s0, s0, s0
	movi.2d	v1, #0000000000000000
	fdiv	s0, s0, s1
	mov	w8, #29208
	movk	w8, #16177, lsl #16
	fmov	s1, w8
	fadd	s0, s0, s1
	ret
	.loh AdrpLdr	Lloh456, Lloh457
	.loh AdrpAdrp	Lloh454, Lloh456
	.loh AdrpLdr	Lloh454, Lloh455
	.loh AdrpLdr	Lloh460, Lloh461
	.loh AdrpAdrp	Lloh458, Lloh460
	.loh AdrpLdr	Lloh458, Lloh459
	.cfi_endproc

	.section	__TEXT,__literal8,8byte_literals
	.p2align	3, 0x0
lCPI82_0:
	.long	0x3e91e9ee
	.long	0x3e789e26
lCPI82_1:
	.long	0x3f2aaaaa
	.long	0x3eccce13
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math6asinhf6asinhf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math6asinhf6asinhf:
	.cfi_startproc
	fabs	s1, s0
	fmov	w8, s1
	lsr	w9, w8, #23
	cmp	w9, #138
	b.ls	LBB82_5
	cmp	w9, #254
	b.hi	LBB82_3
	add	w8, w8, #1199, lsl #12
	add	w8, w8, #2829
	lsr	w9, w8, #23
	sub	w9, w9, #127
	and	w8, w8, #0x7fffff
	mov	w10, #1267
	movk	w10, #16181, lsl #16
	add	w8, w8, w10
	fmov	s1, w8
	fmov	s2, #-1.00000000
	fadd	s1, s1, s2
	fmov	s2, #2.00000000
	fadd	s2, s1, s2
	fdiv	s2, s1, s2
	fmul	s3, s2, s2
	fmul	s4, s3, s3
	dup.2s	v5, v4[0]
Lloh462:
	adrp	x8, lCPI82_0@PAGE
Lloh463:
	ldr	d6, [x8, lCPI82_0@PAGEOFF]
	fmul.2s	v4, v6, v4[0]
Lloh464:
	adrp	x8, lCPI82_1@PAGE
Lloh465:
	ldr	d6, [x8, lCPI82_1@PAGEOFF]
	fadd.2s	v4, v4, v6
	mov.s	v5[0], v3[0]
	fmul.2s	v3, v5, v4
	faddp.2s	s3, v3
	fmov	s4, #0.50000000
	fmul	s4, s1, s4
	fmul	s4, s1, s4
	ucvtf	s5, w9
	fadd	s3, s4, s3
	fmul	s2, s2, s3
	mov	w8, #63441
	movk	w8, #14103, lsl #16
	fmov	s3, w8
	fmul	s3, s5, s3
	fadd	s2, s3, s2
	fsub	s2, s2, s4
	fadd	s1, s1, s2
	mov	w8, #29056
	movk	w8, #16177, lsl #16
	fmov	s2, w8
	fmul	s2, s5, s2
	fadd	s1, s2, s1
LBB82_3:
	mov	w8, #29208
	movk	w8, #16177, lsl #16
	fmov	s2, w8
	fadd	s1, s1, s2
LBB82_4:
	fmov	w8, s0
	fneg	s0, s1
	cmp	w8, #0
	fcsel	s0, s0, s1, mi
	ret
LBB82_5:
	lsr	w8, w8, #30
	cbz	w8, LBB82_10
	fadd	s2, s1, s1
	fmul	s3, s0, s0
	fmov	s4, #1.00000000
	fadd	s3, s3, s4
	; InlineAsm Start
	fsqrt	s3, s3
	; InlineAsm End
	fadd	s1, s1, s3
	fdiv	s1, s4, s1
	fadd	s1, s2, s1
	fmov	w8, s1
	cmp	w8, #2048, lsl #12
	b.lt	LBB82_12
	lsr	w9, w8, #23
	cmp	w9, #254
	b.hi	LBB82_4
	movi.2d	v1, #0000000000000000
	mov	w9, #1065353216
	cmp	w8, w9
	b.eq	LBB82_4
	mov	w9, #-127
	b	LBB82_18
LBB82_10:
	sub	sp, sp, #48
	.cfi_def_cfa_offset 48
	stp	d9, d8, [sp, #16]
	stp	x29, x30, [sp, #32]
	add	x29, sp, #32
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset b8, -24
	.cfi_offset b9, -32
	.cfi_remember_state
	cmp	w9, #114
	b.ls	LBB82_14
	fmul	s2, s0, s0
	fmov	s3, #1.00000000
	fadd	s4, s2, s3
	; InlineAsm Start
	fsqrt	s4, s4
	; InlineAsm End
	fadd	s3, s4, s3
	fdiv	s2, s2, s3
	fadd	s1, s1, s2
	mov.16b	v8, v0
	mov.16b	v0, v1
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math6log1pf6log1pf
	mov.16b	v1, v0
	mov.16b	v0, v8
	b	LBB82_15
LBB82_12:
	.cfi_def_cfa wsp, 0
	.cfi_same_value w30
	.cfi_same_value w29
	.cfi_same_value b8
	.cfi_same_value b9
	fcmp	s1, #0.0
	b.ne	LBB82_16
	fmul	s1, s1, s1
	fmov	s2, #-1.00000000
	fdiv	s1, s2, s1
	fmov	w8, s0
	fneg	s0, s1
	cmp	w8, #0
	fcsel	s0, s0, s1, mi
	ret
LBB82_14:
	.cfi_restore_state
	mov	w8, #2071986176
	fmov	s2, w8
	fadd	s2, s1, s2
	str	s2, [sp, #12]
	ldr	s2, [sp, #12]
LBB82_15:
	.cfi_def_cfa wsp, 48
	ldp	x29, x30, [sp, #32]
	ldp	d9, d8, [sp, #16]
	add	sp, sp, #48
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore b8
	.cfi_restore b9
	fmov	w8, s0
	fneg	s0, s1
	cmp	w8, #0
	fcsel	s0, s0, s1, mi
	ret
LBB82_16:
	tbnz	w8, #31, LBB82_19
	movi.2s	v2, #76, lsl #24
	fmul	s1, s1, s2
	fmov	w8, s1
	mov	w9, #-152
LBB82_18:
	add	w8, w8, #1199, lsl #12
	add	w8, w8, #2829
	add	w9, w9, w8, lsr #23
	and	w8, w8, #0x7fffff
	mov	w10, #1267
	movk	w10, #16181, lsl #16
	add	w8, w8, w10
	fmov	s1, w8
	fmov	s2, #-1.00000000
	fadd	s1, s1, s2
	fmov	s2, #2.00000000
	fadd	s2, s1, s2
	fdiv	s2, s1, s2
	fmul	s3, s2, s2
	fmul	s4, s3, s3
	dup.2s	v5, v4[0]
Lloh466:
	adrp	x8, lCPI82_0@PAGE
Lloh467:
	ldr	d6, [x8, lCPI82_0@PAGEOFF]
	fmul.2s	v4, v6, v4[0]
Lloh468:
	adrp	x8, lCPI82_1@PAGE
Lloh469:
	ldr	d6, [x8, lCPI82_1@PAGEOFF]
	fadd.2s	v4, v4, v6
	mov.s	v5[0], v3[0]
	fmul.2s	v3, v5, v4
	faddp.2s	s3, v3
	fmov	s4, #0.50000000
	fmul	s4, s1, s4
	fmul	s4, s1, s4
	scvtf	s5, w9
	fadd	s3, s4, s3
	fmul	s2, s2, s3
	mov	w8, #63441
	movk	w8, #14103, lsl #16
	fmov	s3, w8
	fmul	s3, s5, s3
	fadd	s2, s3, s2
	fsub	s2, s2, s4
	fadd	s1, s1, s2
	mov	w8, #29056
	movk	w8, #16177, lsl #16
	fmov	s2, w8
	fmul	s2, s5, s2
	fadd	s1, s2, s1
	fmov	w8, s0
	fneg	s0, s1
	cmp	w8, #0
	fcsel	s0, s0, s1, mi
	ret
LBB82_19:
	fsub	s1, s1, s1
	movi.2d	v2, #0000000000000000
	fdiv	s1, s1, s2
	fmov	w8, s0
	fneg	s0, s1
	cmp	w8, #0
	fcsel	s0, s0, s1, mi
	ret
	.loh AdrpLdr	Lloh464, Lloh465
	.loh AdrpAdrp	Lloh462, Lloh464
	.loh AdrpLdr	Lloh462, Lloh463
	.loh AdrpLdr	Lloh468, Lloh469
	.loh AdrpAdrp	Lloh466, Lloh468
	.loh AdrpLdr	Lloh466, Lloh467
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math6atan2f6atan2f
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math6atan2f6atan2f:
	.cfi_startproc
	stp	x20, x19, [sp, #-32]!
	.cfi_def_cfa_offset 32
	stp	x29, x30, [sp, #16]
	add	x29, sp, #16
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_remember_state
	fcmp	s1, s0
	b.vs	LBB83_25
	fmov	w8, s1
	mov	w9, #1065353216
	cmp	w8, w9
	b.ne	LBB83_3
	.cfi_def_cfa wsp, 32
	ldp	x29, x30, [sp, #16]
	ldp	x20, x19, [sp], #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math5atanf5atanf
LBB83_3:
	.cfi_restore_state
	.cfi_remember_state
	fmov	w10, s0
	lsr	w8, w8, #30
	and	w9, w8, #0x2
	mov	x19, x9
	bfxil	w19, w10, #31, #1
	ands	w8, w10, #0x7fffffff
	b.eq	LBB83_8
	fmov	w10, s1
	and	w10, w10, #0x7fffffff
	cbz	w10, LBB83_17
	mov	w11, #2139095040
	cmp	w10, w11
	b.ne	LBB83_9
	cmp	w8, w11
	b.ne	LBB83_18
Lloh470:
	adrp	x8, l_switch.table._RNvNtNtCsdtCTLUYU3dx_4libm4math6atan2f6atan2f@PAGE
Lloh471:
	add	x8, x8, l_switch.table._RNvNtNtCsdtCTLUYU3dx_4libm4math6atan2f6atan2f@PAGEOFF
	b	LBB83_19
LBB83_8:
	mov	w8, #4059
	movk	w8, #49225, lsl #16
	fmov	s1, w8
	cmp	w19, #2
	mov	w8, #4059
	movk	w8, #16457, lsl #16
	fmov	s2, w8
	fcsel	s1, s2, s1, eq
	fcsel	s0, s0, s1, lo
	b	LBB83_20
LBB83_9:
	cmp	w8, w11
	b.eq	LBB83_17
	mov	w11, #218103808
	add	w11, w10, w11
	cmp	w11, w8
	b.lo	LBB83_17
	cbz	w9, LBB83_13
	mov	w9, #218103808
	movi.2d	v2, #0000000000000000
	add	w8, w8, w9
	cmp	w8, w10
	b.lo	LBB83_14
LBB83_13:
	fdiv	s0, s0, s1
	fabs	s0, s0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5atanf5atanf
	mov.16b	v2, v0
LBB83_14:
	cmp	w19, #1
	b.gt	LBB83_22
	mov.16b	v0, v2
	cbz	w19, LBB83_20
	fneg	s0, s0
	b	LBB83_20
LBB83_17:
	mov	w8, #4059
	movk	w8, #16329, lsl #16
	fmov	s1, w8
	mvni.4s	v2, #128, lsl #24
	bit.16b	v0, v1, v2
	.cfi_def_cfa wsp, 32
	b	LBB83_21
LBB83_18:
	.cfi_restore_state
	.cfi_remember_state
Lloh472:
	adrp	x8, l_switch.table._RNvNtNtCsdtCTLUYU3dx_4libm4math6atan2f6atan2f.50@PAGE
Lloh473:
	add	x8, x8, l_switch.table._RNvNtNtCsdtCTLUYU3dx_4libm4math6atan2f6atan2f.50@PAGEOFF
LBB83_19:
	ldr	s0, [x8, x19, lsl #2]
LBB83_20:
	.cfi_def_cfa wsp, 32
LBB83_21:
	ldp	x29, x30, [sp, #16]
	ldp	x20, x19, [sp], #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	ret
LBB83_22:
	.cfi_restore_state
	mov	w8, #48430
	movk	w8, #13243, lsl #16
	fmov	s0, w8
	fadd	s0, s2, s0
	cmp	w19, #2
	b.ne	LBB83_24
	mov	w8, #4059
	movk	w8, #16457, lsl #16
	fmov	s1, w8
	fsub	s0, s1, s0
	b	LBB83_20
LBB83_24:
	mov	w8, #4059
	movk	w8, #49225, lsl #16
	fmov	s1, w8
	fadd	s0, s0, s1
	b	LBB83_20
LBB83_25:
	fadd	s0, s0, s1
	b	LBB83_20
	.loh AdrpAdd	Lloh470, Lloh471
	.loh AdrpAdd	Lloh472, Lloh473
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math6atanhf6atanhf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math6atanhf6atanhf:
	.cfi_startproc
	sub	sp, sp, #48
	.cfi_def_cfa_offset 48
	stp	d9, d8, [sp, #16]
	stp	x29, x30, [sp, #32]
	add	x29, sp, #32
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset b8, -24
	.cfi_offset b9, -32
	mov.16b	v8, v0
	fabs	s0, s0
	fmov	w8, s0
	lsr	w9, w8, #24
	cmp	w9, #63
	b.hs	LBB84_4
	lsr	w8, w8, #23
	cmp	w8, #95
	b.hs	LBB84_5
	cbnz	w8, LBB84_7
	fmul	s1, s8, s8
	str	s1, [sp, #12]
	ldr	s1, [sp, #12]
	b	LBB84_7
LBB84_4:
	fmov	s1, #1.00000000
	fsub	s1, s1, s0
	fdiv	s0, s0, s1
	fadd	s0, s0, s0
	b	LBB84_6
LBB84_5:
	fadd	s1, s0, s0
	fmul	s2, s0, s1
	fmov	s3, #1.00000000
	fsub	s0, s3, s0
	fdiv	s0, s2, s0
	fadd	s0, s1, s0
LBB84_6:
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math6log1pf6log1pf
	fmov	s1, #0.50000000
	fmul	s0, s0, s1
LBB84_7:
	fmov	w8, s8
	fneg	s1, s0
	cmp	w8, #0
	fcsel	s0, s1, s0, mi
	.cfi_def_cfa wsp, 48
	ldp	x29, x30, [sp, #32]
	ldp	d9, d8, [sp, #16]
	add	sp, sp, #48
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore b8
	.cfi_restore b9
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math6exp10f6exp10f
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math6exp10f6exp10f:
	.cfi_startproc
	fmov	w8, s0
	ubfx	w10, w8, #23, #8
	sub	w9, w10, #127
	cmp	w10, #149
	b.ls	LBB85_4
	tst	w8, #0x7fffff
	b.eq	LBB85_3
	mov.16b	v1, v0
	mov.16b	v2, v0
	cmp	w9, #128
	b.eq	LBB85_5
LBB85_3:
	and	w8, w8, #0x80000000
	fmov	s2, w8
	mov.16b	v1, v0
	b	LBB85_5
LBB85_4:
	and	w11, w8, #0x80000000
	fmov	s1, w11
	lsl	w11, w8, w9
	tst	w11, #0x7fffff
	mov	w11, #-8388608
	asr	w9, w11, w9
	and	w8, w9, w8
	fmov	s2, w8
	fsub	s3, s0, s2
	fcsel	s2, s0, s2, eq
	fcsel	s3, s1, s3, eq
	cmp	w10, #127
	fcsel	s1, s1, s2, lo
	fcsel	s2, s0, s3, lo
LBB85_5:
	sub	sp, sp, #32
	.cfi_def_cfa_offset 32
	stp	x29, x30, [sp, #16]
	add	x29, sp, #16
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_remember_state
	fmov	w8, s1
	ubfx	w8, w8, #24, #7
	cmp	w8, #65
	b.hs	LBB85_8
	fcmp	s2, #0.0
	b.ne	LBB85_9
	fcvtzs	x8, s1
Lloh474:
	adrp	x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.94@PAGE
Lloh475:
	add	x9, x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.94@PAGEOFF
	add	x8, x9, x8, lsl #2
	ldr	s0, [x8, #28]
	b	LBB85_18
LBB85_8:
	fcvt	d0, s0
	mov	x8, #41841
	movk	x8, #2425, lsl #16
	movk	x8, #37711, lsl #32
	movk	x8, #16394, lsl #48
	fmov	d1, x8
	fmul	d0, d0, d1
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4exp24exp2
	fcvt	s0, d0
	b	LBB85_18
LBB85_9:
	mov	w8, #39544
	movk	w8, #16468, lsl #16
	fmov	s0, w8
	fmul	s0, s2, s0
	fmov	w8, s0
	and	w9, w8, #0x7fffffff
	mov	w10, #1123811328
	cmp	w9, w10
	b.ls	LBB85_13
	mov	w10, #2139095040
	cmp	w9, w10
	b.hi	LBB85_17
	mov	w9, #1124073471
	cmp	w8, w9
	b.le	LBB85_15
	movi.2s	v2, #127, lsl #24
	fmul	s0, s0, s2
	b	LBB85_17
LBB85_13:
	mov	w8, #855638016
	cmp	w9, w8
	b.hi	LBB85_16
	fmov	s2, #1.00000000
	fadd	s0, s0, s2
	b	LBB85_17
LBB85_15:
	tbnz	w8, #31, LBB85_19
LBB85_16:
	mov	w8, #1228931072
	fmov	s2, w8
	fadd	s2, s0, s2
	mov	w8, #-918552576
	fmov	s3, w8
	fadd	s3, s2, s3
	fmov	w8, s2
	add	w8, w8, #8
	lsr	w9, w8, #4
	add	w9, w9, #1023
	lsl	x9, x9, #52
	fmov	d2, x9
	and	w8, w8, #0xf
	fsub	s0, s0, s3
Lloh476:
	adrp	x9, __RNvNtNtCsdtCTLUYU3dx_4libm4math5exp2f6EXP2FT@PAGE
Lloh477:
	add	x9, x9, __RNvNtNtCsdtCTLUYU3dx_4libm4math5exp2f6EXP2FT@PAGEOFF
	fcvt	d0, s0
	ldr	d3, [x9, w8, uxtw #3]
	fmul	d4, d3, d0
	mov	x8, #210822764691456
	movk	x8, #16334, lsl #48
	fmov	d5, x8
	fmul	d5, d0, d5
	mov	x8, #50865297686528
	movk	x8, #16358, lsl #48
	fmov	d6, x8
	fadd	d5, d5, d6
	fmul	d5, d5, d4
	fadd	d3, d3, d5
	fmul	d5, d0, d0
	fmul	d4, d5, d4
	mov	x8, #3221225472
	movk	x8, #45769, lsl #32
	movk	x8, #16259, lsl #48
	fmov	d5, x8
	fmul	d0, d0, d5
	mov	x8, #2147483648
	movk	x8, #27444, lsl #32
	movk	x8, #16300, lsl #48
	fmov	d5, x8
	fadd	d0, d0, d5
	fmul	d0, d0, d4
	fadd	d0, d3, d0
	fmul	d0, d0, d2
	fcvt	s0, d0
LBB85_17:
	fcvtzs	x8, s1
Lloh478:
	adrp	x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.94@PAGE
Lloh479:
	add	x9, x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.94@PAGEOFF
	add	x8, x9, x8, lsl #2
	ldr	s1, [x8, #28]
	fmul	s0, s0, s1
LBB85_18:
	.cfi_def_cfa wsp, 32
	ldp	x29, x30, [sp, #16]
	add	sp, sp, #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
LBB85_19:
	.cfi_restore_state
	mov	w9, #-1021968385
	cmp	w8, w9
	b.ls	LBB85_21
	mov	w8, #-2147483647
	fmov	s2, w8
	fdiv	s0, s2, s0
	stur	s0, [x29, #-4]
	ldur	s0, [x29, #-4]
	movi.2d	v0, #0000000000000000
	b	LBB85_17
LBB85_21:
	tst	w8, #0xffff
	b.eq	LBB85_16
	mov	w8, #-2147483647
	fmov	s2, w8
	fdiv	s2, s2, s0
	stur	s2, [x29, #-4]
	ldur	s2, [x29, #-4]
	b	LBB85_16
	.loh AdrpAdd	Lloh474, Lloh475
	.loh AdrpAdd	Lloh476, Lloh477
	.loh AdrpAdd	Lloh478, Lloh479
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math6expm1f6expm1f
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math6expm1f6expm1f:
	.cfi_startproc
	fmov	w8, s0
	and	w9, w8, #0x7fffffff
	mov	w10, #47171
	movk	w10, #16789, lsl #16
	cmp	w9, w10
	b.ls	LBB86_5
	mov	w10, #2139095040
	cmp	w9, w10
	fmov	s1, #-1.00000000
	fcsel	s1, s0, s1, hi
	tbnz	w8, #31, LBB86_24
	b.hi	LBB86_24
	fmov	s1, #0.50000000
	mov	w8, #29208
	movk	w8, #17073, lsl #16
	cmp	w9, w8
	b.lo	LBB86_12
	movi.2s	v1, #127, lsl #24
	fmul	s1, s0, s1
	mov.16b	v0, v1
	ret
LBB86_5:
	mov	w10, #29208
	movk	w10, #16049, lsl #16
	cmp	w9, w10
	b.ls	LBB86_8
	mov	w10, #5522
	movk	w10, #16261, lsl #16
	cmp	w9, w10
	b.hs	LBB86_11
	mov	w9, #29056
	movk	w9, #16177, lsl #16
	fmov	s1, w9
	fadd	s1, s0, s1
	mov	w9, #-1
	mov	w10, #63441
	movk	w10, #46871, lsl #16
	fmov	s2, w10
	mov	w10, #29056
	movk	w10, #48945, lsl #16
	fmov	s3, w10
	fadd	s0, s0, s3
	mov	w10, #63441
	movk	w10, #14103, lsl #16
	fmov	s3, w10
	tst	w8, #0x80000000
	csinc	w8, w9, wzr, ne
	fcsel	s2, s2, s3, ne
	fcsel	s1, s1, s0, ne
	b	LBB86_13
LBB86_8:
	lsr	w8, w9, #24
	cmp	w8, #51
	b.hs	LBB86_14
	cmp	w9, #2048, lsl #12
	b.hs	LBB86_25
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	fmul	s1, s0, s0
	str	s1, [sp, #12]
	ldr	s1, [sp, #12]
	add	sp, sp, #16
	.cfi_def_cfa_offset 0
	ret
LBB86_11:
	cmp	w8, #0
	fmov	s1, #0.50000000
	fmov	s2, #-0.50000000
	fcsel	s1, s2, s1, mi
LBB86_12:
	mov	w8, #43579
	movk	w8, #16312, lsl #16
	fmov	s2, w8
	fmul	s2, s0, s2
	fadd	s1, s2, s1
	fcvtzs	w8, s1
	scvtf	s2, w8
	mov	w9, #29056
	movk	w9, #48945, lsl #16
	fmov	s1, w9
	fmul	s1, s2, s1
	fadd	s1, s0, s1
	mov	w9, #63441
	movk	w9, #14103, lsl #16
	fmov	s0, w9
	fmul	s2, s2, s0
LBB86_13:
	fsub	s0, s1, s2
	fsub	s1, s1, s0
	fsub	s1, s1, s2
	b	LBB86_15
LBB86_14:
	mov	w8, #0
	movi.2d	v1, #0000000000000000
LBB86_15:
	fmov	s2, #0.50000000
	fmul	s3, s0, s2
	fmul	s2, s0, s3
	mov	w9, #12304
	movk	w9, #15055, lsl #16
	fmov	s4, w9
	fmul	s4, s2, s4
	mov	w9, #34920
	movk	w9, #48392, lsl #16
	fmov	s5, w9
	fadd	s4, s4, s5
	fmul	s4, s2, s4
	fmov	s5, #1.00000000
	fadd	s4, s4, s5
	fmul	s3, s3, s4
	fmov	s5, #3.00000000
	fsub	s3, s5, s3
	fsub	s4, s4, s3
	fmul	s3, s0, s3
	fmov	s5, #6.00000000
	fsub	s3, s5, s3
	fdiv	s3, s4, s3
	fmul	s3, s2, s3
	cbz	w8, LBB86_19
	fsub	s3, s3, s1
	fmul	s3, s0, s3
	fsub	s1, s3, s1
	fsub	s1, s1, s2
	cmp	w8, #1
	b.eq	LBB86_20
	cmn	w8, #1
	b.ne	LBB86_22
	fsub	s0, s0, s1
	fmov	s1, #0.50000000
	fmul	s0, s0, s1
	fmov	s1, #-0.50000000
	fadd	s0, s0, s1
	ret
LBB86_19:
	fmul	s1, s0, s3
	fsub	s1, s1, s2
	fsub	s0, s0, s1
	ret
LBB86_20:
	fmov	s2, #-0.25000000
	fcmp	s0, s2
	b.pl	LBB86_26
	fmov	s2, #0.50000000
	fadd	s0, s0, s2
	fsub	s0, s1, s0
	fmov	s1, #-2.00000000
	fmul	s0, s0, s1
	ret
LBB86_22:
	lsl	w9, w8, #23
	mov	w10, #1065353216
	add	w11, w9, w10
	fmov	s2, w11
	cmp	w8, #57
	b.hs	LBB86_27
	sub	w9, w10, w9
	fmov	s3, w9
	cmp	w8, #23
	fsub	s4, s0, s1
	fmov	s5, #1.00000000
	fsub	s6, s5, s3
	fadd	s4, s6, s4
	fadd	s1, s1, s3
	fsub	s0, s0, s1
	fadd	s0, s0, s5
	fcsel	s0, s4, s0, lo
	fmul	s1, s0, s2
LBB86_24:
	mov.16b	v0, v1
LBB86_25:
	ret
LBB86_26:
	fsub	s0, s0, s1
	fadd	s0, s0, s0
	fmov	s1, #1.00000000
	fadd	s0, s0, s1
	ret
LBB86_27:
	fsub	s0, s0, s1
	fmov	s1, #1.00000000
	fadd	s0, s0, s1
	fadd	s1, s0, s0
	movi.2s	v3, #127, lsl #24
	fmul	s1, s1, s3
	fmul	s0, s0, s2
	cmp	w8, #128
	fcsel	s0, s1, s0, eq
	fmov	s1, #-1.00000000
	fadd	s0, s0, s1
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math6frexpf6frexpf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math6frexpf6frexpf:
	.cfi_startproc
	fmov	w8, s0
	ubfx	w9, w8, #23, #8
	cmp	w9, #255
	b.eq	LBB87_4
	cbnz	w9, LBB87_5
	fcmp	s0, #0.0
	b.eq	LBB87_4
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	mov	w8, #1602224128
	fmov	s1, w8
	fmul	s0, s0, s1
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math6frexpf6frexpf
	sub	w0, w0, #64
	.cfi_def_cfa wsp, 16
	ldp	x29, x30, [sp], #16
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
LBB87_4:
	mov	w0, #0
	ret
LBB87_5:
	sub	w0, w9, #126
	and	w8, w8, #0x807fffff
	orr	w8, w8, #0x3f000000
	fmov	s0, w8
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math6lgamma6lgamma
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math6lgamma6lgamma:
	.cfi_startproc
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math8lgamma_r8lgamma_r
	.cfi_endproc

	.section	__TEXT,__literal8,8byte_literals
	.p2align	3, 0x0
lCPI89_0:
	.long	0x3e91e9ee
	.long	0x3e789e26
lCPI89_1:
	.long	0x3f2aaaaa
	.long	0x3eccce13
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math6log1pf6log1pf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math6log1pf6log1pf:
	.cfi_startproc
	fmov	w8, s0
	mov	w9, #5071
	movk	w9, #16084, lsl #16
	cmp	w8, w9
	b.le	LBB89_4
	lsr	w8, w8, #23
	cmp	w8, #254
	b.hi	LBB89_9
LBB89_2:
	fmov	s1, #1.00000000
	fadd	s3, s0, s1
	fmov	w8, s3
	add	w8, w8, #1199, lsl #12
	add	w9, w8, #2829
	lsr	w10, w9, #23
	sub	w8, w10, #127
	movi.2d	v4, #0000000000000000
	lsr	w11, w9, #26
	fsub	s2, s0, s3
	fadd	s1, s2, s1
	fmov	s2, #-1.00000000
	fadd	s5, s3, s2
	fsub	s0, s0, s5
	cmp	w10, #128
	fcsel	s0, s1, s0, hi
	fdiv	s0, s0, s3
	cmp	w11, #19
	fcsel	s1, s0, s4, lo
	and	w9, w9, #0x7fffff
	mov	w10, #1267
	movk	w10, #16181, lsl #16
	add	w9, w9, w10
	fmov	s0, w9
	fadd	s0, s0, s2
	scvtf	s2, w8
LBB89_3:
	fmov	s3, #2.00000000
	fadd	s3, s0, s3
	fdiv	s3, s0, s3
	fmul	s4, s3, s3
	fmul	s5, s4, s4
	dup.2s	v6, v5[0]
Lloh480:
	adrp	x8, lCPI89_0@PAGE
Lloh481:
	ldr	d7, [x8, lCPI89_0@PAGEOFF]
	fmul.2s	v5, v7, v5[0]
Lloh482:
	adrp	x8, lCPI89_1@PAGE
Lloh483:
	ldr	d7, [x8, lCPI89_1@PAGEOFF]
	fadd.2s	v5, v5, v7
	mov.s	v6[0], v4[0]
	fmul.2s	v4, v6, v5
	faddp.2s	s4, v4
	fmov	s5, #0.50000000
	fmul	s5, s0, s5
	fmul	s5, s0, s5
	fadd	s4, s5, s4
	fmul	s3, s3, s4
	mov	w8, #63441
	movk	w8, #14103, lsl #16
	fmov	s4, w8
	fmul	s4, s2, s4
	fadd	s1, s1, s4
	fadd	s1, s1, s3
	fsub	s1, s1, s5
	fadd	s0, s0, s1
	mov	w8, #29056
	movk	w8, #16177, lsl #16
	fmov	s1, w8
	fmul	s1, s2, s1
	fadd	s0, s1, s0
	ret
LBB89_4:
	lsr	w9, w8, #23
	cmp	w9, #382
	b.ls	LBB89_7
	fmov	s1, #-1.00000000
	fcmp	s0, s1
	b.ne	LBB89_12
	mov	w8, #-8388608
	fmov	s0, w8
	ret
LBB89_7:
	ubfx	w9, w8, #23, #8
	cmp	w9, #103
	b.hs	LBB89_10
	tst	w8, #0x7f800000
	b.eq	LBB89_13
LBB89_9:
	ret
LBB89_10:
	movi.2d	v1, #0000000000000000
	mov	w9, #63001
	movk	w9, #48789, lsl #16
	cmp	w8, w9
	b.hi	LBB89_2
	movi.2d	v2, #0000000000000000
	b	LBB89_3
LBB89_12:
	fsub	s0, s0, s0
	movi.2d	v1, #0000000000000000
	fdiv	s0, s0, s1
	ret
LBB89_13:
	sub	sp, sp, #16
	.cfi_def_cfa_offset 16
	fmul	s1, s0, s0
	str	s1, [sp, #12]
	ldr	s1, [sp, #12]
	add	sp, sp, #16
	.cfi_def_cfa_offset 0
	ret
	.loh AdrpLdr	Lloh482, Lloh483
	.loh AdrpAdrp	Lloh480, Lloh482
	.loh AdrpLdr	Lloh480, Lloh481
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math6remquo6remquo
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math6remquo6remquo:
	.cfi_startproc
	fcmp	d1, #0.0
	b.eq	LBB90_4
	b.vs	LBB90_4
	fmov	x8, d0
	ubfx	x10, x8, #52, #11
	cmp	w10, #2047
	b.eq	LBB90_4
	fcmp	d0, #0.0
	b.ne	LBB90_5
	mov	w0, #0
	ret
LBB90_4:
	mov	w0, #0
	fmul	d0, d0, d1
	fdiv	d0, d0, d0
	ret
LBB90_5:
	fmov	x9, d1
	ubfx	x11, x9, #52, #11
	cbz	w10, LBB90_13
	mov	x12, #4503599627370496
	bfxil	x12, x8, #0, #52
	cbz	w11, LBB90_16
LBB90_7:
	mov	x13, #4503599627370496
	bfxil	x13, x9, #0, #52
	add	w14, w10, #1
	cmp	w14, w11
	b.ne	LBB90_19
LBB90_8:
	mov	w13, #0
LBB90_9:
	cmp	w10, #0
	b.le	LBB90_28
	mov	x14, #-4503599627370496
	add	x12, x12, x14
	orr	x12, x12, x10, lsl #52
	fabs	d0, d1
	cmp	w10, w11
	b.ne	LBB90_29
LBB90_11:
	fmov	d1, x12
LBB90_12:
	fsub	d0, d1, d0
	fmov	x12, d0
	add	w13, w13, #1
	b	LBB90_32
LBB90_13:
	mov	w10, #0
	lsl	x12, x8, #12
	tbnz	x12, #63, LBB90_15
LBB90_14:
	sub	w10, w10, #1
	lsl	x12, x12, #1
	tbz	x12, #63, LBB90_14
LBB90_15:
	mov	w12, #1
	sub	w12, w12, w10
	lsl	x12, x8, x12
	cbnz	w11, LBB90_7
LBB90_16:
	mov	w11, #0
	lsl	x13, x9, #12
	tbnz	x13, #63, LBB90_18
LBB90_17:
	sub	w11, w11, #1
	lsl	x13, x13, #1
	tbz	x13, #63, LBB90_17
LBB90_18:
	mov	w13, #1
	sub	w13, w13, w11
	lsl	x13, x9, x13
	add	w14, w10, #1
	cmp	w14, w11
	b.eq	LBB90_8
LBB90_19:
	cmp	w10, w11
	b.ge	LBB90_21
	mov	w0, #0
	ret
LBB90_21:
	mov	w14, #0
	b.le	LBB90_24
LBB90_22:
	sub	x15, x12, x13
	cmn	x15, #1
	csel	x12, x15, x12, gt
	lsr	x15, x15, #63
	eor	w15, w15, #0x1
	orr	w14, w14, w15
	lsl	x12, x12, #1
	lsl	w14, w14, #1
	sub	w10, w10, #1
	cmp	w10, w11
	b.gt	LBB90_22
	mov	x10, x11
LBB90_24:
	sub	x13, x12, x13
	cmn	x13, #1
	csel	x12, x13, x12, gt
	lsr	x13, x13, #63
	eor	w13, w13, #0x1
	orr	w13, w14, w13
	cbz	x12, LBB90_27
	lsr	x14, x12, #52
	cbnz	x14, LBB90_9
LBB90_26:
	lsr	x14, x12, #51
	lsl	x12, x12, #1
	sub	w10, w10, #1
	cbz	x14, LBB90_26
	b	LBB90_9
LBB90_27:
	mov	w10, #-60
LBB90_28:
	mov	w14, #1
	sub	w14, w14, w10
	lsr	x12, x12, x14
	fabs	d0, d1
	cmp	w10, w11
	b.eq	LBB90_11
LBB90_29:
	add	w10, w10, #1
	cmp	w10, w11
	b.ne	LBB90_32
	fmov	d1, x12
	fadd	d2, d1, d1
	fcmp	d2, d0
	b.gt	LBB90_12
	and	w10, w13, #0x1
	ccmp	w10, #0, #4, eq
	b.ne	LBB90_12
LBB90_32:
	and	w10, w13, #0x7fffffff
	eor	x9, x9, x8
	cmp	x9, #0
	cneg	w0, w10, mi
	fmov	d0, x12
	fneg	d1, d0
	cmp	x8, #0
	fcsel	d0, d1, d0, mi
	ret
	.cfi_endproc

	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0
lCPI91_0:
	.quad	0x3de5d93a5acfd57c
	.quad	0x3ec71de357b1fe7d
lCPI91_1:
	.quad	0xbe5ae5e68a2b9ceb
	.quad	0xbf2a01a019c161d5
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math6sincos6sincos
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math6sincos6sincos:
	.cfi_startproc
	sub	sp, sp, #48
	.cfi_def_cfa_offset 48
	stp	x29, x30, [sp, #32]
	add	x29, sp, #32
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_remember_state
	fmov	x8, d0
	ubfx	x8, x8, #32, #31
	mov	w9, #8700
	movk	w9, #16361, lsl #16
	cmp	w8, w9
	b.hs	LBB91_4
	mov	w9, #41118
	movk	w9, #15942, lsl #16
	cmp	w8, w9
	b.hs	LBB91_6
	cmp	w8, #256, lsl #12
	b.hs	LBB91_10
	mov	x8, #4066750463515557888
	fmov	d1, x8
	fmul	d1, d0, d1
	b	LBB91_11
LBB91_4:
	lsr	w8, w8, #20
	cmp	w8, #2046
	b.ls	LBB91_7
	fsub	d1, d0, d0
	mov.16b	v0, v1
	b	LBB91_12
LBB91_6:
	fmul	d1, d0, d0
	fmul	d2, d1, d1
	dup.2d	v3, v1[0]
	fmul	d4, d1, d2
Lloh484:
	adrp	x8, lCPI91_0@PAGE
Lloh485:
	ldr	q5, [x8, lCPI91_0@PAGEOFF]
	fmul.2d	v5, v5, v1[0]
Lloh486:
	adrp	x8, lCPI91_1@PAGE
Lloh487:
	ldr	q6, [x8, lCPI91_1@PAGEOFF]
	fadd.2d	v5, v5, v6
	mov.d	v3[0], v4[0]
	fmul.2d	v3, v3, v5
	mov	d4, v3[1]
	mov	x8, #63654
	movk	x8, #4368, lsl #16
	movk	x8, #4369, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fadd	d3, d3, d4
	fmul	d4, d0, d1
	fmul	d3, d1, d3
	mov	x8, #6148914691236517205
	movk	x8, #21833
	movk	x8, #49093, lsl #48
	fmov	d5, x8
	fadd	d3, d3, d5
	fmul	d3, d4, d3
	mov	x8, #-9223372036854775808
	fmov	d4, x8
	fmul	d4, d0, d4
	fadd	d0, d0, d3
	mov	x8, #5520
	movk	x8, #6603, lsl #16
	movk	x8, #416, lsl #32
	movk	x8, #16122, lsl #48
	fmov	d3, x8
	fmul	d3, d1, d3
	mov	x8, #20855
	movk	x8, #5825, lsl #16
	movk	x8, #49516, lsl #32
	movk	x8, #48982, lsl #48
	fmov	d5, x8
	fadd	d3, d3, d5
	fmul	d3, d1, d3
	mov	x8, #6148914691236517205
	movk	x8, #21836
	movk	x8, #16293, lsl #48
	fmov	d5, x8
	fadd	d3, d3, d5
	fmul	d3, d1, d3
	fmul	d2, d2, d2
	mov	x8, #14548
	movk	x8, #48776, lsl #16
	movk	x8, #64233, lsl #32
	movk	x8, #48552, lsl #48
	fmov	d5, x8
	fmul	d5, d1, d5
	mov	x8, #45508
	movk	x8, #48564, lsl #16
	movk	x8, #61086, lsl #32
	movk	x8, #15905, lsl #48
	fmov	d6, x8
	fadd	d5, d5, d6
	fmul	d5, d1, d5
	mov	x8, #21165
	movk	x8, #32924, lsl #16
	movk	x8, #32335, lsl #32
	movk	x8, #48786, lsl #48
	fmov	d6, x8
	fadd	d5, d5, d6
	fmul	d2, d2, d5
	fadd	d2, d3, d2
	fmov	d3, #0.50000000
	fmul	d3, d1, d3
	fmov	d5, #1.00000000
	fsub	d6, d5, d3
	fsub	d5, d5, d6
	fsub	d3, d5, d3
	fmul	d1, d1, d2
	fadd	d1, d1, d4
	fadd	d1, d3, d1
	fadd	d1, d6, d1
	b	LBB91_12
LBB91_7:
	add	x0, sp, #8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math8rem_pio28rem_pio2
	ldr	w8, [sp, #16]
	ldr	d0, [sp, #8]
	ldr	d1, [sp, #24]
	fmul	d3, d0, d0
	fmul	d4, d3, d3
	fmul	d2, d3, d4
	dup.2d	v5, v3[0]
Lloh488:
	adrp	x9, lCPI91_0@PAGE
Lloh489:
	ldr	q6, [x9, lCPI91_0@PAGEOFF]
	fmul.2d	v6, v6, v3[0]
Lloh490:
	adrp	x9, lCPI91_1@PAGE
Lloh491:
	ldr	q7, [x9, lCPI91_1@PAGEOFF]
	fadd.2d	v6, v6, v7
	mov.d	v5[0], v2[0]
	fmul.2d	v2, v5, v6
	mov	d5, v2[1]
	mov	x9, #63654
	movk	x9, #4368, lsl #16
	movk	x9, #4369, lsl #32
	movk	x9, #16257, lsl #48
	fmov	d6, x9
	fadd	d5, d5, d6
	fadd	d2, d2, d5
	fmul	d5, d0, d3
	fmov	d6, #0.50000000
	fmul	d7, d1, d6
	fmul	d2, d5, d2
	fsub	d2, d7, d2
	fmul	d2, d3, d2
	fsub	d2, d2, d1
	mov	x9, #6148914691236517205
	movk	x9, #21833
	movk	x9, #16325, lsl #48
	fmov	d7, x9
	fmul	d5, d5, d7
	fadd	d2, d5, d2
	fsub	d2, d0, d2
	mov	x9, #5520
	movk	x9, #6603, lsl #16
	movk	x9, #416, lsl #32
	movk	x9, #16122, lsl #48
	fmov	d5, x9
	fmul	d5, d3, d5
	mov	x9, #20855
	movk	x9, #5825, lsl #16
	movk	x9, #49516, lsl #32
	movk	x9, #48982, lsl #48
	fmov	d7, x9
	fadd	d5, d5, d7
	fmul	d5, d3, d5
	mov	x9, #6148914691236517205
	movk	x9, #21836
	movk	x9, #16293, lsl #48
	fmov	d7, x9
	fadd	d5, d5, d7
	fmul	d5, d3, d5
	fmul	d4, d4, d4
	mov	x9, #14548
	movk	x9, #48776, lsl #16
	movk	x9, #64233, lsl #32
	movk	x9, #48552, lsl #48
	fmov	d7, x9
	fmul	d7, d3, d7
	mov	x9, #45508
	movk	x9, #48564, lsl #16
	movk	x9, #61086, lsl #32
	movk	x9, #15905, lsl #48
	fmov	d16, x9
	fadd	d7, d7, d16
	fmul	d7, d3, d7
	mov	x9, #21165
	movk	x9, #32924, lsl #16
	movk	x9, #32335, lsl #32
	movk	x9, #48786, lsl #48
	fmov	d16, x9
	fadd	d7, d7, d16
	fmul	d4, d4, d7
	fadd	d4, d5, d4
	fmul	d5, d3, d6
	fmov	d6, #1.00000000
	fsub	d7, d6, d5
	fsub	d6, d6, d7
	fsub	d5, d6, d5
	fmul	d3, d3, d4
	fmul	d0, d0, d1
	fsub	d0, d3, d0
	fadd	d0, d5, d0
	fadd	d0, d7, d0
	and	w8, w8, #0x3
	cmp	w8, #1
	b.gt	LBB91_13
	cbnz	w8, LBB91_15
	mov.16b	v1, v0
	mov.16b	v0, v2
	b	LBB91_12
LBB91_10:
	mov	x8, #5147614374084476928
	fmov	d1, x8
	fadd	d1, d0, d1
LBB91_11:
	str	d1, [sp, #8]
	ldr	d1, [sp, #8]
	fmov	d1, #1.00000000
LBB91_12:
	.cfi_def_cfa wsp, 48
	ldp	x29, x30, [sp, #32]
	add	sp, sp, #48
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
LBB91_13:
	.cfi_restore_state
	cmp	w8, #2
	b.ne	LBB91_16
	fneg	d1, d0
	fneg	d0, d2
	b	LBB91_12
LBB91_15:
	fneg	d1, d2
	b	LBB91_12
LBB91_16:
	fneg	d0, d0
	mov.16b	v1, v2
	b	LBB91_12
	.loh AdrpLdr	Lloh486, Lloh487
	.loh AdrpAdrp	Lloh484, Lloh486
	.loh AdrpLdr	Lloh484, Lloh485
	.loh AdrpLdr	Lloh490, Lloh491
	.loh AdrpAdrp	Lloh488, Lloh490
	.loh AdrpLdr	Lloh488, Lloh489
	.cfi_endproc

	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0
lCPI92_0:
	.quad	0x40040d931ff62705
	.quad	0x3ff0000000000000
lCPI92_1:
	.quad	0x406a5a607bbc3b52
	.quad	0x4050800000000000
lCPI92_2:
	.quad	0x40bf87ac0858d804
	.quad	0x409e140000000000
lCPI92_3:
	.quad	0x4106b6421f8787eb
	.quad	0x40dfe78000000000
lCPI92_4:
	.quad	0x4145f1e95080f44c
	.quad	0x4115d0bc00000000
lCPI92_5:
	.quad	0x417df876f95dcc98
	.quad	0x41441f7b00000000
lCPI92_6:
	.quad	0x41adab0c7bb95f2a
	.quad	0x41697171e0000000
lCPI92_7:
	.quad	0x41d57418f5d3f33f
	.quad	0x4185eeb690000000
lCPI92_8:
	.quad	0x41f67fc1b3a5a1e8
	.quad	0x4199187170000000
lCPI92_9:
	.quad	0x4210b0b522e8261a
	.quad	0x41a1fda6b0000000
lCPI92_10:
	.quad	0x4220a132818ab61a
	.quad	0x419cbd6980000000
lCPI92_11:
	.quad	0x4223fc7075f54c57
	.quad	0x418308a800000000
lCPI92_12:
	.quad	0x4215ea5143c1a49e
	.quad	0x0000000000000000
lCPI92_13:
	.quad	0x3de5d93a5acfd57c
	.quad	0x3ec71de357b1fe7d
lCPI92_14:
	.quad	0xbe5ae5e68a2b9ceb
	.quad	0xbf2a01a019c161d5
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math6tgamma6tgamma
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math6tgamma6tgamma:
	.cfi_startproc
	sub	sp, sp, #96
	.cfi_def_cfa_offset 96
	stp	d11, d10, [sp, #48]
	stp	d9, d8, [sp, #64]
	stp	x29, x30, [sp, #80]
	add	x29, sp, #80
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset b8, -24
	.cfi_offset b9, -32
	.cfi_offset b10, -40
	.cfi_offset b11, -48
	fmov	x9, d0
	lsr	x8, x9, #32
	ubfx	w8, w8, #20, #11
	cmp	w8, #2046
	b.ls	LBB92_2
	mov	x8, #9218868437227405312
	fmov	d1, x8
	fadd	d0, d0, d1
	b	LBB92_45
LBB92_2:
	ubfx	x8, x9, #32, #31
	lsr	w10, w8, #20
	cmp	w10, #969
	b.hs	LBB92_4
	fmov	d1, #1.00000000
	fdiv	d0, d1, d0
	b	LBB92_45
LBB92_4:
	ubfx	x10, x9, #52, #11
	mov.16b	v1, v0
	cmp	w10, #1074
	b.hi	LBB92_8
	cmp	w10, #1022
	b.ls	LBB92_12
	sub	w10, w10, #1023
	mov	x12, #4503599627370495
	lsr	x11, x12, x10
	mov.16b	v1, v0
	tst	x11, x9
	b.eq	LBB92_8
	and	x11, x11, x9, asr #63
	mov	x12, #-4503599627370496
	asr	x10, x12, x10
	add	x11, x11, x9
	and	x10, x11, x10
	fmov	d1, x10
LBB92_8:
	fcmp	d0, d1
	b.ne	LBB92_10
	tbz	x9, #63, LBB92_14
	b	LBB92_18
LBB92_10:
	mov	w10, #1080492031
	cmp	w8, w10
	b.ls	LBB92_23
	tbz	x9, #63, LBB92_20
	b	LBB92_22
LBB92_12:
	fcmp	d0, #0.0
	tbnz	x9, #63, LBB92_17
	b.ne	LBB92_19
LBB92_14:
	fmov	d1, #23.00000000
	fcmp	d0, d1
	b.ls	LBB92_16
	mov	w9, #1080492031
	mov.16b	v4, v0
	cmp	w8, w9
	b.hi	LBB92_20
	b	LBB92_24
LBB92_16:
	fcvtzu	x8, d0
Lloh492:
	adrp	x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.97@PAGE
Lloh493:
	add	x9, x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.97@PAGEOFF
	add	x8, x9, x8, lsl #3
	ldur	d0, [x8, #-8]
	b	LBB92_45
LBB92_17:
	fmov	d1, #-1.00000000
	fcsel	d1, d0, d1, eq
	fcmp	d0, d1
	b.ne	LBB92_21
LBB92_18:
	mov	x8, #9221120237041090560
	fmov	d0, x8
	b	LBB92_45
LBB92_19:
	mov	w9, #1080492031
	cmp	w8, w9
	b.ls	LBB92_23
LBB92_20:
	mov	x8, #9214364837600034816
	fmov	d1, x8
	b	LBB92_44
LBB92_21:
	mov	w9, #1080492031
	cmp	w8, w9
	b.ls	LBB92_23
LBB92_22:
	mov	x8, #4039728865751334912
	fmov	d1, x8
	fdiv	d1, d1, d0
	fcvt	s1, d1
	stur	s1, [x29, #-36]
	ldur	s1, [x29, #-36]
	mov.16b	v8, v0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5floor5floor
	fmov	d1, #0.50000000
	fmul	d9, d0, d1
	fmul	d0, d8, d1
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math5floor5floor
	fcmp	d9, d0
	mov	x8, #-9223372036854775808
	fmov	d0, x8
	movi.2d	v1, #0000000000000000
	fcsel	d0, d1, d0, eq
	b	LBB92_45
LBB92_23:
	fabs	d4, d0
LBB92_24:
	mov	x8, #3112173568
	movk	x8, #6469, lsl #32
	movk	x8, #16406, lsl #48
	fmov	d1, x8
	fadd	d8, d4, d1
	fsub	d2, d8, d4
	mov	x8, #3112173568
	movk	x8, #6469, lsl #32
	movk	x8, #49174, lsl #48
	fmov	d3, x8
	fadd	d2, d2, d3
	fadd	d3, d8, d3
	fsub	d3, d3, d4
	fcmp	d4, d1
	fcsel	d9, d2, d3, gt
	fmov	d1, #-0.50000000
	dup.2d	v2, v4[0]
	fmov	d3, #8.00000000
	mov.16b	v11, v0
	fcmp	d4, d3
	b.pl	LBB92_26
Lloh494:
	adrp	x8, lCPI92_0@PAGE
Lloh495:
	ldr	q0, [x8, lCPI92_0@PAGEOFF]
	fmul.2d	v0, v0, v2[0]
Lloh496:
	adrp	x8, lCPI92_1@PAGE
Lloh497:
	ldr	q3, [x8, lCPI92_1@PAGEOFF]
	fadd.2d	v0, v0, v3
	fmul.2d	v0, v0, v2[0]
Lloh498:
	adrp	x8, lCPI92_2@PAGE
Lloh499:
	ldr	q3, [x8, lCPI92_2@PAGEOFF]
	fadd.2d	v0, v0, v3
	fmul.2d	v0, v0, v2[0]
Lloh500:
	adrp	x8, lCPI92_3@PAGE
Lloh501:
	ldr	q3, [x8, lCPI92_3@PAGEOFF]
	fadd.2d	v0, v0, v3
	fmul.2d	v0, v0, v2[0]
Lloh502:
	adrp	x8, lCPI92_4@PAGE
Lloh503:
	ldr	q3, [x8, lCPI92_4@PAGEOFF]
	fadd.2d	v0, v0, v3
	fmul.2d	v0, v0, v2[0]
Lloh504:
	adrp	x8, lCPI92_5@PAGE
Lloh505:
	ldr	q3, [x8, lCPI92_5@PAGEOFF]
	fadd.2d	v0, v0, v3
	fmul.2d	v0, v0, v2[0]
Lloh506:
	adrp	x8, lCPI92_6@PAGE
Lloh507:
	ldr	q3, [x8, lCPI92_6@PAGEOFF]
	fadd.2d	v0, v0, v3
	fmul.2d	v0, v0, v2[0]
Lloh508:
	adrp	x8, lCPI92_7@PAGE
Lloh509:
	ldr	q3, [x8, lCPI92_7@PAGEOFF]
	fadd.2d	v0, v0, v3
	fmul.2d	v0, v0, v2[0]
Lloh510:
	adrp	x8, lCPI92_8@PAGE
Lloh511:
	ldr	q3, [x8, lCPI92_8@PAGEOFF]
	fadd.2d	v0, v0, v3
	fmul.2d	v0, v0, v2[0]
Lloh512:
	adrp	x8, lCPI92_9@PAGE
Lloh513:
	ldr	q3, [x8, lCPI92_9@PAGEOFF]
	fadd.2d	v0, v0, v3
	fmul.2d	v0, v0, v2[0]
Lloh514:
	adrp	x8, lCPI92_10@PAGE
Lloh515:
	ldr	q3, [x8, lCPI92_10@PAGEOFF]
	fadd.2d	v0, v0, v3
	fmul.2d	v0, v0, v2[0]
Lloh516:
	adrp	x8, lCPI92_11@PAGE
Lloh517:
	ldr	q3, [x8, lCPI92_11@PAGEOFF]
	fadd.2d	v0, v0, v3
	fmul.2d	v0, v0, v2[0]
Lloh518:
	adrp	x8, lCPI92_12@PAGE
Lloh519:
	ldr	q2, [x8, lCPI92_12@PAGEOFF]
	fadd.2d	v0, v0, v2
	b	LBB92_29
LBB92_26:
	mov	x10, #0
	movi.2d	v0, #0000000000000000
Lloh520:
	adrp	x8, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.95@PAGE
Lloh521:
	add	x8, x8, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.95@PAGEOFF
Lloh522:
	adrp	x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.96@PAGE
Lloh523:
	add	x9, x9, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.96@PAGEOFF
LBB92_27:
	fdiv.2d	v0, v0, v2
	lsl	x11, x10, #3
	ldr	d3, [x8, x11]
	add	x11, x9, x11
	ld1.d	{ v3 }[1], [x11]
	fadd.2d	v0, v0, v3
	cmp	x10, #12
	cinc	x11, x10, ne
	b.eq	LBB92_29
	mov	x10, x11
	cmp	x11, #13
	b.lo	LBB92_27
LBB92_29:
	fadd	d10, d4, d1
	dup.2d	v1, v0[1]
	fdiv.2d	v0, v0, v1
	stp	q4, q0, [sp]
	fneg	d0, d8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3exp3exp
	ldr	q1, [sp, #16]
	fmul	d0, d0, d1
	fcmp	d11, #0.0
	b.pl	LBB92_43
	fmov	d1, #0.50000000
	ldr	q7, [sp]
	fmul	d1, d7, d1
	fmov	x8, d1
	ubfx	x9, x8, #52, #11
	mov.16b	v2, v1
	cmp	w9, #1074
	b.hi	LBB92_35
	cmp	w9, #1022
	b.ls	LBB92_34
	sub	w9, w9, #1023
	mov	x11, #4503599627370495
	lsr	x10, x11, x9
	mov.16b	v2, v1
	tst	x10, x8
	b.eq	LBB92_35
	and	x10, x10, x8, asr #63
	mov	x11, #-4503599627370496
	asr	x9, x11, x9
	add	x8, x10, x8
	and	x8, x8, x9
	fmov	d2, x8
	b	LBB92_35
LBB92_34:
	cmn	x8, #1
	fmov	d2, #-1.00000000
	movi.2d	v3, #0000000000000000
	fcsel	d2, d3, d2, gt
LBB92_35:
	fsub	d1, d1, d2
	fadd	d1, d1, d1
	fcvtzs	x8, d1, #2
	add	x8, x8, #1
	add	x8, x8, x8, lsr #63
	asr	x8, x8, #1
	scvtf	d2, x8
	fmov	d3, #-0.50000000
	fmul	d2, d2, d3
	fadd	d1, d1, d2
	mov	x9, #11544
	movk	x9, #21572, lsl #16
	movk	x9, #8699, lsl #32
	movk	x9, #16393, lsl #48
	fmov	d2, x9
	fmul	d1, d1, d2
	cmp	x8, #1
	b.eq	LBB92_40
	cmp	x8, #2
	b.eq	LBB92_39
	cmp	x8, #3
	b.ne	LBB92_41
	fmul	d2, d1, d1
	fmul	d3, d2, d2
	mov	x8, #5520
	movk	x8, #6603, lsl #16
	movk	x8, #416, lsl #32
	movk	x8, #16122, lsl #48
	fmov	d4, x8
	fmul	d4, d2, d4
	mov	x8, #20855
	movk	x8, #5825, lsl #16
	movk	x8, #49516, lsl #32
	movk	x8, #48982, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d4, d2, d4
	mov	x8, #6148914691236517205
	movk	x8, #21836
	movk	x8, #16293, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d4, d2, d4
	fmul	d3, d3, d3
	mov	x8, #14548
	movk	x8, #48776, lsl #16
	movk	x8, #64233, lsl #32
	movk	x8, #48552, lsl #48
	fmov	d5, x8
	fmul	d5, d2, d5
	mov	x8, #45508
	movk	x8, #48564, lsl #16
	movk	x8, #61086, lsl #32
	movk	x8, #15905, lsl #48
	fmov	d6, x8
	fadd	d5, d5, d6
	fmul	d5, d2, d5
	mov	x8, #21165
	movk	x8, #32924, lsl #16
	movk	x8, #32335, lsl #32
	movk	x8, #48786, lsl #48
	fmov	d6, x8
	fadd	d5, d5, d6
	fmul	d3, d3, d5
	fadd	d3, d4, d3
	fmov	d4, #0.50000000
	fmul	d4, d2, d4
	fmov	d5, #1.00000000
	fsub	d6, d5, d4
	fsub	d5, d5, d6
	fsub	d4, d5, d4
	fmul	d2, d2, d3
	mov	x8, #-9223372036854775808
	fmov	d3, x8
	fmul	d1, d1, d3
	fadd	d1, d2, d1
	fadd	d1, d4, d1
	fadd	d1, d6, d1
	fneg	d1, d1
	b	LBB92_42
LBB92_39:
	fmul	d2, d1, d1
	fmul	d3, d2, d2
	fmul	d3, d2, d3
	dup.2d	v4, v2[0]
Lloh524:
	adrp	x8, lCPI92_13@PAGE
Lloh525:
	ldr	q5, [x8, lCPI92_13@PAGEOFF]
	fmul.2d	v5, v5, v2[0]
Lloh526:
	adrp	x8, lCPI92_14@PAGE
Lloh527:
	ldr	q6, [x8, lCPI92_14@PAGEOFF]
	fadd.2d	v5, v5, v6
	mov.d	v4[0], v3[0]
	fmul.2d	v3, v4, v5
	mov	d4, v3[1]
	mov	x8, #63654
	movk	x8, #4368, lsl #16
	movk	x8, #4369, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fadd	d3, d3, d4
	fnmul	d4, d1, d2
	fmul	d2, d2, d3
	mov	x8, #6148914691236517205
	movk	x8, #21833
	movk	x8, #49093, lsl #48
	fmov	d3, x8
	fadd	d2, d2, d3
	fmul	d2, d4, d2
	fsub	d1, d2, d1
	b	LBB92_42
LBB92_40:
	fmul	d2, d1, d1
	fmul	d3, d2, d2
	mov	x8, #5520
	movk	x8, #6603, lsl #16
	movk	x8, #416, lsl #32
	movk	x8, #16122, lsl #48
	fmov	d4, x8
	fmul	d4, d2, d4
	mov	x8, #20855
	movk	x8, #5825, lsl #16
	movk	x8, #49516, lsl #32
	movk	x8, #48982, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d4, d2, d4
	mov	x8, #6148914691236517205
	movk	x8, #21836
	movk	x8, #16293, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d4, d2, d4
	fmul	d3, d3, d3
	mov	x8, #14548
	movk	x8, #48776, lsl #16
	movk	x8, #64233, lsl #32
	movk	x8, #48552, lsl #48
	fmov	d5, x8
	fmul	d5, d2, d5
	mov	x8, #45508
	movk	x8, #48564, lsl #16
	movk	x8, #61086, lsl #32
	movk	x8, #15905, lsl #48
	fmov	d6, x8
	fadd	d5, d5, d6
	fmul	d5, d2, d5
	mov	x8, #21165
	movk	x8, #32924, lsl #16
	movk	x8, #32335, lsl #32
	movk	x8, #48786, lsl #48
	fmov	d6, x8
	fadd	d5, d5, d6
	fmul	d3, d3, d5
	fadd	d3, d4, d3
	fmov	d4, #0.50000000
	fmul	d4, d2, d4
	fmov	d5, #1.00000000
	fsub	d6, d5, d4
	fsub	d5, d5, d6
	fsub	d4, d5, d4
	fmul	d2, d2, d3
	mov	x8, #-9223372036854775808
	fmov	d3, x8
	fmul	d1, d1, d3
	fadd	d1, d2, d1
	fadd	d1, d4, d1
	fadd	d1, d6, d1
	b	LBB92_42
LBB92_41:
	fmul	d2, d1, d1
	fmul	d3, d2, d2
	fmul	d3, d2, d3
	dup.2d	v4, v2[0]
Lloh528:
	adrp	x8, lCPI92_13@PAGE
Lloh529:
	ldr	q5, [x8, lCPI92_13@PAGEOFF]
	fmul.2d	v5, v5, v2[0]
Lloh530:
	adrp	x8, lCPI92_14@PAGE
Lloh531:
	ldr	q6, [x8, lCPI92_14@PAGEOFF]
	fadd.2d	v5, v5, v6
	mov.d	v4[0], v3[0]
	fmul.2d	v3, v4, v5
	mov	d4, v3[1]
	mov	x8, #63654
	movk	x8, #4368, lsl #16
	movk	x8, #4369, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fadd	d3, d3, d4
	fmul	d4, d1, d2
	fmul	d2, d2, d3
	mov	x8, #6148914691236517205
	movk	x8, #21833
	movk	x8, #49093, lsl #48
	fmov	d3, x8
	fadd	d2, d2, d3
	fmul	d2, d4, d2
	fadd	d1, d1, d2
LBB92_42:
	fmul	d1, d7, d1
	fmul	d0, d0, d1
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #49161, lsl #48
	fmov	d1, x8
	fdiv	d0, d1, d0
	fneg	d9, d9
	fneg	d10, d10
LBB92_43:
	mov	x8, #3112173568
	movk	x8, #6469, lsl #32
	movk	x8, #16408, lsl #48
	fmov	d1, x8
	fmul	d1, d9, d1
	fmul	d1, d0, d1
	fdiv	d1, d1, d8
	fadd	d9, d0, d1
	fmov	d0, #0.50000000
	fmul	d1, d10, d0
	mov.16b	v0, v8
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3pow3pow
	fmul	d1, d0, d9
LBB92_44:
	fmul	d0, d0, d1
LBB92_45:
	.cfi_def_cfa wsp, 96
	ldp	x29, x30, [sp, #80]
	ldp	d9, d8, [sp, #64]
	ldp	d11, d10, [sp, #48]
	add	sp, sp, #96
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore b8
	.cfi_restore b9
	.cfi_restore b10
	.cfi_restore b11
	ret
	.loh AdrpAdd	Lloh492, Lloh493
	.loh AdrpLdr	Lloh518, Lloh519
	.loh AdrpAdrp	Lloh516, Lloh518
	.loh AdrpLdr	Lloh516, Lloh517
	.loh AdrpAdrp	Lloh514, Lloh516
	.loh AdrpLdr	Lloh514, Lloh515
	.loh AdrpAdrp	Lloh512, Lloh514
	.loh AdrpLdr	Lloh512, Lloh513
	.loh AdrpAdrp	Lloh510, Lloh512
	.loh AdrpLdr	Lloh510, Lloh511
	.loh AdrpAdrp	Lloh508, Lloh510
	.loh AdrpLdr	Lloh508, Lloh509
	.loh AdrpAdrp	Lloh506, Lloh508
	.loh AdrpLdr	Lloh506, Lloh507
	.loh AdrpAdrp	Lloh504, Lloh506
	.loh AdrpLdr	Lloh504, Lloh505
	.loh AdrpAdrp	Lloh502, Lloh504
	.loh AdrpLdr	Lloh502, Lloh503
	.loh AdrpAdrp	Lloh500, Lloh502
	.loh AdrpLdr	Lloh500, Lloh501
	.loh AdrpAdrp	Lloh498, Lloh500
	.loh AdrpLdr	Lloh498, Lloh499
	.loh AdrpAdrp	Lloh496, Lloh498
	.loh AdrpLdr	Lloh496, Lloh497
	.loh AdrpAdrp	Lloh494, Lloh496
	.loh AdrpLdr	Lloh494, Lloh495
	.loh AdrpAdd	Lloh522, Lloh523
	.loh AdrpAdd	Lloh520, Lloh521
	.loh AdrpLdr	Lloh526, Lloh527
	.loh AdrpAdrp	Lloh524, Lloh526
	.loh AdrpLdr	Lloh524, Lloh525
	.loh AdrpLdr	Lloh530, Lloh531
	.loh AdrpAdrp	Lloh528, Lloh530
	.loh AdrpLdr	Lloh528, Lloh529
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math7lgammaf7lgammaf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math7lgammaf7lgammaf:
	.cfi_startproc
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math9lgammaf_r9lgammaf_r
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math7remquof7remquof
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math7remquof7remquof:
	.cfi_startproc
	fcmp	s1, #0.0
	b.eq	LBB94_4
	b.vs	LBB94_4
	fmov	w8, s0
	ubfx	w10, w8, #23, #8
	cmp	w10, #255
	b.eq	LBB94_4
	fcmp	s0, #0.0
	b.ne	LBB94_5
	mov	w0, #0
	ret
LBB94_4:
	mov	w0, #0
	fmul	s0, s0, s1
	fdiv	s0, s0, s0
	ret
LBB94_5:
	fmov	w9, s1
	ubfx	w11, w9, #23, #8
	cbz	w10, LBB94_13
	mov	w12, #8388608
	bfxil	w12, w8, #0, #23
	cbz	w11, LBB94_16
LBB94_7:
	mov	w13, #8388608
	bfxil	w13, w9, #0, #23
	add	w14, w10, #1
	cmp	w14, w11
	b.ne	LBB94_19
LBB94_8:
	mov	w13, #0
LBB94_9:
	cmp	w10, #0
	b.le	LBB94_28
	sub	w12, w12, #2048, lsl #12
	orr	w12, w12, w10, lsl #23
	fabs	s0, s1
	cmp	w10, w11
	b.ne	LBB94_29
LBB94_11:
	fmov	s1, w12
LBB94_12:
	fsub	s0, s1, s0
	fmov	w12, s0
	add	w13, w13, #1
	b	LBB94_32
LBB94_13:
	mov	w10, #0
	lsl	w12, w8, #9
	tbnz	w12, #31, LBB94_15
LBB94_14:
	sub	w10, w10, #1
	lsl	w12, w12, #1
	tbz	w12, #31, LBB94_14
LBB94_15:
	mov	w12, #1
	sub	w12, w12, w10
	lsl	w12, w8, w12
	cbnz	w11, LBB94_7
LBB94_16:
	mov	w11, #0
	lsl	w13, w9, #9
	tbnz	w13, #31, LBB94_18
LBB94_17:
	sub	w11, w11, #1
	lsl	w13, w13, #1
	tbz	w13, #31, LBB94_17
LBB94_18:
	mov	w13, #1
	sub	w13, w13, w11
	lsl	w13, w9, w13
	add	w14, w10, #1
	cmp	w14, w11
	b.eq	LBB94_8
LBB94_19:
	cmp	w10, w11
	b.ge	LBB94_21
	mov	w0, #0
	ret
LBB94_21:
	mov	w14, #0
	b.le	LBB94_24
LBB94_22:
	sub	w15, w12, w13
	mvn	w16, w15
	cmn	w15, #1
	csel	w12, w15, w12, gt
	orr	w14, w14, w16, lsr #31
	lsl	w12, w12, #1
	lsl	w14, w14, #1
	sub	w10, w10, #1
	cmp	w10, w11
	b.gt	LBB94_22
	mov	x10, x11
LBB94_24:
	sub	w13, w12, w13
	mvn	w15, w13
	cmn	w13, #1
	csel	w12, w13, w12, gt
	orr	w13, w14, w15, lsr #31
	cbz	w12, LBB94_27
	lsr	w14, w12, #23
	cbnz	w14, LBB94_9
LBB94_26:
	sub	w10, w10, #1
	cmp	w12, #1024, lsl #12
	lsl	w12, w12, #1
	b.lo	LBB94_26
	b	LBB94_9
LBB94_27:
	mov	w10, #-30
LBB94_28:
	mov	w14, #1
	sub	w14, w14, w10
	lsr	w12, w12, w14
	fabs	s0, s1
	cmp	w10, w11
	b.eq	LBB94_11
LBB94_29:
	add	w10, w10, #1
	cmp	w10, w11
	b.ne	LBB94_32
	fmov	s1, w12
	fadd	s2, s1, s1
	fcmp	s2, s0
	b.gt	LBB94_12
	and	w10, w13, #0x1
	ccmp	w10, #0, #4, eq
	b.ne	LBB94_12
LBB94_32:
	and	w10, w13, #0x7fffffff
	eor	w9, w9, w8
	cmp	w9, #0
	cneg	w0, w10, mi
	fmov	s0, w12
	fneg	s1, s0
	cmp	w8, #0
	fcsel	s0, s1, s0, mi
	ret
	.cfi_endproc

	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0
lCPI95_0:
	.quad	0x3fdffffffd0c5e81
	.quad	0x3ec6cd878c3b46a7
lCPI95_1:
	.quad	0x3ef99342e0ee5069
	.quad	0x3ec6cd878c3b46a7
lCPI95_2:
	.quad	0xbf56c087e80f1e27
	.quad	0xbf2a00f9e2cae774
lCPI95_3:
	.quad	0x3ec6cd878c3b46a7
	.quad	0x3ef99342e0ee5069
lCPI95_4:
	.quad	0xbf2a00f9e2cae774
	.quad	0xbf56c087e80f1e27
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math7sincosf7sincosf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math7sincosf7sincosf:
	.cfi_startproc
	sub	sp, sp, #48
	.cfi_def_cfa_offset 48
	stp	x20, x19, [sp, #16]
	stp	x29, x30, [sp, #32]
	add	x29, sp, #32
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_remember_state
	fmov	w19, s0
	and	w8, w19, #0x7fffffff
	mov	w9, #4059
	movk	w9, #16201, lsl #16
	cmp	w8, w9
	b.hs	LBB95_4
	lsr	w9, w8, #23
	cmp	w9, #115
	b.hs	LBB95_8
	fmov.2s	v1, #1.00000000
	mov.s	v1[1], v0[0]
	cmp	w8, #256, lsl #12
	b.hs	LBB95_15
	mov	w8, #58720256
	fmov	s2, w8
	fmul	s0, s0, s2
	b	LBB95_16
LBB95_4:
	mov	w9, #21458
	movk	w9, #16507, lsl #16
	cmp	w8, w9
	b.hs	LBB95_10
	fcvt	d0, s0
	mov	w9, #52196
	movk	w9, #16406, lsl #16
	cmp	w8, w9
	b.hs	LBB95_17
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #16377, lsl #48
	tbnz	w19, #31, LBB95_14
	fmov	d1, x8
	fsub	d0, d1, d0
	fmul	d1, d0, d0
	mov	x8, #-41343
	movk	x8, #64780, lsl #16
	movk	x8, #49119, lsl #48
	fmov	d2, x8
	fmul	d2, d1, d2
	fmov	d3, #1.00000000
	fadd	d2, d2, d3
Lloh532:
	adrp	x8, lCPI95_3@PAGE
Lloh533:
	ldr	q3, [x8, lCPI95_3@PAGEOFF]
	fmul.2d	v3, v3, v1[0]
Lloh534:
	adrp	x8, lCPI95_4@PAGE
Lloh535:
	ldr	q4, [x8, lCPI95_4@PAGEOFF]
	fadd.2d	v3, v3, v4
	mov.d	v0[1], v1[0]
	mov	x8, #64434
	movk	x8, #35182, lsl #16
	movk	x8, #4368, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d4, x8
	fmul	d4, d1, d4
	mov	x8, #44151
	movk	x8, #21707, lsl #16
	movk	x8, #21845, lsl #32
	movk	x8, #49093, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	mov	x8, #14914
	movk	x8, #57605, lsl #16
	movk	x8, #21843, lsl #32
	movk	x8, #16293, lsl #48
	fmov	d5, x8
	dup.2d	v5, v5[0]
	mov.d	v5[0], v4[0]
	fmul.2d	v4, v0, v1[0]
	fmul.2d	v5, v4, v5
	mov.d	v0[1], v2[0]
	fadd.2d	v0, v0, v5
	mov.16b	v2, v4
	mov.d	v2[1], v1[0]
	fmul.2d	v1, v2, v4[1]
	fmul.2d	v1, v1, v3
	fadd.2d	v0, v1, v0
	b	LBB95_9
LBB95_8:
	fcvt	d0, s0
	fmul	d1, d0, d0
	fmul	d2, d1, d1
	fmul	d3, d1, d0
	mov	x8, #64434
	movk	x8, #35182, lsl #16
	movk	x8, #4368, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d4, x8
	fmul	d4, d1, d4
	mov	x8, #44151
	movk	x8, #21707, lsl #16
	movk	x8, #21845, lsl #32
	movk	x8, #49093, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	mov	x8, #20585
	movk	x8, #57582, lsl #16
	movk	x8, #37698, lsl #32
	movk	x8, #16121, lsl #48
	fmov	d5, x8
	fmul	d5, d1, d5
	mov	x8, #7719
	movk	x8, #59407, lsl #16
	movk	x8, #49287, lsl #32
	movk	x8, #48982, lsl #48
	fmov	d6, x8
	fadd	d5, d5, d6
Lloh536:
	adrp	x8, lCPI95_0@PAGE
Lloh537:
	ldr	q6, [x8, lCPI95_0@PAGEOFF]
	fmul.2d	v6, v6, v1[0]
	fmov.2d	v7, #1.00000000
	fsub.2d	v7, v7, v6
	mov	x8, #59252
	movk	x8, #58058, lsl #16
	movk	x8, #249, lsl #32
	movk	x8, #48938, lsl #48
	dup.2d	v16, x8
	fadd.2d	v6, v6, v16
	mov	x8, #14914
	movk	x8, #57605, lsl #16
	movk	x8, #21843, lsl #32
	movk	x8, #16293, lsl #48
	fmov	d16, x8
	dup.2d	v16, v16[0]
	mov.d	v16[1], v3[0]
	fmul.2d	v16, v16, v2[0]
	fadd.2d	v7, v7, v16
	fmul.2d	v6, v6, v16
	mov.d	v7[1], v6[1]
	fmul	d3, d3, d4
	fmul	d1, d1, d2
	fadd	d0, d3, d0
	fmul	d1, d1, d5
	mov.d	v1[1], v0[0]
	fadd.2d	v0, v7, v1
LBB95_9:
	fcvtn	v1.2s, v0.2d
	b	LBB95_27
LBB95_10:
	mov	w9, #12758
	movk	w9, #16610, lsl #16
	cmp	w8, w9
	b.hs	LBB95_19
	fcvt	d0, s0
	mov	w9, #60896
	movk	w9, #16559, lsl #16
	cmp	w8, w9
	b.hs	LBB95_21
	tbnz	w19, #31, LBB95_30
	mov	x8, #8658
	movk	x8, #32563, lsl #16
	movk	x8, #55676, lsl #32
	movk	x8, #49170, lsl #48
LBB95_14:
	fmov	d1, x8
	fadd	d0, d0, d1
	fmul	d1, d0, d0
	fmul	d2, d1, d1
	mov	x8, #20585
	movk	x8, #57582, lsl #16
	movk	x8, #37698, lsl #32
	movk	x8, #16121, lsl #48
	fmov	d3, x8
	fmul	d3, d1, d3
	mov	x8, #7719
	movk	x8, #59407, lsl #16
	movk	x8, #49287, lsl #32
	movk	x8, #48982, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	mov	x8, #-41343
	movk	x8, #64780, lsl #16
	movk	x8, #49119, lsl #48
	fmov	d4, x8
	fmul	d4, d1, d4
	fmov	d5, #1.00000000
	fadd	d4, d4, d5
	mov	x8, #14914
	movk	x8, #57605, lsl #16
	movk	x8, #21843, lsl #32
	movk	x8, #16293, lsl #48
	fmov	d5, x8
	fmul	d5, d2, d5
	fadd	d4, d4, d5
	fmul	d5, d1, d2
	fmul	d3, d5, d3
	fadd	d3, d4, d3
	fcvt	s3, d3
	fneg	s3, s3
	mov	x8, #18087
	movk	x8, #35899, lsl #16
	movk	x8, #52615, lsl #32
	movk	x8, #16070, lsl #48
	fmov	d4, x8
	fmul	d4, d1, d4
	mov	x8, #59252
	movk	x8, #58058, lsl #16
	movk	x8, #249, lsl #32
	movk	x8, #48938, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d5, d0, d1
	mov	x8, #64434
	movk	x8, #35182, lsl #16
	movk	x8, #4368, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d6, x8
	fmul	d1, d1, d6
	mov	x8, #44151
	movk	x8, #21707, lsl #16
	movk	x8, #21845, lsl #32
	movk	x8, #49093, lsl #48
	fmov	d6, x8
	fadd	d1, d1, d6
	fmul	d1, d5, d1
	fadd	d0, d0, d1
	fmul	d1, d5, d2
	fmul	d1, d1, d4
	fadd	d0, d1, d0
	fcvt	s1, d0
	mov.s	v1[1], v3[0]
	b	LBB95_27
LBB95_15:
	mov	w8, #2071986176
	fmov	s2, w8
	fadd	s0, s0, s2
LBB95_16:
	str	s0, [sp, #8]
	ldr	s0, [sp, #8]
	b	LBB95_27
LBB95_17:
	tbnz	w19, #31, LBB95_25
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #49161, lsl #48
	b	LBB95_26
LBB95_19:
	lsr	w9, w8, #23
	cmp	w9, #254
	b.ls	LBB95_23
	fsub	s0, s0, s0
	dup.2s	v1, v0[0]
	b	LBB95_27
LBB95_21:
	tbnz	w19, #31, LBB95_31
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #49177, lsl #48
	b	LBB95_32
LBB95_23:
	str	xzr, [sp, #8]
	mov	w10, #4058
	movk	w10, #19913, lsl #16
	cmp	w8, w10
	b.hi	LBB95_28
	fcvt	d0, s0
	mov	x8, #51331
	movk	x8, #28105, lsl #16
	movk	x8, #24368, lsl #32
	movk	x8, #16356, lsl #48
	fmov	d1, x8
	fmul	d1, d0, d1
	mov	x8, #4843621399236968448
	fmov	d2, x8
	fadd	d1, d1, d2
	mov	x8, #-4379750637617807360
	fmov	d2, x8
	fadd	d1, d1, d2
	fcvtzs	w0, d1
	mov	x8, #1342177280
	movk	x8, #8699, lsl #32
	movk	x8, #49145, lsl #48
	fmov	d2, x8
	fmul	d2, d1, d2
	fadd	d0, d0, d2
	mov	x8, #25187
	movk	x8, #24858, lsl #16
	movk	x8, #4276, lsl #32
	movk	x8, #48721, lsl #48
	fmov	d2, x8
	fmul	d1, d1, d2
	fadd	d0, d0, d1
	b	LBB95_34
LBB95_25:
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #16393, lsl #48
LBB95_26:
	fmov	d1, x8
	fadd	d0, d0, d1
	fmul	d1, d0, d0
	fmul	d2, d1, d1
	fmul	d3, d0, d1
	mov	x8, #64434
	movk	x8, #35182, lsl #16
	movk	x8, #4368, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d4, x8
	fmul	d4, d1, d4
	mov	x8, #44151
	movk	x8, #21707, lsl #16
	movk	x8, #21845, lsl #32
	movk	x8, #49093, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d4, d3, d4
	dup.2d	v5, v1[0]
Lloh538:
	adrp	x8, lCPI95_1@PAGE
Lloh539:
	ldr	q6, [x8, lCPI95_1@PAGEOFF]
	fmul.2d	v6, v6, v1[0]
Lloh540:
	adrp	x8, lCPI95_2@PAGE
Lloh541:
	ldr	q7, [x8, lCPI95_2@PAGEOFF]
	fadd.2d	v6, v6, v7
	mov	x8, #-41343
	movk	x8, #64780, lsl #16
	movk	x8, #49119, lsl #48
	fmov	d7, x8
	fmul	d1, d1, d7
	fmov	d7, #1.00000000
	fadd	d1, d1, d7
	mov	x8, #14914
	movk	x8, #57605, lsl #16
	movk	x8, #21843, lsl #32
	movk	x8, #16293, lsl #48
	fmov	d7, x8
	fmul	d7, d2, d7
	fadd	d0, d0, d4
	fadd	d1, d1, d7
	mov.d	v5[1], v3[0]
	fmul.2d	v2, v5, v2[0]
	fmul.2d	v2, v2, v6
	mov.d	v1[1], v0[0]
	fadd.2d	v0, v1, v2
	fcvtn	v0.2s, v0.2d
	fneg.2s	v1, v0
LBB95_27:
	mov	s0, v1[1]
	.cfi_def_cfa wsp, 48
	ldp	x29, x30, [sp, #32]
	ldp	x20, x19, [sp, #16]
	add	sp, sp, #48
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	ret
LBB95_28:
	.cfi_restore_state
	sub	w3, w9, #150
	sub	w8, w8, w3, lsl #23
	fmov	s0, w8
	fcvt	d0, s0
	str	d0, [sp]
	mov	x0, sp
	add	x2, sp, #8
	mov	w1, #1
	mov	x4, #0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math14rem_pio2_large14rem_pio2_large
	tbnz	w19, #31, LBB95_33
	ldr	d0, [sp, #8]
	b	LBB95_34
LBB95_30:
	mov	x8, #8658
	movk	x8, #32563, lsl #16
	movk	x8, #55676, lsl #32
	movk	x8, #16402, lsl #48
	fmov	d1, x8
	fadd	d0, d0, d1
	fmul	d1, d0, d0
	fmul	d2, d1, d1
	mov	x8, #20585
	movk	x8, #57582, lsl #16
	movk	x8, #37698, lsl #32
	movk	x8, #16121, lsl #48
	fmov	d3, x8
	fmul	d3, d1, d3
	mov	x8, #7719
	movk	x8, #59407, lsl #16
	movk	x8, #49287, lsl #32
	movk	x8, #48982, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	mov	x8, #-41343
	movk	x8, #64780, lsl #16
	movk	x8, #49119, lsl #48
	fmov	d4, x8
	fmul	d4, d1, d4
	fmov	d5, #1.00000000
	fadd	d4, d4, d5
	mov	x8, #14914
	movk	x8, #57605, lsl #16
	movk	x8, #21843, lsl #32
	movk	x8, #16293, lsl #48
	fmov	d5, x8
	fmul	d5, d2, d5
	fadd	d4, d4, d5
	fmul	d5, d1, d2
	fmul	d3, d5, d3
	fadd	d3, d4, d3
	fcvt	s3, d3
	mov	x8, #18087
	movk	x8, #35899, lsl #16
	movk	x8, #52615, lsl #32
	movk	x8, #16070, lsl #48
	fmov	d4, x8
	fmul	d4, d1, d4
	mov	x8, #59252
	movk	x8, #58058, lsl #16
	movk	x8, #249, lsl #32
	movk	x8, #48938, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d5, d0, d1
	mov	x8, #64434
	movk	x8, #35182, lsl #16
	movk	x8, #4368, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d6, x8
	fmul	d1, d1, d6
	mov	x8, #44151
	movk	x8, #21707, lsl #16
	movk	x8, #21845, lsl #32
	movk	x8, #49093, lsl #48
	fmov	d6, x8
	fadd	d1, d1, d6
	fmul	d1, d5, d1
	fadd	d0, d0, d1
	fmul	d1, d5, d2
	fmul	d1, d1, d4
	fadd	d0, d1, d0
	fcvt	s0, d0
	fneg	s1, s0
	mov.s	v1[1], v3[0]
	b	LBB95_27
LBB95_31:
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #16409, lsl #48
LBB95_32:
	fmov	d1, x8
	fadd	d0, d0, d1
	fmul	d1, d0, d0
	fmul	d2, d1, d1
	fmul	d3, d0, d1
	mov	x8, #64434
	movk	x8, #35182, lsl #16
	movk	x8, #4368, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d4, x8
	fmul	d4, d1, d4
	mov	x8, #44151
	movk	x8, #21707, lsl #16
	movk	x8, #21845, lsl #32
	movk	x8, #49093, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d4, d3, d4
	dup.2d	v5, v1[0]
Lloh542:
	adrp	x8, lCPI95_1@PAGE
Lloh543:
	ldr	q6, [x8, lCPI95_1@PAGEOFF]
	fmul.2d	v6, v6, v1[0]
Lloh544:
	adrp	x8, lCPI95_2@PAGE
Lloh545:
	ldr	q7, [x8, lCPI95_2@PAGEOFF]
	fadd.2d	v6, v6, v7
	mov	x8, #-41343
	movk	x8, #64780, lsl #16
	movk	x8, #49119, lsl #48
	fmov	d7, x8
	fmul	d1, d1, d7
	fmov	d7, #1.00000000
	fadd	d1, d1, d7
	mov	x8, #14914
	movk	x8, #57605, lsl #16
	movk	x8, #21843, lsl #32
	movk	x8, #16293, lsl #48
	fmov	d7, x8
	fmul	d7, d2, d7
	fadd	d0, d0, d4
	fadd	d1, d1, d7
	mov.d	v5[1], v3[0]
	fmul.2d	v2, v5, v2[0]
	fmul.2d	v2, v2, v6
	mov.d	v1[1], v0[0]
	fadd.2d	v0, v1, v2
	b	LBB95_9
LBB95_33:
	neg	w0, w0
	ldr	d0, [sp, #8]
	fneg	d0, d0
LBB95_34:
	fmul	d1, d0, d0
	fmul	d2, d1, d1
	mov	x8, #64434
	movk	x8, #35182, lsl #16
	movk	x8, #4368, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d3, x8
	fmul	d3, d1, d3
	mov	x8, #44151
	movk	x8, #21707, lsl #16
	movk	x8, #21845, lsl #32
	movk	x8, #49093, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	mov	x8, #20585
	movk	x8, #57582, lsl #16
	movk	x8, #37698, lsl #32
	movk	x8, #16121, lsl #48
	fmov	d4, x8
	fmul	d4, d1, d4
	mov	x8, #7719
	movk	x8, #59407, lsl #16
	movk	x8, #49287, lsl #32
	movk	x8, #48982, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
Lloh546:
	adrp	x8, lCPI95_0@PAGE
Lloh547:
	ldr	q5, [x8, lCPI95_0@PAGEOFF]
	fmul.2d	v5, v5, v1[0]
	fmov.2d	v6, #1.00000000
	mov	x8, #59252
	movk	x8, #58058, lsl #16
	movk	x8, #249, lsl #32
	movk	x8, #48938, lsl #48
	dup.2d	v7, x8
	fsub.2d	v6, v6, v5
	fadd.2d	v5, v5, v7
	mov	x8, #14914
	movk	x8, #57605, lsl #16
	movk	x8, #21843, lsl #32
	movk	x8, #16293, lsl #48
	fmov	d7, x8
	dup.2d	v7, v7[0]
	fmul	d16, d0, d1
	mov.d	v7[1], v16[0]
	fmul.2d	v7, v7, v2[0]
	fadd.2d	v6, v6, v7
	fmul.2d	v5, v5, v7
	mov.d	v6[1], v5[1]
	fmul	d3, d16, d3
	fmul	d1, d1, d2
	fadd	d0, d0, d3
	fmul	d1, d1, d4
	mov.d	v1[1], v0[0]
	fadd.2d	v0, v6, v1
	fcvtn	v1.2s, v0.2d
	and	w8, w0, #0x3
	cmp	w8, #1
	b.gt	LBB95_37
	cbz	w8, LBB95_27
	fneg.2s	v0, v1
	ext.8b	v1, v0, v1, #4
	b	LBB95_27
LBB95_37:
	cmp	w8, #2
	b.ne	LBB95_39
	fneg.2s	v1, v1
	b	LBB95_27
LBB95_39:
	fneg.2s	v0, v1
	ext.8b	v1, v1, v0, #4
	b	LBB95_27
	.loh AdrpLdr	Lloh534, Lloh535
	.loh AdrpAdrp	Lloh532, Lloh534
	.loh AdrpLdr	Lloh532, Lloh533
	.loh AdrpLdr	Lloh536, Lloh537
	.loh AdrpLdr	Lloh540, Lloh541
	.loh AdrpAdrp	Lloh538, Lloh540
	.loh AdrpLdr	Lloh538, Lloh539
	.loh AdrpLdr	Lloh544, Lloh545
	.loh AdrpAdrp	Lloh542, Lloh544
	.loh AdrpLdr	Lloh542, Lloh543
	.loh AdrpLdr	Lloh546, Lloh547
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math7tgammaf7tgammaf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math7tgammaf7tgammaf:
	.cfi_startproc
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	fcvt	d0, s0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math6tgamma6tgamma
	fcvt	s0, d0
	.cfi_def_cfa wsp, 16
	ldp	x29, x30, [sp], #16
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math8copysign8copysign
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math8copysign8copysign:
	.cfi_startproc
	movi.2d	v2, #0xffffffffffffffff
	fneg.2d	v2, v2
	bif.16b	v0, v1, v2
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math8copysign9copysignf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math8copysign9copysignf:
	.cfi_startproc
	mvni.4s	v2, #128, lsl #24
	bif.16b	v0, v1, v2
	ret
	.cfi_endproc

	.section	__TEXT,__literal16,16byte_literals
	.p2align	4, 0x0
lCPI99_0:
	.quad	0x3de5d93a5acfd57c
	.quad	0x3ec71de357b1fe7d
lCPI99_1:
	.quad	0xbe5ae5e68a2b9ceb
	.quad	0xbf2a01a019c161d5
lCPI99_2:
	.quad	0x3f00bfecdd17e945
	.quad	0x3edebaf7a5b38140
lCPI99_3:
	.quad	0x3f5e26b67368f239
	.quad	0x3f497ddaca41a95b
lCPI99_4:
	.quad	0x3f9b481c7e939961
	.quad	0x3f9317ea742ed475
lCPI99_5:
	.quad	0x3fc2bb9cbee5f2f7
	.quad	0x3fc601edccfbdf27
lCPI99_6:
	.quad	0x3fd4d98f4f139f59
	.quad	0x3fe71a1893d3dcdc
lCPI99_7:
	.quad	0x3fcb848b36e20878
	.quad	0x3ff645a762c4ab74
lCPI99_8:
	.quad	0xbfb3c467e37db0c8
	.quad	0x3ff0000000000000
lCPI99_9:
	.quad	0x3f8b678bbf2bab09
	.quad	0x3f6a5abb57d0cf61
lCPI99_10:
	.quad	0x3fcd4eaef6010924
	.quad	0x3fbaae55d6537c88
lCPI99_11:
	.quad	0x3fef497644ea8450
	.quad	0x3fe89dfbe45050af
lCPI99_12:
	.quad	0x3ff7475cd119bd6f
	.quad	0x40010725a42b18f5
lCPI99_13:
	.quad	0x3fe4401e8b005dff
	.quad	0x4003a5d7c2bd619c
lCPI99_14:
	.quad	0x3f347f24ecc38c38
	.quad	0x3f35fd3ee8c2d3f4
lCPI99_15:
	.quad	0xbf6e2effb3e914d7
	.quad	0x3f6282d32e15c915
lCPI99_16:
	.quad	0x3f9266e7970af9ec
	.quad	0xbf851f9fba91ec6a
lCPI99_17:
	.quad	0xbfc2e4278dc6c509
	.quad	0x3fb08b4294d5419b
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math8lgamma_r8lgamma_r
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math8lgamma_r8lgamma_r:
	.cfi_startproc
	sub	sp, sp, #80
	.cfi_def_cfa_offset 80
	stp	d9, d8, [sp, #16]
	stp	x22, x21, [sp, #32]
	stp	x20, x19, [sp, #48]
	stp	x29, x30, [sp, #64]
	add	x29, sp, #64
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset w21, -40
	.cfi_offset w22, -48
	.cfi_offset b8, -56
	.cfi_offset b9, -64
	fmov	x19, d0
	lsr	x8, x19, #32
	ubfx	w8, w8, #20, #11
	cmp	w8, #2046
	b.ls	LBB99_2
	fmul	d0, d0, d0
	mov	w0, #1
	b	LBB99_47
LBB99_2:
	ubfx	x20, x19, #32, #31
	lsr	w21, w20, #20
	cmp	w21, #953
	b.hs	LBB99_4
	asr	x8, x19, #63
	orr	w19, w8, #0x1
	fabs	d0, d0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3log3log
	mov	x0, x19
	fneg	d0, d0
	b	LBB99_47
LBB99_4:
	tbnz	x19, #63, LBB99_12
	mov	w0, #1
	movi.2d	v8, #0000000000000000
	mov	w8, #1072693248
	cmp	w20, w8
	b.eq	LBB99_7
LBB99_6:
	mov	w8, #1073741824
	cmp	w20, w8
	b.ne	LBB99_8
LBB99_7:
	cbz	w19, LBB99_19
LBB99_8:
	lsr	w8, w20, #30
	cbnz	w8, LBB99_16
	mov	w8, #52429
	movk	w8, #16364, lsl #16
	cmp	w20, w8
	b.hs	LBB99_20
	mov	x21, x0
	str	q0, [sp]
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3log3log
	fneg	d0, d0
	fmov	d1, #1.00000000
	mov	w8, #26947
	movk	w8, #16359, lsl #16
	cmp	w20, w8
	b.ls	LBB99_36
	mov	x0, x21
	ldr	q2, [sp]
	b	LBB99_22
LBB99_12:
	mov.16b	v1, v0
	fmov	d0, #-0.50000000
	mov.16b	v6, v1
	fmul	d1, d1, d0
	fmov	x8, d1
	ubfx	x9, x8, #52, #11
	mov.16b	v2, v1
	cmp	w9, #1074
	b.hi	LBB99_26
	cmp	w9, #1022
	b.ls	LBB99_25
	sub	w9, w9, #1023
	mov	x11, #4503599627370495
	lsr	x10, x11, x9
	mov.16b	v2, v1
	tst	x10, x8
	b.eq	LBB99_26
	and	x10, x10, x8, asr #63
	mov	x11, #-4503599627370496
	asr	x9, x11, x9
	add	x8, x10, x8
	and	x8, x8, x9
	fmov	d2, x8
	b	LBB99_26
LBB99_16:
	lsr	w8, w20, #21
	cmp	w8, #513
	b.hs	LBB99_23
	fcvtzs	w8, d0
	mov.16b	v1, v0
	scvtf	d0, w8
	fsub	d0, d1, d0
Lloh548:
	adrp	x9, lCPI99_2@PAGE
Lloh549:
	ldr	q1, [x9, lCPI99_2@PAGEOFF]
	fmul.2d	v1, v1, v0[0]
Lloh550:
	adrp	x9, lCPI99_3@PAGE
Lloh551:
	ldr	q2, [x9, lCPI99_3@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul.2d	v1, v1, v0[0]
Lloh552:
	adrp	x9, lCPI99_4@PAGE
Lloh553:
	ldr	q2, [x9, lCPI99_4@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul.2d	v1, v1, v0[0]
Lloh554:
	adrp	x9, lCPI99_5@PAGE
Lloh555:
	ldr	q2, [x9, lCPI99_5@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul.2d	v1, v1, v0[0]
Lloh556:
	adrp	x9, lCPI99_6@PAGE
Lloh557:
	ldr	q2, [x9, lCPI99_6@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul.2d	v1, v1, v0[0]
Lloh558:
	adrp	x9, lCPI99_7@PAGE
Lloh559:
	ldr	q2, [x9, lCPI99_7@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul.2d	v1, v1, v0[0]
Lloh560:
	adrp	x9, lCPI99_8@PAGE
Lloh561:
	ldr	q2, [x9, lCPI99_8@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul	d2, d0, d1
	fmov	d3, #0.50000000
	fmul	d3, d0, d3
	mov	d1, v1[1]
	fdiv	d1, d2, d1
	fadd	d9, d3, d1
	cmp	w8, #2
	b.le	LBB99_46
	fmov	d1, #6.00000000
	fadd	d1, d0, d1
	cmp	w8, #6
	fmov	d2, #1.00000000
	fcsel	d1, d1, d2, hi
	fmov	d3, #5.00000000
	fadd	d3, d0, d3
	fmul	d1, d3, d1
	cmp	w8, #5
	fcsel	d1, d1, d2, hi
	fmov	d3, #4.00000000
	fadd	d3, d0, d3
	fmul	d1, d3, d1
	cmp	w8, #4
	fcsel	d1, d1, d2, hi
	fmov	d3, #3.00000000
	fadd	d3, d0, d3
	fmul	d1, d3, d1
	cmp	w8, #3
	fcsel	d1, d2, d1, eq
	fmov	d2, #2.00000000
	fadd	d0, d0, d2
	fmul	d0, d0, d1
	mov	x20, x0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3log3log
	mov	x0, x20
	fadd	d9, d9, d0
	b	LBB99_46
LBB99_19:
	movi.2d	v9, #0000000000000000
	b	LBB99_46
LBB99_20:
	mov	w8, #46274
	movk	w8, #16379, lsl #16
	mov.16b	v2, v0
	cmp	w20, w8
	b.ls	LBB99_38
	fmov	d1, #2.00000000
	movi.2d	v0, #0000000000000000
LBB99_22:
	fsub	d1, d1, d2
	fmul	d2, d1, d1
	mov	x8, #64082
	movk	x8, #17036, lsl #16
	movk	x8, #28788, lsl #32
	movk	x8, #16122, lsl #48
	fmov	d3, x8
	fmul	d3, d2, d3
	mov	x8, #58701
	movk	x8, #60688, lsl #16
	movk	x8, #62188, lsl #32
	movk	x8, #16172, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d3, d2, d3
	mov	x8, #16221
	movk	x8, #4463, lsl #16
	movk	x8, #35476, lsl #32
	movk	x8, #16211, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d3, d2, d3
	mov	x8, #61416
	movk	x8, #46735, lsl #16
	movk	x8, #16463, lsl #32
	movk	x8, #16254, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d3, d2, d3
	mov	x8, #25255
	movk	x8, #6741, lsl #16
	movk	x8, #15872, lsl #32
	movk	x8, #16305, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d3, d2, d3
	mov	x8, #45256
	movk	x8, #58237, lsl #16
	movk	x8, #50279, lsl #32
	movk	x8, #16307, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	mov	x8, #22583
	movk	x8, #37028, lsl #16
	movk	x8, #34190, lsl #32
	movk	x8, #16135, lsl #48
	fmov	d4, x8
	fmul	d4, d2, d4
	mov	x8, #64263
	movk	x8, #39037, lsl #16
	movk	x8, #20616, lsl #32
	movk	x8, #16156, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d4, d2, d4
	mov	x8, #39936
	movk	x8, #35257, lsl #16
	movk	x8, #46790, lsl #32
	movk	x8, #16192, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d4, d2, d4
	mov	x8, #37483
	movk	x8, #52407, lsl #16
	movk	x8, #44504, lsl #32
	movk	x8, #16231, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d4, d2, d4
	mov	x8, #21627
	movk	x8, #44178, lsl #16
	movk	x8, #4898, lsl #32
	movk	x8, #16277, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d4, d2, d4
	mov	x8, #4013
	movk	x8, #50342, lsl #16
	movk	x8, #41804, lsl #32
	movk	x8, #16340, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d2, d2, d4
	fmul	d3, d1, d3
	fadd	d2, d3, d2
	fmov	d3, #-0.50000000
	fmul	d1, d1, d3
	fadd	d1, d2, d1
	b	LBB99_45
LBB99_23:
	mov	x20, x0
	str	q0, [sp]
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3log3log
	cmp	w21, #1081
	b.hs	LBB99_41
	fmov	d1, #1.00000000
	ldr	q5, [sp]
	fdiv	d1, d1, d5
	fmul	d2, d1, d1
	mov	x8, #17380
	movk	x8, #2974, lsl #16
	movk	x8, #47261, lsl #32
	movk	x8, #48986, lsl #48
	fmov	d3, x8
	fmul	d3, d2, d3
	mov	x8, #54737
	movk	x8, #19674, lsl #16
	movk	x8, #26554, lsl #32
	movk	x8, #16203, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d3, d2, d3
	mov	x8, #59201
	movk	x8, #35855, lsl #16
	movk	x8, #32971, lsl #32
	movk	x8, #48963, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d3, d2, d3
	mov	x8, #14518
	movk	x8, #39119, lsl #16
	movk	x8, #415, lsl #32
	movk	x8, #16202, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d3, d2, d3
	mov	x8, #11868
	movk	x8, #5808, lsl #16
	movk	x8, #49516, lsl #32
	movk	x8, #48998, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d2, d2, d3
	mov	x8, #6148914691236517205
	movk	x8, #21819
	movk	x8, #16309, lsl #48
	fmov	d3, x8
	fadd	d2, d2, d3
	fmul	d1, d1, d2
	mov	x8, #32105
	movk	x8, #37065, lsl #16
	movk	x8, #53219, lsl #32
	movk	x8, #16346, lsl #48
	fmov	d2, x8
	fadd	d1, d1, d2
	fmov	d2, #-0.50000000
	fadd	d2, d5, d2
	fmov	d3, #-1.00000000
	fadd	d0, d0, d3
	fmul	d0, d2, d0
	fadd	d9, d1, d0
	mov	x0, x20
	b	LBB99_46
LBB99_25:
	cmn	x8, #1
	fmov	d2, #-1.00000000
	movi.2d	v3, #0000000000000000
	fcsel	d2, d3, d2, gt
LBB99_26:
	fsub	d1, d1, d2
	fadd	d1, d1, d1
	fcvtzs	w8, d1, #2
	add	w8, w8, #1
	add	w8, w8, w8, lsr #31
	asr	w8, w8, #1
	scvtf	d2, w8
	fmul	d0, d2, d0
	fadd	d0, d1, d0
	mov	x9, #11544
	movk	x9, #21572, lsl #16
	movk	x9, #8699, lsl #32
	movk	x9, #16393, lsl #48
	fmov	d1, x9
	fmul	d0, d0, d1
	cmp	w8, #1
	b.eq	LBB99_31
	cmp	w8, #2
	b.eq	LBB99_30
	cmp	w8, #3
	b.ne	LBB99_32
	fmul	d1, d0, d0
	fmul	d2, d1, d1
	mov	x8, #5520
	movk	x8, #6603, lsl #16
	movk	x8, #416, lsl #32
	movk	x8, #16122, lsl #48
	fmov	d3, x8
	fmul	d3, d1, d3
	mov	x8, #20855
	movk	x8, #5825, lsl #16
	movk	x8, #49516, lsl #32
	movk	x8, #48982, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d3, d1, d3
	mov	x8, #6148914691236517205
	movk	x8, #21836
	movk	x8, #16293, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d3, d1, d3
	fmul	d2, d2, d2
	mov	x8, #14548
	movk	x8, #48776, lsl #16
	movk	x8, #64233, lsl #32
	movk	x8, #48552, lsl #48
	fmov	d4, x8
	fmul	d4, d1, d4
	mov	x8, #45508
	movk	x8, #48564, lsl #16
	movk	x8, #61086, lsl #32
	movk	x8, #15905, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d4, d1, d4
	mov	x8, #21165
	movk	x8, #32924, lsl #16
	movk	x8, #32335, lsl #32
	movk	x8, #48786, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d2, d2, d4
	fadd	d2, d3, d2
	fmov	d3, #0.50000000
	fmul	d3, d1, d3
	fmov	d4, #1.00000000
	fsub	d5, d4, d3
	fsub	d4, d4, d5
	fsub	d3, d4, d3
	fmul	d1, d1, d2
	mov	x8, #-9223372036854775808
	fmov	d2, x8
	fmul	d0, d0, d2
	fadd	d0, d1, d0
	fadd	d0, d3, d0
	fadd	d0, d5, d0
	fneg	d0, d0
	b	LBB99_33
LBB99_30:
	fmul	d1, d0, d0
	fmul	d2, d1, d1
	fmul	d2, d1, d2
	dup.2d	v3, v1[0]
Lloh562:
	adrp	x8, lCPI99_0@PAGE
Lloh563:
	ldr	q4, [x8, lCPI99_0@PAGEOFF]
	fmul.2d	v4, v4, v1[0]
Lloh564:
	adrp	x8, lCPI99_1@PAGE
Lloh565:
	ldr	q5, [x8, lCPI99_1@PAGEOFF]
	fadd.2d	v4, v4, v5
	mov.d	v3[0], v2[0]
	fmul.2d	v2, v3, v4
	mov	d3, v2[1]
	mov	x8, #63654
	movk	x8, #4368, lsl #16
	movk	x8, #4369, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fadd	d2, d2, d3
	fnmul	d3, d0, d1
	fmul	d1, d1, d2
	mov	x8, #6148914691236517205
	movk	x8, #21833
	movk	x8, #49093, lsl #48
	fmov	d2, x8
	fadd	d1, d1, d2
	fmul	d1, d3, d1
	fsub	d0, d1, d0
	b	LBB99_33
LBB99_31:
	fmul	d1, d0, d0
	fmul	d2, d1, d1
	mov	x8, #5520
	movk	x8, #6603, lsl #16
	movk	x8, #416, lsl #32
	movk	x8, #16122, lsl #48
	fmov	d3, x8
	fmul	d3, d1, d3
	mov	x8, #20855
	movk	x8, #5825, lsl #16
	movk	x8, #49516, lsl #32
	movk	x8, #48982, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d3, d1, d3
	mov	x8, #6148914691236517205
	movk	x8, #21836
	movk	x8, #16293, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fmul	d3, d1, d3
	fmul	d2, d2, d2
	mov	x8, #14548
	movk	x8, #48776, lsl #16
	movk	x8, #64233, lsl #32
	movk	x8, #48552, lsl #48
	fmov	d4, x8
	fmul	d4, d1, d4
	mov	x8, #45508
	movk	x8, #48564, lsl #16
	movk	x8, #61086, lsl #32
	movk	x8, #15905, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d4, d1, d4
	mov	x8, #21165
	movk	x8, #32924, lsl #16
	movk	x8, #32335, lsl #32
	movk	x8, #48786, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d2, d2, d4
	fadd	d2, d3, d2
	fmov	d3, #0.50000000
	fmul	d3, d1, d3
	fmov	d4, #1.00000000
	fsub	d5, d4, d3
	fsub	d4, d4, d5
	fsub	d3, d4, d3
	fmul	d1, d1, d2
	mov	x8, #-9223372036854775808
	fmov	d2, x8
	fmul	d0, d0, d2
	fadd	d0, d1, d0
	fadd	d0, d3, d0
	fadd	d0, d5, d0
	b	LBB99_33
LBB99_32:
	fmul	d1, d0, d0
	fmul	d2, d1, d1
	fmul	d2, d1, d2
	dup.2d	v3, v1[0]
Lloh566:
	adrp	x8, lCPI99_0@PAGE
Lloh567:
	ldr	q4, [x8, lCPI99_0@PAGEOFF]
	fmul.2d	v4, v4, v1[0]
Lloh568:
	adrp	x8, lCPI99_1@PAGE
Lloh569:
	ldr	q5, [x8, lCPI99_1@PAGEOFF]
	fadd.2d	v4, v4, v5
	mov.d	v3[0], v2[0]
	fmul.2d	v2, v3, v4
	mov	d3, v2[1]
	mov	x8, #63654
	movk	x8, #4368, lsl #16
	movk	x8, #4369, lsl #32
	movk	x8, #16257, lsl #48
	fmov	d4, x8
	fadd	d3, d3, d4
	fadd	d2, d2, d3
	fmul	d3, d0, d1
	fmul	d1, d1, d2
	mov	x8, #6148914691236517205
	movk	x8, #21833
	movk	x8, #49093, lsl #48
	fmov	d2, x8
	fadd	d1, d1, d2
	fmul	d1, d3, d1
	fadd	d0, d0, d1
LBB99_33:
	fcmp	d0, #0.0
	b.ne	LBB99_35
	fsub	d0, d6, d6
	fmov	d1, #1.00000000
	mov	w0, #1
	fdiv	d0, d1, d0
	b	LBB99_47
LBB99_35:
	fneg	d6, d6
	str	q6, [sp]
	fneg	d1, d0
	fcsel	d0, d0, d1, gt
	mov	w8, #-1
	csinc	w22, w8, wzr, gt
	fmul	d0, d0, d6
	mov	x8, #11544
	movk	x8, #21572, lsl #16
	movk	x8, #8699, lsl #32
	movk	x8, #16393, lsl #48
	fmov	d1, x8
	fdiv	d0, d1, d0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math3log3log
	mov	x0, x22
	mov.16b	v8, v0
	ldr	q0, [sp]
	mov	w8, #1072693248
	cmp	w20, w8
	b.ne	LBB99_6
	b	LBB99_7
LBB99_36:
	mov	w8, #42592
	movk	w8, #16333, lsl #16
	cmp	w20, w8
	mov	x0, x21
	b.ls	LBB99_42
	mov	x8, #63740
	movk	x8, #36186, lsl #16
	movk	x8, #35681, lsl #32
	movk	x8, #49117, lsl #48
	fmov	d1, x8
	ldr	q2, [sp]
	b	LBB99_40
LBB99_38:
	mov	w8, #46275
	movk	w8, #16371, lsl #16
	cmp	w20, w8
	b.ls	LBB99_43
	movi.2d	v0, #0000000000000000
	mov	x8, #48703
	movk	x8, #25430, lsl #16
	movk	x8, #25304, lsl #32
	movk	x8, #49143, lsl #48
	fmov	d1, x8
LBB99_40:
	fadd	d1, d2, d1
	fmul	d2, d1, d1
	fmul	d3, d1, d2
	mov	x8, #48119
	movk	x8, #27662, lsl #16
	movk	x8, #44909, lsl #32
	movk	x8, #16180, lsl #48
	fmov	d4, x8
	fmul	d4, d3, d4
	mov	x8, #6897
	movk	x8, #48941, lsl #16
	movk	x8, #65166, lsl #32
	movk	x8, #48982, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d4, d3, d4
	mov	x8, #58180
	movk	x8, #58224, lsl #16
	movk	x8, #64736, lsl #32
	movk	x8, #16248, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d4, d3, d4
	mov	x8, #46867
	movk	x8, #57141, lsl #16
	movk	x8, #51624, lsl #32
	movk	x8, #49056, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
	fmul	d4, d3, d4
	mov	x8, #14498
	movk	x8, #51438, lsl #16
	movk	x8, #63275, lsl #32
	movk	x8, #16350, lsl #48
	fmov	d5, x8
	fadd	d4, d4, d5
Lloh570:
	adrp	x8, lCPI99_14@PAGE
Lloh571:
	ldr	q5, [x8, lCPI99_14@PAGEOFF]
	fmul.2d	v5, v5, v3[0]
	mov	x8, #43241
	movk	x8, #61281, lsl #16
	movk	x8, #57100, lsl #32
	movk	x8, #16204, lsl #48
	fmov	d6, x8
	fsub.2d	v6, v6, v5
	mov	x8, #57580
	movk	x8, #40051, lsl #16
	movk	x8, #42512, lsl #32
	movk	x8, #48961, lsl #48
	dup.2d	v7, x8
	fadd.2d	v5, v5, v7
	mov.d	v6[1], v5[1]
	fmul.2d	v5, v6, v3[0]
Lloh572:
	adrp	x8, lCPI99_15@PAGE
Lloh573:
	ldr	q6, [x8, lCPI99_15@PAGEOFF]
	fadd.2d	v5, v5, v6
	fmul.2d	v5, v5, v3[0]
Lloh574:
	adrp	x8, lCPI99_16@PAGE
Lloh575:
	ldr	q6, [x8, lCPI99_16@PAGEOFF]
	fadd.2d	v5, v5, v6
	fmul.2d	v5, v5, v3[0]
Lloh576:
	adrp	x8, lCPI99_17@PAGE
Lloh577:
	ldr	q6, [x8, lCPI99_17@PAGEOFF]
	fadd.2d	v5, v5, v6
	fmul	d2, d2, d4
	fmul.d	d1, d1, v5[1]
	fadd	d1, d5, d1
	fmul	d1, d3, d1
	mov	x8, #38687
	movk	x8, #42122, lsl #16
	movk	x8, #51146, lsl #32
	movk	x8, #48208, lsl #48
	fmov	d3, x8
	fsub	d1, d3, d1
	fsub	d1, d2, d1
	mov	x8, #35394
	movk	x8, #48323, lsl #16
	movk	x8, #6585, lsl #32
	movk	x8, #49087, lsl #48
	fmov	d2, x8
	fadd	d1, d1, d2
	b	LBB99_45
LBB99_41:
	fmov	d1, #-1.00000000
	fadd	d0, d0, d1
	ldr	q1, [sp]
	fmul	d9, d1, d0
	mov	x0, x20
	b	LBB99_46
LBB99_42:
	ldr	q2, [sp]
	b	LBB99_44
LBB99_43:
	fmov	d0, #-1.00000000
	fadd	d2, d2, d0
	movi.2d	v0, #0000000000000000
LBB99_44:
Lloh578:
	adrp	x8, lCPI99_9@PAGE
Lloh579:
	ldr	q1, [x8, lCPI99_9@PAGEOFF]
	mov.16b	v4, v2
	fmul.2d	v1, v1, v2[0]
Lloh580:
	adrp	x8, lCPI99_10@PAGE
Lloh581:
	ldr	q2, [x8, lCPI99_10@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul.2d	v1, v1, v4[0]
Lloh582:
	adrp	x8, lCPI99_11@PAGE
Lloh583:
	ldr	q2, [x8, lCPI99_11@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul.2d	v1, v1, v4[0]
Lloh584:
	adrp	x8, lCPI99_12@PAGE
Lloh585:
	ldr	q2, [x8, lCPI99_12@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul.2d	v1, v1, v4[0]
Lloh586:
	adrp	x8, lCPI99_13@PAGE
Lloh587:
	ldr	q2, [x8, lCPI99_13@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul.2d	v1, v1, v4[0]
Lloh588:
	adrp	x8, lCPI99_8@PAGE
Lloh589:
	ldr	q2, [x8, lCPI99_8@PAGEOFF]
	fadd.2d	v1, v1, v2
	fmul	d2, d4, d1
	fmov	d3, #-0.50000000
	fmul	d3, d4, d3
	mov	d1, v1[1]
	fdiv	d1, d2, d1
	fadd	d1, d1, d3
LBB99_45:
	fadd	d9, d0, d1
LBB99_46:
	fsub	d0, d8, d9
	cmn	x19, #1
	fcsel	d0, d9, d0, gt
LBB99_47:
	.cfi_def_cfa wsp, 80
	ldp	x29, x30, [sp, #64]
	ldp	x20, x19, [sp, #48]
	ldp	x22, x21, [sp, #32]
	ldp	d9, d8, [sp, #16]
	add	sp, sp, #80
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	.cfi_restore b8
	.cfi_restore b9
	ret
	.loh AdrpLdr	Lloh560, Lloh561
	.loh AdrpAdrp	Lloh558, Lloh560
	.loh AdrpLdr	Lloh558, Lloh559
	.loh AdrpAdrp	Lloh556, Lloh558
	.loh AdrpLdr	Lloh556, Lloh557
	.loh AdrpAdrp	Lloh554, Lloh556
	.loh AdrpLdr	Lloh554, Lloh555
	.loh AdrpAdrp	Lloh552, Lloh554
	.loh AdrpLdr	Lloh552, Lloh553
	.loh AdrpAdrp	Lloh550, Lloh552
	.loh AdrpLdr	Lloh550, Lloh551
	.loh AdrpAdrp	Lloh548, Lloh550
	.loh AdrpLdr	Lloh548, Lloh549
	.loh AdrpLdr	Lloh564, Lloh565
	.loh AdrpAdrp	Lloh562, Lloh564
	.loh AdrpLdr	Lloh562, Lloh563
	.loh AdrpLdr	Lloh568, Lloh569
	.loh AdrpAdrp	Lloh566, Lloh568
	.loh AdrpLdr	Lloh566, Lloh567
	.loh AdrpLdr	Lloh576, Lloh577
	.loh AdrpAdrp	Lloh574, Lloh576
	.loh AdrpLdr	Lloh574, Lloh575
	.loh AdrpAdrp	Lloh572, Lloh574
	.loh AdrpLdr	Lloh572, Lloh573
	.loh AdrpLdr	Lloh570, Lloh571
	.loh AdrpLdr	Lloh588, Lloh589
	.loh AdrpAdrp	Lloh586, Lloh588
	.loh AdrpLdr	Lloh586, Lloh587
	.loh AdrpAdrp	Lloh584, Lloh586
	.loh AdrpLdr	Lloh584, Lloh585
	.loh AdrpAdrp	Lloh582, Lloh584
	.loh AdrpLdr	Lloh582, Lloh583
	.loh AdrpAdrp	Lloh580, Lloh582
	.loh AdrpLdr	Lloh580, Lloh581
	.loh AdrpAdrp	Lloh578, Lloh580
	.loh AdrpLdr	Lloh578, Lloh579
	.cfi_endproc

	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math8rem_pio28rem_pio2:
	.cfi_startproc
	sub	sp, sp, #80
	.cfi_def_cfa_offset 80
	stp	x20, x19, [sp, #48]
	stp	x29, x30, [sp, #64]
	add	x29, sp, #64
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_remember_state
	fmov	x19, d0
	ubfx	x8, x19, #32, #31
	mov	w9, #27259
	movk	w9, #16399, lsl #16
	cmp	w8, w9
	b.hs	LBB100_5
	lsr	x9, x19, #32
	mov	w10, #45056
	movk	w10, #37407, lsl #16
	cmp	w10, w9, lsl #12
	b.eq	LBB100_11
	mov	w9, #55677
	movk	w9, #16386, lsl #16
	cmp	w8, w9
	b.hs	LBB100_14
	tbnz	x19, #63, LBB100_27
	mov	x8, #1413480448
	movk	x8, #8699, lsl #32
	movk	x8, #49145, lsl #48
	fmov	d1, x8
	fadd	d0, d0, d1
	mov	x8, #25393
	movk	x8, #6754, lsl #16
	movk	x8, #46177, lsl #32
	movk	x8, #48592, lsl #48
	fmov	d1, x8
	fadd	d2, d0, d1
	fsub	d0, d0, d2
	mov	w8, #1
	b	LBB100_31
LBB100_5:
	mov	w9, #17980
	movk	w9, #16412, lsl #16
	cmp	w8, w9
	b.hs	LBB100_10
	mov	w9, #64957
	movk	w9, #16405, lsl #16
	cmp	w8, w9
	b.hs	LBB100_16
	mov	w9, #55676
	movk	w9, #16402, lsl #16
	cmp	w8, w9
	b.eq	LBB100_17
	tbnz	x19, #63, LBB100_29
	mov	x8, #2133852160
	movk	x8, #55676, lsl #32
	movk	x8, #49170, lsl #48
	fmov	d1, x8
	fadd	d0, d0, d1
	mov	x8, #38090
	movk	x8, #42899, lsl #16
	movk	x8, #3729, lsl #32
	movk	x8, #48617, lsl #48
	fmov	d1, x8
	fadd	d2, d0, d1
	fsub	d0, d0, d2
	mov	w8, #3
	b	LBB100_31
LBB100_10:
	mov	w9, #8699
	movk	w9, #16697, lsl #16
	cmp	w8, w9
	b.hs	LBB100_21
LBB100_11:
	mov	x9, #51331
	movk	x9, #28105, lsl #16
	movk	x9, #24368, lsl #32
	movk	x9, #16356, lsl #48
	fmov	d1, x9
	fmul	d1, d0, d1
	mov	x9, #4843621399236968448
	fmov	d2, x9
	fadd	d1, d1, d2
	mov	x9, #-4379750637617807360
	fmov	d2, x9
	fadd	d1, d1, d2
	mov	x9, #1413480448
	movk	x9, #8699, lsl #32
	movk	x9, #49145, lsl #48
	fmov	d2, x9
	fmul	d2, d1, d2
	fadd	d0, d0, d2
	mov	x9, #25393
	movk	x9, #6754, lsl #16
	movk	x9, #46177, lsl #32
	movk	x9, #15824, lsl #48
	fmov	d2, x9
	fmul	d2, d1, d2
	fsub	d3, d0, d2
	fmov	x9, d3
	ubfx	x9, x9, #52, #11
	lsr	w8, w8, #20
	sub	w9, w8, w9
	cmp	w9, #17
	b.lt	LBB100_20
	mov	x9, #442499072
	movk	x9, #46177, lsl #32
	movk	x9, #15824, lsl #48
	fmov	d2, x9
	fmul	d2, d1, d2
	fsub	d4, d0, d2
	mov	x9, #28787
	movk	x9, #11779, lsl #16
	movk	x9, #6538, lsl #32
	movk	x9, #15267, lsl #48
	fmov	d3, x9
	fmul	d3, d1, d3
	fsub	d0, d0, d4
	fsub	d0, d0, d2
	fsub	d2, d3, d0
	fsub	d3, d4, d2
	fmov	x9, d3
	ubfx	x9, x9, #52, #11
	sub	w8, w8, w9
	cmp	w8, #50
	b.lt	LBB100_19
LBB100_13:
	mov	x8, #771751936
	movk	x8, #6538, lsl #32
	movk	x8, #15267, lsl #48
	fmov	d0, x8
	fmul	d2, d1, d0
	fsub	d0, d4, d2
	mov	x8, #18881
	movk	x8, #9504, lsl #16
	movk	x8, #33690, lsl #32
	movk	x8, #14715, lsl #48
	fmov	d3, x8
	fmul	d3, d1, d3
	fsub	d4, d4, d0
	fsub	d2, d4, d2
	fsub	d2, d3, d2
	fsub	d3, d0, d2
	b	LBB100_20
LBB100_14:
	tbnz	x19, #63, LBB100_28
	mov	x8, #1413480448
	movk	x8, #8699, lsl #32
	movk	x8, #49161, lsl #48
	fmov	d1, x8
	fadd	d0, d0, d1
	mov	x8, #25393
	movk	x8, #6754, lsl #16
	movk	x8, #46177, lsl #32
	movk	x8, #48608, lsl #48
	fmov	d1, x8
	fadd	d2, d0, d1
	fsub	d0, d0, d2
	mov	w8, #2
	b	LBB100_31
LBB100_16:
	mov	w9, #8699
	movk	w9, #16409, lsl #16
	cmp	w8, w9
	b.ne	LBB100_23
LBB100_17:
	mov	x8, #51331
	movk	x8, #28105, lsl #16
	movk	x8, #24368, lsl #32
	movk	x8, #16356, lsl #48
	fmov	d1, x8
	fmul	d1, d0, d1
	mov	x8, #4843621399236968448
	fmov	d2, x8
	fadd	d1, d1, d2
	mov	x8, #-4379750637617807360
	fmov	d2, x8
	fadd	d1, d1, d2
	mov	x8, #1413480448
	movk	x8, #8699, lsl #32
	movk	x8, #49145, lsl #48
	fmov	d2, x8
	fmul	d2, d1, d2
	fadd	d0, d0, d2
	mov	x8, #25393
	movk	x8, #6754, lsl #16
	movk	x8, #46177, lsl #32
	movk	x8, #15824, lsl #48
	fmov	d2, x8
	fmul	d2, d1, d2
	fsub	d3, d0, d2
	fmov	x8, d3
	ubfx	x8, x8, #52, #11
	cmp	x8, #1008
	b.hi	LBB100_20
	mov	x8, #442499072
	movk	x8, #46177, lsl #32
	movk	x8, #15824, lsl #48
	fmov	d2, x8
	fmul	d2, d1, d2
	fsub	d4, d0, d2
	mov	x8, #28787
	movk	x8, #11779, lsl #16
	movk	x8, #6538, lsl #32
	movk	x8, #15267, lsl #48
	fmov	d3, x8
	fmul	d3, d1, d3
	fsub	d0, d0, d4
	fsub	d0, d0, d2
	fsub	d2, d3, d0
	fsub	d3, d4, d2
	fmov	x8, d3
	ubfx	x8, x8, #56, #7
	cmp	x8, #60
	b.ls	LBB100_13
LBB100_19:
	mov.16b	v0, v4
LBB100_20:
	fcvtzs	w8, d1
	fsub	d0, d0, d3
	fsub	d0, d0, d2
	str	w8, [x0, #8]
	str	d3, [x0]
	b	LBB100_32
LBB100_21:
	lsr	w8, w8, #20
	cmp	w8, #2046
	b.ls	LBB100_25
	fsub	d0, d0, d0
	str	wzr, [x0, #8]
	str	d0, [x0]
	b	LBB100_32
LBB100_23:
	tbnz	x19, #63, LBB100_30
	mov	x8, #1413480448
	movk	x8, #8699, lsl #32
	movk	x8, #49177, lsl #48
	fmov	d1, x8
	fadd	d0, d0, d1
	mov	x8, #25393
	movk	x8, #6754, lsl #16
	movk	x8, #46177, lsl #32
	movk	x8, #48624, lsl #48
	fmov	d1, x8
	fadd	d2, d0, d1
	fsub	d0, d0, d2
	mov	w8, #4
	b	LBB100_31
LBB100_25:
	mov	x20, x0
	mov	x9, #4710765210229538816
	bfxil	x9, x19, #0, #52
	fmov	d0, x9
	fcvtzs	w9, d0
	scvtf	d1, w9
	fsub	d0, d0, d1
	mov	x9, #4715268809856909312
	fmov	d2, x9
	fcvtzs	w9, d0, #24
	fmul	d0, d0, d2
	scvtf	d3, w9
	stp	d1, d3, [sp]
	fsub	d0, d0, d3
	fmul	d0, d0, d2
	str	d0, [sp, #16]
	cmp	w9, #0
	mov	w9, #1
	cinc	x9, x9, ne
	fcmp	d0, #0.0
	mov	w10, #3
	csel	x1, x9, x10, eq
	stp	xzr, xzr, [sp, #24]
	str	xzr, [sp, #40]
	sub	w3, w8, #1046
	mov	x0, sp
	add	x2, sp, #24
	mov	w4, #1
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math14rem_pio2_large14rem_pio2_large
	tbnz	x19, #63, LBB100_34
	ldp	d0, d1, [sp, #24]
	str	w0, [x20, #8]
	b	LBB100_35
LBB100_27:
	mov	x8, #1413480448
	movk	x8, #8699, lsl #32
	movk	x8, #16377, lsl #48
	fmov	d1, x8
	fadd	d0, d0, d1
	mov	x8, #25393
	movk	x8, #6754, lsl #16
	movk	x8, #46177, lsl #32
	movk	x8, #15824, lsl #48
	fmov	d1, x8
	fadd	d2, d0, d1
	fsub	d0, d0, d2
	mov	w8, #-1
	b	LBB100_31
LBB100_28:
	mov	x8, #1413480448
	movk	x8, #8699, lsl #32
	movk	x8, #16393, lsl #48
	fmov	d1, x8
	fadd	d0, d0, d1
	mov	x8, #25393
	movk	x8, #6754, lsl #16
	movk	x8, #46177, lsl #32
	movk	x8, #15840, lsl #48
	fmov	d1, x8
	fadd	d2, d0, d1
	fsub	d0, d0, d2
	mov	w8, #-2
	b	LBB100_31
LBB100_29:
	mov	x8, #2133852160
	movk	x8, #55676, lsl #32
	movk	x8, #16402, lsl #48
	fmov	d1, x8
	fadd	d0, d0, d1
	mov	x8, #38090
	movk	x8, #42899, lsl #16
	movk	x8, #3729, lsl #32
	movk	x8, #15849, lsl #48
	fmov	d1, x8
	fadd	d2, d0, d1
	fsub	d0, d0, d2
	mov	w8, #-3
	b	LBB100_31
LBB100_30:
	mov	x8, #1413480448
	movk	x8, #8699, lsl #32
	movk	x8, #16409, lsl #48
	fmov	d1, x8
	fadd	d0, d0, d1
	mov	x8, #25393
	movk	x8, #6754, lsl #16
	movk	x8, #46177, lsl #32
	movk	x8, #15856, lsl #48
	fmov	d1, x8
	fadd	d2, d0, d1
	fsub	d0, d0, d2
	mov	w8, #-4
LBB100_31:
	str	w8, [x0, #8]
	fadd	d0, d0, d1
	str	d2, [x0]
LBB100_32:
	str	d0, [x0, #16]
LBB100_33:
	.cfi_def_cfa wsp, 80
	ldp	x29, x30, [sp, #64]
	ldp	x20, x19, [sp, #48]
	add	sp, sp, #80
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	ret
LBB100_34:
	.cfi_restore_state
	neg	w8, w0
	ldp	d0, d1, [sp, #24]
	fneg	d0, d0
	fneg	d1, d1
	str	w8, [x20, #8]
LBB100_35:
	str	d0, [x20]
	str	d1, [x20, #16]
	b	LBB100_33
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math9fmin_fmax4fmax
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math9fmin_fmax4fmax:
	.cfi_startproc
	fcmp	d0, d1
	fccmp	d0, d0, #1, pl
	fcsel	d0, d1, d0, vs
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math9fmin_fmax4fmin
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math9fmin_fmax4fmin:
	.cfi_startproc
	fcmp	d0, d1
	fccmp	d1, d1, #1, pl
	fcsel	d0, d0, d1, vs
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math9fmin_fmax5fmaxf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math9fmin_fmax5fmaxf:
	.cfi_startproc
	fcmp	s0, s1
	fccmp	s0, s0, #1, pl
	fcsel	s0, s1, s0, vs
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math9fmin_fmax5fminf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math9fmin_fmax5fminf:
	.cfi_startproc
	fcmp	s0, s1
	fccmp	s1, s1, #1, pl
	fcsel	s0, s0, s1, vs
	ret
	.cfi_endproc

	.section	__TEXT,__literal8,8byte_literals
	.p2align	3, 0x0
lCPI105_0:
	.long	0x3e91e9ee
	.long	0x3e789e26
lCPI105_1:
	.long	0x3f2aaaaa
	.long	0x3eccce13
lCPI105_2:
	.long	0x3805ff67
	.long	0x36f5d7bd
lCPI105_3:
	.long	0x3af135b4
	.long	0x3a4beed6
lCPI105_4:
	.long	0x3cda40e4
	.long	0x3c98bf54
lCPI105_5:
	.long	0x3e15dce6
	.long	0x3e300f6e
lCPI105_6:
	.long	0x3ea6cc7a
	.long	0x3f38d0c5
lCPI105_7:
	.long	0x3e5c245a
	.long	0x3fb22d3b
lCPI105_8:
	.long	0xbd9e233f
	.long	0x3f800000
lCPI105_9:
	.long	0x3c5b3c5e
	.long	0x3b52d5db
lCPI105_10:
	.long	0x3e6a7578
	.long	0x3dd572af
lCPI105_11:
	.long	0x3f7a4bb2
	.long	0x3f44efdf
lCPI105_12:
	.long	0x3fba3ae7
	.long	0x4008392d
lCPI105_13:
	.long	0x3f2200f4
	.long	0x401d2ebe
lCPI105_14:
	.long	0x39a3f927
	.long	0x39afe9f7
lCPI105_15:
	.long	0xbb7177fe
	.long	0x3b141699
lCPI105_16:
	.long	0x3c93373d
	.long	0xbc28fcfe
lCPI105_17:
	.long	0xbe17213c
	.long	0x3d845a15
	.section	__TEXT,__text,regular,pure_instructions
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math9lgammaf_r9lgammaf_r
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math9lgammaf_r9lgammaf_r:
	.cfi_startproc
	sub	sp, sp, #80
	.cfi_def_cfa_offset 80
	stp	d9, d8, [sp, #16]
	stp	x22, x21, [sp, #32]
	stp	x20, x19, [sp, #48]
	stp	x29, x30, [sp, #64]
	add	x29, sp, #64
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset w21, -40
	.cfi_offset w22, -48
	.cfi_offset b8, -56
	.cfi_offset b9, -64
	.cfi_remember_state
	mov.16b	v7, v0
	fabs	s0, s0
	fmov	w20, s0
	lsr	w21, w20, #23
	cmp	w21, #254
	b.ls	LBB105_2
	fmul	s0, s7, s7
	mov	w0, #1
	b	LBB105_38
LBB105_2:
	fmov	w19, s7
	lsr	w8, w20, #24
	cmp	w8, #53
	b.hs	LBB105_5
	asr	w8, w19, #31
	orr	w0, w8, #0x1
	cmp	w20, #2048, lsl #12
	b.lo	LBB105_11
	mov	w8, #-127
	b	LBB105_22
LBB105_5:
	movi.2d	v9, #0000000000000000
	tbnz	w19, #31, LBB105_13
	mov	w0, #1
LBB105_7:
	movi.2d	v8, #0000000000000000
LBB105_8:
	mov	w9, #1065353216
	cmp	w20, w9
	b.eq	LBB105_10
	mov	w9, #1073741824
	cmp	w20, w9
	b.ne	LBB105_17
LBB105_10:
	fsub	s0, s8, s9
	cmn	w19, #1
	fcsel	s0, s9, s0, gt
	b	LBB105_38
LBB105_11:
	fcmp	s7, #0.0
	b.ne	LBB105_21
	fmul	s0, s7, s7
	fmov	s1, #-1.00000000
	fdiv	s0, s1, s0
	fneg	s0, s0
	b	LBB105_38
LBB105_13:
	fmov	s0, #-0.50000000
	fmul	s0, s7, s0
	fmov	w9, s0
	ubfx	w10, w9, #23, #8
	mov.16b	v1, v0
	cmp	w10, #149
	b.hi	LBB105_27
	cmp	w10, #126
	b.ls	LBB105_26
	sub	w10, w10, #127
	mov	w11, #8388607
	lsr	w11, w11, w10
	mov.16b	v1, v0
	tst	w11, w9
	b.eq	LBB105_27
	and	w11, w11, w9, asr #31
	mov	w12, #-8388608
	asr	w10, w12, w10
	add	w9, w11, w9
	and	w9, w9, w10
	fmov	s1, w9
	b	LBB105_27
LBB105_17:
	lsr	w9, w20, #30
	cbnz	w9, LBB105_23
	mov	w8, #26215
	movk	w8, #16230, lsl #16
	cmp	w20, w8
	b.hs	LBB105_41
	mov	x21, x0
	str	q7, [sp]
	mov.16b	v0, v7
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4logf4logf
	fneg	s0, s0
	fmov	s1, #1.00000000
	mov	w8, #18975
	movk	w8, #16187, lsl #16
	cmp	w20, w8
	b.ls	LBB105_46
	mov	x0, x21
	ldr	q7, [sp]
	b	LBB105_43
LBB105_21:
	movi.2s	v1, #76, lsl #24
	fmul	s0, s0, s1
	fmov	w20, s0
	mov	w8, #-152
LBB105_22:
	add	w9, w20, #1199, lsl #12
	add	w9, w9, #2829
	add	w8, w8, w9, lsr #23
	and	w9, w9, #0x7fffff
	mov	w10, #1267
	movk	w10, #16181, lsl #16
	add	w9, w9, w10
	fmov	s0, w9
	fmov	s1, #-1.00000000
	fadd	s0, s0, s1
	fmov	s1, #2.00000000
	fadd	s1, s0, s1
	fdiv	s1, s0, s1
	fmul	s2, s1, s1
	fmul	s3, s2, s2
	dup.2s	v4, v3[0]
Lloh590:
	adrp	x9, lCPI105_0@PAGE
Lloh591:
	ldr	d5, [x9, lCPI105_0@PAGEOFF]
	fmul.2s	v3, v5, v3[0]
Lloh592:
	adrp	x9, lCPI105_1@PAGE
Lloh593:
	ldr	d5, [x9, lCPI105_1@PAGEOFF]
	fadd.2s	v3, v3, v5
	mov.s	v4[0], v2[0]
	fmul.2s	v2, v4, v3
	faddp.2s	s2, v2
	fmov	s3, #0.50000000
	fmul	s3, s0, s3
	fmul	s3, s0, s3
	scvtf	s4, w8
	fadd	s2, s3, s2
	fmul	s1, s1, s2
	mov	w8, #63441
	movk	w8, #14103, lsl #16
	fmov	s2, w8
	fmul	s2, s4, s2
	fadd	s1, s2, s1
	fsub	s1, s1, s3
	fadd	s0, s0, s1
	mov	w8, #29056
	movk	w8, #16177, lsl #16
	fmov	s1, w8
	fmul	s1, s4, s1
	fadd	s0, s1, s0
	fneg	s0, s0
	b	LBB105_38
LBB105_23:
	cmp	w8, #65
	b.hs	LBB105_44
	fcvtzs	w8, s7
	scvtf	s0, w8
	fsub	s0, s7, s0
Lloh594:
	adrp	x9, lCPI105_2@PAGE
Lloh595:
	ldr	d1, [x9, lCPI105_2@PAGEOFF]
	fmul.2s	v1, v1, v0[0]
Lloh596:
	adrp	x9, lCPI105_3@PAGE
Lloh597:
	ldr	d2, [x9, lCPI105_3@PAGEOFF]
	fadd.2s	v1, v1, v2
	fmul.2s	v1, v1, v0[0]
Lloh598:
	adrp	x9, lCPI105_4@PAGE
Lloh599:
	ldr	d2, [x9, lCPI105_4@PAGEOFF]
	fadd.2s	v1, v1, v2
	fmul.2s	v1, v1, v0[0]
Lloh600:
	adrp	x9, lCPI105_5@PAGE
Lloh601:
	ldr	d2, [x9, lCPI105_5@PAGEOFF]
	fadd.2s	v1, v1, v2
	fmul.2s	v1, v1, v0[0]
Lloh602:
	adrp	x9, lCPI105_6@PAGE
Lloh603:
	ldr	d2, [x9, lCPI105_6@PAGEOFF]
	fadd.2s	v1, v1, v2
	fmul.2s	v1, v1, v0[0]
Lloh604:
	adrp	x9, lCPI105_7@PAGE
Lloh605:
	ldr	d2, [x9, lCPI105_7@PAGEOFF]
	fadd.2s	v1, v1, v2
	fmul.2s	v1, v1, v0[0]
Lloh606:
	adrp	x9, lCPI105_8@PAGE
Lloh607:
	ldr	d2, [x9, lCPI105_8@PAGEOFF]
	fadd.2s	v1, v1, v2
	fmul	s2, s0, s1
	fmov	s3, #0.50000000
	fmul	s3, s0, s3
	mov	s1, v1[1]
	fdiv	s1, s2, s1
	fadd	s9, s3, s1
	cmp	w8, #2
	b.le	LBB105_10
	fmov	s1, #6.00000000
	fadd	s1, s0, s1
	cmp	w8, #6
	fmov	s2, #1.00000000
	fcsel	s1, s1, s2, hi
	fmov	s3, #5.00000000
	fadd	s3, s0, s3
	fmul	s1, s3, s1
	cmp	w8, #5
	fcsel	s1, s1, s2, hi
	fmov	s3, #4.00000000
	fadd	s3, s0, s3
	fmul	s1, s3, s1
	cmp	w8, #4
	fcsel	s1, s1, s2, hi
	fmov	s3, #3.00000000
	fadd	s3, s0, s3
	fmul	s1, s3, s1
	cmp	w8, #3
	fcsel	s1, s2, s1, eq
	fmov	s2, #2.00000000
	fadd	s0, s0, s2
	fmul	s0, s0, s1
	mov	x20, x0
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4logf4logf
	mov	x0, x20
	fadd	s9, s9, s0
	b	LBB105_10
LBB105_26:
	cmn	w9, #1
	fmov	s1, #-1.00000000
	movi.2d	v2, #0000000000000000
	fcsel	s1, s2, s1, gt
LBB105_27:
	fsub	s0, s0, s1
	fadd	s0, s0, s0
	fcvtzs	x9, s0, #2
	add	x9, x9, #1
	add	x9, x9, x9, lsr #63
	asr	x9, x9, #1
	fcvt	d0, s0
	scvtf	d1, x9
	fmov	d2, #-0.50000000
	fmul	d1, d1, d2
	fadd	d0, d0, d1
	mov	x10, #11544
	movk	x10, #21572, lsl #16
	movk	x10, #8699, lsl #32
	movk	x10, #16393, lsl #48
	fmov	d1, x10
	fmul	d1, d0, d1
	fmul	d0, d1, d1
	cmp	x9, #1
	b.eq	LBB105_35
	cmp	x9, #2
	b.eq	LBB105_34
	cmp	x9, #3
	fmul	d2, d0, d0
	mov	x9, #20585
	movk	x9, #57582, lsl #16
	movk	x9, #37698, lsl #32
	movk	x9, #16121, lsl #48
	fmov	d3, x9
	fmul	d3, d0, d3
	mov	x9, #7719
	movk	x9, #59407, lsl #16
	movk	x9, #49287, lsl #32
	movk	x9, #48982, lsl #48
	fmov	d4, x9
	fadd	d3, d3, d4
	mov	x9, #-41343
	movk	x9, #64780, lsl #16
	movk	x9, #49119, lsl #48
	fmov	d4, x9
	fmul	d4, d0, d4
	fmov	d5, #1.00000000
	fadd	d4, d4, d5
	mov	x9, #14914
	movk	x9, #57605, lsl #16
	movk	x9, #21843, lsl #32
	movk	x9, #16293, lsl #48
	fmov	d5, x9
	fmul	d5, d2, d5
	fadd	d4, d4, d5
	fmul	d5, d0, d2
	fmul	d3, d5, d3
	fadd	d3, d4, d3
	fcvt	s3, d3
	fneg	s3, s3
	mov	x9, #18087
	movk	x9, #35899, lsl #16
	movk	x9, #52615, lsl #32
	movk	x9, #16070, lsl #48
	fmov	d4, x9
	fmul	d4, d0, d4
	mov	x9, #59252
	movk	x9, #58058, lsl #16
	movk	x9, #249, lsl #32
	movk	x9, #48938, lsl #48
	fmov	d5, x9
	fadd	d4, d4, d5
	fmul	d5, d1, d0
	mov	x9, #64434
	movk	x9, #35182, lsl #16
	movk	x9, #4368, lsl #32
	movk	x9, #16257, lsl #48
	fmov	d6, x9
	fmul	d0, d0, d6
	mov	x9, #44151
	movk	x9, #21707, lsl #16
	movk	x9, #21845, lsl #32
	movk	x9, #49093, lsl #48
	fmov	d6, x9
	fadd	d0, d0, d6
	fmul	d0, d5, d0
	fadd	d0, d1, d0
	fmul	d1, d5, d2
	fmul	d1, d1, d4
	fadd	d0, d1, d0
	fcvt	s0, d0
	fcsel	s0, s3, s0, eq
	fcmp	s0, #0.0
	b.eq	LBB105_37
LBB105_30:
	fneg	s7, s7
	fneg	s1, s0
	fcsel	s0, s0, s1, gt
	mov	w9, #-1
	csinc	w0, w9, wzr, gt
	fmul	s0, s0, s7
	mov	w9, #4059
	movk	w9, #16457, lsl #16
	fmov	s1, w9
	fdiv	s8, s1, s0
	fmov	w9, s8
	cmp	w9, #2048, lsl #12
	b.lt	LBB105_39
	mov	w10, #2139095039
	cmp	w9, w10
	b.hi	LBB105_8
	mov	w10, #1065353216
	cmp	w9, w10
	b.eq	LBB105_7
	mov	w10, #-127
	b	LBB105_54
LBB105_34:
	fmul	d2, d0, d0
	mov	x9, #18087
	movk	x9, #35899, lsl #16
	movk	x9, #52615, lsl #32
	movk	x9, #16070, lsl #48
	fmov	d3, x9
	fmul	d3, d0, d3
	mov	x9, #59252
	movk	x9, #58058, lsl #16
	movk	x9, #249, lsl #32
	movk	x9, #48938, lsl #48
	fmov	d4, x9
	fadd	d3, d3, d4
	fnmul	d4, d1, d0
	mov	x9, #64434
	movk	x9, #35182, lsl #16
	movk	x9, #4368, lsl #32
	movk	x9, #16257, lsl #48
	fmov	d5, x9
	fmul	d0, d0, d5
	mov	x9, #44151
	movk	x9, #21707, lsl #16
	movk	x9, #21845, lsl #32
	movk	x9, #49093, lsl #48
	fmov	d5, x9
	fadd	d0, d0, d5
	fmul	d0, d4, d0
	fsub	d0, d0, d1
	fmul	d1, d4, d2
	fmul	d1, d1, d3
	fadd	d0, d1, d0
	b	LBB105_36
LBB105_35:
	fmul	d1, d0, d0
	mov	x9, #20585
	movk	x9, #57582, lsl #16
	movk	x9, #37698, lsl #32
	movk	x9, #16121, lsl #48
	fmov	d2, x9
	fmul	d2, d0, d2
	mov	x9, #7719
	movk	x9, #59407, lsl #16
	movk	x9, #49287, lsl #32
	movk	x9, #48982, lsl #48
	fmov	d3, x9
	fadd	d2, d2, d3
	mov	x9, #-41343
	movk	x9, #64780, lsl #16
	movk	x9, #49119, lsl #48
	fmov	d3, x9
	fmul	d3, d0, d3
	fmov	d4, #1.00000000
	fadd	d3, d3, d4
	mov	x9, #14914
	movk	x9, #57605, lsl #16
	movk	x9, #21843, lsl #32
	movk	x9, #16293, lsl #48
	fmov	d4, x9
	fmul	d4, d1, d4
	fadd	d3, d3, d4
	fmul	d0, d0, d1
	fmul	d0, d0, d2
	fadd	d0, d3, d0
LBB105_36:
	fcvt	s0, d0
	fcmp	s0, #0.0
	b.ne	LBB105_30
LBB105_37:
	fsub	s0, s7, s7
	fmov	s1, #1.00000000
	mov	w0, #1
	fdiv	s0, s1, s0
LBB105_38:
	.cfi_def_cfa wsp, 80
	ldp	x29, x30, [sp, #64]
	ldp	x20, x19, [sp, #48]
	ldp	x22, x21, [sp, #32]
	ldp	d9, d8, [sp, #16]
	add	sp, sp, #80
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	.cfi_restore b8
	.cfi_restore b9
	ret
LBB105_39:
	.cfi_restore_state
	fcmp	s8, #0.0
	b.ne	LBB105_52
	fmul	s0, s8, s8
	fmov	s1, #-1.00000000
	fdiv	s8, s1, s0
	b	LBB105_8
LBB105_41:
	mov	w8, #42519
	movk	w8, #16349, lsl #16
	cmp	w20, w8
	b.ls	LBB105_48
	fmov	s1, #2.00000000
	movi.2d	v0, #0000000000000000
LBB105_43:
	fsub	s1, s1, s7
	fmul	s2, s1, s1
	mov	w8, #33698
	movk	w8, #14291, lsl #16
	fmov	s3, w8
	fmul	s3, s2, s3
	mov	w8, #38759
	movk	w8, #14695, lsl #16
	fmov	s4, w8
	fadd	s3, s3, s4
	fmul	s3, s2, s3
	mov	w8, #21665
	movk	w8, #15004, lsl #16
	fmov	s4, w8
	fadd	s3, s3, s4
	fmul	s3, s2, s3
	mov	w8, #638
	movk	w8, #15346, lsl #16
	fmov	s4, w8
	fadd	s3, s3, s4
	fmul	s3, s2, s3
	mov	w8, #61441
	movk	w8, #15753, lsl #16
	fmov	s4, w8
	fadd	s3, s3, s4
	fmul	s3, s2, s3
	mov	w8, #9023
	movk	w8, #15774, lsl #16
	fmov	s4, w8
	fadd	s3, s3, s4
	mov	w8, #11381
	movk	w8, #14396, lsl #16
	fmov	s4, w8
	fmul	s4, s2, s4
	mov	w8, #33861
	movk	w8, #14562, lsl #16
	fmov	s5, w8
	fadd	s4, s4, s5
	fmul	s4, s2, s4
	mov	w8, #46644
	movk	w8, #14853, lsl #16
	fmov	s5, w8
	fadd	s4, s4, s5
	fmul	s4, s2, s4
	mov	w8, #28358
	movk	w8, #15165, lsl #16
	fmov	s5, w8
	fadd	s4, s4, s5
	fmul	s4, s2, s4
	mov	w8, #39189
	movk	w8, #15528, lsl #16
	fmov	s5, w8
	fadd	s4, s4, s5
	fmul	s4, s2, s4
	mov	w8, #6758
	movk	w8, #16037, lsl #16
	fmov	s5, w8
	fadd	s4, s4, s5
	fmul	s2, s2, s4
	fmul	s3, s1, s3
	fadd	s2, s3, s2
	fmov	s3, #-0.50000000
	fmul	s1, s1, s3
	fadd	s1, s2, s1
	fadd	s9, s0, s1
	b	LBB105_10
LBB105_44:
	mov	x20, x0
	mov.16b	v0, v7
	str	q7, [sp]
	bl	__RNvNtNtCsdtCTLUYU3dx_4libm4math4logf4logf
	cmp	w21, #185
	b.hs	LBB105_51
	fmov	s1, #1.00000000
	ldr	q5, [sp]
	fdiv	s1, s1, s5
	fmul	s2, s1, s1
	mov	w8, #50408
	movk	w8, #47829, lsl #16
	fmov	s3, w8
	fmul	s3, s2, s3
	mov	w8, #15826
	movk	w8, #14939, lsl #16
	fmov	s4, w8
	fadd	s3, s3, s4
	fmul	s3, s2, s3
	mov	w8, #1628
	movk	w8, #47644, lsl #16
	fmov	s4, w8
	fadd	s3, s3, s4
	fmul	s3, s2, s3
	mov	w8, #3325
	movk	w8, #14928, lsl #16
	fmov	s4, w8
	fadd	s3, s3, s4
	fmul	s3, s2, s3
	mov	w8, #2913
	movk	w8, #47926, lsl #16
	fmov	s4, w8
	fadd	s3, s3, s4
	fmul	s2, s2, s3
	mov	w8, #43691
	movk	w8, #15786, lsl #16
	fmov	s3, w8
	fadd	s2, s2, s3
	fmul	s1, s1, s2
	mov	w8, #32541
	movk	w8, #16086, lsl #16
	fmov	s2, w8
	fadd	s1, s1, s2
	fmov	s2, #-0.50000000
	fadd	s2, s5, s2
	fmov	s3, #-1.00000000
	fadd	s0, s0, s3
	fmul	s0, s2, s0
	fadd	s9, s1, s0
	mov	x0, x20
	b	LBB105_10
LBB105_46:
	mov	w8, #13063
	movk	w8, #15981, lsl #16
	cmp	w20, w8
	mov	x0, x21
	b.ls	LBB105_55
	mov	w8, #23308
	movk	w8, #48876, lsl #16
	fmov	s1, w8
	ldr	q7, [sp]
	b	LBB105_50
LBB105_48:
	mov	w8, #42527
	movk	w8, #16285, lsl #16
	cmp	w20, w8
	b.ls	LBB105_56
	movi.2d	v0, #0000000000000000
	mov	w8, #5827
	movk	w8, #49083, lsl #16
	fmov	s1, w8
LBB105_50:
	fadd	s1, s7, s1
	fmul	s2, s1, s1
	fmul	s3, s1, s2
	mov	w8, #31595
	movk	w8, #14757, lsl #16
	fmov	s4, w8
	fmul	s4, s3, s4
	mov	w8, #62582
	movk	w8, #47799, lsl #16
	fmov	s5, w8
	fadd	s4, s4, s5
	fmul	s4, s3, s4
	mov	w8, #59143
	movk	w8, #15303, lsl #16
	fmov	s5, w8
	fadd	s4, s4, s5
	fmul	s4, s3, s4
	mov	w8, #19783
	movk	w8, #48390, lsl #16
	fmov	s5, w8
	fadd	s4, s4, s5
	fmul	s4, s3, s4
	mov	w8, #47454
	movk	w8, #16119, lsl #16
	fmov	s5, w8
	fadd	s4, s4, s5
Lloh608:
	adrp	x8, lCPI105_14@PAGE
Lloh609:
	ldr	d5, [x8, lCPI105_14@PAGEOFF]
	fmul.2s	v5, v5, v3[0]
	mov	w8, #63591
	movk	w8, #14950, lsl #16
	fmov	s6, w8
	fsub.2s	v6, v6, v5
	mov	w8, #12421
	movk	w8, #47629, lsl #16
	dup.2s	v7, w8
	fadd.2s	v5, v5, v7
	mov.s	v6[1], v5[1]
	fmul.2s	v5, v6, v3[0]
Lloh610:
	adrp	x8, lCPI105_15@PAGE
Lloh611:
	ldr	d6, [x8, lCPI105_15@PAGEOFF]
	fadd.2s	v5, v5, v6
	fmul.2s	v5, v5, v3[0]
Lloh612:
	adrp	x8, lCPI105_16@PAGE
Lloh613:
	ldr	d6, [x8, lCPI105_16@PAGEOFF]
	fadd.2s	v5, v5, v6
	fmul.2s	v5, v5, v3[0]
Lloh614:
	adrp	x8, lCPI105_17@PAGE
Lloh615:
	ldr	d6, [x8, lCPI105_17@PAGEOFF]
	fadd.2s	v5, v5, v6
	fmul	s2, s2, s4
	fmul.s	s1, s1, v5[1]
	fadd	s1, s5, s1
	fmul	s1, s3, s1
	mov	w8, #7250
	movk	w8, #12774, lsl #16
	fmov	s3, w8
	fsub	s1, s3, s1
	fsub	s1, s2, s1
	mov	w8, #52685
	movk	w8, #48632, lsl #16
	fmov	s2, w8
	fadd	s1, s1, s2
	fadd	s9, s0, s1
	b	LBB105_10
LBB105_51:
	fmov	s1, #-1.00000000
	fadd	s0, s0, s1
	ldr	q1, [sp]
	fmul	s9, s1, s0
	mov	x0, x20
	b	LBB105_10
LBB105_52:
	tbnz	w9, #31, LBB105_58
	movi.2s	v0, #76, lsl #24
	fmul	s0, s8, s0
	fmov	w9, s0
	mov	w10, #-152
LBB105_54:
	add	w9, w9, #1199, lsl #12
	add	w9, w9, #2829
	add	w10, w10, w9, lsr #23
	and	w9, w9, #0x7fffff
	mov	w11, #1267
	movk	w11, #16181, lsl #16
	add	w9, w9, w11
	fmov	s0, w9
	fmov	s1, #-1.00000000
	fadd	s0, s0, s1
	fmov	s1, #2.00000000
	fadd	s1, s0, s1
	fdiv	s1, s0, s1
	fmul	s2, s1, s1
	fmul	s3, s2, s2
	dup.2s	v4, v3[0]
Lloh616:
	adrp	x9, lCPI105_0@PAGE
Lloh617:
	ldr	d5, [x9, lCPI105_0@PAGEOFF]
	fmul.2s	v3, v5, v3[0]
Lloh618:
	adrp	x9, lCPI105_1@PAGE
Lloh619:
	ldr	d5, [x9, lCPI105_1@PAGEOFF]
	fadd.2s	v3, v3, v5
	mov.s	v4[0], v2[0]
	fmul.2s	v2, v4, v3
	faddp.2s	s2, v2
	fmov	s3, #0.50000000
	fmul	s3, s0, s3
	fmul	s3, s0, s3
	scvtf	s4, w10
	fadd	s2, s3, s2
	fmul	s1, s1, s2
	mov	w9, #63441
	movk	w9, #14103, lsl #16
	fmov	s2, w9
	fmul	s2, s4, s2
	fadd	s1, s2, s1
	fsub	s1, s1, s3
	fadd	s0, s0, s1
	mov	w9, #29056
	movk	w9, #16177, lsl #16
	fmov	s1, w9
	fmul	s1, s4, s1
	fadd	s8, s1, s0
	b	LBB105_8
LBB105_55:
	ldr	q7, [sp]
	b	LBB105_57
LBB105_56:
	fmov	s0, #-1.00000000
	fadd	s7, s7, s0
	movi.2d	v0, #0000000000000000
LBB105_57:
Lloh620:
	adrp	x8, lCPI105_9@PAGE
Lloh621:
	ldr	d1, [x8, lCPI105_9@PAGEOFF]
	fmul.2s	v1, v1, v7[0]
Lloh622:
	adrp	x8, lCPI105_10@PAGE
Lloh623:
	ldr	d2, [x8, lCPI105_10@PAGEOFF]
	fadd.2s	v1, v1, v2
	fmul.2s	v1, v1, v7[0]
Lloh624:
	adrp	x8, lCPI105_11@PAGE
Lloh625:
	ldr	d2, [x8, lCPI105_11@PAGEOFF]
	fadd.2s	v1, v1, v2
	fmul.2s	v1, v1, v7[0]
Lloh626:
	adrp	x8, lCPI105_12@PAGE
Lloh627:
	ldr	d2, [x8, lCPI105_12@PAGEOFF]
	fadd.2s	v1, v1, v2
	fmul.2s	v1, v1, v7[0]
Lloh628:
	adrp	x8, lCPI105_13@PAGE
Lloh629:
	ldr	d2, [x8, lCPI105_13@PAGEOFF]
	fadd.2s	v1, v1, v2
	fmul.2s	v1, v1, v7[0]
Lloh630:
	adrp	x8, lCPI105_8@PAGE
Lloh631:
	ldr	d2, [x8, lCPI105_8@PAGEOFF]
	fadd.2s	v1, v1, v2
	fmul	s2, s7, s1
	fmov	s3, #-0.50000000
	fmul	s3, s7, s3
	mov	s1, v1[1]
	fdiv	s1, s2, s1
	fadd	s1, s1, s3
	fadd	s9, s0, s1
	b	LBB105_10
LBB105_58:
	fsub	s0, s8, s8
	movi.2d	v1, #0000000000000000
	fdiv	s8, s0, s1
	b	LBB105_8
	.loh AdrpLdr	Lloh592, Lloh593
	.loh AdrpAdrp	Lloh590, Lloh592
	.loh AdrpLdr	Lloh590, Lloh591
	.loh AdrpLdr	Lloh606, Lloh607
	.loh AdrpAdrp	Lloh604, Lloh606
	.loh AdrpLdr	Lloh604, Lloh605
	.loh AdrpAdrp	Lloh602, Lloh604
	.loh AdrpLdr	Lloh602, Lloh603
	.loh AdrpAdrp	Lloh600, Lloh602
	.loh AdrpLdr	Lloh600, Lloh601
	.loh AdrpAdrp	Lloh598, Lloh600
	.loh AdrpLdr	Lloh598, Lloh599
	.loh AdrpAdrp	Lloh596, Lloh598
	.loh AdrpLdr	Lloh596, Lloh597
	.loh AdrpAdrp	Lloh594, Lloh596
	.loh AdrpLdr	Lloh594, Lloh595
	.loh AdrpLdr	Lloh614, Lloh615
	.loh AdrpAdrp	Lloh612, Lloh614
	.loh AdrpLdr	Lloh612, Lloh613
	.loh AdrpAdrp	Lloh610, Lloh612
	.loh AdrpLdr	Lloh610, Lloh611
	.loh AdrpLdr	Lloh608, Lloh609
	.loh AdrpLdr	Lloh618, Lloh619
	.loh AdrpAdrp	Lloh616, Lloh618
	.loh AdrpLdr	Lloh616, Lloh617
	.loh AdrpLdr	Lloh630, Lloh631
	.loh AdrpAdrp	Lloh628, Lloh630
	.loh AdrpLdr	Lloh628, Lloh629
	.loh AdrpAdrp	Lloh626, Lloh628
	.loh AdrpLdr	Lloh626, Lloh627
	.loh AdrpAdrp	Lloh624, Lloh626
	.loh AdrpLdr	Lloh624, Lloh625
	.loh AdrpAdrp	Lloh622, Lloh624
	.loh AdrpLdr	Lloh622, Lloh623
	.loh AdrpAdrp	Lloh620, Lloh622
	.loh AdrpLdr	Lloh620, Lloh621
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math9remainder9remainder
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math9remainder9remainder:
	.cfi_startproc
	b	__RNvNtNtCsdtCTLUYU3dx_4libm4math6remquo6remquo
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math9roundeven10roundevenf
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math9roundeven10roundevenf:
	.cfi_startproc
	fmov	w8, s0
	and	w9, w8, #0x7f000000
	mov	w10, #1249902592
	cmp	w9, w10
	b.hi	LBB107_4
	tbnz	w8, #31, LBB107_5
	movi.2s	v1, #75, lsl #24
	fadd	s0, s0, s1
	movi.2s	v1, #203, lsl #24
	fadd	s0, s0, s1
	fcmp	s0, #0.0
	b.ne	LBB107_4
	movi.2d	v0, #0000000000000000
LBB107_4:
	ret
LBB107_5:
	movi.2s	v1, #203, lsl #24
	fadd	s0, s0, s1
	movi.2s	v1, #75, lsl #24
	fadd	s0, s0, s1
	fcmp	s0, #0.0
	b.ne	LBB107_4
	movi.2s	v0, #128, lsl #24
	ret
	.cfi_endproc

	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math9roundeven9roundeven
	.p2align	2
__RNvNtNtCsdtCTLUYU3dx_4libm4math9roundeven9roundeven:
	.cfi_startproc
	fmov	x8, d0
	and	x9, x8, #0x7ff0000000000000
	mov	x10, #4836865999795912704
	cmp	x9, x10
	b.hi	LBB108_4
	tbnz	x8, #63, LBB108_5
	mov	x8, #4841369599423283200
	fmov	d1, x8
	fadd	d0, d0, d1
	mov	x8, #-4382002437431492608
	fmov	d1, x8
	fadd	d0, d0, d1
	fcmp	d0, #0.0
	b.ne	LBB108_4
	movi.2d	v0, #0000000000000000
LBB108_4:
	ret
LBB108_5:
	mov	x8, #-4382002437431492608
	fmov	d1, x8
	fadd	d0, d0, d1
	mov	x8, #4841369599423283200
	fmov	d1, x8
	fadd	d0, d0, d1
	fcmp	d0, #0.0
	b.ne	LBB108_4
	mov	x8, #-9223372036854775808
	fmov	d0, x8
	ret
	.cfi_endproc

	.globl	__RNvXs0_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsjNtB5_3Int12unsigned_abs
	.p2align	2
__RNvXs0_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsjNtB5_3Int12unsigned_abs:
	.cfi_startproc
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
Lloh632:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.98@PAGE
Lloh633:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.98@PAGEOFF
Lloh634:
	adrp	x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.100@PAGE
Lloh635:
	add	x2, x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.100@PAGEOFF
	mov	w1, #15
	bl	__RNvNtCsgtPOCBgevO_4core9panicking5panic
	.loh AdrpAdd	Lloh634, Lloh635
	.loh AdrpAdd	Lloh632, Lloh633
	.cfi_endproc

	.globl	__RNvXs0_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsjNtB5_3Int5ilog2
	.p2align	2
__RNvXs0_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsjNtB5_3Int5ilog2:
	.cfi_startproc
	cbz	x0, LBB110_2
	clz	x8, x0
	eor	w0, w8, #0x3f
	ret
LBB110_2:
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
Lloh636:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.100@PAGE
Lloh637:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.100@PAGEOFF
	bl	__RNvNtNtNtCsgtPOCBgevO_4core3num3imp9int_log1030panic_for_nonpositive_argument
	.loh AdrpAdd	Lloh636, Lloh637
	.cfi_endproc

	.globl	__RNvXs2_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsiNtB5_3Int5ilog2
	.p2align	2
__RNvXs2_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsiNtB5_3Int5ilog2:
	.cfi_startproc
	cmp	x0, #1
	b.lt	LBB111_2
	clz	x8, x0
	eor	w0, w8, #0x3f
	ret
LBB111_2:
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
Lloh638:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.100@PAGE
Lloh639:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.100@PAGEOFF
	bl	__RNvNtNtNtCsgtPOCBgevO_4core3num3imp9int_log1030panic_for_nonpositive_argument
	.loh AdrpAdd	Lloh638, Lloh639
	.cfi_endproc

	.globl	__RNvXs2_NtNtNtCsdtCTLUYU3dx_4libm4math7support3bigoNtNtB7_10int_traits4HInt9widen_mul
	.p2align	2
__RNvXs2_NtNtNtCsdtCTLUYU3dx_4libm4math7support3bigoNtNtB7_10int_traits4HInt9widen_mul:
	.cfi_startproc
	umulh	x9, x2, x0
	umulh	x10, x3, x0
	mul	x11, x3, x0
	umulh	x12, x2, x1
	umulh	x13, x3, x1
	mul	x14, x3, x1
	mul	x15, x2, x1
	adds	x9, x9, x15
	cinc	x12, x12, hs
	adds	x9, x9, x11
	cinc	x10, x10, hs
	adds	x11, x12, x14
	cinc	x12, x13, hs
	adds	x10, x11, x10
	cinc	x11, x12, hs
	stp	x10, x11, [x8]
	mul	x10, x2, x0
	stp	x10, x9, [x8, #16]
	ret
	.cfi_endproc

	.globl	__RNvXs2_NtNtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traits13narrowing_divNtNtB9_3big4u256NtB5_12NarrowingDiv27unchecked_narrowing_div_rem
	.p2align	2
__RNvXs2_NtNtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traits13narrowing_divNtNtB9_3big4u256NtB5_12NarrowingDiv27unchecked_narrowing_div_rem:
	.cfi_startproc
	stp	x26, x25, [sp, #-80]!
	.cfi_def_cfa_offset 80
	stp	x24, x23, [sp, #16]
	stp	x22, x21, [sp, #32]
	stp	x20, x19, [sp, #48]
	stp	x29, x30, [sp, #64]
	add	x29, sp, #64
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	.cfi_offset w19, -24
	.cfi_offset w20, -32
	.cfi_offset w21, -40
	.cfi_offset w22, -48
	.cfi_offset w23, -56
	.cfi_offset w24, -64
	.cfi_offset w25, -72
	.cfi_offset w26, -80
	mov	x19, x8
	clz	x8, x5
	clz	x9, x4
	add	x9, x9, #64
	cmp	x5, #0
	csel	x24, x8, x9, ne
	lsl	x8, x2, x24
	eor	w9, w24, #0x7f
	lsr	x10, x3, x9
	lsl	x11, x3, #1
	lsl	x11, x11, x24
	tst	x9, #0x40
	csel	x12, xzr, x10, ne
	lsr	x9, x2, x9
	lsl	x13, x3, x24
	orr	x9, x11, x9
	csel	x9, x10, x9, ne
	tst	x24, #0x40
	csel	x25, xzr, x8, ne
	lsr	x10, x2, #1
	mvn	w11, w24
	lsr	x10, x10, x11
	orr	x10, x13, x10
	extr	x9, x12, x9, #1
	csel	x22, x8, x10, ne
	lsl	x8, x1, x24
	lsr	x10, x0, #1
	lsr	x10, x10, x11
	orr	x8, x8, x10
	lsl	x10, x0, x24
	csel	x8, x10, x8, ne
	csel	x10, xzr, x10, ne
	orr	x23, x9, x10
	orr	x1, x8, x12, lsr #1
	lsl	x8, x4, x24
	csel	x26, xzr, x8, ne
	lsl	x9, x5, x24
	lsr	x10, x4, #1
	lsr	x10, x10, x11
	orr	x9, x9, x10
	csel	x20, x8, x9, ne
	cmp	x20, x1
	b.ls	LBB113_3
	mov	x0, x23
	mov	x2, x20
	mov	x3, #0
	bl	___udivti3
	mov	x21, x0
	msub	x8, x0, x20, x23
	umulh	x9, x0, x26
	mul	x10, x0, x26
	subs	x22, x22, x10
	sbcs	x1, x8, x9
	b.hs	LBB113_5
	sub	x8, x21, #1
	b	LBB113_4
LBB113_3:
	sub	x1, x23, x26
	mov	x8, #-1
LBB113_4:
	adds	x9, x22, x26
	adcs	x10, x1, x20
	cset	w11, hs
	sub	x12, x8, #1
	adds	x13, x9, x26
	adc	x14, x10, x20
	tst	w11, #0x1
	csel	x21, x8, x12, ne
	csel	x22, x9, x13, ne
	csel	x1, x10, x14, ne
LBB113_5:
	cmp	x20, x1
	b.ls	LBB113_8
	mov	x0, x22
	mov	x2, x20
	mov	x3, #0
	bl	___udivti3
	msub	x8, x0, x20, x22
	umulh	x9, x0, x26
	mul	x10, x0, x26
	subs	x25, x25, x10
	sbcs	x8, x8, x9
	b.hs	LBB113_10
	sub	x9, x0, #1
	b	LBB113_9
LBB113_8:
	sub	x8, x22, x26
	mov	x9, #-1
LBB113_9:
	adds	x10, x25, x26
	adcs	x8, x8, x20
	cset	w11, hs
	sub	x12, x9, #1
	adds	x13, x10, x26
	adc	x14, x8, x20
	tst	w11, #0x1
	csel	x0, x9, x12, ne
	csel	x25, x10, x13, ne
	csel	x8, x8, x14, ne
LBB113_10:
	lsr	x9, x25, x24
	mvn	w10, w24
	lsl	x11, x8, #1
	lsl	x10, x11, x10
	orr	x9, x10, x9
	lsr	x8, x8, x24
	tst	x24, #0x40
	csel	x9, x8, x9, ne
	stp	x0, x21, [x19]
	csel	x8, xzr, x8, ne
	stp	x9, x8, [x19, #16]
	.cfi_def_cfa wsp, 80
	ldp	x29, x30, [sp, #64]
	ldp	x20, x19, [sp, #48]
	ldp	x22, x21, [sp, #32]
	ldp	x24, x23, [sp, #16]
	ldp	x26, x25, [sp], #80
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	.cfi_restore w23
	.cfi_restore w24
	.cfi_restore w25
	.cfi_restore w26
	ret
	.cfi_endproc

	.globl	__RNvXs3_NtNtNtCsdtCTLUYU3dx_4libm4math7support3bignNtNtB7_10int_traits4HInt9widen_mul
	.p2align	2
__RNvXs3_NtNtNtCsdtCTLUYU3dx_4libm4math7support3bignNtNtB7_10int_traits4HInt9widen_mul:
	.cfi_startproc
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
Lloh640:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.101@PAGE
Lloh641:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.101@PAGEOFF
Lloh642:
	adrp	x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.103@PAGE
Lloh643:
	add	x2, x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.103@PAGEOFF
	mov	w1, #117
	bl	__RNvNtCsgtPOCBgevO_4core9panicking9panic_fmt
	.loh AdrpAdd	Lloh642, Lloh643
	.loh AdrpAdd	Lloh640, Lloh641
	.cfi_endproc

	.globl	__RNvXs4_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitshNtB5_3Int12unsigned_abs
	.p2align	2
__RNvXs4_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitshNtB5_3Int12unsigned_abs:
	.cfi_startproc
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
Lloh644:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.98@PAGE
Lloh645:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.98@PAGEOFF
Lloh646:
	adrp	x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.104@PAGE
Lloh647:
	add	x2, x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.104@PAGEOFF
	mov	w1, #15
	bl	__RNvNtCsgtPOCBgevO_4core9panicking5panic
	.loh AdrpAdd	Lloh646, Lloh647
	.loh AdrpAdd	Lloh644, Lloh645
	.cfi_endproc

	.globl	__RNvXs4_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitshNtB5_3Int5ilog2
	.p2align	2
__RNvXs4_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitshNtB5_3Int5ilog2:
	.cfi_startproc
	tst	w0, #0xff
	b.eq	LBB116_2
	lsl	w8, w0, #24
	clz	w8, w8
	eor	w0, w8, #0x7
	ret
LBB116_2:
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
Lloh648:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.104@PAGE
Lloh649:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.104@PAGEOFF
	bl	__RNvNtNtNtCsgtPOCBgevO_4core3num3imp9int_log1030panic_for_nonpositive_argument
	.loh AdrpAdd	Lloh648, Lloh649
	.cfi_endproc

	.globl	__RNvXs6_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsaNtB5_3Int5ilog2
	.p2align	2
__RNvXs6_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsaNtB5_3Int5ilog2:
	.cfi_startproc
	sxtb	w8, w0
	cmp	w8, #1
	b.lt	LBB117_2
	lsl	w8, w0, #24
	clz	w8, w8
	eor	w0, w8, #0x7
	ret
LBB117_2:
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
Lloh650:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.104@PAGE
Lloh651:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.104@PAGEOFF
	bl	__RNvNtNtNtCsgtPOCBgevO_4core3num3imp9int_log1030panic_for_nonpositive_argument
	.loh AdrpAdd	Lloh650, Lloh651
	.cfi_endproc

	.globl	__RNvXs8_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitstNtB5_3Int12unsigned_abs
	.p2align	2
__RNvXs8_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitstNtB5_3Int12unsigned_abs:
	.cfi_startproc
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
Lloh652:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.98@PAGE
Lloh653:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.98@PAGEOFF
Lloh654:
	adrp	x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.105@PAGE
Lloh655:
	add	x2, x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.105@PAGEOFF
	mov	w1, #15
	bl	__RNvNtCsgtPOCBgevO_4core9panicking5panic
	.loh AdrpAdd	Lloh654, Lloh655
	.loh AdrpAdd	Lloh652, Lloh653
	.cfi_endproc

	.globl	__RNvXs8_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitstNtB5_3Int5ilog2
	.p2align	2
__RNvXs8_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitstNtB5_3Int5ilog2:
	.cfi_startproc
	tst	w0, #0xffff
	b.eq	LBB119_2
	lsl	w8, w0, #16
	clz	w8, w8
	eor	w0, w8, #0xf
	ret
LBB119_2:
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
Lloh656:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.105@PAGE
Lloh657:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.105@PAGEOFF
	bl	__RNvNtNtNtCsgtPOCBgevO_4core3num3imp9int_log1030panic_for_nonpositive_argument
	.loh AdrpAdd	Lloh656, Lloh657
	.cfi_endproc

	.globl	__RNvXsa_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitssNtB5_3Int5ilog2
	.p2align	2
__RNvXsa_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitssNtB5_3Int5ilog2:
	.cfi_startproc
	sxth	w8, w0
	cmp	w8, #1
	b.lt	LBB120_2
	lsl	w8, w0, #16
	clz	w8, w8
	eor	w0, w8, #0xf
	ret
LBB120_2:
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
Lloh658:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.105@PAGE
Lloh659:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.105@PAGEOFF
	bl	__RNvNtNtNtCsgtPOCBgevO_4core3num3imp9int_log1030panic_for_nonpositive_argument
	.loh AdrpAdd	Lloh658, Lloh659
	.cfi_endproc

	.globl	__RNvXsc_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsmNtB5_3Int12unsigned_abs
	.p2align	2
__RNvXsc_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsmNtB5_3Int12unsigned_abs:
	.cfi_startproc
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
Lloh660:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.98@PAGE
Lloh661:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.98@PAGEOFF
Lloh662:
	adrp	x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.106@PAGE
Lloh663:
	add	x2, x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.106@PAGEOFF
	mov	w1, #15
	bl	__RNvNtCsgtPOCBgevO_4core9panicking5panic
	.loh AdrpAdd	Lloh662, Lloh663
	.loh AdrpAdd	Lloh660, Lloh661
	.cfi_endproc

	.globl	__RNvXsc_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsmNtB5_3Int5ilog2
	.p2align	2
__RNvXsc_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsmNtB5_3Int5ilog2:
	.cfi_startproc
	cbz	w0, LBB122_2
	clz	w8, w0
	eor	w0, w8, #0x1f
	ret
LBB122_2:
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
Lloh664:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.106@PAGE
Lloh665:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.106@PAGEOFF
	bl	__RNvNtNtNtCsgtPOCBgevO_4core3num3imp9int_log1030panic_for_nonpositive_argument
	.loh AdrpAdd	Lloh664, Lloh665
	.cfi_endproc

	.globl	__RNvXse_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitslNtB5_3Int5ilog2
	.p2align	2
__RNvXse_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitslNtB5_3Int5ilog2:
	.cfi_startproc
	cmp	w0, #1
	b.lt	LBB123_2
	clz	w8, w0
	eor	w0, w8, #0x1f
	ret
LBB123_2:
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
Lloh666:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.106@PAGE
Lloh667:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.106@PAGEOFF
	bl	__RNvNtNtNtCsgtPOCBgevO_4core3num3imp9int_log1030panic_for_nonpositive_argument
	.loh AdrpAdd	Lloh666, Lloh667
	.cfi_endproc

	.globl	__RNvXsg_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsyNtB5_3Int12unsigned_abs
	.p2align	2
__RNvXsg_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsyNtB5_3Int12unsigned_abs:
	.cfi_startproc
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
Lloh668:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.98@PAGE
Lloh669:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.98@PAGEOFF
Lloh670:
	adrp	x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.107@PAGE
Lloh671:
	add	x2, x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.107@PAGEOFF
	mov	w1, #15
	bl	__RNvNtCsgtPOCBgevO_4core9panicking5panic
	.loh AdrpAdd	Lloh670, Lloh671
	.loh AdrpAdd	Lloh668, Lloh669
	.cfi_endproc

	.globl	__RNvXsg_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsyNtB5_3Int5ilog2
	.p2align	2
__RNvXsg_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsyNtB5_3Int5ilog2:
	.cfi_startproc
	cbz	x0, LBB125_2
	clz	x8, x0
	eor	w0, w8, #0x3f
	ret
LBB125_2:
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
Lloh672:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.107@PAGE
Lloh673:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.107@PAGEOFF
	bl	__RNvNtNtNtCsgtPOCBgevO_4core3num3imp9int_log1030panic_for_nonpositive_argument
	.loh AdrpAdd	Lloh672, Lloh673
	.cfi_endproc

	.globl	__RNvXsi_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsxNtB5_3Int5ilog2
	.p2align	2
__RNvXsi_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsxNtB5_3Int5ilog2:
	.cfi_startproc
	cmp	x0, #1
	b.lt	LBB126_2
	clz	x8, x0
	eor	w0, w8, #0x3f
	ret
LBB126_2:
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
Lloh674:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.107@PAGE
Lloh675:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.107@PAGEOFF
	bl	__RNvNtNtNtCsgtPOCBgevO_4core3num3imp9int_log1030panic_for_nonpositive_argument
	.loh AdrpAdd	Lloh674, Lloh675
	.cfi_endproc

	.globl	__RNvXsk_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsoNtB5_3Int12unsigned_abs
	.p2align	2
__RNvXsk_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsoNtB5_3Int12unsigned_abs:
	.cfi_startproc
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
Lloh676:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.98@PAGE
Lloh677:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.98@PAGEOFF
Lloh678:
	adrp	x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.108@PAGE
Lloh679:
	add	x2, x2, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.108@PAGEOFF
	mov	w1, #15
	bl	__RNvNtCsgtPOCBgevO_4core9panicking5panic
	.loh AdrpAdd	Lloh678, Lloh679
	.loh AdrpAdd	Lloh676, Lloh677
	.cfi_endproc

	.globl	__RNvXsk_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsoNtB5_3Int5ilog2
	.p2align	2
__RNvXsk_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsoNtB5_3Int5ilog2:
	.cfi_startproc
	orr	x8, x0, x1
	cbz	x8, LBB128_2
	clz	x8, x0
	add	w8, w8, #64
	clz	x9, x1
	cmp	x1, #0
	csel	w8, w9, w8, ne
	eor	w0, w8, #0x7f
	ret
LBB128_2:
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
Lloh680:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.108@PAGE
Lloh681:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.108@PAGEOFF
	bl	__RNvNtNtNtCsgtPOCBgevO_4core3num3imp9int_log1030panic_for_nonpositive_argument
	.loh AdrpAdd	Lloh680, Lloh681
	.cfi_endproc

	.globl	__RNvXsm_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsnNtB5_3Int5ilog2
	.p2align	2
__RNvXsm_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsnNtB5_3Int5ilog2:
	.cfi_startproc
	cmp	x0, #1
	sbcs	xzr, x1, xzr
	b.lt	LBB129_2
	clz	x8, x0
	add	w8, w8, #64
	clz	x9, x1
	cmp	x1, #0
	csel	w8, w9, w8, ne
	eor	w0, w8, #0x7f
	ret
LBB129_2:
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
Lloh682:
	adrp	x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.108@PAGE
Lloh683:
	add	x0, x0, l_anon.55ed7662f03e4ae27c90b6ed7e696e35.108@PAGEOFF
	bl	__RNvNtNtNtCsgtPOCBgevO_4core3num3imp9int_log1030panic_for_nonpositive_argument
	.loh AdrpAdd	Lloh682, Lloh683
	.cfi_endproc

	.section	__TEXT,__cstring,cstring_literals
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.0:
	.asciz	"/Users/idlewile/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libm-0.2.16/src/math/generic/fmod.rs"

	.section	__DATA,__const
	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.1:
	.quad	l_anon.55ed7662f03e4ae27c90b6ed7e696e35.0
	.asciz	"i\000\000\000\000\000\000\000Y\000\000\000\t\000\000"

	.section	__TEXT,__cstring,cstring_literals
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.2:
	.asciz	"/Users/idlewile/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libm-0.2.16/src/math/support/modular.rs"

	.section	__TEXT,__const
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.3:
	.ascii	"assertion failed: y <= U::MAX >> 2"

	.section	__DATA,__const
	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.4:
	.quad	l_anon.55ed7662f03e4ae27c90b6ed7e696e35.2
	.asciz	"l\000\000\000\000\000\000\000\026\000\000\000\005\000\000"

	.section	__TEXT,__const
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.5:
	.ascii	"assertion failed: x < (y << 1)"

	.section	__DATA,__const
	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.6:
	.quad	l_anon.55ed7662f03e4ae27c90b6ed7e696e35.2
	.asciz	"l\000\000\000\000\000\000\000\027\000\000\000\005\000\000"

	.section	__TEXT,__const
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.7:
	.ascii	"assertion failed: n > (_1 << (U::BITS - 3))"

	.section	__DATA,__const
	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.8:
	.quad	l_anon.55ed7662f03e4ae27c90b6ed7e696e35.2
	.asciz	"l\000\000\000\000\000\000\000b\000\000\000\t\000\000"

	.section	__TEXT,__const
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.9:
	.ascii	"assertion failed: n < (_1 << (U::BITS - 2))"

	.section	__DATA,__const
	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.10:
	.quad	l_anon.55ed7662f03e4ae27c90b6ed7e696e35.2
	.asciz	"l\000\000\000\000\000\000\000c\000\000\000\t\000\000"

	.section	__TEXT,__const
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.11:
	.ascii	"assertion failed: x < m"

	.section	__DATA,__const
	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.12:
	.quad	l_anon.55ed7662f03e4ae27c90b6ed7e696e35.2
	.asciz	"l\000\000\000\000\000\000\000e\000\000\000\t\000\000"

	.section	__TEXT,__const
	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.13:
	.asciz	"\003\000\000\000\000\000\000\000\004\000\000\000\000\000\000\000\004\000\000\000\000\000\000\000\006\000\000\000\000\000\000"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.14:
	.asciz	"\203\371\242\000DNn\000\374)\025\000\321W'\000\3354\365\000b\333\300\000<\231\225\000A\220C\000cQ\376\000\273\336\253\000\267a\305\000:n$\000\322MB\000I\006\340\000\t\352.\000\034\222\321\000\353\035\376\000)\261\034\000\350>\247\000\3655\202\000D\273.\000\234\351\204\000\264&p\000A~_\000\326\2219\000S\2039\000\234\3649\000\213_\204\000(\371\275\000\370\037;\000\336\377\227\000\017\230\005\000\021/\357\000\nZ\213\000m\037m\000\317~6\000\t\313'\000FO\267\000\236f?\000-\352_\000\272'u\000\345\353\307\000={\361\000\3679\007\000\222R\212\000\373k\352\000\037\261_\000\b]\215\0000\003V\000{\374F\000\360\253k\000 \274\317\0006\364\232\000\343\251\035\000^a\221\000\b\033\346\000\205\231e\000\240\024_\000\215@h\000\200\330\377\000'sM\000\006\0061\000\312V\025\000\311\250s\000{\342`\000k\214\300\000\031\304G\000\315g\303\000\t\350\334\000Y\203*\000\213v\304\000\246\034\226\000D\257\335\000\031W\321\000\245>\005\000\005\007\377\0003~?\000\3022\350\000\230O\336\000\273}2\000&=\303\000\036k\357\000\237\370^\0005\037:\000\177\362\312\000\361\207\035\000|\220!\000j$|\000\325n\372\0000-w\000\025;C\000\265\024\306\000\303\031\235\000\255\304\302\000,MA\000\f\000]\000\206}F\000\343q-\000\233\306\232\0003b\000\000\264\322|\000\264\247\227\0007U\325\000\327>\366\000\243\020\030\000Mv\374\000d\235*\000p\327\253\000c|\370\000z\260W\000\027\025\347\000\300IV\000;\326\331\000\247\2048\000$#\313\000\326\212w\000ZT#\000\000\037\271\000\361\n\033\000\031\316\337\000\2371\377\000f\036j\000\231Wa\000\254\373G\000~\177\330\000\"e\267\0002\350\211\000\346\277`\000\357\304\315\000l6\t\000]?\324\000\026\336\327\000X;\336\000\336\233\222\000\322\"(\000(\206\350\000\342XM\000\306\3122\000\b\343\026\000\340}\313\000\027\300P\000\363\035\247\000\030\340[\000.\0234\000\203\022b\000\203H\001\000\365\216[\000\255\260\177\000\036\351\362\000HJC\000\020g\323\000\252\335\330\000\256_B\000ja\316\000\n(\244\000\323\231\264\000\006\246\362\000\\w\177\000\243\302\203\000a<\210\000\212sx\000\257\214Z\000o\327\275\000-\246c\000\364\277\313\000\215\201\357\000&\301g\000U\312E\000\312\3316\000(\250\322\000\302a\215\000\022\311w\000\004&\024\000\022F\233\000\304Y\304\000\310\305D\000M\262\221\000\000\027\363\000\324C\255\000)I\345\000\375\325\020\000\000\276\374\000\036\224\314\000p\316\356\000\023>\365\000\354\361\200\000\263\347\303\000\307\370(\000\223\005\224\000\301q>\000.\t\263\000\013E\363\000\210\022\234\000\253 {\000.\265\237\000G\222\302\000{2/\000\fUm\000r\247\220\000k\347\037\0001\313\226\000y\026J\000Ay\342\000\364\337\211\000\350\224\227\000\342\346\204\000\2311\227\000\210\355k\000__6\000\273\375\016\000H\232\264\000g\244l\000qrB\000\215]2\000\237\025\270\000\274\345\t\000\2151%\000\367t9\0000\005\034\000\r\f\001\000K\bh\000,\356X\000G\252\220\000t\347\002\000\275\326$\000\367}\246\000nHr\000\237\026\357\000\216\224\246\000\264\221\366\000\321SQ\000\317\n\362\000 \2303\000\365K~\000\262ch\000\335>_\000@]\003\000\205\211\177\000UR)\0007d\300\000m\330\020\0002H2\000[Lu\000Nq\324\000ETn\000\013\t\301\000*\365i\000\024f\325\000'\007\235\000]\004P\000\264;\333\000\352v\305\000\207\371\027\000Ik}\000\035'\272\000\226i)\000\306\314\254\000\255\024T\000\220\342j\000\210\331\211\000,rP\000\004\244\276\000w\007\224\000\3630p\000\000\374'\000\352q\250\000f\302I\000d\340=\000\227\335\203\000\243?\227\000C\224\375\000\r\206\214\0001A\336\000\2229\235\000\335p\214\000\027\267\347\000\b\337;\000\0257+\000\\\200\240\000Z\200\223\000\020\021\222\000\017\350\330\000l\200\257\000\333\377K\0008\220\017\000Y\030v\000b\245\025\000a\313\273\000\307\211\271\000\020@\275\000\322\362\004\000Iu'\000\353\266\366\000\333\"\273\000\n\024\252\000\211&/\000d\203v\000\t;3\000\016\224\032\000Q:\252\000\035\243\302\000\257\355\256\000\\&\022\000m\302M\000-z\234\000\300V\227\000\003?\203\000\t\360\366\000+@\214\000m1\231\0009\264\007\000\f \025\000\330\303[\000\365\222\304\000\306\255K\000N\312\245\000\2477\315\000\346\2516\000\253\222\224\000\335Bh\000\031c\336\000v\214\357\000h\213R\000\374\3337\000\256\241\253\000\337\0251\000\000\256\241\000\f\373\332\000dMf\000\355\005\267\000)e0\000WV\277\000G\377:\000j\371\271\000u\276\363\000(\223\337\000\253\2000\000f\214\366\000\004\313\025\000\372\"\006\000\331\344\035\000=\263\244\000W\033\217\0006\315\t\000NB\351\000\023\276\244\0003#\265\000\360\252\032\000Oe\250\000\322\301\245\000\013?\017\000[x\315\000#\371v\000{\213\004\000\211\027r\000\306\246S\000on\342\000\357\353\000\000\233JX\000\304\332\267\000\252f\272\000v\317\317\000\321\002\035\000\261\361-\000\214\231\301\000\303\255w\000\206H\332\000\367]\240\000\306\200\364\000\254\360/\000\335\354\232\000?\\\274\000\320\336m\000\220\307\037\000*\333\266\000\243%:\000\000\257\232\000\255S\223\000\266W\004\000)-\264\000K\200~\000\332\007\247\000v\252\016\000{Y\241\000\026\022*\000\334\267-\000\372\345\375\000\211\333\376\000\211\276\375\000\344vl\000\006\251\374\000>\200p\000\205n\025\000\375\207\377\000(>\007\000ag3\000*\030\206\000M\275\352\000\263\347\257\000\217mn\000\225g9\0001\277[\000\204\327H\0000\337\026\000\307-C\000%a5\000\311p\316\0000\313\270\000\277l\375\000\244\000\242\000\005l\344\000Z\335\240\000!oG\000b\022\322\000\271\\\204\000paI\000kV\340\000\231R\001\000PU7\000\036\325\267\0003\361\304\000\023n_\000]0\344\000\205.\251\000\035\262\303\000\24126\000\b\267\244\000\352\261\324\000\026\367!\000\217i\344\000'\377w\000\f\003\200\000\215@-\000O\315\240\000 \245\231\000\263\242\323\000/]\n\000\264\371B\000\021\332\313\000}\276\320\000\233\333\301\000\253\027\275\000\312\242\201\000\bj\\\000.U\027\000'\000U\000\177\024\360\000\341\007\206\000\024\013d\000\226A\215\000\207\276\336\000\332\375*\000k%\266\000{\2114\000\005\363\376\000\271\277\236\000hjO\000J*\250\000O\304Z\000-\370\274\000\327Z\230\000\364\307\225\000\rM\215\000 :\246\000\244W_\000\024?\261\000\2008\225\000\314 \001\000q\335\206\000\311\336\266\000\277`\365\000Me\021\000\001\007k\000\214\260\254\000\262\300\320\000QUH\000\036\373\016\000\225r\303\000\243\006;\000\300@5\000\006\334{\000\340E\314\000N)\372\000\326\312\310\000\350\363A\000|d\336\000\233d\330\000\331\2761\000\244\227\303\000wX\324\000i\343\305\000\360\332\023\000\272:<\000F\030F\000Uu_\000\322\275\365\000n\222\306\000\254.]\000\016D\355\000\034>B\000a\304\207\000)\375\351\000\347\326\363\000\"|\312\000o\2215\000\b\340\305\000\377\327\215\000nj\342\000\260\375\306\000\223\b\301\000|]t\000k\255\262\000\315n\235\000>r{\000\306\021j\000\367\317\251\000)s\337\000\265\311\272\000\267\000Q\000\342\262\r\000t\272$\000\345}`\000t\330\212\000\r\025,\000\201\030\f\000~f\224\000\001)\026\000\237zv\000\375\375\276\000VE\357\000\331~6\000\354\331\023\000\213\272\271\000\304\227\374\0001\250'\000\361n\303\000\224\3056\000\330\250V\000\264\250\265\000\317\314\016\000\022\211-\000oW4\000,V\211\000\231\316\343\000\326 \271\000k^\252\000>*\234\000\021_\314\000\375\013J\000\341\364\373\000\216;m\000\342\206,\000\351\324\204\000\374\264\251\000\357\356\321\000.5\311\000/9a\0008!D\000\033\331\310\000\201\374\n\000\373Jj\000/\034\330\000S\264\204\000N\231\214\000T\"\314\000*U\334\000\300\306\326\000\013\031\226\000\032p\270\000i\225d\000&Z`\000?R\356\000\177\021\017\000\364\265\021\000\374\313\365\0004\274-\0004\274\356\000\350]\314\000\335^`\000g\216\233\000\2223\357\000\311\027\270\000aX\233\000\341W\274\000Q\203\306\000\330>\020\000\335qH\000-\034\335\000\257\030\241\000!,F\000Y\363\327\000\331z\230\000\236T\300\000O\206\372\000V\006\374\000\345y\256\000\211\"6\0008\255\"\000g\223\334\000U\350\252\000\202&8\000\312\347\233\000Q\r\244\000\2313\261\000\251\327\016\000i\005H\000e\262\360\000\177\210\247\000\210L\227\000\371\3216\000!\222\263\000{\202J\000\230\317!\000@\237\334\000\334GU\000\341t:\000g\353B\000\376\235\337\000^\324_\000{g\244\000\272\254z\000U\366\242\000+\210#\000A\272U\000Yn\b\000!*\206\0009G\203\000\211\343\346\000\345\236\324\000I\373@\000\377V\351\000\034\017\312\000\305Y\212\000\224\372+\000\323\301\305\000\017\305\317\000\333Z\256\000G\305\206\000\205Cb\000!\206;\000,y\224\000\020a\207\000*L{\000\200,\032\000C\277\022\000\210&\220\000x<\211\000\250\304\344\000\345\333{\000\304:\302\000&\364\352\000\367g\212\000\r\222\277\000e\243+\000=\223\261\000\275|\013\000\244Q\334\000'\335c\000i\341\335\000\232\224\031\000\250)\225\000h\316(\000\t\355\264\000D\237 \000N\230\312\000p\202c\000~|#\000\017\2712\000\247\365\216\000\024V\347\000!\361\b\000\265\235*\000o~M\000\245\031Q\000\265\371\253\000\202\337\326\000\226\335a\000\0266\002\000\304:\237\000\203\242\241\000r\355m\0009\215z\000\202\270\251\000k2\\\000F'[\000\0004\355\000\322\000w\000\374\364U\000\001YM\000\340q\200"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.15:
	.ascii	"\000\000\000@\373!\371?\000\000\000\000-Dt>\000\000\000\200\230F\370<\000\000\000`Q\314x;\000\000\000\200\203\033\3609\000\000\000@ %z8\000\000\000\200\"\202\3436\000\000\000\000\035\363i5"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.16:
	.ascii	"m\002'\351\026\323w\276B\036^Ib\377\261\277C\250$\21296\367\277\363\247\257\355\363\212\036\300\003c$\305\346b&\300\017\347\217\257\201\336\t\300"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.17:
	.ascii	"YY\213\220e86@\217\207\350\016\236\006a@\233\007\352B\206\347p@\377\372\266:\003<c@\t\0309D\263P-@"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.18:
	.ascii	"\206\252\341o\003\341%\276K\342\300\367\366\377\261\277t\200\244\256\262E\003\300\024N\271LJ\3675\300E\032\nB\"\nM\300\017\330\222\250\254r?\300"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.19:
	.ascii	"\323}\007\204\222\355A@\016|JF9\230v@\326a\020mn\246\222@~\233\303\270\374\237\221@\201\2207\374\226\262e@"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.20:
	.ascii	"\314\225\344G\261\030\251\275\306\373\232\346\377\377\261\277\277k\f\371p\243\020\300\203\027}Z/\353P\300c\314Bg\263\263t\300\327\213\243(\357\246u\300"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.21:
	.ascii	"\336\305\230\f\201`N@d(~\\\002m\220@`\035\276\217\370Z\267@8\372v\372\270\314\302@d\350\013\307\035\314\242@"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.22:
	.ascii	"\000\000\000\000\000\000\000\0002\375\377\377\377\377\261\277y\247O\264\320) \300c\350\031{\002\021p\300\374\312M\315nj\243\300=d\3146\013\205\264\300"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.23:
	.ascii	"Qg\251\0073\"]@8iYP}\363\255@_\260\266n\273\322\343@\275\251\237\217\017\201\374@\334I,OwA\347@"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.24:
	.ascii	"\333k\367T;1\204>4>\210>\305\276\262?\234w'\347\227\370\377?\345o\371\252\277\375,@J\334\373)\216\252?@\264K\201q\261@0@"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.25:
	.ascii	"\355z\300\367\226]>@@K\321\344\221\325p@\"\277\263\"Ef\212@\024\302\\\234|\227\213@e\023\000\016S\225j@1)\263\370j>\025\300"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.26:
	.ascii	"\202\313\336j\003\3152>B\b\215\016\356\277\262?\365\234\024a\374\300\n@\335\256-\226\230OE@\037\375^\342\333Ye@\340!\372\201|\327d@"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.27:
	.ascii	"\246C\343\277\"aH@\263NT\206\203-\206@c\374M\344K\360\254@(j\307\327l<\271@\300\261O\331\252\250\243@\017\364\034 \353\247b\300"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.28:
	.ascii	"\331\214\314)\217=\264=L\260r\321\377\277\262?\323=\225\271\260W\027@\351\210\207\n\222\343`@\201\304\310\235\371\f\220@\246\343\306S\351\027\237@"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.29:
	.ascii	"C\025^\373\263\261T@\316\300!\332\240;\240@m\036Y{\322g\322@r#\340\227\343\265\353@\240Tz\037\030\221\341@\t\306\333\276W\352\264\300"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.30:
	.ascii	"\000\000\000\000\000\000\000\000,\376\377\377\377\277\262?\3264\263[R\211'@%\0300\025cm\201@m\364\030>\231M\301@f\025\220\016\324\022\342@"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.31:
	.ascii	"\2749[6\325xd@c\005kNX\242\277@?\214\323TRf\001AC+\245\203\332\203(A=\013\336(k\246)A\3050\225,m\375\024\301"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.32:
	.ascii	"\364DU\366\324\351|>\203\rv\276B\377\275?\300\256\217\371\267\362\002@d\251q\1777|(@\342\216\177\027\250\2611@\376\301t\245IK\024@"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.33:
	.ascii	"\334\354\325\212\275o5@\325,\371\024\223R_@\331\333\242\325\330\bm@\251\204\030\332zk]@\222QN\364\261\272 @"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.34:
	.ascii	"\335\236\255\247!\374)>{\321![\365\377\275?\212\255^\350\274v\017@)\321\246\235H\217A@7\030,M\205\303V@\345>\250\216\217GH@"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.35:
	.ascii	"\234\0064\241IeA@_\247\361\0073\fu@#\3257P|[\220@\3511.\243}\326\213@S\355.|m\362Y@"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.36:
	.ascii	"}\312\341\332g\006\255=C\000\301\342\377\377\275?\343\025cn\0046\033@\355\002&E\271\023[@I\326R\320\026-\200@\267\f~\273\270\205\200@"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.37:
	.ascii	"=c\257\250\352\243M@\001g\006\0336\373\216@\373\266\006WD\351\264@\025\273\245\270\260\244\276@Q^o\0030\200\227@"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.38:
	.ascii	"\000\000\000\000\000\000\000\000\316\374\377\377\377\377\275?\316\177\1775\235z*@\220\245.e\324\300y@\3142\245\243}E\256@\335\202'\303z\352\276@"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.39:
	.ascii	"\254le\216E\215\\@O'M\226\334\205\254@\177\273\305\227\206\013\342@\273\027\217\262,\324\367@-\013zi\021\025\336@"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.40:
	.ascii	"\322&\306D&\361\207\276\020\260H\221\216>\272\277\332N\273i\204\004\006\300\177\220h\301\342\2513\300\252J\020\336\243)E\300Rn\31796_5\300"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.41:
	.ascii	"\377d\256x\212\210=@\272\034\202\333h\237o@\367\240I\316\005\254\207@)\300\324H%\033\207@\324\330><^~c@k\350\033\347\206\326\023\300"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.42:
	.ascii	"O\310\217\323\251\3175\276T\355\256Q\353?\272\277\377\331\0023\302p\022\300\332\026]\302q\354L\300_\325\030G\323\207l\300\366\033\\_\271fk\300"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.43:
	.ascii	"\344g\323\314#\325G@>\3561\300\353\016\205@\232|\216DNh\252@\246T\035\246\272\253\265@K\337\324\rz\274\235@\0371\n)p\346`\300"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.44:
	.ascii	"\230\240\241\032C\372\266\275\357\177Y\313\377?\272\277K\255\003\312\346\034 \300\260\271\247lm\365f\300Os1i\306t\225\300\235\247\375\210\343h\244\300"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.45:
	.ascii	"\262\021Z\377\262QT@9\370{\3471\037\237@)\316d\r\037\017\321@\227\321\272\252mW\350@K6|\317\004K\333@\004\240\377\374.o\262\300"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.46:
	.ascii	"\000\000\000\000\000\000\000\000\363\375\377\377\377?\272\277\367yg\242\221E0\300v\265\344S\320\274\207\300\025t\370@\347$\307\300j\234\320e\320\246\347\300"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.47:
	.ascii	"\345\315[\336\246,d@\031\204\330\320b\221\276@\230^\267\260\232W\000A\031\234\206rS\366%A\\\255\031w\322W$A\030\252\245\016i\371\021\301"

	.section	__TEXT,__literal16,16byte_literals
	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.48:
	.ascii	"\000\000\000\000\000\000\340?\000\000\000\000\000\000\340\277"

	.section	__TEXT,__const
	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.49:
	.ascii	"\267\230\276\263\022\373\217\275\314\261\271\277\237W\364\3006\0273\301\r\364N\300"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.50:
	.ascii	"-\303\261A\3604\bC2<\207C\032\340\031C\232\205jA"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.51:
	.ascii	"\033\b/\261\270\377\217\275\225-\032\300R\272\257\301\022Qh\302e\225\373\301"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.52:
	.ascii	"\224l\017B\312\301\264Cs3\225D\346\377\214D\270\224-C"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.53:
	.ascii	"\212\305H\255\377\377\217\275\210\033\205\300{Y\207\302\233\235\245\303y7\255\303"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.54:
	.ascii	"\b\004sB\023h\203D\304\327\272E\310e\026F\356`\026E"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.55:
	.ascii	"\000\000\000\000\000\000\220\275\206N\001\301\024\210\200\303vS\033\305Z(\244\305"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.56:
	.ascii	"\230\021\351B\353\233oE\333\225\036G|\b\344G\272\013:G"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.57:
	.ascii	"\333\211!4*\366\225=\277\304\377?\375\355gAqT\375A\214\005\202A"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.58:
	.ascii	"\270\354\362A\217\254\206C)2SD\345\273\\D\230\252TCX\363\251\300"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.59:
	.ascii	"\033h\2261p\377\225=\343\007V@\305|*B\337\316*C\344\273&C"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.60:
	.ascii	"\026\tCB\034l1D_\202gEg\343\311EWE\035EY?\025\303"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.61:
	.ascii	"y\354\241-\377\377\225=\206\275\272@\220\034\007C\315g\200DK\277\370D"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.62:
	.ascii	"\240\215\245B\007\335\001E\224>\223F\035\257]G\301\210\fG\276R\247\305"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.63:
	.ascii	"\000\000\000\000\000\000\226=\223J<A\031k\013D\312l\nF\240\226\020G"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.64:
	.ascii	"\252\306#C\302\022\375E\2232\013H\324\036DIY3MIi\353\247\310"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.65:
	.ascii	"\250N\3473\026\372\357=\300\225\027@\274\341CAA\215\215AMZ\242@"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.66:
	.ascii	"\354}\253A\231\224\372B\307FhC\327[\353B\220\325\005A"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.67:
	.ascii	"\r\341O1\253\377\357=\347\265{@Ez\fB*\034\266B|<BB"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.68:
	.ascii	"M*\013B\230a\250C\343\333\202D\355\263^Dl\223\317B"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.69:
	.ascii	"?3h-\377\377\357=#\260\331@\312\235\330B\267h\001D\306-\004D"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.70:
	.ascii	"U\037mB\261\331wD#J\247E\206%\365E\200\001\274D"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.71:
	.ascii	"\000\000\000\000\000\000\360=\352\324SA\243\006\316C\355+rE\326S\367E"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.72:
	.ascii	",j\344B\345.dE5\\\020Gf\241\276G\213\250\360F"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.73:
	.ascii	"2\211?\264u\364\321\275#$0\300\026O\235\301\037M)\302\262\371\252\301"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.74:
	.ascii	"TD\354AG\373|C.`=D*\3318D\362\362\033C7\264\236\300"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.75:
	.ascii	"O}\256\261[\377\321\275\022\206\223\300\216cg\302\232>d\303\3135[\303"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.76:
	.ascii	"\036\251>B^w(DrBSE\325]\255E\320\343\355D\2013\007\303"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.77:
	.ascii	"\031\322\267\255\376\377\321\2756\347\000\301k\2537\3033\246\253\304\034G#\305"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.78:
	.ascii	"\230\215\242B\217\371\370D\370x\210Fm\273BG&X\332Fxy\223\305"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.79:
	.ascii	"\000\000\000\000\000\000\322\275\215,\202\301\203\346=\304:'9\306\2036=\307"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.80:
	.ascii	"7e!C\027\213\364E\326\274\002H\234\262/I\224\276\"IH\313\217\310"

	.section	__TEXT,__literal16,16byte_literals
	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.81:
	.ascii	"\000\000\000\000\000\000\360?\000\000\000\000\000\000\370?"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.82:
	.ascii	"\000\000\000\000\000\000\000\000\006\320\317C\353\375L>"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.83:
	.ascii	"\000\000\000\000\000\000\000\000\000\000\000@\003\270\342?"

	.section	__TEXT,__const
	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.84:
	.ascii	"O\273a\005g\254\335?\030-DT\373!\351?\233\366\201\322\013s\357?\030-DT\373!\371?"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.85:
	.ascii	"\342e/\"\177+z<\007\\\0243&\246\201<\275\313\360z\210\007p<\007\\\0243&\246\221<"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.86:
	.ascii	"\000\000\000\000\000\000\360?\213r\215\371\242(\364?=n=\245\376e\371?"

	.p2align	3, 0x0
__RNvNtNtCsdtCTLUYU3dx_4libm4math4exp23TBL:
	.ascii	"]=\177f\236\240\346?\000\000\000\000\000\2109=D\027u\372R\260\346?\000\000\000\000\000\000\330<\376\331\013u\022\300\346?\000\000\000\000\000x(\275\277v\324\335\334\317\346?\000\000\000\000\000\300\036=)\032e<\262\337\346?\000\000\000\000\000\000\330\274\343:Y\230\222\357\346?\000\000\000\000\000\000\274\274\206\223Q\371}\377\346?\000\000\000\000\000\330/\275\243-\364ft\017\347?\000\000\000\000\000\210,\275\303_\354\350u\037\347?\000\000\000\000\000\300\023=\005\317\352\206\202/\347?\000\000\000\000\00008\275R\201\245H\232?\347?\000\000\000\000\000\300\000\275\374\314\3275\275O\347?\000\000\000\000\000\210/=\361gBV\353_\347?\000\000\000\000\000\340\003=Hm\253\261$p\347?\000\000\000\000\000\320'\2758]\336Oi\200\347?\000\000\000\000\000\000\335\274\000\035\2548\271\220\347?\000\000\000\000\000\000\343<x\001\353s\024\241\347?\000\000\000\000\000\000\355\274`\320v\t{\261\347?\000\000\000\000\000@ =3\3010\001\355\301\347?\000\000\000\000\000\000\240<6\206\377bj\322\347?\000\000\000\000\000\220&\275;N\3176\363\342\347?\000\000\000\000\000\340\002\275\350\303\221\204\207\363\347?\000\000\000\000\000X$\275N\033>T'\004\350?\000\000\000\000\000\0003=\032\007\321\255\322\024\350?\000\000\000\000\000\000\017=~\315L\231\211%\350?\000\000\000\000\000\300!\275\320B\271\036L6\350?\000\000\000\000\000\320)=\265\312#F\032G\350?\000\000\000\000\000\020G=\274[\237\027\364W\350?\000\000\000\000\000`\"=\257\221D\233\331h\350?\000\000\000\000\000\3042\275\225\2431\331\312y\350?\000\000\000\000\000\000#\275\270e\212\331\307\212\350?\000\000\000\000\000\200*\275\000Xx\244\320\233\350?\000\000\000\000\000\000\355\274#\242*B\345\254\350?\000\000\000\000\000(3=\372\031\326\272\005\276\350?\000\000\000\000\000\264B=\203C\265\0262\317\350?\000\000\000\000\000\320.\275Lf\b^j\340\350?\000\000\000\000\000P \275\007x\025\231\256\361\350?\000\000\000\000\000((=\016,(\320\376\002\351?\000\000\000\000\000\260\034\275\226\377\221\013[\024\351?\000\000\000\000\000\340\005\275\371/\252S\303%\351?\000\000\000\000\000@\365<J\306\315\26077\351?\000\000\000\000\000 \027=\256\230_+\270H\351?\000\000\000\000\000\000\t\275\313R\310\313DZ\351?\000\000\000\000\000h%=!ov\232\335k\351?\000\000\000\000\000\3206\275*N\336\237\202}\351?\000\000\000\000\000\000\001\275\243#z\3443\217\351?\000\000\000\000\000\000-=\004\006\312p\361\240\351?\000\000\000\000\000\2448\275\211\377SM\273\262\351?\000\000\000\000\000\\5=[\361\243\202\221\304\351?\000\000\000\000\000\270&=\305\270K\031t\326\351?\000\000\000\000\000\000\354\274\216#\343\031c\350\351?\000\000\000\000\000\320\027=\002\363\007\215^\372\351?\000\000\000\000\000@\026=M\345]{f\f\352?\000\000\000\000\000\000\365\274\366\270\216\355z\036\352?\000\000\000\000\000\340\t='.J\354\2330\352?\000\000\000\000\000\330*=]\nF\200\311B\352?\000\000\000\000\000\360\032\275\233%>\262\003U\352?\000\000\000\000\000`\013=\023b\364\212Jg\352?\000\000\000\000\000\2108=\247\2630\023\236y\352?\000\000\000\000\000 \021=\215.\301S\376\213\352?\000\000\000\000\000\300\006=\322\374yUk\236\352?\000\000\000\000\000\270)\275\270o5!\345\260\352?\000\000\000\000\000p+=\201\363\323\277k\303\352?\000\000\000\000\000\000\331<\200'<:\377\325\352?\000\000\000\000\000\000\344<\243\322Z\231\237\350\352?\000\000\000\000\000\220,\275g\363\"\346L\373\352?\000\000\000\000\000P\026=\220\267\215)\007\016\353?\000\000\000\000\000\324/=\251\211\232l\316 \353?\000\000\000\000\000p\022=K\032O\270\2423\353?\000\000\000\000\000GM=\347G\267\025\204F\353?\000\000\000\000\00088\275:Y\345\215rY\353?\000\000\000\000\000\000\230<j\305\361)nl\353?\000\000\000\000\000\320\n=P^\373\362v\177\353?\000\000\000\000\000\200\336<\262I'\362\214\222\353?\000\000\000\000\000\300\004\275\003\006\2410\260\245\353?\000\000\000\000\000p\r\275fo\232\267\340\270\353?\000\000\000\000\000\220\r=\377\301K\220\036\314\353?\000\000\000\000\000\240\002=o\241\363\303i\337\353?\000\000\000\000\000x\037\275\270\035\327[\302\362\353?\000\000\000\000\000\240\020\275\351\262Aa(\006\354?\000\000\000\000\000@\021\275\340R\205\335\233\031\354?\000\000\000\000\000\340\013=\356d\372\331\034-\354?\000\000\000\000\000@\t\275/\320\377_\253@\354?\000\000\000\000\000\320\016\275\025\375\372xGT\354?\000\000\000\000\000f9=\313\320W.\361g\354?\000\000\000\000\000\020\032\275\266\301\210\211\250{\354?\000\000\000\000\200EX\2753\347\006\224m\217\354?\000\000\000\000\000H\032\275\337\304QW@\243\354?\000\000\000\000\000\000\313<\224\220\357\334 \267\354?\000\000\000\000\000@\001=\211\026m.\017\313\354?\000\000\000\000\000 \360<\022\304]U\013\337\354?\000\000\000\000\000`\363<;\253[[\025\363\354?\000\000\000\000\000\220\006\275\274\211\007J-\007\355?\000\000\000\000\000\240\t=\372\310\b+S\033\355?\000\000\000\000\000\340\025\275\205\212\r\b\207/\355?\000\000\000\000\000(\035=\003\242\312\352\310C\355?\000\000\000\000\000\240\001=\221\244\373\334\030X\355?\000\000\000\000\000\000\337<\241\346b\350vl\355?\000\000\000\000\000\240\003\275N\203\311\026\343\200\355?\000\000\000\000\000\330\f\275\220`\377q]\225\355?\000\000\000\000\000\300\364<\2562\333\003\346\251\355?\000\000\000\000\000\220\377<%\203:\326|\276\355?\000\000\000\000\000\200\351<E\264\001\363!\323\355?\000\000\000\000\000 \365\274\277\005\034d\325\347\355?\000\000\000\000\000p\035\275\354\232{3\227\374\355?\000\000\000\000\000\024\026\275^}\031kg\021\356?\000\000\000\000\000H\013=\347\243\365\024F&\356?\000\000\000\000\000\316@=\\\356\026;3;\356?\000\000\000\000\000h\f=\264?\213\347.P\356?\000\000\000\000\0000\t\275hmg$9e\356?\000\000\000\000\000\000\345\274DL\307\373Qz\356?\000\000\000\000\000\370\007\275&\267\315wy\217\356?\000\000\000\000\000p\363\274\350\220\244\242\257\244\356?\000\000\000\000\000\320\345<\344\312|\206\364\271\356?\000\000\000\000\000\032\026=\rh\216-H\317\356?\000\000\000\000\000P\365<\024\205\030\242\252\344\356?\000\000\000\000\000@\306<\023Za\356\033\372\356?\000\000\000\000\000\200\356\274\006A\266\034\234\017\357?\000\000\000\000\000\210\372\274c\271k7+%\357?\000\000\000\000\000\220,\275ur\335H\311:\357?\000\000\000\000\000\000\252<$En[vP\357?\000\000\000\000\000\360\364\274\375D\210y2f\357?\000\000\000\000\000\200\312<8\276\234\255\375{\357?\000\000\000\000\000\274\372<\202<$\002\330\221\357?\000\000\000\000\000`\324\274\216\220\236\201\301\247\357?\000\000\000\000\000\f\013\275\021\325\2226\272\275\357?\000\000\000\000\000\340\300\274\224q\217+\302\323\357?\000\000\000\000\200\336\020\275\356#*k\331\351\357?\000\000\000\000\000C\356<\000\000\000\000\000\000\360?\000\000\000\000\000\000\000\000\276\274Z\372\032\013\360?\000\000\000\000\000@\263\274\0033\373\251=\026\360?\000\000\000\000\000\027\022\275\202\002;\024h!\360?\000\000\000\000\000@\272<l\200w>\232,\360?\000\000\000\000\000\230\357<\312\273\021.\3247\360?\000\000\000\000\000@\307\274\211\177n\350\025C\360?\000\000\000\000\0000\330<gT\366r_N\360?\000\000\000\000\000?\032\275Z\205\025\323\260Y\360?\000\000\000\000\000\204\002\275\225\037<\016\ne\360?\000\000\000\000\000`\361<\032\367\335)kp\360?\000\000\000\000\000$\025=-\250r+\324{\360?\000\000\000\000\000\240\351\274\320\233u\030E\207\360?\000\000\000\000\000@\346<\310\007f\366\275\222\360?\000\000\000\000\000x\000\275\203\363\306\312>\236\360?\000\000\000\000\000\000\230\27409\037\233\307\251\360?\000\000\000\000\000\240\377<\374\210\371lX\265\360?\000\000\000\000\000\310\372\274\212l\344E\361\300\360?\000\000\000\000\000\300\331<\026Hr+\222\314\360?\000\000\000\000\000 \005=\330]9#;\330\360?\000\000\000\000\000\320\372\274\363\321\3232\354\343\360?\000\000\000\000\000\254\033=\246\251\337_\245\357\360?\000\000\000\000\000\350\004\275\360\322\376\257f\373\360?\000\000\000\000\0000\r\275K#\327(0\007\361?\000\000\000\000\000P\361<[[\022\320\001\023\361?\000\000\000\000\000\000\354<\371*^\253\333\036\361?\000\000\000\000\000\274\026=\3251l\300\275*\361?\000\000\000\000\000@\350<}\004\362\024\2506\361?\000\000\000\000\000\320\016\275\351-\251\256\232B\361?\000\000\000\000\000\340\350<81O\223\225N\361?\000\000\000\000\000@\353<q\216\245\310\230Z\361?\000\000\000\000\0000\005=\337\303qT\244f\361?\000\000\000\000\0008\003=\021R}<\270r\361?\000\000\000\000\000\324(=\237\273\225\206\324~\361?\000\000\000\000\000\320\005\275\223\215\2148\371\212\361?\000\000\000\000\000\210\034\275f]7X&\227\361?\000\000\000\000\000\360\021=\247\313o\353[\243\361?\000\000\000\000\000H\020=\343\207\023\370\231\257\361?\000\000\000\000\0009G\275T]\004\204\340\273\361?\000\000\000\000\000\344$=C\034(\225/\310\361?\000\000\000\000\000 \n\275\262\271h1\207\324\361?\000\000\000\000\000\200\343<1@\264^\347\340\361?\000\000\000\000\000\300\352<8\331\374\"P\355\361?\000\000\000\000\000\220\001=\367\3158\204\301\371\361?\000\000\000\000\000x\033\275\217\215b\210;\006\362?\000\000\000\000\000\224-=\036\250x5\276\022\362?\000\000\000\000\000\000\330<A\335}\221I\037\362?\000\000\000\000\0004+=#\023y\242\335+\362?\000\000\000\000\000\370\031=\347aunz8\362?\000\000\000\000\000\310\031\275'\024\202\373\037E\362?\000\000\000\000\0000\002=\002\246\262O\316Q\362?\000\000\000\000\000H\023\275\260\316\036q\205^\362?\000\000\000\000\000p\022=\026}\342eEk\362?\000\000\000\000\000\320\021=\017\340\0354\016x\362?\000\000\000\000\000\3561=>c\365\341\337\204\362?\000\000\000\000\000\300\024\2750\273\221u\272\221\362?\000\000\000\000\000\330\023\275\t\337\037\365\235\236\362?\000\000\000\000\000\260\b=\233\016\321f\212\253\362?\000\000\000\000\000|\"\275:\332\332\320\177\270\362?\000\000\000\000\0004*=\371\032w9~\305\362?\000\000\000\000\000\200\020\275\331\002\344\246\205\322\362?\000\000\000\000\000\320\016\275y\025d\037\226\337\362?\000\000\000\000\000 \364\274\317.>\251\257\354\362?\000\000\000\000\000\230$\275\"\210\275J\322\371\362?\000\000\000\000\0000\026\275%\2661\n\376\006\363?\000\000\000\000\00062\275\013\245\356\3552\024\363?\000\000\000\000\200\337p\275\270\327L\374p!\363?\000\000\000\000\000H\"\275\242\351\250;\270.\363?\000\000\000\000\000\230%\275f\027d\262\b<\363?\000\000\000\000\000\320\036='\372\343fbI\363?\000\000\000\000\000\000\334\274\017\237\222_\305V\363?\000\000\000\000\000\3300\275\271\210\336\2421d\363?\000\000\000\000\000\310\"=9\252:7\247q\363?\000\000\000\000\000` =\376t\036#&\177\363?\000\000\000\000\000`\026\2758\330\005m\256\214\363?\000\000\000\000\000\340\n\275\303>q\033@\232\363?\000\000\000\000\000rD\275 \240\3454\333\247\363?\000\000\000\000\000 \b=\225n\354\277\177\265\363?\000\000\000\000\000\200>=\362\250\023\303-\303\363?\000\000\000\000\000\200\357<\"\341\355D\345\320\363?\000\000\000\000\000\240\027\275\2734\022L\246\336\363?\000\000\000\000\0000&=\314N\034\337p\354\363?\000\000\000\000\000\246H\275\214~\254\004E\372\363?\000\000\000\000\000\334<\275\273\240g\303\"\b\364?\000\000\000\000\000\270%=\225.\367!\n\026\364?\000\000\000\000\000\300\036=FF\t'\373#\364?\000\000\000\000\000`\023\275 \251P\331\3651\364?\000\000\000\000\000\230#=\353\271\204?\372?\364?\000\000\000\000\000\000\372<\031\211a`\bN\364?\000\000\000\000\000\300\366\274\001\322\247B \\\364?\000\000\000\000\000\300\013\275\026\000\035\355Aj\364?\000\000\000\000\000\200\022\275&3\213fmx\364?\000\000\000\000\000\3400=\000<\301\265\242\206\364?\000\000\000\000\000@-\275\004\257\222\341\341\224\364?\000\000\000\000\000 \f=r\323\327\360*\243\364?\000\000\000\000\000P\036\275\001\270m\352}\261\364?\000\000\000\000\000\200\007=\341)6\325\332\277\364?\000\000\000\000\000\200\023\2752\301\027\270A\316\364?\000\000\000\000\000\200\000=\333\335\375\231\262\334\364?\000\000\000\000\000p,=\226\253\330\201-\353\364?\000\000\000\000\000\340\034\275\002-\235v\262\371\364?\000\000\000\000\000 \031=\3011E\177A\b\365?\000\000\000\000\000\300\b\275*f\317\242\332\026\365?\000\000\000\000\000\000\372\274\352Q?\350}%\365?\000\000\000\000\000\bJ=\332N\235V+4\365?\000\000\000\000\000\330&\275\032\254\366\364\342B\365?\000\000\000\000\000D2\275\333\224]\312\244Q\365?\000\000\000\000\000<H=k\021\351\335p`\365?\000\000\000\000\000\260$=\336)\2656Go\365?\000\000\000\000\000ZA=\016\304\342\333'~\365?\000\000\000\000\000\340)\275o\307\227\324\022\215\365?\000\000\000\000\000\b#\275L\013\377'\b\234\365?\000\000\000\000\000\354M='TH\335\007\253\365?\000\000\000\000\000\000\304\274\364z\250\373\021\272\365?\000\000\000\000\000\b0=\013FY\212&\311\365?\000\000\000\000\000\310&\275?\216\231\220E\330\365?\000\000\000\000\000\232F=\341 \255\025o\347\365?\000\000\000\000\000@\033\275\312\353\334 \243\366\365?\000\000\000\000\000p\027=\270\334v\271\341\005\366?\000\000\000\000\000\370&=\025\367\315\346*\025\366?\000\000\000\000\000\000\001=1U:\260~$\366?\000\000\000\000\000\320\025\275\265)\031\035\3353\366?\000\000\000\000\000\320\022\275\023\303\3144FC\366?\000\000\000\000\000\200\352\274\372\216\274\376\271R\366?\000\000\000\000\000`(\275\2273U\2028b\366?\000\000\000\000\000\376q=\2162\b\307\301q\366?\000\000\000\000\000 7\275~\251L\324U\201\366?\000\000\000\000\000\200\346<q\224\236\261\364\220\366?\000\000\000\000\000x)\275"

	.section	__TEXT,__literal8,8byte_literals
	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.87:
	.ascii	"\000\000\000?\000\000\000\277"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.88:
	.ascii	"\000\000\200?\000\000\300?"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.89:
	.ascii	"\000\000\000\000\334\317\3215"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.90:
	.ascii	"\000\000\000\000\000\300\025?"

	.section	__TEXT,__literal16,16byte_literals
	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.91:
	.ascii	"8c\355>\332\017I?^\230{?\332\017\311?"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.92:
	.ascii	"i7\2541h!\"3\264\017\0243h!\2423"

	.section	__TEXT,__const
	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.93:
	.ascii	"\026V\347\236\257\003\322<\233+\241\206\233\204\006=\202vIh\302%<=\021\352-\201\231\227q=\225dy\341\177\375\245=\273\275\327\331\337|\333=\225\326&\350\013.\021>:\2140\342\216yE>H\257\274\232\362\327z>\215\355\265\240\367\306\260>\361h\343\210\265\370\344>-C\034\353\3426\032?\374\251\361\322MbP?{\024\256G\341z\204?\232\231\231\231\231\231\271?\000\000\000\000\000\000\360?\000\000\000\000\000\000$@\000\000\000\000\000\000Y@\000\000\000\000\000@\217@\000\000\000\000\000\210\303@\000\000\000\000\000j\370@\000\000\000\000\200\204.A\000\000\000\000\320\022cA\000\000\000\000\204\327\227A\000\000\000\000e\315\315A\000\000\000 _\240\002B\000\000\000\350vH7B\000\000\000\242\224\032mB\000\000@\345\2340\242B\000\000\220\036\304\274\326B\000\0004&\365k\fC"

	.p2align	3, 0x0
__RNvNtNtCsdtCTLUYU3dx_4libm4math5exp2f6EXP2FT:
	.ascii	"\315;\177f\236\240\346?\207\001\353s\024\241\347?\333\240*B\345\254\350?\220\360\243\202\221\304\351?\255\323Z\231\237\350\352?\234R\205\335\233\031\354?\207\244\373\334\030X\355?\332\220\244\242\257\244\356?\000\000\000\000\000\000\360?\017\211\371lX\265\360?{Q}<\270r\361?8bunz8\362?\025\2671\n\376\006\363?\"4\022L\246\336\363?'*6\325\332\277\364?)TH\335\007\253\365?"

	.p2align	2, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.94:
	.ascii	"\225\277\3263\2757\2065\254\305'7\027\267\3218o\022\203:\n\327#<\315\314\314=\000\000\200?\000\000 A\000\000\310B\000\000zD\000@\034F\000P\303G\000$tI\200\226\030K"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.95:
	.ascii	"\236\244\301CQ\352\025BWL\365up\374#B\032\266\212\2012\241 B\032&\350\"\265\260\020B\350\241\245\263\301\177\366A?\363\323\365\030t\325A*_\271{\f\253\255A\230\314]\371v\370}AL\364\200P\351\361EA\353\207\207\037B\266\006A\004\330X\b\254\207\277@R;\274{`Zj@\005'\366\037\223\r\004@"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.96:
	.ascii	"\000\000\000\000\000\000\000\000\000\000\000\000\250\b\203A\000\000\000\200i\275\234A\000\000\000\260\246\375\241A\000\000\000pq\030\231A\000\000\000\220\266\356\205A\000\000\000\340qqiA\000\000\000\000{\037DA\000\000\000\000\274\320\025A\000\000\000\000\200\347\337@\000\000\000\000\000\024\236@\000\000\000\000\000\200P@\000\000\000\000\000\000\360?"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.97:
	.ascii	"\000\000\000\000\000\000\360?\000\000\000\000\000\000\360?\000\000\000\000\000\000\000@\000\000\000\000\000\000\030@\000\000\000\000\000\0008@\000\000\000\000\000\000^@\000\000\000\000\000\200\206@\000\000\000\000\000\260\263@\000\000\000\000\000\260\343@\000\000\000\000\000&\026A\000\000\000\000\200\257KA\000\000\000\000\250\b\203A\000\000\000\000\374\214\274A\000\000\000\300\2142\367A\000\000\000(;L4B\000\000\200uw\007sB\000\000\200uw\007\263B\000\000\330\354\3567\364B\000\000s\312\354\2766C\000\220h0\271\002{C\000ZA\276\263\341\300C \306\265\351;(\006Dl\360YaRwND"

	.globl	__RNvNtNtNtCsdtCTLUYU3dx_4libm4math7generic4sqrt9RSQRT_TAB
	.p2align	1, 0x0
__RNvNtNtNtCsdtCTLUYU3dx_4libm4math7generic4sqrt9RSQRT_TAB:
	.ascii	"Q\264\360\262\226\261D\260\371\256\266\255y\254C\253\024\252\353\250\310\247\252\246\222\245\200\244s\243k\242h\241j\240p\237{\236\212\235\235\234\265\233\321\232\360\231\023\231:\230e\227\223\226\304\225\370\2240\224k\223\251\222\352\221.\221u\220\276\217\n\217Y\216\252\215\376\214T\214\254\213\007\213d\212\304\211%\211\211\210\356\207V\207\300\206+\206\231\205\b\205y\204\354\203a\203\330\202P\202\311\201E\201\302\200@\200\002\377\016\375%\373G\371s\367\252\365\352\3634\362\207\360\343\356G\355\263\353'\352\243\350'\347\262\345C\344\334\342z\341 \340\313\336}\3354\334\361\332\263\331{\330H\327\032\326\361\324\315\323\255\322\222\321{\320i\317[\316Q\315J\314H\313J\312O\311X\310d\307t\306\207\305\235\304\267\303\324\302\364\301\026\301<\300e\277\220\276\276\275\357\274#\274Y\273\221\272\314\271\n\271J\270\214\267\320\266\027\266`\265"

l_anon.55ed7662f03e4ae27c90b6ed7e696e35.98:
	.ascii	"not implemented"

	.section	__TEXT,__cstring,cstring_literals
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.99:
	.asciz	"/Users/idlewile/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libm-0.2.16/src/math/support/int_traits.rs"

	.section	__DATA,__const
	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.100:
	.quad	l_anon.55ed7662f03e4ae27c90b6ed7e696e35.99
	.asciz	"o\000\000\000\000\000\000\000\"\001\000\000\001\000\000"

	.section	__TEXT,__const
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.101:
	.ascii	"not implemented: signed i128 widening multiply is not used"

	.section	__TEXT,__cstring,cstring_literals
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.102:
	.asciz	"/Users/idlewile/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libm-0.2.16/src/math/support/big.rs"

	.section	__DATA,__const
	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.103:
	.quad	l_anon.55ed7662f03e4ae27c90b6ed7e696e35.102
	.asciz	"h\000\000\000\000\000\000\000\374\000\000\000\t\000\000"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.104:
	.quad	l_anon.55ed7662f03e4ae27c90b6ed7e696e35.99
	.asciz	"o\000\000\000\000\000\000\000#\001\000\000\001\000\000"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.105:
	.quad	l_anon.55ed7662f03e4ae27c90b6ed7e696e35.99
	.asciz	"o\000\000\000\000\000\000\000$\001\000\000\001\000\000"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.106:
	.quad	l_anon.55ed7662f03e4ae27c90b6ed7e696e35.99
	.asciz	"o\000\000\000\000\000\000\000%\001\000\000\001\000\000"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.107:
	.quad	l_anon.55ed7662f03e4ae27c90b6ed7e696e35.99
	.asciz	"o\000\000\000\000\000\000\000&\001\000\000\001\000\000"

	.p2align	3, 0x0
l_anon.55ed7662f03e4ae27c90b6ed7e696e35.108:
	.quad	l_anon.55ed7662f03e4ae27c90b6ed7e696e35.99
	.asciz	"o\000\000\000\000\000\000\000'\001\000\000\001\000\000"

	.section	__TEXT,__const
	.p2align	3, 0x0
l_switch.table._RNvNtNtCsdtCTLUYU3dx_4libm4math5atan25atan2:
	.quad	0x3fe921fb54442d18
	.quad	0xbfe921fb54442d18
	.quad	0x4002d97c7f3321d2
	.quad	0xc002d97c7f3321d2

	.p2align	3, 0x0
l_switch.table._RNvNtNtCsdtCTLUYU3dx_4libm4math5atan25atan2.49:
	.quad	0x0000000000000000
	.quad	0x8000000000000000
	.quad	0x400921fb54442d18
	.quad	0xc00921fb54442d18

	.section	__TEXT,__literal16,16byte_literals
	.p2align	2, 0x0
l_switch.table._RNvNtNtCsdtCTLUYU3dx_4libm4math6atan2f6atan2f:
	.long	0x3f490fdb
	.long	0xbf490fdb
	.long	0x4016cbe4
	.long	0xc016cbe4

	.p2align	2, 0x0
l_switch.table._RNvNtNtCsdtCTLUYU3dx_4libm4math6atan2f6atan2f.50:
	.long	0x00000000
	.long	0x80000000
	.long	0x40490fdb
	.long	0xc0490fdb

	.globl	__RNvXNtNtNtCsdtCTLUYU3dx_4libm4math7support12float_traitsfNtB2_5Float8copysign
__RNvXNtNtNtCsdtCTLUYU3dx_4libm4math7support12float_traitsfNtB2_5Float8copysign = __RNvNtNtCsdtCTLUYU3dx_4libm4math8copysign9copysignf
	.globl	__RNvXs_NtNtNtCsdtCTLUYU3dx_4libm4math7support12float_traitsdNtB4_5Float8copysign
__RNvXs_NtNtNtCsdtCTLUYU3dx_4libm4math7support12float_traitsdNtB4_5Float8copysign = __RNvNtNtCsdtCTLUYU3dx_4libm4math8copysign8copysign
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math6scalbn7scalbnf
__RNvNtNtCsdtCTLUYU3dx_4libm4math6scalbn7scalbnf = __RNvNtNtCsdtCTLUYU3dx_4libm4math5ldexp6ldexpf
	.globl	__RNvXs0_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsjNtB5_3Int3abs
__RNvXs0_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsjNtB5_3Int3abs = __RNvXs0_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsjNtB5_3Int12unsigned_abs
	.globl	__RNvXs4_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitshNtB5_3Int3abs
__RNvXs4_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitshNtB5_3Int3abs = __RNvXs4_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitshNtB5_3Int12unsigned_abs
	.globl	__RNvXs8_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitstNtB5_3Int3abs
__RNvXs8_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitstNtB5_3Int3abs = __RNvXs8_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitstNtB5_3Int12unsigned_abs
	.globl	__RNvXsc_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsmNtB5_3Int3abs
__RNvXsc_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsmNtB5_3Int3abs = __RNvXsc_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsmNtB5_3Int12unsigned_abs
	.globl	__RNvXsg_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsyNtB5_3Int3abs
__RNvXsg_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsyNtB5_3Int3abs = __RNvXsg_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsyNtB5_3Int12unsigned_abs
	.globl	__RNvXsk_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsoNtB5_3Int3abs
__RNvXsk_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsoNtB5_3Int3abs = __RNvXsk_NtNtNtCsdtCTLUYU3dx_4libm4math7support10int_traitsoNtB5_3Int12unsigned_abs
	.globl	__RNvXs3_NtNtNtCsdtCTLUYU3dx_4libm4math7support3bignNtNtB7_10int_traits4HInt14zero_widen_mul
__RNvXs3_NtNtNtCsdtCTLUYU3dx_4libm4math7support3bignNtNtB7_10int_traits4HInt14zero_widen_mul = __RNvXs2_NtNtNtCsdtCTLUYU3dx_4libm4math7support3bigoNtNtB7_10int_traits4HInt9widen_mul
	.globl	__RNvNtNtCsdtCTLUYU3dx_4libm4math6scalbn6scalbn
__RNvNtNtCsdtCTLUYU3dx_4libm4math6scalbn6scalbn = __RNvNtNtCsdtCTLUYU3dx_4libm4math5ldexp5ldexp
.subsections_via_symbols
