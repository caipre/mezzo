; implement multiboot2 header per spec:
; http://nongnu.askapache.com/grub/phcoder/multiboot.pdf

magic equ 0xe85250d6
arch  equ 0x0

section .multiboot-header
header_start:
   dd magic                      ; multiboot2 header magic number
   dd arch                       ; 32-bit i386, protected mode
   dd header_end - header_start  ; header length
   dd 0x100000000 - (magic + arch + (header_end - header_start))  ; checksum

   ; optional tags...

   dw 0x0  ; tags terminator
   dw 0x0  ;
   dd 0x8  ;
header_end:
