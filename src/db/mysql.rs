use anyhow::Result;

use crate::component::{Child, Database, Table};
use crate::{connection::Connection, db::DBBehavior};
use crate::db::{Records, ColumnInfo, TableProperties};
use crate::logger::debug;
use std::process::Command;

pub struct Mysql {}

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
    fn fetch_databases(conn: &Connection) -> Result<Vec<Database>> {
        debug("mysql: connecting");
        use mysql::prelude::*;
        use mysql::params;

        let url = Mysql::database_url(conn)?;
        let opts = mysql::Opts::from_url(&url)?;
        let mut c = mysql::Conn::new(opts)?;
        debug("mysql: connected");

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

    fn fetch_records(
        conn: &Connection,
        database: &str,
        table: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Records> {
        use mysql::prelude::*;
        use mysql::{params, Value};
        let url = Mysql::database_url(conn)?;
        let opts = mysql::Opts::from_url(&url)?;
        let mut c = mysql::Conn::new(opts)?;

        // columns
        let cols_q = r#"SELECT COLUMN_NAME FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = :schema AND TABLE_NAME = :table ORDER BY ORDINAL_POSITION"#;
        let columns: Vec<String> = c.exec(cols_q, params! { "schema" => database, "table" => table })?;

        // rows
        let q = format!("SELECT * FROM `{}`.`{}` LIMIT {} OFFSET {}", database, table, limit, offset);
        let result = c.query_iter(q)?;
        let mut rows_vec = Vec::new();
        for row in result {
            let row: mysql::Row = row?;
            let mut out = Vec::new();
            for v in row.unwrap() {
                let s = match v {
                    Value::NULL => String::new(),
                    Value::Bytes(b) => String::from_utf8_lossy(&b).into_owned(),
                    Value::Int(i) => i.to_string(),
                    Value::UInt(u) => u.to_string(),
                    Value::Float(f) => f.to_string(),
                    Value::Double(d) => d.to_string(),
                    Value::Date(y,m,d,h,mi,s, _us) => format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y,m,d,h,mi,s),
                    Value::Time(neg, d, h, mi, s, _us) => {
                        let hours = d * 24 + u32::from(h);
                        format!("{}{:02}:{:02}:{:02}", if neg {"-"} else {""}, hours, mi, s)
                    }
                };
                out.push(s);
            }
            rows_vec.push(out);
        }

        Ok(Records { columns, rows: rows_vec })
    }

    fn fetch_properties(
        conn: &Connection,
        database: &str,
        table: &str,
    ) -> Result<TableProperties> {
        use mysql::prelude::*;
        use mysql::params;
        let url = Mysql::database_url(conn)?;
        let opts = mysql::Opts::from_url(&url)?;
        let mut c = mysql::Conn::new(opts)?;

        let q = r#"
            SELECT COLUMN_NAME, COLUMN_TYPE, IS_NULLABLE, COLUMN_DEFAULT, COLUMN_KEY
            FROM information_schema.COLUMNS
            WHERE TABLE_SCHEMA = :schema AND TABLE_NAME = :table
            ORDER BY ORDINAL_POSITION
        "#;
        let rows: Vec<(String, String, String, Option<String>, Option<String>)> =
            c.exec(q, params! { "schema" => database, "table" => table })?;
        let columns = rows
            .into_iter()
            .map(|(name, coltype, is_nullable, default, colkey)| ColumnInfo {
                name,
                data_type: coltype,
                nullable: is_nullable.eq_ignore_ascii_case("YES"),
                default,
                primary_key: colkey.as_deref() == Some("PRI"),
            })
            .collect();
        Ok(TableProperties { columns })
    }
    
    fn cli_tool_name() -> &'static str {
        "mycli"
    }
    
    fn is_cli_tool_available() -> bool {
        Command::new("which")
            .arg("mycli")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
    
    fn launch_cli_tool(conn: &Connection) -> Result<std::process::ExitStatus> {
        let db_url = Self::database_url(conn)?;
        debug(&format!("Launching mycli with URL: {}", db_url));
        
        Command::new("mycli")
            .arg(&db_url)
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to launch mycli: {}", e))
    }
}
