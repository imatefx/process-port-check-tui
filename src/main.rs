mod app;
mod ports;
mod ui;

use std::io;
use std::process::Command;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;

use app::App;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // Setup terminal
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    // Run app
    let mut app = App::new();
    let result = run(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> color_eyre::Result<()> {
    loop {
        // Clear old status messages
        app.clear_old_status();

        terminal.draw(|frame| ui::render(frame, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.show_terminate_popup {
                        // Handle popup keys
                        match key.code {
                            KeyCode::Esc => app.close_popup(),
                            KeyCode::Left | KeyCode::Char('h') => app.popup_prev(),
                            KeyCode::Right | KeyCode::Char('l') => app.popup_next(),
                            KeyCode::Tab => app.popup_next(),
                            KeyCode::BackTab => app.popup_prev(),
                            KeyCode::Enter => {
                                if let Some((pid, force)) = app.execute_popup_action() {
                                    kill_process(pid, force, app);
                                }
                            }
                            _ => {}
                        }
                    } else {
                        // Normal mode keys
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                            KeyCode::Char('r') => app.refresh(),
                            KeyCode::Down | KeyCode::Char('j') => app.next(),
                            KeyCode::Up | KeyCode::Char('k') => app.previous(),
                            KeyCode::Left | KeyCode::Char('h') => app.scroll_left(),
                            KeyCode::Right | KeyCode::Char('l') => app.scroll_right(),
                            KeyCode::Char('t') | KeyCode::Enter => app.open_terminate_popup(),
                            KeyCode::Home => app.scroll_offset = 0,
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

fn kill_process(pid: u32, force: bool, app: &mut App) {
    let signal = if force { "KILL" } else { "TERM" };

    let result = Command::new("kill")
        .arg(format!("-{}", signal))
        .arg(pid.to_string())
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                let msg = if force {
                    format!("Force killed PID {}", pid)
                } else {
                    format!("Terminated PID {}", pid)
                };
                app.set_status(&msg);
                // Auto-refresh after kill
                app.refresh();
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                app.set_status(&format!("Failed: {}", stderr.trim()));
            }
        }
        Err(e) => {
            app.set_status(&format!("Error: {}", e));
        }
    }
}
