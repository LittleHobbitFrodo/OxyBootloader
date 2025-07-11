
use uefi::prelude::*;
use uefi::proto::media::fs::SimpleFileSystem;
use uefi::proto::media::file::{File, FileMode, FileAttribute, FileType};
use uefi::CStr16;
use crate::{print, println};

pub fn list_root() {
    let mut sfs = boot::get_image_file_system(boot::image_handle()).expect("failed to get image filesystem");

    let mut root = sfs.get_mut().expect("failed to get root directory").open_volume().unwrap();

    let mut path_buf = [0u16; 64];
    let path = CStr16::from_str_with_buf("\\EFI\\BOOT", &mut path_buf).unwrap();

    let dir_handle = root.open(path, FileMode::Read, FileAttribute::empty()).unwrap();

    match dir_handle.into_type().unwrap() {
        FileType::Dir(mut dir) => {
            let mut buffer = [0u8; 512];
            loop {
                match dir.read_entry(&mut buffer) {
                    Ok(Some(info)) => {
                        let name = info.file_name();

                        if info.attribute().contains(FileAttribute::DIRECTORY) {
                            print!("dir: ", );
                        }
                        println!("{}", name);
                    }
                    Ok(None) => break, // End of directory
                    Err(e) => {
                        println!("Error reading directory: {:?}", e);
                        break;
                    }
                }
            }
        },
        _ => println!("regularclear
        "),
    }

}