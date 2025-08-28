use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState},
};

use super::Component;
use crate::app::AppMsg;
use crate::cmd::{Command, Update};
use crate::component::{DashboardMsg, RootMsg};
use crate::db::DBBehavior;
use crate::logger::{error, info};
use crate::{connection::Connection, db};

#[derive(Clone, PartialEq, Debug)]
pub struct Database {
    pub name: String,
    pub children: Vec<Child>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Child {
    Table(Table),
    Schema(Schema),
}

impl From<Table> for Child {
    fn from(t: Table) -> Self {
        Child::Table(t)
    }
}

impl From<Schema> for Child {
    fn from(s: Schema) -> Self {
        Child::Schema(s)
    }
}

impl Database {
    pub fn new(database: String, children: Vec<Child>) -> Self {
        Self {
            name: database,
            children,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Schema {
    pub name: String,
    pub tables: Vec<Table>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    pub name: String,
    pub engine: Option<String>,
    pub schema: Option<String>,
}

pub enum DBListMsg {
    LeaveDashboard,
    MoveUp,
    MoveDown,
    MoveTop,
    MoveBottom,
    Expand,
    Fold,
    SelectTable { database: String, table: String },
    Filter,
    Load(Connection),
    Loaded(Vec<Database>),
    LoadFailed(String),
}

pub enum Focus {
    Tree,
    Filter,
}

#[derive(Debug, Clone)]
pub struct FlatNode {
    pub name: String,
    pub level: usize,
    pub node_type: FlatNodeType,
    pub is_expanded: bool,
    pub has_children: bool,
}

#[derive(Debug, Clone)]
pub enum FlatNodeType {
    Database(String),
    Schema {
        database: String,
        schema: String,
    },
    Table {
        database: String,
        table: String,
        schema: Option<String>,
    },
}

pub struct DBListComponent {
    databases: Vec<Database>,
    flat_nodes: Vec<FlatNode>,
    selected: usize,
    focus: Focus,
    filter_query: String,
    expanded_databases: std::collections::HashSet<String>,
    expanded_schemas: std::collections::HashSet<(String, String)>, // (database, schema)
}

impl DBListComponent {
    pub fn new() -> Self {
        let mut component = Self {
            databases: vec![],
            flat_nodes: vec![],
            selected: 0,
            focus: Focus::Tree,
            filter_query: String::new(),
            expanded_databases: std::collections::HashSet::new(),
            expanded_schemas: std::collections::HashSet::new(),
        };
        component.rebuild_flat_list();
        component
    }

    fn rebuild_flat_list(&mut self) {
        let mut flat_nodes = Vec::new();
        for database in &self.databases {
            self.flatten_database(database, &mut flat_nodes);
        }
        self.flat_nodes = flat_nodes;
    }

    fn flatten_database(&self, database: &Database, flat_nodes: &mut Vec<FlatNode>) {
        let is_expanded = self.expanded_databases.contains(&database.name);
        let has_children = !database.children.is_empty();

        flat_nodes.push(FlatNode {
            name: database.name.clone(),
            level: 0,
            node_type: FlatNodeType::Database(database.name.clone()),
            is_expanded,
            has_children,
        });

        if is_expanded {
            for child in &database.children {
                match child {
                    Child::Table(table) => {
                        flat_nodes.push(FlatNode {
                            name: table.name.clone(),
                            level: 1,
                            node_type: FlatNodeType::Table {
                                database: database.name.clone(),
                                table: table.name.clone(),
                                schema: table.schema.clone(),
                            },
                            is_expanded: false,
                            has_children: false,
                        });
                    }
                    Child::Schema(schema) => {
                        let schema_key = (database.name.clone(), schema.name.clone());
                        let is_schema_expanded = self.expanded_schemas.contains(&schema_key);
                        let has_schema_children = !schema.tables.is_empty();

                        flat_nodes.push(FlatNode {
                            name: schema.name.clone(),
                            level: 1,
                            node_type: FlatNodeType::Schema {
                                database: database.name.clone(),
                                schema: schema.name.clone(),
                            },
                            is_expanded: is_schema_expanded,
                            has_children: has_schema_children,
                        });

                        if is_schema_expanded {
                            for table in &schema.tables {
                                flat_nodes.push(FlatNode {
                                    name: table.name.clone(),
                                    level: 2,
                                    node_type: FlatNodeType::Table {
                                        database: database.name.clone(),
                                        table: table.name.clone(),
                                        schema: Some(schema.name.clone()),
                                    },
                                    is_expanded: false,
                                    has_children: false,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    fn toggle_expand(&mut self) {
        if let Some(node) = self.flat_nodes.get(self.selected) {
            if node.has_children {
                match &node.node_type {
                    FlatNodeType::Database(db_name) => {
                        if self.expanded_databases.contains(db_name) {
                            self.expanded_databases.remove(db_name);
                        } else {
                            self.expanded_databases.insert(db_name.clone());
                        }
                    }
                    FlatNodeType::Schema { database, schema } => {
                        let schema_key = (database.clone(), schema.clone());
                        if self.expanded_schemas.contains(&schema_key) {
                            self.expanded_schemas.remove(&schema_key);
                        } else {
                            self.expanded_schemas.insert(schema_key);
                        }
                    }
                    FlatNodeType::Table { .. } => {
                        // Tables cannot be expanded
                    }
                }
                self.rebuild_flat_list();
            }
        }
    }

    fn selected_node(&self) -> Option<&FlatNode> {
        self.flat_nodes.get(self.selected)
    }
}

impl Component for DBListComponent {
    type Msg = DBListMsg;

    fn update(&mut self, msg: Self::Msg) -> Update<Self::Msg> {
        match msg {
            DBListMsg::Load(conn) => {
                // Fetch DB structure in background based on the selected connection
                let task = move |tx: std::sync::mpsc::Sender<AppMsg>| {
                    info(&format!("DBList: loading databases for {:?}", conn.r#type));
                    let result = db::DB::fetch_databases(&conn);
                    let msg = match result {
                        Ok(dbs) => {
                            info(&format!("DBList: loaded {} database(s)", dbs.len()));
                            AppMsg::Root(RootMsg::Dashboard(DashboardMsg::DBListMsg(
                                DBListMsg::Loaded(dbs),
                            )))
                        }
                        Err(e) => {
                            error(&format!("DBList: load failed: {}", e));
                            AppMsg::Root(RootMsg::Dashboard(DashboardMsg::DBListMsg(
                                DBListMsg::LoadFailed(e.to_string()),
                            )))
                        }
                    };
                    let _ = tx.send(msg);
                };
                Update::cmd(Command::Spawn(Box::new(task)))
            }
            DBListMsg::Loaded(dbs) => {
                self.databases = dbs;
                self.expanded_databases.clear();
                self.expanded_schemas.clear();
                self.selected = 0;
                self.rebuild_flat_list();
                Update::none()
            }
            DBListMsg::LoadFailed(_err) => {
                // Keep current state; optionally we could surface error in UI later
                Update::none()
            }
            DBListMsg::MoveUp => {
                if !self.flat_nodes.is_empty() {
                    self.selected = self.selected.saturating_sub(1);
                }
                Update::none()
            }
            DBListMsg::MoveDown => {
                if !self.flat_nodes.is_empty() {
                    self.selected = (self.selected + 1).min(self.flat_nodes.len() - 1);
                }
                Update::none()
            }
            DBListMsg::MoveTop => {
                if !self.flat_nodes.is_empty() {
                    self.selected = 0;
                }
                Update::none()
            }
            DBListMsg::MoveBottom => {
                if !self.flat_nodes.is_empty() {
                    self.selected = self.flat_nodes.len() - 1;
                }
                Update::none()
            }
            DBListMsg::Expand => {
                self.toggle_expand();
                Update::none()
            }
            DBListMsg::Fold => {
                self.toggle_expand();
                Update::none()
            }
            DBListMsg::Filter => {
                self.focus = Focus::Filter;
                Update::none()
            }
            _ => Update::none(),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Update<Self::Msg> {
        use crossterm::event::KeyCode::*;

        match self.focus {
            Focus::Tree => match key.code {
                Up | Char('k') => Update::msg(DBListMsg::MoveUp),
                Down | Char('j') => Update::msg(DBListMsg::MoveDown),
                Char('g') => Update::msg(DBListMsg::MoveTop),
                Char('G') => Update::msg(DBListMsg::MoveBottom),
                Right | Char('l') => Update::msg(DBListMsg::Expand),
                Left | Char('h') => Update::msg(DBListMsg::Fold),
                Char('/') => Update::msg(DBListMsg::Filter),
                Esc => Update::msg(DBListMsg::LeaveDashboard),
                Enter => {
                    if let Some(node) = self.selected_node() {
                        match &node.node_type {
                            FlatNodeType::Table {
                                database, table, ..
                            } => {
                                return Update::msg(DBListMsg::SelectTable {
                                    database: database.clone(),
                                    table: table.clone(),
                                });
                            }
                            FlatNodeType::Database(_) | FlatNodeType::Schema { .. } => {
                                // Expand/collapse on Enter for databases and schemas
                                self.toggle_expand();
                            }
                        }
                    }
                    Update::none()
                }
                _ => Update::none(),
            },
            Focus::Filter => match key.code {
                Enter | Esc => {
                    self.focus = Focus::Tree;
                    Update::none()
                }
                Char(c) => {
                    self.filter_query.push(c);
                    Update::none()
                }
                Backspace => {
                    self.filter_query.pop();
                    Update::none()
                }
                _ => Update::none(),
            },
        }
    }

    fn draw(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area);

        // Draw tree
        let tree_area = chunks[0];
        let filter_area = chunks[1];

        // Tree view
        let items: Vec<ListItem> = if self.flat_nodes.is_empty() {
            vec![ListItem::new("(no database structure)")]
        } else {
            self.flat_nodes
                .iter()
                .map(|node| {
                    let indent = "  ".repeat(node.level);
                    let prefix = match &node.node_type {
                        FlatNodeType::Database(_) => {
                            if node.is_expanded {
                                "▼ 📁"
                            } else if node.has_children {
                                "▶ 📁"
                            } else {
                                "  📁"
                            }
                        }
                        FlatNodeType::Schema { .. } => {
                            if node.is_expanded {
                                "▼ 📂"
                            } else if node.has_children {
                                "▶ 📂"
                            } else {
                                "  📂"
                            }
                        }
                        FlatNodeType::Table { .. } => "  📄",
                    };
                    ListItem::new(Span::raw(format!("{}{} {}", indent, prefix, node.name)))
                })
                .collect()
        };

        let tree_style = if focused && matches!(self.focus, Focus::Tree) {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .title("Database Structure")
                    .borders(Borders::ALL)
                    .border_style(tree_style),
            )
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        let mut state = ListState::default();
        if !self.flat_nodes.is_empty() {
            state.select(Some(self.selected));
        }

        f.render_stateful_widget(list, tree_area, &mut state);

        // Filter input
        let filter_style = if focused && matches!(self.focus, Focus::Filter) {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let filter_block = Block::default()
            .title("Filter")
            .borders(Borders::ALL)
            .border_style(filter_style);

        let filter_text = if matches!(self.focus, Focus::Filter) {
            format!("{}_", self.filter_query)
        } else {
            self.filter_query.clone()
        };

        let filter_paragraph = ratatui::widgets::Paragraph::new(filter_text).block(filter_block);
        f.render_widget(filter_paragraph, filter_area);
    }
}
