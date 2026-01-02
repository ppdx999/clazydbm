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

#[derive(Debug, Clone, Copy, PartialEq)]
enum NodePath {
    Database(usize),
    Schema(usize, usize),
    TableInDb(usize, usize),
    TableInSchema(usize, usize, usize),
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Databases {
    data: Vec<Database>,
    selected: Option<NodePath>,
    filter: String,
}

impl Databases {
    pub fn new(databases: Vec<Database>) -> Self {
        Self {
            data: databases,
            selected: None,
            filter: String::new(),
        }
    }

    // Filter methods
    pub fn filter(&self) -> &str {
        &self.filter
    }

    pub fn push_filter_char(&mut self, c: char) {
        self.filter.push(c);
    }

    pub fn pop_filter_char(&mut self) {
        self.filter.pop();
    }

    pub fn clear_filter(&mut self) {
        self.filter.clear();
    }

    fn filter_lower(&self) -> String {
        self.filter.to_lowercase()
    }

    fn node_matches_filter(&self, path: NodePath) -> bool {
        if self.filter.is_empty() {
            return true;
        }
        let filter = self.filter_lower();
        match path {
            NodePath::Database(db_idx) => self.db_matches_filter(&self.data[db_idx], &filter),
            NodePath::Schema(db_idx, child_idx) => {
                if let Child::Schema(s) = &self.data[db_idx].children[child_idx] {
                    self.schema_matches_filter(s, &filter)
                } else {
                    false
                }
            }
            NodePath::TableInDb(db_idx, child_idx) => {
                if let Child::Table(t) = &self.data[db_idx].children[child_idx] {
                    t.name.to_lowercase().contains(&filter)
                } else {
                    false
                }
            }
            NodePath::TableInSchema(db_idx, child_idx, table_idx) => {
                if let Child::Schema(s) = &self.data[db_idx].children[child_idx] {
                    s.tables[table_idx].name.to_lowercase().contains(&filter)
                } else {
                    false
                }
            }
        }
    }

    pub fn select_first(&mut self) {
        // Find first visible (filtered) database
        for db_idx in 0..self.data.len() {
            let path = NodePath::Database(db_idx);
            if self.node_matches_filter(path) {
                self.selected = Some(path);
                return;
            }
        }
        self.selected = None;
    }

    pub fn select_last(&mut self) {
        // Find last visible node
        for db_idx in (0..self.data.len()).rev() {
            if self.node_matches_filter(NodePath::Database(db_idx)) {
                self.selected = Some(self.last_visible_in_db_filtered(db_idx));
                return;
            }
        }
        self.selected = None;
    }

    pub fn select_next(&mut self) {
        if let Some(next) = self.find_next_filtered() {
            self.selected = Some(next);
        }
    }

    pub fn select_prev(&mut self) {
        if let Some(prev) = self.find_prev_filtered() {
            self.selected = Some(prev);
        }
    }

    fn find_next_filtered(&self) -> Option<NodePath> {
        let mut current = self.selected?;

        loop {
            let next = self.find_next_raw(current)?;
            if self.node_matches_filter(next) {
                return Some(next);
            }
            current = next;
        }
    }

    fn find_prev_filtered(&self) -> Option<NodePath> {
        let mut current = self.selected?;

        loop {
            let prev = self.find_prev_raw(current)?;
            if self.node_matches_filter(prev) {
                return Some(prev);
            }
            current = prev;
        }
    }

    fn find_next_raw(&self, current: NodePath) -> Option<NodePath> {
        match current {
            NodePath::Database(db_idx) => {
                let db = &self.data[db_idx];
                if db.is_expanded && !db.children.is_empty() {
                    match &db.children[0] {
                        Child::Table(_) => Some(NodePath::TableInDb(db_idx, 0)),
                        Child::Schema(_) => Some(NodePath::Schema(db_idx, 0)),
                    }
                } else if db_idx + 1 < self.data.len() {
                    Some(NodePath::Database(db_idx + 1))
                } else {
                    None
                }
            }
            NodePath::Schema(db_idx, child_idx) => {
                if let Child::Schema(s) = &self.data[db_idx].children[child_idx] {
                    if s.is_expanded && !s.tables.is_empty() {
                        Some(NodePath::TableInSchema(db_idx, child_idx, 0))
                    } else {
                        self.next_sibling_or_parent(db_idx, child_idx)
                    }
                } else {
                    None
                }
            }
            NodePath::TableInDb(db_idx, child_idx) => {
                self.next_sibling_or_parent(db_idx, child_idx)
            }
            NodePath::TableInSchema(db_idx, child_idx, table_idx) => {
                if let Child::Schema(s) = &self.data[db_idx].children[child_idx] {
                    if table_idx + 1 < s.tables.len() {
                        Some(NodePath::TableInSchema(db_idx, child_idx, table_idx + 1))
                    } else {
                        self.next_sibling_or_parent(db_idx, child_idx)
                    }
                } else {
                    None
                }
            }
        }
    }

    fn next_sibling_or_parent(&self, db_idx: usize, child_idx: usize) -> Option<NodePath> {
        let db = &self.data[db_idx];
        if child_idx + 1 < db.children.len() {
            match &db.children[child_idx + 1] {
                Child::Table(_) => Some(NodePath::TableInDb(db_idx, child_idx + 1)),
                Child::Schema(_) => Some(NodePath::Schema(db_idx, child_idx + 1)),
            }
        } else if db_idx + 1 < self.data.len() {
            Some(NodePath::Database(db_idx + 1))
        } else {
            None
        }
    }

    fn find_prev_raw(&self, current: NodePath) -> Option<NodePath> {
        match current {
            NodePath::Database(db_idx) => {
                if db_idx == 0 {
                    None
                } else {
                    Some(self.last_visible_in_db(db_idx - 1))
                }
            }
            NodePath::Schema(db_idx, child_idx) | NodePath::TableInDb(db_idx, child_idx) => {
                if child_idx == 0 {
                    Some(NodePath::Database(db_idx))
                } else {
                    Some(self.last_visible_in_child(db_idx, child_idx - 1))
                }
            }
            NodePath::TableInSchema(db_idx, child_idx, table_idx) => {
                if table_idx == 0 {
                    Some(NodePath::Schema(db_idx, child_idx))
                } else {
                    Some(NodePath::TableInSchema(db_idx, child_idx, table_idx - 1))
                }
            }
        }
    }

    fn last_visible_in_db(&self, db_idx: usize) -> NodePath {
        let db = &self.data[db_idx];
        if db.is_expanded && !db.children.is_empty() {
            self.last_visible_in_child(db_idx, db.children.len() - 1)
        } else {
            NodePath::Database(db_idx)
        }
    }

    fn last_visible_in_db_filtered(&self, db_idx: usize) -> NodePath {
        let db = &self.data[db_idx];
        if db.is_expanded && !db.children.is_empty() {
            // Find last matching child
            for child_idx in (0..db.children.len()).rev() {
                let path = match &db.children[child_idx] {
                    Child::Table(_) => NodePath::TableInDb(db_idx, child_idx),
                    Child::Schema(_) => NodePath::Schema(db_idx, child_idx),
                };
                if self.node_matches_filter(path) {
                    return self.last_visible_in_child_filtered(db_idx, child_idx);
                }
            }
        }
        NodePath::Database(db_idx)
    }

    fn last_visible_in_child(&self, db_idx: usize, child_idx: usize) -> NodePath {
        match &self.data[db_idx].children[child_idx] {
            Child::Table(_) => NodePath::TableInDb(db_idx, child_idx),
            Child::Schema(s) => {
                if s.is_expanded && !s.tables.is_empty() {
                    NodePath::TableInSchema(db_idx, child_idx, s.tables.len() - 1)
                } else {
                    NodePath::Schema(db_idx, child_idx)
                }
            }
        }
    }

    fn last_visible_in_child_filtered(&self, db_idx: usize, child_idx: usize) -> NodePath {
        match &self.data[db_idx].children[child_idx] {
            Child::Table(_) => NodePath::TableInDb(db_idx, child_idx),
            Child::Schema(s) => {
                if s.is_expanded && !s.tables.is_empty() {
                    let filter = self.filter_lower();
                    for table_idx in (0..s.tables.len()).rev() {
                        if self.filter.is_empty() || s.tables[table_idx].name.to_lowercase().contains(&filter) {
                            return NodePath::TableInSchema(db_idx, child_idx, table_idx);
                        }
                    }
                }
                NodePath::Schema(db_idx, child_idx)
            }
        }
    }

    pub fn get_selected(&self) -> Option<SelectedRef> {
        match self.selected? {
            NodePath::Database(db_idx) => {
                Some(SelectedRef::Database(&self.data[db_idx].name))
            }
            NodePath::Schema(db_idx, child_idx) => {
                if let Child::Schema(s) = &self.data[db_idx].children[child_idx] {
                    Some(SelectedRef::Schema {
                        database: &self.data[db_idx].name,
                        schema: &s.name,
                    })
                } else {
                    None
                }
            }
            NodePath::TableInDb(db_idx, child_idx) => {
                if let Child::Table(t) = &self.data[db_idx].children[child_idx] {
                    Some(SelectedRef::Table {
                        database: &self.data[db_idx].name,
                        schema: None,
                        table: &t.name,
                    })
                } else {
                    None
                }
            }
            NodePath::TableInSchema(db_idx, child_idx, table_idx) => {
                if let Child::Schema(s) = &self.data[db_idx].children[child_idx] {
                    Some(SelectedRef::Table {
                        database: &self.data[db_idx].name,
                        schema: Some(&s.name),
                        table: &s.tables[table_idx].name,
                    })
                } else {
                    None
                }
            }
        }
    }

    pub fn toggle_expand_selected(&mut self) {
        match self.selected {
            Some(NodePath::Database(db_idx)) => {
                let db = &mut self.data[db_idx];
                if db.has_children() {
                    db.toggle_expand();
                }
            }
            Some(NodePath::Schema(db_idx, child_idx)) => {
                if let Child::Schema(s) = &mut self.data[db_idx].children[child_idx] {
                    if s.has_children() {
                        s.toggle_expand();
                    }
                }
            }
            _ => {}
        }
    }

    pub fn expand_selected(&mut self) {
        match self.selected {
            Some(NodePath::Database(db_idx)) => {
                let db = &mut self.data[db_idx];
                if db.has_children() && !db.is_expanded {
                    db.expand();
                }
            }
            Some(NodePath::Schema(db_idx, child_idx)) => {
                if let Child::Schema(s) = &mut self.data[db_idx].children[child_idx] {
                    if s.has_children() && !s.is_expanded {
                        s.expand();
                    }
                }
            }
            _ => {}
        }
    }

    pub fn fold_selected(&mut self) {
        match self.selected {
            Some(NodePath::Database(db_idx)) => {
                self.data[db_idx].fold();
            }
            Some(NodePath::Schema(db_idx, child_idx)) => {
                if let Child::Schema(s) = &mut self.data[db_idx].children[child_idx] {
                    s.fold();
                }
            }
            Some(NodePath::TableInDb(db_idx, _)) => {
                // Table selected: fold parent database and select it
                self.data[db_idx].fold();
                self.selected = Some(NodePath::Database(db_idx));
            }
            Some(NodePath::TableInSchema(db_idx, child_idx, _)) => {
                // Table in schema: fold schema and select it
                if let Child::Schema(s) = &mut self.data[db_idx].children[child_idx] {
                    s.fold();
                }
                self.selected = Some(NodePath::Schema(db_idx, child_idx));
            }
            None => {}
        }
    }

    /// Build list items with filter applied, return (items, selected_index)
    pub fn build_list_items(&self) -> (Vec<(String, usize)>, Option<usize>) {
        let mut items = Vec::new();
        let mut selected_index = None;
        let mut index = 0;
        let filter_lower = self.filter_lower();

        for (db_idx, db) in self.data.iter().enumerate() {
            // Check if database or any children match filter
            let db_matches = self.filter.is_empty() || self.db_matches_filter(db, &filter_lower);
            if !db_matches {
                continue;
            }

            let prefix = if db.is_expanded {
                "▼ 📁"
            } else if db.has_children() {
                "▶ 📁"
            } else {
                "  📁"
            };
            items.push((format!("{} {}", prefix, db.name), 0));
            if self.selected == Some(NodePath::Database(db_idx)) {
                selected_index = Some(index);
            }
            index += 1;

            if db.is_expanded {
                for (child_idx, child) in db.children.iter().enumerate() {
                    match child {
                        Child::Table(t) => {
                            if !self.filter.is_empty() && !t.name.to_lowercase().contains(&filter_lower) {
                                continue;
                            }
                            items.push((format!("    📄 {}", t.name), 1));
                            if self.selected == Some(NodePath::TableInDb(db_idx, child_idx)) {
                                selected_index = Some(index);
                            }
                            index += 1;
                        }
                        Child::Schema(s) => {
                            let schema_matches = self.filter.is_empty() || self.schema_matches_filter(s, &filter_lower);
                            if !schema_matches {
                                continue;
                            }

                            let prefix = if s.is_expanded {
                                "▼ 📂"
                            } else if s.has_children() {
                                "▶ 📂"
                            } else {
                                "  📂"
                            };
                            items.push((format!("  {} {}", prefix, s.name), 1));
                            if self.selected == Some(NodePath::Schema(db_idx, child_idx)) {
                                selected_index = Some(index);
                            }
                            index += 1;

                            if s.is_expanded {
                                for (table_idx, t) in s.tables.iter().enumerate() {
                                    if !self.filter.is_empty() && !t.name.to_lowercase().contains(&filter_lower) {
                                        continue;
                                    }
                                    items.push((format!("      📄 {}", t.name), 2));
                                    if self.selected == Some(NodePath::TableInSchema(db_idx, child_idx, table_idx)) {
                                        selected_index = Some(index);
                                    }
                                    index += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        (items, selected_index)
    }

    fn db_matches_filter(&self, db: &Database, filter: &str) -> bool {
        if db.name.to_lowercase().contains(filter) {
            return true;
        }
        for child in &db.children {
            match child {
                Child::Table(t) if t.name.to_lowercase().contains(filter) => return true,
                Child::Schema(s) if self.schema_matches_filter(s, filter) => return true,
                _ => {}
            }
        }
        false
    }

    fn schema_matches_filter(&self, schema: &Schema, filter: &str) -> bool {
        if schema.name.to_lowercase().contains(filter) {
            return true;
        }
        schema.tables.iter().any(|t| t.name.to_lowercase().contains(filter))
    }
}

pub enum SelectedRef<'a> {
    Database(&'a str),
    Schema { database: &'a str, schema: &'a str },
    Table { database: &'a str, schema: Option<&'a str>, table: &'a str },
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

pub struct DBListComponent {
    databases: Databases,
    focus: Focus,
}

impl DBListComponent {
    pub fn new() -> Self {
        Self {
            databases: Databases::default(),
            focus: Focus::Tree,
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
        self.databases = Databases::new(dbs);
        self.focus = Focus::Tree;
        self.databases.select_first();
        Update::none()
    }

    fn move_focus_to_filter(&mut self) {
        self.focus = Focus::Filter;
    }

    fn move_focus_to_tree(&mut self) {
        self.focus = Focus::Tree;
    }

    fn push_filter_char(&mut self, c: char) {
        self.databases.push_filter_char(c);
    }

    fn pop_filter_char(&mut self) {
        self.databases.pop_filter_char();
    }

    fn move_up(&mut self) {
        self.databases.select_prev();
    }

    fn move_down(&mut self) {
        self.databases.select_next();
    }

    fn move_top(&mut self) {
        self.databases.select_first();
    }

    fn move_bottom(&mut self) {
        self.databases.select_last();
    }

    fn expand(&mut self) {
        self.databases.expand_selected();
    }

    fn fold(&mut self) {
        self.databases.fold_selected();
    }

    fn toggle_expand(&mut self) {
        self.databases.toggle_expand_selected();
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
                    match self.databases.get_selected() {
                        Some(SelectedRef::Table { database, table, .. }) => DBListMsg::SelectTable {
                            database: database.to_string(),
                            table: table.to_string(),
                        },
                        Some(SelectedRef::Database(_)) | Some(SelectedRef::Schema { .. }) => DBListMsg::Expand,
                        None => return Update::none(),
                    }
                    .into()
                }
                Left | Char('h') => DBListMsg::Fold.into(),
                Char('/') => DBListMsg::Filter.into(),
                Esc => DBListMsg::LeaveDashboard.into(),
                Enter | Tab => {
                    match self.databases.get_selected() {
                        Some(SelectedRef::Table { database, table, .. }) => DBListMsg::SelectTable {
                            database: database.to_string(),
                            table: table.to_string(),
                        },
                        Some(SelectedRef::Database(_)) | Some(SelectedRef::Schema { .. }) => DBListMsg::ToggleExpand,
                        None => return Update::none(),
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

        let tree_area = chunks[0];
        let filter_area = chunks[1];

        // Build list items
        let (list_items, selected_index) = self.databases.build_list_items();
        let items: Vec<ListItem> = if list_items.is_empty() {
            vec![ListItem::new("(no database structure)")]
        } else {
            list_items
                .into_iter()
                .map(|(text, _)| ListItem::new(Span::raw(text)))
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
        state.select(selected_index);

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
            format!("{}_", self.databases.filter())
        } else {
            self.databases.filter().to_string()
        };

        let filter_paragraph = ratatui::widgets::Paragraph::new(filter_text).block(filter_block);
        f.render_widget(filter_paragraph, filter_area);
    }
}
