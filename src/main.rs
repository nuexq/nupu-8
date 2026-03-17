use assembler::Assembler;
use clap::{Parser, Subcommand};
use colored::Colorize;
use log::info;
use std::io::Write;
use std::{fs, path::PathBuf};

#[derive(Parser, Debug)]
#[command(
    name = "nupu-8",
    about = "A simple 8-bit cpu",
    version = env!("CARGO_PKG_VERSION")
)]
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
        binary: PathBuf,

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
        .format(|buf, record| writeln!(buf, "{}: {}", record.target().dimmed(), record.args()))
        .filter_level(log::LevelFilter::Info)
        .init();
    if let Err(err) = try_main() {
        eprintln!("{}", err);

        std::process::exit(1);
    }
}

fn try_main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { input, hz } => {
            info!("reading binary {:?}", input);
            info!("clock speed: {:?}", hz);
        }
        Commands::Exec { binary, hz } => {
            info!("reading binary {:?}", binary);
            info!("clock speed: {:?}", hz);
        }
        Commands::Asm { input, output } => {
            let mut asm = Assembler::new();

            let content = fs::read_to_string(&input)?;
            let binary = asm.assemble(&content)?;

            log::info!(
                "{} {} {}",
                "✔".green(),
                "assembly successful".bold(),
                format!("-> {}", output.display()).cyan()
            );

            fs::write(output, binary)?;
        }
    }

    Ok(())
}
