use std::sync::mpsc::Sender;

use crate::app::AppMsg;

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
}

pub trait MapMsg<M> {
    fn map<ParentMsg>(self, wrap: impl FnOnce(M) -> ParentMsg) -> Update<ParentMsg>;
}

impl<M> MapMsg<M> for Update<M> {
    fn map<ParentMsg>(self, wrap: impl FnOnce(M) -> ParentMsg) -> Update<ParentMsg> {
        Update {
            msg: self.msg.map(wrap),
            cmd: self.cmd,
        }
    }
}
