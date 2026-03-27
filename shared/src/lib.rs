use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum DecodeError {
    #[error("Unknown Opcode 0x{0:X}")]
    InvalidOpcode(u8),

    #[error("Unknown Branch Type {0}")]
    InvalidBranchType(u8),
}

pub const MODE: u8 = 0xFF;
pub const TXT_MODE: u8 = 0;
pub const VRAM_START: u8 = 0x7F;
pub const CURSOR_PTR_ADDR: u8 = 0xFE;

/// The raw hex values for the CPU operations.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Opcode {
    SET = 0x0,
    LOAD = 0x1,
    STORE = 0x2,
    ADD = 0x3,
    SUB = 0x4,
    AND = 0x5,
    OR = 0x6,
    NOT = 0x7,
    CMP = 0x8,
    BRANCH = 0x9,
    SHIFT = 0xA,
    OUT = 0xB,
    HALT = 0xF,
}

impl Opcode {
    pub fn from_u8(value: u8) -> Option<Self> {
        use Opcode::*;
        match value {
            0x0 => Some(SET),
            0x1 => Some(LOAD),
            0x2 => Some(STORE),
            0x3 => Some(ADD),
            0x4 => Some(SUB),
            0x5 => Some(AND),
            0x6 => Some(OR),
            0x7 => Some(NOT),
            0x8 => Some(CMP),
            0x9 => Some(BRANCH),
            0xA => Some(SHIFT),
            0xB => Some(OUT),
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
    LoadIndirect { dst: u8, ptr: u8 },
    Store { src: u8, addr: u8 },
    StoreIndirect { src: u8, ptr: u8 },
    Add { dst: u8, src: u8 },
    AddI { dst: u8, imm: u8 },
    Sub { dst: u8, src: u8 },
    SubI { dst: u8, imm: u8 },
    And { dst: u8, src: u8 },
    AndI { dst: u8, imm: u8 },
    Or { dst: u8, src: u8 },
    OrI { dst: u8, imm: u8 },
    Not { src: u8 },
    NotI { imm: u8 },
    Cmp { reg1: u8, reg2: u8 },
    CmpI { reg: u8, imm: u8 },
    Brz { addr: u8 },
    Brn { addr: u8 },
    Brc { addr: u8 },
    Jmp { addr: u8 },
    Shl { src: u8, amt: u8 },
    Shr { src: u8, amt: u8 },
    Out { reg: u8, port: u8 },
    Halt,
}

impl Instruction {
    /// Internal helper to pack the bits safely.
    /// Format: [4 bits opcode] [3 bits reg] [1 bit unused] [8 bits operand]
    fn pack(opcode: Opcode, mode: u8, reg: u8, operand: u8) -> u16 {
        let op_shifted = (opcode as u16) << 12;
        let mode_shifted = (mode as u16) << 11;
        let reg_shifted = ((reg & 0x07) as u16) << 8;
        let operand_cast = operand as u16;

        op_shifted | mode_shifted | reg_shifted | operand_cast
    }

    /// ASSEMBLER: Converts the structured enum into a 16-bit binary instruction.
    pub fn encode(&self) -> u16 {
        use Instruction::*;
        use Opcode::*;
        match self {
            LoadI { dst, imm } => Self::pack(SET, 0, *dst, *imm),
            Mov { dst, src } => Self::pack(SET, 1, *dst, *src),
            Load { dst, addr } => Self::pack(LOAD, 0, *dst, *addr),
            LoadIndirect { dst, ptr } => Self::pack(LOAD, 1, *dst, *ptr),
            Store { src, addr } => Self::pack(STORE, 0, *src, *addr),
            StoreIndirect { src, ptr } => Self::pack(STORE, 1, *src, *ptr),
            Add { dst, src } => Self::pack(ADD, 0, *dst, *src),
            AddI { dst, imm } => Self::pack(ADD, 1, *dst, *imm),
            Sub { dst, src } => Self::pack(SUB, 0, *dst, *src),
            SubI { dst, imm } => Self::pack(SUB, 1, *dst, *imm),
            And { dst, src } => Self::pack(AND, 0, *dst, *src),
            AndI { dst, imm } => Self::pack(AND, 1, *dst, *imm),
            Or { dst, src } => Self::pack(OR, 0, *dst, *src),
            OrI { dst, imm } => Self::pack(OR, 1, *dst, *imm),
            Not { src } => Self::pack(NOT, 0, *src, 0),
            NotI { imm } => Self::pack(NOT, 1, 0, *imm),
            Cmp { reg1, reg2 } => Self::pack(CMP, 0, *reg1, *reg2),
            CmpI { reg, imm } => Self::pack(CMP, 1, *reg, *imm),
            Jmp { addr } => Self::pack(BRANCH, 0, 0, *addr),
            Brz { addr } => Self::pack(BRANCH, 0, 1, *addr),
            Brn { addr } => Self::pack(BRANCH, 0, 2, *addr),
            Brc { addr } => Self::pack(BRANCH, 0, 3, *addr),
            Shl { src, amt } => Self::pack(SHIFT, 0, *src, *amt),
            Shr { src, amt } => Self::pack(SHIFT, 1, *src, *amt),
            Out { reg, port } => Self::pack(OUT, 0, *reg, *port),
            Halt => Self::pack(HALT, 0, 0, 0),
        }
    }

    /// CPU: Converts a 16-bit binary instruction back into the structured enum.
    pub fn decode(raw: u16) -> Result<Self, DecodeError> {
        let opcode_raw = ((raw >> 12) & 0x0F) as u8;
        let mode = ((raw >> 11) & 0x01) as u8;
        let reg = ((raw >> 8) & 0x07) as u8;
        let operand = (raw & 0xFF) as u8;

        let opcode = Opcode::from_u8(opcode_raw).ok_or(DecodeError::InvalidOpcode(opcode_raw))?;

        use Instruction::*;
        use Opcode::*;
        match (opcode, mode) {
            (SET, 0) => Ok(LoadI {
                dst: reg,
                imm: operand,
            }),
            (SET, 1) => Ok(Mov {
                dst: reg,
                src: operand,
            }),
            (LOAD, 0) => Ok(Load {
                dst: reg,
                addr: operand,
            }),
            (LOAD, 1) => Ok(LoadIndirect {
                dst: reg,
                ptr: operand,
            }),
            (STORE, 0) => Ok(Store {
                src: reg,
                addr: operand,
            }),
            (STORE, 1) => Ok(StoreIndirect {
                src: reg,
                ptr: operand,
            }),
            (ADD, 0) => Ok(Add {
                dst: reg,
                src: operand,
            }),
            (ADD, 1) => Ok(AddI {
                dst: reg,
                imm: operand,
            }),
            (SUB, 0) => Ok(Sub {
                dst: reg,
                src: operand,
            }),
            (SUB, 1) => Ok(SubI {
                dst: reg,
                imm: operand,
            }),
            (AND, 0) => Ok(And {
                dst: reg,
                src: operand,
            }),
            (AND, 1) => Ok(AndI {
                dst: reg,
                imm: operand,
            }),
            (OR, 0) => Ok(Or {
                dst: reg,
                src: operand,
            }),
            (OR, 1) => Ok(OrI {
                dst: reg,
                imm: operand,
            }),
            (NOT, 0) => Ok(Not { src: reg }),
            (NOT, 1) => Ok(NotI { imm: operand }),
            (CMP, 0) => Ok(Cmp {
                reg1: reg,
                reg2: operand,
            }),
            (CMP, 1) => Ok(CmpI { reg, imm: operand }),
            (BRANCH, 0) => match reg {
                0 => Ok(Jmp { addr: operand }),
                1 => Ok(Brz { addr: operand }),
                2 => Ok(Brn { addr: operand }),
                3 => Ok(Brc { addr: operand }),
                _ => Err(DecodeError::InvalidBranchType(reg)),
            },
            (SHIFT, 0) => Ok(Shl {
                src: reg,
                amt: operand,
            }),
            (SHIFT, 1) => Ok(Shr {
                src: reg,
                amt: operand,
            }),
            (OUT, 0) => Ok(Out {
                reg,
                port: operand,
            }),
            (HALT, 0) => Ok(Halt),
            _ => unreachable!(),
        }
    }
}
