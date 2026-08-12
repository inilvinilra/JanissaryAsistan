#!/usr/bin/env bash
set -euo pipefail

: "${DATABASE_URL:?DATABASE_URL must be set}"
if [[ "${CONFIRM_RESTORE:-}" != "RESTORE" ]]; then
  printf 'Restore would replace database contents. Re-run with CONFIRM_RESTORE=RESTORE.\n' >&2
  exit 2
fi

backup_file="${1:?Usage: CONFIRM_RESTORE=RESTORE ./restore-db.sh <backup.dump>}"
pg_restore_bin="${PG_RESTORE_BIN:-pg_restore}"

if [[ ! -f "$backup_file" ]]; then
  printf 'Backup file not found: %s\n' "$backup_file" >&2
  exit 1
fi

if [[ -f "${backup_file}.sha256" ]]; then
  (cd "$(dirname "$backup_file")" && sha256sum --check "$(basename "${backup_file}.sha256")")
fi

"$pg_restore_bin" --clean --if-exists --no-owner --dbname "$DATABASE_URL" "$backup_file"
printf 'Restore completed from: %s\n' "$backup_file"
