use allocator_api2::boxed::Box;
use elf_loader::object::{ElfBinary, ElfObject};
use uefi::boot::{MemoryType, ScopedProtocol, PAGE_SIZE};
use uefi::prelude::*;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::proto::media::file::{File, FileAttribute, FileInfo, FileMode, FileType};
use uefi::CStr16;
use crate::config::Config;
use crate::{print, println, String, config::ReadError};
use core::fmt::Write;
use core::ptr::NonNull;
use allocator_api2::vec;

use elf_loader::{Elf, ElfExec};

pub enum FileTypeCheck {
    DoesNotExist,
    WrongType,
    Ok
}


pub type Sfs = ScopedProtocol<SimpleFileSystem>;

pub fn file_exists(file: &str) -> bool {
    let mut sfs = boot::get_image_file_system(boot::image_handle()).expect("failed to get image filesystem");

    let mut root = sfs.get_mut().expect("failed to get root directory").open_volume().unwrap();

    let mut path_buf = [0u16; 64];
    let path = CStr16::from_str_with_buf(file, &mut path_buf).unwrap();


    match root.open(path, FileMode::Read, FileAttribute::empty()) {
        Ok(_) => true,
        Err(_) => false,
    }

}

pub fn is_file(file: &str) -> bool {
    let mut sfs = boot::get_image_file_system(boot::image_handle()).expect("failed to get image filesystem");

    let mut root = sfs.get_mut().expect("failed to get root directory").open_volume().unwrap();

    let mut path_buf = [0u16; 64];
    let path = CStr16::from_str_with_buf(file, &mut path_buf).unwrap();


    let file = match root.open(path, FileMode::Read, FileAttribute::empty()) {
        Ok(f) => f,
        Err(_) => return false,
    };

    match file.into_type().expect("critical: file.into_type()") {
        FileType::Regular(_) => true,
        _ => false,
    }
    


}

pub fn exists_and_is_file(sfs: &mut Sfs, path: &str) -> FileTypeCheck {

    let mut root = sfs.get_mut().expect("failed to get root directory").open_volume().unwrap();

    let mut path_buf = [0u16; 64];
    let path = CStr16::from_str_with_buf(path, &mut path_buf).unwrap();

    let file = match root.open(path, FileMode::Read, FileAttribute::empty()) {
        Ok(f) => f,
        Err(_) => return FileTypeCheck::DoesNotExist,
    };

    match file.into_type().expect("critical: file.into_type()") {
        FileType::Regular(_) => FileTypeCheck::Ok,
        _ => FileTypeCheck::WrongType,
    }

}

pub fn exists_and_is_dir(path: &str) -> FileTypeCheck {
    let mut sfs = boot::get_image_file_system(boot::image_handle()).expect("failed to get image filesystem");

    let mut root = sfs.get_mut().expect("failed to get root directory").open_volume().unwrap();

    let mut path_buf = [0u16; 64];
    let path = CStr16::from_str_with_buf(path, &mut path_buf).unwrap();


    let file = match root.open(path, FileMode::Read, FileAttribute::empty()) {
        Ok(f) => f,
        Err(_) => return FileTypeCheck::DoesNotExist,
    };

    match file.into_type().expect("critical: file.into_type()") {
        FileType::Dir(_) => FileTypeCheck::Ok,
        _ => FileTypeCheck::WrongType,
    }

}

pub fn read_file(sfs: &mut Sfs, path: &str) -> Result<String, ReadError> {

    //  Open volume

    let mut root = sfs.get_mut().expect("failed to get root directory").open_volume().unwrap();

    //  prepare path

    let mut path_buf = [0u16; 64];
    let path = CStr16::from_str_with_buf(path, &mut path_buf).unwrap();

    //  open handle to the file

    let file_handle = match root.open(path, FileMode::Read, FileAttribute::empty()) {
        Ok(f) => f,
        Err(_) => {
            let mut msg = String::with_capacity(64);
            _ = write!(&mut msg, "file \"{path}\" does not exist");
            return Err(ReadError::DoesNotExist(Some(msg)))
        },
    };

    //  checkfile type

    let mut file = match file_handle.into_type().expect("critical: file.into_type()") {
        FileType::Regular(file) => file,
        _ => {
            let mut msg = String::with_capacity(64);
            _ = write!(&mut msg, "file \"{path}\" is not a regular file -> cannot read");
            return Err(ReadError::FailedToReadConfig(Some(msg)))
        },
    };

    //  read data from the file

    let mut buffer = [0u8; 4096];

    let read_size = match file.read(&mut buffer) {
        Ok(size) => size,
        Err(_) => return Err(ReadError::FailedToReadConfig(None)),
    };

    //  returns file content

    let s = unsafe { core::str::from_utf8_unchecked(&buffer[0..read_size]) };

    Ok(String::from(s))

}

#[inline(never)]
pub fn load_kernel(config: &Config) -> Result</*KernelData*/(NonNull<u8>, usize), Option<String>> {

    //  Open SimpleFileSystem protocol

    let mut sfs = match boot::get_image_file_system(boot::image_handle()) {
        Ok(sfs) => sfs,
        Err(_) => return Err(Some(String::from("failed to open SFS")))
    };

    //  Open volume

    /*let root = match sfs.get_mut() {
        Some(sfs) => sfs,
        None => return Err(Some(String::from("failed to open sfs")))
    };*/

    let mut root = match sfs.open_volume() {
        Ok(vol) => vol,
        Err(_) => return Err(Some(String::from("failed to open root volume"))),
    };



    //  Prepare path

    let mut path_buf = [0u16; 64];

    let path = match config.kernel_path() {
        Some(p) => p,
        None => return Err(Some(String::from("kernel path is not set, please specify it in the config"))),
    };

    let path = CStr16::from_str_with_buf(path, &mut path_buf).unwrap();



    //  Open kernel file handle

    let file_handle = match root.open(path, FileMode::Read, FileAttribute::empty()) {
        Ok(fh) => fh,
        Err(e) => return Err(Some(String::from("failed to open handle for kernel"))),
    };


    //  Open kernel file

    let mut file = match file_handle.into_regular_file() {
        Some(f) => f,
        None => return Err(Some(String::from("kernel is not a regular file"))),
    };



    //  Get size of the kernel file

    let mut buf= [0u8; 1024];

    let mut file_size = match file.get_info::<FileInfo>(&mut buf) {
        Ok(info) => info.file_size(),
        Err(mut e) => {

            //  Try again with bigger buffer
            if let Some(size) = e.data() {
                let mut buf = vec![0u8; *size];

                if let Ok(data) = file.get_info::<FileInfo>(&mut buf) {
                    data.file_size()
                } else {
                    return Err(Some(String::from("failed to read kernel metadata")));
                }

            } else {
                return Err(Some(String::from("failed to read kernel metadata")));
            }

        },
    } as usize;

    let _ = buf;


    //  Allocate pages for to load kernel

        //  align to pages
    file_size = (file_size + PAGE_SIZE - 1) / PAGE_SIZE;

    let mut kernel = match boot::allocate_pages(boot::AllocateType::AnyPages, MemoryType::LOADER_DATA, file_size) {
        Ok(ptr) => ptr,
        Err(_) => return Err(Some(String::from("failed to allocate pages for kernel")))
    };

    let mut buf = unsafe { core::slice::from_raw_parts_mut(kernel.as_mut(), file_size * PAGE_SIZE) };

    
    if let Err(e) = file.read(&mut buf) {
        return Err(Some(String::from("failed to read kernel data")));
    }

    let elf = ElfBinary::new("kernel", &buf);

    //ElfExec::relocate(self, scope, pre_find, deal_unknown, local_lazy_scope)
    


    //let elf = ElfExec::relocate(self, scope, pre_find, deal_unknown, local_lazy_scope)

    //let elf = ElfExec::new(image).map_err(|_| "Invalid ELF")?;

    //Ok(KernelData::new(kernel, file_size))

    Ok((kernel, file_size))


}