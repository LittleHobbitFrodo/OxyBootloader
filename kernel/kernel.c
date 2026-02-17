#define NULL (void*)0

#include "boot_info.h"

//  stops kernel execution
void hang(void) {
    for (;;) {
        asm volatile("cli\nhlt");
    }
}

void draw_vline(struct OxyBootFrameBuffer* volatile fb, u64 x, u64 y, u64 len) {
    for (u64 i = 0; i < len; i++) {
        fb->pointer[((y + i) * fb->w) + x] = 0xffffff;
    }
}

void draw_hline(struct OxyBootFrameBuffer* volatile fb, u64 x, u64 y, u64 len) {
    for (u64 i = 0; i < len; i++) {
        fb->pointer[(y * fb->w) + i + x] = 0xffffff;
    }
}

//  the kernel entrypoint
extern void _start(struct OxyBootInfo* volatile info) {

    if (info == NULL) {
        hang();
    }

    if (info->framebuffer.pointer == NULL) {
        hang();
    }

    struct OxyBootFrameBuffer* volatile fb = &info->framebuffer;

    const u64 x = (info->framebuffer.w / 2) - 64;
    const u64 y = (info->framebuffer.h / 2) - 64;

    //  draw square
    draw_hline(fb, x, y, 128);
    draw_hline(fb, x, y + 128, 128);

    draw_vline(fb, x, y, 128);
    draw_vline(fb, x + 128, y, 128);

    hang();
}