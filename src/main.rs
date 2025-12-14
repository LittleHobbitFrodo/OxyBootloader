#![no_main]
#![no_std]

pub mod utils;
pub mod prelude;
pub mod fs;
pub mod config;
pub mod kernel;
pub mod misc;

//use core::ptr::NonNull;

pub use prelude::*;
//use serde::de::IntoDeserializer;
//use uefi::mem::memory_map::MemoryMapMut;
use misc::FrameBuffer;

use crate::kernel::BootInfo;

//use crate::{kernel::BootInfo, misc::hang};


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


    let metadata = match kernel::prepare(kernel) {
        Ok(p) => p,
        Err(msg) => {
            println!("kernel preparation failed: {msg}");
            panic!();
        }
    };

    let pml4 = match kernel::setup_paging(&metadata) {
        Ok(p) => p,
        Err(e) => {
            uefi::println!("failed to setup paging: {e}");
            panic!();
        }
    };

    uefi::println!("pml4: {pml4:p}");

    /*let boot_info = match BootInfo::collect() {
        Ok(bi) => bi,
        Err(msg) => {
            uefi::println!("{msg}");
            panic!()
        }
    };*/

    

    uefi::println!("DONE");
    boot::stall(1_000_000_000);

    Status::SUCCESS


    /*let kernel_stack = match misc::setup_stack::<32>() {
        Ok(s) => s,
        Err(_) => {
            crate::println!("failed to setup stack");
            hang()
        }
    };

    uefi::println!("new stack: {kernel_stack:p}");


    let (pages, kernel_entry) = match kernel::prepare(kernel) {
        Ok((pages, entry)) => (pages, entry),
        Err(msg) => {
            uefi::println!("failed to prepare kernel: {msg}");
            panic!();
        },
    };

    uefi::println!("kernel entry: {kernel_entry:p}");


    let fb = match misc::get_framebuffer() {
        Ok(fb) => fb,
        Err(e) => {
            uefi::println!("failed to get framebuffer: {e}");
            panic!();
        }
    };

    let boot_info: *mut BootInfo = {
        let tmp = allocator_api2::boxed::Box::new(BootInfo {
            framebuffer: FrameBuffer::empty(),
            stack_size: 32 * 1024,
        });
        allocator_api2::boxed::Box::leak(tmp)
    };

    *FRAMEBUFFER.lock() = fb;



    //  EXIT BOOT SERVICES
    let mut memmap = unsafe { boot::exit_boot_services(None) };
    memmap.sort();


    crate::println!("mapping pages as executable");

    for page in pages.iter() {
        if let AllocatedPage::Executable(p) = page {
            kernel::make_executable(p);
        }
    }
    misc::flush_page_cache();

    crate::println!("creating boot information");

    unsafe {
        let bi = boot_info.as_mut().unwrap();
        bi.framebuffer = core::mem::replace(FRAMEBUFFER.lock().as_mut(), FrameBuffer::empty());
        bi.stack_size = 32 * 1024;
    }

    unsafe {
        boot_info.as_mut().unwrap().framebuffer.print("switching to kernel\n");
    }

    misc::switch_to_kernel(kernel_entry, kernel_stack, boot_info);*/
    
}


pub enum Either<A: Sized, B: Sized> {
    One(A),
    Two(B),
}