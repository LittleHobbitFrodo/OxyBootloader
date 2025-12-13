//! Loads and parses the kernel

use core::ops::BitOr;
use core::{arch::asm, slice};
use core::ptr::NonNull;

use crate::config::Config;
use allocator_api2::{boxed::Box, vec::Vec};
use bitflags::{Flags, bitflags};
use goblin::{elf::{Elf, ProgramHeader}, error::Error};
use uefi::{CStr16, Status, allocator, boot::{self, AllocateType, MemoryAttribute, MemoryType, allocate_pages}, proto::media::file::{File, FileAttribute, FileInfo, FileMode}};
use x86_64::structures::paging::{PageTable, PageTableFlags};
use x86_64::structures::paging::page_table::{PageTableEntry, PageTableLevel};
use crate::{Either, String};

use x86_64::{VirtAddr, registers::control::Cr3};

//use uefi::boot::MemoryDescriptor

//static PLM4: RwLock<>

/// Loads the kernel into `Box`
pub fn load(config: &Config) -> Result<Box<[u8]>, &'static str> {

    let kernel_path = match config.kernel_path() {
        Some(path) => path,
        None => return Err("no path to the kernel".into())
    };

    let mut sfs = match boot::get_image_file_system(boot::image_handle()) {
        Ok(sfs) => sfs,
        Err(_) => return Err("cannot open simple filesystem protocol")
    };

    let mut root = match sfs.open_volume() {
        Ok(r) => r,
        Err(_) => return Err("cannot open the volume")
    };

    let mut name_buf = [0u16; 256];

    let filename = match CStr16::from_str_with_buf(kernel_path.as_str().trim_end_matches('\0'), &mut name_buf) {
        Ok(name) => name,
        Err(_) => return Err("failed to convert string into UTF16"),
    };

    let handle = match root.open(filename, FileMode::Read, FileAttribute::empty()) {
        Ok(h) => h,
        Err(_) => return Err("cannot open kernel file")
    };

    let mut file = match handle.into_regular_file() {
        Some(f) => f,
        None => return Err("path is not pointing to regular file")
    };

    let mut info_buf = [0u8; 512];

    let info: &mut FileInfo = match file.get_info(&mut info_buf) {
        Ok(info) => info,
        Err(_) => return Err("cannot get file info")
    };
    let file_len = info.file_size() as usize;

    let mut loaded: Box<[u8]> = unsafe { Box::new_zeroed_slice(file_len).assume_init() };

    if let Err(_) = file.read(loaded.as_mut()) {
        Err("failed to load the file")
    } else {
        Ok(loaded)
    }

}



/// Parses the kernel and allocates memory for it
pub fn prepare(kernel: Box<[u8]>) -> Result<(Vec<AllocatedPage>, extern "C" fn() -> !), String> {

    let elf = match Elf::parse(kernel.as_ref()) {
        Ok(elf) => elf,
        Err(e) => {
            let mut msg = String::from("failed to parse kernel: ");
            match e {
                Error::BadMagic(_) => {
                    msg.push_str("bad magic number");
                }
                Error::BufferTooShort(_, m) => {
                    msg.push_str("buffer too short: ");
                    msg.push_str(m);
                },
                Error::IO(_) => {
                    msg.push_str("unknown IO error");
                },
                Error::Malformed(m) => {
                    msg.push_str("malformed: ");
                    msg.push_str(m.as_str());
                },
                Error::Scroll(_) => msg.push_str("scroll error"),
                _ => msg.push_str("unknown error"),
            }
            return Err(msg)
        }
    };

    let mut pages = Vec::new();


    for header in elf.program_headers.iter() {
        pages.push(match prepare_segment(&kernel, header) {
            Ok(page) => page,
            Err(e) => {
                let mut msg = String::from("failed to prepare segment: ");
                msg.push_str(e.as_str());
                return Err(msg)
            },
        });

    }

    let entry = unsafe { core::mem::transmute(elf.entry as usize) };

    Ok((pages, entry))

}

/// Points out the difference between regular page and executable page
/// - executable pages has to be marked executable manually
#[derive(Debug)]
pub enum AllocatedPage {
    /// Pages that needs to be marked as executable
    Executable(Page),
    /// Regular (RW) pages
    Regular(Page)
}

#[derive(Debug)]
pub struct Page {
    /// Virtual address to the page
    pub address: NonNull<u8>,
    /// Page count
    pub count: usize,
}

/// Allocates pages on correct location in memory and returns the `AllocatedPage` enum indicating if the memory has to be marked as executable manually
fn prepare_segment(kernel: &'_ Box<[u8]>, header: &'_ ProgramHeader) -> Result<AllocatedPage, String> {

    let range = header.vm_range();
    let count = ((range.end - range.start) as usize / 4096) + 1;

    //let ptr = match allocate_pages(boot::AllocateType::Address(range.start as u64), MemoryType::LOADER_DATA, count) {
    let ptr = match allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, count) {
        Ok(ptr) => ptr,
        Err(e) => {
            const OUT_OF_RESOURCES: usize = Status::OUT_OF_RESOURCES.0;
            const INVALID_PARAMETER: usize = Status::INVALID_PARAMETER.0;
            const UNACCEPTED: usize = MemoryType::UNACCEPTED.0 as usize;
            const NOT_FOUND: usize = Status::NOT_FOUND.0;

            let mut msg = String::from("failed to allocate pages: ");
            msg.push_str(match e.status().0 {
                OUT_OF_RESOURCES => "out of resources",
                INVALID_PARAMETER => "invalid parameter",
                UNACCEPTED => "unaccepted memory",
                NOT_FOUND => "not found",
                _ => "unknown error"
            });
            return Err(msg)
        }
    };

    let frange = {
        let r = header.file_range();
        //uefi::println!("\tf: range({} => {}) : {}", r.start, r.end, kernel.len());
        //uefi::println!("\tp_filesz({}),\tp_offset({})", header.p_filesz, header.p_offset);
        unsafe { slice::from_raw_parts(kernel.as_ptr(), r.end - r.start) }
    };

    unsafe {
        ptr.copy_from_nonoverlapping(NonNull::new_unchecked(frange.as_ptr() as *mut u8), frange.len());
    }

    if header.is_executable() {
        Ok(AllocatedPage::Executable(Page { address: ptr, count }))
    } else {
        Ok(AllocatedPage::Regular(Page { address: ptr, count }))
    }
    
    //Ok(allocate_pages(boot::AllocateType::Address(range.start as u64), mem_type, count).map_err(|_| () )?)
}


/// Sets the exec bit on for this virtual address
/// - boot services must be exitted
pub fn make_executable(page: &Page) -> Option<PageTableEntry> {

    
    let virt = VirtAddr::new((page.address.as_ptr() as usize) as u64);
    
    let get_index = |idx: i32| -> u64 {
        assert!(idx < 4 && idx >= 0);

        unsafe { core::hint::assert_unchecked(idx < 4 && idx >= 0) }

        match idx {
            0 => virt.p1_index().into(),
            1 => virt.p2_index().into(),
            2 => virt.p3_index().into(),
            3 => virt.p4_index().into(),
            _ => unreachable!(),
        }
    };

    //crate::println!("virt: {virt:?}");

    //  get address to the PML4 table
    let mut table = unsafe {
        let ptr: u64;
        asm!(
            "mov {}, cr3",
            out(reg) ptr,
            options(nomem, nostack, preserves_flags)
        );
        NonNull::new_unchecked(ptr as *mut PageTable).as_mut()
    };

    for level in (1..4).rev() {

        let index = get_index(level);

        //crate::println!("\tlevel {level}: index = {index}");

        let mut entry = match table.iter_mut().nth(index as usize) {
            Some(e) => e,
            None => {
                crate::println!("failed to index page table with {index}");
                panic!();
            }
        };

        if entry.flags().bits() & PageTableFlags::HUGE_PAGE.bits() != 0 {
            //crate::println!("\t{entry:?}");
            //crate::println!("\t\tbits disabled: {entry:?}");
            disable_bits(entry);
            return Some(entry.clone())
        }

        if entry.addr().is_null() {
            crate::println!("page address is null");
            panic!();
        }

        table = unsafe { ((entry.addr().as_u64() as usize) as *mut PageTable).as_mut().unwrap() };
    }

    //crate::println!("disabling single page bits");
    disable_bits(table.iter_mut().nth(get_index(0) as usize).unwrap());

    //crate::println!("done");

    return None;

    /// Disables `NO_EXECUTE` and `WRITEABLE` bits
    fn disable_bits(entry: &mut PageTableEntry) {
        //entry.set_flags(entry.flags().difference(PageTableFlags::NO_EXECUTE.bitor(PageTableFlags::WRITABLE)));
        entry.flags().remove(PageTableFlags::NO_EXECUTE);
        entry.flags().remove(PageTableFlags::WRITABLE);
    }

}

#[repr(C, packed)]
pub struct PageEntry { inner: u64 }

impl PageEntry {

    /// The present bit: the page is in memory
    pub const PRESENT: u64 = 0b1;
    /// Writing into the page is allowed
    pub const WRITE: u64 = 0b10;

    pub const unsafe fn new(value: u64) -> Self {
        Self { inner: value }
    }

    /// Checks if all bits are set
    pub fn check_bits(&self, bits: u64) -> bool { self.inner & bits == bits }

    /// Indicates whether the page is in memory
    #[inline]
    pub fn present(&self) -> bool { self.check_bits(Self::PRESENT) }

    /// Writing into the page is allowed
    #[inline]
    pub fn write(&self) -> bool { self.inner & Self::WRITE == Self::WRITE }

    


}