use crossterm::event::KeyEvent;
use ratatui::{Frame, layout::Rect};

use crate::update::Update;

mod connection;
mod dashboard;
mod dblist;
mod root;
mod table;

pub use connection::{ConnectionComponent, ConnectionMsg};
pub use dashboard::{DashboardComponent, DashboardMsg};
pub use dblist::{Child, DBListComponent, DBListMsg, Database, Schema, Table};
pub use root::{RootComponent, RootMsg};
pub use table::{TableComponent, TableMsg};

pub trait Component {
    type Msg;

    /// Pure update (no IO). Return bubbled message (optional) + Command.
    fn update(&mut self, msg: Self::Msg) -> Update<Self::Msg>;

    /// Handle raw input only if focused; otherwise ignore or implement soft reactions later.
    fn handle_key(&self, key: KeyEvent) -> Update<Self::Msg>;

    /// Draw is side-effectful but only touches the frame.
    fn draw(&self, f: &mut Frame, area: Rect, focused: bool);
}
