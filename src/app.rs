use anyhow::Result;
use crate::component::ConnectionMsg;
use crate::component::{Component, DBListMsg, DashboardMsg, RootComponent, RootMsg, TableMsg};
use crate::terminal::Terminal;
use crate::update::Command;
use crate::update::MapMsg;
use crate::update::Update;
use crossterm::event::KeyModifiers;
use crossterm::event::{self, Event, KeyCode};
use ratatui::prelude::Backend;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::time::Duration;

#[allow(dead_code)]
pub enum AppMsg {
    Quit,
    Root(RootMsg),
}

impl From<RootMsg> for AppMsg {
    fn from(msg: RootMsg) -> Self {
        AppMsg::Root(msg)
    }
}

// Convenience: allow bubbling lower-level messages directly into AppMsg
impl From<DashboardMsg> for AppMsg {
    fn from(msg: DashboardMsg) -> Self {
        AppMsg::from(RootMsg::from(msg))
    }
}
impl From<DBListMsg> for AppMsg {
    fn from(msg: DBListMsg) -> Self {
        AppMsg::from(DashboardMsg::from(msg))
    }
}
impl From<TableMsg> for AppMsg {
    fn from(msg: TableMsg) -> Self {
        AppMsg::from(DashboardMsg::from(msg))
    }
}
impl From<ConnectionMsg> for AppMsg {
    fn from(msg: ConnectionMsg) -> Self {
        AppMsg::from(RootMsg::from(msg))
    }
}

pub struct App<B: Backend> {
    term: Terminal<B>,
    root: RootComponent,
    rx: Receiver<AppMsg>,
    tx: Sender<AppMsg>,
    should_quit: bool,
}

impl<B: Backend> App<B> {
    pub fn new(term: Terminal<B>) -> Result<Self> {
        let (tx, rx) = std::sync::mpsc::channel();
        Ok(Self {
            term,
            root: RootComponent::new()?,
            rx,
            tx,
            should_quit: false,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        while !self.should_quit {
            self.handle_async_messages();
            self.draw()?;
            self.handle_event()?;
        }
        Ok(())
    }


    fn draw(&mut self) -> Result<()> {
        self.term.draw(|f| {
            self.root.draw(f, f.size(), true);
        })?;
        Ok(())
    }

    fn handle_event(&mut self) -> Result<()> {
        if !event::poll(Duration::from_millis(250))? {
            return Ok(());
        }

        let Event::Key(key) = event::read()? else {
            return Ok(());
        };

        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
            return Ok(());
        }

        let update = self.root.handle_key(key).map_auto();
        self.handle_update(update);

        Ok(())
    }

    fn handle_update(&mut self, update: Update<AppMsg>) {
        if let Some(msg) = update.msg {
            let u = self.handle_msg(msg);
            self.handle_update(u);
        }
        self.run_command(update.cmd);
    }

    fn handle_msg(&mut self, msg: AppMsg) -> Update<AppMsg> {
        match msg {
            AppMsg::Quit => {
                self.should_quit = true;
                Update::none()
            }
            AppMsg::Root(m) => self.root.update(m).map_auto(),
        }
    }

    fn handle_async_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            self.handle_update(Update::msg(msg));
        }
    }

    fn run_command(&mut self, cmd: Command) {
        match cmd {
            Command::None => {}
            Command::Batch(list) => {
                for c in list {
                    self.run_command(c)
                }
            }
            Command::Spawn(task) => {
                let tx = self.tx.clone();
                std::thread::spawn(move || {
                    task(tx);
                });
            }
            Command::SuspendTerminal(task) => {
                if let Err(e) = self.term.with_suspended(|| {
                    task().map_err(|e| -> Box<dyn std::error::Error> { e })
                }) {
                    crate::logger::error(&format!("Terminal suspend error: {}", e));
                }
            }
        }
    }
}

pub fn run_app<B: Backend>(terminal: Terminal<B>) -> Result<()> {
    let mut app = App::new(terminal)?;
    app.run()
}
