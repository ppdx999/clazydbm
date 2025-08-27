use anyhow::Result;

use crate::{connection::Connection, db::DBBehavior};

pub struct Postgres {}

impl DBBehavior for Postgres {
    fn database_url(conn: &Connection) -> Result<String> {
        let user = conn
            .user
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("type postgres needs the user field"))?;
        let host = conn
            .host
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("type postgres needs the host field"))?;
        let port = conn
            .port
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("type postgres needs the port field"))?;
        let password = conn
            .password
            .as_ref()
            .map_or(String::new(), |p| p.to_string());

        match conn.database.as_ref() {
            Some(database) => Ok(format!(
                "postgres://{user}:{password}@{host}:{port}/{database}",
                user = user,
                password = password,
                host = host,
                port = port,
                database = database
            )),
            None => Ok(format!(
                "postgres://{user}:{password}@{host}:{port}",
                user = user,
                password = password,
                host = host,
                port = port,
            )),
        }
    }
}
