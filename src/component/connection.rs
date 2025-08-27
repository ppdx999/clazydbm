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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DatabaseType;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn create_test_connection(name: &str, db_type: DatabaseType) -> Connection {
        Connection {
            r#type: db_type,
            name: Some(name.to_string()),
            user: Some("testuser".to_string()),
            host: Some("localhost".to_string()),
            port: Some(5432),
            path: None,
            password: Some("password".to_string()),
            database: Some("testdb".to_string()),
        }
    }

    fn create_component_with_connections(connections: Vec<Connection>) -> ConnectionComponent {
        ConnectionComponent {
            items: connections,
            selected: 0,
        }
    }

    #[test]
    fn test_empty_component() {
        let component = create_component_with_connections(vec![]);
        assert_eq!(component.items.len(), 0);
        assert_eq!(component.selected, 0);
        assert!(component.selected_connection().is_none());
    }

    #[test]
    fn test_component_with_connections() {
        let connections = vec![
            create_test_connection("Test DB 1", DatabaseType::Postgres),
            create_test_connection("Test DB 2", DatabaseType::MySql),
        ];
        let component = create_component_with_connections(connections);
        
        assert_eq!(component.items.len(), 2);
        assert_eq!(component.selected, 0);
        assert!(component.selected_connection().is_some());
        assert_eq!(component.selected_connection().unwrap().name.as_ref().unwrap(), "Test DB 1");
    }

    #[test]
    fn test_move_up() {
        let connections = vec![
            create_test_connection("DB 1", DatabaseType::Postgres),
            create_test_connection("DB 2", DatabaseType::MySql),
            create_test_connection("DB 3", DatabaseType::Sqlite),
        ];
        let mut component = create_component_with_connections(connections);
        component.selected = 2;

        let update = component.update(ConnectionMsg::MoveUp);
        assert_eq!(component.selected, 1);
        assert!(update.msg.is_none());
        assert!(matches!(update.cmd, crate::cmd::Command::None));
    }

    #[test]
    fn test_move_up_at_beginning() {
        let connections = vec![
            create_test_connection("DB 1", DatabaseType::Postgres),
            create_test_connection("DB 2", DatabaseType::MySql),
        ];
        let mut component = create_component_with_connections(connections);
        component.selected = 0;

        let _update = component.update(ConnectionMsg::MoveUp);
        assert_eq!(component.selected, 0); // Should stay at 0
    }

    #[test]
    fn test_move_down() {
        let connections = vec![
            create_test_connection("DB 1", DatabaseType::Postgres),
            create_test_connection("DB 2", DatabaseType::MySql),
            create_test_connection("DB 3", DatabaseType::Sqlite),
        ];
        let mut component = create_component_with_connections(connections);
        component.selected = 0;

        let update = component.update(ConnectionMsg::MoveDown);
        assert_eq!(component.selected, 1);
        assert!(update.msg.is_none());
    }

    #[test]
    fn test_move_down_at_end() {
        let connections = vec![
            create_test_connection("DB 1", DatabaseType::Postgres),
            create_test_connection("DB 2", DatabaseType::MySql),
        ];
        let mut component = create_component_with_connections(connections);
        component.selected = 1;

        let _update = component.update(ConnectionMsg::MoveDown);
        assert_eq!(component.selected, 1); // Should stay at 1
    }

    #[test]
    fn test_move_empty_list() {
        let mut component = create_component_with_connections(vec![]);

        let _update = component.update(ConnectionMsg::MoveUp);
        assert_eq!(component.selected, 0);

        let _update = component.update(ConnectionMsg::MoveDown);
        assert_eq!(component.selected, 0);
    }

    #[test]
    fn test_handle_key_enter_with_connection() {
        let connections = vec![
            create_test_connection("Test DB", DatabaseType::Postgres),
        ];
        let mut component = create_component_with_connections(connections);

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let update = component.handle_key(key);

        assert!(update.msg.is_some());
        if let Some(ConnectionMsg::ConnectionSelected(conn)) = update.msg {
            assert_eq!(conn.name.as_ref().unwrap(), "Test DB");
        } else {
            panic!("Expected ConnectionSelected message");
        }
    }

    #[test]
    fn test_handle_key_enter_empty_list() {
        let mut component = create_component_with_connections(vec![]);

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let update = component.handle_key(key);

        assert!(update.msg.is_none());
    }

    #[test]
    fn test_handle_key_movement() {
        let connections = vec![
            create_test_connection("DB 1", DatabaseType::Postgres),
            create_test_connection("DB 2", DatabaseType::MySql),
        ];
        let mut component = create_component_with_connections(connections);

        // Test Up key
        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        let update = component.handle_key(key);
        assert!(matches!(update.msg, Some(ConnectionMsg::MoveUp)));

        // Test 'k' key
        let key = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
        let update = component.handle_key(key);
        assert!(matches!(update.msg, Some(ConnectionMsg::MoveUp)));

        // Test Down key
        let key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
        let update = component.handle_key(key);
        assert!(matches!(update.msg, Some(ConnectionMsg::MoveDown)));

        // Test 'j' key
        let key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
        let update = component.handle_key(key);
        assert!(matches!(update.msg, Some(ConnectionMsg::MoveDown)));
    }

    #[test]
    fn test_handle_key_ignored() {
        let connections = vec![
            create_test_connection("DB 1", DatabaseType::Postgres),
        ];
        let mut component = create_component_with_connections(connections);

        // Test ignored key
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        let update = component.handle_key(key);
        assert!(update.msg.is_none());
    }
}
