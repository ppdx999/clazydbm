use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Tabs},
};

use super::Component;
use crate::cmd::Update;

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
}

impl TableComponent {
    pub fn new() -> Self {
        Self {
            table_info: None,
            focus: TableFocus::Records,
        }
    }

    pub fn set_table(&mut self, database: String, table: String) {
        self.table_info = Some(TableInfo { database, table });
    }

    pub fn get_table_info(&self) -> Option<&TableInfo> {
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
            TableMsg::BackToDBList => Update::msg(TableMsg::BackToDBList),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Update<Self::Msg> {
        use crossterm::event::KeyCode::*;

        match key.code {
            // Tab switching based on ARCHITECTURE.md
            Char('1') => Update::msg(TableMsg::FocusRecords),
            Char('2') => Update::msg(TableMsg::FocusSQL),
            Char('3') => Update::msg(TableMsg::FocusProperties),
            // Back to DBList focus
            Esc => Update::msg(TableMsg::BackToDBList),
            _ => Update::none(),
        }
    }

    fn draw(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        if let Some(table_info) = &self.table_info {
            // Create tabs
            let tabs = vec!["Records", "SQL", "Properties"];
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
                    let records_block = Block::default()
                        .title("Records")
                        .borders(Borders::ALL)
                        .border_style(content_style);

                    let records_content = Paragraph::new("Records view - Not implemented yet\n\nPress:\n- j/k: Navigate rows\n- h/l: Navigate columns\n- /: Filter records")
                        .block(records_block);

                    f.render_widget(records_content, content_area);
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
