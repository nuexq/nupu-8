use cpu::Cpu;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Paragraph,
        canvas::{Canvas, Points},
    },
};
use shared::{CURSOR_PTR_ADDR, MODE, TXT_MODE, VRAM_START};

pub fn render_ui(f: &mut ratatui::Frame, cpu: &Cpu, hz: u32) {
    let mode_is_txt = cpu.memory[MODE as usize] == TXT_MODE;

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .spacing(4)
        .constraints([Constraint::Length(53), Constraint::Max(32)])
        .split(f.area());

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // main title
            Constraint::Length(2),  // status (+empty line)
            Constraint::Length(2),  // registers
            Constraint::Length(2),  // flags (+empty line)
            Constraint::Length(18), // memory (+empty line)
            Constraint::Length(1),  // footer
        ])
        .split(main_chunks[0]);

    let right_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(16), Constraint::Min(0)])
        .split(main_chunks[1]);

    let right_chunk = right_layout[0];

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

    f.render_widget(title_content, left_chunks[0]);

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
    f.render_widget(status_content, left_chunks[1]);

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

    f.render_widget(registers_content, left_chunks[2]);

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

    f.render_widget(flags_content, left_chunks[3]);

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

    f.render_widget(memory_content, left_chunks[4]);

    // Footer
    let key_style = Style::default().bold().reversed();
    let footer_content = Line::from(vec![
        Span::styled(" Q ", key_style),
        Span::styled(" Quit", Style::default().gray()),
        Span::raw("   "),
        Span::styled(" R ", key_style),
        Span::styled(" Reset", Style::default().gray()),
    ]);
    f.render_widget(Paragraph::new(footer_content), left_chunks[5]);

    // Display
    let display_block = Block::default()
        .title(" OUTPUT ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    if !mode_is_txt {
        // PIX MODE: 32x32 screen
        let canvas = Canvas::default()
            .block(display_block)
            .x_bounds([0.0, 31.0])
            .y_bounds([0.0, 31.0])
            .marker(ratatui::symbols::Marker::HalfBlock)
            .paint(|ctx| {
                for i in 0..128 {
                    let byte = cpu.memory[VRAM_START as usize + i];
                    for bit in 0..8 {
                        if (byte >> (7 - bit)) & 1 == 1 {
                            let x = (i % 4) * 8 + bit;
                            let y = 31 - (i / 4);
                            ctx.draw(&Points {
                                coords: &[(x as f64, y as f64)],
                                color: Color::White,
                            });
                        }
                    }
                }
            });
        f.render_widget(canvas, right_chunk);
    } else {
        // TXT MODE
        let cursor_val = cpu.memory[CURSOR_PTR_ADDR as usize] as usize;
        let mut display_text = String::new();

        if cursor_val != 0 && cursor_val < CURSOR_PTR_ADDR as usize {
            let text_data = &cpu.memory[cursor_val..CURSOR_PTR_ADDR as usize];

            let clean_text: String = text_data
                .iter()
                .filter(|&&b| b != 0)
                .map(|&b| b as char)
                .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
                .collect();

            display_text = clean_text.chars().rev().collect();
        }

        f.render_widget(
            Paragraph::new(display_text)
                .block(display_block)
                .wrap(ratatui::widgets::Wrap { trim: false }),
            right_chunk,
        );
    }
}
