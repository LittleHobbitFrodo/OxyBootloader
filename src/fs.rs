use uefi::boot::ScopedProtocol;
use uefi::prelude::*;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::proto::media::file::{File, FileMode, FileAttribute, FileType};
use uefi::CStr16;
use crate::{print, println, String, config::ReadError};
use core::fmt::Write;

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