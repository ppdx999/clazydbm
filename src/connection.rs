use anyhow::Result;
use serde::Deserialize;

use crate::{config::Config, db::DatabaseType};

#[derive(Debug, Deserialize, Clone)]
pub struct Connection {
    pub r#type: DatabaseType,
    pub name: Option<String>,
    pub user: Option<String>,
    pub host: Option<String>,
    pub port: Option<u64>,
    pub path: Option<std::path::PathBuf>,
    pub password: Option<String>,
    pub database: Option<String>,
}

pub fn load_connections() -> Result<Vec<Connection>> {
    let config = Config::new()?;
    Ok(config.conn)
}
