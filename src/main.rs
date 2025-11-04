#![no_main]
#![no_std]

pub mod utils;
pub mod memmap;
pub mod prelude;
use elf_loader::{mmap::MmapImpl, segment::PAGE_SIZE};
pub use prelude::*;
pub mod fs;
pub mod config;
pub use prelude::elf;

use oxyboot_requests::{KernelEntryRequest, Request};
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

    let (kernel, size) = match load_kernel(&config) {
        Ok(data) => data,
        Err(o) => if let Some(msg) = o {
            println!("error while loading kernel: {msg}");
            panic!();
        } else {
            println!("error while loading kernel : unknown error");
            panic!();
        }
    };

    println!("kernel loaded at {kernel:p} (size: {size})");




    boot::stall(100_000_000);
    Status::SUCCESS
}
