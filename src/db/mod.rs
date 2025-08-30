mod mysql;
mod postgres;
mod sqlite;

use crate::{component::Database, connection::Connection};
use anyhow::Result;
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

pub trait DBBehavior: Send + Sync {
    fn database_url(conn: &Connection) -> Result<String>;
    fn fetch_databases(conn: &Connection) -> Result<Vec<Database>>;
    fn fetch_records(
        conn: &Connection,
        database: &str,
        table: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Records>;
    fn fetch_properties(
        conn: &Connection,
        database: &str,
        table: &str,
    ) -> Result<TableProperties>;
}

pub struct DB;

impl DBBehavior for DB {
    fn database_url(conn: &Connection) -> Result<String> {
        match conn.r#type {
            DatabaseType::MySql => Mysql::database_url(conn),
            DatabaseType::Postgres => Postgres::database_url(conn),
            DatabaseType::Sqlite => Sqlite::database_url(conn),
        }
    }
    fn fetch_databases(conn: &Connection) -> Result<Vec<Database>> {
        match conn.r#type {
            DatabaseType::MySql => Mysql::fetch_databases(conn),
            DatabaseType::Postgres => Postgres::fetch_databases(conn),
            DatabaseType::Sqlite => Sqlite::fetch_databases(conn),
        }
    }
    fn fetch_records(
        conn: &Connection,
        database: &str,
        table: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Records> {
        match conn.r#type {
            DatabaseType::MySql => Mysql::fetch_records(conn, database, table, limit, offset),
            DatabaseType::Postgres => Postgres::fetch_records(conn, database, table, limit, offset),
            DatabaseType::Sqlite => Sqlite::fetch_records(conn, database, table, limit, offset),
        }
    }
    fn fetch_properties(
        conn: &Connection,
        database: &str,
        table: &str,
    ) -> Result<TableProperties> {
        match conn.r#type {
            DatabaseType::MySql => Mysql::fetch_properties(conn, database, table),
            DatabaseType::Postgres => Postgres::fetch_properties(conn, database, table),
            DatabaseType::Sqlite => Sqlite::fetch_properties(conn, database, table),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Records {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>, // each inner Vec is a row of stringified values
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default: Option<String>,
    pub primary_key: bool,
}

#[derive(Debug, Clone)]
pub struct TableProperties {
    pub columns: Vec<ColumnInfo>,
}

// end
