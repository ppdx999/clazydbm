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
    
    // CLI tool related methods
    fn cli_tool_name() -> &'static str;
    fn is_cli_tool_available() -> bool;
    fn launch_cli_tool(conn: &Connection) -> Result<std::process::ExitStatus>;
}

pub struct DB;

impl DB {
    pub fn cli_tool_name_for(conn: &Connection) -> &'static str {
        match conn.r#type {
            DatabaseType::MySql => Mysql::cli_tool_name(),
            DatabaseType::Postgres => Postgres::cli_tool_name(),
            DatabaseType::Sqlite => Sqlite::cli_tool_name(),
        }
    }
    
    pub fn is_cli_tool_available_for(conn: &Connection) -> bool {
        match conn.r#type {
            DatabaseType::MySql => Mysql::is_cli_tool_available(),
            DatabaseType::Postgres => Postgres::is_cli_tool_available(),
            DatabaseType::Sqlite => Sqlite::is_cli_tool_available(),
        }
    }
    
    pub fn launch_cli_tool_for(conn: &Connection) -> Result<std::process::ExitStatus> {
        match conn.r#type {
            DatabaseType::MySql => Mysql::launch_cli_tool(conn),
            DatabaseType::Postgres => Postgres::launch_cli_tool(conn),
            DatabaseType::Sqlite => Sqlite::launch_cli_tool(conn),
        }
    }
}

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
    
    fn cli_tool_name() -> &'static str {
        unreachable!("Use type-specific implementations")
    }
    
    fn is_cli_tool_available() -> bool {
        unreachable!("Use type-specific implementations")
    }
    
    fn launch_cli_tool(_conn: &Connection) -> Result<std::process::ExitStatus> {
        unreachable!("Use type-specific implementations")
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
