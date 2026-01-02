mod app;
mod component;
mod config;
mod connection;
mod db;
mod logger;
mod terminal;
mod update;

use clap::Parser;

use crate::{
    app::run_app,
    logger::{error, init},
    terminal::with_terminal,
};

#[derive(Parser)]
#[command(name = "clazydbm")]
#[command(about = "A modern TUI database management tool")]
struct Cli {
    /// Path to additional config file
    #[arg(short, long)]
    config: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Pass CLI config path via internal environment variable
    if let Some(config_path) = cli.config {
        std::env::set_var("CLAZYDBM_CONFIG_CLI", config_path);
    }

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
