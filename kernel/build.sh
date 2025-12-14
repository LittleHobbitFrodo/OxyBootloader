#!/bin/bash

if [[ $(uname -m) != x86_64 || $(uname -o) != "GNU/Linux" ]]; then
    echo "kernel build is supported only on an x86_64 linux machine"
    exit 1
fi


gcc -nostdlib -ffreestanding -nostartfiles -mgeneral-regs-only -fPIE -c ./kernel.c -o kernel.o

ld -m elf_x86_64 ./kernel.o -nostdlib -static -pie --no-dynamic-linker -z text -z max-page-size=0x1000 -T ./linker.ld -o kernel.elf

