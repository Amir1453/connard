.data
.globl x
x:
.quad 10
.text
.globl print
print:
	pushq	%rbp
	movq	%rsp, %rbp
	subq	$16, %rsp
	movq	%rdi, -8(%rbp)
.LRet:
	movq	%rbp, %rsp
	popq	%rbp
	retq
.text
.globl test_shadow
test_shadow:
	pushq	%rbp
	movq	%rsp, %rbp
	subq	$48, %rsp
	movq	%rdi, -8(%rbp)
.LEntry:
	movq	$5, -8(%rbp)
	movq	-8(%rbp), %r11
	addq	-8(%rbp), %r11
	movq	%r11, -16(%rbp)
	movq	-16(%rbp), %r11
	movq	%r11, -24(%rbp)
	movq	-24(%rbp), %r11
	movq	%r11, -32(%rbp)
	movq	-32(%rbp), %rax
	movq	%rbp, %rsp
	popq	%rbp
	retq
.text
.globl use_global
use_global:
	pushq	%rbp
	movq	%rsp, %rbp
	subq	$32, %rsp
.LEntry:
	movq	$2, -8(%rbp)
	movq	x(%rip), %rax
	imulq	-8(%rbp)
	movq	%rax, -16(%rbp)
	movq	-16(%rbp), %r11
	movq	%r11, -24(%rbp)
	movq	-24(%rbp), %rax
	movq	%rbp, %rsp
	popq	%rbp
	retq
.text
.globl main
main:
	pushq	%rbp
	movq	%rsp, %rbp
	subq	$160, %rsp
.LEntry:
	movq	x(%rip), %rdi
	callq	print
	movq	%rax, -8(%rbp)
	movq	$20, -16(%rbp)
	movq	-16(%rbp), %rdi
	callq	test_shadow
	movq	%rax, -24(%rbp)
	movq	-24(%rbp), %rdi
	callq	print
	movq	%rax, -32(%rbp)
	callq	use_global
	movq	%rax, -40(%rbp)
	movq	-40(%rbp), %rdi
	callq	print
	movq	%rax, -48(%rbp)
	movq	$30, -56(%rbp)
	movq	-56(%rbp), %r11
	movq	%r11, -64(%rbp)
	movq	-64(%rbp), %rdi
	callq	print
	movq	%rax, -72(%rbp)
	movq	-64(%rbp), %rdi
	callq	test_shadow
	movq	%rax, -80(%rbp)
	movq	-80(%rbp), %rdi
	callq	print
	movq	%rax, -88(%rbp)
	callq	use_global
	movq	%rax, -96(%rbp)
	movq	-96(%rbp), %rdi
	callq	print
	movq	%rax, -104(%rbp)
	movq	$20, -112(%rbp)
	movq	-64(%rbp), %r11
	subq	-112(%rbp), %r11
	movq	%r11, -120(%rbp)
	cmpq	$0, -120(%rbp)
	jg	.L0
	jmp	.L1
.L1:
.L0:
	movq	$50, -128(%rbp)
	movq	-128(%rbp), %r11
	movq	%r11, -136(%rbp)
	movq	-136(%rbp), %rdi
	callq	print
	movq	%rax, -144(%rbp)
	movq	-64(%rbp), %rdi
	callq	print
	movq	%rax, -152(%rbp)
	movq	%rbp, %rsp
	popq	%rbp
	retq
