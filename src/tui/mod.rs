//! Terminal User Interface for wt status.

mod app;
mod ui;

pub use app::App;

use std::io;
use std::time::Duration;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use crate::error::Result;

/// Run the TUI application
pub fn run() -> Result<()> {
    // Setup terminal
    enable_raw_mode().map_err(|e| crate::error::WtError::Io {
        operation: "enable raw mode".to_string(),
        path: "terminal".to_string(),
        message: e.to_string(),
    })?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(|e| {
        crate::error::WtError::Io {
            operation: "enter alternate screen".to_string(),
            path: "terminal".to_string(),
            message: e.to_string(),
        }
    })?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| crate::error::WtError::Io {
        operation: "create terminal".to_string(),
        path: "terminal".to_string(),
        message: e.to_string(),
    })?;

    // Create app and run
    let mut app = App::new()?;
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode().ok();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .ok();
    terminal.show_cursor().ok();

    result
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let tick_rate = Duration::from_secs(2);

    loop {
        terminal
            .draw(|f| ui::draw(f, app))
            .map_err(|e| crate::error::WtError::Io {
                operation: "draw terminal".to_string(),
                path: "terminal".to_string(),
                message: e.to_string(),
            })?;

        // Poll for events with timeout
        if event::poll(tick_rate).map_err(|e| crate::error::WtError::Io {
            operation: "poll events".to_string(),
            path: "terminal".to_string(),
            message: e.to_string(),
        })? {
            if let Event::Key(key) = event::read().map_err(|e| crate::error::WtError::Io {
                operation: "read event".to_string(),
                path: "terminal".to_string(),
                message: e.to_string(),
            })? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Enter | KeyCode::Char('e') => {
                            if let Err(e) = app.enter_selected() {
                                // Exit TUI to show error
                                return Err(e);
                            }
                            // After entering tmux, we exit the TUI
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
        } else {
            // Tick: refresh data
            app.refresh()?;
        }
    }
}
