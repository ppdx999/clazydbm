mod app;
mod update;
mod component;
mod config;
mod connection;
mod db;
mod logger;
mod terminal;

use crate::{app::run_app, logger::{error, init}, terminal::with_terminal};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize file logging under the app config directory
    if let Ok(dir) = crate::config::Config::app_config_dir() {
        let log_path = dir.join("clazydbm.log");
        let _ = init(log_path);
    }

    let result = with_terminal(run_app);

    if let Err(err) = result {
        println!("{:?}", err);
        error(&format!("fatal error: {:?}", err));
    }

    Ok(())
}
