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
            viewport: Viewport::Inline(26),
        },
    )?;

    let target_fps = 60;
    let frame_duration = Duration::from_micros(1_000_000 / target_fps);

    let mut tick_accumulator: f64 = 0.0;
    let mut last_frame_time = Instant::now();

    loop {
        let frame_start = Instant::now();

        if event::poll(Duration::ZERO)?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            use KeyCode::*;
            match key.code {
                Char('q') => {
                    break;
                }
                Char('r') => {
                    display = NupuDisplay::new();
                    cpu.reset(&binary)?;
                }
                Char(' ') => {
                    cpu_state = match cpu_state {
                        CpuState::Running => CpuState::Paused,
                        CpuState::Paused => CpuState::Running,
                    };
                }
                _ => {}
            }
        }
        let should_run = cpu.halted.is_none() && cpu_state == CpuState::Running;

        if should_run {
            let elapsed = last_frame_time.elapsed().as_secs_f64();
            last_frame_time = Instant::now();

            tick_accumulator += hz as f64 * elapsed;

            while tick_accumulator >= 1.0 {
                cpu.tick()?;
                display.update(&mut cpu.ports);
                cpu.next_step();

                tick_accumulator -= 1.0;

                if cpu.halted.is_some() {
                    break;
                }
            }
        } else {
            last_frame_time = Instant::now();
        }

        terminal.draw(|f| tui::render_ui(f, &cpu, &display.vram, hz, &cpu_state))?;

        let frame_elapsed = frame_start.elapsed();
        if frame_elapsed < frame_duration {
            std::thread::sleep(frame_duration - frame_elapsed);
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
