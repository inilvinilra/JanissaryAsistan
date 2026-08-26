# Test Execution Report

## Scope

Automated and live verification of the evaluation system, recorded against the
current tree. It supplements the role-based manual checklist in
`SYSTEM-TEST-PLAN.md`.

Every figure below was produced by running the command named beside it. Rows
that were not executed say so.

## Automated suites

| Area | Command | Result |
| --- | --- | --- |
| Backend, no database available | `cargo test` | 144 passed, 0 failed |
| Backend, persistence suite | `TEST_DATABASE_URL=… cargo test` | 144 passed, of which 9 execute real SQL |
| Backend formatting | `cargo fmt --check` | Clean |
| Backend compilation | `cargo build`, `cargo check` | No warnings |
| Frontend type checking | `npm run typecheck` (`astro check`) | 0 errors, 0 warnings across 70 files |
| Frontend unit tests | `npm test` | 6 passed |
| Frontend production build | `npm run build` | Built |
| Dependency audit | `npm audit --omit=dev --audit-level=high` | 0 vulnerabilities |
| Compose configuration | `docker compose config` | Valid |

The persistence suite creates and drops a database per test, so it needs a
server it may administer. It is skipped when `TEST_DATABASE_URL` is unset, which
keeps `cargo test` green on a machine with no PostgreSQL. CI provides a
`postgres:16-alpine` service so it runs there.

Roughly 2,900 of the backend's 17,800 source lines are tests.

## MVP gates, verified against a running stack

Exercised end to end through the API with PostgreSQL 18, a live Mistral key, and
a corpus of thirteen Turkish reports plus deliberately malformed submissions.

| # | Gate | Evidence |
| --- | --- | --- |
| 01 | Language and template detection | Turkish detected on every corpus report; 12 language unit tests covering Turkish, English, Azerbaijani and shared-diacritic cases |
| 02 | Template compliance | Section score, language match and word-count bounds enforced; 20 template unit tests |
| 03 | Headings and content | Structural checks plus off-topic detection: a report with all seven correct headings and 168 words in each, filled with unrelated budget prose, is flagged on all six required sections, while the four real reports produce no off-topic finding |
| 04 | Category fit | 12/12 correct across the corpus with 0 falsely marked for review |
| 05 | Similarity | Independent reports pass (Jaccard 0.25–0.34); near-duplicates are flagged (0.76–0.85); a duplicate upload scores 1.00 |
| 06 | AI criterion evaluation and feedback | Runs with Mistral and offline; evidence quoted from the report and verified against it; applicant feedback populated in every area |

## Live verification

| Check | Evidence |
| --- | --- |
| Configuration safety | A `.env` with an unquoted value aborts startup naming the offending line, instead of silently dropping every variable below it |
| Removed upload route | `POST /projects` returns 405; `GET /projects` still 200 |
| Bulk analysis | 11 projects queued, 11 completed, 0 failed in 96s, with progress tracked from 71.8% to 100% while the run was in flight |
| API responsiveness under upload | Health checks stayed at 5–14 ms while a report was parsed and scored |
| Applicant confidentiality | A real contestant account receives no `risks` field and no project reference of any kind |
| Malformed upload isolation | A file whose contents do not match its extension is rejected with 415 on its own; the other files in the same import complete |
| Offline evaluation | With no model key, 2 of 3 criteria carry quoted evidence and confidence sits at 0.47–0.58 |

## Not executed here

- **Browser end-to-end (`npm run test:e2e`).** The Playwright specs are skipped
  unless `JURY_E2E_EMAIL` and `JURY_E2E_PASSWORD` are set, and CI does not run
  them. They need an isolated test account.
- **Production Compose profile.** The ClamAV signature service and the uploads
  volume are configured and the Compose file validates, but the stack was not
  started with `APP_ENV=production` on this machine — Docker was unavailable.
  Verify `docker compose up` and a first upload before go-live.
- **Desktop packaging.** The Tauri shell builds the Astro frontend through
  `tauri.conf.json`; no package was produced in this run.
- **Multi-instance SSE and backup restore rehearsal.** Both need more than one
  running instance and a restore target.

## Production inputs still required

- A model service credential for the deployment environment, and a decision on
  whether `AI_SCORING_URL` or the built-in Mistral path is used.
- The institution-approved `ALERT_WEBHOOK_URL` and log aggregation destination.
- Formal sign-off by authorised competition operations stakeholders.
- An approved, non-sensitive test submission before exercising live third-party
  source search.
