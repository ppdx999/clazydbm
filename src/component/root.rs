use crate::cmd::{MapMsg, Update};
use crate::component::{
    Component, ConnectionComponent, ConnectionMsg, DashboardComponent, DashboardMsg,
};
use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};

pub enum RootMsg {
    Connection(ConnectionMsg),
    Dashboard(DashboardMsg),
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
            RootMsg::Connection(ConnectionMsg::ConnectionSelected(_conn)) => {
                self.focus = Focus::Dashboard;
                Update::none()
            }
            RootMsg::Dashboard(DashboardMsg::Leave) => {
                self.focus = Focus::Connection;
                Update::none()
            }
            RootMsg::Connection(m) => self.connection.update(m).map(RootMsg::Connection),
            RootMsg::Dashboard(m) => self.dashboard.update(m).map(RootMsg::Dashboard),
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
