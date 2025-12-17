#!/bin/bash

x86_64-linux-gnu-gcc -nostdlib -ffreestanding -fno-builtin -fno-tree-vectorize -nostartfiles -mgeneral-regs-only -fPIE -c ./kernel.c -o kernel.o

x86_64-linux-gnu-ld -m elf_x86_64 ./kernel.o -nostdlib -static -pie --no-dynamic-linker -z text -z max-page-size=0x1000 -T ./linker.ld -o kernel.elf

