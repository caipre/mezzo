; entrypoint from bootloader into kernel

global start

extern long_mode_start

section .text
bits 32

; fn start()
;   setup stack, check system, setup paging, enter long mode
start:
   mov esp, stack_top
   call clear

   call check_multiboot
   call check_cpuid
   call check_sse
   call check_long_mode

   call mezzo

   call setup_paging_tables
   call enable_paging
   call enable_sse

   call enter_long_mode
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
;   print '(mezzo)'
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

; fn check_sse()
;   verify that processor supports sse
check_sse:
   mov eax, 0x1        ; cpuid: return processor info and feature bits
   cpuid
   test edx, 1 << 25
   jz .no_sse
   ret

   .no_sse:
      mov al, "03"
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
      mov ax, '04'
      jmp error

; fn setup_paging_tables()
;   map p4 -> p3 -> p2 -> physical
setup_paging_tables:
   mov eax, p3_table   ; mark p3 as writable and present, and
   or  eax, 0b11       ; add it as the first entry in p4
   mov [p4_table], eax ;

   mov eax, p2_table   ; the same for p2 in p3
   or  eax, 0b11       ;
   mov [p3_table], eax ;

   ; identity map each p2 entry as a 2mib frame (512 * 2mib = 1gib)
   mov ecx, 0x0
   .map_p2_entries:
      mov eax, 0x200000
      mul ecx
      or  eax, 0x83    ; huge + writable + present
      mov [p2_table + (ecx * 8)], eax

      inc ecx
      cmp ecx, 512
      jb  .map_p2_entries

   ret

; fn enable_paging()
;   set various bits to enable long mode and paging
enable_paging:
   ; map cr3 to p4 paging table
   mov eax, p4_table
   mov cr3, eax

   ; set "physical address extended" bit of cr4
   mov eax, cr4
   or  eax, 1 << 5
   mov cr4, eax

   ; set "long mode" bit of "extended feature enable register"
   mov ecx, 0xc0000080
   rdmsr
   or eax, 1 << 8
   wrmsr

   ; set "paging" bit in the cr0 register
   mov eax, cr0
   or eax, 1 << 31
   mov cr0, eax

   ret

; fn enable_sse
;   set various bits to enable sse
enable_sse:
    mov eax, cr0
    and ax, 0xfffb     ; clear coprocessor emulation cr0.em
    or ax, 0x2         ; set coprocessor monitoring  cr0.mp
    mov cr0, eax
    mov eax, cr4
    or ax, 0b11 << 9   ; set cr4.osfxsr and cr4.osxmmexcpt at the same time
    mov cr4, eax

    ret

; fn enter_long_mode()
;   load the global descriptor table and setup selectors then jump to long mode
enter_long_mode:
   lgdt [gdt64.pointer]

   mov ax, gdt64.data
   mov ss, ax
   mov ds, ax
   mov es, ax

   jmp gdt64.code:long_mode_start

section .bss
align 4096

p4_table:
   resb 4096
p3_table:
   resb 4096
p2_table:
   resb 4096

stack_bottom:
   resb 64
stack_top:

section .rodata

gdt64:
   dq 0                ; zero entry

   ; code segment: descriptor + present + reads + executable + page limit
   .code: equ $ - gdt64
      dq   (1 << 44) | (1 << 47) | (1 << 41) | (1 << 43) | (1 << 53)

   ; data segment: descriptor + present + writes
   .data: equ $ - gdt64
      dq   (1 << 44) | (1 << 47) | (1 << 41)

   .pointer:
      dw $ - gdt64 - 1
      dq gdt64
