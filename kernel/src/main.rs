
#![no_std]
#![no_main]

pub extern crate oxyboot_requests;
use core::{panic::PanicInfo, arch::asm};

use oxyboot_requests::*;



static KERNEL_ENTRY_REQUEST: KernelEntryRequest = KernelEntryRequest::new(0, _start);


#[unsafe(no_mangle)]
fn _start() {



}

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    hang();
}

pub fn hang() -> ! {
    unsafe {
        asm!("cli");
        loop {
            asm!("hlt");
        }
    }
}