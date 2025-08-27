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
}

impl Component for RootComponent {
    type Msg = RootMsg;

    fn update(&mut self, msg: Self::Msg) -> Update<Self::Msg> {
        match msg {
            RootMsg::ConnectionSelected(_conn) => {
                self.focus = Focus::Dashboard;
                Update::none()
            }
            RootMsg::LeaveDashboard => {
                self.focus = Focus::Connection;
                Update::none()
            }
            RootMsg::Connection(m) => self.connection.update(m).map(RootMsg::from),
            RootMsg::Dashboard(m) => self.dashboard.update(m).map(RootMsg::from),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Update<Self::Msg> {
        match self.focus {
            Focus::Connection => self.connection.handle_key(key).map(RootMsg::Connection),
            Focus::Dashboard => self.dashboard.handle_key(key).map(RootMsg::Dashboard),
        }
    }

    fn draw(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        match self.focus {
            Focus::Connection => self.connection.draw(f, area, focused),
            Focus::Dashboard => self.dashboard.draw(f, area, focused),
        }
    }
}
