use crate::app::{App, PopupButton};
use ratatui::{
    layout::{Constraint, Layout, Rect, Flex},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table},
    Frame,
};

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

    render_table(frame, app, chunks[0]);
    render_footer(frame, app, chunks[1]);

    if app.show_terminate_popup {
        render_terminate_popup(frame, app);
    }
}

fn render_table(frame: &mut Frame, app: &mut App, area: Rect) {
    let header_style = Style::default().add_modifier(Modifier::BOLD);
    let header = Row::new(vec![
        Cell::from("Port").style(header_style),
        Cell::from("PID").style(header_style),
        Cell::from("Process").style(header_style),
        Cell::from("Path").style(header_style),
    ])
    .height(1)
    .bottom_margin(1);

    let rows: Vec<Row> = app
        .ports
        .iter()
        .map(|p| {
            let path = p
                .exe_path
                .as_ref()
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or_else(|| "-".to_string());

            // Apply horizontal scroll offset to path
            let scrolled_path = if app.scroll_offset as usize >= path.len() {
                String::new()
            } else {
                path.chars().skip(app.scroll_offset as usize).collect()
            };

            Row::new(vec![
                Cell::from(p.port.to_string()),
                Cell::from(p.pid.to_string()),
                Cell::from(p.process_name.clone()),
                Cell::from(scrolled_path),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Length(20),
        Constraint::Fill(1),
    ];

    let title = format!(" Listening TCP Ports ({}) ", app.ports.len());

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        )
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(table, area, &mut app.table_state);

    if let Some(error) = &app.error {
        let error_msg = Paragraph::new(format!("Error: {error}"))
            .style(Style::default().fg(Color::Red));
        let error_area = Rect {
            x: area.x + 2,
            y: area.y + 3,
            width: area.width.saturating_sub(4),
            height: 1,
        };
        frame.render_widget(error_msg, error_area);
    }
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let key_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
    let status_style = Style::default().fg(Color::Cyan);

    let mut spans = vec![
        Span::raw(" "),
        Span::styled("q", key_style),
        Span::raw(" quit  "),
        Span::styled("r", key_style),
        Span::raw(" refresh  "),
        Span::styled("Enter/t", key_style),
        Span::raw(" details  "),
        Span::styled("\u{2190}/h", key_style),
        Span::raw(" scroll left  "),
        Span::styled("\u{2192}/l", key_style),
        Span::raw(" scroll right  "),
        Span::styled("\u{2191}/k", key_style),
        Span::raw(" up  "),
        Span::styled("\u{2193}/j", key_style),
        Span::raw(" down"),
    ];

    if let Some(status) = &app.status_message {
        spans.push(Span::raw("  |  "));
        spans.push(Span::styled(status.clone(), status_style));
    }

    let footer = Line::from(spans);
    frame.render_widget(Paragraph::new(footer), area);
}

fn render_terminate_popup(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Calculate popup size - wider and taller to show more content
    let popup_width = 80.min(area.width.saturating_sub(4));
    let popup_height = 24.min(area.height.saturating_sub(4));

    // Center the popup
    let popup_area = centered_rect(popup_width, popup_height, area);

    // Clear the background
    frame.render_widget(Clear, popup_area);

    // Get selected port info
    let port_info = app.get_selected_port();
    let content_width = popup_width as usize - 4;

    let (title, details) = if let Some(p) = port_info {
        let path_str = p.exe_path
            .as_ref()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_else(|| "-".to_string());

        let cwd_str = p.cwd
            .as_ref()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_else(|| "-".to_string());

        let cmd_str = if p.cmd_args.is_empty() {
            "-".to_string()
        } else {
            p.cmd_args.join(" ")
        };

        let mut lines = vec![
            Line::from(vec![
                Span::styled("Process: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&p.process_name),
            ]),
            Line::from(vec![
                Span::styled("PID:     ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(p.pid.to_string()),
            ]),
            Line::from(vec![
                Span::styled("Port:    ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(p.port.to_string()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Path:", Style::default().add_modifier(Modifier::BOLD)),
            ]),
        ];

        // Add wrapped path lines
        for line in wrap_text(&path_str, content_width) {
            lines.push(Line::from(line));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Working Dir:", Style::default().add_modifier(Modifier::BOLD)),
        ]));

        // Add wrapped cwd lines
        for line in wrap_text(&cwd_str, content_width) {
            lines.push(Line::from(line));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Command:", Style::default().add_modifier(Modifier::BOLD)),
        ]));

        // Add wrapped command lines
        for line in wrap_text(&cmd_str, content_width) {
            lines.push(Line::from(line));
        }

        (format!(" Process Details (Port {}) ", p.port), lines)
    } else {
        (
            " Process Details ".to_string(),
            vec![Line::from("No process selected")],
        )
    };

    // Popup styling
    let popup_bg = Color::Rgb(30, 35, 45);
    let border_color = Color::Rgb(100, 150, 200);
    let title_style = Style::default()
        .fg(Color::Rgb(150, 200, 255))
        .add_modifier(Modifier::BOLD);

    let block = Block::default()
        .title(Span::styled(title, title_style))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(popup_bg));

    // Split popup into content and buttons
    let inner = block.inner(popup_area);
    let chunks = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Length(1),
    ]).split(inner);

    frame.render_widget(block, popup_area);

    // Render process details with background
    let label_style = Style::default()
        .fg(Color::Rgb(130, 180, 230))
        .add_modifier(Modifier::BOLD);
    let value_style = Style::default().fg(Color::Rgb(220, 220, 220));

    // Re-style the details with better colors
    let styled_details: Vec<Line> = details.into_iter().map(|line| {
        let spans: Vec<Span> = line.spans.into_iter().map(|span| {
            if span.style.add_modifier.contains(Modifier::BOLD) {
                Span::styled(span.content, label_style)
            } else {
                Span::styled(span.content, value_style)
            }
        }).collect();
        Line::from(spans)
    }).collect();

    let details_paragraph = Paragraph::new(styled_details)
        .style(Style::default().bg(popup_bg));
    frame.render_widget(details_paragraph, chunks[0]);

    // Render buttons with better styling
    let button_normal = Style::default()
        .fg(Color::Rgb(180, 180, 180))
        .bg(Color::Rgb(50, 55, 65));
    let button_selected = Style::default()
        .fg(Color::Black)
        .bg(Color::Rgb(100, 200, 255))
        .add_modifier(Modifier::BOLD);
    let button_danger = Style::default()
        .fg(Color::Rgb(255, 100, 100))
        .bg(Color::Rgb(50, 55, 65));
    let button_danger_selected = Style::default()
        .fg(Color::White)
        .bg(Color::Rgb(200, 60, 60))
        .add_modifier(Modifier::BOLD);

    let cancel_style = if app.popup_selection == PopupButton::Cancel {
        button_selected
    } else {
        button_normal
    };
    let term_style = if app.popup_selection == PopupButton::Terminate {
        button_selected
    } else {
        button_normal
    };
    let kill_style = if app.popup_selection == PopupButton::ForceKill {
        button_danger_selected
    } else {
        button_danger
    };

    let button_bg = Style::default().bg(popup_bg);
    let buttons = Line::from(vec![
        Span::styled("  ", button_bg),
        Span::styled(" Cancel (q) ", cancel_style),
        Span::styled("   ", button_bg),
        Span::styled(" Terminate (t) ", term_style),
        Span::styled("   ", button_bg),
        Span::styled(" Force Kill (k) ", kill_style),
        Span::styled("  ", button_bg),
    ]);

    frame.render_widget(Paragraph::new(buttons).style(button_bg), chunks[2]);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let horizontal = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .split(area);

    Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .split(horizontal[0])[0]
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if text.is_empty() {
        return vec!["-".to_string()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            if word.len() > max_width {
                // Word is too long, split it
                let mut remaining = word;
                while remaining.len() > max_width {
                    lines.push(remaining[..max_width].to_string());
                    remaining = &remaining[max_width..];
                }
                current_line = remaining.to_string();
            } else {
                current_line = word.to_string();
            }
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            if word.len() > max_width {
                let mut remaining = word;
                while remaining.len() > max_width {
                    lines.push(remaining[..max_width].to_string());
                    remaining = &remaining[max_width..];
                }
                current_line = remaining.to_string();
            } else {
                current_line = word.to_string();
            }
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push("-".to_string());
    }

    lines
}
