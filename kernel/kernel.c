#define NULL (void*)0
#include "boot_info.h"

void hang(void) {
    asm volatile("cli\nhlt");
}

extern void _start(struct OxyBootInfo* info) {

    if (info->framebuffer.pointer == NULL) {
        hang();
    }

    u32* ptr = info->framebuffer.pointer;

    const u64 w = info->framebuffer.w;
    const u64 h = info->framebuffer.h;

    usize len;
    if (w < h) { len = w; } else { len = h; }

    for (usize i = 0; i < len; i++) {
        ptr[(i * w) + i] = 0xffffff;
    }



    hang();
}