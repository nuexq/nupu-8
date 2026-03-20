use assembler::Assembler;
use clap::{Parser, Subcommand};
use colored::Colorize;
use cpu::Cpu;
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
        Commands::Exec { input, hz } => {
            let mut cpu = Cpu::default();

            let binary = fs::read(&input)?;
            cpu.load_memory(binary)?;

            let tick_rate = std::time::Duration::from_secs_f64(1.0 / hz as f64);

            info!("Starting CPU at {} Hz", hz);

            loop {
                let start = std::time::Instant::now();
                cpu.tick()?;

                if let Some(pc) = cpu.halted {
                    info!("Halt instruction reached. PC: {:#04X}", pc);
                    break;
                }

                let elapsed = start.elapsed();
                if elapsed < tick_rate {
                    std::thread::sleep(tick_rate - elapsed);
                }
            }
        }
        Commands::Asm { input, output } => {
            let mut asm = Assembler::default();

            let content = fs::read_to_string(&input)?;

            let start_compile = std::time::Instant::now();
            let binary = asm.assemble(&content)?;
            let compile_time = start_compile.elapsed();

            info!(
                "{} {} {} bytes {} {:.2?} {}",
                "✔".green(),
                "Assembly successful!".bold(),
                binary.len().to_string().yellow(),
                "(".truecolor(150, 150, 150),
                compile_time,
                ")".truecolor(150, 150, 150)
            );

            fs::write(output, binary)?;
        }
    }

    Ok(())
}
