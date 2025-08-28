use std::path::{Path, PathBuf};

use anyhow::Result;
use async_trait::async_trait;

use crate::{component::Database, connection::Connection, db::DBBehavior};

pub struct Sqlite {}

#[async_trait]
impl DBBehavior for Sqlite {
    fn database_url(conn: &Connection) -> Result<String> {
        let path = conn.path.as_ref().map_or(
            Err(anyhow::anyhow!("type sqlite needs the path field")),
            |path| expand_path(path).ok_or_else(|| anyhow::anyhow!("cannot expand file path")),
        )?;

        Ok(format!("sqlite://{path}", path = path.to_str().unwrap()))
    }
    async fn get_databases(&self) -> Result<Vec<Database>> {
        // Unused for now; DBList fetch is routed via free function below
        Ok(vec![])
    }
}

fn expand_path(path: &Path) -> Option<PathBuf> {
    let mut expanded_path = PathBuf::new();
    let mut path_iter = path.iter();
    if path.starts_with("~") {
        path_iter.next()?;
        expanded_path = expanded_path.join(dirs_next::home_dir()?);
    }
    for path in path_iter {
        let path = path.to_str()?;
        expanded_path = if cfg!(unix) && path.starts_with('$') {
            expanded_path.join(std::env::var(path.strip_prefix('$')?).unwrap_or_default())
        } else if cfg!(windows) && path.starts_with('%') && path.ends_with('%') {
            expanded_path
                .join(std::env::var(path.strip_prefix('%')?.strip_suffix('%')?).unwrap_or_default())
        } else {
            expanded_path.join(path)
        }
    }
    Some(expanded_path)
}

/// Placeholder: implement actual SQLite fetching here
pub fn fetch_databases(_conn: &Connection) -> Result<Vec<Database>> {
    Ok(vec![])
}
