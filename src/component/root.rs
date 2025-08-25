use crate::{cmd::Update, component::Component};
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

pub enum RootMsg {
    // Define messages that RootComponent can handle
}

pub struct RootComponent {
    // fields for RootComponent
}

impl RootComponent {
    pub fn new() -> Self {
        Self {
            // Initialize fields
        }
    }
}

impl Component for RootComponent {
    type Msg = RootMsg;

    fn update(&mut self, msg: Self::Msg) -> Update<Self::Msg> {
        // handle update logic
        Update::none()
    }

    fn handle_key(&mut self, key: KeyEvent) -> Update<Self::Msg> {
        // handle key events
        Update::none()
    }

    fn draw(&mut self, f: &mut Frame, _area: Rect, _focused: bool) {
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
                "Press Ctrl-C to quit",
                Style::default().fg(Color::Gray),
            )),
        ]);

        let paragraph = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center);

        f.render_widget(paragraph, inner);
    }
}
