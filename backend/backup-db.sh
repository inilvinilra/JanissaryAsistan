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

# Uploaded reports live on disk; the database only stores their paths. A dump
# alone restores rows pointing at files that no longer exist, so set
# UPLOADS_DIR to archive them in the same run. Under Compose the files are in
# the jury-uploads volume, reachable with:
#   docker run --rm -v jury-uploads:/uploads -v "$PWD/backups":/backups alpine \
#     tar -czf /backups/uploads.tar.gz -C /uploads .
if [[ -n "${UPLOADS_DIR:-}" ]]; then
  if [[ ! -d "$UPLOADS_DIR" ]]; then
    printf 'UPLOADS_DIR is set but not a directory: %s\n' "$UPLOADS_DIR" >&2
    exit 1
  fi
  uploads_file="${backup_dir}/jury-assistant-uploads-${timestamp}.tar.gz"
  tar -czf "$uploads_file" -C "$UPLOADS_DIR" .
  sha256sum "$uploads_file" >"${uploads_file}.sha256"
  printf 'Uploads archived: %s\nChecksum: %s.sha256\n' "$uploads_file" "$uploads_file"
else
  printf 'Note: UPLOADS_DIR is not set, so submitted files were NOT backed up.\n' >&2
fi
