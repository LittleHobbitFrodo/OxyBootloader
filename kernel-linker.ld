ENTRY(_start)

SECTIONS
{
    . = 0xFFFFFFFF80000000;   /* or where your kernel should be mapped */

    .text :
    {
        *(.text*)
    }

    .rodata :
    {
        *(.rodata*)
    }

    .data :
    {
        *(.data*)
    }

    .bss :
    {
        *(.bss*)
        *(COMMON)
    }
}
