//! Loads and parses the kernel

use core::{fmt::Debug, time::Duration};
use core::arch::asm;
use core::ptr::NonNull;

use crate::KERNEL_PATH;
use crate::misc::KernelEntry;
use allocator_api2::{boxed::Box, vec::Vec};
use goblin::{elf::Elf, error::Error};
use uefi::{CStr16, boot::{self, AllocateType, MemoryType, allocate_pages, exit_boot_services}, proto::media::file::{File, FileAttribute, FileInfo, FileMode}};
use x86_64::{PhysAddr, VirtAddr, registers::control::{Cr0, Cr0Flags}, structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB}};
use crate::String;

use goblin::elf::program_header;

//use x86_64::VirtAddr;

mod boot_info;
pub use boot_info::*;


/// Loads the kernel into `Box`
pub fn load() -> Result<Box<[u8]>, &'static str> {

    let mut sfs = match boot::get_image_file_system(boot::image_handle()) {
        Ok(sfs) => sfs,
        Err(_) => return Err("cannot open simple filesystem protocol")
    };

    let mut root = match sfs.open_volume() {
        Ok(r) => r,
        Err(_) => return Err("cannot open the volume")
    };

    let mut name_buf = [0u16; 256];

    let filename = match CStr16::from_str_with_buf(KERNEL_PATH.trim_end_matches('\0'), &mut name_buf) {
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


/// copies the data from the loaded ELF file, returns metadata for the `kernel::setup_paging()` function
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

        uefi::println!("preparing section {:p}", (header.p_vaddr as usize) as *const u8);

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


//pub type Page = [u8; 4096];


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


/*#[repr(transparent)]
pub struct Pml4 {
    physical: NonNull<u8>
}

impl core::fmt::Pointer for Pml4 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:p}", self.physical)
    }
}*/

/// Covers the prepared kernel in virtual address space
pub fn setup_paging(meta: &MetaData, info: &BootInfo) -> Result<(), &'static str> {

    //  makes it possible to write into write-protected PML4 table
    unsafe { Cr0::update(|flags| flags.remove(Cr0Flags::WRITE_PROTECT)); }

    let mut pml4 = unsafe {
        let mut cr3: u64 = 0;
        core::arch::asm!(
            "mov {}, cr3",
            out(reg) cr3,
        );
        NonNull::new_unchecked(((cr3 & !0xfff) as usize) as *mut PageTable)
    };

    let mut frame_alloc = FrameAlloc;
    let mut mapper = unsafe { OffsetPageTable::new(pml4.as_mut(), VirtAddr::new(0)) };


    //  map kernel sections
    for section in &meta.sections {
        unsafe {
            uefi::println!("mapping section {:p} ({:p})", section.address, meta.entry);
            if let Err(s) = map_kernel_section(&mut mapper, section, &mut frame_alloc) {
                uefi::println!("failed to map kernel section: {s}");
            }
        }
    }

    unsafe { Cr0::update(|flags| flags.insert(Cr0Flags::WRITE_PROTECT)); }

    return Ok(());


    struct FrameAlloc;

    unsafe impl FrameAllocator<Size4KiB> for FrameAlloc {
        fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
            let frame = allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, 1).ok()?;
            let phys = PhysAddr::new((frame.as_ptr() as usize) as u64);
            Some(PhysFrame::containing_address(phys))
        }
    }

}



pub unsafe fn map_kernel_section(mapper: &mut OffsetPageTable, section: &Section, frame_alloc: &mut impl FrameAllocator<Size4KiB>) -> Result<(), &'static str> {

    let mut virt = VirtAddr::new((section.address.as_ptr() as usize) as u64).align_down(4096u64);
    let mut frame: PhysFrame<Size4KiB> = PhysFrame::containing_address(PhysAddr::new(section.phys as u64).align_down(4096u64));
    let flags = section.perms.into_page_flags();

    for _ in 0..section.page_count {
        unsafe {
            //match mapper.map_to(Page::containing_address(virt), frame, flags, frame_alloc) {
            match mapper.map_to_with_table_flags(Page::containing_address(virt), frame, flags, PageTableFlags::WRITABLE | PageTableFlags::PRESENT, frame_alloc) {
                Ok(flush) => flush.ignore(),
                Err(_) => {
                    uefi::println!("failed to map kernel section");
                    panic!();
                }
            }
            virt += 4096;
            frame += 4096;
        };
    }


    Ok(())
}




/// Performs context switch to the kernel while giving it its own stack memory
/// - exits boot services
pub fn switch_to_kernel(kernel_meta: MetaData, info: BootInfo) -> ! {
    let stack = info.stack_top().as_ptr();
    let boot_info = Box::leak(Box::new(info)).as_ptr() as *mut BootInfo;

    _ = uefi::system::with_stdout(|x| x.clear() );

    let _ = unsafe { exit_boot_services(None) };

    unsafe {
        asm!(
            r#"
            cli
            mov rdi, {info}
            mov rsp, {stack}
            xor rbp, rbp
            call {entry}"#,
            entry = in(reg) kernel_meta.entry,
            info = in(reg) boot_info,
            stack = in(reg) stack
        );
    }

    unreachable!();

}