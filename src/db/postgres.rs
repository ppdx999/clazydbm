use anyhow::Result;

use crate::component::{Child, Database, Schema, Table};
use crate::{connection::Connection, db::DBBehavior};
use crate::db::Records;
use crate::logger::debug;

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
    fn fetch_databases(conn: &Connection) -> Result<Vec<Database>> {
        debug("postgres: connecting");
        let url = Postgres::database_url(conn)?;
        let mut client = postgres::Client::connect(&url, postgres::NoTls)?;
        debug("postgres: connected");

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
            by_schema.entry(schema.clone()).or_default().push(Table {
                name: table,
                engine: None,
                schema: Some(schema),
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

    fn fetch_records(
        conn: &Connection,
        _database: &str,
        table: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Records> {
        // columns
        let url = Postgres::database_url(conn)?;
        let mut client = postgres::Client::connect(&url, postgres::NoTls)?;
        let cols_rows = client.query(
            "SELECT column_name FROM information_schema.columns WHERE table_name = $1 ORDER BY ordinal_position",
            &[&table],
        )?;
        let columns: Vec<String> = cols_rows.into_iter().map(|r| r.get::<_, String>(0)).collect();

        // rows as JSON strings per row for broad type coverage (placeholder)
        let q = format!("SELECT to_jsonb(t.*)::text FROM {} t LIMIT $1 OFFSET $2", table);
        let rows = client.query(&q, &[&(limit as i64), &(offset as i64)])?;
        let mut rows_vec = Vec::new();
        for r in rows {
            let j: String = r.get(0);
            rows_vec.push(vec![j]);
        }

        let columns = if columns.is_empty() { vec!["json".to_string()] } else { columns };
        Ok(Records { columns, rows: rows_vec })
    }
}
