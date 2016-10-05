; entrypoint from assembly into long mode

global long_mode_start

extern __main__

section .text
bits 64

; fn long_mode_start()
;   call into rust
long_mode_start:
   call mezzo64
   call __main__
   call os_exit
   hlt

; fn mezzo64()
;   print '64'
mezzo64:
   mov qword [0xb80b4], 0x0000000007340736
   ret

; fn os_exit()
;   print 'fin'
os_exit:
   mov dword [0xb81e4], 0x07660000
   mov dword [0xb81e8], 0x076e0769
   ret
