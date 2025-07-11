#![no_main]
#![no_std]

/*use uefi::boot::{self, SearchType};

use uefi::proto::loaded_image::LoadedImage;
use uefi::{Identify, Result};
use uefi::proto::device_path::text::{
    AllowShortcuts, DevicePathToText, DisplayOnly,
};*/

pub mod utils;
pub mod memmap;
pub mod prelude;
pub use prelude::*;
pub mod fs;

use crate::memmap::MEMMAP;


#[entry]
fn main() -> Status {

    uefi::helpers::init().unwrap();

    println!("Hello world!\n\n");

    memmap::parse();

    list_root();


    boot::stall(10_000_000);
    Status::SUCCESS
}


