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
  ('"'"'Alice'"'"', '"'"'alice@example.com'"'"'),
  ('"'"'Bob'"'"', '"'"'bob@example.com'"'"'),
  ('"'"'Carol'"'"', '"'"'carol@example.com'"'"'),
  ('"'"'Dave'"'"', '"'"'dave@example.com'"'"'),
  ('"'"'Eve'"'"', '"'"'eve@example.com'"'"');

-- Large table for display/scroll testing
CREATE TABLE IF NOT EXISTS big_users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT
);
DELETE FROM big_users;
WITH RECURSIVE seq(x) AS (
  SELECT 1
  UNION ALL
  SELECT x + 1 FROM seq WHERE x < 1000
)
INSERT INTO big_users (id, name, email)
SELECT x, printf('"'"'User %04d'"'"', x), printf('"'"'user%04d@example.com'"'"', x)
FROM seq;'

# Add stress tables for scrolling/performance tests
sql_script+='
-- Very large row count table (10,000 rows)
CREATE TABLE IF NOT EXISTS huge_users (
    id INTEGER PRIMARY KEY,
    name TEXT,
    email TEXT
);
DELETE FROM huge_users;
WITH RECURSIVE a(n) AS (
  SELECT 1
  UNION ALL
  SELECT n + 1 FROM a WHERE n < 100
),
b(n) AS (
  SELECT 1
  UNION ALL
  SELECT n + 1 FROM b WHERE n < 100
)
INSERT INTO huge_users (id, name, email)
SELECT (b.n - 1) * 100 + a.n,
       printf('"'"'User %05d'"'"', (b.n - 1) * 100 + a.n),
       printf('"'"'user%05d@example.com'"'"', (b.n - 1) * 100 + a.n)
FROM a CROSS JOIN b;

-- Wide table with many columns to test Properties scrolling
CREATE TABLE IF NOT EXISTS wide_table (
    id INTEGER PRIMARY KEY,
    c01 TEXT, c02 TEXT, c03 TEXT, c04 TEXT, c05 TEXT, c06 TEXT, c07 TEXT, c08 TEXT, c09 TEXT, c10 TEXT,
    c11 TEXT, c12 TEXT, c13 TEXT, c14 TEXT, c15 TEXT, c16 TEXT, c17 TEXT, c18 TEXT, c19 TEXT, c20 TEXT,
    c21 TEXT, c22 TEXT, c23 TEXT, c24 TEXT, c25 TEXT, c26 TEXT, c27 TEXT, c28 TEXT, c29 TEXT, c30 TEXT,
    c31 TEXT, c32 TEXT, c33 TEXT, c34 TEXT, c35 TEXT, c36 TEXT, c37 TEXT, c38 TEXT, c39 TEXT, c40 TEXT,
    c41 TEXT, c42 TEXT, c43 TEXT, c44 TEXT, c45 TEXT, c46 TEXT, c47 TEXT, c48 TEXT, c49 TEXT, c50 TEXT,
    c51 TEXT, c52 TEXT, c53 TEXT, c54 TEXT, c55 TEXT, c56 TEXT, c57 TEXT, c58 TEXT, c59 TEXT, c60 TEXT
);
DELETE FROM wide_table;
INSERT INTO wide_table (id) VALUES (1);
'

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
  ('Alice', 'alice@example.com'),
  ('Bob', 'bob@example.com'),
  ('Carol', 'carol@example.com'),
  ('Dave', 'dave@example.com'),
  ('Eve', 'eve@example.com');

-- Large table for display/scroll testing
CREATE TABLE IF NOT EXISTS big_users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT
);
DELETE FROM big_users;
WITH RECURSIVE seq(x) AS (
  SELECT 1
  UNION ALL
  SELECT x + 1 FROM seq WHERE x < 1000
)
INSERT INTO big_users (id, name, email)
SELECT x, printf('User %04d', x), printf('user%04d@example.com', x)
FROM seq;

-- Very large row count table (10,000 rows)
CREATE TABLE IF NOT EXISTS huge_users (
    id INTEGER PRIMARY KEY,
    name TEXT,
    email TEXT
);
DELETE FROM huge_users;
WITH RECURSIVE a(n) AS (
  SELECT 1
  UNION ALL
  SELECT n + 1 FROM a WHERE n < 100
),
b(n) AS (
  SELECT 1
  UNION ALL
  SELECT n + 1 FROM b WHERE n < 100
)
INSERT INTO huge_users (id, name, email)
SELECT (b.n - 1) * 100 + a.n,
       printf('User %05d', (b.n - 1) * 100 + a.n),
       printf('user%05d@example.com', (b.n - 1) * 100 + a.n)
FROM a CROSS JOIN b;

-- Wide table with many columns to test Properties scrolling
CREATE TABLE IF NOT EXISTS wide_table (
    id INTEGER PRIMARY KEY,
    c01 TEXT, c02 TEXT, c03 TEXT, c04 TEXT, c05 TEXT, c06 TEXT, c07 TEXT, c08 TEXT, c09 TEXT, c10 TEXT,
    c11 TEXT, c12 TEXT, c13 TEXT, c14 TEXT, c15 TEXT, c16 TEXT, c17 TEXT, c18 TEXT, c19 TEXT, c20 TEXT,
    c21 TEXT, c22 TEXT, c23 TEXT, c24 TEXT, c25 TEXT, c26 TEXT, c27 TEXT, c28 TEXT, c29 TEXT, c30 TEXT,
    c31 TEXT, c32 TEXT, c33 TEXT, c34 TEXT, c35 TEXT, c36 TEXT, c37 TEXT, c38 TEXT, c39 TEXT, c40 TEXT,
    c41 TEXT, c42 TEXT, c43 TEXT, c44 TEXT, c45 TEXT, c46 TEXT, c47 TEXT, c48 TEXT, c49 TEXT, c50 TEXT,
    c51 TEXT, c52 TEXT, c53 TEXT, c54 TEXT, c55 TEXT, c56 TEXT, c57 TEXT, c58 TEXT, c59 TEXT, c60 TEXT
);
DELETE FROM wide_table;
INSERT INTO wide_table (id) VALUES (1);
SQL
fi

echo "[seed-sqlite] Seeded SQLite at $db_path"
