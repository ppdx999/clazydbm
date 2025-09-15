use crossterm::event::KeyEvent;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use super::{Component, DBListComponent, DBListMsg, TableComponent, TableMsg};
use crate::{
    update::{MapMsg, Update},
    connection::Connection,
};

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
    ConnectionSelected(Connection),
    DBListMsg(DBListMsg),
    TableMsg(TableMsg),
}

impl From<DBListMsg> for DashboardMsg {
    fn from(msg: DBListMsg) -> Self {
        match msg {
            DBListMsg::SelectTable { database, table } => {
                DashboardMsg::SelectTable { database, table }
            }
            DBListMsg::LeaveDashboard => DashboardMsg::Leave,
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

    fn move_to_table(&mut self, database: String, table: String) -> Update<DashboardMsg> {
        self.table.set_table(database, table);
        if let Some(conn) = &self.connection {
            self.table.set_connection(conn.clone());
        }
        self.focus = DashboardFocus::Table;
        if let Some(conn) = &self.connection {
            DashboardMsg::TableMsg(TableMsg::LoadRecords(conn.clone())).into()
        } else {
            Update::none()
        }
    }

    fn move_to_dblist(&mut self) -> Update<DashboardMsg> {
        self.focus = DashboardFocus::DBList;
        Update::none()
    }

    fn on_connection_selected(&mut self, conn: Connection) -> Update<DashboardMsg> {
        // Store selected connection
        self.connection = Some(conn.clone());
        // Trigger DBList load immediately
        self.dblist.update(DBListMsg::Load(conn)).map_auto()
    }
}

impl Component for DashboardComponent {
    type Msg = DashboardMsg;

    fn update(&mut self, msg: Self::Msg) -> Update<Self::Msg> {
        match msg {
            DashboardMsg::SelectTable { database, table } => self.move_to_table(database, table),
            DashboardMsg::BackToDBList => self.move_to_dblist(),
            DashboardMsg::Leave => DashboardMsg::Leave.into(),
            DashboardMsg::ConnectionSelected(conn) => self.on_connection_selected(conn),
            DashboardMsg::DBListMsg(m) => self.dblist.update(m).map_auto(),
            DashboardMsg::TableMsg(TableMsg::FocusProperties) => {
                // Set focus first
                let _ = self.table.update(TableMsg::FocusProperties);
                if let Some(conn) = &self.connection {
                    DashboardMsg::TableMsg(TableMsg::LoadProperties(conn.clone())).into()
                } else {
                    Update::none()
                }
            }
            DashboardMsg::TableMsg(m) => self.table.update(m).map_auto(),
        }
    }

    fn handle_key(&self, key: KeyEvent) -> Update<Self::Msg> {
        // Forward key to focused component - let update handle side effects
        match self.focus {
            DashboardFocus::DBList => self.dblist.handle_key(key).map_auto(),
            DashboardFocus::Table => self.table.handle_key(key).map_auto(),
        }
    }

    fn draw(&self, f: &mut Frame, area: Rect, focused: bool) {
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
