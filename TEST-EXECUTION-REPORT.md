# Test Execution Report

## Scope

This report records the automated and live verification completed in the isolated local environment on 2026-08-12. It supplements the role-based manual checklist in `SYSTEM-TEST-PLAN.md`.

## Completed Verification

| Area | Evidence | Result |
| --- | --- | --- |
| Backend unit and integration-oriented checks | `cargo test --quiet` | 24 passed |
| Backend compilation | `cargo check --quiet` | Passed |
| API smoke flow | `backend/scripts/api-smoke-test.sh` | 8 checks passed |
| Authorization regression | `backend/scripts/authorization-regression-test.sh` | 18 checks passed |
| Frontend unit tests | `npm test` | 4 passed |
| Browser workflow | `npm run test:e2e` with Chromium | 2 passed |
| Responsive browser checks | 1440px and 390px viewports | No horizontal overflow |
| AI HTTP contract | Backend project creation through local contract fixture | Complete KPI set persisted |
| Single-instance SSE | Project change emitted `refresh` | Passed |
| Multi-instance SSE | Change through port 3000 reached subscription through port 3001 | Passed |
| Schema migrations | Baseline and operational index migrations | Versions 1 and 2 recorded |
| Backup and restore | PostgreSQL 16 isolated restore rehearsal | Restored data readable |
| Monitoring | Prometheus metrics and Alertmanager profile | Configuration and Alertmanager readiness passed |
| Container build | `docker compose build` | Backend and frontend images built |

## Manual Acceptance Ownership

The remaining rows in `SYSTEM-TEST-PLAN.md` require a designated competition operator, judge, or administrator to validate real policy decisions, notification recipients, production secrets, and live institutional integrations. Record the tester, date, environment, and evidence against each row before production approval.

## Production Inputs Still Required

- A trained model service URL and its credential for `AI_SCORING_URL` and `AI_SCORING_TOKEN`.
- The institution-approved `ALERT_WEBHOOK_URL` and log aggregation destination.
- Formal sign-off by authorized competition operations stakeholders.
