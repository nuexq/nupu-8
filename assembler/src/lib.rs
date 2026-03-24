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

            use Instruction::*;
            let instruction = match opcode {
                "loadi" => {
                    let reg_str = tokens
                        .get(1)
                        .ok_or(AssemblerError::MissingArgument { line: line_num })?;
                    let imm_str = tokens
                        .get(2)
                        .ok_or(AssemblerError::MissingArgument { line: line_num })?;

                    LoadI {
                        dst: parse_reg(reg_str, line_num)?,
                        imm: parse_imm(imm_str, line_num)?,
                    }
                }
                "mov" | "add" | "sub" | "cmp" | "and" | "or" => {
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
                        "mov" => Mov { dst: r1, src: r2 },
                        "add" => Add { dst: r1, src: r2 },
                        "sub" => Sub { dst: r1, src: r2 },
                        "and" => And { dst: r1, src: r2 },
                        "or" => Or { dst: r1, src: r2 },
                        "cmp" => Cmp { reg1: r1, reg2: r2 },
                        _ => unreachable!(),
                    }
                }
                "load" => {
                    let dst = parse_reg(
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

                    Load { dst, addr }
                }
                "store" => {
                    let src = parse_reg(
                        tokens
                            .get(1)
                            .ok_or(AssemblerError::MissingArgument { line: line_num })?,
                        line_num,
                    )?;
                    let target = tokens
                        .get(2)
                        .ok_or(AssemblerError::MissingArgument { line: line_num })?;

                    if target.starts_with('[') && target.ends_with(']') {
                        let inner = &target[1..target.len() - 1];
                        let ptr = parse_reg(inner, line_num)?;

                        StoreIndirect { src, ptr }
                    } else {
                        let addr = parse_imm(target, line_num)?;

                        Store { src, addr }
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
                        "brz" => Brz { addr },
                        "brn" => Brn { addr },
                        "brc" => Brc { addr },
                        "jmp" => Jmp { addr },
                        _ => unreachable!(),
                    }
                }
                "not" => {
                    let reg = parse_reg(
                        tokens
                            .get(1)
                            .ok_or(AssemblerError::MissingArgument { line: line_num })?,
                        line_num,
                    )?;

                    Not { dst: reg }
                }
                "halt" => Instruction::Halt,
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
