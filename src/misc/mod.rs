use core::{ffi::c_void, ptr::NonNull, slice};

use uefi::{boot::{self, ScopedProtocol, get_handle_for_protocol, open_protocol}, proto::console::gop::{GraphicsOutput, PixelBitmask}};
use uefi::boot::{OpenProtocolAttributes, OpenProtocolParams};
use crate::String;

mod framebuffer;
pub use framebuffer::*;


/// Returns the framebuffer while leaving the gop progotol opened
pub fn get_framebuffer() -> Result<FrameBuffer, String> {

    let handle = match get_handle_for_protocol::<GraphicsOutput>() {
        Ok(h) => h,
        Err(_) => return Err(String::from("failed to get GOP handle"))
    };

    let gop_params = OpenProtocolParams {
        handle, agent: boot::image_handle(), controller: None,
    };

    let mut gop: ScopedProtocol<GraphicsOutput> = match unsafe { open_protocol(gop_params, OpenProtocolAttributes::Exclusive) } {
        Ok(gop) => gop,
        Err(_) => return Err(String::from("failed to open the GOP protocol"))
    };

    let info = gop.current_mode_info();
    let res = info.resolution();
    let mut fb = gop.frame_buffer();
    let fmt = info.pixel_format() as u64;
    let mask = info.pixel_bitmask().unwrap_or_default();
    let position = get_cursor_position();
    uefi::println!("cursor position: {} : {}", position.0, position.1);

    let ptr = match NonNull::new(fb.as_mut_ptr()) {
        Some(p) => p,
        None => {
            uefi::println!("pointer to FB is null");
            panic!()
        }
    };


    Ok(FrameBuffer::new(res, fmt, mask, ptr, position))

}


/// Prints text to the screen (used only after boot services were exitted)
/// 
/// syntax: `print!(framebuffer: "text")`
#[macro_export]
macro_rules! print {
    ($fb:ident: $($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = write!($fb, $($arg)*);
    }};
}

/// Prints text to the screen (used only after boot services were exitted)
/// 
/// syntax: `println!(framebuffer: "text")`
#[macro_export]
macro_rules! println {
    ($fb:ident: $($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = writeln!($fb, $($arg)*);
    }};
}

use uefi::proto::console::text::Output;

fn get_cursor_position() -> (usize, usize) {
    uefi::system::with_stdout(|x| x.cursor_position() )
}