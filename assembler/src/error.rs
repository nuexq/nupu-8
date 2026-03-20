use colored::*;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, AssemblerError>;

#[derive(Error, Debug)]
pub enum AssemblerError {
    #[error("{} line {line}: unknown opcode '{}'",
        "error".red().bold(),
        opcode.yellow()
    )]
    UnknownOpcode { line: usize, opcode: String },

    #[error("{} line {line}: invalid register '{}'. must be r0-r15",
        "error".red().bold(),
        reg.yellow()
    )]
    InvalidRegister { line: usize, reg: String },

    #[error("{} line {line}: label '{}' not found",
        "error".red().bold(),
        label.yellow()
    )]
    UnknownLabel { line: usize, label: String },

    #[error("{} line {line}: value '{}' is too large for 8 bits",
        "error".red().bold(),
        val.to_string().yellow()
    )]
    ValueOverflow { line: usize, val: u16 },

    #[error("{} line {line}: missing argument for instruction",
        "error".red().bold()
    )]
    MissingArgument { line: usize },

    #[error("{} line {line}: invalid number format '{}'",
        "error".red().bold(),
        val.yellow()
    )]
    InvalidNumber { line: usize, val: String },
}
