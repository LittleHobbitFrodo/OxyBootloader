use core::ptr::addr_of;

use allocator_api2::vec::Vec;
use uefi::boot::{MemoryDescriptor, MemoryType, PAGE_SIZE};
use spin::Mutex;
pub mod entry_type;
pub use entry_type::*;
use uefi::boot;

use uefi::mem::memory_map::MemoryMap;
use uefi::mem::memory_map::MemoryMapMut;

use uefi::{print, println};



pub static MEMMAP: Mutex<Vec<MemmapEntry>> = Mutex::new(Vec::new());





/// Describes one memory map entry
/// - size is in bytes
#[repr(C)]
#[derive(Copy, Clone)]
pub struct MemmapEntry {
    tp: MemmapEntryType,
    attr: u64,
    base: Base,
    size: u64,
}


impl MemmapEntry {

    /// Constructs `MemmapEntry` with no data
    /// - sets type to `Bad`
    pub const fn empty() -> Self {
        Self {
            tp: MemmapEntryType::Bad,
            attr: 0,
            base: Base::empty(),
            size: 0,
        }
    }

    /// Constructs new `MemmapEntry` with specified data
    pub const fn new(tp: MemmapEntryType, attr: u64, base: Base, size: u64) -> Self {
        Self {
            tp,
            attr,
            base,
            size,
        }
    }

    pub fn from_uefi(uefi: &MemoryDescriptor) -> Self {
        Self {
            tp: MemmapEntryType::from_uefi(uefi.ty),
            attr: uefi.att.bits(),
            base: Base::from_uefi(uefi),
            size: uefi.page_count * PAGE_SIZE as u64,
        }
    }

    pub const fn get_type(&self) -> MemmapEntryType { self.tp }
    pub const fn attributes(&self) -> u64 { self.attr }
    pub const fn start_physical(&self) -> u64 { self.base.phys() }
    pub const fn start_virtual(&self) -> *const u8 { self.base.virt() }
    pub const fn size(&self) -> u64 { self.size }


}

impl core::fmt::Display for MemmapEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:p} ({:p}) -> {} | {}", self.base.phys() as *const u8, self.base.virt(), self.size, self.tp)
    }
}



#[derive(Copy, Clone)]
#[repr(C)]
pub struct Base {
    phys: u64,
    virt: u64,
}

impl Base {
    /// Constructs `Base` with no data
    pub const fn empty() -> Self {
        Self { phys: 0, virt: 0, }
    }

    /// Constructs `Base` with specified values
    pub const fn new(phys: u64, virt: u64) -> Self {
        Self { phys, virt, }
    }

    /// Converts `uefi::MemoryDescriptor` to `Self`
    pub const fn from_uefi(uefi: &MemoryDescriptor) -> Self {
        Self { phys: uefi.phys_start, virt: uefi.virt_start }
    }

    /// Returns physical address of the entry start
    pub const fn phys(&self) -> u64 {
        self.phys
    }

    /// Returns the virtual address of the entry start
    pub const fn virt(&self) -> *const u8 {
        self.virt as *const u8
    }

}


/// Parses the uefi memory map (panics if fails)
pub fn parse() {

    let mut map = boot::memory_map(MemoryType::LOADER_DATA).expect("failed to get memory map");

    map.sort();

    let mut memmap = MEMMAP.try_lock().expect("MEMMAP variable is locked");

    memmap.push(MemmapEntry::from_uefi(unsafe { map.get(0).unwrap_unchecked() }));

    let mut entry: &MemoryDescriptor;
    let mut next: &MemoryDescriptor;

    for i in 0..map.meta().entry_count()-1 {

        entry = unsafe { map.get(i).unwrap_unchecked() };
        next = unsafe { map.get(i+1).unwrap_unchecked() };

        if MemmapEntryType::from_uefi(entry.ty) == MemmapEntryType::from_uefi(next.ty) {
            continue
        }

        let last = unsafe { memmap.last_mut().unwrap_unchecked() };

        last.size = entry.phys_start + (entry.page_count * PAGE_SIZE as u64) - last.start_physical();

        memmap.push(MemmapEntry::from_uefi(next));

    }

}
