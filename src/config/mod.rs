use uefi::boot;
use uefi::println;
use uefi::CStr16;
use uefi::proto::media::file::{File, FileMode, FileAttribute, FileType};

mod config_holder;
pub use config_holder::Config;
use uefi::Status;
use crate::Sfs;
use crate::String;
use crate::fs::read_file;

pub const CONFIG_PATH: &'static str = "\\oxy\\oxyboot.toml";
/// Variable storing the configuration


pub(crate) enum UnexpectedErrorData {
    Status(uefi::Status),
    Str(String),
}

impl Clone for UnexpectedErrorData {
    fn clone(&self) -> Self {
        match self {
            Self::Status(s) => Self::Status(s.clone()),
            Self::Str(s) => Self::Str(s.clone()),
        }
    }
}



pub enum ReadError {
    DoesNotExist(Option<String>),
    NotFound(uefi::Status),
    InvalidName(Option<String>),
    UnexpectedError(Option<UnexpectedErrorData>),
    FailedToReadConfig(Option<String>),
    ParserError(Option<String>),
}

impl core::fmt::Display for ReadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::DoesNotExist(o) => {
                if let Some(s) = o {
                    write!(f, "Does not exist : {s}")
                } else {
                    write!(f, "Does not exist : unknown error")
                }
            },
            Self::NotFound(status) => write!(f, "Not found : {status}"),
            Self::InvalidName(o) => {
                if let Some(s) = o {
                    write!(f, "Invalid name : {s}")
                } else {
                    write!(f, "Invalid name : unknown error")
                }
            },
            Self::UnexpectedError(o) => {
                if let Some(data) = o {
                    match data {
                        UnexpectedErrorData::Status(status) => write!(f, "Unexpected error: {status}"),
                        UnexpectedErrorData::Str(string) => write!(f, "Unexpected error: {string}"),
                    }
                } else {
                    write!(f, "Unexpected error : unknown error")
                }
            },
            Self::FailedToReadConfig(o) => {
                if let Some(s) = o {
                    write!(f, "Failed to read config : {s}")
                } else {
                    write!(f, "Failed to read config : unknown error")
                }
            },
            Self::ParserError(o) => {
                if let Some(s) = o {
                    write!(f, "Failed to parse config: {s}")
                } else {
                    write!(f, "Failed to parse config : unknown error")
                }
            }
        }
    }
}


pub fn load() -> Result<Config, ReadError> {


    //  Open the SFS protocol

    let mut sfs = if let Ok(s) = boot::get_image_file_system(boot::image_handle()) {
        s
    } else {
        return Err(ReadError::UnexpectedError(None))
    };


    //  Read the config file

    let string = match read_file(&mut sfs, CONFIG_PATH) {
        Ok(s) => s,
        Err(e) => {
            println!("failed to read config: {e}");
            panic!();
        }
    };


    //  Parse the configuration file

    let config = match Config::parse(&mut sfs, string.as_str()) {
        Ok(cfg) => cfg,
        Err(o) => return if let Some(msg) = o {
            Err(ReadError::ParserError(Some(msg)))
        } else {
            Err(ReadError::ParserError(None))
        },
    };


    Ok(config)
}
