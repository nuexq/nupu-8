use crate::error::{CpuError, Result};
use shared::Instruction;

mod error;

pub struct Flags {
    pub zero: bool,
    pub carry: bool,
    pub negative: bool,
}

pub struct Cpu {
    pub pc: u8,              // Program Counter (8-bit)
    pub ir: u16,             // Instruction Register (16-bit)
    pub registers: [u8; 16], // 16 General Purpose Registers (8-bit)
    pub flags: Flags,

    pub memory: [u8; 256],
    pub step: u8, // Tracks fetch/decode/execute cycles
    pub halted: Option<u8>,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            registers: [0; 16],
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
    pub fn load_memory(&mut self, data: Vec<u8>) -> Result<()> {
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
            Store { src, addr } => self.memory[addr as usize] = self.registers[src as usize],
            StoreIndirect { src, ptr } => self.memory[self.registers[ptr as usize] as usize] = self.registers[src as usize],
            i @ (Add { .. } | Sub { .. } | And { .. } | Or { .. } | Not { .. } | Cmp { .. }) => {
                self.alu(i)?
            }
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
        use Instruction::{Add, And, Cmp, Not, Or, Sub};
        match instr {
            Add { dst, src } => {
                let a = self.registers[dst as usize];
                let b = self.registers[src as usize];
                let (result, carry) = a.overflowing_add(b);

                self.registers[dst as usize] = result;

                self.flags.zero = result == 0;
                self.flags.negative = (result & 0x80) != 0;
                self.flags.carry = carry;
            }
            Sub { dst, src } => {
                let a = self.registers[dst as usize];
                let b = self.registers[src as usize];
                let (result, borrow) = a.overflowing_sub(b);

                self.registers[dst as usize] = result;

                self.flags.zero = result == 0;
                self.flags.negative = (result & 0x80) != 0;
                self.flags.carry = !borrow;
            }
            And { dst, src } => {
                let a = self.registers[dst as usize];
                let b = self.registers[src as usize];
                let result = a & b;

                self.registers[dst as usize] = result;

                self.flags.zero = result == 0;
                self.flags.negative = (result & 0x80) != 0;
                self.flags.carry = false;
            }

            Or { dst, src } => {
                let a = self.registers[dst as usize];
                let b = self.registers[src as usize];
                let result = a | b;

                self.registers[dst as usize] = result;

                self.flags.zero = result == 0;
                self.flags.negative = (result & 0x80) != 0;
                self.flags.carry = false;
            }

            Not { dst } => {
                let a = self.registers[dst as usize];
                let result = !a;

                self.registers[dst as usize] = result;

                self.flags.zero = result == 0;
                self.flags.negative = (result & 0x80) != 0;
            }
            Cmp { reg1, reg2 } => {
                let a = self.registers[reg1 as usize];
                let b = self.registers[reg2 as usize];
                let result = a.wrapping_sub(b);

                self.flags.zero = result == 0;
                self.flags.negative = (result & 0x80) != 0;
                self.flags.carry = a >= b;
            }
            _ => return Err(CpuError::InvalidAluOperation(instr)),
        }
        Ok(())
    }
    pub fn next_step(&mut self) {
        self.step = (self.step + 1) % 3;
    }
}
