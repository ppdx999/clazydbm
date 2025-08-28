use crate::cmd::{MapMsg, Update};
use crate::component::{
    Component, ConnectionComponent, ConnectionMsg, DashboardComponent, DashboardMsg,
};
use crate::connection::Connection;
use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};

pub enum RootMsg {
    ConnectionSelected(Connection),
    LeaveDashboard,
    Connection(ConnectionMsg),
    Dashboard(DashboardMsg),
}

impl From<ConnectionMsg> for RootMsg {
    fn from(msg: ConnectionMsg) -> Self {
        match msg {
            ConnectionMsg::ConnectionSelected(conn) => RootMsg::ConnectionSelected(conn),
            m => RootMsg::Connection(m),
        }
    }
}
impl From<DashboardMsg> for RootMsg {
    fn from(msg: DashboardMsg) -> Self {
        match msg {
            DashboardMsg::Leave => RootMsg::LeaveDashboard,
            m => RootMsg::Dashboard(m),
        }
    }
}

enum Focus {
    Connection,
    Dashboard,
}

pub struct RootComponent {
    focus: Focus,
    connection: ConnectionComponent,
    dashboard: DashboardComponent,
}

impl RootComponent {
    pub fn new() -> Self {
        Self {
            focus: Focus::Connection,
            connection: ConnectionComponent::new(),
            dashboard: DashboardComponent::new(),
        }
    }
    fn move_to_dashboard(&mut self, conn: Connection) -> Update<RootMsg> {
        // Store selected connection and trigger DBList load immediately
        self.focus = Focus::Dashboard;
        self.dashboard
            .update(DashboardMsg::ConnectionSelected(conn))
            .map_auto()
    }
    fn move_to_connection(&mut self) -> Update<RootMsg> {
        self.focus = Focus::Connection;
        Update::none()
    }
}

impl Component for RootComponent {
    type Msg = RootMsg;

    fn update(&mut self, msg: Self::Msg) -> Update<Self::Msg> {
        match msg {
            RootMsg::ConnectionSelected(conn) => self.move_to_dashboard(conn),
            RootMsg::LeaveDashboard => self.move_to_connection(),
            RootMsg::Connection(m) => self.connection.update(m).map_auto(),
            RootMsg::Dashboard(m) => self.dashboard.update(m).map_auto(),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Update<Self::Msg> {
        match self.focus {
            Focus::Connection => self.connection.handle_key(key).map_auto(),
            Focus::Dashboard => self.dashboard.handle_key(key).map_auto(),
        }
    }

    fn draw(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        match self.focus {
            Focus::Connection => self.connection.draw(f, area, focused),
            Focus::Dashboard => self.dashboard.draw(f, area, focused),
        }
    }
}
