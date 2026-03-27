use assembler::Assembler;
use cpu::Cpu;
use log::info;
use ratatui::{
    Terminal, TerminalOptions, Viewport,
    backend::CrosstermBackend,
    crossterm::{
        cursor,
        event::{self, Event, KeyCode, KeyEventKind},
        execute, terminal,
        terminal::{disable_raw_mode, enable_raw_mode},
    },
};
use std::io::stdout;
use std::path::PathBuf;
use std::{
    fs,
    time::{Duration, Instant},
};

use crate::{display::NupuDisplay, tui};

#[derive(PartialEq)]
pub enum CpuState {
    Running,
    Paused,
}

pub fn run_cpu(binary: Vec<u8>, hz: u32) -> anyhow::Result<()> {
    let mut stdout = stdout();
    execute!(
        stdout,
        cursor::MoveToColumn(0),
        terminal::Clear(terminal::ClearType::CurrentLine)
    )?;

    let mut cpu = Cpu::default();
    cpu.load_memory(&binary)?;

    let mut display = NupuDisplay::new();

    let mut cpu_state = CpuState::Running;

    enable_raw_mode()?;
    let mut terminal = Terminal::with_options(
        CrosstermBackend::new(stdout),
        TerminalOptions {
            viewport: Viewport::Inline(34),
        },
    )?;

    let tick_rate = Duration::from_secs_f64(1.0 / hz as f64);
    loop {
        let start = Instant::now();

        if event::poll(Duration::ZERO)?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            use KeyCode::*;
            match key.code {
                Char('q') => {
                    break;
                }
                Char('r') => cpu.reset(&binary)?,
                Char(' ') => {
                    cpu_state = match cpu_state {
                        CpuState::Running => CpuState::Paused,
                        CpuState::Paused => CpuState::Running,
                    };
                }
                _ => {}
            }
        }
        let not_halted = cpu.halted.is_none();
        let should_run = not_halted && cpu_state == CpuState::Running;

        if should_run {
            cpu.tick()?;
            display.update(&mut cpu.ports);
        }

        terminal.draw(|f| tui::render_ui(f, &cpu, &display.vram, hz, &cpu_state))?;

        if should_run {
            cpu.next_step();
        }

        let elapsed = start.elapsed();
        if elapsed < tick_rate {
            std::thread::sleep(tick_rate - elapsed);
        }
    }

    disable_raw_mode()?;
    terminal.clear()?;
    Ok(())
}

pub fn run_assembler(input: PathBuf) -> anyhow::Result<Vec<u8>> {
    let mut asm = Assembler::default();
    let content = fs::read_to_string(&input)?;
    let binary = asm.assemble(&content)?;
    info!("✔ Assembly successful! ({} bytes)", binary.len());
    Ok(binary)
}
