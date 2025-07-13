#![no_main]
#![no_std]

pub mod utils;
pub mod memmap;
pub mod prelude;
pub use prelude::*;
pub mod fs;
pub mod config;


#[entry]
fn main() -> Status {

    uefi::helpers::init().unwrap();

    println!("Hello world!\n\n");

    memmap::parse();

    let config = match config::read() {
        Ok(cfg) => cfg,
        Err(status) => {
            println!("error while reading config: {status}");
            panic!();
        },
    };


    println!("\nconfig found:");
    if let Some(delay) = config.delay() {
        println!("\tdelay: {delay}");
    } else {
        println!("\tdelay is not set");
    }

    if let Some(path) = config.kernel_path() {
        println!("\tpath: {path}");
    } else {
        println!("\tpath not set");
    }

    if let Some(params) = config.kernel_params() {
        println!("\tparams: {params}");
    } else {
        println!("\tparams not set");
    }


    boot::stall(100_000_000);
    Status::SUCCESS
}
