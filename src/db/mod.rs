mod mysql;
mod postgres;
mod sqlite;

use anyhow::Result;
use serde::Deserialize;

use crate::connection::Connection;

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

pub trait DBBehavior {
    fn database_url(conn: &Connection) -> Result<String>;
}

pub struct DB {}
impl DBBehavior for DB {
    fn database_url(conn: &Connection) -> Result<String> {
        match conn.r#type {
            DatabaseType::MySql => Mysql::database_url(conn),
            DatabaseType::Postgres => Postgres::database_url(conn),
            DatabaseType::Sqlite => Sqlite::database_url(conn),
        }
    }
}
