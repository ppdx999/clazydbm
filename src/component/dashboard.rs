use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use super::{Component, DBListComponent, DBListMsg, TableComponent, TableMsg};
use crate::{cmd::MapMsg, cmd::Update, connection::Connection};

/// Messages the Dashboard component can emit
pub enum DashboardMsg {
    /// Request to leave dashboard back to Connection
    Leave,
    /// DBList wants to select a table
    SelectTable {
        database: String,
        table: String,
    },
    /// Table wants to go back to DBList focus
    BackToDBList,
    DBListMsg(DBListMsg),
    TableMsg(TableMsg),
}

impl From<DBListMsg> for DashboardMsg {
    fn from(msg: DBListMsg) -> Self {
        match msg {
            DBListMsg::SelectTable { database, table } => {
                DashboardMsg::SelectTable { database, table }
            }
            m => DashboardMsg::DBListMsg(m),
        }
    }
}
impl From<TableMsg> for DashboardMsg {
    fn from(msg: TableMsg) -> Self {
        match msg {
            TableMsg::BackToDBList => DashboardMsg::BackToDBList,
            m => DashboardMsg::TableMsg(m),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DashboardFocus {
    DBList,
    Table,
}

pub struct DashboardComponent {
    dblist: DBListComponent,
    table: TableComponent,
    focus: DashboardFocus,
    connection: Option<Connection>,
}

impl DashboardComponent {
    pub fn new() -> Self {
        Self {
            dblist: DBListComponent::new(),
            table: TableComponent::new(),
            focus: DashboardFocus::DBList,
            connection: None,
        }
    }

    pub fn set_connection(&mut self, conn: Connection) {
        self.connection = Some(conn.clone());
    }

    fn move_to_table(&mut self, database: String, table: String) -> Update<DashboardMsg> {
        self.table.set_table(database, table);
        self.focus = DashboardFocus::Table;
        Update::none()
    }

    fn move_to_dblist(&mut self) -> Update<DashboardMsg> {
        self.focus = DashboardFocus::DBList;
        Update::none()
    }
}

impl Component for DashboardComponent {
    type Msg = DashboardMsg;

    fn update(&mut self, msg: Self::Msg) -> Update<Self::Msg> {
        match msg {
            DashboardMsg::SelectTable { database, table } => self.move_to_table(database, table),
            DashboardMsg::BackToDBList => self.move_to_dblist(),
            DashboardMsg::Leave => Update::msg(DashboardMsg::Leave),
            DashboardMsg::DBListMsg(m) => self.dblist.update(m).map_auto(),
            DashboardMsg::TableMsg(m) => self.table.update(m).map_auto(),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Update<Self::Msg> {
        // Global keys
        if key.code == crossterm::event::KeyCode::Esc {
            return Update::msg(DashboardMsg::Leave);
        }

        // Forward key to focused component - let update handle side effects
        match self.focus {
            DashboardFocus::DBList => self.dblist.handle_key(key).map_auto(),
            DashboardFocus::Table => self.table.handle_key(key).map_auto(),
        }
    }

    fn draw(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        // Create layout: 15% left (DBList), 85% right (Table)
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(15), Constraint::Percentage(85)])
            .split(area);

        let dblist_area = chunks[0];
        let table_area = chunks[1];

        // Draw DBList
        let dblist_focused = focused && matches!(self.focus, DashboardFocus::DBList);
        self.dblist.draw(f, dblist_area, dblist_focused);

        // Draw Table
        let table_focused = focused && matches!(self.focus, DashboardFocus::Table);
        self.table.draw(f, table_area, table_focused);
    }
}
