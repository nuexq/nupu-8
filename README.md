An 8-bit virtual CPU with a fully custom architecture, built in Rust.

![output](https://github.com/user-attachments/assets/8fa65e7b-3c7c-446b-a65c-d732e7804e6a)

---

## Quick Specs

- 8-bit data width
- 8 general-purpose registers (r0-r7)
- 256 bytes of addressable RAM (0x00-0xFF)
- Port Mapped I/O for device communication

## Registers
### General Purpose

- **R0–R7:** 8-bit registers used for data manipulation.

### Special Purpose

- **PC (Program Counter):** 8-bit. Points to the next instruction in memory.
- **IR (Instruction Register):** 16-bit. Holds the current instruction being executed.
- **FLAGS:**
  - **Z (Zero):** Set if the last result was 0.
  - **C (Carry):** Set if the last operation resulted in an arithmetic carry.
  - **N (Negative):** Set if the high bit (MSB) of the result is 1.

## Instruction Format

**Instruction Size:** 16 bits (2 bytes)

| Byte 1 (High)                                          | Byte 2 (Low)        |
| :----------------------------------------------------- | :------------------ |
| `[4 bits: Opcode][1 bit: mode][3 bits: Register Dest]` | `[8 bits: Operand]` |

**Operand Roles:**

- Immediate (8-bit constant)
- Register Index (r0-r7).
- Memory Address (0x00-0xFF).
- Register Indirect (memory Address stored in a register).

## Instruction Set

| Opcode  | Mode | Dest (3b) | Operand (8b) | Description                                                 |
| :------ | :--- | :-------- | :----------- | :---------------------------------------------------------- |
| **0x0** | 0    | `r1`      | `imm`        | `loadi r1, imm` (Load immediate to r1)                        |
| **0x0** | 1    | `r1`      | `r2`         | `mov r1, r2` (Copy r2 to r1)                                |
| **0x1** | 0    | `r1`      | `addr`       | `load r1, [addr]` (Direct memory load)                      |
| **0x1** | 1    | `r1`      | `r2`         | `load r1, [r2]` (Indirect memory load)                      |
| **0x2** | 0    | `r1`      | `addr`       | `store r1, [addr]` (Direct memory store)                    |
| **0x2** | 1    | `r1`      | `r2`         | `store r1, [r2]` (Indirect memory store)                    |
| **0x3** | 0    | `r1`      | `r2`         | `add r1, r2` (r1 = r1 + r2)                                 |
| **0x3** | 1    | `r1`      | `imm`        | `add r1, imm` (r1 = r1 + imm)                               |
| **0x4** | 0    | `r1`      | `r2`         | `sub r1, r2` (r1 = r1 - r2)                                 |
| **0x4** | 1    | `r1`      | `imm`        | `sub r1, imm` (r1 = r1 - imm)                               |
| **0x5** | 0    | `r1`      | `r2`         | `and r1, r2` (Bitwise AND)                                  |
| **0x5** | 1    | `r1`      | `imm`        | `and r1, imm` (Bitwise AND with imm)                        |
| **0x6** | 0    | `r1`      | `r2`         | `or r1, r2` (Bitwise OR)                                    |
| **0x6** | 1    | `r1`      | `imm`        | `or r1, imm` (Bitwise OR with imm)                          |
| **0x7** | 0    | `r1`      | `0`          | `not r1` (Bitwise NOT on r1)                                |
| **0x7** | 1    | `r1`       | `imm`        | `not r1, imm` (Bitwise NOT with imm)                            |
| **0x8** | 0    | `r1`      | `r2`         | `cmp r1, r2` (Compare r1 and r2, set flags)                 |
| **0x8** | 1    | `r1`      | `imm`        | `cmp r1, imm` (Compare r1 and imm, set flags)               |
| **0x9** | 0    | `type`    | `addr`       | **type 0**: `jmp`, **1**: `brz`, **2**: `brn`, **3**: `brc` |
| **0xA** | 0    | `r1`      | `amt`        | `shl r1, amt` (Shift left by amt)                         |
| **0xA** | 1    | `r1`      | `amt`        | `shr r1, amt` (Shift right by amt)                        |
| **0xB** | 0    | `r1`      | `port`       | `out r1, port` (Output r1 to port)                 |
| **0xF** | 0    | `0`       | `0`          | `halt` (end program)                       |


## Ports Mapping

| Port | Name | Function |
| :--- | :--- | :--- |
| **0x10** | `X_PTR` | Horizontal drawing position (0–127). |
| **0x11** | `PAGE_PTR` | Vertical page (0–7). 8 pixels high each. |
| **0x12** | `DATA` | Writes 8-bit vertical slice to VRAM; auto-increments `X_PTR`. |


## Excution Model
- Each instruction takes **3 steps**:
  1. Load high byte of instruction
  2. Load low byte of instruction
  3. Execute instruction 
- `PC` increments by 2 after fetch

## Building

Try it using nix:

```bash
nix shell github:nuexq/nupu-8
```

Or build manually:

```bash
git clone https://github.com/nuexq/nupu-8
cd nupu-8
cargo build --release
```

## Usage

```bash
# Assemble and run immediately (default 100 Hz)
nupu-8 run path/to/program.asm --hz 100

# Execute a pre-compiled binary
nupu-8 exec path/to/program.bin

# Assemble only (output defaults to out.bin)
nupu-8 asm path/to/program.asm -o out.bin
```

**run an example program**:
```bash
nupu-8 run ./example/foo.asm --hz 500
```


## In the Future maybe
when returning to this project, I plan to
- write tests
- improve the assembler (better error reporting, support for macros and constants)
- add stack and subroutine support (`SP`, `push`, `pop`, `call`, `ret`)
- implement input-handling instructions
- expand to a 16-bit data bus (for a 64 KB RAM)
- maybe switch to memory-mapped I/O
- add support for data directives in assembly.
