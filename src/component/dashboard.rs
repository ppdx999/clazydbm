use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::cmd::Update;
use super::Component;

/// Messages the Dashboard component can emit
pub enum DashboardMsg {
    /// Request to leave dashboard back to Connection
    Leave,
}

pub struct DashboardComponent;

impl DashboardComponent {
    pub fn new() -> Self {
        Self
    }
}

impl Component for DashboardComponent {
    type Msg = DashboardMsg;

    fn update(&mut self, _msg: Self::Msg) -> Update<Self::Msg> {
        // No internal state yet
        Update::none()
    }

    fn handle_key(&mut self, key: KeyEvent) -> Update<Self::Msg> {
        // Esc returns to Connection via Root
        if key.code == crossterm::event::KeyCode::Esc {
            return Update::msg(DashboardMsg::Leave);
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
            "Dashboard",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        );
        let block = Block::default().title(title).borders(Borders::ALL);

        let text = Text::from(vec![
            Line::from(Span::raw("Dashboard skeleton")),
            Line::from(Span::styled(
                "Press Esc to return to Connection",
                Style::default().fg(Color::Gray),
            )),
        ]);

        let paragraph = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center);

        f.render_widget(paragraph, inner);
    }
}

