use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DecodeError {
    #[error("Unknown Opcode 0x{0:X}")]
    InvalidOpcode(u8),
}

pub const MODE: u8 = 0x7F;
pub const TXT_MODE: u8 = 0;
pub const VRAM_START: u8 = 0x80;

/// The raw hex values for the CPU operations.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Opcode {
    LOADI = 0x0,
    MOV = 0x1,
    LOAD = 0x2,
    STORE = 0x3,
    STORER = 0x4,
    ADD = 0x5,
    SUB = 0x6,
    AND = 0x7,
    OR = 0x8,
    NOT = 0x9,
    CMP = 0xA,
    BRZ = 0xB,
    BRN = 0xC,
    BRC = 0xD,
    JMP = 0xE,
    HALT = 0xF,
}

impl Opcode {
    pub fn from_u8(value: u8) -> Option<Self> {
        use Opcode::*;
        match value {
            0x0 => Some(LOADI),
            0x1 => Some(MOV),
            0x2 => Some(LOAD),
            0x3 => Some(STORE),
            0x4 => Some(STORER),
            0x5 => Some(ADD),
            0x6 => Some(SUB),
            0x7 => Some(AND),
            0x8 => Some(OR),
            0x9 => Some(NOT),
            0xA => Some(CMP),
            0xB => Some(BRZ),
            0xC => Some(BRN),
            0xD => Some(BRC),
            0xE => Some(JMP),
            0xF => Some(HALT),
            _ => None,
        }
    }
}

/// The structured representation of an instruction.
/// This guarantees the Assembler and CPU agree on the arguments.
#[derive(Debug, PartialEq)]
pub enum Instruction {
    LoadI { dst: u8, imm: u8 },
    Mov { dst: u8, src: u8 },
    Load { dst: u8, addr: u8 },
    Store { src: u8, addr: u8 },
    StoreIndirect { src: u8, ptr: u8 },
    Add { dst: u8, src: u8 },
    Sub { dst: u8, src: u8 },
    And { dst: u8, src: u8 },
    Or { dst: u8, src: u8 },
    Not { dst: u8 },
    Cmp { reg1: u8, reg2: u8 },
    Brz { addr: u8 },
    Brn { addr: u8 },
    Brc { addr: u8 },
    Jmp { addr: u8 },
    Halt,
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
        use Opcode::*;
        match self {
            LoadI { dst, imm } => Self::pack(LOADI, *dst, *imm),
            Mov { dst, src } => Self::pack(MOV, *dst, *src),
            Load { dst, addr } => Self::pack(LOAD, *dst, *addr),
            Store { src, addr } => Self::pack(STORE, *src, *addr),
            StoreIndirect { src, ptr } => Self::pack(STORER, *src, *ptr),
            Add { dst, src } => Self::pack(ADD, *dst, *src),
            Sub { dst, src } => Self::pack(SUB, *dst, *src),
            And { dst, src } => Self::pack(AND, *dst, *src),
            Or { dst, src } => Self::pack(OR, *dst, *src),
            Not { dst } => Self::pack(NOT, *dst, 0),
            Cmp { reg1, reg2 } => Self::pack(CMP, *reg1, *reg2),
            Brz { addr } => Self::pack(BRZ, 0, *addr),
            Brn { addr } => Self::pack(BRN, 0, *addr),
            Brc { addr } => Self::pack(BRC, 0, *addr),
            Jmp { addr } => Self::pack(JMP, 0, *addr),
            Halt => Self::pack(Opcode::HALT, 0, 0),
        }
    }

    /// CPU: Converts a 16-bit binary instruction back into the structured enum.
    pub fn decode(raw: u16) -> Result<Self, DecodeError> {
        let opcode_raw = ((raw >> 12) & 0x0F) as u8;
        let reg = ((raw >> 8) & 0x0F) as u8;
        let operand = (raw & 0xFF) as u8;

        let opcode = Opcode::from_u8(opcode_raw).ok_or(DecodeError::InvalidOpcode(opcode_raw))?;

        use Instruction::*;
        use Opcode::*;
        match opcode {
            LOADI => Ok(LoadI {
                dst: reg,
                imm: operand,
            }),
            MOV => Ok(Mov {
                dst: reg,
                src: operand,
            }),
            LOAD => Ok(Load {
                dst: reg,
                addr: operand,
            }),
            STORE => Ok(Store {
                src: reg,
                addr: operand,
            }),
            STORER => Ok(StoreIndirect {
                src: reg,
                ptr: operand,
            }),

            ADD => Ok(Add {
                dst: reg,
                src: operand,
            }),
            SUB => Ok(Sub {
                dst: reg,
                src: operand,
            }),
            AND => Ok(And {
                dst: reg,
                src: operand,
            }),
            OR => Ok(Or {
                dst: reg,
                src: operand,
            }),
            NOT => Ok(Not { dst: reg }),
            CMP => Ok(Cmp {
                reg1: reg,
                reg2: operand,
            }),
            BRZ => Ok(Brz { addr: operand }),
            BRN => Ok(Brn { addr: operand }),
            BRC => Ok(Brc { addr: operand }),
            JMP => Ok(Jmp { addr: operand }),
            HALT => Ok(Halt),
        }
    }
}
