#!/usr/bin/env bash
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
repo_root=$(cd "$here/.." && pwd)

echo "[dev-demo] Starting MySQL and Postgres via docker-compose..."
docker compose -f "$repo_root/dev/docker-compose.yml" up -d

echo "[dev-demo] Waiting for databases to accept connections..."
# Simple wait loops for default ports
for i in {1..60}; do
  ok=0
  (echo > /dev/tcp/127.0.0.1/3306) >/dev/null 2>&1 && ok=$((ok+1)) || true
  (echo > /dev/tcp/127.0.0.1/5432) >/dev/null 2>&1 && ok=$((ok+1)) || true
  if [ "$ok" -eq 2 ]; then
    break
  fi
  sleep 1
done

echo "[dev-demo] Seeding SQLite sample DB..."
cargo run --quiet --bin seed_sqlite

cfg_dir=${XDG_CONFIG_HOME:-$HOME/.config}/clazydbm
mkdir -p "$cfg_dir"
cat > "$cfg_dir/config.yaml" <<'YAML'
conn:
  - type: mysql
    name: demo-mysql
    user: root
    password: rootpass
    host: 127.0.0.1
    port: 3306
    database: demo
  - type: postgres
    name: demo-postgres
    user: postgres
    password: postgres
    host: 127.0.0.1
    port: 5432
    database: demo
  - type: sqlite
    name: demo-sqlite
    path: dev/sqlite/sample.db
YAML

echo "[dev-demo] Launching clazydbm..."
cargo run
