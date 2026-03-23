use cpu::Cpu;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::{Line, Span, Text},
};
use shared::{MODE, TXT_MODE};

pub fn render_ui(f: &mut ratatui::Frame, cpu: &Cpu, hz: u32) {
    let mode_is_txt = cpu.memory[MODE as usize] == TXT_MODE;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(4),
            Constraint::Length(2),
            Constraint::Length(18),
        ])
        .split(f.area());

    // Main title
    let title_content = Line::from(vec![
        Span::styled("NUPU-8 ", Style::default().bold()),
        Span::styled(
            format!(" v{} ", env!("CARGO_PKG_VERSION")),
            Style::default().dark_gray(),
        ),
        Span::styled(" - ", Style::default().gray()),
        Span::styled(format!("{} Hz", hz), Style::default().gray()),
    ]);

    f.render_widget(title_content, chunks[0]);

    // Status
    let status_content = Line::from(vec![
        // PC
        Span::styled("PC ", Style::default().gray()),
        Span::styled(
            format!(" {:#04X} ", cpu.pc),
            Style::default().bold().reversed(),
        ),
        Span::raw(" "),
        // MODE
        Span::styled(" MODE ", Style::default().gray()),
        Span::styled(
            if mode_is_txt { " TXT " } else { " PIX " },
            Style::default().bold().reversed(),
        ),
        Span::raw("  "),
        // STEP
        Span::styled("STEP: ", Style::default().gray()),
        Span::styled(cpu.step.to_string(), Style::default().white().bold()),
        Span::raw("  "),
        // IR
        Span::styled("IR: ", Style::default().gray()),
        Span::styled(format!("{:#06X}", cpu.ir), Style::default().white().bold()),
    ]);
    f.render_widget(status_content, chunks[1]);

    // Registers
    let mut registers_lines = vec![Line::styled("REGISTERS", Style::default().bold())];

    for (i, chunk) in cpu.registers.chunks(8).enumerate() {
        let mut spans = vec![];

        // label (r[a-b]):
        spans.push(Span::styled(
            format!("r[{:02}-{:02}]:", i * 8, i * 8 + chunk.len() - 1),
            Style::default().gray(),
        ));

        // values
        for &val in chunk {
            let style = if val == 0 {
                Style::default().dark_gray()
            } else {
                Style::default().white().bold()
            };
            spans.push(Span::styled(format!(" {:#04X}", val), style));
        }

        registers_lines.push(Line::from(spans));
    }

    let registers_content = Text::from(registers_lines);

    f.render_widget(registers_content, chunks[2]);

    // Flags
    let z = cpu.flags.zero;
    let n = cpu.flags.negative;
    let c = cpu.flags.carry;

    let style_flag = |label: &str, active: bool| -> Vec<Span> {
        if active {
            vec![Span::styled(
                format!(" {} ", label),
                Style::default().bold().reversed(),
            )]
        } else {
            vec![Span::styled(
                format!(" {} ", label),
                Style::default().dark_gray(),
            )]
        }
    };

    let mut flag_spans = vec![Span::styled("FLAGS: ", Style::default().bold())];

    flag_spans.extend(style_flag("Z", z));
    flag_spans.push(Span::raw(" "));
    flag_spans.extend(style_flag("N", n));
    flag_spans.push(Span::raw(" "));
    flag_spans.extend(style_flag("C", c));

    let flags_content = Text::from(vec![Line::from(flag_spans)]);

    f.render_widget(flags_content, chunks[3]);

    // Memory
    let mut memory_lines = vec![Line::styled("MEMORY", Style::default().bold())];

    for (i, chunk) in cpu.memory.chunks(16).enumerate() {
        let mut spans = vec![];

        // Address Label (0xAB)
        spans.push(Span::styled(
            format!("{:#04X}:", i * 16),
            Style::default().gray(),
        ));

        // values 0xAB:
        for (col, &val) in chunk.iter().enumerate() {
            let addr = (i * 16) + col;

            let is_pc = addr == cpu.pc as usize;
            let is_next_byte = addr == (cpu.pc as usize).wrapping_add(1);

            let val_style = if is_pc || (cpu.step == 1 && is_next_byte) {
                Style::default().bold().reversed()
            } else if val == 0 {
                Style::default().dark_gray()
            } else {
                Style::default().white().bold()
            };

            let space_style = if cpu.step == 1 && is_next_byte {
                Style::default().bold().reversed()
            } else {
                Style::default()
            };

            spans.push(Span::styled(" ", space_style));
            spans.push(Span::styled(format!("{:02X}", val), val_style));
        }

        memory_lines.push(Line::from(spans));
    }

    let memory_content = Text::from(memory_lines);

    f.render_widget(memory_content, chunks[4]);
}
