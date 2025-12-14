//! Loads and parses the kernel

use core::fmt::Debug;
//use core::{arch::asm, slice};
use core::ptr::NonNull;

use crate::config::Config;
use crate::misc::KernelEntry;
use allocator_api2::{boxed::Box, vec::Vec};
use goblin::{elf::Elf, error::Error};
use uefi::{CStr16, boot::{self, AllocateType, MemoryType, allocate_pages}, proto::media::file::{File, FileAttribute, FileInfo, FileMode}};
use x86_64::{PhysAddr, VirtAddr, structures::paging::{PageTable, PageTableFlags, PhysFrame}};
use crate::String;

use goblin::elf::program_header;

//use x86_64::VirtAddr;

mod boot_info;
pub use boot_info::*;


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


pub fn prepare(kernel: Box<[u8]>) -> Result<MetaData, String> {

    //  let goblin parse the elf file
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

    let mut sections = Vec::new();

    for header in elf.program_headers.iter() {

        if header.p_type != program_header::PT_LOAD {
            //  not loadable segment
            continue;
        }

        let page_count = ((header.p_memsz / 4096) + 1) as usize;

        let copy_size = header.p_filesz as usize;
        let file_offset = header.p_offset as usize;

        let perms = if header.is_executable() {
            Permissions::Executable
        } else if header.is_write() {
            Permissions::Writeable
        } else {
            Permissions::ReadOnly
        };

        //  allocate pages
        let pages = match allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, page_count) {
            Ok(pg) => pg,
            Err(e) => {
                use core::fmt::Write;
                let mut msg = String::new();
                _ = write!(&mut msg, "{e}");
                return Err(msg);
            }
        };

        let file_ptr = NonNull::new(kernel.as_ptr() as *mut u8).unwrap();

        unsafe {
            //  copy segment data from the file into the allocated memory
            pages.copy_from_nonoverlapping(file_ptr.add(file_offset), copy_size);

            //  set any other data to zero
            pages.add(copy_size).write_bytes(0, (page_count as usize * 4096) - copy_size);
        }

        sections.push(Section {
            address: NonNull::new(header.vm_range().start as *mut u8).unwrap(),
            phys: pages.as_ptr() as usize,
            page_count,
            perms: perms
        });

    }


    let entry: KernelEntry = unsafe { core::mem::transmute(elf.entry as usize) };
    Ok(MetaData { sections, entry})

}


pub type Page = [u8; 4096];


/// Holds metadata about the kernel loaded
#[derive(Debug)]
pub struct MetaData {
    /// Marks each loaded progrm header
    pub sections: Vec<Section>,
    pub entry: KernelEntry
}

/// Holds data for one kernel section
pub struct Section {
    /// Marks the start of the virtual address space
    pub address: NonNull<u8>,
    /// Starting address of the physical address space
    pub phys: usize,
    /// How many pages does this section take
    pub page_count: usize,
    /// Permissions for this section
    pub perms: Permissions
}

impl Debug for Section {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Section {{ a: {:p}, phys: {:p}, p_count: {}, {:?} }}",
            self.address,
            self.phys as *const u8,
            self.page_count,
            self.perms)
    }
}

/// Describes MMU permissions for each section of the kernel
pub enum Permissions {
    /// Only read permissions (not executable)
    ReadOnly,
    /// Read + write permissions
    Writeable,
    /// Only readable and executable
    Executable,
}

impl Permissions {
    /// Always sets the `PRESENT` flag on
    pub fn into_page_flags(&self) -> PageTableFlags {
        match self {
            Self::Executable => PageTableFlags::PRESENT,
            Self::ReadOnly => PageTableFlags::NO_EXECUTE | PageTableFlags::PRESENT,
            Self::Writeable => PageTableFlags::NO_EXECUTE | PageTableFlags::WRITABLE | PageTableFlags::PRESENT,
        }
    }
}

impl Debug for Permissions {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Executable => write!(f, "r-x"),
            Self::ReadOnly => write!(f, "r--"),
            Self::Writeable => write!(f, "rw-"),
        }
    }
}


/// Sets up paging for loaded kernel
/// - returns pointer to the `PML4` table
/// - no huge pages are used
pub fn setup_paging(meta: &MetaData) -> Result<NonNull<u8>, &'static str> {

    let mut pml4 = allocate_table()?;

    for section in &meta.sections {

        let perms = section.perms.into_page_flags();
        let virt = VirtAddr::from_ptr(section.address.as_ptr());


        let mut current = unsafe { pml4.as_mut() };
        


        //  pml4 -> pdpt -> pd
        for level in (1..4).rev() {

            let index = level_index(virt, level);
            uefi::println!("level {level}: index = {index}");

            let entry = current.iter_mut().nth(index).unwrap();

            if entry.is_unused() {  //  allocate the next level
                uefi::println!("    allocating new table");
                let mut table = allocate_table()?;
                let phys = PhysAddr::new((table.as_ptr() as usize) as u64);

                entry.set_addr(phys, PageTableFlags::PRESENT | PageTableFlags::WRITABLE);

                current = unsafe { table.as_mut() };
                continue

            } else {
                current = match NonNull::new((entry.addr().as_u64() as usize) as *mut PageTable) {
                    Some(mut c) => unsafe { c.as_mut() },
                    None => return Err("page entry is null (unexpected)"),
                };
            }
        }


        uefi::println!("pt = {:p}", section.address);
        //  current = pt (last) level
        let entry = match current.iter_mut().nth(virt.p1_index().into()) {
            Some(ent) => ent,
            None => {
                let index: usize = virt.p1_index().into();
                uefi::println!("failed to index entry with {index}");
                panic!();
            }
        };
        //let entry = current.iter_mut().nth(virt.p1_index().into()).unwrap();

        let frame = match PhysFrame::from_start_address(PhysAddr::new((section.address.as_ptr() as usize) as u64)) {
            Ok(f) => f,
            Err(_) => return Err("virtual address is not aligned properly"),
        };

        entry.set_frame(frame, perms);

    }

    uefi::println!("returning");

    
    return Ok(pml4.cast());

    /// Allocates table and nulls its contents
    fn allocate_table() -> Result<NonNull<PageTable>, &'static str> {
        let mut table = match allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 1) {
            Ok(ptr) => ptr,
            Err(_) => return Err("failed to allocate table"),
        }.cast::<PageTable>();

        unsafe {
            table.as_mut().zero();
        }

        Ok(table)
    }

    /// Returns index for certain level of pages from given virtual address
    fn level_index(address: VirtAddr, level: i32) -> usize {
        match level {
            3 => address.p4_index().into(),
            2 => address.p3_index().into(),
            1 => address.p2_index().into(),
            0 => address.p1_index().into(),
            _ => {
                uefi::println!("level index: index {level} is out of bounds");
                panic!();
            }
        }
    }

    /*/// Tells if flags are sets
    fn has_flags(entry: &PageTableEntry, flags: PageTableFlags) -> bool {
        entry.flags().intersection(flags).bits() != 0
    }*/

}





/*/// Parses the kernel and allocates memory for it
pub fn prepare(kernel: Box<[u8]>) -> Result<(Vec<AllocatedPage>, KernelEntry), String> {



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

    uefi::println!("header p_paddr: {:p}", header.p_paddr as *const u8);
    let range = header.vm_range();
    let count = ((range.end - range.start) as usize / 4096) + 1;

    //let ptr = match allocate_pages(boot::AllocateType::Address(range.start as u64), MemoryType::LOADER_DATA, count) {
    let ptr = match allocate_pages(AllocateType::Address(header.p_paddr), MemoryType::LOADER_DATA, count) {
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

    uefi::print!("\tpage {ptr:p} ");

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
        uefi::println!("is executable");
        Ok(AllocatedPage::Executable(Page { address: ptr, count }))
    } else {
        uefi::println!("is reqular");
        Ok(AllocatedPage::Regular(Page { address: ptr, count }))
    }
    
    //Ok(allocate_pages(boot::AllocateType::Address(range.start as u64), mem_type, count).map_err(|_| () )?)
}


/// Sets the exec bit on for this virtual address
/// - boot services must be exitted
pub fn make_executable(page: &Page) {

    
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

        let entry = match table.iter_mut().nth(index as usize) {
            Some(e) => e,
            None => {
                crate::println!("failed to index page table with {index}");
                panic!();
            }
        };

        if entry.flags().bits() & PageTableFlags::HUGE_PAGE.bits() != 0 {
            disable_bits(entry);
            return
        }

        if entry.addr().is_null() {
            crate::println!("page address is null");
            panic!();
        }

        table = unsafe { ((entry.addr().as_u64() as usize) as *mut PageTable).as_mut().unwrap() };
    }

    disable_bits(table.iter_mut().nth(get_index(0) as usize).unwrap());

    /// Disables `NO_EXECUTE` and `WRITEABLE` bits
    fn disable_bits(entry: &mut PageTableEntry) {
        //entry.set_flags(entry.flags().difference(PageTableFlags::NO_EXECUTE.bitor(PageTableFlags::WRITABLE)));
        entry.flags().remove(PageTableFlags::NO_EXECUTE);
        entry.flags().remove(PageTableFlags::WRITABLE);
    }

}*/