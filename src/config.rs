use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::connection::Connection;

const APP_NAME: &str = "clazydbm";
const CONFIG_FILENAME: &str = "config.yaml";

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub conn: Vec<Connection>,
}

impl Config {
    pub fn new() -> Result<Self> {
        let path = Self::app_config_dir()?.join(CONFIG_FILENAME);
        if Path::new(&path).exists() {
            let data = fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?;
            let cfg: Config = serde_yaml::from_slice(&data)
                .with_context(|| format!("failed to parse YAML at {}", path.display()))?;
            Ok(cfg)
        } else {
            Config { conn: Vec::new() }
                .try_into()
                .with_context(|| "failed to create default config")
        }
    }

    /// Public accessor for the per-user app config directory.
    /// Used by other subsystems (e.g. logging) to store runtime files.
    pub fn app_config_dir() -> Result<PathBuf> {
        let mut path = if cfg!(target_os = "macos") {
            dirs_next::home_dir().map(|h| h.join(".config"))
        } else {
            dirs_next::config_dir()
        }
        .ok_or_else(|| anyhow::anyhow!("failed to find os config dir."))?;

        path.push(APP_NAME);
        fs::create_dir_all(&path)?;
        Ok(path)
    }
}
