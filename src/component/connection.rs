use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
};

use super::Component;
use crate::cmd::Update;

/// Messages the Connection component can emit
pub enum ConnectionMsg {
    /// User selected a connection (placeholder; no payload yet)
    SelectConnection,
}

pub struct ConnectionComponent;

impl ConnectionComponent {
    pub fn new() -> Self {
        Self
    }
}

impl Component for ConnectionComponent {
    type Msg = ConnectionMsg;

    fn update(&mut self, _msg: Self::Msg) -> Update<Self::Msg> {
        // No internal state yet
        Update::none()
    }

    fn handle_key(&mut self, key: KeyEvent) -> Update<Self::Msg> {
        // Enter moves to Dashboard via Root
        if key.code == crossterm::event::KeyCode::Enter {
            return Update::msg(ConnectionMsg::SelectConnection);
        }
        Update::none()
    }

    fn draw(&mut self, f: &mut Frame, area: Rect, _focused: bool) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(20),
                Constraint::Percentage(40),
            ])
            .split(area);

        let inner = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(chunks[1])[1];

        let title = Span::styled(
            "Connection",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
        let block = Block::default().title(title).borders(Borders::ALL);

        let text = Text::from(vec![
            Line::from(Span::raw("Select a connection (skeleton)")),
            Line::from(Span::styled(
                "Press Enter to open Dashboard",
                Style::default().fg(Color::Gray),
            )),
        ]);

        let paragraph = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center);

        f.render_widget(paragraph, inner);
    }
}
