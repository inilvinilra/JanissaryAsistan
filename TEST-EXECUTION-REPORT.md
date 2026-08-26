# Test Execution Report

## Scope

This report records the automated and live verification completed in the isolated local environment through 2026-08-13. It supplements the role-based manual checklist in `SYSTEM-TEST-PLAN.md`.

## Completed Verification

| Area | Evidence | Result |
| --- | --- | --- |
| Backend unit and integration-oriented checks | `cargo test --quiet` | 27 passed |
| Backend compilation | `cargo check --quiet` | Passed |
| API smoke flow | `backend/scripts/api-smoke-test.sh` | 8 checks passed |
| Authorization regression | `backend/scripts/authorization-regression-test.sh` | 18 checks passed |
| Frontend unit tests | `npm test` | 4 passed |
| Browser workflow | `npm run test:e2e` with Chromium | 2 passed |
| Responsive browser checks | 1440px and 390px viewports | No horizontal overflow |
| AI HTTP contract | Backend project creation through local contract fixture | Complete KPI set persisted |
| Single-instance SSE | Project change emitted `refresh` | Passed |
| Multi-instance SSE | Change through port 3000 reached subscription through port 3001 | Passed |
| Schema migrations | Baseline, operational-index, and AI-analysis migrations | Versions 1, 2, and 3 recorded |
| Backup and restore | PostgreSQL 16 isolated restore rehearsal | Restored data readable |
| Monitoring | Prometheus metrics and Alertmanager profile | Configuration and Alertmanager readiness passed |
| Container build | `docker compose build` | Backend and frontend images built |
| Explainable AI workspace | Version-aware research, advisory similarity, and evidence-grounded Copilot routes | Passed through backend unit checks and local API verification |
| Tauri desktop shell | `cargo check` in `kys-app/src-tauri` | Passed with native file select/save plugin integration |
| Tauri desktop package | `npm run desktop:build` from `frontend` | Passed; Linux `.deb` package and desktop executable produced |
| Runtime recovery | API and frontend health checks after a clean Astro restart | Both returned `200`; React hydration verified in Chromium |
| Assessment workflow API acceptance | `backend/scripts/assessment-api-smoke-test.sh` in an isolated PostgreSQL environment | Passed: two project uploads, category-fit analysis, internal similarity analysis, and assessment-readiness gates |
| Production upload safety | Isolated production-profile API attempt without a ready ClamAV service | Passed: upload rejected with `503`; 2FA enrollment was required before protected operations |
| Account-specific 2FA policy | Dedicated authorization-policy unit tests and local database verification | Passed: 63/63 backend tests; one approved system-administrator account exempted, with no other exempt accounts |
| Assessment panel browser acceptance — access | Manual acceptance by a system administrator in the local dashboard | Passed: `Assessment readiness` panel and authorized `Run analyses` button were visible |
| Assessment panel browser acceptance — existing project | Manual run against a project without an analyzed report | Passed validation behavior: API returned `422 A parsed project report is required`; analysis was not run without source content |
| Assessment panel browser acceptance — report upload prerequisite | Manual `Proje Ekle` attempt with a Markdown report in the production-profile backend | Blocked safely: API returned `503 Virus scanner is unavailable; upload is blocked`. No ClamAV service was running, so no unscanned file was accepted. |
| Assessment panel browser acceptance — remaining | Manual test with a parsed report after ClamAV is available | Pending: category-fit result, similarity result, persistence after refresh, high-similarity `Review` flag, and stage-advance gate |

## Manual Acceptance Ownership

The remaining rows in `SYSTEM-TEST-PLAN.md` require a designated competition operator, judge, or administrator to validate real policy decisions, notification recipients, production secrets, and live institutional integrations. Record the tester, date, environment, and evidence against each row before production approval.

## Production Inputs Still Required

- A trained model service URL and its credential for `AI_SCORING_URL` and `AI_SCORING_TOKEN`.
- The institution-approved `ALERT_WEBHOOK_URL` and log aggregation destination.
- Formal sign-off by authorized competition operations stakeholders.
- An explicitly approved, non-sensitive test submission and configured provider key before exercising live third-party source search.
