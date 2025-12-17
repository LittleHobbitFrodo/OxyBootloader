use core::ptr::NonNull;

use uefi::{boot::{self, ScopedProtocol, get_handle_for_protocol, open_protocol}, proto::console::gop::{GraphicsOutput, PixelBitmask, PixelFormat}};
use uefi::boot::{OpenProtocolAttributes, OpenProtocolParams};
use crate::kernel::BootInfo;
//use crate::String;

mod framebuffer;
pub use framebuffer::*;


/// Returns the framebuffer while leaving the gop progotol opened
pub fn get_framebuffer() -> Result<FrameBuffer, &'static str> {

    let handle = match get_handle_for_protocol::<GraphicsOutput>() {
        Ok(h) => h,
        Err(_) => return Err("failed to get GOP handle")
    };

    
    let gop_params = OpenProtocolParams {
        handle, agent: boot::image_handle(), controller: None,
    };

    let mut gop: ScopedProtocol<GraphicsOutput> = match unsafe { open_protocol(gop_params, OpenProtocolAttributes::GetProtocol) } {
        Ok(gop) => gop,
        Err(_) => return Err("failed to open the GOP protocol")
    };

    let info = gop.current_mode_info();
    let res = info.resolution();
    let mut fb = gop.frame_buffer();
    let fmt = info.pixel_format() as u64;
    let mask = info.pixel_bitmask().unwrap_or_default();
    let byte_size = info.resolution().0 * info.resolution().1 * 4;

    let ptr = match NonNull::new(fb.as_mut_ptr()) {
        Some(p) => p,
        None => {
            uefi::println!("pointer to FB is null");
            panic!()
        }
    };


    Ok(FrameBuffer::new(res, fmt, mask, ptr, (0, 0), byte_size))

}

pub type KernelEntry = extern "C" fn(*mut BootInfo) -> !;