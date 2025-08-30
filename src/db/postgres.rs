use anyhow::Result;

use crate::component::{Child, Database, Schema, Table};
use crate::{connection::Connection, db::DBBehavior};
use crate::db::{Records, ColumnInfo, TableProperties};
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

        // Build SELECT casting each column to text for consistent string output
        let select_list = if columns.is_empty() {
            "*".to_string()
        } else {
            columns
                .iter()
                .map(|c| format!("\"{}\"::text", c.replace('"', "\"\"")))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let q = format!("SELECT {} FROM \"{}\" LIMIT $1 OFFSET $2", select_list, table.replace('"', "\"\""));
        let rows = client.query(&q, &[&(limit as i64), &(offset as i64)])?;
        let mut rows_vec = Vec::new();
        for r in rows {
            let mut row_vec = Vec::new();
            let cols = r.len();
            for i in 0..cols {
                let v: Option<String> = r.get(i);
                row_vec.push(v.unwrap_or_default());
            }
            rows_vec.push(row_vec);
        }

        let columns = if columns.is_empty() { vec!["(no columns)".to_string()] } else { columns };
        Ok(Records { columns, rows: rows_vec })
    }

    fn fetch_properties(
        conn: &Connection,
        _database: &str,
        table: &str,
    ) -> Result<TableProperties> {
        let url = Postgres::database_url(conn)?;
        let mut client = postgres::Client::connect(&url, postgres::NoTls)?;

        // columns
        let cols_rows = client.query(
            "SELECT column_name, data_type, is_nullable, column_default
             FROM information_schema.columns
             WHERE table_name = $1
             ORDER BY ordinal_position",
            &[&table],
        )?;
        let mut columns: Vec<ColumnInfo> = cols_rows
            .into_iter()
            .map(|r| ColumnInfo {
                name: r.get::<_, String>(0),
                data_type: r.get::<_, String>(1),
                nullable: {
                    let s: String = r.get(2);
                    s.eq_ignore_ascii_case("YES")
                },
                default: r.get::<_, Option<String>>(3),
                primary_key: false, // fill below
            })
            .collect();

        // primary key columns (use information_schema to avoid regclass parameter typing issues)
        let pk_rows = client.query(
            "SELECT kcu.column_name
             FROM information_schema.table_constraints tc
             JOIN information_schema.key_column_usage kcu
               ON tc.constraint_name = kcu.constraint_name
              AND tc.table_schema = kcu.table_schema
             WHERE tc.constraint_type = 'PRIMARY KEY'
               AND tc.table_name = $1",
            &[&table],
        )?;
        let pk: std::collections::HashSet<String> =
            pk_rows.into_iter().map(|r| r.get::<_, String>(0)).collect();
        for c in &mut columns {
            if pk.contains(&c.name) {
                c.primary_key = true;
            }
        }

        Ok(TableProperties { columns })
    }
}
