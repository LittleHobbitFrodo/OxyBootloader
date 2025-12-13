#![no_main]
#![no_std]

pub mod utils;
pub mod prelude;
pub mod fs;
pub mod config;
pub mod kernel;
pub mod misc;

pub use prelude::*;
use uefi::mem::memory_map::MemoryMapMut;
use kernel::AllocatedPage;
use misc::FrameBuffer;


pub static FRAMEBUFFER: Mutex<FrameBuffer> = Mutex::new(FrameBuffer::empty());


//  TODO: rewrite the config module

#[entry]
fn main() -> Status {

    uefi::helpers::init().unwrap();


    let config = match config::load() {
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



    let (pages, kernel_entry) = match kernel::prepare(kernel) {
        Ok((pages, entry)) => (pages, entry),
        Err(msg) => {
            uefi::println!("failed to prepare kernel: {msg}");
            panic!();
        },
    };



    let fb = match misc::get_framebuffer() {
        Ok(fb) => fb,
        Err(e) => {
            uefi::println!("failed to get framebuffer: {e}");
            panic!();
        }
    };

    *FRAMEBUFFER.lock() = fb;
    
    let mut memmap = unsafe { boot::exit_boot_services(None) };
    memmap.sort();

    for page in pages.iter() {
        if let AllocatedPage::Executable(p) = page {
            kernel::make_executable(p);
        }
    }
    misc::flush_page_cache();

    crate::println!("kernel marked as executable");

    let stack = match misc::setup_stack::<32>() {
        Ok(s) => s,
        Err(_) => {
            crate::println!("failed to setup stack");
            panic!()
        }
    };

    let boot_info = kernel::BootInfo {
        framebuffer: core::mem::replace(FRAMEBUFFER.lock().as_mut(), FrameBuffer::empty()),
        stack_size: 32*1024,
    };


    misc::switch_to_kernel(kernel_entry, stack, boot_info);
    
}


pub enum Either<A: Sized, B: Sized> {
    One(A),
    Two(B),
}