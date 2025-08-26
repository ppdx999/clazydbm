use crossterm::event::KeyEvent;
use ratatui::{layout::Rect, Frame};

use crate::cmd::Update;

mod root;
mod connection;
mod dashboard;

pub use connection::{ConnectionComponent, ConnectionMsg};
pub use dashboard::{DashboardComponent, DashboardMsg};
pub use root::{RootComponent, RootMsg};

pub trait Component {
    type Msg;

    /// Pure update (no IO). Return bubbled message (optional) + Command.
    fn update(&mut self, msg: Self::Msg) -> Update<Self::Msg>;

    /// Handle raw input only if focused; otherwise ignore or implement soft reactions later.
    fn handle_key(&mut self, key: KeyEvent) -> Update<Self::Msg>;

    /// Draw is side-effectful but only touches the frame.
    fn draw(&mut self, f: &mut Frame, area: Rect, focused: bool);
}
