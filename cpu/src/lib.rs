use crate::error::{CpuError, Result};
use colored::Colorize;
use log::info;
use shared::Instruction;

mod error;

pub struct Flags {
    pub zero: bool,
    pub carry: bool,
    pub negative: bool,
}

pub struct Cpu {
    pc: u8,              // Program Counter (8-bit)
    ir: u16,             // Instruction Register (16-bit)
    registers: [u8; 16], // 16 General Purpose Registers (8-bit)
    flags: Flags,

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

    pub fn tick(&mut self) -> Result<()> {
        match self.step {
            0 => {
                self.ir = (self.memory[self.pc as usize] as u16) << 8;
                self.step = 1;
            }
            1 => {
                self.ir |= self.memory[self.pc.wrapping_add(1) as usize] as u16;
                self.pc = self.pc.wrapping_add(2);
                self.step = 2;
                info!("IR: {:016b}", self.ir)
            }
            2 => {
                self.execute()?;
                self.step = 0; // Reset cycle
            }
            _ => self.step = 0,
        }
        Ok(())
    }

    pub fn execute(&mut self) -> Result<()> {
        let instr = Instruction::decode(self.ir)?;

        use Instruction::*;
        match instr {
            LoadI { dest, imm } => { 
                self.registers[dest as usize] = imm;
                info!("{:?}", self.registers[dest as usize]);
            },
            Mov { dest, src } => self.registers[dest as usize] = self.registers[src as usize],
            Load { dest, addr } => self.registers[dest as usize] = self.memory[addr as usize],
            Store { src, addr } => self.memory[addr as usize] = self.registers[src as usize],
            i @ (Add { .. } | Sub { .. } | Cmp { .. }) => self.alu(i)?,
            Print { src } => {
                let value = self.registers[src as usize];
                info!("{} {}", "CPU OUT:".green().bold(), value);
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
                    info!("should: {:?}, flag: {:?}", i, self.flags.zero);
                    let target = addr as usize;

                    if target >= self.memory.len() {
                        return Err(CpuError::InvalidJump {
                            addr: target,
                            limit: self.memory.len() - 1,
                        });
                    }

                    self.pc = target as u8;
                    info!("{} Jumped to 0x{:02X}", format!("{:?}:", i).cyan(), addr);
                }
            }
            Halt => self.halted = Some(self.pc),
            Nop => info!("NOP: skipping cycle..."),
        }

        Ok(())
    }

    fn alu(&mut self, instr: Instruction) -> Result<()> {
        use Instruction::{Add, Cmp, Sub};
        match instr {
            Add { dest, src } => {
                let a = self.registers[dest as usize];
                let b = self.registers[src as usize];
                let (result, carry) = a.overflowing_add(b);

                self.registers[dest as usize] = result;

                self.flags.zero = result == 0;
                self.flags.negative = (result & 0x80) != 0;
                self.flags.carry = carry;
            }
            Sub { dest, src } => {
                let a = self.registers[dest as usize];
                let b = self.registers[src as usize];
                let (result, borrow) = a.overflowing_sub(b);
                info!("{} - {}: {}, borrow: {}",a, b, result, borrow);

                self.registers[dest as usize] = result;

                self.flags.zero = result == 0;
                self.flags.negative = (result & 0x80) != 0;
                self.flags.carry = !borrow;
            }
            Cmp { reg1, reg2 } => {
                let a = self.registers[reg1 as usize];
                let b = self.registers[reg2 as usize];
                let result = a.wrapping_sub(b);

                self.flags.zero = result == 0;
                self.flags.negative = (result & 0x80) != 0;
                self.flags.carry = a >= b;

                info!(
                    "CMP R{} ({}) - R{} ({}): Z={}, N={}, C={}",
                    reg1, a, reg2, b, self.flags.zero, self.flags.negative, self.flags.carry
                );
            }
            _ => return Err(CpuError::InvalidAluOperation(instr)),
        }
        Ok(())
    }
}
