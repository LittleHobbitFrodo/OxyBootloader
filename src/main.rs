#![no_main]
#![no_std]

pub mod utils;
pub mod memmap;
pub mod prelude;
use core::{ops::DerefMut, time::Duration};

pub use prelude::*;
use uefi::{boot::exit_boot_services, mem::memory_map::MemoryMapMut, proto::console::text::Output};
pub mod fs;
pub mod config;

//use oxyboot_requests::{KernelEntryRequest, Request};
//use uefi::{boot::{exit_boot_services, load_image, start_image}, proto::media::load_file};

pub mod kernel;
pub mod misc;
use misc::*;

use kernel::AllocatedPage;

pub static FRAMEBUFFER: Mutex<FrameBuffer> = Mutex::new(FrameBuffer::empty());


//  TODO: rewrite the config module

#[entry]
fn main() -> Status {

    uefi::helpers::init().unwrap();

    memmap::parse();

    let config = match config::read() {
        Ok(cfg) => cfg,
        Err(status) => {
            uefi::println!("error while reading config: {status}");
            panic!();
        },
    };

    let kernel = match kernel::load(&config) {
        Ok(slice) => slice,
        Err(e) => panic!("failed to load kernel: {e}"),
    };

    uefi::println!("kernel loaded");

    let (pages, kernel_entry) = match kernel::prepare(kernel) {
        Ok((pages, entry)) => (pages, entry),
        Err(msg) => {
            uefi::println!("failed to prepare kernel: {msg}");
            panic!();
        },
    };

    uefi::println!("kernel prepared");

    let fb = match get_framebuffer() {
        Ok(fb) => fb,
        Err(e) => {
            uefi::println!("failed to get framebuffer: {e}");
            panic!();
        }
    };

    *FRAMEBUFFER.lock() = fb;
    
    let mut memmap = unsafe { exit_boot_services(None) };
    memmap.sort();

    for page in pages.iter() {
        if let AllocatedPage::Executable(p) = page {
            if let Some(entry) = kernel::make_executable(p) {
                crate::println!("returned entry: {entry:?}");
            }
            flush_page_cache();
        }
    }

    crate::println!("kernel marked as executable");

    let stack = match setup_stack::<32>() {
        Ok(s) => s,
        Err(_) => {
            crate::println!("failed to setup stack");
            panic!()
        }
    };


    switch_to_kernel(kernel_entry, stack);
    
}

/*#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    loop {}
}*/


pub enum Either<A: Sized, B: Sized> {
    One(A),
    Two(B),
}