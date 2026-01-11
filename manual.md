
# Minimal bootloader in Rust
> Creating minimalistic UEFI bootloader in rust targeting x86_64

## Table of content
1. [About the manual](#about-the-manual)
2. [Intorduction](#introduction)
    - [Boot Sequence](#boot-sequence)
    - [The task of the bootloader](#the-task-of-the-bootloader)
3. [Briefly into UEFI](#briefly-into-uefi)
4. [Hello UEFI!](#hello-world-and-efi-system-partition)
5. [Example kernel in assembly](#example-kernel-in-assembly)
6. [Loading the kernel](#loading-the-kernel)
7. [Paging and the x86_64 crate](#paging-and-the-x86_64-crate)
8. [Parsing the ELF using goblin](#parsing-the-elf-using-goblin)
9. [Gathering boot info](#gathering-boot-info)
10. [Passing the information to the kernel](#passing-the-information-to-the-kernel)
11. [Simple kernel in C](#simple-kernel-in-c)

---

## About the manual

> **PLEASE READ**: Most examples (including creating an EFI system partition) are performed on a **LINUX SYSTEM**. More specifically, on Fedora 43

This text is not a complete manual, but rather a guide describing the development of a very simple bootloader for the x86_64 bootloader. It covers most of what a bootloader needs to do to start the kernel.

To be precise, the bootloader itself will be written in the Rust programming language using the `uefi` library. The demo kernel is written in C, but to simplify the early stages of development, an Assembly language kernel will be used.

My [github repository](https://github.com/LittleHobbitFrodo/OxyBootloader) contains source code for this guide (altogether with the `util` script to build the bootloader).

## Introduction

If you are reading this article, you are probably interested in bootloader development, but you cannot develop a bootloader without knowing something about why it exists and what it does.

So let's start with a quick introduction.

### Boot sequence
> What happens when you press the start button on your computer?

When you press the Start button, your computer will probably spin up its fans, then display the manufacturer's logo, and here is your operating system. Pretty straightforward, right? Well, not so much...

The Start button causes electricity to magically flow into the processor, which launches the firmware. Firmware is software stored in ROM (**R**ead-**O**nly **M**emory) on the motherboard. As you may have already figured out, firmware is the first piece of code that runs on your computer, but what does that actually mean? Firmware must initialize all components in your computer, including the keyboard, mouse, displays, and everything else, in order to run our bootloader.

In fact, the firmware must also initialize the CPU. The reason for this is that, for backward compatibility with older software, it first runs in 16-bit mode. This means that, theoretically, you could run MS-DOS on your computer!

After completing the initialization process, the firmware searches for our bootloader. If it finds it, it hands control over to it.
- Modern UEFI systems use the EFI system partition, where the firmware attempts to locate the bootloader
- Older BIOS systems (the predecessor to UEFI) used MBR, where the bootloader itself was stored in the first 512 bytes of the disk. Bootloaders at that time were relatively primitive and mostly written in assembler.


### The task of the bootloader

Our bootloader has a seemingly simple task:

1. Find and load the kernel executable file
2. Analyze the executable file and map its virtual address space
3. Collect the data that the kernel needs
4. Transfer control to the kernel and jump into it


## Briefly into UEFI

> What is UEFI? How do we interact with it?

UEFI stands for **U**nified **E**xtensible **F**irmware **I**nterface. It's not just a magical thing that initializes your computer and loads the boot loader. It also provides services. Specifically, UEFI **Boot** and **Runtime** services. What's the difference between them?

UEFI **Boot** services are there to help your bootloader. They consist of interfaces for working with memory, loading and saving data from disks, etc. In short, it gives your bootloader the superpowers it needs.

**Runtime** services, on the other hand, help the operating system itself perform some fairly specific tasks. For example, shutting down or restarting the computer.

For this project, we will be using the `uefi` crate to handle the firmware communication. It provides fairly simple safe abstractions for interaction with the boot and/or runtime services.

Let's focus on something practical for a change!


## Hello world and EFI system partition

## Example kernel in assembly

## Loading the kernel

## Paging and the x86_64 crate

## Parsing the ELF using goblin

## Gathering boot info

## Passing the information to the kernel

## Simple kernel in C
### Drawing lines
