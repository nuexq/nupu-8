use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DecodeError {
    #[error("Unknown Opcode 0x{0:X}")]
    InvalidOpcode(u8),
}

/// The raw hex values for the CPU operations.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Opcode {
    LOADI = 0x0,
    MOV = 0x1,
    LOAD = 0x2,
    STORE = 0x3,
    ADD = 0x4,
    SUB = 0x5,
    PRINT = 0x6,
    CMP = 0x8,
    BRZ = 0x9,
    BRN = 0xA,
    BRC = 0xB,
    JMP = 0xC,
    HALT = 0xE,
    NOP = 0xF,
}

impl Opcode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x0 => Some(Opcode::LOADI),
            0x1 => Some(Opcode::MOV),
            0x2 => Some(Opcode::LOAD),
            0x3 => Some(Opcode::STORE),
            0x4 => Some(Opcode::ADD),
            0x5 => Some(Opcode::SUB),
            0x6 => Some(Opcode::PRINT),
            0x8 => Some(Opcode::CMP),
            0x9 => Some(Opcode::BRZ),
            0xA => Some(Opcode::BRN),
            0xB => Some(Opcode::BRC),
            0xC => Some(Opcode::JMP),
            0xE => Some(Opcode::HALT),
            0xF => Some(Opcode::NOP),
            _ => None,
        }
    }
}

/// The structured representation of an instruction.
/// This guarantees the Assembler and CPU agree on the arguments.
#[derive(Debug, PartialEq)]
pub enum Instruction {
    LoadI { dest: u8, imm: u8 },
    Mov { dest: u8, src: u8 },
    Load { dest: u8, addr: u8 },
    Store { src: u8, addr: u8 },
    Add { dest: u8, src: u8 },
    Sub { dest: u8, src: u8 },
    Print { src: u8 },
    Cmp { reg1: u8, reg2: u8 },
    Brz { addr: u8 },
    Brn { addr: u8 },
    Brc { addr: u8 },
    Jmp { addr: u8 },
    Halt,
    Nop,
}

impl Instruction {
    /// Internal helper to pack the bits safely.
    /// Format: [4 bits opcode] [3 bits reg] [1 bit unused] [8 bits operand]
    fn pack(opcode: Opcode, reg: u8, operand: u8) -> u16 {
        let op_shifted = (opcode as u16) << 12;
        let reg_shifted = ((reg & 0x0F) as u16) << 8;
        let operand_cast = operand as u16;

        op_shifted | reg_shifted | operand_cast
    }

    /// ASSEMBLER: Converts the structured enum into a 16-bit binary instruction.
    pub fn encode(&self) -> u16 {
        use Instruction::*;
        match self {
            LoadI { dest, imm } => Self::pack(Opcode::LOADI, *dest, *imm),
            Mov { dest, src } => Self::pack(Opcode::MOV, *dest, *src),
            Load { dest, addr } => Self::pack(Opcode::LOAD, *dest, *addr),
            Store { src, addr } => Self::pack(Opcode::STORE, *src, *addr),
            Add { dest, src } => Self::pack(Opcode::ADD, *dest, *src),
            Sub { dest, src } => Self::pack(Opcode::SUB, *dest, *src),
            Print { src } => Self::pack(Opcode::PRINT, 0, *src),
            Cmp { reg1, reg2 } => Self::pack(Opcode::CMP, *reg1, *reg2),
            Brz { addr } => Self::pack(Opcode::BRZ, 0, *addr),
            Brn { addr } => Self::pack(Opcode::BRN, 0, *addr),
            Brc { addr } => Self::pack(Opcode::BRC, 0, *addr),
            Jmp { addr } => Self::pack(Opcode::JMP, 0, *addr),
            Halt => Self::pack(Opcode::HALT, 0, 0),
            Nop => Self::pack(Opcode::NOP, 0, 0),
        }
    }

    /// CPU: Converts a 16-bit binary instruction back into the structured enum.
    pub fn decode(raw: u16) -> Result<Self, DecodeError> {
        let opcode_raw = ((raw >> 12) & 0x0F) as u8;
        let reg = ((raw >> 9) & 0x07) as u8;
        let operand = (raw & 0xFF) as u8;

        let opcode =
            Opcode::from_u8(opcode_raw).ok_or(DecodeError::InvalidOpcode(opcode_raw))?;

        use Opcode::*;
        match opcode {
            LOADI => Ok(Instruction::LoadI {
                dest: reg,
                imm: operand,
            }),
            MOV => Ok(Instruction::Mov {
                dest: reg,
                src: operand,
            }),
            LOAD => Ok(Instruction::Load {
                dest: reg,
                addr: operand,
            }),
            STORE => Ok(Instruction::Store {
                src: reg,
                addr: operand,
            }),
            ADD => Ok(Instruction::Add {
                dest: reg,
                src: operand,
            }),
            SUB => Ok(Instruction::Sub {
                dest: reg,
                src: operand,
            }),
            PRINT => Ok(Instruction::Print { src: operand }),
            CMP => Ok(Instruction::Cmp {
                reg1: reg,
                reg2: operand,
            }),
            BRZ => Ok(Instruction::Brz { addr: operand }),
            BRN => Ok(Instruction::Brn { addr: operand }),
            BRC => Ok(Instruction::Brc { addr: operand }),
            JMP => Ok(Instruction::Jmp { addr: operand }),
            HALT => Ok(Instruction::Halt),
            NOP => Ok(Instruction::Nop),
        }
    }
}
