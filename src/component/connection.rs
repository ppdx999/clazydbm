use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState},
};

use super::Component;
use crate::{cmd::Update, db::DBBehavior};
use crate::{connection::Connection, connection::load_connections, db::DB};

pub enum ConnectionMsg {
    ConnectionSelected(Connection),
    MoveUp,
    MoveDown,
}

pub struct ConnectionComponent {
    items: Vec<Connection>,
    selected: usize,
}

impl ConnectionComponent {
    pub fn new() -> Self {
        Self {
            items: load_connections().unwrap(),
            selected: 0,
        }
    }
    fn selected_connection(&self) -> Option<&Connection> {
        self.items.get(self.selected)
    }
}

impl Component for ConnectionComponent {
    type Msg = ConnectionMsg;

    fn update(&mut self, msg: Self::Msg) -> Update<Self::Msg> {
        match msg {
            ConnectionMsg::MoveUp => {
                if !self.items.is_empty() {
                    self.selected = self.selected.saturating_sub(1);
                }
                Update::none()
            }
            ConnectionMsg::MoveDown => {
                if !self.items.is_empty() {
                    self.selected = (self.selected + 1).min(self.items.len() - 1);
                }
                Update::none()
            }
            _ => Update::none(),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Update<Self::Msg> {
        use crossterm::event::KeyCode::*;
        match key.code {
            Enter => match self.selected_connection() {
                Some(conn) => Update::msg(ConnectionMsg::ConnectionSelected(conn.clone())),
                // TODO: Set error state to show in UI
                None => Update::none(),
            },
            Up | Char('k') => Update::msg(ConnectionMsg::MoveUp),
            Down | Char('j') => Update::msg(ConnectionMsg::MoveDown),
            _ => Update::none(),
        }
    }

    fn draw(&mut self, f: &mut Frame, area: Rect, _focused: bool) {
        let outer = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(35),
                Constraint::Percentage(30),
                Constraint::Percentage(35),
            ])
            .split(area);

        let inner = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(outer[1])[1];

        let title = Span::styled(
            "Connections",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
        let block = Block::default().title(title).borders(Borders::ALL);

        let items: Vec<ListItem> = if self.items.is_empty() {
            vec![ListItem::new("(no connections found)")]
        } else {
            self.items
                .iter()
                .map(|c| {
                    ListItem::new(Span::raw(format!(
                        "{} ({})",
                        c.name.clone().unwrap_or("unknown".to_string()),
                        DB::database_url(c).unwrap_or("invalid config".to_string())
                    )))
                })
                .collect()
        };

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        let mut state = ListState::default();
        if !self.items.is_empty() {
            state.select(Some(self.selected));
        }

        f.render_stateful_widget(list, inner, &mut state);
    }
}
