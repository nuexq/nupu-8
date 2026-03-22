use colored::*;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AssemblerError>;

#[derive(Error, Debug)]
pub enum AssemblerError {
    #[error("line {line}: unknown opcode '{}'",
        opcode.yellow()
    )]
    UnknownOpcode { line: usize, opcode: String },

    #[error("line {line}: invalid register '{}'. must be r0-r15",
        reg.yellow()
    )]
    InvalidRegister { line: usize, reg: String },

    #[error("line {line}: label '{}' not found",
        label.yellow()
    )]
    UnknownLabel { line: usize, label: String },

    #[error("line {line}: value '{}' is too large for 8 bits",
        val.to_string().yellow()
    )]
    ValueOverflow { line: usize, val: u16 },

    #[error("line {line}: missing argument for instruction")]
    MissingArgument { line: usize },

    #[error("line {line}: invalid number format '{}'",
        val.yellow()
    )]
    InvalidNumber { line: usize, val: String },
}
