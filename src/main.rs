mod app;
mod ports;
mod ui;

use std::io;
use std::panic;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use ratatui::prelude::*;

use app::App;

/// Poll timeout for event loop (milliseconds)
const EVENT_POLL_TIMEOUT_MS: u64 = 100;

fn main() -> color_eyre::Result<()> {
    // Install color-eyre for better error reporting
    color_eyre::install()?;

    // Set up panic hook to restore terminal on panic
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        // Restore terminal before printing panic
        let _ = disable_raw_mode();
        let _ = io::stdout().execute(LeaveAlternateScreen);
        original_hook(panic_info);
    }));

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

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> color_eyre::Result<()> {
    loop {
        app.clear_old_status();

        terminal.draw(|frame| ui::render(frame, app))?;

        if event::poll(std::time::Duration::from_millis(EVENT_POLL_TIMEOUT_MS))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.show_terminate_popup {
                        handle_popup_key(key.code, app);
                    } else if handle_main_key(key.code, app) {
                        return Ok(());
                    }
                }
            }
        }
    }
}

/// Handle keyboard input in main view. Returns true if app should quit.
fn handle_main_key(code: KeyCode, app: &mut App) -> bool {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => true,
        KeyCode::Char('r') => {
            app.refresh();
            false
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.next();
            false
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.previous();
            false
        }
        KeyCode::Left | KeyCode::Char('h') => {
            app.scroll_left();
            false
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.scroll_right();
            false
        }
        KeyCode::Char('t') | KeyCode::Enter => {
            app.open_terminate_popup();
            false
        }
        KeyCode::Home => {
            app.scroll_offset = 0;
            false
        }
        _ => false,
    }
}

/// Handle keyboard input in popup
fn handle_popup_key(code: KeyCode, app: &mut App) {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.close_popup(),
        KeyCode::Left | KeyCode::Char('h') => app.popup_prev(),
        KeyCode::Right | KeyCode::Char('l') => app.popup_next(),
        KeyCode::Tab => app.popup_next(),
        KeyCode::BackTab => app.popup_prev(),
        KeyCode::Enter => {
            if let Some((pid, force)) = app.execute_popup_action() {
                kill_process(pid, force, app);
            }
        }
        KeyCode::Char('t') => {
            if let Some(pid) = app.get_selected_port().map(|p| p.pid) {
                app.close_popup();
                kill_process(pid, false, app);
            }
        }
        KeyCode::Char('k') => {
            if let Some(pid) = app.get_selected_port().map(|p| p.pid) {
                app.close_popup();
                kill_process(pid, true, app);
            }
        }
        _ => {}
    }
}

/// Kill a process using proper signal handling via nix
fn kill_process(pid: u32, force: bool, app: &mut App) {
    let signal = if force { Signal::SIGKILL } else { Signal::SIGTERM };
    let nix_pid = Pid::from_raw(pid as i32);

    match kill(nix_pid, signal) {
        Ok(()) => {
            let msg = if force {
                format!("Force killed PID {pid}")
            } else {
                format!("Terminated PID {pid}")
            };
            app.set_status(&msg);
            app.refresh();
        }
        Err(e) => {
            app.set_status(&format!("Failed: {e}"));
        }
    }
}
