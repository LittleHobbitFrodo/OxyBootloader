
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
    1. [Assembly routine](#assembly-routine)
    2. [Linking](#linking)
    3. [Compilation](#compilation-1)
6. [Loading the kernel](#loading-the-kernel)
    1. [Protocols and handles](#protocols-and-handles)
    2. [Loading the kernel](#loading-the-kernel-1)
        1. [Opening the root directory](#opening-the-root-directory)
        2. [Path to the kernel](#path-to-the-kernel)
        3. [Obtaining information about the file](#obtaining-information-about-the-file)
        4. [Reading the file](#reading-the-file)
7. [Paging and the x86_64 crate](#paging-and-the-x86_64-crate)
    1. [Memory protection in general](#memory-protection-in-general)
    2. [Paging - briefly](#paging---briefly)
        1. [Paging in general](#paging-in-general)
        2. [Math, tables and entries](#math-tables-and-entries)
        3. [Paging is like onion](#paging-is-like-onion)
        4. [Virtual addresses](#virtual-addresses)
        5. [Caching](#caching)
        6. [Identity mapping](#identity-mapping)
    3. [`x86_64` crate](#x86_64-crate)
8. [Parsing the ELF using goblin](#parsing-the-elf-using-goblin)
9. [Gathering boot info](#gathering-boot-info)
10. [Passing the information to the kernel](#passing-the-information-to-the-kernel)
11. [Simple kernel in C](#simple-kernel-in-c)

---

## About the guide

> **PLEASE READ**: Most example routines are performed on a **LINUX SYSTEM** (Fedora 43).

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

For simplicity, the bootloader itself must be located in the `/EFI/BOOT/` directory and must be named `BOOTX64.EFI` (on x86_64 systems). The compiled bootloader is usually put into `target/x86_64-unknown-uefi/debug-or-release` (from project root directory). Other components, such as configuration or kernel, can be located anywhere in the partition, as the bootloader loads them manually. The kernel in the example project is located in `/oxy/kernel.elf`.

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
4. `jmp loop`: Just a safeguard to prevent uninitialized memory from being executed by creating infinite loop.


### Linking

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

### Compilation

To compile the kernel, we need the `nasm` assembler and any linker capable of linking freestanding binary for the x86_64 architecture. Pesonally, I prefer the `x86_64-linux-gnu-ld`, although better options may be available.

1. Create Object file by running `nasm -f elf64 kernel.asm -o kernel.o`
2. Link it with the linker script: `x86_64-linux-gnu-ld -o kernel.elf kernel.o -T linker.ld`
    - Do not forget to use your linker script with the `-T` switch!

## Loading the kernel

Reading files is not an easy process when you don't have any OS to support you. But perhaps it's not that difficult when you have UEFI instead of a regular operating system...

### Protocols and handles

Almost every action done using UEFI needs some protocols. Let's take a look at them!

UEFI provides access to a set of protocols and their handles. Handles usually represent some kind of resource, whether it be a disk, screen, or something else. Each handle serves as a gateway to an associated protocol.

On the other hand, protocols are raw interfaces to associated resources, usually consisting of a list of functions that are called.

The uefi crate wraps all these handles as structures, ensuring Rust's safety guarantees and keeping you safe from low-level issues.

### Loading the kernel

#### Opening the root directory

To load the kernel (or any other file), we need to get the `SimpleFileSystem` protocol and open its root directory, where we can then locate the file.

The `SimpleFileSystem` protocol is opened by calling the `uefi::boot::get_image_file_system()` function, which we pass the UEFI system partition descriptor to by calling the `uefi::boot::image_handle()` function.

Once we have obtained the SFS protocol, we can open its root directory. We do this by calling the `open_volume()` function on the `SimpleFileSystem` protocol we obtained in the previous step.

The simplified final code looks like this:
```rust
let mut sfs = uefi::boot::get_image_file_system(uefi::boot::image_handle())
    .expect("failed to open SFS");
 let root_dir = sfs.open_volume().expect("failed to open root dir");
```

#### Path to the kernel

UEFI is a standard made for advanced C programs. Therefore all/most paths are UTF16 encoded and null terminated. Wouldn't want to do this stuff in C, would you..

Fortunately, we are rustaceans and we have a convenient way to do this: introducing `CStr16`, UTF16 encoded, null terminated string.

The conversion is simple: First we need to allocate buffer, gool old array on the stack will do. Then we call the `CStr16::from_str_with_buf()` function, which does the conversion.

> **Note**: all paths are also separated by backslash (like on windows).

```rust
let mut name_buf = [0u16; 256];
let filename = CStr16::from_str_with_buf("\\oxy\\kernel.elf", &mut name_buf)
    .expect("failed to convert to UTF16");
```

#### Obtaining information about the file

The next step is to find out how long the file is. We also need to check wheether the file is actually a regular file or directory. To do this, we need to obtain handle to the file.

We can obtain `FileHandle` by calling the `open()` function on the root directory structure. This function takes the following parameters:
1. `&Cstr16`: Converted file name.
2. `FileMode`: Choose between reading and/or writing.
3. `FileAttributes`: Special functions, such as read-only files or backup files, we want the attributes to be empty.

To check the file type, we simply call the `into_regular_file()` function, which consumes the handle. The function returns `Some()` if the file is a regular file, and `None` if it is a directory.

```rust
let handle = root_dir.open(filename, FileMode::Read, FileAttribute::empty())
    .expect("failed to open the kernel");
let file = handle.into_regular_file().expect("found directory");
```

Once we have the file opened, we can finally measure the length of the file. In bytes of course. But since we are working with uefi, its not as simple as it could be. We have to get the `FileInfo` structure into preallocated buffer.

As I said, we need to preallocate a buffer in advance (and again) array on the stack will do. Then we need to call `get_info()` function on the regular file handle and pass reference to the buffer.
> **Note**: The `get_info()` function requires explixit type annotation (`&FileInfo`).

```rust
let mut info_buf = [0u8; 512];
let info: &FileInfo = file.get_info(&mut info_buf)
    .expect("failed to get file info");
```

Now we can finally find out the length of the file using the `file_size()` function from the returned `FileInfo` structure.

```rust
let file_size = info.file_size() as usize;
```

#### Reading the file

Before we can continue analyzing the file, we need to load it into one location and then copy some of its data to a predetermined location. Well, now we have other things to worry about...

When reading a file, we need to allocate buffer in advance. And since we don't know the length of the file at compile time, we can't use the good old stack allocated buffer. However, we can use the `allocator-api2` crate, which is also used by the standard library. `allocator-api2` provides all the useful things, such as `Vec` or `Box`.

The function used to read the file is, as expected, `file.read()`. It takes a mutable reference to the buffer and returns a `Result` indicating whether it failed.

```rust
let mut loaded: Box<[u8]> = unsafe { Box::new_zeroed_slice(file_size)
    .assume_init() };

file.read(&mut loaded).expect("failed to load the kernel");
```


## Paging and the `x86_64` crate

### Memory protection in general

Some computers, such as controllers or other single-processor computers have two types of memory. RAM and ROM. ROM (short for **R**ead **O**nly **M**emory) contains all executalbe code. On the other hand, RAM (**R**andom **A**ccess **M**eory) stores the stack and heap (mutable memory in general).

But when advanced computers came along, operating systems commonly loaded more programs into RAM at the same time. The problem was that all the programs a single one memory. One shared memory for all executable code, stacks, heaps and the operating system itself. You guessed it, this caused certain problems...

At this time, the first attempts to create a certain level of memory protection appeared. The first memory protection mechanism in x86 was implemented on 16-bit processors. It is called segmentation.

[Segmentation](https://wiki.osdev.org/Segmentation) is not that complicated: you have a list of segment descriptors that is used to divide existing memory into segments, each with its own permissions. X86 processors also has a set of segment registers. These registers are used by the operating system to set which memory a currently running program can use and how.

When first 32-bit processors we introduced, this mechanism was simply extended to 32-bit address space and descriptors with backwards compatibility in mind.

The problem with segmentation is that it is very difficult to manage effectively. It's a common vector problem: it's effective for push and pop, but not so much for inserting or removing.

Paging solves this problem!

And adds a lot of complexity and abstraction to it...

### Paging - briefly

> This chapter contains only the most important information, visit [OSDev wiki](https://wiki.osdev.org/Paging) to read more.

Have you ever looked at your pointers and said to yourself, "Wait a minute, I don't have that much memory"?

No? Nevermind...

#### Paging in general

Compared to segmentation, paging is a pretty abstract and complicated concept. Don't worry if you don't get it right away. It took me about a month to understand it, and mistakes are still on my daily menu.

Address types:
- **Virtual** addresses are converted to physical addresses by the CPU. They are used to strictly limit what memory each program can use and how it can use it. Regular computers are using 48 bits of the virtual address.
- **Physical** addresses are pointing to the real location in the physical memory.

#### Math, tables and entries

To do a deep dive into the concept of paging, we must first to introduce a few terms:
- A **page table** is an **array of 512 page entries**. It occupies exactly 4KB of memory and its address is always aligned to 4KB. Why these numbers? Those numbers are quite important...
- A **page Entry** is a 64-bit binary structure that stores 36 bits of memory address. The remaining 28 bits are treated as flags or are unused.

Lets take a look at the numbers. Each page entry is 64 bits, or 8 bytes. Each page table stores 512 entries. How much is `8 * 512`? Exactly `4096`, or 4KB. Now we see that each page table is 4KB in size.

Remember how I told you that only 48 bits of a virtual address are used? And how I told you that each page entry contains 36 bits of the physical address? Yes, correct. I lied.

Since converting 48 bit virtual addresses to 36 bit physical addresses makes no sense, page entries use one simple trick: they actually contain only 36 bits of the address, but are interpreted as 48 bits. How? When you look at the table entry [structure](https://wiki.osdev.org/images/thumb/4/41/64-bit_page_tables1.png/450px-64-bit_page_tables1.png), you can see that the first 12 bits are flags. How much is `36 + 12`? Yes, `48`. See how it fits together? Those 36 bits of the physical address stored in the entry are only the upper 36 bits. The other (lower) 12 bits are not present in the entry and are treated as cleared.

These 12 unused/cleared bits ensure that each address referenced by a table entry is aligned to 2 to the power of 12. Which is `4096`, or 4KB. And now you know why the page tables are aligned too!

#### Paging is like onion

Paging consists of a four-level tree structure that the CPU traverses. The path through the graph is the actual virtual address, which contains indexes for each level.

Levels:
1. **PML4** - **P**age **M**ap **L**evel **4**, **PS** page size: 512 GB.
2. **PDPT** - **P**age **D**irectory **P**ointer **T**able, **PS** page size: 1 GB.
3. **PD** - **P**age **D**irectory, **PS** page size: 2 MB.
4. **PT** - **P**age **T**able entry, **PS** is unsupported at this level.
> The list is reversed, PML4 is generally considered to be the fourth level.

The CPU begins the traversal by looking at the address in the `cr3` control register. This is the physical address of the **PML4** table. It then obtains the PML4 index from the virtual address to read an entry at that position.

The CPU then continues by reading the flags of the entry: if the **P**resent flag is cleared, a page fault is triggered, which is caught by the operating system. Otherwise, it continues by checking the **P**age **S**ize flag. If it is set, the pml4 entry is considered a pointer to a 512 GB page. Otherwise, the entry is considered a pointer to a third-level table.

The CPU then obtains the index for the next level and reads entry at the position. Checks the **P**resent and **P**age **S**ize flags. If **PS** is set, the entry is considered a pointer to 1GB (or 2MB at the PD level) page. Otherwise the CPU goes to the next level. This cycle continues to the first level, where all entries always point to a 4 KB page, or until it finds entry with the **PS** flag set.

When the page address is found, the CPU simply adds the offset from the virtual address to it.

#### Virtual addresses

Paging works on the concept of translating virtual addresses to physical addresses in the MMU (**M**emory **M**anagement **U**nit) within the CPU. Each virtual address is a binary structure that allows the MMU to navigate within a **4 level tree structure** to translate the virtual address to a physical address. Mind blowing, right?

Each virtual address is 64-bit binary structure. Lets see what's inside:
- **Offset** is the lowest 12 bits, representing the offset from the 4KB aligned memory region referenced by the last page table.
- **PT index**: Next 9 bits are used to navigate the fourth page table (in the PT level). As you can see that 9 bits can hold maximum value of 511.
- **PD index**: Next 9 bits used to navigate the third page table (in the PD level).
- **PDPT index**: Another 9 bits. Used to navigate the second page table (in the PDPT level).
- **PML4 index**: The last 9 bits. Used to navigate the first page table (in the PML4 level).
- **Sign**: The remaining 16 bits. All bits must be set to 0 for user processes or to 1 for the OS kernel.

You can visualize it like this (MSB on the left):
| 64 - 48 | 47 - 39 | 38 - 30 | 29 - 21 | 20 - 12 | 11 - 0 |
|------|------|------|----|----|--------|
| Sign | PML4 | PDPT | PD | PT | Offset |
| 16b | 9b | 9b | 9b | 9b | 12b |

### Caching

To make the conversion of virtual addresses to physical addresses faster, the CPU contains a cache that speeds up the conversion.

The whole cache can be flushed by simply writing into the `cr3` register. On the other hand the `INVLPG` (invalidate tlb entry) instruction invalidates/removes only one address. This is important when you remove valid addresses from the address space to prevent some problems.


#### Identity mapping

UEFI runs the bootloader in what is known as an [identity-mapped](https://wiki.osdev.org/Identity_Paging) virtual address space. Identity mapping means that each virtual address directly corresponds to its physical address. Simply put, virtual address `0xBEEF` would refer to physical address `0xBEEF`.

### `x86_64` crate

Although x86_64 paging makes sense and is well designed, it can be quite a challenge. Fortunately (for us), the Rust OSdev team maintains the [`x86_64`](https://crates.io/crates/x86_64) crate. It provides safe abstractions over some x86_64-specific instructions and (most importantly) virtual address space mappers. These are the things that do paging for us.

We will use the `Cr0::update()` function to disable memory protection. And for paging, we will use the `OffsetPageTable` mapper together with our own frame allocator.

## Parsing the ELF using goblin



## Gathering boot info

## Passing the information to the kernel

## Simple kernel in C
