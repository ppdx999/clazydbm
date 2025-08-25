use crate::cmd::Command;
use crate::cmd::MapMsg;
use crate::component::{Component, RootComponent, RootMsg};
use crossterm::event::KeyModifiers;
use crossterm::event::{self, Event, KeyCode};
use ratatui::Terminal;
use ratatui::prelude::Backend;
use std::io::Result;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::time::Duration;

pub enum AppMsg {
    Quit,
    Root(RootMsg),
}

pub struct App<B: Backend> {
    term: Terminal<B>,
    root: RootComponent,
    rx: Receiver<AppMsg>,
    tx: Sender<AppMsg>,
    should_quit: bool,
}

impl<B: Backend> App<B> {
    pub fn new(term: Terminal<B>) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            term,
            root: RootComponent::new(),
            rx,
            tx,
            should_quit: false,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        while !self.should_quit {
            self.handle_messages();
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

        match event::read()? {
            Event::Key(key)
                if key.code == KeyCode::Char('c')
                    && key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.should_quit = true;
            }

            Event::Key(key) => {
                let cmd = self.root.handle_key(key).map(AppMsg::Root).cmd;
                self.run_command(cmd);
            }

            _ => {}
        }

        Ok(())
    }

    fn handle_messages(&mut self) {
        while let Ok(msg) = self.rx.try_recv() {
            if let Some(cmd) = self.apply(msg) {
                self.run_command(cmd);
            }
        }
    }

    fn apply(&mut self, msg: AppMsg) -> Option<Command> {
        match msg {
            AppMsg::Quit => {
                self.should_quit = true;
                None
            }
            AppMsg::Root(m) => self.root.update(m).map(AppMsg::Root).cmd.into(),
        }
    }

    fn run_command(&self, cmd: Command) {
        match cmd {
            Command::None => {}
            Command::Batch(list) => {
                for c in list {
                    self.run_command(c)
                }
            }
            Command::Spawn(task) => task(self.tx.clone()),
        }
    }
}

pub fn run_app<B: Backend>(terminal: Terminal<B>) -> Result<()> {
    let mut app = App::new(terminal);
    app.run()
}
