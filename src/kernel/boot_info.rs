
//! the information passed to the kernel

/// Stack size in pages (8 (32 KB) by default)
pub const KERNEL_STACK_SIZE: usize = 8;

use crate::misc::{FrameBuffer, get_framebuffer};
use crate::String;
use core::ptr::NonNull;
use uefi::boot::{allocate_pages, AllocateType, MemoryType};

#[repr(C)]
#[derive(Debug)]
pub struct BootInfo {

    /// The default framebuffer
    pub framebuffer: FrameBuffer,

    /// Pointer to the bottom of the stack
    pub stack_bottom: NonNull<u8>,

    /// Stack size in bytes
    pub stack_size: usize,
    
}

impl BootInfo {

    /// returns pointer to self
    pub const fn as_ptr(&self) -> *const Self { self }

    /// Collects all the information needed for the `BootInfo` structure
    // /// - the collected framebuffer is stored into the `FRAMEBUFFER` static
    // ///   - use the `prepare()` function to be able to pass the information to the kernel
    // /// - You can use the custom `crate::print!()` macros after using this function
    pub fn collect() -> Result<Self, String> {

        let stack = match allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, KERNEL_STACK_SIZE) {
            Ok(ptr) => ptr,
            Err(_) => return Err(String::from("failed to allocate memory for stack"))
        };

        //uefi::println!("getting the framebuffer:");

        let fb = match get_framebuffer() {
            Ok(fb) => fb,
            Err(e) => {
                return Err(String::from("failed to get framebuffer: ") + e)
            }
        };

        uefi::println!("fb: {} x {}", fb.width(), fb.height());

        //uefi::println!("got framebuffer");

        Ok(Self {
            stack_bottom: stack,
            stack_size: KERNEL_STACK_SIZE * 4096,
            framebuffer: fb,
        })

    }

    pub fn stack_top(&self) -> NonNull<u8> {
        unsafe { self.stack_bottom.add(self.stack_size) }
    }
}