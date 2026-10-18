	.build_version macos, 11, 0
	.section	__TEXT,__text,regular,pure_instructions
	.p2align	2
__RINvMNtNtCsa9BKTri5B3M_3std4sync9once_lockINtB3_8OnceLockNtCs3sCpZ7x6bLi_7ca72_rt5CoresE10initializeNCINvB2_11get_or_initNCNvBV_5cores0E0zEBV_:
	.cfi_startproc
Lloh0:
	adrp	x8, __RNvNvCs3sCpZ7x6bLi_7ca72_rt5cores5CORES@PAGE+16
Lloh1:
	add	x8, x8, __RNvNvCs3sCpZ7x6bLi_7ca72_rt5cores5CORES@PAGEOFF+16
	ldapr	x8, [x8]
	cbnz	x8, LBB0_2
	ret
LBB0_2:
	sub	sp, sp, #48
	stp	x29, x30, [sp, #32]
	add	x29, sp, #32
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
Lloh2:
	adrp	x8, __RNvNvCs3sCpZ7x6bLi_7ca72_rt5cores5CORES@PAGE
Lloh3:
	add	x8, x8, __RNvNvCs3sCpZ7x6bLi_7ca72_rt5cores5CORES@PAGEOFF
	sub	x9, x29, #1
	stp	x8, x9, [sp]
	mov	x9, sp
	str	x9, [sp, #16]
Lloh4:
	adrp	x3, l_anon.652ba69ed9cd9f2957681739c1d77359.0@PAGE
Lloh5:
	add	x3, x3, l_anon.652ba69ed9cd9f2957681739c1d77359.0@PAGEOFF
Lloh6:
	adrp	x4, l_anon.652ba69ed9cd9f2957681739c1d77359.2@PAGE
Lloh7:
	add	x4, x4, l_anon.652ba69ed9cd9f2957681739c1d77359.2@PAGEOFF
	add	x0, x8, #16
	add	x2, sp, #16
	mov	w1, #1
	bl	__RNvMNtNtNtNtCsa9BKTri5B3M_3std3sys4sync4once5queueNtB2_4Once4call
	ldp	x29, x30, [sp, #32]
	add	sp, sp, #48
	ret
	.loh AdrpAdd	Lloh0, Lloh1
	.loh AdrpAdd	Lloh6, Lloh7
	.loh AdrpAdd	Lloh4, Lloh5
	.loh AdrpAdd	Lloh2, Lloh3
	.cfi_endproc

	.p2align	2
__RNCINvMs0_NtNtCsa9BKTri5B3M_3std4sync4onceNtB8_4Once15call_once_forceNCINvMNtBa_9once_lockINtB1b_8OnceLockNtCs3sCpZ7x6bLi_7ca72_rt5CoresE10initializeNCINvB1a_11get_or_initNCNvB1I_5cores0E0zE0E0B1I_:
Lfunc_begin0:
	.cfi_startproc
	.cfi_personality 155, _rust_eh_personality
	.cfi_lsda 16, Lexception0
	sub	sp, sp, #96
	.cfi_def_cfa_offset 96
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
	.cfi_remember_state
	ldr	x8, [x0]
	ldr	x21, [x8]
	str	xzr, [x8]
	cbz	x21, LBB1_15
	bl	__RNvNtNtCsa9BKTri5B3M_3std6thread9functions21available_parallelism
	mov	x20, x1
	cmp	x0, #1
	b.ne	LBB1_9
	mov	x19, x1
	and	x8, x1, #0x3
	mov	w20, #1
	sub	x9, x8, #2
	cmp	x9, #2
	b.lo	LBB1_9
	cbz	x8, LBB1_9
	ldr	x20, [x19, #-1]!
	ldr	x22, [x19, #8]
	ldr	x8, [x22]
	cbz	x8, LBB1_6
Ltmp0:
	mov	x0, x20
	blr	x8
Ltmp1:
LBB1_6:
	ldr	x1, [x22, #8]
	cbz	x1, LBB1_8
	ldr	x2, [x22, #16]
	mov	x0, x20
	bl	__RNvCs9wFQrvczXsK_7___rustc14___rust_dealloc
LBB1_8:
	mov	x0, x19
	mov	w1, #24
	mov	w2, #8
	bl	__RNvCs9wFQrvczXsK_7___rustc14___rust_dealloc
	mov	w20, #1
LBB1_9:
Lloh8:
	adrp	x0, l_anon.652ba69ed9cd9f2957681739c1d77359.9@PAGE
Lloh9:
	add	x0, x0, l_anon.652ba69ed9cd9f2957681739c1d77359.9@PAGEOFF
	mov	w1, #14
	bl	__RNvNtCs3sCpZ7x6bLi_7ca72_rt5cores6sysctl
	mov	x8, x20
	cmp	x0, #1
	b.ne	LBB1_14
	mov	x8, x20
	cmp	x1, #2
	b.lo	LBB1_14
Lloh10:
	adrp	x0, l_anon.652ba69ed9cd9f2957681739c1d77359.10@PAGE
Lloh11:
	add	x0, x0, l_anon.652ba69ed9cd9f2957681739c1d77359.10@PAGEOFF
	mov	w1, #25
	bl	__RNvNtCs3sCpZ7x6bLi_7ca72_rt5cores6sysctl
	mov	x8, x20
	cmp	x0, #1
	b.ne	LBB1_14
	cbz	x20, LBB1_16
	cmp	x1, x20
	csel	x8, x1, x20, lo
	cmp	x1, #0
	csinc	x8, x8, xzr, ne
LBB1_14:
	stp	x20, x8, [x21]
	.cfi_def_cfa wsp, 96
	ldp	x29, x30, [sp, #80]
	ldp	x20, x19, [sp, #64]
	ldp	x22, x21, [sp, #48]
	add	sp, sp, #96
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	ret
LBB1_15:
	.cfi_restore_state
Lloh12:
	adrp	x0, l_anon.652ba69ed9cd9f2957681739c1d77359.3@PAGE
Lloh13:
	add	x0, x0, l_anon.652ba69ed9cd9f2957681739c1d77359.3@PAGEOFF
	bl	__RNvNtCsgtPOCBgevO_4core6option13unwrap_failed
LBB1_16:
	mov	w8, #1
	stp	x8, xzr, [sp]
Lloh14:
	adrp	x8, __RNvXsZ_NtNtCsgtPOCBgevO_4core3fmt3numjNtB7_5Debug3fmt@PAGE
Lloh15:
	add	x8, x8, __RNvXsZ_NtNtCsgtPOCBgevO_4core3fmt3numjNtB7_5Debug3fmt@PAGEOFF
	mov	x9, sp
	stp	x9, x8, [sp, #16]
	add	x9, sp, #8
	stp	x9, x8, [sp, #32]
Lloh16:
	adrp	x0, l_anon.652ba69ed9cd9f2957681739c1d77359.11@PAGE
Lloh17:
	add	x0, x0, l_anon.652ba69ed9cd9f2957681739c1d77359.11@PAGEOFF
Lloh18:
	adrp	x2, l_anon.652ba69ed9cd9f2957681739c1d77359.5@PAGE
Lloh19:
	add	x2, x2, l_anon.652ba69ed9cd9f2957681739c1d77359.5@PAGEOFF
	add	x1, sp, #16
	bl	__RNvNtCsgtPOCBgevO_4core9panicking9panic_fmt
LBB1_17:
Ltmp2:
	mov	x21, x0
	ldr	x1, [x22, #8]
	cbz	x1, LBB1_19
	ldr	x2, [x22, #16]
	mov	x0, x20
	bl	__RNvCs9wFQrvczXsK_7___rustc14___rust_dealloc
LBB1_19:
	mov	x0, x19
	mov	w1, #24
	mov	w2, #8
	bl	__RNvCs9wFQrvczXsK_7___rustc14___rust_dealloc
	mov	x0, x21
	bl	__Unwind_Resume
	.loh AdrpAdd	Lloh8, Lloh9
	.loh AdrpAdd	Lloh10, Lloh11
	.loh AdrpAdd	Lloh12, Lloh13
	.loh AdrpAdd	Lloh18, Lloh19
	.loh AdrpAdd	Lloh16, Lloh17
	.loh AdrpAdd	Lloh14, Lloh15
Lfunc_end0:
	.cfi_endproc
	.section	__TEXT,__gcc_except_tab
	.p2align	2, 0x0
GCC_except_table1:
Lexception0:
	.byte	255
	.byte	255
	.byte	1
	.uleb128 Lcst_end0-Lcst_begin0
Lcst_begin0:
	.uleb128 Lfunc_begin0-Lfunc_begin0
	.uleb128 Ltmp0-Lfunc_begin0
	.byte	0
	.byte	0
	.uleb128 Ltmp0-Lfunc_begin0
	.uleb128 Ltmp1-Ltmp0
	.uleb128 Ltmp2-Lfunc_begin0
	.byte	0
	.uleb128 Ltmp1-Lfunc_begin0
	.uleb128 Lfunc_end0-Ltmp1
	.byte	0
	.byte	0
Lcst_end0:
	.p2align	2, 0x0

	.section	__TEXT,__text,regular,pure_instructions
	.p2align	2
__RNSNvYNCINvMs0_NtNtCsa9BKTri5B3M_3std4sync4onceNtBd_4Once15call_once_forceNCINvMNtBf_9once_lockINtB1g_8OnceLockNtCs3sCpZ7x6bLi_7ca72_rt5CoresE10initializeNCINvB1f_11get_or_initNCNvB1N_5cores0E0zE0E0INtNtNtCsgtPOCBgevO_4core3ops8function6FnOnceTRNtBd_9OnceStateEE9call_once6vtableB1N_:
	.cfi_startproc
	sub	sp, sp, #32
	.cfi_def_cfa_offset 32
	stp	x29, x30, [sp, #16]
	add	x29, sp, #16
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	ldr	x8, [x0]
	str	x8, [sp, #8]
	add	x0, sp, #8
	bl	__RNCINvMs0_NtNtCsa9BKTri5B3M_3std4sync4onceNtB8_4Once15call_once_forceNCINvMNtBa_9once_lockINtB1b_8OnceLockNtCs3sCpZ7x6bLi_7ca72_rt5CoresE10initializeNCINvB1a_11get_or_initNCNvB1I_5cores0E0zE0E0B1I_
	.cfi_def_cfa wsp, 32
	ldp	x29, x30, [sp, #16]
	add	sp, sp, #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
	.cfi_endproc

	.globl	__RNvCs3sCpZ7x6bLi_7ca72_rt18scheduled_realtime
	.p2align	2
__RNvCs3sCpZ7x6bLi_7ca72_rt18scheduled_realtime:
	.cfi_startproc
	sub	sp, sp, #64
	.cfi_def_cfa_offset 64
	stp	x29, x30, [sp, #48]
	add	x29, sp, #48
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	str	xzr, [sp, #32]
	movi.2d	v0, #0000000000000000
	stp	q0, q0, [sp]
	mov	w8, #10
	stur	w8, [x29, #-4]
	bl	_pthread_self
	bl	_pthread_mach_thread_np
	mov	x2, sp
	sub	x3, x29, #4
	mov	w1, #3
	bl	_thread_info
	ldur	w8, [x29, #-4]
	ldr	w9, [sp, #20]
	cmp	w9, #1
	cset	w9, ne
	cmp	w8, #5
	ccmp	w0, #0, #0, hi
	mov	w8, #2
	csel	w0, w9, w8, eq
	.cfi_def_cfa wsp, 64
	ldp	x29, x30, [sp, #48]
	add	sp, sp, #64
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
	.cfi_endproc

	.globl	__RNvCs3sCpZ7x6bLi_7ca72_rt5cores
	.p2align	2
__RNvCs3sCpZ7x6bLi_7ca72_rt5cores:
	.cfi_startproc
Lloh20:
	adrp	x8, __RNvNvCs3sCpZ7x6bLi_7ca72_rt5cores5CORES@PAGE+16
Lloh21:
	add	x8, x8, __RNvNvCs3sCpZ7x6bLi_7ca72_rt5cores5CORES@PAGEOFF+16
	ldapr	x8, [x8]
	cbnz	x8, LBB4_2
Lloh22:
	adrp	x8, __RNvNvCs3sCpZ7x6bLi_7ca72_rt5cores5CORES@PAGE
Lloh23:
	add	x8, x8, __RNvNvCs3sCpZ7x6bLi_7ca72_rt5cores5CORES@PAGEOFF
	ldp	x0, x1, [x8]
	ret
LBB4_2:
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	bl	__RINvMNtNtCsa9BKTri5B3M_3std4sync9once_lockINtB3_8OnceLockNtCs3sCpZ7x6bLi_7ca72_rt5CoresE10initializeNCINvB2_11get_or_initNCNvBV_5cores0E0zEBV_
	.cfi_def_cfa wsp, 16
	ldp	x29, x30, [sp], #16
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
Lloh24:
	adrp	x8, __RNvNvCs3sCpZ7x6bLi_7ca72_rt5cores5CORES@PAGE
Lloh25:
	add	x8, x8, __RNvNvCs3sCpZ7x6bLi_7ca72_rt5cores5CORES@PAGEOFF
	ldp	x0, x1, [x8]
	ret
	.loh AdrpAdd	Lloh20, Lloh21
	.loh AdrpAdd	Lloh22, Lloh23
	.loh AdrpAdd	Lloh24, Lloh25
	.cfi_endproc

	.globl	__RNvCs3sCpZ7x6bLi_7ca72_rt7promote
	.p2align	2
__RNvCs3sCpZ7x6bLi_7ca72_rt7promote:
	.cfi_startproc
	sub	sp, sp, #144
	.cfi_def_cfa_offset 144
	stp	x28, x27, [sp, #48]
	stp	x26, x25, [sp, #64]
	stp	x24, x23, [sp, #80]
	stp	x22, x21, [sp, #96]
	stp	x20, x19, [sp, #112]
	stp	x29, x30, [sp, #128]
	add	x29, sp, #128
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
	.cfi_offset w27, -88
	.cfi_offset w28, -96
	.cfi_remember_state
	mov	x21, x3
	mov	x20, x2
	mov	x23, x1
	mov	x22, x0
	mov	x19, x8
	str	xzr, [sp]
	mov	x0, sp
	bl	_mach_timebase_info
	cbnz	w0, LBB5_4
	ldr	w24, [sp]
	cbz	w24, LBB5_4
	mov	w25, #51712
	movk	w25, #15258, lsl #16
	umulh	x8, x22, x25
	mul	x9, x22, x25
	adds	x9, x9, w23, uxtw
	cinc	x8, x8, hs
	ldr	w26, [sp, #4]
	umulh	x10, x9, x26
	umaddl	x1, w8, w26, x10
	mul	x0, x9, x26
	mov	x2, x24
	mov	x3, #0
	bl	___udivti3
	mov	w27, #-1
	cmp	x0, x27
	csel	x8, x0, x27, lo
	cmp	x1, #0
	csinv	w28, w8, wzr, eq
	cmp	w23, w21
	cset	w8, lo
	cmp	x22, x20
	cset	w9, lo
	csel	w8, w8, w9, eq
	cmp	w8, #0
	csel	w8, w23, w21, ne
	csel	x9, x22, x20, ne
	umulh	x10, x9, x25
	mul	x9, x9, x25
	adds	x8, x9, x8
	cinc	x9, x10, hs
	umulh	x10, x8, x26
	umaddl	x1, w9, w26, x10
	mul	x0, x8, x26
	mov	x2, x24
	mov	x3, #0
	bl	___udivti3
	cmp	x0, x27
	csel	x8, x0, x27, lo
	cmp	x1, #0
	csinv	w8, w8, wzr, eq
	stp	w28, w8, [sp, #12]
	mov	w8, #1
	stp	w28, w8, [sp, #20]
	bl	_pthread_self
	bl	_pthread_mach_thread_np
	add	x2, sp, #12
	mov	w1, #2
	mov	w3, #4
	bl	_thread_policy_set
	str	w0, [sp, #28]
	cbz	w0, LBB5_6
Lloh26:
	adrp	x8, __RNvXs9_NtNtNtCsgtPOCBgevO_4core3fmt3num3implNtB9_7Display3fmt@GOTPAGE
Lloh27:
	ldr	x8, [x8, __RNvXs9_NtNtNtCsgtPOCBgevO_4core3fmt3num3implNtB9_7Display3fmt@GOTPAGEOFF]
	add	x9, sp, #28
	stp	x9, x8, [sp, #32]
Lloh28:
	adrp	x0, l_anon.652ba69ed9cd9f2957681739c1d77359.7@PAGE
Lloh29:
	add	x0, x0, l_anon.652ba69ed9cd9f2957681739c1d77359.7@PAGEOFF
	add	x1, sp, #32
	mov	x8, x19
	bl	__RNvNvNtCslJsSqIQKwiA_5alloc3fmt6format12format_inner
	b	LBB5_7
LBB5_4:
	bl	__RNvCs9wFQrvczXsK_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	w0, #16
	mov	w1, #1
	bl	__RNvCs9wFQrvczXsK_7___rustc12___rust_alloc
	cbz	x0, LBB5_8
Lloh30:
	adrp	x8, l_anon.652ba69ed9cd9f2957681739c1d77359.8@PAGE
Lloh31:
	add	x8, x8, l_anon.652ba69ed9cd9f2957681739c1d77359.8@PAGEOFF
Lloh32:
	ldr	q0, [x8]
	str	q0, [x0]
	mov	w8, #16
	stp	x8, x0, [x19]
	str	x8, [x19, #16]
	b	LBB5_7
LBB5_6:
	mov	x8, #-1
	str	x8, [x19]
LBB5_7:
	.cfi_def_cfa wsp, 144
	ldp	x29, x30, [sp, #128]
	ldp	x20, x19, [sp, #112]
	ldp	x22, x21, [sp, #96]
	ldp	x24, x23, [sp, #80]
	ldp	x26, x25, [sp, #64]
	ldp	x28, x27, [sp, #48]
	add	sp, sp, #144
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
	.cfi_restore w27
	.cfi_restore w28
	ret
LBB5_8:
	.cfi_restore_state
	mov	w0, #1
	mov	w1, #16
	bl	__RNvNtCslJsSqIQKwiA_5alloc7raw_vec12handle_error
	.loh AdrpAdd	Lloh28, Lloh29
	.loh AdrpLdrGot	Lloh26, Lloh27
	.loh AdrpAddLdr	Lloh30, Lloh31, Lloh32
	.cfi_endproc

	.globl	__RNvMCs3sCpZ7x6bLi_7ca72_rtNtB2_12WorkInterval3new
	.p2align	2
__RNvMCs3sCpZ7x6bLi_7ca72_rtNtB2_12WorkInterval3new:
	.cfi_startproc
	sub	sp, sp, #96
	.cfi_def_cfa_offset 96
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
	.cfi_remember_state
	mov	x19, x8
	mov	x8, sp
	bl	__RNvXs0_NvMs_NtNtCslJsSqIQKwiA_5alloc3ffi5c_strNtBa_7CString3newReNtB5_11SpecNewImpl13spec_new_impl
	ldp	x1, x0, [sp]
	cmp	x1, #1
	b.lt	LBB6_2
LBB6_1:
	mov	w2, #1
	bl	__RNvCs9wFQrvczXsK_7___rustc14___rust_dealloc
	b	LBB6_3
LBB6_2:
	cmn	x1, #1
	b.eq	LBB6_5
LBB6_3:
	mov	x8, #0
LBB6_4:
	str	x8, [x19]
	.cfi_def_cfa wsp, 96
	ldp	x29, x30, [sp, #80]
	ldp	x20, x19, [sp, #64]
	ldp	x22, x21, [sp, #48]
	ldp	d9, d8, [sp, #32]
	add	sp, sp, #96
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
LBB6_5:
	.cfi_restore_state
	mov	x21, x0
	ldr	x20, [sp, #16]
	str	xzr, [sp]
	mov	x0, sp
	bl	_mach_timebase_info
	cbnz	w0, LBB6_12
	ldr	w8, [sp]
	cbz	w8, LBB6_12
	mov	x0, x21
	mov	w1, #32
	mov	x2, #0
	bl	_AudioWorkIntervalCreate
	mov	x22, x0
	cbz	x0, LBB6_14
	ldr	d8, [sp]
	strb	wzr, [x21]
	cbz	x20, LBB6_10
LBB6_9:
	mov	x0, x21
	mov	x1, x20
	mov	w2, #1
	bl	__RNvCs9wFQrvczXsK_7___rustc14___rust_dealloc
LBB6_10:
	cbz	x22, LBB6_3
	str	x22, [x19, #8]
	str	d8, [x19, #16]
	mov	w8, #1
	b	LBB6_4
LBB6_12:
	strb	wzr, [x21]
	cbz	x20, LBB6_3
	mov	x0, x21
	mov	x1, x20
	b	LBB6_1
LBB6_14:
	bl	_os_release
	strb	wzr, [x21]
	cbnz	x20, LBB6_9
	b	LBB6_10
	.cfi_endproc

	.globl	__RNvMCs3sCpZ7x6bLi_7ca72_rtNtB2_12WorkInterval4join
	.p2align	2
__RNvMCs3sCpZ7x6bLi_7ca72_rtNtB2_12WorkInterval4join:
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
	ldr	x20, [x0]
	bl	__RNvCs9wFQrvczXsK_7___rustc35___rust_no_alloc_shim_is_unstable_v2
	mov	w0, #40
	mov	w1, #4
	bl	__RNvCs9wFQrvczXsK_7___rustc19___rust_alloc_zeroed
	cbz	x0, LBB7_4
	mov	x19, x0
	mov	x0, x20
	mov	x1, x19
	bl	_os_workgroup_join
	cbz	w0, LBB7_3
	mov	x0, x19
	mov	w1, #40
	mov	w2, #4
	bl	__RNvCs9wFQrvczXsK_7___rustc14___rust_dealloc
	mov	x19, #0
LBB7_3:
	mov	x0, x19
	.cfi_def_cfa wsp, 32
	ldp	x29, x30, [sp, #16]
	ldp	x20, x19, [sp], #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	ret
LBB7_4:
	.cfi_restore_state
	mov	w0, #4
	mov	w1, #40
	bl	__RNvNtCslJsSqIQKwiA_5alloc5alloc18handle_alloc_error
	.cfi_endproc

	.globl	__RNvMCs3sCpZ7x6bLi_7ca72_rtNtB2_12WorkInterval5leave
	.p2align	2
__RNvMCs3sCpZ7x6bLi_7ca72_rtNtB2_12WorkInterval5leave:
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
	mov	x19, x1
	ldr	x0, [x0]
	bl	_os_workgroup_leave
	mov	x0, x19
	mov	w1, #40
	mov	w2, #4
	.cfi_def_cfa wsp, 32
	ldp	x29, x30, [sp, #16]
	ldp	x20, x19, [sp], #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	b	__RNvCs9wFQrvczXsK_7___rustc14___rust_dealloc
	.cfi_endproc

	.globl	__RNvMCs3sCpZ7x6bLi_7ca72_rtNtB2_12WorkInterval5start
	.p2align	2
__RNvMCs3sCpZ7x6bLi_7ca72_rtNtB2_12WorkInterval5start:
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
	mov	x8, x2
	ldr	w2, [x0, #8]
	cbz	w2, LBB9_2
	mov	x19, x0
	mov	w9, #51712
	movk	w9, #15258, lsl #16
	umulh	x10, x1, x9
	mul	x9, x1, x9
	adds	x8, x9, w8, uxtw
	cinc	x9, x10, hs
	ldr	w10, [x0, #12]
	umulh	x11, x8, x10
	umaddl	x1, w9, w10, x11
	mul	x0, x8, x10
	mov	x3, #0
	bl	___udivti3
	mov	x20, x0
	bl	_mach_absolute_time
	mov	x1, x0
	ldr	x0, [x19]
	add	x2, x1, x20
	mov	x3, #0
	bl	_os_workgroup_interval_start
	cmp	w0, #0
	cset	w0, eq
	.cfi_def_cfa wsp, 32
	ldp	x29, x30, [sp, #16]
	ldp	x20, x19, [sp], #32
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	ret
LBB9_2:
	.cfi_restore_state
Lloh33:
	adrp	x0, l_anon.652ba69ed9cd9f2957681739c1d77359.6@PAGE
Lloh34:
	add	x0, x0, l_anon.652ba69ed9cd9f2957681739c1d77359.6@PAGEOFF
	bl	__RNvNtNtCsgtPOCBgevO_4core9panicking11panic_const23panic_const_div_by_zero
	.loh AdrpAdd	Lloh33, Lloh34
	.cfi_endproc

	.globl	__RNvMCs3sCpZ7x6bLi_7ca72_rtNtB2_12WorkInterval6finish
	.p2align	2
__RNvMCs3sCpZ7x6bLi_7ca72_rtNtB2_12WorkInterval6finish:
	.cfi_startproc
	stp	x29, x30, [sp, #-16]!
	.cfi_def_cfa_offset 16
	mov	x29, sp
	.cfi_def_cfa w29, 16
	.cfi_offset w30, -8
	.cfi_offset w29, -16
	ldr	x0, [x0]
	mov	x1, #0
	bl	_os_workgroup_interval_finish
	cmp	w0, #0
	cset	w0, eq
	.cfi_def_cfa wsp, 16
	ldp	x29, x30, [sp], #16
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	ret
	.cfi_endproc

	.p2align	2
__RNvNtCs3sCpZ7x6bLi_7ca72_rt5cores6sysctl:
	.cfi_startproc
	sub	sp, sp, #96
	.cfi_def_cfa_offset 96
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
	add	x8, sp, #8
	bl	__RNvXs0_NvMs_NtNtCslJsSqIQKwiA_5alloc3ffi5c_strNtBa_7CString3newReNtB5_11SpecNewImpl13spec_new_impl
	ldp	x1, x0, [sp, #8]
	cmp	x1, #1
	b.lt	LBB11_2
	mov	w2, #1
	bl	__RNvCs9wFQrvczXsK_7___rustc14___rust_dealloc
	mov	w21, #0
	b	LBB11_6
LBB11_2:
	cmn	x1, #1
	b.eq	LBB11_4
	mov	w21, #0
	b	LBB11_6
LBB11_4:
	ldr	x19, [sp, #24]
	stur	wzr, [x29, #-36]
	mov	w8, #4
	str	x8, [sp, #8]
	sub	x1, x29, #36
	add	x2, sp, #8
	mov	x20, x0
	mov	x3, #0
	mov	x4, #0
	bl	_sysctlbyname
	mov	x8, x0
	mov	x0, x20
	cmp	w8, #0
	ldur	w8, [x29, #-36]
	sxtw	x20, w8
	ccmp	w8, #0, #4, eq
	cset	w21, gt
	strb	wzr, [x0]
	cbz	x19, LBB11_6
	mov	x1, x19
	mov	w2, #1
	bl	__RNvCs9wFQrvczXsK_7___rustc14___rust_dealloc
LBB11_6:
	mov	w0, w21
	mov	x1, x20
	.cfi_def_cfa wsp, 96
	ldp	x29, x30, [sp, #80]
	ldp	x20, x19, [sp, #64]
	ldp	x22, x21, [sp, #48]
	add	sp, sp, #96
	.cfi_def_cfa_offset 0
	.cfi_restore w30
	.cfi_restore w29
	.cfi_restore w19
	.cfi_restore w20
	.cfi_restore w21
	.cfi_restore w22
	ret
	.cfi_endproc

	.globl	__RNvXs1_NtCs3sCpZ7x6bLi_7ca72_rt9workgroupNtB5_6HandleNtNtNtCsgtPOCBgevO_4core3ops4drop4Drop4drop
	.p2align	2
__RNvXs1_NtCs3sCpZ7x6bLi_7ca72_rt9workgroupNtB5_6HandleNtNtNtCsgtPOCBgevO_4core3ops4drop4Drop4drop:
	.cfi_startproc
	ldr	x0, [x0]
	b	_os_release
	.cfi_endproc

	.p2align	2
__RNvXsZ_NtNtCsgtPOCBgevO_4core3fmt3numjNtB7_5Debug3fmt:
	.cfi_startproc
	ldr	w8, [x1, #16]
	tbnz	w8, #25, LBB13_3
	tbnz	w8, #26, LBB13_4
	b	__RNvXsi_NtNtNtCsgtPOCBgevO_4core3fmt3num3impjNtB9_7Display3fmt
LBB13_3:
	b	__RNvXs6_NtNtCsgtPOCBgevO_4core3fmt3numjNtB7_8LowerHex3fmt
LBB13_4:
	b	__RNvXs8_NtNtCsgtPOCBgevO_4core3fmt3numjNtB7_8UpperHex3fmt
	.cfi_endproc

	.section	__DATA,__const
	.p2align	3, 0x0
l_anon.652ba69ed9cd9f2957681739c1d77359.0:
	.asciz	"\000\000\000\000\000\000\000\000\b\000\000\000\000\000\000\000\b\000\000\000\000\000\000"
	.quad	__RNSNvYNCINvMs0_NtNtCsa9BKTri5B3M_3std4sync4onceNtBd_4Once15call_once_forceNCINvMNtBf_9once_lockINtB1g_8OnceLockNtCs3sCpZ7x6bLi_7ca72_rt5CoresE10initializeNCINvB1f_11get_or_initNCNvB1N_5cores0E0zE0E0INtNtNtCsgtPOCBgevO_4core3ops8function6FnOnceTRNtBd_9OnceStateEE9call_once6vtableB1N_
	.quad	__RNCINvMs0_NtNtCsa9BKTri5B3M_3std4sync4onceNtB8_4Once15call_once_forceNCINvMNtBa_9once_lockINtB1b_8OnceLockNtCs3sCpZ7x6bLi_7ca72_rt5CoresE10initializeNCINvB1a_11get_or_initNCNvB1I_5cores0E0zE0E0B1I_

	.section	__TEXT,__cstring,cstring_literals
l_anon.652ba69ed9cd9f2957681739c1d77359.1:
	.asciz	"/rustc/8bab26f4f68e0e26f0bb7960be334d5b520ea452/library/std/src/sync/once.rs"

	.section	__DATA,__const
	.p2align	3, 0x0
l_anon.652ba69ed9cd9f2957681739c1d77359.2:
	.quad	l_anon.652ba69ed9cd9f2957681739c1d77359.1
	.asciz	"L\000\000\000\000\000\000\000\342\000\000\000\024\000\000"

	.p2align	3, 0x0
l_anon.652ba69ed9cd9f2957681739c1d77359.3:
	.quad	l_anon.652ba69ed9cd9f2957681739c1d77359.1
	.asciz	"L\000\000\000\000\000\000\000\342\000\000\0001\000\000"

	.section	__TEXT,__cstring,cstring_literals
l_anon.652ba69ed9cd9f2957681739c1d77359.4:
	.asciz	"crates/ca72-rt/src/lib.rs"

	.section	__DATA,__const
	.p2align	3, 0x0
l_anon.652ba69ed9cd9f2957681739c1d77359.5:
	.quad	l_anon.652ba69ed9cd9f2957681739c1d77359.4
	.asciz	"\031\000\000\000\000\000\000\000\232\000\000\0004\000\000"

	.p2align	3, 0x0
l_anon.652ba69ed9cd9f2957681739c1d77359.6:
	.quad	l_anon.652ba69ed9cd9f2957681739c1d77359.4
	.asciz	"\031\000\000\000\000\000\000\000(\004\000\000\031\000\000"

	.section	__TEXT,__cstring,cstring_literals
l_anon.652ba69ed9cd9f2957681739c1d77359.7:
	.asciz	"\032thread_policy_set failed (\300\001)"

	.section	__TEXT,__literal16,16byte_literals
l_anon.652ba69ed9cd9f2957681739c1d77359.8:
	.ascii	"no Mach timebase"

	.section	__TEXT,__const
l_anon.652ba69ed9cd9f2957681739c1d77359.9:
	.ascii	"hw.nperflevels"

l_anon.652ba69ed9cd9f2957681739c1d77359.10:
	.ascii	"hw.perflevel0.physicalcpu"

	.section	__DATA,__data
	.p2align	3, 0x0
__RNvNvCs3sCpZ7x6bLi_7ca72_rt5cores5CORES:
	.space	16
	.asciz	"\003\000\000\000\000\000\000"

	.section	__TEXT,__cstring,cstring_literals
l_anon.652ba69ed9cd9f2957681739c1d77359.11:
	.asciz	"\021min > max. min = \300\b, max = \300"

.subsections_via_symbols
