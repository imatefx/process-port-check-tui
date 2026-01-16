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
        let error_msg = Paragraph::new(format!("Error: {}", error))
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

    // Calculate popup size
    let popup_width = 60.min(area.width.saturating_sub(4));
    let popup_height = 14.min(area.height.saturating_sub(4));

    // Center the popup
    let popup_area = centered_rect(popup_width, popup_height, area);

    // Clear the background
    frame.render_widget(Clear, popup_area);

    // Get selected port info
    let port_info = app.get_selected_port();

    let (title, details) = if let Some(p) = port_info {
        let path_str = p.exe_path
            .as_ref()
            .map(|path| path.to_string_lossy().to_string())
            .unwrap_or_else(|| "-".to_string());

        let cmd_str = if p.cmd_args.is_empty() {
            "-".to_string()
        } else {
            p.cmd_args.join(" ")
        };

        (
            format!(" Process Details (Port {}) ", p.port),
            vec![
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
                Line::from(truncate_path(&path_str, popup_width as usize - 4)),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Command:", Style::default().add_modifier(Modifier::BOLD)),
                ]),
                Line::from(truncate_path(&cmd_str, popup_width as usize - 4)),
            ]
        )
    } else {
        (
            " Process Details ".to_string(),
            vec![Line::from("No process selected")],
        )
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    // Split popup into content and buttons
    let inner = block.inner(popup_area);
    let chunks = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Length(1),
    ]).split(inner);

    frame.render_widget(block, popup_area);

    // Render process details
    let details_paragraph = Paragraph::new(details);
    frame.render_widget(details_paragraph, chunks[0]);

    // Render buttons
    let button_style = Style::default().fg(Color::White);
    let selected_style = Style::default()
        .fg(Color::Black)
        .bg(Color::White)
        .add_modifier(Modifier::BOLD);

    let cancel_style = if app.popup_selection == PopupButton::Cancel {
        selected_style
    } else {
        button_style
    };
    let term_style = if app.popup_selection == PopupButton::Terminate {
        selected_style
    } else {
        button_style
    };
    let kill_style = if app.popup_selection == PopupButton::ForceKill {
        selected_style
    } else {
        Style::default().fg(Color::Red)
    };
    let kill_selected_style = if app.popup_selection == PopupButton::ForceKill {
        selected_style
    } else {
        kill_style
    };

    let buttons = Line::from(vec![
        Span::raw("  "),
        Span::styled(" Cancel ", cancel_style),
        Span::raw("   "),
        Span::styled(" Terminate (SIGTERM) ", term_style),
        Span::raw("   "),
        Span::styled(" Force Kill (SIGKILL) ", kill_selected_style),
        Span::raw("  "),
    ]);

    frame.render_widget(Paragraph::new(buttons), chunks[2]);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let horizontal = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .split(area);

    Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .split(horizontal[0])[0]
}

fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        format!("...{}", &path[path.len().saturating_sub(max_len - 3)..])
    }
}
