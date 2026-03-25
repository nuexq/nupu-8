use crate::error::{CpuError, Result};
use shared::Instruction;

mod error;

pub struct Flags {
    pub zero: bool,
    pub carry: bool,
    pub negative: bool,
}

pub struct Cpu {
    pub pc: u8,             // Program Counter (8-bit)
    pub ir: u16,            // Instruction Register (16-bit)
    pub registers: [u8; 8], // 7 General Purpose Registers (8-bit)
    pub flags: Flags,

    pub memory: [u8; 256],
    pub step: u8, // Tracks fetch/decode/execute cycles
    pub halted: Option<u8>,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            registers: [0; 8],
            pc: 0,
            ir: 0,
            flags: Flags {
                zero: false,
                carry: false,
                negative: false,
            },
            memory: [0; 256],
            step: 0,
            halted: None,
        }
    }
}

impl Cpu {
    pub fn load_memory(&mut self, data: &Vec<u8>) -> Result<()> {
        if data.len() > self.memory.len() {
            return Err(CpuError::MemoryOverflow {
                actual: data.len(),
                limit: self.memory.len(),
            });
        }

        self.memory[..data.len()].copy_from_slice(&data);

        Ok(())
    }

    pub fn tick(&mut self) -> Result<&mut Self> {
        match self.step {
            0 => {
                self.ir = (self.memory[self.pc as usize] as u16) << 8;
            }
            1 => {
                self.ir |= self.memory[self.pc.wrapping_add(1) as usize] as u16;
            }
            2 => {
                self.pc = self.pc.wrapping_add(2);
                self.execute()?;
            }
            _ => self.step = 0,
        }
        Ok(self)
    }

    pub fn execute(&mut self) -> Result<()> {
        let instr = Instruction::decode(self.ir)?;

        use Instruction::*;
        match instr {
            LoadI { dst, imm } => self.registers[dst as usize] = imm,
            Mov { dst, src } => self.registers[dst as usize] = self.registers[src as usize],
            Load { dst, addr } => self.registers[dst as usize] = self.memory[addr as usize],
            LoadIndirect { dst, ptr } => {
                let addr = self.registers[ptr as usize] as usize;
                self.registers[dst as usize] = self.memory[addr];
            }
            Store { src, addr } => self.memory[addr as usize] = self.registers[src as usize],
            StoreIndirect { src, ptr } => {
                let addr = self.registers[ptr as usize] as usize;
                self.memory[addr] = self.registers[src as usize]
            }
            i @ (Add { .. }
            | AddI { .. }
            | Sub { .. }
            | SubI { .. }
            | And { .. }
            | AndI { .. }
            | Or { .. }
            | OrI { .. }
            | Not { .. }
            | NotI { .. }
            | Cmp { .. }
            | CmpI { .. }
            | Shl { .. }
            | Shr { .. }) => self.alu(i)?,
            i @ (Brz { addr } | Brn { addr } | Brc { addr } | Jmp { addr }) => {
                let should_jump = match i {
                    Brz { .. } => self.flags.zero,
                    Brn { .. } => self.flags.negative,
                    Brc { .. } => self.flags.carry,
                    Jmp { .. } => true,
                    _ => false,
                };

                if should_jump {
                    let target = addr as usize;

                    if target >= self.memory.len() {
                        return Err(CpuError::InvalidJump {
                            addr: target,
                            limit: self.memory.len() - 1,
                        });
                    }

                    self.pc = target as u8;
                }
            }
            Halt => self.halted = Some(self.pc),
        }

        Ok(())
    }

    fn alu(&mut self, instr: Instruction) -> Result<()> {
        use Instruction::*;

        let (dst, a, b) = match instr {
            Add { dst, src } => (
                dst,
                self.registers[dst as usize],
                self.registers[src as usize],
            ),
            AddI { dst, imm } => (dst, self.registers[dst as usize], imm),

            Sub { dst, src } => (
                dst,
                self.registers[dst as usize],
                self.registers[src as usize],
            ),
            SubI { dst, imm } => (dst, self.registers[dst as usize], imm),

            And { dst, src } => (
                dst,
                self.registers[dst as usize],
                self.registers[src as usize],
            ),
            AndI { dst, imm } => (dst, self.registers[dst as usize], imm),

            Or { dst, src } => (
                dst,
                self.registers[dst as usize],
                self.registers[src as usize],
            ),
            OrI { dst, imm } => (dst, self.registers[dst as usize], imm),

            Cmp { reg1, reg2 } => (
                reg1,
                self.registers[reg1 as usize],
                self.registers[reg2 as usize],
            ),
            CmpI { reg, imm } => (reg, self.registers[reg as usize], imm),

            Not { src } => (src, self.registers[src as usize], 0),
            NotI { imm } => (0, imm, 0),

            Shl { src, amt } => (src, self.registers[src as usize], amt),
            Shr { src, amt } => (src, self.registers[src as usize], amt),

            _ => return Err(CpuError::InvalidAluOperation(instr)),
        };

        match instr {
            Add { .. } | AddI { .. } => {
                let (result, carry) = a.overflowing_add(b);
                self.registers[dst as usize] = result;
                self.flags.zero = result == 0;
                self.flags.negative = (result & 0x80) != 0;
                self.flags.carry = carry;
            }
            Sub { .. } | SubI { .. } => {
                let (result, borrow) = a.overflowing_sub(b);
                self.registers[dst as usize] = result;
                self.flags.zero = result == 0;
                self.flags.negative = (result & 0x80) != 0;
                self.flags.carry = !borrow;
            }
            And { .. } | AndI { .. } => {
                let result = a & b;
                self.registers[dst as usize] = result;
                self.flags.zero = result == 0;
                self.flags.negative = (result & 0x80) != 0;
                self.flags.carry = false;
            }
            Or { .. } | OrI { .. } => {
                let result = a | b;
                self.registers[dst as usize] = result;
                self.flags.zero = result == 0;
                self.flags.negative = (result & 0x80) != 0;
                self.flags.carry = false;
            }
            Cmp { .. } | CmpI { .. } => {
                let result = a.wrapping_sub(b);
                self.flags.zero = result == 0;
                self.flags.negative = (result & 0x80) != 0;
                self.flags.carry = a >= b;
            }
            Not { .. } => {
                let result = !a;
                self.registers[dst as usize] = result;
                self.flags.zero = result == 0;
                self.flags.negative = (result & 0x80) != 0;
            }
            NotI { .. } => {
                let result = !a;
                self.flags.zero = result == 0;
                self.flags.negative = (result & 0x80) != 0;
            }
            Shl { .. } => {
                let result = a.wrapping_shl(b as u32); // a=value, b=amout
                self.registers[dst as usize] = result;

                self.flags.zero = result == 0;
                self.flags.negative = (result & 0x80) != 0;
            }
            Shr { .. } => {
                let result = a.wrapping_shl(b as u32); // a=value, b=amout
                self.registers[dst as usize] = result;

                self.flags.zero = result == 0;
                self.flags.negative = (result & 0x80) != 0;
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    pub fn next_step(&mut self) {
        self.step = (self.step + 1) % 3;
    }

    pub fn reset(&mut self, binary: &Vec<u8>) -> Result<()> {
        *self = Cpu::default();
        self.load_memory(binary)?;
        Ok(())
    }
}
