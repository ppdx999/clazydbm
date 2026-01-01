use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState},
};

use super::Component;
use crate::{update::Update, db::DBBehavior};
use crate::{connection::Connection, connection::load_connections, db::DB};

pub enum ConnectionMsg {
    ConnectionSelected(Connection),
    MoveUp,
    MoveDown,
    MoveTop,
    MoveBottom,
    MovePageUp,
    MovePageDown,
}

pub struct ConnectionComponent {
    items: Vec<Connection>,
    selected: usize,
}

impl ConnectionComponent {
    pub fn new() -> Result<Self> {
        Ok(Self {
            items: load_connections()?,
            selected: 0,
        })
    }
    fn selected_connection(&self) -> Option<&Connection> {
        self.items.get(self.selected)
    }
    fn move_up(&mut self) {
        if !self.items.is_empty() {
            self.selected = self.selected.saturating_sub(1);
        }
    }
    fn move_down(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + 1).min(self.items.len() - 1);
        }
    }
    fn move_top(&mut self) {
        if !self.items.is_empty() {
            self.selected = 0;
        }
    }
    fn move_bottom(&mut self) {
        if !self.items.is_empty() {
            self.selected = self.items.len() - 1;
        }
    }
    fn move_page_up(&mut self) {
        if !self.items.is_empty() {
            self.selected = self.selected.saturating_sub(10);
        }
    }
    fn move_page_down(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + 10).min(self.items.len() - 1);
        }
    }
}

impl Component for ConnectionComponent {
    type Msg = ConnectionMsg;

    fn update(&mut self, msg: Self::Msg) -> Update<Self::Msg> {
        match msg {
            ConnectionMsg::MoveUp => self.move_up().into(),
            ConnectionMsg::MoveDown => self.move_down().into(),
            ConnectionMsg::MoveTop => self.move_top().into(),
            ConnectionMsg::MoveBottom => self.move_bottom().into(),
            ConnectionMsg::MovePageUp => self.move_page_up().into(),
            ConnectionMsg::MovePageDown => self.move_page_down().into(),
            ConnectionMsg::ConnectionSelected(_) => Update::none(), // Handled by parent
        }
    }

    fn handle_key(&self, key: KeyEvent) -> Update<Self::Msg> {
        use crossterm::event::KeyCode::*;
        match key.code {
            Enter => match self.selected_connection() {
                Some(conn) => ConnectionMsg::ConnectionSelected(conn.clone()).into(),
                None => Update::none(),
            },
            Up | Char('k') => ConnectionMsg::MoveUp.into(),
            Down | Char('j') => ConnectionMsg::MoveDown.into(),
            PageUp => ConnectionMsg::MovePageUp.into(),
            PageDown => ConnectionMsg::MovePageDown.into(),
            Home => ConnectionMsg::MoveTop.into(),
            End => ConnectionMsg::MoveBottom.into(),
            _ => Update::none(),
        }
    }

    fn draw(&self, f: &mut Frame, area: Rect, _focused: bool) {
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

        // Determine visible window based on available height
        let border_rows = 2u16; // top+bottom borders
        let avail = inner.height.saturating_sub(border_rows);
        let visible = usize::try_from(avail).unwrap_or(0).max(1);
        let total = self.items.len();
        let start = if total <= visible {
            0
        } else {
            self.selected.saturating_add(1).saturating_sub(visible)
        };
        let end = (start + visible).min(total);

        let items: Vec<ListItem> = if total == 0 {
            vec![ListItem::new("(no connections found)")]
        } else {
            self.items[start..end]
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
        if total > 0 {
            // Adjust the selection index within the window
            state.select(Some(self.selected - start));
        }
        f.render_stateful_widget(list, inner, &mut state);
    }
}
