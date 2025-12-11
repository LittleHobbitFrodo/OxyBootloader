
use core::fmt::{self, Write};

use toml::{self, Deserializer};

use crate::{FileTypeCheck, String};

use allocator_api2::vec::Vec;
use serde::de::{self, Visitor};
use toml::Value;
use crate::{print, println};

use crate::fs::exists_and_is_file;


/// Structure representing the configuration loaded from the config file
#[derive(Debug)]
pub struct Config {
    /// Delay in microseconds
    delay: Option<u64>,
    /// Path to the kernel
    kernel: Option<String>,
    /// Kernel parameters
    params: Option<String>,
}

impl Config {

    /// Constructor
    pub const fn new() -> Self {
        Self {
            delay: None,
            kernel: None,
            params: None,
        }
    }

    /// Returns the `delay` filed
    #[inline(always)]
    pub fn delay(&self) -> Option<u64> { self.delay }


    /// Returns the path to the kernel
    /// - if `None` is returned, the value have not been set by the config file
    #[inline(always)]
    pub fn kernel_path(&self) -> Option<&String> {
        match self.kernel {
            Some(ref path) => Some(&path),
            None => None
        }
    }

    /// Returns kernel parameters
    /// - if `None` is returned, the value has not been set by the config file
    #[inline(always)]
    pub fn kernel_params(&self) -> Option<&String> {
        match self.params {
            Some(ref params) => Some(params),
            None => None
        }
    }


    /// Parses toml configuration from string and returns message for the user if fails
    #[inline(never)]
    pub fn parse(sfs: &mut crate::fs::Sfs, string: &str) -> Result<Self, Option<String>> {

        let mut config = Config::new();

        let cfg: Value = match toml::from_str(string) {
            Ok(v) => v,
            Err(e) => return Err(Some(String::from(e.message())))
        };

        match cfg {
            toml::Value::Table(t) => {
                let iter = t.into_iter();

                for (i, (name, value)) in iter.enumerate() {

                    match name.as_str() {
                        "delay" => {
                            if config.delay.is_some() {
                                return Err(Some(String::from("field `delay` is set twice")))
                            }
                            if let Value::Integer(i) = value {
                                config.delay = Some((i as u64) * 1_000_000)
                            } else {
                                return Err(Some(String::from("field \"delay\" expects integer, got ")
                                    + value_to_string(&value)))
                            }
                        },
                        "kernel" => {

                            if config.kernel.is_some() {
                                return Err(Some(String::from("field `kernel` is set twice")))
                            }

                            if let Value::String(s) = value {
                                
                                match exists_and_is_file(sfs, s.as_str()) {
                                    FileTypeCheck::Ok => {
                                        config.kernel = Some(s.clone())
                                    },
                                    FileTypeCheck::DoesNotExist => {
                                        return Err(Some(String::from("file does not exist")))
                                    },
                                    FileTypeCheck::WrongType => {
                                        return Err(Some(String::from("Wrong type of file")))
                                    }
                                }
                            } else {
                                return Err(Some(String::from("field \"kernel\" expects path (text), got ")
                                + value_to_string(&value)))
                            }
                        },
                        "params" => {
                            if config.params.is_some() {
                                return Err(Some(String::from("field `params` is set twice")))
                            }

                            if let Value::String(s) = value {
                                config.params = Some(s.clone())
                            } else {
                                return Err(Some(String::from("field \"params\" expects text, got ")
                                + value_to_string(&value)))
                            }
                        },
                        _ => {
                            let mut msg = String::new();
                            _ = write!(&mut msg, "unknown field \"{name}\"",);
                            return Err(Some(msg))
                        }
                    }
                }
            },
            _ => return Err(None),
        }

        Ok(config)

    }


}

fn value_to_string(val: &Value) -> &'static str {
    match *val {
        Value::Array(_) => "array",
        Value::Boolean(_) => "bool",
        Value::Datetime(_) => "date and time",
        Value::Float(_) => "decimal",
        Value::Integer(_) => "integral",
        Value::String(_) => "string (text)",
        Value::Table(_) => "structure"
    }
}


