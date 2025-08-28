mod mysql;
mod postgres;
mod sqlite;

use crate::{component::Database, connection::Connection};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;

pub use mysql::Mysql;
pub use postgres::Postgres;
pub use sqlite::Sqlite;

#[derive(Debug, Deserialize, Clone)]
pub enum DatabaseType {
    #[serde(rename = "mysql")]
    MySql,
    #[serde(rename = "postgres")]
    Postgres,
    #[serde(rename = "sqlite")]
    Sqlite,
}

#[async_trait]
pub trait DBBehavior: Send + Sync {
    fn database_url(conn: &Connection) -> Result<String>;
    async fn get_databases(&self) -> Result<Vec<Database>>;
}

pub struct DB;

#[async_trait]
impl DBBehavior for DB {
    fn database_url(conn: &Connection) -> Result<String> {
        match conn.r#type {
            DatabaseType::MySql => Mysql::database_url(conn),
            DatabaseType::Postgres => Postgres::database_url(conn),
            DatabaseType::Sqlite => Sqlite::database_url(conn),
        }
    }
    async fn get_databases(&self) -> Result<Vec<Database>> {
        Ok(vec![])
    }
}

/// Fetch databases/schemas/tables for the given connection.
/// Each backend implements its own logic in its module.
pub fn fetch_databases(conn: &Connection) -> Result<Vec<Database>> {
    match conn.r#type {
        DatabaseType::MySql => mysql::fetch_databases(conn),
        DatabaseType::Postgres => postgres::fetch_databases(conn),
        DatabaseType::Sqlite => sqlite::fetch_databases(conn),
    }
}
