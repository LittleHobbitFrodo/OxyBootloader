
#include <stdint.h>

typedef uint64_t u64;
typedef uint64_t usize;

#ifndef KERNEL_BOOT_INFO_H
    #define KERNEL_BOOT_INFO_H
#endif

struct OxyBootInfo {

};

struct OxyBootFrameBuffer {
    u64 w;
    u64 h;
    u64 pixel_fmt;
};