use uefi::boot::MemoryType;
use uefi::boot::PAGE_SIZE;
use uefi::mem::memory_map::MemoryMap;
use uefi::mem::memory_map::MemoryMapMut;
pub use uefi::prelude::*;

pub use crate::memmap;
pub use crate::fs::*;


pub use uefi::print;
pub use uefi::println;

pub extern crate allocator_api2;
pub extern crate bitflags;
pub extern crate spin;
pub extern crate toml;
pub extern crate serde;
pub extern crate oxyboot_requests;
//pub extern crate elf_loader;
extern crate alloc;

pub use alloc::string::String;

//pub use elf_loader as elf;

pub use oxyboot_requests::*;

pub use bitflags::*;
pub use spin::Mutex;


pub use crate::memmap::MemmapEntry;



#[global_allocator]
static ALLOCATOR: uefi::allocator::Allocator = uefi::allocator::Allocator;


#[macro_export]
macro_rules! dbg {

    () => {
        $crate::eprintln!("[{}:{}:{}]", core::file!(), core::line!(), core::column!());
    };
    ($val:expr $(,)?) => {{


        let value = &$val;

        $crate::print!("[{}:{}:{}] {} = {:#?}", core::file!(), core::line!(), core::column!(), core::stringify!($val),
        &&value as &dyn core::fmt::Debug);
    }};
}