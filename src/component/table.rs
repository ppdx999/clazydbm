use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Tabs},
};

use super::Component;
use crate::app::AppMsg;
use crate::connection::Connection;
use crate::db::{DB, DBBehavior, Records, TableProperties, DatabaseType};
use crate::logger::{debug, error};
use crate::update::{Command, Update};
use std::process::Command as StdCommand;

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub database: String,
    pub table: String,
}

pub enum TableMsg {
    FocusRecords,
    FocusSQL,
    FocusProperties,
    BackToDBList,
    LoadRecords(Connection),
    RecordsLoaded(Records),
    RecordsLoadFailed(String),
    LoadProperties(Connection),
    PropertiesLoaded(TableProperties),
    PropertiesLoadFailed(String),
    LaunchSQLCli(Connection),
    // Scrolling controls for Records view
    ScrollRecordsBy(i32),
    ScrollTop,
    ScrollBottom,
    // Horizontal column paging for Records view
    ScrollColsBy(i32),
    ColsStart,
    ColsEnd,
    // Scrolling controls for Properties view
    ScrollPropsBy(i32),
    ScrollPropsTop,
    ScrollPropsBottom,
    // Horizontal for Properties
    ScrollPropsColsBy(i32),
    PropsColsStart,
    PropsColsEnd,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TableFocus {
    Records,
    SQL,
    Properties,
}

pub struct TableComponent {
    table_info: Option<TableInfo>,
    connection: Option<Connection>,
    focus: TableFocus,
    records: Option<Records>,
    properties: Option<TableProperties>,
    records_scroll: usize,
    records_col_scroll: usize,
    properties_scroll: usize,
    properties_col_scroll: usize,
}

impl TableComponent {
    pub fn new() -> Self {
        Self {
            table_info: None,
            connection: None,
            focus: TableFocus::Records,
            records: None,
            properties: None,
            records_scroll: 0,
            records_col_scroll: 0,
            properties_scroll: 0,
            properties_col_scroll: 0,
        }
    }

    pub fn set_table(&mut self, database: String, table: String) {
        self.table_info = Some(TableInfo { database, table });
        self.records = None;
        self.properties = None;
        self.records_scroll = 0;
        self.records_col_scroll = 0;
        self.properties_scroll = 0;
        self.properties_col_scroll = 0;
    }

    pub fn set_connection(&mut self, conn: Connection) {
        self.connection = Some(conn);
    }


    fn get_cli_tool_name(db_type: &DatabaseType) -> &'static str {
        match db_type {
            DatabaseType::Postgres => "pgcli",
            DatabaseType::MySql => "mycli",
            DatabaseType::Sqlite => "litecli",
        }
    }

    fn check_cli_tool_available(tool_name: &str) -> bool {
        StdCommand::new("which")
            .arg(tool_name)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn launch_external_cli(conn: &Connection) -> Box<dyn FnOnce() -> Result<(), Box<dyn std::error::Error>> + Send> {
        let tool_name = Self::get_cli_tool_name(&conn.r#type);
        let conn = conn.clone();
        
        Box::new(move || {
            if !Self::check_cli_tool_available(tool_name) {
                return Err(format!("CLI tool '{}' not found. Please install it first.", tool_name).into());
            }

            let result = match conn.r#type {
                DatabaseType::Postgres | DatabaseType::MySql => {
                    let db_url = DB::database_url(&conn)
                        .map_err(|e| format!("Failed to build database URL: {}", e))?;
                    debug(&format!("Launching {} with URL: {}", tool_name, db_url));
                    
                    StdCommand::new(tool_name)
                        .arg(&db_url)
                        .status()
                }
                DatabaseType::Sqlite => {
                    // litecli expects the database file path directly
                    let path = conn.path.as_ref()
                        .ok_or_else(|| "SQLite connection requires a path")?;
                    debug(&format!("Launching {} with file: {:?}", tool_name, path));
                    
                    StdCommand::new(tool_name)
                        .arg(path)
                        .status()
                }
            };

            match result {
                Ok(status) => {
                    if status.success() {
                        debug(&format!("Successfully completed {}", tool_name));
                        Ok(())
                    } else {
                        Err(format!("{} exited with status: {}", tool_name, status).into())
                    }
                }
                Err(e) => Err(format!("Failed to launch {}: {}", tool_name, e).into()),
            }
        })
    }
}

impl Component for TableComponent {
    type Msg = TableMsg;

    fn update(&mut self, msg: Self::Msg) -> Update<Self::Msg> {
        match msg {
            TableMsg::FocusRecords => {
                self.focus = TableFocus::Records;
                Update::none()
            }
            TableMsg::FocusSQL => {
                self.focus = TableFocus::SQL;
                Update::none()
            }
            TableMsg::FocusProperties => {
                self.focus = TableFocus::Properties;
                Update::none()
            }
            TableMsg::BackToDBList => TableMsg::BackToDBList.into(),
            TableMsg::LoadRecords(conn) => {
                let Some(info) = self.table_info.clone() else {
                    return Update::none();
                };
                debug(&format!("Table: loading {}.{}", info.database, info.table));
                let task = move |tx: std::sync::mpsc::Sender<AppMsg>| {
                    let res = DB::fetch_records(&conn, &info.database, &info.table, 200, 0);
                    let msg = match res {
                        Ok(recs) => TableMsg::RecordsLoaded(recs).into(),
                        Err(e) => {
                            error(&format!("Table: load failed: {}", e));
                            TableMsg::RecordsLoadFailed(e.to_string()).into()
                        }
                    };
                    let _ = tx.send(msg);
                };
                Command::Spawn(Box::new(task)).into()
            }
            TableMsg::RecordsLoaded(recs) => {
                self.records = Some(recs);
                self.records_scroll = 0;
                self.records_col_scroll = 0;
                Update::none()
            }
            TableMsg::RecordsLoadFailed(_e) => Update::none(),
            TableMsg::LoadProperties(conn) => {
                let Some(info) = self.table_info.clone() else {
                    return Update::none();
                };
                debug(&format!("Props: loading {}.{}", info.database, info.table));
                let task = move |tx: std::sync::mpsc::Sender<AppMsg>| {
                    let res = DB::fetch_properties(&conn, &info.database, &info.table);
                    let msg = match res {
                        Ok(props) => TableMsg::PropertiesLoaded(props).into(),
                        Err(e) => {
                            error(&format!("Props: load failed: {}", e));
                            TableMsg::PropertiesLoadFailed(e.to_string()).into()
                        }
                    };
                    let _ = tx.send(msg);
                };
                Command::Spawn(Box::new(task)).into()
            }
            TableMsg::PropertiesLoaded(props) => {
                self.properties = Some(props);
                self.properties_scroll = 0;
                Update::none()
            }
            TableMsg::PropertiesLoadFailed(_e) => Update::none(),
            TableMsg::LaunchSQLCli(conn) => {
                let task = Self::launch_external_cli(&conn);
                Command::SuspendTerminal(task).into()
            }
            TableMsg::ScrollRecordsBy(delta) => {
                if matches!(self.focus, TableFocus::Records) {
                    if delta < 0 {
                        self.records_scroll = self.records_scroll.saturating_sub((-delta) as usize);
                    } else if delta > 0 {
                        self.records_scroll = self.records_scroll.saturating_add(delta as usize);
                    }
                }
                Update::none()
            }
            TableMsg::ScrollTop => {
                if matches!(self.focus, TableFocus::Records) {
                    self.records_scroll = 0;
                }
                Update::none()
            }
            TableMsg::ScrollBottom => {
                if matches!(self.focus, TableFocus::Records) {
                    // Will be clamped in draw
                    self.records_scroll = usize::MAX / 2;
                }
                Update::none()
            }
            TableMsg::ScrollColsBy(delta) => {
                if matches!(self.focus, TableFocus::Records) {
                    if delta < 0 {
                        self.records_col_scroll = self
                            .records_col_scroll
                            .saturating_sub((-delta) as usize);
                    } else if delta > 0 {
                        self.records_col_scroll = self
                            .records_col_scroll
                            .saturating_add(delta as usize);
                    }
                }
                Update::none()
            }
            TableMsg::ColsStart => {
                if matches!(self.focus, TableFocus::Records) {
                    self.records_col_scroll = 0;
                }
                Update::none()
            }
            TableMsg::ColsEnd => {
                if matches!(self.focus, TableFocus::Records) {
                    self.records_col_scroll = usize::MAX / 2;
                }
                Update::none()
            }
            TableMsg::ScrollPropsBy(delta) => {
                if matches!(self.focus, TableFocus::Properties) {
                    if delta < 0 {
                        self.properties_scroll = self.properties_scroll.saturating_sub((-delta) as usize);
                    } else if delta > 0 {
                        self.properties_scroll = self.properties_scroll.saturating_add(delta as usize);
                    }
                }
                Update::none()
            }
            TableMsg::ScrollPropsTop => {
                if matches!(self.focus, TableFocus::Properties) {
                    self.properties_scroll = 0;
                }
                Update::none()
            }
            TableMsg::ScrollPropsBottom => {
                if matches!(self.focus, TableFocus::Properties) {
                    self.properties_scroll = usize::MAX / 2;
                }
                Update::none()
            }
            TableMsg::ScrollPropsColsBy(delta) => {
                if matches!(self.focus, TableFocus::Properties) {
                    if delta < 0 {
                        self.properties_col_scroll = self
                            .properties_col_scroll
                            .saturating_sub((-delta) as usize);
                    } else if delta > 0 {
                        self.properties_col_scroll = self
                            .properties_col_scroll
                            .saturating_add(delta as usize);
                    }
                }
                Update::none()
            }
            TableMsg::PropsColsStart => {
                if matches!(self.focus, TableFocus::Properties) {
                    self.properties_col_scroll = 0;
                }
                Update::none()
            }
            TableMsg::PropsColsEnd => {
                if matches!(self.focus, TableFocus::Properties) {
                    self.properties_col_scroll = usize::MAX / 2;
                }
                Update::none()
            }
        }
    }

    fn handle_key(&self, key: KeyEvent) -> Update<Self::Msg> {
        use crossterm::event::KeyCode::*;

        match key.code {
            // Tab switching based on ARCHITECTURE.md
            Char('1') => TableMsg::FocusRecords.into(),
            Char('2') => TableMsg::FocusSQL.into(),
            Char('3') => TableMsg::FocusProperties.into(),
            // Back to DBList focus
            Tab | Esc => TableMsg::BackToDBList.into(),
            // Scrolling shortcuts: route based on focus
            Up => {
                if matches!(self.focus, TableFocus::Properties) {
                    TableMsg::ScrollPropsBy(-1).into()
                } else {
                    TableMsg::ScrollRecordsBy(-1).into()
                }
            }
            Down => {
                if matches!(self.focus, TableFocus::Properties) {
                    TableMsg::ScrollPropsBy(1).into()
                } else {
                    TableMsg::ScrollRecordsBy(1).into()
                }
            }
            PageUp => {
                if matches!(self.focus, TableFocus::Properties) {
                    TableMsg::ScrollPropsBy(-10).into()
                } else {
                    TableMsg::ScrollRecordsBy(-10).into()
                }
            }
            PageDown => {
                if matches!(self.focus, TableFocus::Properties) {
                    TableMsg::ScrollPropsBy(10).into()
                } else {
                    TableMsg::ScrollRecordsBy(10).into()
                }
            }
            Home => {
                if matches!(self.focus, TableFocus::Properties) {
                    TableMsg::ScrollPropsTop.into()
                } else {
                    TableMsg::ScrollTop.into()
                }
            }
            End => {
                if matches!(self.focus, TableFocus::Properties) {
                    TableMsg::ScrollPropsBottom.into()
                } else {
                    TableMsg::ScrollBottom.into()
                }
            }
            // Horizontal column paging: route to Records or Properties based on focus
            Left | Char('h') => {
                if matches!(self.focus, TableFocus::Properties) {
                    // Shift properties columns left by 1
                    TableMsg::ScrollPropsColsBy(-1).into()
                } else {
                    TableMsg::ScrollColsBy(-1).into()
                }
            }
            Right | Char('l') => {
                if matches!(self.focus, TableFocus::Properties) {
                    TableMsg::ScrollPropsColsBy(1).into()
                } else {
                    TableMsg::ScrollColsBy(1).into()
                }
            }
            // Jump columns by 5 using '[' and ']'
            Char('[') => {
                if matches!(self.focus, TableFocus::Properties) {
                    TableMsg::ScrollPropsColsBy(-5).into()
                } else {
                    TableMsg::ScrollColsBy(-5).into()
                }
            }
            Char(']') => {
                if matches!(self.focus, TableFocus::Properties) {
                    TableMsg::ScrollPropsColsBy(5).into()
                } else {
                    TableMsg::ScrollColsBy(5).into()
                }
            }
            // Go to first/last column with Ctrl-A / Ctrl-E
            Char('a') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                if matches!(self.focus, TableFocus::Properties) {
                    TableMsg::PropsColsStart.into()
                } else {
                    TableMsg::ColsStart.into()
                }
            }
            Char('e') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                if matches!(self.focus, TableFocus::Properties) {
                    TableMsg::PropsColsEnd.into()
                } else {
                    TableMsg::ColsEnd.into()
                }
            }
            Char('k') => {
                if matches!(self.focus, TableFocus::Properties) {
                    TableMsg::ScrollPropsBy(-1).into()
                } else {
                    TableMsg::ScrollRecordsBy(-1).into()
                }
            }
            Char('j') => {
                if matches!(self.focus, TableFocus::Properties) {
                    TableMsg::ScrollPropsBy(1).into()
                } else {
                    TableMsg::ScrollRecordsBy(1).into()
                }
            }
            Enter => {
                if matches!(self.focus, TableFocus::SQL) {
                    if let Some(conn) = &self.connection {
                        TableMsg::LaunchSQLCli(conn.clone()).into()
                    } else {
                        Update::none()
                    }
                } else {
                    Update::none()
                }
            }
            _ => Update::none(),
        }
    }

    fn draw(&self, f: &mut Frame, area: Rect, focused: bool) {
        if let Some(table_info) = &self.table_info {
            // Create tabs with hotkey hints (1/2/3)
            let tabs = vec!["Records [1]", "SQL [2]", "Properties [3]"];
            let selected_tab = match self.focus {
                TableFocus::Records => 0,
                TableFocus::SQL => 1,
                TableFocus::Properties => 2,
            };

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(area);

            // Tab bar
            let tab_style = if focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };

            let tabs_widget = Tabs::new(tabs)
                .block(
                    Block::default()
                        .title(format!("{}.{}", table_info.database, table_info.table))
                        .borders(Borders::ALL)
                        .border_style(tab_style),
                )
                .select(selected_tab)
                .style(Style::default().fg(Color::White))
                .highlight_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                );

            f.render_widget(tabs_widget, chunks[0]);

            // Content area
            let content_area = chunks[1];
            let content_style = if focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };

            match self.focus {
                TableFocus::Records => {
                    if let Some(recs) = &self.records {
                        use ratatui::widgets::{Cell as TuiCell, Row, Table as TuiTable};
                        // Determine visible columns based on width and horizontal scroll
                        let border_cols = 2u16; // left+right border
                        let col_width: u16 = 16; // fixed column width for rendering
                        let avail_w = content_area.width.saturating_sub(border_cols);
                        let visible_cols = std::cmp::max(1u16, avail_w / col_width) as usize;
                        let total_cols = recs.columns.len();
                        let max_col_start = total_cols.saturating_sub(visible_cols);
                        let col_start = self.records_col_scroll.min(max_col_start);
                        let col_end = (col_start + visible_cols).min(total_cols);

                        let header = Row::new(recs.columns[col_start..col_end].iter().map(|c| {
                            TuiCell::from(c.as_str())
                                .style(Style::default().add_modifier(Modifier::BOLD))
                        }));
                        // Compute visible rows slice based on area height and scroll offset
                        let border_rows = 2u16; // top+bottom border
                        let header_rows = 1u16; // header row
                        let avail = content_area
                            .height
                            .saturating_sub(border_rows)
                            .saturating_sub(header_rows);
                        let visible_count = usize::try_from(avail).unwrap_or(0);
                        let total = recs.rows.len();
                        let max_start = total.saturating_sub(visible_count);
                        let start = self.records_scroll.min(max_start);
                        let end = start.saturating_add(visible_count).min(total);
                        let rows = recs.rows[start..end]
                            .iter()
                            .map(|r| Row::new(r[col_start..col_end].iter().map(|v| v.as_str())));
                        let widths: Vec<Constraint> = (col_start..col_end)
                            .map(|_| Constraint::Length(col_width))
                            .collect();
                        let title = if total > 0 && visible_count > 0 {
                            format!(
                                "Records  rows [{}-{} / {}], cols [{}-{} / {}]  (↑/↓, PgUp/PgDn, Home/End; ←/→, [/], Ctrl-A/E)",
                                start.saturating_add(1), end, total,
                                col_start.saturating_add(1), col_end, total_cols
                            )
                        } else {
                            "Records".to_string()
                        };
                        let table = TuiTable::new(rows, widths).header(header).block(
                            Block::default()
                                .title(title)
                                .borders(Borders::ALL)
                                .border_style(content_style),
                        );
                        f.render_widget(table, content_area);
                    } else {
                        let records_block = Block::default()
                            .title("Records")
                            .borders(Borders::ALL)
                            .border_style(content_style);
                        let records_content =
                            Paragraph::new("Loading records...").block(records_block);
                        f.render_widget(records_content, content_area);
                    }
                }
                TableFocus::SQL => {
                    let sql_block = Block::default()
                        .title("SQL")
                        .borders(Borders::ALL)
                        .border_style(content_style);

                    let (tool_info, instructions) = if let Some(conn) = &self.connection {
                        let tool_name = Self::get_cli_tool_name(&conn.r#type);
                        let available = Self::check_cli_tool_available(tool_name);
                        
                        if available {
                            (
                                format!("External CLI tool: {} (available)", tool_name),
                                "Press [Enter] to launch external SQL CLI\n\nThis will open the appropriate CLI tool:\n• PostgreSQL: pgcli\n• MySQL: mycli\n• SQLite: litecli".to_string()
                            )
                        } else {
                            (
                                format!("External CLI tool: {} (NOT INSTALLED)", tool_name),
                                format!("Please install {} to use SQL functionality:\n\npip install {}", tool_name, tool_name)
                            )
                        }
                    } else {
                        ("No connection available".to_string(), "No database connection available".to_string())
                    };

                    let sql_content = Paragraph::new(format!("{}\n\n{}", tool_info, instructions))
                        .block(sql_block);

                    f.render_widget(sql_content, content_area);
                }
                TableFocus::Properties => {
                    if let Some(props) = &self.properties {
                        use ratatui::widgets::{Cell as TuiCell, Row, Table as TuiTable};
                        // Build headers and widths with concise labels
                        let header_labels = ["Column", "Type", "N", "Def", "PK"];
                        let widths_all: [u16; 5] = [20, 14, 3, 20, 3];
                        // Horizontal column window calculation based on available width
                        let border_cols = 2u16; // left+right borders
                        let avail_w = content_area.width.saturating_sub(border_cols);
                        // Calculate start from scroll offset
                        let col_start = self.properties_col_scroll.min(header_labels.len().saturating_sub(1));
                        // Determine how many columns fit from col_start
                        let mut sum = 0u16;
                        let mut col_end = col_start;
                        while col_end < header_labels.len() {
                            let w = widths_all[col_end];
                            if sum + w > avail_w { break; }
                            sum += w;
                            col_end += 1;
                        }
                        if col_end == col_start { col_end = (col_start + 1).min(header_labels.len()); }
                        let header = Row::new(header_labels[col_start..col_end].iter().map(|c| {
                            TuiCell::from(*c).style(Style::default().add_modifier(Modifier::BOLD))
                        }));
                        // Visible slice based on height and properties_scroll
                        let border_rows = 2u16;
                        let header_rows = 1u16;
                        let avail = content_area
                            .height
                            .saturating_sub(border_rows)
                            .saturating_sub(header_rows);
                        let visible_count = usize::try_from(avail).unwrap_or(0);
                        let total = props.columns.len();
                        let max_start = total.saturating_sub(visible_count);
                        let start = self.properties_scroll.min(max_start);
                        let end = start.saturating_add(visible_count).min(total);
                        let rows = props.columns[start..end].iter().map(|c| {
                            let fields_all = [
                                c.name.as_str(),
                                c.data_type.as_str(),
                                if c.nullable { "YES" } else { "NO" },
                                c.default.as_deref().unwrap_or(""),
                                if c.primary_key { "✔" } else { "" },
                            ];
                            Row::new(fields_all[col_start..col_end].iter().cloned())
                        });
                        let widths = widths_all[col_start..col_end]
                            .iter()
                            .cloned()
                            .map(Constraint::Length)
                            .collect::<Vec<_>>();
                        let title = if total > 0 && visible_count > 0 {
                            format!(
                                "Properties  rows [{}-{} / {}], cols [{}-{} / {}]  (↑/↓, PgUp/PgDn, Home/End; ←/→)",
                                start.saturating_add(1), end, total,
                                col_start.saturating_add(1), col_end, header_labels.len()
                            )
                        } else {
                            "Properties".to_string()
                        };
                        let table = TuiTable::new(rows, widths)
                            .header(header)
                            .block(
                                Block::default()
                                    .title(title)
                                    .borders(Borders::ALL)
                                    .border_style(content_style),
                            );
                        f.render_widget(table, content_area);
                    } else {
                        let properties_block = Block::default()
                            .title("Properties")
                            .borders(Borders::ALL)
                            .border_style(content_style);
                        let properties_content =
                            Paragraph::new("Loading properties...").block(properties_block);
                        f.render_widget(properties_content, content_area);
                    }
                }
            }
        } else {
            // No table selected
            let block = Block::default()
                .title("Table View")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White));

            let content = Paragraph::new(
                "No table selected\n\nSelect a table from the database structure on the left.",
            )
            .block(block);

            f.render_widget(content, area);
        }
    }
}
