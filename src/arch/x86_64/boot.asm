; entrypoint from bootloader for kernel

global start

section .text
bits 32
start:
   ; print '(mezzo)' and halt
   mov dword [0xb8000], 0x00000000
   mov dword [0xb8004], 0x00000000
   mov dword [0xb8008], 0x00000000
   mov dword [0xb800c], 0x00000000
   mov dword [0xb8010], 0x00000000
   mov dword [0xb8014], 0x00000000
   mov dword [0xb8018], 0x00000000
   mov dword [0xb801c], 0x00000000
   mov dword [0xb8020], 0x00000000

   mov dword [0xb80a4], 0x096d0728
   mov dword [0xb80a8], 0x097a0965
   mov dword [0xb80ac], 0x096f097a
   mov dword [0xb80b0], 0x00000729
   hlt
