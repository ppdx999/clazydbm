use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::component::{Child, Database, Table};
use crate::{connection::Connection, db::DBBehavior};
use crate::db::Records;
use crate::logger::debug;

pub struct Sqlite {}

impl DBBehavior for Sqlite {
    fn database_url(conn: &Connection) -> Result<String> {
        let path = conn.path.as_ref().map_or(
            Err(anyhow::anyhow!("type sqlite needs the path field")),
            |path| expand_path(path).ok_or_else(|| anyhow::anyhow!("cannot expand file path")),
        )?;

        Ok(format!("sqlite://{path}", path = path.to_str().unwrap()))
    }
    fn fetch_databases(conn: &Connection) -> Result<Vec<Database>> {
        debug("sqlite: opening file");
        use rusqlite::Connection as SqliteConn;

        let path = conn
            .path
            .as_ref()
            .and_then(|p| expand_path(p))
            .ok_or_else(|| anyhow::anyhow!("invalid sqlite path"))?;

        let dbname = conn
            .name
            .clone()
            .or_else(|| path.file_stem().map(|s| s.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "sqlite".to_string());

        let sc = SqliteConn::open(path)?;
        debug("sqlite: opened");
        let mut stmt = sc.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

        let mut children = Vec::new();
        for r in rows {
            let name = r?;
            children.push(Child::Table(Table {
                name,
                engine: None,
                schema: None,
            }));
        }

        Ok(vec![Database::new(dbname, children)])
    }

    fn fetch_records(
        conn: &Connection,
        database: &str,
        table: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Records> {
        use rusqlite::Connection as SqliteConn;
        let _ = database; // not used for sqlite
        let path = conn
            .path
            .as_ref()
            .and_then(|p| expand_path(p))
            .ok_or_else(|| anyhow::anyhow!("invalid sqlite path"))?;
        let sc = SqliteConn::open(path)?;

        // columns
        let mut col_stmt = sc.prepare(&format!("PRAGMA table_info({});", table))?;
        let col_iter = col_stmt.query_map([], |row| row.get::<_, String>(1))?; // name is col 1
        let mut columns = Vec::new();
        for c in col_iter { columns.push(c?); }

        // rows: read ValueRef per column and stringify conservatively
        use rusqlite::types::ValueRef;
        let mut rows_vec: Vec<Vec<String>> = Vec::new();
        let q = format!("SELECT * FROM {} LIMIT {} OFFSET {}", table, limit, offset);
        let mut stmt = sc.prepare(&q)?;
        let col_count = stmt.column_count();
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let mut v = Vec::with_capacity(col_count);
            for i in 0..col_count {
                let cell = row.get_ref(i)?;
                let s = match cell {
                    ValueRef::Null => String::new(),
                    ValueRef::Integer(i) => i.to_string(),
                    ValueRef::Real(f) => f.to_string(),
                    ValueRef::Text(t) => String::from_utf8_lossy(t).into_owned(),
                    ValueRef::Blob(b) => format!("<blob {} bytes>", b.len()),
                };
                v.push(s);
            }
            rows_vec.push(v);
        }

        Ok(Records { columns, rows: rows_vec })
    }
}

fn expand_path(path: &Path) -> Option<PathBuf> {
    let mut expanded_path = PathBuf::new();
    let mut path_iter = path.iter();
    if path.starts_with("~") {
        path_iter.next()?;
        expanded_path = expanded_path.join(dirs_next::home_dir()?);
    }
    for path in path_iter {
        let path = path.to_str()?;
        expanded_path = if cfg!(unix) && path.starts_with('$') {
            expanded_path.join(std::env::var(path.strip_prefix('$')?).unwrap_or_default())
        } else if cfg!(windows) && path.starts_with('%') && path.ends_with('%') {
            expanded_path
                .join(std::env::var(path.strip_prefix('%')?.strip_suffix('%')?).unwrap_or_default())
        } else {
            expanded_path.join(path)
        }
    }
    Some(expanded_path)
}

// (fetch_databases moved into trait impl above)
