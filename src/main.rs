#![no_main]
#![no_std]

use core::time::Duration;
use uefi::{boot::{MemoryType, memory_map}, mem::memory_map::MemoryMap};
pub use uefi::{self, entry, Status, print, println};
mod prelude;
pub mod kernel;
pub mod misc;
pub use prelude::*;


pub const KERNEL_PATH: &str = "\\oxy\\kernel.elf";
use crate::{kernel::BootInfo};

#[entry]
fn main() -> Status {


    println!("Hello world!");

    boot::stall(100_000_000);


    uefi::helpers::init().expect("failed to initialize helpers");

    let mut boot_info = match BootInfo::collect() {
        Ok(bi) => bi,
        Err(msg) => {
            uefi::println!("failed to get bootinfo{msg}");
            panic!()
        }
    };


    let kernel = match kernel::load() {
        Ok(slice) => slice,
        Err(e) => panic!("failed to load kernel: {e}"),
    };


    let prepared_kernel = match kernel::prepare(kernel) {
        Ok(p) => p,
        Err(msg) => {
            println!("kernel preparation failed: {msg}");
            panic!();
        }
    };

    uefi::println!("kernel prepared");

    match kernel::setup_paging(&prepared_kernel, &mut boot_info) {
        Ok(p) => p,
        Err(e) => {
            uefi::println!("failed to setup paging: {e}");
            panic!();
        }
    };

    uefi::println!("paging set");

    for i in (1..5).rev() {
        uefi::println!("booting the kernel in {i}");
        boot::stall(Duration::from_secs_f32(0.75f32).as_micros() as usize);
    }

    
    kernel::switch_to_kernel(prepared_kernel, boot_info);
    
}


pub enum Either<A: Sized, B: Sized> {
    One(A),
    Two(B),
}