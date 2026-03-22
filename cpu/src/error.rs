use colored::*;
use shared::Instruction;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, CpuError>;

#[derive(Error, Debug)]
pub enum CpuError {
    #[error("binary size ({actual}) exceeds memory capacity ({limit})")]
    MemoryOverflow { actual: usize, limit: usize },

    #[error("{} {} jump to 0x{addr:02X} is out of memory bounds (max 0x{limit:02X})",
        "cpu:".dimmed(),
        "error:".red().bold(),
    )]
    InvalidJump { addr: usize, limit: usize },

    #[error("Instruction '{0:?}' is not a valid ALU operation")]
    InvalidAluOperation(Instruction),

    #[error("instruction decoder: {0}")]
    Decode(#[from] shared::DecodeError),
}
