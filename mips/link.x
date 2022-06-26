
START_ADDR = 0x0;
ENTRY(__start)



SECTIONS
{
    .text :
    {
        . = ALIGN(0x8);
        *(.text.start)
        . = ALIGN(0x8);
        *(.text*)
    }

     _gp = ALIGN(8);
    .got : 
    {
        . = ALIGN(0x8);
        *(.got)
        . = ALIGN(0x8);
        *(.got*)   
    }

        .rodata : 
    {
        . = ALIGN(0x8);
        *(.rodata)
        . = ALIGN(0x8);
        *(.rodata*)   
    }

    .data : 
    {
        . = ALIGN(0x8);
        *(.data)
        . = ALIGN(0x8);
        *(.data*)   
    }

    .bss : 
    {
        . = ALIGN(0x8);
        *(.bss)
        . = ALIGN(0x8);
        *(.bss*)   
    }
    . = ALIGN(0x8);
    _sp = . + 0x5000;

    .pload : 
    {
        . = 0x0;
        . = ALIGN(0x8);
        *(.text)
        . = ALIGN(0x8);
        *(.got)
        . = ALIGN(0x8);
        *(.data)
        . = ALIGN(0x8);
        *(.rodata)
        . = ALIGN(0x8);
        *(.bss)
    }

    /DISCARD/ :
    {
        *( *gnu* )
    }
}