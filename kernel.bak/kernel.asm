;   this is example "kernel"
;   the only thing it does is to boot and then it halts the CPU 

extern _start

_start:
cli
loop:
hlt
jmp loop