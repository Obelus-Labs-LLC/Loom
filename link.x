/* Linker script for Loom on FabricOS */

ENTRY(_start)

SECTIONS {
    /* Load address - after kernel (assuming kernel at 0x100000) */
    . = 0x200000;
    
    .text : {
        *(.text._start)
        *(.text .text.*)
    }
    
    .rodata : {
        *(.rodata .rodata.*)
    }
    
    .data : {
        *(.data .data.*)
    }
    
    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    }
    
    /* Heap starts after BSS */
    __heap_start = .;
    
    /* Stack at 32MB (grows down) */
    __stack_top = 0x2000000;
    
    /DISCARD/ : {
        *(.comment)
        *(.note*)
    }
}
