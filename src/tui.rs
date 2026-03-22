use cpu::Cpu;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    widgets::{
        Paragraph,
    },
};

pub fn render_ui(f: &mut ratatui::Frame, cpu: &Cpu) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(f.area());

    let status = format!(
        "PC: {:#04X} | MODE: {} | PORT: {:02X}",
        cpu.pc,
        if cpu.memory[0x7F] == 1 { "PIX" } else { "TXT" },
        cpu.memory[0xFF]
    );
    f.render_widget(Paragraph::new(status).dark_gray(), chunks[0]);
}
