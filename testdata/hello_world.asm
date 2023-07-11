section .text
global _start

_start:
    ; pop instruction
    pop rax
    push qword [0xcafebab]
    pop word [rax]
    pop qword [rax]
    pop word [rcx*2]
    pop qword [rcx*2]
    pop word [ecx*4+0xcafe]
    pop qword [ecx*4+0xcafe]

    pop fs
    pop gs
    
    ; push instructions
    ; push selectors
    push fs
    push gs
    ; Push Immediates
    push 0x8
    push 1024
    push 0xcafebab
    ; 0xFF /6
    push dx
;    push ebp
    push r13
    push r10w
;    push edi
    push rax
    push word [rax]
    push qword [rax]
    push word [rcx*2]
    push qword [rcx*2]
    push word [ecx*4+0xcafe]
    push qword [ecx*4+0xcafe]

    ; AND instructions with a 0x35 prefix
    and al, 0x1
    and ax, 0x1000
    and eax, 0x100000
    and rax, 0x5fffffff

    and bl, 0x1
    and bx, 0x1000

    ; 0x80 prefix
    and bl, 0x1

    ; 0x81 prefix
    and bx, 0x1000
    and ebx, 0x11111111
    and rbx, 0x5fffffff

    ; 0x83 prefix is only sign extensions, so no good
    
    ; 0x20 prefix
    and al, bl

    ; 0x21 prefix
    and ax, bx
    and eax, ebx
    and rax, rbx

    ; 0x21
    and cx, ax
    and [ecx], eax
    and [r9], r10
    ; 0x22
    and dl, [r15*2]
    ; 0x23
    and cx, [r12]
    and cx, [r12 + 0x19]
    and cx, [r9*2 + 0xcafeb19]
    and cx, [r12 + 0xcafeb19]
    and cx, [r12*2]
    and cx, [r12*8 + 0x19]
    and cx, [r15]
    and cx, [r15*2 + 0x19]
    and cx, [r15*4 + 0xcafeb19]
    and cx, [r15*8 + 0x19]
    and cx, [r15*2]
    and ecx, eax
    and r10, [r9]

    ; ADC instructions
    ; 0x14
    adc al, 10
    ; 0x15
    adc ax, 1700
    adc eax, 0xafebabe
    adc rax, 0xafebabe
    ; 0x80
    adc cl, 9
    adc rcx, 80
    ; 0x81
    adc dx, 110
    adc edx, 890
    adc r13, 0xabecafe
    ; 0x83
    adc dx, -10
    adc edx, -10
    adc rcx, -10
    ; 0x10
    adc cl, al
    ; 0x11
    adc cx, ax
    adc [ecx], eax
    adc [r9], r10
    ; 0x12
    adc dl, [r15*2]
    ; 0x13
    adc cx, [r12*8 + 0x19]
    adc ecx, eax
    adc r10, [r9]
    
    ; ADD instructions
    ; 0x04
    add al, 10
    ; 0x05
    add ax, 1700
    add eax, 0xafebabe
    add rax, 0xafebabe
    ; 0x80
    add cl, 9
    add rcx, 80
    ; 0x81
    add dx, 110
    add edx, 890
    add r13, 0xabecafe
    ; 0x83
    add dx, -10
    add edx, -10
    add rcx, -10
    ; 0x00
    add cl, al
    ; 0x01
    add cx, ax
    add [ecx], eax
    add [r9], r10
    ; 0x02
    add cl, [rax*2]
    ; 0x03
    add cx, [eax*8 + 0x09]
    add ecx, eax
    add r10, [r9]
    
    ; XOR instructions with a 0x35 prefix
    xor al, 0x1
    xor ax, 0x1000
    xor eax, 0x100000
    xor rax, 0x5fffffff

    xor bl, 0x1
    xor bx, 0x1000

    ; 0x80 prefix
    xor bl, 0x1

    ; 0x81 prefix
    xor bx, 0x1000
    xor ebx, 0x11111111
    xor rbx, 0x5fffffff

    ; 0x83 prefix is only sign extensions, so no good
    
    ; 0x30 prefix
    xor al, bl

    ; 0x31 prefix
    xor ax, bx
    xor eax, ebx
    xor rax, rbx

    ; LEA Instructions to identify addressing
    lea eax, [edx*8]
    lea ecx, [eax]
    lea edx, [edx + 8*EAX + 4]
    lea ebx, [rax]  
    lea rbx, [rax]  
    lea rbx, [rax * 8]  
    lea rbx, [r10 * 8]  
    lea r9, [r8]  
    lea r12, [r12]  
    lea r13, [r13]  
    lea r12, [r12*4 + 10]  
    lea r13, [rax + r13*8 + 3]  
    lea r12, [rbp + r12*8 + 3]  
    lea rsp, [ebp + ebp*8 + 3]   
    
    ; Hello World Code
    mov rdi, 0x1
    mov rsi, hello
    mov rdx, helloLen
    mov rax, 0x1
    syscall

    xor rdi, rdi
    mov rax, 0x3c
    syscall

section .data
    hello db "Hello World", 0xa
    helloLen equ $-hello
