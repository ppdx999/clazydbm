use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::connection::Connection;

const APP_NAME: &str = "clazydbm";
const CONFIG_FILENAME: &str = "config.yaml";

const CONFIG_SAMPLE: &str = r#"conn:
  # MySQL example
  - type: mysql
    name: my-mysql
    user: root
    password: secret
    host: 127.0.0.1
    port: 3306
    database: mydb

  # PostgreSQL example
  - type: postgres
    name: my-postgres
    user: postgres
    password: secret
    host: 127.0.0.1
    port: 5432
    database: mydb

  # SQLite example
  - type: sqlite
    name: my-sqlite
    path: ~/data/sample.db
"#;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub conn: Vec<Connection>,
}

impl Config {
    pub fn new() -> Result<Self> {
        let mut all_connections: Vec<Connection> = Vec::new();

        // 1. Global config: ~/.config/clazydbm/config.yaml
        let global_path = Self::app_config_dir()?.join(CONFIG_FILENAME);
        if let Some(cfg) = Self::load_from_path(&global_path)? {
            all_connections.extend(cfg.conn);
        }

        // 2. Local config: ./.clazydbm.yaml
        let local_path = PathBuf::from(".clazydbm.yaml");
        if let Some(cfg) = Self::load_from_path(&local_path)? {
            all_connections.extend(cfg.conn);
        }

        // 3. Environment variable: CLAZYDBM_CONFIG
        if let Ok(env_path) = std::env::var("CLAZYDBM_CONFIG") {
            let path = PathBuf::from(&env_path);
            if let Some(cfg) = Self::load_from_path(&path)? {
                all_connections.extend(cfg.conn);
            }
        }

        // 4. CLI option (passed via internal env var)
        if let Ok(cli_path) = std::env::var("CLAZYDBM_CONFIG_CLI") {
            let path = PathBuf::from(&cli_path);
            if let Some(cfg) = Self::load_from_path(&path)? {
                all_connections.extend(cfg.conn);
            }
        }

        Ok(Config { conn: all_connections })
    }

    fn load_from_path(path: &Path) -> Result<Option<Config>> {
        if !path.exists() {
            return Ok(None);
        }
        let data = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
        let cfg: Config = serde_yaml::from_slice(&data).map_err(|e| {
            anyhow!(
                "failed to parse YAML at {}\n\nError: {}\n\nExpected format:\n{}",
                path.display(),
                e,
                CONFIG_SAMPLE
            )
        })?;
        Ok(Some(cfg))
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
