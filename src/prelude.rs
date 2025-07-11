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

pub use bitflags::*;
pub use spin::Mutex;


pub use crate::memmap::MemmapEntry;

pub use allocator_api2 as alloc;


#[global_allocator]
static ALLOCATOR: uefi::allocator::Allocator = uefi::allocator::Allocator;