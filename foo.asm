loadi r0, 10
loadi r1, 5
loadi r2, 2

add r0, r1
sub r0, r2

store r0, 0x80
loadi r0, 0
load r0, 0x80

loadi r3, 3

start_print:
    print r0
    loadi r4, 1
    sub r3, r4
    brz end_prog
    jmp start_print

end_prog:
    halt

