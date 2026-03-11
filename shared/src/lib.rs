pub enum Opcode {
    LOADI = 0x0,
    MOV = 0x1,
    LOAD = 0x2,
    STORE = 0x3,
    ADD = 0x4,
    SUB = 0x5,
    MUL = 0x6,
    DIV = 0x7,
    CMP = 0x8,
    BRZ = 0xA,
    BRN = 0xB,
    BRC = 0xC,
    JMP = 0xD,
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
            0x6 => Some(Opcode::MUL),
            0x7 => Some(Opcode::DIV),
            0x8 => Some(Opcode::CMP),
            0xA => Some(Opcode::BRZ),
            0xB => Some(Opcode::BRN),
            0xC => Some(Opcode::BRC),
            0xD => Some(Opcode::JMP),
            0xE => Some(Opcode::HALT),
            0xF => Some(Opcode::NOP),
            _ => None,
        }
    }
}
