use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use super::Component;
use crate::app::AppMsg;
use crate::cmd::{Command, Update};
use crate::component::RootMsg;
use crate::config::{self, ConnectionConfig};

/// Messages the Connection component can emit
pub enum ConnectionMsg {
    /// Internal: trigger async load of connections from config
    Load,
    /// Connections loaded (Ok or error string)
    Loaded(Result<Vec<ConnectionConfig>, String>),
    /// User selected a connection
    SelectConnection,
    /// Move selection
    MoveUp,
    MoveDown,
}

pub struct ConnectionComponent {
    items: Vec<ConnectionConfig>,
    selected: usize,
}

impl ConnectionComponent {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected: 0,
        }
    }
}

impl Component for ConnectionComponent {
    type Msg = ConnectionMsg;

    fn update(&mut self, msg: Self::Msg) -> Update<Self::Msg> {
        match msg {
            ConnectionMsg::Load => {
                // Spawn loader task to read config file
                let task = |tx: std::sync::mpsc::Sender<AppMsg>| {
                    let res = config::load_connections().map_err(|e| e.to_string());
                    let _ = tx.send(AppMsg::Root(RootMsg::Connection(
                        ConnectionMsg::Loaded(res),
                    )));
                };
                Update::cmd(Command::Spawn(Box::new(task)))
            }
            ConnectionMsg::Loaded(Ok(items)) => {
                self.items = items;
                self.selected = 0;
                Update::none()
            }
            ConnectionMsg::Loaded(Err(_err)) => {
                // Keep empty list; TODO: surface error in UI
                Update::none()
            }
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
            ConnectionMsg::SelectConnection => Update::msg(ConnectionMsg::SelectConnection),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Update<Self::Msg> {
        use crossterm::event::KeyCode::*;
        match key.code {
            Enter => Update::msg(ConnectionMsg::SelectConnection),
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
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        );
        let block = Block::default().title(title).borders(Borders::ALL);

        let items: Vec<ListItem> = if self.items.is_empty() {
            vec![ListItem::new("(no connections found)")]
        } else {
            self.items
                .iter()
                .map(|c| ListItem::new(c.name.clone()))
                .collect()
        };

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .highlight_symbol("▶ ");

        let mut state = ListState::default();
        if !self.items.is_empty() {
            state.select(Some(self.selected));
        }

        f.render_stateful_widget(list, inner, &mut state);
    }
}
