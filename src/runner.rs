use assembler::Assembler;
use cpu::Cpu;
use log::info;
use ratatui::crossterm::cursor::RestorePosition;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode};
use ratatui::{Terminal, TerminalOptions, Viewport, backend::CrosstermBackend};
use std::io::stdout;
use std::path::PathBuf;
use std::{
    fs,
    time::{Duration, Instant},
};

use crate::tui;

pub fn run_cpu(binary: Vec<u8>, hz: u32) -> anyhow::Result<()> {
    execute!(stdout(), RestorePosition, Clear(ClearType::FromCursorDown))?;

    let mut cpu = Cpu::default();
    cpu.load_memory(binary)?;

    enable_raw_mode()?;
    let mut terminal = Terminal::with_options(
        CrosstermBackend::new(stdout()),
        TerminalOptions {
            viewport: Viewport::Inline(28),
        },
    )?;

    let tick_rate = Duration::from_secs_f64(1.0 / hz as f64);
    loop {
        let start = Instant::now();

        if event::poll(Duration::from_millis(0))?
            && let Event::Key(key) = event::read()?
        {
            use KeyCode::*;
            match key.code {
                Char('q') if key.kind == KeyEventKind::Press => {
                    break;
                }
                _ => {}
            }
        }

        cpu.tick()?;

        terminal.draw(|f| tui::render_ui(f, &cpu, hz))?;

        cpu.next_step();

        if let Some(_) = cpu.halted {
            break;
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
