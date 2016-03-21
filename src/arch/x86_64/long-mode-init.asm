; entrypoint from kernel into long mode

global long_mode_start

section .text
bits 64

; fn long_mode_start()
;   call into rust
long_mode_start:
   call mezzo
   hlt

; fn mezzo()
;   print '64' and halt
mezzo:
   mov qword [0xb80b4], 0x0000000007340736
   ret
