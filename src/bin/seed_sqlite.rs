use anyhow::Result;

fn main() -> Result<()> {
    let path = std::path::Path::new("dev/sqlite");
    std::fs::create_dir_all(path)?;
    let db_path = path.join("sample.db");
    let conn = rusqlite::Connection::open(&db_path)?;
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT
        );
        DELETE FROM users;
        INSERT INTO users (name, email) VALUES
          ('Alice', 'alice@example.com'),
          ('Bob', 'bob@example.com'),
          ('Carol', 'carol@example.com'),
          ('Dave', 'dave@example.com'),
          ('Eve', 'eve@example.com');
        "#,
    )?;
    println!("Seeded SQLite at {}", db_path.display());
    Ok(())
}
