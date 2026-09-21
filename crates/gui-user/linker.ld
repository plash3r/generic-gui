ENTRY(_start)

PHDRS
{
    text PT_LOAD FLAGS(5);
    data PT_LOAD FLAGS(6);
}

SECTIONS
{
    . = 0x400000;

    .text : ALIGN(4096)
    {
        *(.text*)
    } :text

    .rodata : ALIGN(4096)
    {
        *(.rodata*)
    } :text

    . = ALIGN(4096);

    .data :
    {
        *(.data*)
        *(.got*)
    } :data

    .bss (NOLOAD) :
    {
        *(.bss*)
        *(COMMON)
    } :data

    /DISCARD/ :
    {
        *(.eh_frame*)
        *(.eh_frame_hdr*)
        *(.comment*)
        *(.note*)
    }
}
