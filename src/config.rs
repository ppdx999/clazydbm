use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const APP_NAME: &str = "clazydbm";
const CONNECTIONS_FILE: &str = "connections.yaml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub name: String,
    pub dsn: String,
}

/// Return the application config directory path, creating it if missing.
pub fn get_app_config_path() -> Result<PathBuf> {
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
    Ok(get_app_config_path()?.join(CONNECTIONS_FILE))
}

/// Load connections configuration. Returns empty list if file does not exist.
pub fn load_connections() -> Result<Vec<ConnectionConfig>> {
    let path = connections_path()?;
    if !Path::new(&path).exists() {
        return Ok(Vec::new());
    }
    let data = fs::read(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let list: Vec<ConnectionConfig> = serde_yaml::from_slice(&data)
        .with_context(|| format!("failed to parse YAML at {}", path.display()))?;
    Ok(list)
}

