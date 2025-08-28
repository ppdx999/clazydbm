#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "postgres")]
mod postgres;
mod sqlite;

use crate::{component::Database, connection::Connection};
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;

#[cfg(feature = "mysql")]
pub use mysql::Mysql;
#[cfg(feature = "postgres")]
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
            DatabaseType::MySql => {
                #[cfg(feature = "mysql")]
                {
                    Mysql::database_url(conn)
                }
                #[cfg(not(feature = "mysql"))]
                {
                    Err(anyhow::anyhow!("mysql feature disabled"))
                }
            }
            DatabaseType::Postgres => {
                #[cfg(feature = "postgres")]
                {
                    Postgres::database_url(conn)
                }
                #[cfg(not(feature = "postgres"))]
                {
                    Err(anyhow::anyhow!("postgres feature disabled"))
                }
            }
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
        DatabaseType::MySql => {
            #[cfg(feature = "mysql")]
            {
                mysql::fetch_databases(conn)
            }
            #[cfg(not(feature = "mysql"))]
            {
                Err(anyhow::anyhow!("mysql feature disabled"))
            }
        }
        DatabaseType::Postgres => {
            #[cfg(feature = "postgres")]
            {
                postgres::fetch_databases(conn)
            }
            #[cfg(not(feature = "postgres"))]
            {
                Err(anyhow::anyhow!("postgres feature disabled"))
            }
        }
        DatabaseType::Sqlite => sqlite::fetch_databases(conn),
    }
}
