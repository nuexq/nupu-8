loadi r0, 0
load r2, 4
out r0, 0x10    
out r2, 0x11    

; N
loadi r1, 0xFF
out r1, 0x12  
loadi r1, 0x02
out r1, 0x12  
loadi r1, 0x04
out r1, 0x12  
loadi r1, 0x08
out r1, 0x12  
loadi r1, 0xFF
out r1, 0x12  

; space
loadi r1, 0
out r1, 0x12

; U
loadi r1, 0x7F
out r1, 0x12

loadi r2, 0
u_bottom_loop:
    loadi r1, 0x80
    out r1, 0x12
    add r2, 1
    cmp r2, 4
    brz end_u
    jmp u_bottom_loop

end_u:
  loadi r1, 0x7F
  out r1, 0x12

; space
loadi r1, 0
out r1, 0x12

; P
loadi r1, 0xFF
out r1, 0x12

loadi r2, 0
p_loop:
  loadi r1, 0x09
  out r1, 0x12
  add r2, 1
  cmp r2, 3
  brz end_p
  jmp p_loop

end_p:
  loadi r1, 0x06
  out r1, 0x12

; space
loadi r1, 0
out r1, 0x12

; U
loadi r1, 0x7F
out r1, 0x12

loadi r2, 0
u_bottom_loop_2:
    loadi r1, 0x80
    out r1, 0x12
    add r2, 1
    cmp r2, 4
    brz end_u_2
    jmp u_bottom_loop_2

end_u_2:
  loadi r1, 0x7F
  out r1, 0x12

; space
loadi r1, 0
out r1, 0x12

; -
load r2, 0
dash_loop:
  loadi r1, 0x08
  out r1, 0x12
  add r2, 1
  cmp r2, 4
  brz dash_end
  jmp dash_loop

dash_end:

; space
loadi r1, 0
out r1, 0x12

; 8
loadi r1, 0x76
out r1, 0x12

loadi r2, 0
eight_loop:
  loadi r1, 0x89
  out r1, 0x12
  add r2, 1
  cmp r2, 3
  brz eight_end
  jmp eight_loop

eight_end:
  loadi r1, 0x76
  out r1, 0x12

halt
