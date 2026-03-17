use crate::error::{AssemblerError, Result};
use colored::Colorize;
use shared::Instruction;
use std::collections::HashMap;

mod error;

#[derive(Default)]
pub struct Assembler {
    symbol_table: HashMap<String, u16>,
}

impl Assembler {
    pub fn assemble(&mut self, input: &str) -> Result<Vec<u8>> {
        let mut address_counter = 0;
        let mut raw_instructions: Vec<(usize, &str)> = Vec::new();
        let mut binary_output: Vec<u8> = Vec::new();

        for (i, line) in input.lines().enumerate() {
            let line_num = i + 1;
            let line = line.trim();

            if line.is_empty() || line.starts_with(';') {
                continue;
            }

            let code = line.split(';').next().unwrap().trim();

            if code.ends_with(':') {
                let label = code.strip_suffix(':').unwrap().to_string();
                self.symbol_table.insert(label, address_counter);
                continue;
            }

            raw_instructions.push((line_num, code));
            address_counter += 2;
        }

        // Encoding
        for (line_num, instr) in raw_instructions {
            let tokens: Vec<&str> = instr
                .split([' ', ',', '\t'])
                .filter(|s| !s.is_empty())
                .collect();

            let opcode = tokens[0];

            let instruction = match opcode {
                "loadi" => {
                    let reg_str = tokens
                        .get(1)
                        .ok_or(AssemblerError::MissingArgument { line: line_num })?;
                    let imm_str = tokens
                        .get(2)
                        .ok_or(AssemblerError::MissingArgument { line: line_num })?;

                    Instruction::LoadI {
                        dest: parse_reg(reg_str, line_num)?,
                        imm: parse_imm(imm_str, line_num)?,
                    }
                }
                "mov" | "add" | "sub" | "cmp" => {
                    let r1 = parse_reg(
                        tokens
                            .get(1)
                            .ok_or(AssemblerError::MissingArgument { line: line_num })?,
                        line_num,
                    )?;
                    let r2 = parse_reg(
                        tokens
                            .get(2)
                            .ok_or(AssemblerError::MissingArgument { line: line_num })?,
                        line_num,
                    )?;

                    match opcode {
                        "mov" => Instruction::Mov { dest: r1, src: r2 },
                        "add" => Instruction::Add { dest: r1, src: r2 },
                        "sub" => Instruction::Sub { dest: r1, src: r2 },
                        "cmp" => Instruction::Cmp { reg1: r1, reg2: r2 },
                        _ => unreachable!(),
                    }
                }
                "load" | "store" => {
                    let reg = parse_reg(
                        tokens
                            .get(1)
                            .ok_or(AssemblerError::MissingArgument { line: line_num })?,
                        line_num,
                    )?;
                    let addr = parse_imm(
                        tokens
                            .get(2)
                            .ok_or(AssemblerError::MissingArgument { line: line_num })?,
                        line_num,
                    )?;

                    match opcode {
                        "load" => Instruction::Load { dest: reg, addr },
                        "store" => Instruction::Store { src: reg, addr },
                        _ => unreachable!(),
                    }
                }
                "brz" | "brn" | "brc" | "jmp" => {
                    let target_str = tokens
                        .get(1)
                        .ok_or(AssemblerError::MissingArgument { line: line_num })?;

                    let addr = if let Some(&address) = self.symbol_table.get(*target_str) {
                        address as u8
                    } else {
                        parse_imm(target_str, line_num)?
                    };

                    match opcode {
                        "brz" => Instruction::Brz { addr },
                        "brn" => Instruction::Brn { addr },
                        "brc" => Instruction::Brc { addr },
                        "jmp" => Instruction::Jmp { addr },
                        _ => unreachable!(),
                    }
                }
                "print" => Instruction::Print {
                    src: parse_reg(
                        tokens
                            .get(1)
                            .ok_or(AssemblerError::MissingArgument { line: line_num })?,
                        line_num,
                    )?,
                },
                "halt" => Instruction::Halt,
                "nop" => Instruction::Nop,
                _ => {
                    return Err(AssemblerError::UnknownOpcode {
                        line: line_num,
                        opcode: opcode.to_string(),
                    });
                }
            };
            let bytes = instruction.encode();

            let high = (bytes >> 8) as u8;
            let low = bytes as u8;

            binary_output.push(high);
            binary_output.push(low);

            log::info!(
                "line {} {}",
                format!("{:02}", line_num).cyan(),
                format!("{:?}", instr).bold()
            );

            log::info!("encoded {}", format!("{:016b}", bytes).green(),);
        }

        Ok(binary_output)
    }
}

fn parse_reg(reg: &str, line: usize) -> Result<u8> {
    let parsed = reg
        .strip_prefix('r')
        .ok_or_else(|| AssemblerError::InvalidRegister {
            line,
            reg: reg.to_string(),
        })?
        .parse::<u8>()
        .map_err(|_| AssemblerError::InvalidRegister {
            line,
            reg: reg.to_string(),
        })?;

    // Validate it's strictly 0-15
    if parsed > 15 {
        return Err(AssemblerError::InvalidRegister {
            line,
            reg: reg.to_string(),
        });
    }

    Ok(parsed)
}

fn parse_imm(imm: &str, line: usize) -> Result<u8> {
    let val = if let Some(hex_str) = imm.strip_prefix("0x") {
        u16::from_str_radix(hex_str, 16).map_err(|_| AssemblerError::InvalidNumber {
            line,
            val: imm.to_string(),
        })?
    } else {
        imm.parse::<u16>()
            .map_err(|_| AssemblerError::InvalidNumber {
                line,
                val: imm.to_string(),
            })?
    };

    if val > 255 {
        return Err(AssemblerError::ValueOverflow { line, val });
    }

    Ok(val as u8)
}
