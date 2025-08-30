#!/usr/bin/env bash
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$here/.." && pwd)
db_dir="$repo_root/dev/sqlite"
db_path="$db_dir/sample.db"

mkdir -p "$db_dir"

sql_script='CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT
);
DELETE FROM users;
INSERT INTO users (name, email) VALUES
  ("Alice", "alice@example.com"),
  ("Bob", "bob@example.com"),
  ("Carol", "carol@example.com"),
  ("Dave", "dave@example.com"),
  ("Eve", "eve@example.com");'

if command -v sqlite3 >/dev/null 2>&1; then
  echo "[seed-sqlite] Using local sqlite3 CLI"
  printf '%s' "$sql_script" | sqlite3 "$db_path"
else
  echo "[seed-sqlite] Local sqlite3 not found; using Docker image keinos/sqlite3"
  # Run sqlite3 in a container, mounting the dev/sqlite directory
  docker run --rm -i \
    -v "$db_dir":/db \
    keinos/sqlite3 \
    sqlite3 /db/sample.db <<'SQL'
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT
);
DELETE FROM users;
INSERT INTO users (name, email) VALUES
  ("Alice", "alice@example.com"),
  ("Bob", "bob@example.com"),
  ("Carol", "carol@example.com"),
  ("Dave", "dave@example.com"),
  ("Eve", "eve@example.com");
SQL
fi

echo "[seed-sqlite] Seeded SQLite at $db_path"

