; --- Setup ---
loadi r0, 10      ; Load initial value 10 into R0
loadi r1, 5       ; Load 5 into R1
loadi r2, 2       ; Load 2 into R2 (for subtraction)

; --- Math Ops ---
add r0, r1        ; R0 = R0 + R1 (10 + 5 = 15)
sub r0, r2        ; R0 = R0 - R2 (15 - 2 = 13)

; --- Memory Ops ---
store r0, 0x80    ; Save the result (13) to memory address 0x80
loadi r0, 0       ; Clear R0 to prove LOAD works
load r0, 0x80     ; Load the result back from 0x80 into R0

; --- Loop & Branching ---
loadi r3, 3       ; Set a loop counter in R3

start_print:
    print r0      ; Print the result (13)
    loadi r4, 1
    sub r3, r4    ; Decrement counter R3
    brz end_prog  ; If R3 is 0, jump to end
    jmp start_print ; Otherwise, loop back

end_prog:
    halt          ; Stop the CPU
