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
    pub is_expanded: bool,
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
            is_expanded: false,
        }
    }

    pub fn expand(&mut self) {
        self.is_expanded = true;
    }

    pub fn fold(&mut self) {
        self.is_expanded = false;
    }

    pub fn toggle_expand(&mut self) {
        self.is_expanded = !self.is_expanded;
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Schema {
    pub name: String,
    pub tables: Vec<Table>,
    pub is_expanded: bool,
}

impl Schema {
    pub fn expand(&mut self) {
        self.is_expanded = true;
    }

    pub fn fold(&mut self) {
        self.is_expanded = false;
    }

    pub fn toggle_expand(&mut self) {
        self.is_expanded = !self.is_expanded;
    }

    pub fn has_children(&self) -> bool {
        !self.tables.is_empty()
    }
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
    selected: usize,
    focus: Focus,
    filter_query: String,
}

impl DBListComponent {
    pub fn new() -> Self {
        Self {
            databases: vec![],
            selected: 0,
            focus: Focus::Tree,
            filter_query: String::new(),
        }
    }

    fn move_up(&mut self) {
        let len = self.flat_nodes().len();
        if len > 0 {
            self.selected = self.selected.saturating_sub(1);
        }
    }

    fn move_down(&mut self) {
        let len = self.flat_nodes().len();
        if len > 0 {
            self.selected = (self.selected + 1).min(len - 1);
        }
    }

    fn move_top(&mut self) {
        if !self.flat_nodes().is_empty() {
            self.selected = 0;
        }
    }

    fn move_bottom(&mut self) {
        let len = self.flat_nodes().len();
        if len > 0 {
            self.selected = len - 1;
        }
    }

    fn flat_nodes(&self) -> Vec<FlatNode> {
        let mut nodes = Vec::new();
        for database in &self.databases {
            Self::flatten_database(database, &mut nodes);
        }

        // Apply filter if query is not empty
        if !self.filter_query.is_empty() {
            let filter_lower = self.filter_query.to_lowercase();
            nodes.retain(|node| node.name.to_lowercase().contains(&filter_lower));
        }

        nodes
    }

    fn flatten_database(database: &Database, flat_nodes: &mut Vec<FlatNode>) {
        let has_children = !database.children.is_empty();

        flat_nodes.push(FlatNode {
            name: database.name.clone(),
            level: 0,
            node_type: FlatNodeType::Database(database.name.clone()),
            is_expanded: database.is_expanded,
            has_children,
        });

        if database.is_expanded {
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
                        let has_schema_children = !schema.tables.is_empty();

                        flat_nodes.push(FlatNode {
                            name: schema.name.clone(),
                            level: 1,
                            node_type: FlatNodeType::Schema {
                                database: database.name.clone(),
                                schema: schema.name.clone(),
                            },
                            is_expanded: schema.is_expanded,
                            has_children: has_schema_children,
                        });

                        if schema.is_expanded {
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
        let nodes = self.flat_nodes();
        let Some(node) = nodes.get(self.selected) else {
            return;
        };
        if !node.has_children {
            return;
        }

        match &node.node_type {
            FlatNodeType::Database(db_name) => {
                if let Some(db) = self.find_database_mut(db_name) {
                    db.toggle_expand();
                }
            }
            FlatNodeType::Schema { database, schema } => {
                if let Some(s) = self.find_schema_mut(database, schema) {
                    s.toggle_expand();
                }
            }
            FlatNodeType::Table { .. } => {}
        }
    }

    fn expand(&mut self) {
        let nodes = self.flat_nodes();
        let Some(node) = nodes.get(self.selected) else {
            return;
        };
        if !node.has_children || node.is_expanded {
            return;
        }

        match &node.node_type {
            FlatNodeType::Database(db_name) => {
                if let Some(db) = self.find_database_mut(db_name) {
                    db.expand();
                }
            }
            FlatNodeType::Schema { database, schema } => {
                if let Some(s) = self.find_schema_mut(database, schema) {
                    s.expand();
                }
            }
            FlatNodeType::Table { .. } => {}
        }
    }

    fn fold(&mut self) {
        let nodes = self.flat_nodes();
        let Some(node) = nodes.get(self.selected).cloned() else {
            return;
        };

        match &node.node_type {
            FlatNodeType::Database(db_name) => {
                if node.is_expanded {
                    if let Some(db) = self.find_database_mut(db_name) {
                        db.fold();
                    }
                }
            }
            FlatNodeType::Schema { database, schema } => {
                if node.is_expanded {
                    if let Some(s) = self.find_schema_mut(database, schema) {
                        s.fold();
                    }
                }
            }
            FlatNodeType::Table { database, schema, .. } => {
                // Fold parent and move cursor to it
                if let Some(schema_name) = schema {
                    if let Some(s) = self.find_schema_mut(database, schema_name) {
                        s.fold();
                    }
                    self.select_schema(database, schema_name);
                } else {
                    if let Some(db) = self.find_database_mut(database) {
                        db.fold();
                    }
                    self.select_database(database);
                }
            }
        }
    }

    fn find_database_mut(&mut self, name: &str) -> Option<&mut Database> {
        self.databases.iter_mut().find(|d| d.name == name)
    }

    fn find_schema_mut(&mut self, database: &str, schema: &str) -> Option<&mut Schema> {
        self.find_database_mut(database).and_then(|db| {
            db.children.iter_mut().find_map(|child| {
                if let Child::Schema(s) = child {
                    if s.name == schema {
                        return Some(s);
                    }
                }
                None
            })
        })
    }

    fn select_database(&mut self, db_name: &str) {
        for (i, node) in self.flat_nodes().iter().enumerate() {
            if let FlatNodeType::Database(name) = &node.node_type {
                if name == db_name {
                    self.selected = i;
                    return;
                }
            }
        }
    }

    fn select_schema(&mut self, db_name: &str, schema_name: &str) {
        for (i, node) in self.flat_nodes().iter().enumerate() {
            if let FlatNodeType::Schema { database, schema } = &node.node_type {
                if database == db_name && schema == schema_name {
                    self.selected = i;
                    return;
                }
            }
        }
    }

    fn selected_node(&self) -> Option<FlatNode> {
        self.flat_nodes().get(self.selected).cloned()
    }

    fn clamp_selected(&mut self) {
        let len = self.flat_nodes().len();
        if self.selected >= len && len > 0 {
            self.selected = len - 1;
        }
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
        self.selected = 0;
        self.filter_query.clear();
        self.focus = Focus::Tree;
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
        self.clamp_selected();
    }

    fn pop_filter_char(&mut self) {
        self.filter_query.pop();
        self.clamp_selected();
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
            DBListMsg::Expand => self.expand().into(),
            DBListMsg::Fold => self.fold().into(),
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
                Right | Char('l') => {
                    let Some(node) = self.selected_node() else {
                        return Update::none();
                    };
                    match &node.node_type {
                        FlatNodeType::Table { database, table, .. } => DBListMsg::SelectTable {
                            database: database.clone(),
                            table: table.clone(),
                        },
                        _ => DBListMsg::Expand,
                    }
                    .into()
                }
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
        let flat_nodes = self.flat_nodes();
        let items: Vec<ListItem> = if flat_nodes.is_empty() {
            vec![ListItem::new("(no database structure)")]
        } else {
            flat_nodes
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
        if !flat_nodes.is_empty() {
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
