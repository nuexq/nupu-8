use cpu::Cpu;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    widgets::Paragraph,
};
use shared::{MODE, TXT_MODE};

pub fn render_ui(f: &mut ratatui::Frame, cpu: &Cpu) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(f.area());

    let status = format!(
        "PC: {:#04X} | MODE: {}",
        cpu.pc,
        if cpu.memory[MODE as usize] == TXT_MODE {
            "TXT"
        } else {
            "PIX"
        },
    );
    f.render_widget(Paragraph::new(status).dark_gray(), chunks[0]);
}
