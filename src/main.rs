#![no_main]
#![no_std]

pub mod utils;
pub mod memmap;
pub mod prelude;
//use elf_loader::{mmap::MmapImpl, segment::PAGE_SIZE};
use goblin::elf::Elf;
pub use prelude::*;
pub mod fs;
pub mod config;
//pub use prelude::elf;

use oxyboot_requests::{KernelEntryRequest, Request};
use uefi::{boot::{exit_boot_services, load_image, start_image}, proto::media::load_file};
use core::ptr::NonNull;


#[entry]
fn main() -> Status {

    uefi::helpers::init().unwrap();

    println!("Hello world!\n\n");

    memmap::parse();

    let config = match config::read() {
        Ok(cfg) => cfg,
        Err(status) => {
            println!("error while reading config: {status}");
            panic!();
        },
    };

    let kernel = match load_kernel(config) {
        Ok(slice) => slice,
        Err(e) => panic!("{e}"),
    };

    println!("kernel loaded at {:p} (size: {})", kernel.as_ptr(), kernel.len());

    //let memmap = unsafe { exit_boot_services(None) };

    let elf = match Elf::parse(kernel) {
        Ok(elf) => elf,
        Err(e) => {
            println!("failed to parse elf file: {e}");
            panic!("failed to parse the elf file: {e}")
        },
    };

    println!("kernel entry: {:p}", elf.entry as *mut u8);
    

    /*let kernel = unsafe {
        core::slice::from_raw_parts(kernel.as_ptr(), 4096)
    };

    println!("kernel:");
    for i in kernel {
        print!("{}", *i as char);
    }*/


    //load_image(parent_image_handle, source)
    //start_image(image_handle)
    //exit_boot_services(None)

    boot::stall(100_000_000);
    Status::SUCCESS
}

