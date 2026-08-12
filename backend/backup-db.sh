#!/usr/bin/env bash
set -euo pipefail

: "${DATABASE_URL:?DATABASE_URL must be set}"

backup_dir="${BACKUP_DIR:-./backups}"
pg_dump_bin="${PG_DUMP_BIN:-pg_dump}"
timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
backup_file="${backup_dir}/jury-assistant-${timestamp}.dump"

mkdir -p "$backup_dir"
"$pg_dump_bin" --format=custom --no-owner --file "$backup_file" "$DATABASE_URL"
sha256sum "$backup_file" >"${backup_file}.sha256"

printf 'Backup created: %s\nChecksum: %s.sha256\n' "$backup_file" "$backup_file"
