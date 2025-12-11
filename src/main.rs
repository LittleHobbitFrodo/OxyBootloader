#![no_main]
#![no_std]

pub mod utils;
pub mod memmap;
pub mod prelude;
pub use prelude::*;
use uefi::{boot::exit_boot_services, mem::memory_map::MemoryMapMut, proto::console::text::Output};
pub mod fs;
pub mod config;

//use oxyboot_requests::{KernelEntryRequest, Request};
//use uefi::{boot::{exit_boot_services, load_image, start_image}, proto::media::load_file};

pub mod kernel;
pub mod misc;
use misc::*;


//  TODO: rewrite the config module

#[entry]
fn main() -> Status {

    uefi::helpers::init().unwrap();

    uefi::println!("Hello world!\n\n");

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

    let pages = match kernel::prepare(kernel) {
        Ok(pages) => pages,
        Err(msg) => {
            uefi::println!("failed to prepare kernel: {msg}");
            panic!();
        },
    };

    uefi::println!("kernel prepared");

    let mut fb = match get_framebuffer() {
        Ok(fb) => fb,
        Err(e) => {
            uefi::println!("failed to get framebuffer: {e}");
            panic!();
        }
    };
    //println!(fb: "hello world!");

    //boot::stall(100_000_000);


    let mut memmap = unsafe { exit_boot_services(None) };
    memmap.sort();

    /*for page in pages.iter() {
        if let AllocatedPage::Executable(p) = page {
            if let Err(_) = kernel::make_executable(p) {
                panic!();   //  cannot print text yet
            }
        }
    }*/
    

    boot::stall(100_000_000);
    Status::SUCCESS
}

/*#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    loop {}
}*/