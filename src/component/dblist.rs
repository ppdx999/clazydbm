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
use crate::db::DBBehavior;
use crate::logger::{error, info};
use crate::update::{Command, Update};
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
    ToggleExpand,
    Filter,
    Load(Connection),
    Loaded(Vec<Database>),
    LoadFailed(String),
    FilterPush(char),
    FilterPop,
    FilterConfirm,
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
        #[allow(dead_code)]
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

    fn move_up(&mut self) {
        if !self.flat_nodes.is_empty() {
            self.selected = self.selected.saturating_sub(1);
        }
    }
    fn move_down(&mut self) {
        if !self.flat_nodes.is_empty() {
            self.selected = (self.selected + 1).min(self.flat_nodes.len() - 1);
        }
    }
    fn move_top(&mut self) {
        if !self.flat_nodes.is_empty() {
            self.selected = 0;
        }
    }
    fn move_bottom(&mut self) {
        if !self.flat_nodes.is_empty() {
            self.selected = self.flat_nodes.len() - 1;
        }
    }

    fn rebuild_flat_list(&mut self) {
        let mut flat_nodes = Vec::new();
        for database in &self.databases {
            self.flatten_database(database, &mut flat_nodes);
        }

        // Apply filter if query is not empty
        if !self.filter_query.is_empty() {
            let filter_lower = self.filter_query.to_lowercase();
            flat_nodes.retain(|node| node.name.to_lowercase().contains(&filter_lower));
        }

        self.flat_nodes = flat_nodes;

        // Ensure selected index is valid after filtering
        if self.selected >= self.flat_nodes.len() && !self.flat_nodes.is_empty() {
            self.selected = self.flat_nodes.len() - 1;
        }
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

    fn on_load(conn: Connection) -> impl FnOnce(std::sync::mpsc::Sender<AppMsg>) + Send + 'static {
        move |tx: std::sync::mpsc::Sender<AppMsg>| {
            info(&format!("DBList: loading databases for {:?}", conn.r#type));
            let result = db::DB::fetch_databases(&conn);
            let msg = match result {
                Ok(dbs) => {
                    info(&format!("DBList: loaded {} database(s)", dbs.len()));
                    DBListMsg::Loaded(dbs).into()
                }
                Err(e) => {
                    error(&format!("DBList: load failed: {}", e));
                    DBListMsg::LoadFailed(e.to_string()).into()
                }
            };
            let _ = tx.send(msg);
        }
    }
    fn on_loaded(&mut self, dbs: Vec<Database>) -> Update<DBListMsg> {
        self.databases = dbs;
        self.expanded_databases.clear();
        self.expanded_schemas.clear();
        self.selected = 0;
        self.filter_query.clear();
        self.focus = Focus::Tree;
        self.rebuild_flat_list();
        Update::none()
    }

    fn move_focus_to_filter(&mut self) {
        self.focus = Focus::Filter;
    }

    fn move_focus_to_tree(&mut self) {
        self.focus = Focus::Tree;
    }

    fn push_filter_char(&mut self, c: char) {
        self.filter_query.push(c);
        self.rebuild_flat_list();
    }
    fn pop_filter_char(&mut self) {
        self.filter_query.pop();
        self.rebuild_flat_list();
    }
}

impl Component for DBListComponent {
    type Msg = DBListMsg;

    fn update(&mut self, msg: Self::Msg) -> Update<Self::Msg> {
        match msg {
            DBListMsg::Load(conn) => Command::Spawn(Box::new(Self::on_load(conn))).into(),
            DBListMsg::Loaded(dbs) => self.on_loaded(dbs),
            DBListMsg::LoadFailed(_err) => Update::none(),
            DBListMsg::MoveUp => self.move_up().into(),
            DBListMsg::MoveDown => self.move_down().into(),
            DBListMsg::MoveTop => self.move_top().into(),
            DBListMsg::MoveBottom => self.move_bottom().into(),
            DBListMsg::Expand => self.toggle_expand().into(),
            DBListMsg::Fold => self.toggle_expand().into(),
            DBListMsg::Filter => self.move_focus_to_filter().into(),
            DBListMsg::LeaveDashboard => Update::none(), // Handled by parent
            DBListMsg::ToggleExpand => self.toggle_expand().into(),
            DBListMsg::SelectTable {
                database: _,
                table: _,
            } => Update::none(), // Handled by parent
            DBListMsg::FilterPush(c) => self.push_filter_char(c).into(),
            DBListMsg::FilterPop => self.pop_filter_char().into(),
            DBListMsg::FilterConfirm => {
                self.move_focus_to_tree();
                self.rebuild_flat_list();
                Update::none()
            }
        }
    }

    fn handle_key(&self, key: KeyEvent) -> Update<Self::Msg> {
        use crossterm::event::KeyCode::*;

        match self.focus {
            Focus::Tree => match key.code {
                Up | Char('k') => DBListMsg::MoveUp.into(),
                Down | Char('j') => DBListMsg::MoveDown.into(),
                Char('g') => DBListMsg::MoveTop.into(),
                Char('G') => DBListMsg::MoveBottom.into(),
                Right | Char('l') => DBListMsg::Expand.into(),
                Left | Char('h') => DBListMsg::Fold.into(),
                Char('/') => DBListMsg::Filter.into(),
                Esc => DBListMsg::LeaveDashboard.into(),
                Enter | Tab => {
                    let Some(node) = self.selected_node() else {
                        return Update::none();
                    };
                    match &node.node_type {
                        FlatNodeType::Table {
                            database, table, ..
                        } => DBListMsg::SelectTable {
                            database: database.clone(),
                            table: table.clone(),
                        },
                        // Expand/collapse on Enter for databases and schemas
                        FlatNodeType::Database(_) => DBListMsg::ToggleExpand,
                        FlatNodeType::Schema { .. } => DBListMsg::ToggleExpand,
                    }
                    .into()
                }
                _ => Update::none(),
            },
            Focus::Filter => match key.code {
                Esc | Enter => DBListMsg::FilterConfirm.into(),
                Char(c) => DBListMsg::FilterPush(c).into(),
                Backspace => DBListMsg::FilterPop.into(),
                _ => Update::none(),
            },
        }
    }

    fn draw(&self, f: &mut Frame, area: Rect, focused: bool) {
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
