mod runner;
mod tui;

use clap::{Parser, Subcommand};
use colored::*;
use std::{
    fs,
    io::Write,
    path::PathBuf,
};

#[derive(Parser, Debug)]
#[command(name = "nupu-8", about = "An 8-bit CPU", version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Assemble an .asm file and run it immediately on the CPU
    Run {
        /// The .asm source file
        input: PathBuf,

        /// Optional: Set clock speed in Hz
        #[arg(long, default_value_t = 100)]
        hz: u32,
    },

    /// Execute a pre-compiled binary directly on the CPU
    Exec {
        /// The .bin file to execute
        input: PathBuf,

        /// Optional: Set clock speed in Hz
        #[arg(long, default_value_t = 100)]
        hz: u32,
    },

    /// Assemble an 8-bit .asm file
    Asm {
        /// The .asm file to read
        input: PathBuf,

        /// The output binary file
        #[arg(short, long, default_value = "out.bin")]
        output: PathBuf,
    },
}

fn main() {
    env_logger::Builder::from_default_env()
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .filter_level(log::LevelFilter::Info)
        .init();

    if let Err(err) = try_main() {
        eprintln!("{} {}", "error".red().bold(), err);
        std::process::exit(1);
    }
}

fn try_main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { input, hz } => {
            log::set_max_level(log::LevelFilter::Warn); // remove from assembler
            let binary = runner::run_assembler(input)?;
            runner::run_cpu(binary, hz)?;
        }
        Commands::Exec { input, hz } => {
            let binary = fs::read(&input)?;
            runner::run_cpu(binary, hz)?;
        }
        Commands::Asm { input, output } => {
            let binary = runner::run_assembler(input)?;
            fs::write(output, binary)?;
        }
    }
    Ok(())
}
