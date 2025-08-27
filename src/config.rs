use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::connection::Connection;

const APP_NAME: &str = "clazydbm";
const CONNECTIONS_FILE: &str = "config.yaml";

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub conn: Vec<Connection>,
}

impl Config {
    pub fn new() -> Result<Self> {
        let path = Self::connections_path()?;
        Self::create_config(&path)
    }

    fn create_config(path: &Path) -> Result<Self> {
        if Path::new(&path).exists() {
            Self::create_config_from_path(&path)
                .with_context(|| format!("failed to create config at {}", path.display()))
        } else {
            Self::create_default_config()
                .try_into()
                .with_context(|| "failed to create default config")
        }
    }
    fn create_default_config() -> Self {
        Config { conn: Vec::new() }
    }

    fn create_config_from_path(path: &Path) -> Result<Self> {
        let data = fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?;
        let cfg: Config = serde_yaml::from_slice(&data)
            .with_context(|| format!("failed to parse YAML at {}", path.display()))?;
        Ok(cfg)
    }

    fn get_app_config_path() -> Result<PathBuf> {
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

    fn connections_path() -> Result<PathBuf> {
        Ok(Self::get_app_config_path()?.join(CONNECTIONS_FILE))
    }

    pub fn load_connections() -> Result<Vec<Connection>> {
        let cfg = Self::new()?;
        Ok(cfg.conn)
    }
}
