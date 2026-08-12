# Database Backup and Restore

Run the scripts on a secured operations host with PostgreSQL client tools (`pg_dump`, `pg_restore`). `DATABASE_URL` must point to the application database. Never place production passwords in source control or shell history.

Use client tools that match the PostgreSQL server major version. The scripts accept `PG_DUMP_BIN` and `PG_RESTORE_BIN` to select an approved versioned client binary.

Create a backup:

```bash
DATABASE_URL='postgres://...' ./backup-db.sh
```

The command creates a timestamped custom-format dump and SHA-256 integrity file under `./backups` by default. Set `BACKUP_DIR` to change the location. Restrict backup access to authorized operations staff and store backups in encrypted storage.

Restore replaces existing database contents. The command refuses to run without an explicit confirmation variable:

```bash
DATABASE_URL='postgres://...' CONFIRM_RESTORE=RESTORE ./restore-db.sh ./backups/jury-assistant-YYYYMMDDTHHMMSSZ.dump
```

Before production restore, take a current backup, use a maintenance window, and verify application/database access through `/health` afterward. First rehearse every restore into an isolated database using the same PostgreSQL major-version client as the target server.
