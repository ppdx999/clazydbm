use anyhow::Result;
use async_trait::async_trait;

use crate::component::{Child, Database, Table};
use crate::{connection::Connection, db::DBBehavior};

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

/// Fetch MySQL databases and tables
pub fn fetch_databases(conn: &Connection) -> Result<Vec<Database>> {
    use mysql::prelude::*;
    use mysql::params;

    let url = Mysql::database_url(conn)?;
    let opts = mysql::Opts::from_url(&url)?;
    let mut c = mysql::Conn::new(opts)?;

    // Determine database list
    let dbs: Vec<String> = match conn.database.as_ref() {
        Some(db) => vec![db.clone()],
        None => c.query::<String, _>("SHOW DATABASES")?,
    };

    // For each database, list tables via information_schema
    let mut out = Vec::new();
    for dbname in dbs {
        // Skip internal schemas
        if dbname == "information_schema" || dbname == "mysql" || dbname == "performance_schema" || dbname == "sys" {
            continue;
        }

        let q = r#"
            SELECT TABLE_NAME, ENGINE
            FROM information_schema.TABLES
            WHERE TABLE_SCHEMA = :schema
            ORDER BY TABLE_NAME
        "#;
        let rows: Vec<(String, Option<String>)> = c.exec(q, params! { "schema" => &dbname })?;

        let children = rows
            .into_iter()
            .map(|(name, engine)| {
                let t = Table { name, engine, schema: None };
                Child::Table(t)
            })
            .collect();

        out.push(Database::new(dbname, children));
    }

    Ok(out)
}
