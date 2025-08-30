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
use crate::db::{DB, DBBehavior, Records, TableProperties};
use crate::logger::{debug, error};
use crate::update::{Command, Update};

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
    // Scrolling controls for Records view
    ScrollRecordsBy(i32),
    ScrollTop,
    ScrollBottom,
    // Scrolling controls for Properties view
    ScrollPropsBy(i32),
    ScrollPropsTop,
    ScrollPropsBottom,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TableFocus {
    Records,
    SQL,
    Properties,
}

pub struct TableComponent {
    table_info: Option<TableInfo>,
    focus: TableFocus,
    records: Option<Records>,
    properties: Option<TableProperties>,
    records_scroll: usize,
    properties_scroll: usize,
}

impl TableComponent {
    pub fn new() -> Self {
        Self {
            table_info: None,
            focus: TableFocus::Records,
            records: None,
            properties: None,
            records_scroll: 0,
            properties_scroll: 0,
        }
    }

    pub fn set_table(&mut self, database: String, table: String) {
        self.table_info = Some(TableInfo { database, table });
        self.records = None;
        self.properties = None;
        self.records_scroll = 0;
        self.properties_scroll = 0;
    }

    fn get_table_info(&self) -> Option<&TableInfo> {
        self.table_info.as_ref()
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
                        let header = Row::new(recs.columns.iter().map(|c| {
                            TuiCell::from(c.as_str())
                                .style(Style::default().add_modifier(Modifier::BOLD))
                        }));
                        // Compute visible slice based on area height and scroll offset
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
                            .map(|r| Row::new(r.iter().map(|v| v.as_str())));
                        let widths: Vec<Constraint> = recs
                            .columns
                            .iter()
                            .map(|_| Constraint::Length(20))
                            .collect();
                        let title = if total > 0 && visible_count > 0 {
                            format!(
                                "Records  [{}-{} / {}]  (↑/↓, PgUp/PgDn, Home/End)",
                                start.saturating_add(1),
                                end,
                                total
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

                    let sql_content = Paragraph::new(
                        "SQL view - Not implemented yet\n\nWrite and execute SQL queries here.",
                    )
                    .block(sql_block);

                    f.render_widget(sql_content, content_area);
                }
                TableFocus::Properties => {
                    if let Some(props) = &self.properties {
                        use ratatui::widgets::{Cell as TuiCell, Row, Table as TuiTable};
                        let header = Row::new([
                            TuiCell::from("Column")
                                .style(Style::default().add_modifier(Modifier::BOLD)),
                            TuiCell::from("Type")
                                .style(Style::default().add_modifier(Modifier::BOLD)),
                            TuiCell::from("Null")
                                .style(Style::default().add_modifier(Modifier::BOLD)),
                            TuiCell::from("Default")
                                .style(Style::default().add_modifier(Modifier::BOLD)),
                            TuiCell::from("PK")
                                .style(Style::default().add_modifier(Modifier::BOLD)),
                        ]);
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
                            Row::new([
                                c.name.as_str(),
                                c.data_type.as_str(),
                                if c.nullable { "YES" } else { "NO" },
                                c.default.as_deref().unwrap_or(""),
                                if c.primary_key { "✔" } else { "" },
                            ])
                        });
                        let widths = [
                            Constraint::Length(24),
                            Constraint::Length(18),
                            Constraint::Length(6),
                            Constraint::Length(24),
                            Constraint::Length(4),
                        ];
                        let title = if total > 0 && visible_count > 0 {
                            format!(
                                "Properties  [{}-{} / {}]  (↑/↓, PgUp/PgDn, Home/End)",
                                start.saturating_add(1),
                                end,
                                total
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
