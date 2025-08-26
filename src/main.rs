mod app;
mod cmd;
mod component;
mod config;
mod terminal;

use crate::{app::run_app, terminal::with_terminal};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = with_terminal(run_app);

    if let Err(err) = result {
        println!("{:?}", err);
    }

    Ok(())
}
