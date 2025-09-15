use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui;
use ratatui::{Terminal as RatatuiTerminal, backend::CrosstermBackend};
use ratatui::prelude::Backend;
use std::io::{self, Result, Stdout, stdout, Write};

/// Custom terminal wrapper that handles suspension and restoration
pub struct Terminal<B: Backend> {
    inner: RatatuiTerminal<B>,
}

impl<B: Backend> Terminal<B> {
    pub fn new(terminal: RatatuiTerminal<B>) -> Self {
        Self { inner: terminal }
    }

    /// Execute a closure with suspended terminal
    pub fn with_suspended<F, R>(&mut self, f: F) -> std::result::Result<R, Box<dyn std::error::Error>>
    where
        F: FnOnce() -> std::result::Result<R, Box<dyn std::error::Error>>,
    {
        self.suspend()?;
        let result = f();
        self.restore()?;
        result
    }

    /// Suspend the terminal (exit alternate screen, disable raw mode)
    fn suspend(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        // Clear screen and restore cursor
        self.inner.clear()?;
        
        // Leave alternate screen
        let mut stdout = stdout();
        write!(stdout, "\x1b[?1049l")?; // Exit alternate screen buffer
        stdout.flush()?;
        
        // Disable raw mode
        disable_raw_mode()?;
        
        Ok(())
    }

    /// Restore the terminal (re-enter alternate screen, enable raw mode)
    fn restore(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        // Re-enable raw mode
        enable_raw_mode()?;
        
        // Re-enter alternate screen
        let mut stdout = stdout();
        write!(stdout, "\x1b[?1049h")?; // Enter alternate screen buffer
        stdout.flush()?;
        
        // Clear and redraw
        self.inner.clear()?;
        
        Ok(())
    }

    /// Delegate to the inner terminal's draw method
    pub fn draw<F>(&mut self, f: F) -> std::io::Result<()>
    where
        F: FnOnce(&mut ratatui::Frame),
    {
        self.inner.draw(f).map(|_| ())
    }

    /// Delegate to the inner terminal's clear method
    #[allow(dead_code)]
    pub fn clear(&mut self) -> std::io::Result<()> {
        self.inner.clear()
    }
}

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
    let ratatui_terminal = RatatuiTerminal::new(backend)?;
    let terminal = Terminal::new(ratatui_terminal);

    // Run the function with the terminal
    let result = f(terminal);

    // Cleanup terminal state
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;

    result
}
