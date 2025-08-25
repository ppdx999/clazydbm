use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, Result, Stdout};

/// Terminal wrapper that handles setup and cleanup automatically
pub fn with_terminal<F, R>(f: F) -> Result<R>
where
    F: FnOnce(Terminal<CrosstermBackend<Stdout>>) -> Result<R>,
{
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    // Run the function with the terminal
    let result = f(terminal);

    // Cleanup terminal state
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;

    result
}
