use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Tabs},
};

use super::Component;
use crate::app::AppMsg;
use crate::update::{Command, Update};
use crate::connection::Connection;
use crate::db::{DB, DBBehavior, Records};
use crate::logger::{debug, error};

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
    TableSelected { database: String, table: String },
    LoadRecords(Connection),
    RecordsLoaded(Records),
    RecordsLoadFailed(String),
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
}

impl TableComponent {
    pub fn new() -> Self {
        Self {
            table_info: None,
            focus: TableFocus::Records,
            records: None,
        }
    }

    fn set_table(&mut self, database: String, table: String) {
        self.table_info = Some(TableInfo { database, table });
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
            TableMsg::TableSelected { database, table } => {
                self.set_table(database, table);
                self.records = None; // Clear previous records
                Update::none()
            }
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
                Update::none()
            }
            TableMsg::RecordsLoadFailed(_e) => Update::none(),
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
                        let rows = recs
                            .rows
                            .iter()
                            .map(|r| Row::new(r.iter().map(|v| v.as_str())));
                        let widths: Vec<Constraint> = recs
                            .columns
                            .iter()
                            .map(|_| Constraint::Length(20))
                            .collect();
                        let table = TuiTable::new(rows, widths).header(header).block(
                            Block::default()
                                .title("Records")
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
                    let properties_block = Block::default()
                        .title("Properties")
                        .borders(Borders::ALL)
                        .border_style(content_style);

                    let properties_content = Paragraph::new("Properties view - Not implemented yet\n\nTable schema and metadata will be shown here.")
                        .block(properties_block);

                    f.render_widget(properties_content, content_area);
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
