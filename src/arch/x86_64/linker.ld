ENTRY(start)

SECTIONS {
   . = 1M;

   .boot :
   {
      KEEP(*(.multiboot-header))
   }

   .text :
   {
      *(.text)
   }
}
