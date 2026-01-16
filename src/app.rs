use crate::ports::{get_listening_ports, PortInfo};
use ratatui::widgets::TableState;
use std::time::Instant;

/// How long status messages are shown (seconds)
const STATUS_DISPLAY_DURATION_SECS: u64 = 2;

/// Horizontal scroll step size
const SCROLL_STEP: u16 = 10;

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum PopupButton {
    #[default]
    Cancel,
    Terminate,
    ForceKill,
}

pub struct App {
    pub ports: Vec<PortInfo>,
    pub table_state: TableState,
    pub error: Option<String>,
    pub scroll_offset: u16,
    pub status_message: Option<String>,
    pub status_time: Option<Instant>,
    pub show_terminate_popup: bool,
    pub popup_selection: PopupButton,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let (ports, error) = match get_listening_ports() {
            Ok(p) => (p, None),
            Err(e) => (vec![], Some(e.to_string())),
        };

        let mut table_state = TableState::default();
        if !ports.is_empty() {
            table_state.select(Some(0));
        }

        Self {
            ports,
            table_state,
            error,
            scroll_offset: 0,
            status_message: None,
            status_time: None,
            show_terminate_popup: false,
            popup_selection: PopupButton::default(),
        }
    }

    pub fn refresh(&mut self) {
        self.set_status("Refreshing...");
        match get_listening_ports() {
            Ok(p) => {
                self.ports = p;
                self.error = None;
                self.adjust_selection();
                self.set_status(&format!("Refreshed - {} ports", self.ports.len()));
            }
            Err(e) => {
                self.error = Some(e.to_string());
                self.set_status("Refresh failed");
            }
        }
    }

    /// Adjust selection to stay within bounds after port list changes
    fn adjust_selection(&mut self) {
        if let Some(selected) = self.table_state.selected() {
            if selected >= self.ports.len() {
                self.table_state.select(if self.ports.is_empty() {
                    None
                } else {
                    Some(self.ports.len() - 1)
                });
            }
        } else if !self.ports.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    pub fn set_status(&mut self, msg: &str) {
        self.status_message = Some(msg.to_string());
        self.status_time = Some(Instant::now());
    }

    pub fn clear_old_status(&mut self) {
        if let Some(time) = self.status_time {
            if time.elapsed().as_secs() >= STATUS_DISPLAY_DURATION_SECS {
                self.status_message = None;
                self.status_time = None;
            }
        }
    }

    pub fn next(&mut self) {
        if self.ports.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => (i + 1).min(self.ports.len() - 1),
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.ports.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => i.saturating_sub(1),
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn scroll_left(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(SCROLL_STEP);
    }

    pub fn scroll_right(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(SCROLL_STEP);
    }

    pub fn open_terminate_popup(&mut self) {
        if self.table_state.selected().is_some() && !self.ports.is_empty() {
            self.show_terminate_popup = true;
            self.popup_selection = PopupButton::default();
        }
    }

    pub fn close_popup(&mut self) {
        self.show_terminate_popup = false;
    }

    pub fn popup_next(&mut self) {
        self.popup_selection = match self.popup_selection {
            PopupButton::Cancel => PopupButton::Terminate,
            PopupButton::Terminate => PopupButton::ForceKill,
            PopupButton::ForceKill => PopupButton::Cancel,
        };
    }

    pub fn popup_prev(&mut self) {
        self.popup_selection = match self.popup_selection {
            PopupButton::Cancel => PopupButton::ForceKill,
            PopupButton::Terminate => PopupButton::Cancel,
            PopupButton::ForceKill => PopupButton::Terminate,
        };
    }

    pub fn get_selected_port(&self) -> Option<&PortInfo> {
        self.table_state.selected().and_then(|i| self.ports.get(i))
    }

    pub fn execute_popup_action(&mut self) -> Option<(u32, bool)> {
        let result = self.get_selected_port().and_then(|p| {
            let pid = p.pid;
            match self.popup_selection {
                PopupButton::Cancel => None,
                PopupButton::Terminate => Some((pid, false)),
                PopupButton::ForceKill => Some((pid, true)),
            }
        });

        self.close_popup();
        result
    }
}
