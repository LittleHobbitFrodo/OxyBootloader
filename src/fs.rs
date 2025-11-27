use allocator_api2::boxed::Box;
//use elf_loader::object::{ElfBinary, ElfObject};
use uefi::boot::{LoadImageSource, MemoryType, PAGE_SIZE, ScopedProtocol, load_image};
use uefi::prelude::*;
use uefi::proto::BootPolicy;
use uefi::proto::device_path::{self, DevicePath};
use uefi::proto::loaded_image::LoadedImage;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::proto::media::file::{File, FileAttribute, FileInfo, FileMode, FileType};
use uefi::CStr16;
use crate::config::Config;
use crate::{print, println, String, config::ReadError};
use core::fmt::Write;
use core::ptr::NonNull;
use core::slice;
use allocator_api2::vec;

//use elf_loader::{Elf, ElfExec};
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

pub fn load_kernel(config: Config) -> Result</*LoadedImage*/&'static mut [u8], String> {

    let kernel_path = match config.kernel_path() {
        Some(p) => p,
        None => return Err("no path given".into())
    };

    let mut sfs = match boot::get_image_file_system(boot::image_handle()) {
        Ok(fs) => fs,
        Err(_) => return Err("failed to open simple filesystem".into())
    };

    let mut root = match sfs.open_volume() {
        Ok(r) => r,
        Err(_) => return Err("failed to open the volume".into()),
    };

    let mut name_buf = [0u16; 256];

    assert!(kernel_path.len() < 256);
    let filename = CStr16::from_str_with_buf(kernel_path.as_str().trim_end_matches('\0'), &mut name_buf).expect("failed to convert string to utf16");

    let handle = match root.open(filename, FileMode::Read, FileAttribute::empty()) {
        Ok(h) => h,
        Err(_) => return Err("failed to open kernel".into())
    };

    let mut file = match handle.into_regular_file() {
        Some(f) => f,
        None => return Err("path is not pointing to file".into())
    };

    let mut info_buf = [0u8; 512];

    let info: &mut FileInfo = file.get_info(&mut info_buf).unwrap();
    let file_len = info.file_size() as usize;

    

    let (ptr, mut pages) = match boot::allocate_pages(boot::AllocateType::AnyPages, MemoryType::LOADER_DATA, (file_len / 4096) + 1) {
        Ok(ptr) => (ptr, unsafe { slice::from_raw_parts_mut(ptr.as_ptr(), file_len) }),
        Err(_) => return Err("failed to allocate pages".into())
    };

    unsafe { ptr.write_bytes(0, file_len); }

    file.read(&mut pages).expect("failed to read kernel file");

    Ok(pages)

}
