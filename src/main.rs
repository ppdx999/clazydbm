use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal in raw mode with alternate screen
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run TUI app
    let result = run_app(&mut terminal);

    // Restore terminal regardless of run result
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Surface any error from the app loop
    if let Err(err) = result {
        eprintln!("{err}");
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            // Fullscreen layout with a centered area for content
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(40),
                    Constraint::Percentage(20),
                    Constraint::Percentage(40),
                ])
                .split(f.size());

            let inner = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                ])
                .split(chunks[1])[1];

            let title = Span::styled(
                "clazydbm",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );

            let block = Block::default().title(title).borders(Borders::ALL);

            let text = Text::from(vec![
                Line::from(Span::raw("Database Management TUI")),
                Line::from(Span::styled(
                    "Press q or Esc to quit",
                    Style::default().fg(Color::Gray),
                )),
            ]);

            let paragraph = Paragraph::new(text)
                .block(block)
                .alignment(Alignment::Center);

            f.render_widget(paragraph, inner);
        })?;

        // Basic input handling: quit on 'q' or Esc
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
