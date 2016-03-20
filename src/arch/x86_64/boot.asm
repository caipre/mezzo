; entrypoint from bootloader into kernel

global start

section .text
bits 32

; fn start()
;   setup stack, check system, setup paging, call into rust
start:
   mov esp, stack_top
   call clear

   call check_multiboot
   call check_cpuid
   call check_long_mode

   call mezzo
   hlt

; fn clear()
;   clear text from the first line
clear:
   mov dword [0xb8000], 0x00000000
   mov dword [0xb8004], 0x00000000
   mov dword [0xb8008], 0x00000000
   mov dword [0xb800c], 0x00000000
   mov dword [0xb8010], 0x00000000
   mov dword [0xb8014], 0x00000000
   mov dword [0xb8018], 0x00000000
   mov dword [0xb801c], 0x00000000
   mov dword [0xb8020], 0x00000000
   ret

; fn mezzo()
;   print '(mezzo)' and halt
mezzo:
   mov dword [0xb80a4], 0x096d0728
   mov dword [0xb80a8], 0x097a0965
   mov dword [0xb80ac], 0x096f097a
   mov dword [0xb80b0], 0x00000729
   ret

; fn error(err)
;   print 'ERROR:' with ascii code from ax and halt
error:
   mov dword [0xb80a4], 0x0c520c45
   mov dword [0xb80a8], 0x0c4f0c52
   mov dword [0xb80ac], 0x073a0c52
   mov dword [0xb80b0], 0x07200720
   mov dword [0xb80b4], 0x07200720
   or  byte  [0xb80b2], al
   or  byte  [0xb80b4], ah
   hlt

; fn check_multiboot()
;   verify that we were entered from a multiboot compliant loader
check_multiboot:
   cmp eax, 0x36d76289
   jne .no_multiboot
   ret

   .no_multiboot:
      mov ax, '01'
      jmp error

; fn check_cpuid()
;   verify that cpuid is supported (that we can flip flags bit 21)
check_cpuid:
   pushfd              ; copy flags to eax via stack
   pop eax             ;
   mov ecx, eax        ;   ...and also to ecx

   xor eax, 1 << 21    ; flip cpuid bit

   push eax            ; copy modified eax into flags
   popfd
   pushfd              ;   ...and back out again
   pop eax

   push ecx            ; restore original flags
   popfd

   cmp eax, ecx        ; compare original and modified flags
   je .no_cpuid
   ret

   .no_cpuid:
      mov ax, '02'
      jmp error

; fn check_long_mode()
;   verify that processor supports long mode
check_long_mode:
   mov eax, 0x80000000 ; cpuid: return highest function supported
   cpuid
   cmp eax, 0x80000001
   jb .no_long_mode

   mov eax, 0x80000001 ; cpuid: return extended processor features
   cpuid
   test edx, 1 << 29
   jz .no_long_mode
   ret

   .no_long_mode:
      mov ax, '03'
      jmp error

section .bss
stack_bottom:
   resb 64
stack_top:
