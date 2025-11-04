# OxyBoot

OxyBoot is simple bootloader written entirelly in rust with use of the `uefi-rs` crate. It is meant to be an example project with (propably) no real usecase.

## Build dependencies

1. `cargo`: please follow [instructions](https://www.rust-lang.org/tools/install)
2. `qemu`: install qemu from your distro repositories
3. `OVMF`: install OVMF from your distro repositories

## Build

just simply run `./util build`
- use `./util build run` to run the bootloader in emulator

## Important note
The `util` helper script was tested on Fedora Linux version 42. The path to the OVMF firmware may not be the same on other linux distributions