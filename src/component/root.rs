use crate::cmd::{MapMsg, Update};
use crate::component::{Component, ConnectionComponent, ConnectionMsg, DashboardComponent, DashboardMsg};
use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};

pub enum RootMsg {
    Connection(ConnectionMsg),
    Dashboard(DashboardMsg),
}

enum Route {
    Connection,
    Dashboard,
}

pub struct RootComponent {
    route: Route,
    connection: ConnectionComponent,
    dashboard: DashboardComponent,
}

impl RootComponent {
    pub fn new() -> Self {
        Self {
            route: Route::Connection,
            connection: ConnectionComponent::new(),
            dashboard: DashboardComponent::new(),
        }
    }
}

impl Component for RootComponent {
    type Msg = RootMsg;

    fn update(&mut self, msg: Self::Msg) -> Update<Self::Msg> {
        match (&self.route, msg) {
            // Handle child-intent messages that change route
            (Route::Connection, RootMsg::Connection(ConnectionMsg::SelectConnection)) => {
                self.route = Route::Dashboard;
                Update::none()
            }
            (Route::Dashboard, RootMsg::Dashboard(DashboardMsg::Leave)) => {
                self.route = Route::Connection;
                Update::none()
            }
            // Forward other messages to the active child
            (Route::Connection, RootMsg::Connection(m)) => {
                self.connection.update(m).map(RootMsg::Connection)
            }
            (Route::Dashboard, RootMsg::Dashboard(m)) => {
                self.dashboard.update(m).map(RootMsg::Dashboard)
            }
            // Messages for inactive panes are ignored for now
            _ => Update::none(),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Update<Self::Msg> {
        match self.route {
            Route::Connection => self
                .connection
                .handle_key(key)
                .map(RootMsg::Connection),
            Route::Dashboard => self
                .dashboard
                .handle_key(key)
                .map(RootMsg::Dashboard),
        }
    }

    fn draw(&mut self, f: &mut Frame, area: Rect, focused: bool) {
        match self.route {
            Route::Connection => self.connection.draw(f, area, focused),
            Route::Dashboard => self.dashboard.draw(f, area, focused),
        }
    }
}
