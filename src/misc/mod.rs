use core::{arch::asm, ptr::NonNull};

use allocator_api2::boxed::Box;
use uefi::{boot::{self, MemoryType, ScopedProtocol, get_handle_for_protocol, open_protocol}, proto::console::gop::{GraphicsOutput, PixelBitmask}};
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

    let mut gop: ScopedProtocol<GraphicsOutput> = match unsafe { open_protocol(gop_params, OpenProtocolAttributes::Exclusive) } {
        Ok(gop) => gop,
        Err(_) => return Err("failed to open the GOP protocol")
    };

    let info = gop.current_mode_info();
    let res = info.resolution();
    let mut fb = gop.frame_buffer();
    let fmt = info.pixel_format() as u64;
    let mask = info.pixel_bitmask().unwrap_or_default();
    let mut position = uefi_cursor_position();
    position.1 *= 2;    //  adjust to font being +- 8x16

    let ptr = match NonNull::new(fb.as_mut_ptr()) {
        Some(p) => p,
        None => {
            uefi::println!("pointer to FB is null");
            panic!()
        }
    };


    Ok(FrameBuffer::new(res, fmt, mask, ptr, position))

}


/// Flushes the page cache by reading the `cr3` value and then setting it
pub fn flush_page_cache() {
    let pml4 = unsafe {
        let ptr: u64;
        core::arch::asm!(
            "mov {}, cr3",
            out(reg) ptr,
            options(nomem, nostack, preserves_flags)
        );
        ptr
    };
    unsafe {
        core::arch::asm!(
            "mov cr3, {}",
            in(reg) pml4,
            options(nomem, nostack, preserves_flags),
        );
    }
}


/// Prints text to the screen (used only after boot services were exitted)
/// 
/// syntax: `print!(framebuffer: "text")`
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = write!(crate::FRAMEBUFFER.lock(), $($arg)*);
    }};
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
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let _ = writeln!(crate::FRAMEBUFFER.lock(), $($arg)*);
    }};
}

pub fn uefi_cursor_position() -> (usize, usize) {
    uefi::system::with_stdout(|x| x.cursor_position() )
}

/*/// Allocates memory for kernel stack
/// - returned pointer is already pointing at the start of the stack (highest address)
/// - given generic argument is in kilobytes
pub fn setup_stack<const STACK_SIZE: usize>() -> Result<NonNull<u8>, ()> {

    let real_size = STACK_SIZE * 1024;

    let stack = match boot::allocate_pages(boot::AllocateType::AnyPages, MemoryType::LOADER_DATA, (real_size / 4096) + 1) {
        Ok(stack) => stack,
        Err(_) => return Err(()),
    };

    unsafe { Ok(stack.add(real_size)) }
}*/


#[deprecated]
pub fn switch_to_kernel(kernel_entry: KernelEntry, stack_top: NonNull<u8>, boot_info: *mut BootInfo) -> ! {

    unsafe { boot_info.as_mut().unwrap() }.framebuffer.print("calling the asm routine");

    unsafe {
        asm!(r#"
            cli
            xor rbp, rbp
            mov rsp, {stack}
            mov rdi, {info}
            jmp {entry}
            "#,
            stack = in(reg) stack_top.as_ptr(),
            info = in(reg) boot_info,
            entry = in(reg) kernel_entry
        );
    }

    //kernel_entry(boot_info);

    //  set stack, disable interrupts
    /*unsafe {

        core::arch::asm!(
            // Disable interrupts
            "cli",

            // Set stack
            "mov rsp, {stack}",

            // Align stack for SysV ABI
            "and rsp, -16",
            "sub rsp, 8",

            // First argument: RDI
            "mov rdi, {boot_info}",

            // Jump to kernel (no return!)
            "jmp {entry}",

            stack = in(reg) stack_top.as_ptr(),
            boot_info = in(reg) boot_info,
            entry = in(reg) kernel_entry,

            options(noreturn)
        );
    }*/

    hang()
}

pub fn hang() -> ! {
    unsafe {
        loop {
            core::arch::asm!("hlt");
        }
    }
}

pub type KernelEntry = extern "C" fn(*mut BootInfo) -> !;