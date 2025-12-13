
//! the information passed to the kernel

use crate::misc::FrameBuffer;


#[repr(C)]
pub struct BootInfo {
    /// The default framebuffer
    pub framebuffer: FrameBuffer,
    /// Stack size (in kb)
    pub stack_size: usize,
}


