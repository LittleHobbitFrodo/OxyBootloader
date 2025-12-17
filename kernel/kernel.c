#define NULL (void*)0

#include "boot_info.h"

//  stops kernel execution
void hang(void) {
    for (;;) {
        asm volatile("cli\nhlt");
    }
}

#define draw_vline(x, y, len)\
do {\
    usize _y = (y);\
    usize _x = (x);\
    usize _len = (len);\
    for (usize i = 0; i < _len; i++) {\
        fb[((_y + i) * w) + _x] = 0xffffff;\
    }\
} while (0)

#define draw_hline(x, y, len)\
do {\
    usize _y = (y);\
    usize _x = (x);\
    usize _len = (len);\
    for (usize i = 0; i < _len; i++) {\
        fb[(_y * w) + i + _x] = 0xffffff;\
    }\
} while (0)

//  the kernel entrypoint
extern void _start(struct OxyBootInfo* info) {

    if (info == NULL) {
        hang();
    }

    if (info->framebuffer.pointer == NULL) {
        hang();
    }

    volatile u32* fb = info->framebuffer.pointer;
    u64 w = info->framebuffer.w;
    u64 h = info->framebuffer.h;

    const u64 x = (w / 2) - 64;
    const u64 y = (h / 2) - 64;

    //  draw square
    draw_hline(x, y, 128);
    draw_hline(x, y + 128, 128);

    draw_vline(x, y, 128);
    draw_vline(x + 128, y, 128);




    hang();
}