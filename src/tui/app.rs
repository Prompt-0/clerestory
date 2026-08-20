//! TUI application state machine and event loop.

use crate::error::Result;
use crate::model::host::HostReport;
use crate::probe::HostInspector;
use crate::tui::ui::UiRenderer;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::stdout;
use std::time::Duration;

pub struct TuiState {
    pub current_tab: usize,
    pub selected_cores: u32,
    pub should_quit: bool,
}

impl Default for TuiState {
    fn default() -> Self {
        Self {
            current_tab: 0,
            selected_cores: 8,
            should_quit: false,
        }
    }
}

pub struct TuiApp;

impl TuiApp {
    /// Launches the interactive setup wizard.
    pub fn run_wizard() -> Result<()> {
        let inspector = HostInspector::live();
        let report = inspector.inspect("/var/lib/libvirt/images")?;
        Self::run_with_report(&report)
    }

    /// Launches the TUI doctor inspection view.
    pub fn run_doctor(report: &HostReport) -> Result<()> {
        Self::run_with_report(report)
    }

    fn run_with_report(report: &HostReport) -> Result<()> {
        enable_raw_mode().map_err(|e| crate::error::ClerestoryError::IoError {
            path: std::path::PathBuf::from("terminal"),
            source: e,
        })?;

        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen).map_err(|e| {
            crate::error::ClerestoryError::IoError {
                path: std::path::PathBuf::from("terminal"),
                source: e,
            }
        })?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal =
            Terminal::new(backend).map_err(|e| crate::error::ClerestoryError::IoError {
                path: std::path::PathBuf::from("terminal"),
                source: e,
            })?;

        let mut state = TuiState::default();
        let res = Self::event_loop(&mut terminal, &mut state, report);

        // Terminal cleanup
        let _ = disable_raw_mode();
        let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
        let _ = terminal.show_cursor();

        res
    }

    fn event_loop<B: ratatui::backend::Backend>(
        terminal: &mut Terminal<B>,
        state: &mut TuiState,
        report: &HostReport,
    ) -> Result<()> {
        while !state.should_quit {
            terminal
                .draw(|f| {
                    UiRenderer::render(f, state, report);
                })
                .map_err(|e| crate::error::ClerestoryError::IoError {
                    path: std::path::PathBuf::from("terminal"),
                    source: e,
                })?;

            if event::poll(Duration::from_millis(100)).map_err(|e| {
                crate::error::ClerestoryError::IoError {
                    path: std::path::PathBuf::from("terminal"),
                    source: e,
                }
            })? {
                if let Event::Key(key) =
                    event::read().map_err(|e| crate::error::ClerestoryError::IoError {
                        path: std::path::PathBuf::from("terminal"),
                        source: e,
                    })?
                {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            state.should_quit = true;
                        }
                        KeyCode::Tab => {
                            state.current_tab = (state.current_tab + 1) % 5;
                        }
                        KeyCode::BackTab => {
                            state.current_tab = if state.current_tab == 0 {
                                4
                            } else {
                                state.current_tab - 1
                            };
                        }
                        KeyCode::Up => {
                            state.selected_cores = (state.selected_cores + 2).min(32);
                        }
                        KeyCode::Down => {
                            state.selected_cores = (state.selected_cores.saturating_sub(2)).max(2);
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            state.should_quit = true;
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
}
