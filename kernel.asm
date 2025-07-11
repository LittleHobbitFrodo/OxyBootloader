;   this is example "kernel"
;   the only thing it does is to boot and then it halts the CPU 

cli
loop:
hlt
jmp loop