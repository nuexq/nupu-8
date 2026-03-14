use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::fmt;

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
    /// Run a compiled binary on the virtual CPU
    Run {
        /// The binary file to execute
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
    fmt()
        .without_time()
        .with_target(true)
        .with_level(true)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run { binary, hz } => {
            info!(?binary, "reading binary");
            info!(hz, "clock speed");
        }

        Commands::Asm { input, output } => {
            info!(?input, "reading file");
            info!(?output, "writing output");
        }
    }
}
