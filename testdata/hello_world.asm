section .text
global _start

_start:
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
 
    ; XOR instructions with a 0x35 prefix
    xor al, 0x1
    xor ax, 0x1000
    xor eax, 0x100000
    xor rax, 0x5fffffff

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
