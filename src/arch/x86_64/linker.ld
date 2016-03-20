ENTRY(start)

SECTIONS {
   . = 1M;

   .boot :
   {
      *(.multiboot-header)
   }

   .text :
   {
      *(.text)
   }
}
