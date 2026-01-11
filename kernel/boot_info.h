#ifndef KERNEL_BOOT_INFO_H
    #define KERNEL_BOOT_INFO_H
#endif

#include <stdint.h>
typedef uint64_t u64;
typedef uint32_t u32;
typedef uint16_t u16;
typedef uint8_t u8;
typedef uint64_t usize;


struct PixelBitmask {
    u32 red;
    u32 green;
    u32 blue;
    u32 reserved;
};

struct OxyBootFrameBuffer {
    u64 w;
    u64 h;
    u32* volatile pointer;
    u64 pixel_fmt;
    usize row;
    usize line;
    u32 color;
    struct PixelBitmask bit_mask;
    usize byte_size;
};

struct OxyBootInfo {
    struct OxyBootFrameBuffer framebuffer;
    u8* stack_bottom;
    usize stack_size;
};