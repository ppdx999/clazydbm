use anyhow::Result;
use async_trait::async_trait;

use crate::{component::Database, connection::Connection, db::DBBehavior};

pub struct Mysql {}

#[async_trait]
impl DBBehavior for Mysql {
    fn database_url(conn: &Connection) -> Result<String> {
        let user = conn
            .user
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("type mysql needs the user field"))?;
        let host = conn
            .host
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("type mysql needs the host field"))?;
        let port = conn
            .port
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("type mysql needs the port field"))?;
        let password = conn
            .password
            .as_ref()
            .map_or(String::new(), |p| p.to_string());

        match conn.database.as_ref() {
            Some(database) => Ok(format!(
                "mysql://{user}:{password}@{host}:{port}/{database}",
                user = user,
                password = password,
                host = host,
                port = port,
                database = database
            )),
            None => Ok(format!(
                "mysql://{user}:{password}@{host}:{port}",
                user = user,
                password = password,
                host = host,
                port = port,
            )),
        }
    }
    async fn get_databases(&self) -> Result<Vec<Database>> {
        // Unused for now; DBList fetch is routed via free function below
        Ok(vec![])
    }
}

/// Placeholder: implement actual MySQL fetching here
pub fn fetch_databases(_conn: &Connection) -> Result<Vec<Database>> {
    Ok(vec![])
}
