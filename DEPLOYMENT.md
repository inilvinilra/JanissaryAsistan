# Deployment Guide

## Prerequisites

- Docker Engine with Docker Compose v2
- A DNS name and HTTPS reverse proxy for public production use
- A model service only when automatic AI scoring is enabled

## Production configuration

Copy the template and replace every placeholder with a production secret:

```bash
cp .env.production.example .env
```

Generate the encryption key with a cryptographically secure secret manager or equivalent process. It must decode to exactly 32 bytes and be Base64 encoded. Do not commit `.env`.

Required values:

| Variable | Purpose |
| --- | --- |
| `POSTGRES_PASSWORD` | PostgreSQL application password. |
| `FILE_ENCRYPTION_KEY` | Base64 AES-256-GCM key for files and TOTP secrets. |
| `BOOTSTRAP_ADMIN_EMAIL` | Initial system-administrator email; required only for the first production start. |
| `BOOTSTRAP_ADMIN_PASSWORD` | Initial system-administrator password; required only for the first production start. |
| `REQUIRE_TWO_FACTOR` | Requires TOTP enrollment for privileged roles; set to `true` in production. |

The backend image contains ClamAV and Turkish/English OCR packages. Before enabling uploads, update ClamAV signatures through the platform's scheduled `freshclam` job or a managed scanner integration. Because `VIRUS_SCAN_REQUIRED=true`, an unavailable or outdated scanner rejects uploads instead of accepting unscanned content.

## Monitoring and backups

`observability/prometheus.yml` scrapes the API `/metrics` endpoint and `observability/prometheus-alerts.yml` defines availability and rate-limit alerts. The optional Alertmanager profile renders `observability/alertmanager.yml.template` using `ALERT_WEBHOOK_URL` and sends firing/resolved alerts to the institution-approved webhook.

The backend emits structured JSON logs to standard output. Set `RUST_LOG` (default: `backend=info`) to control verbosity, then forward container standard output to the institution's approved log aggregation platform. Request logs include method, route, status, and duration, but do not include session tokens or request bodies.

For horizontal backend scaling, all instances must use the same PostgreSQL database. Authenticated dashboard refresh events are distributed through PostgreSQL `LISTEN/NOTIFY`; configure each instance with its own `BIND_ADDRESS` only when it is not behind a reverse proxy.

To start the optional local monitoring profile, set `ALERT_WEBHOOK_URL` and run `docker compose --profile monitoring up -d`. Prometheus is exposed locally on port `9090` and Alertmanager on port `9093`; production deployment should restrict both ports to monitoring infrastructure.

`ops/backup-crontab.example` is a daily backup schedule template. Replace the connection value through the host secret manager rather than committing it, store generated backups in encrypted restricted storage, and rehearse restoration with `backend/restore-db.sh`.
| `PUBLIC_FRONTEND_ORIGIN` | Exact HTTPS dashboard origin allowed by API CORS. |
| `PUBLIC_API_URL` | API origin compiled into the frontend. |

Optional AI scoring uses `AI_SCORING_URL` and `AI_SCORING_TOKEN`. See [backend/AI-SCORING-CONTRACT.md](backend/AI-SCORING-CONTRACT.md).

## Start

```bash
docker compose up --build -d
docker compose ps
curl --fail http://127.0.0.1:3000/health
curl --fail http://127.0.0.1:3000/metrics
```

The backend refuses production startup without a valid encryption key. Production Compose disables sample data and requires malware scanning.

## Backup and restore

Use [backend/BACKUP-RESTORE.md](backend/BACKUP-RESTORE.md) for database backup and recovery. Run a restore rehearsal before go-live and schedule backups outside the application container.

## Production checklist

- Terminate TLS at a reverse proxy and set `PUBLIC_FRONTEND_ORIGIN` to the HTTPS dashboard URL.
- Keep PostgreSQL and encryption keys in managed secret storage.
- Install required Tesseract language packs before setting `OCR_LANGUAGES`, for example `tur+eng`.
- Scrape `/metrics`, define alerts for health failures and rate-limit rejections, and aggregate container logs.
- Schedule encrypted backups and periodic restore rehearsals.
- Connect a real model endpoint before enabling `AI_SCORING_URL`.
