use std::sync::mpsc::Sender;

use crate::app::AppMsg;
use crate::component::{ConnectionMsg, DashboardMsg, DBListMsg, RootMsg, TableMsg};

pub enum Command {
    None,
    Batch(Vec<Command>),
    Spawn(Box<dyn FnOnce(Sender<AppMsg>) + Send>), // runs async and posts AppMsg
}

impl Command {
    pub fn none() -> Self {
        Command::None
    }
    pub fn batch(cmds: impl IntoIterator<Item = Command>) -> Self {
        let v = cmds
            .into_iter()
            .filter(|c| !matches!(c, Command::None))
            .collect::<Vec<_>>();
        if v.is_empty() {
            Command::None
        } else {
            Command::Batch(v)
        }
    }
}

pub struct Update<T> {
    pub msg: Option<T>,
    pub cmd: Command,
}

impl<T> Update<T> {
    pub fn none() -> Self {
        Self {
            msg: None,
            cmd: Command::None,
        }
    }
    pub fn cmd(cmd: Command) -> Self {
        Self { msg: None, cmd }
    }
    pub fn msg(msg: T) -> Self {
        Self {
            msg: Some(msg),
            cmd: Command::None,
        }
    }
    pub fn with_cmd(cmd: Command) -> Self {
        Self { msg: None, cmd }
    }
    pub fn msg_cmd(msg: T, cmd: Command) -> Self {
        Self {
            msg: Some(msg),
            cmd,
        }
    }
}

impl<M> From<()> for Update<M> {
    fn from(_: ()) -> Self {
        Update::none()
    }
}

impl<M> From<Command> for Update<M> {
    fn from(cmd: Command) -> Self {
        Update::with_cmd(cmd)
    }
}

// Message-specific From implementations to allow `msg.into()` ergonomics
impl From<DBListMsg> for Update<DBListMsg> {
    fn from(msg: DBListMsg) -> Self {
        Update::msg(msg)
    }
}
impl From<TableMsg> for Update<TableMsg> {
    fn from(msg: TableMsg) -> Self {
        Update::msg(msg)
    }
}
impl From<DashboardMsg> for Update<DashboardMsg> {
    fn from(msg: DashboardMsg) -> Self {
        Update::msg(msg)
    }
}
impl From<ConnectionMsg> for Update<ConnectionMsg> {
    fn from(msg: ConnectionMsg) -> Self {
        Update::msg(msg)
    }
}
impl From<RootMsg> for Update<RootMsg> {
    fn from(msg: RootMsg) -> Self {
        Update::msg(msg)
    }
}

pub trait MapMsg<M> {
    fn map<ParentMsg>(self, wrap: impl FnOnce(M) -> ParentMsg) -> Update<ParentMsg>;
    fn map_auto<ParentMsg>(self) -> Update<ParentMsg>
    where
        ParentMsg: From<M>;
}

impl<M> MapMsg<M> for Update<M> {
    fn map<ParentMsg>(self, wrap: impl FnOnce(M) -> ParentMsg) -> Update<ParentMsg> {
        Update {
            msg: self.msg.map(wrap),
            cmd: self.cmd,
        }
    }

    fn map_auto<ParentMsg>(self) -> Update<ParentMsg>
    where
        ParentMsg: From<M>,
    {
        Update {
            msg: self.msg.map(ParentMsg::from),
            cmd: self.cmd,
        }
    }
}

