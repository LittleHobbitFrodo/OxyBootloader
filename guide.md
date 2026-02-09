
# Minimal bootloader in Rust
> Creating minimalistic UEFI bootloader in rust targeting x86_64

## Table of content
1. [About the guide](#about-the-guide)
2. [Intorduction](#introduction)
    1. [Boot Sequence](#boot-sequence)
    2. [The task of the bootloader](#the-task-of-the-bootloader)
3. [Briefly into UEFI](#briefly-into-uefi)
4. [Hello world!](#hello-world)
    1. [`no_std` and the `core` library](#no_std-and-the-core-library)
    2. [The `uefi` crate](#the-uefi-crate)
    3. [Hello world program](#hello-world-program)
5. [Running in an emulator](#running-in-an-emulator)
    1. [Compilation](#compilation)
    2. [Creating UEFI system partition](#creating-uefi-system-partition)
    3. [Emulator and OVMF](#emulator-and-ovmf)
5. [Example kernel in assembly](#example-kernel-in-assembly)
6. [Loading the kernel](#loading-the-kernel)
7. [Paging and the x86_64 crate](#paging-and-the-x86_64-crate)
8. [Parsing the ELF using goblin](#parsing-the-elf-using-goblin)
9. [Gathering boot info](#gathering-boot-info)
10. [Passing the information to the kernel](#passing-the-information-to-the-kernel)
11. [Simple kernel in C](#simple-kernel-in-c)

---

## About the guide

> **PLEASE READ**: Most examples (including creating an EFI system partition) are performed on a **LINUX SYSTEM**. More specifically, on Fedora 43

This text is not a complete manual, but rather a guide describing the development of a very simple bootloader for the x86_64 bootloader. It covers most of what a bootloader needs to do to start the kernel.

To be precise, the bootloader itself will be written in the Rust programming language using the `uefi` library. The demo kernel is written in C, but to simplify the early stages of development, an Assembly language kernel will be used.

My [github repository](https://github.com/LittleHobbitFrodo/OxyBootloader) contains source code for this guide (altogether with the `util` script to build the bootloader).

## Introduction

If you are reading this article, you are probably interested in bootloader development, but you cannot develop a bootloader without knowing something about why it exists and what it does.

So let's start with a quick introduction.

### Boot sequence
> What happens when you press the start button on your computer?

~~When you press the Start button, your computer will probably spin up its fans, then display the manufacturer's logo, and here is your operating system. Pretty straightforward, right? Well, not so much...~~

The Start button causes electricity to magically flow into the processor, which launches the firmware. The firmware is software stored in ROM (**R**ead-**O**nly **M**emory) on the motherboard. As you may have already figured out, firmware is the first piece of code that runs on your computer, but what does that actually mean? The firmware must initialize all components in your computer, including the keyboard, mouse, displays, and everything else, in order to run our bootloader.

In fact, the firmware must also initialize the CPU. The reason for this is that, for backward compatibility with older software, it first runs in 16-bit mode. This means that, theoretically, you could run MS-DOS on your computer!

After completing the initialization process, the firmware searches for our bootloader. If it finds it, it hands control over to it.
- Modern UEFI systems use the EFI system partition, where the firmware attempts to locate the bootloader
- Older BIOS systems (the predecessor to UEFI) used MBR, where the bootloader itself was stored in the first 512 bytes of the disk. Bootloaders at that time were relatively primitive and mostly written in assembler.


### The task of the bootloader

Our bootloader has a seemingly simple task:

1. Find and load the kernel executable file.
2. Analyze the executable file and map its virtual address space.
3. Collect the data that the kernel needs.
4. Transfer control to the kernel and jump into it.


## Briefly into UEFI

> What is UEFI? How do we interact with it?

UEFI stands for **U**nified **E**xtensible **F**irmware **I**nterface. It's not just a magical thing that initializes your computer and loads the boot loader. It also provides services. Specifically, UEFI **Boot** and **Runtime** services. What's the difference between them?

UEFI **Boot** services are there to help your bootloader. They consist of interfaces for working with memory, loading and saving data from disks, etc. ~~In short, it gives your bootloader the superpowers it needs.~~

**Runtime** services, on the other hand, help the operating system itself perform some fairly specific tasks. For example, shutting down or restarting the computer.

For this project, we will be using the `uefi` crate to handle the firmware communication. It provides fairly simple safe abstractions for interaction with the boot and/or runtime services.

Let's focus on something practical for a change!


## Hello world!

In order to create the bootloader executable, we need to talk about rust and its `no_std` executables.

### `no_std` and the `core` library
Most rust programs require and operating system to be able to work. But bootloaders are made to bring the operating system to life. So how can we make rust work with no operating system? Fortunately, Rust developers built it in with the `no_std` attribute.

`#![no_std]` is crate level attribute that tells the compiler that the crate is supposed to be linked with the `core` library instead of `std`. Core is smaller than std but its providing only very basic functionalities and types. It is also not dependent on any platform and/or operating system, so it can work with no OS at all.


### The `uefi` crate

We cannot make an bootloader only with the `core` library. Luckily we can use the [uefi](https://crates.io/crates/uefi) create.

Uefi provides all the features you need to work with UEFI firmware in a secure and convenient way. It also provides some primitive features from the standard library. For example, `Box`, `Vec` or the `print!()` macro.

In order for our program to work with the library, we need to use the `#![no_std]` attribute together with `#![no_main]`. The `no_main` attribute tells Rust not to expect the main function, because we cannot use it with the uefi crate. Instead, we have the procedural macro `#[entry]`

The reason why UEFI does not support Rust's typical main function is that the firmware expects you to return a status indicating success or failure and its cause. UEFI provides the `Status` structure for this purpose.

### Hello world program

```rust
#![no_main]
#![no_std]

use uefi::{self, entry, Status, print, println};

#[entry]
fn main() -> Status {

    uefi::helpers::init()
        .expect("failed to initialize helpers");
        //  optional: initializes features such as allocator

    println!("Hello world!");

    uefi::boot::stall(10_000_000);

    Status::SUCCESS
}
```

In the code above, the `uefi::boot::stall()` function causes the program to take a break for 10 seconds and then continue execution. That is to prevent immediate reboot.

## Running in an emulator

> You can take a look at the [`util` script](./util), which contains all the logic for creating the ISO image and running the emulator.

### Compilation

Compilation of the bootloader is quite simple. Since we can use `cargo`, a single command is all it takes. However, because we are not compiling for a native target (OS and architecture) and/or operating system, we need to tell `cargo` what target to compile for. That means we need to use the `--target` switch following with the uefi target which is `x86_64-unknown-uefi`.

So `cargo build --target x86_64-unknown-uefi` and we are done.

### Creating UEFI system partition

The bootloader is stored in UEFI system partition. It is (usually) small FAT-32 partition that stores the bootloader and its files together with the OS kernel that will be loaded. Luckily, you do not have to create an image of the partition, just its file structure. So lets begin.

For simplicity, the bootloader itself must be located in the `/EFI/BOOT/` directory and must be named `BOOTX64.EFI` (on x86_64 systems). Other components, such as configuration or kernel, can be located anywhere in the partition, as the bootloader loads them manually. The kernel in the example project is located in `/oxy/kernel.elf`.

Once you have the UEFI system partition structure ready, you can run the bootloader in emulator.

### Emulator and OVMF

Since testing the bootloader on a real computer is very slow and inconvenient in almost every way, I will explain how to use an emulator. Qemu (**Q**uick **EMU**lator) supports a large number of platforms and is widely used to test operating systems and/or bootloaders.

Most emulators (including Qemu) can emulate only legacy BIOS by default. However, we can use [OVMF](https://github.com/tianocore/tianocore.github.io/wiki/OVMF) (**O**pen **V**irtual **M**achine **F**irmware), which is an open source implementation of UEFI for Qemu. You need to install it and then point Qemu to where it is located.

First we need to tell Qemu where to find the UEFI system partition and how to treat it as disk. This is what the `-drive` switch is for. Typical boot drive configuration can look like this: `-drive format=raw,file=fat:rw:efi-img/`. But what does it do?

- `format=raw` makes the drive exposed as raw bytes-to-bytes hard drive.
- `file=fat:rw:path/to/UEFI-syspart/` use the drive/partition as FAT-32 formatted.
  - `rw` allow reading and writing, just `r` will do for our usecase.
  - `path/to/UEFI-syspart/` path to the [UEFI system partition structure](#creating-uefi-system-partition).


Now we need to tell Qemu where to find the firmware. We can use the `-bios` switch: `-bios path/to/OVMF_CODE.fd`.

With these two switches, the resulting command looks like this: `qemu-system-x86_64 -drive format=raw,file=fat:rw:efi-img/ -bios path/to//OVMF_CODE.fd`.

After running, a window will appear:

![Hello world in qemu](assets/qemu.png)

## Example kernel in assembly

In order to work on the bootloader, we need something to load. In this chapter, we will create a primitive kernel — a few lines of assembly.

### Assembly routine

The whole "kernel" can be described in this code:
```
extern _start

_start:
cli
loop:
hlt
jmp loop
```

The whole code does exactly this:
1. `extern _start`: Creates `_start` symbol reference and leaves its resolution to the linker.
    - Makes it visible to our bootloader.
2. `cli` (**CL**ear **I**nterrupt): This instruction disables interrupts by setting a bit in one of the control registers.
    - You can find more on the [OSDev Wiki](https://wiki.osdev.org/Interrupts)
3. `hlt`: Disables execution of code until interrupt occur
    - Interrupts are disabled, so the execution shall not continue from here.
4. `jmp loop`: Just a safeguard to prevent uninitialized memory from being executed.


### Compiling and Linking

#### Theory

In order to compile the kernel, we must first create an object file and then link it. An object file is a partially compiled program. It contains part of the machine code and references to other functions/symbols, which are resolved by the linker.

To link the files together, we need a special linker script. The linker script does not contain any logic; it is more like a map that tells the linker where each section of memory is located in the final executable file.

Since the kernel does not use any data, our linker script can be very simple:
```ld
ENTRY(_start)

SECTIONS
{
    . = 0xFFFFFFFF80000000;
    .text : { *(.text*) }
}
```

Before I explain what the linking script actually does, I should tell you something about sections. In a nutshell, of course!

Sections are essentially parts of your program. Each section can store data or executable code and has its own set of permissions that are enforced by the processor. Typical sections are:
- `.text`: Stores the executable code.
- `.data`: Stores variables and statically allocated memory.
- `.rodata`: Read-only data, constants.

Now back to the linker script: what does the script actually do?
- `ENTRY(_start)` sets the `_start` function as the program's entry point.
- `. = 0xFFFFFFFF80000000;`: This tells the linker to place the start of the executable at virtual address `0xFFFFFFFF80000000`. This is important, because most kernels use the higher half of the virtual address space.
  - For more information, see the [OSDev wiki](https://wiki.osdev.org/Higher_Half_x86_Bare_Bones).
- `.text : { *(.text*) }`: This tells the compiler that the `.text` section (executable code) should be placed as the first section. Thus on address `0xFFFFFFFF80000000`.


#### Compilation

To compile the kernel, we need the `nasm` assembler and any linker capable of linking freestanding binary for the x86_64 architecture. Pesonally, I prefer the `x86_64-linux-gnu-ld`, although better options may be available.

1. Create Object file by running `nasm -f elf64 kernel.asm -o kernel.o`
2. Link it with the linker script: `x86_64-linux-gnu-ld -o kernel.elf kernel.o -T linker.ld`
    - Do not forget to use your linker script with the `-T` switch!

## Loading the kernel

## Paging and the x86_64 crate

## Parsing the ELF using goblin

## Gathering boot info

## Passing the information to the kernel

## Simple kernel in C
### Drawing lines
