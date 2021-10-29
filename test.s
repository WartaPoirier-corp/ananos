.intel_syntax noprefix

main:
    mov rbx, 0xc1
    mov rax, 1
    int 0x80
    int3
    mov rbx, 0
loop:
    mov rax, 0
    int 0x80
    add rbx, 5

    jmp loop

