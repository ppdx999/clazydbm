use anyhow::Result;
use async_trait::async_trait;

use crate::component::{Child, Database, Schema, Table};
use crate::{connection::Connection, db::DBBehavior};

pub struct Postgres {}

#[async_trait]
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
    async fn get_databases(&self) -> Result<Vec<Database>> {
        // Unused for now; DBList fetch is routed via free function below
        Ok(vec![])
    }
}

/// Fetch Postgres schemas and tables for the connected database
pub fn fetch_databases(conn: &Connection) -> Result<Vec<Database>> {
    let url = Postgres::database_url(conn)?;
    let mut client = postgres::Client::connect(&url, postgres::NoTls)?;

    // Collect schema -> tables
    let rows = client.query(
        "SELECT table_schema, table_name
         FROM information_schema.tables
         WHERE table_type = 'BASE TABLE'
           AND table_schema NOT IN ('pg_catalog','information_schema')
         ORDER BY table_schema, table_name",
        &[],
    )?;

    use std::collections::BTreeMap;
    let mut by_schema: BTreeMap<String, Vec<Table>> = BTreeMap::new();
    for row in rows {
        let schema: String = row.get(0);
        let table: String = row.get(1);
        by_schema.entry(schema).or_default().push(Table {
            name: table,
            engine: None,
            schema: None,
        });
    }

    // Database name from connection
    let dbname = conn
        .database
        .clone()
        .unwrap_or_else(|| "postgres".to_string());

    let mut children = Vec::new();
    for (schema, tables) in by_schema {
        children.push(Child::Schema(Schema { name: schema, tables }));
    }

    Ok(vec![Database::new(dbname, children)])
}
