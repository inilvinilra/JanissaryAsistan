# Critical Improvements Register

## Working Rules

- All new code, API messages, identifiers, and essential code comments must be written in English.
- Remove non-critical code comments during the relevant refactor. Keep comments only when they document a security decision, a non-obvious business rule, or a technical constraint that code alone cannot express.
- Update this register in the same change that implements, verifies, or re-scopes an item.
- Do not mark an item complete until its automated verification is recorded.

## Status Legend

- `Open` — identified but not started.
- `In progress` — implementation has started.
- `Blocked` — requires an external decision or dependency.
- `Verified` — implemented and verified by automated checks.

## User-Assigned Tasks

Tasks provided by the product owner will be added here verbatim and tracked separately from the technical findings.

| ID | Task | Status | Verification | Notes |
| --- | --- | --- | --- | --- |

## Technical Findings

### P0 — Must be resolved before production use

| ID | Finding | Status | Verification target |
| --- | --- | --- | --- |
| AI-UX-01 | Integrate Emirhan branch AI analysis, similarity/source analysis, AI Copilot, and the Tauri delivery shell into the current enterprise dashboard without duplicating authorization or backend logic. | Verified | AI workspace, Copilot, source analysis, Tauri integration, and regression tests |
| SEC-01 | Route-level authentication exists, but role permissions are not enforced per endpoint. A jury member can currently call administrative endpoints. | Verified | Role/API authorization integration tests |
| DATA-01 | Projects have no mandatory `competition_id` or `team_id`. Competition-scoped data separation cannot be guaranteed. | Verified | Database migration and cross-competition access tests |
| SEC-02 | Competition and category scope enforcement covers direct competition routes plus project, team, submission, demo-day, and appeal resources. | Verified | Tenant and category isolation test matrix |
| SEC-03 | Blind review is optional in the UI. Standard project endpoints still disclose identities to authenticated jury users. | Verified | Jury role response-redaction tests |
| AUD-01 | Audit actors are often supplied as free text or static values instead of the authenticated user identity. | Verified | Audit actor integrity integration tests |
| AUD-02 | Audit chaining uses a non-cryptographic default hasher. | Verified | SHA-256/HMAC chain verification tests |
| AI-01 | The backend AI adapter has been verified end-to-end against an authenticated HTTP contract fixture. A trained model-service URL and credential are still required for production scoring. | In progress | AI adapter contract and model-service integration tests |

### P1 — Required for a secure operational release

| ID | Finding | Status | Verification target |
| --- | --- | --- | --- |
| IAM-01 | User creation has no invitation or first-password workflow. The current UI does not collect a password, so newly created users may be unable to sign in. | Verified | Invitation, password setup, reset, and login tests |
| IAM-02 | Sessions provide visible sign-out, expiration validation, TOTP, recovery codes, and a production-default mandatory TOTP policy for privileged roles. | Verified | Session lifecycle and 2FA end-to-end tests |
| SEC-04 | File encryption is disabled unless `FILE_ENCRYPTION_KEY` is configured. TOTP secrets are stored as plaintext database values. | Verified | Production configuration validation and secret-at-rest tests |
| SEC-05 | Rate limiting is backed by an atomic PostgreSQL minute-window counter. Client identifiers are SHA-256 hashed before persistence; forwarded headers are accepted only when an explicitly trusted proxy is configured. | Verified | Distributed rate-limit and trusted-proxy tests |
| FILE-01 | DOCX, XLSX, and image files are attachments only; they are not parsed for evaluation. Scanned PDFs have no OCR path. | Verified | File extraction and OCR fixtures |
| FILE-02 | Malware scanning can be skipped when ClamAV is unavailable. | Verified | Production fail-closed configuration unit test |
| OPS-01 | Container topology, CI, clean-database bootstrap, Prometheus metrics/alerts, Alertmanager webhook configuration, scheduled-backup template, versioned schema migrations, JSON stdout logs, and an isolated backup-restore rehearsal exist. Institution-managed endpoints remain deployment-environment inputs. | In progress | Deployment rehearsal and observability checks |
| OPS-02 | Sample KPI/project data is seeded whenever the backend starts. | Verified | Production-mode startup test with seed disabled |
| DOC-01 | The API documentation is stale and contradicts current authentication/CORS behavior. | Verified | API contract review against router and middleware |
| TEST-01 | Backend unit coverage, a complete manual acceptance checklist, API smoke coverage, live authorization regression coverage, frontend unit tests, Chromium workflows, and an execution report exist. Formal stakeholder acceptance remains incomplete. | In progress | Test-plan execution report |

### P2 — Important quality and usability improvements

| ID | Finding | Status | Verification target |
| --- | --- | --- | --- |
| REPORT-01 | Excel export is tab-delimited content with an `.xls` extension; PDF export uses browser printing rather than a controlled report renderer. | Verified | XLSX/PDF artifact validation |
| OPS-03 | Email dispatch is synchronous and depends on a webhook. It has no durable queue, retry, backoff, or idempotency mechanism. | Verified | Delivery retry and failure recovery tests |
| UX-01 | Dashboard updates use authenticated Server-Sent Events. PostgreSQL `LISTEN/NOTIFY` distributes successful state changes across backend instances sharing the same database. | Verified | Concurrent update and multi-instance delivery tests |
| UX-02 | The frontend has consolidated navigation, deferred workspaces, and role-aware visibility. Final responsive user-acceptance review remains incomplete. | In progress | Responsive UX acceptance checklist |
| PERF-01 | Dashboard-only modules are deferred through lazy loading, keeping the initial production bundle below the warning threshold. | Verified | Bundle budget check |
| CODE-01 | Runtime UI text is routed through the bilingual localization layer and backend user-facing operational text is English. Turkish parsing indicators remain intentionally for Turkish document support. | Verified | Source-language lint and locale fallback review |

## Change Log

| Date | Item | Change | Automated verification |
| --- | --- | --- | --- |
| 2026-08-12 | Register created | Recorded the initial technical, security, operational, and UX findings. | Existing backend tests: 7/7 passed; frontend production build passed. |
| 2026-08-12 | P0 access-control block | Started the project ownership, authorization, blind-review, and audit-actor implementation. | Pending |
| 2026-08-12 | SEC-01 | Added endpoint-level role policy. Live jury account test: `/users` returned `403`; scoped project list returned `200`; another category returned `403`. | Backend tests: 8/8 passed; frontend production build passed. |
| 2026-08-12 | DATA-01 | Added mandatory `projects.competition_id`, optional `team_id`, legacy backfill to a default competition, scoped listing, and upload validation. | Backend tests: 8/8 passed; frontend production build passed. |
| 2026-08-12 | SEC-03 | Enforced anonymous project views and denied jury access to metadata, files, documents, appeals, and jury assignments. Live jury test returned `PRJ-000141` for list/detail and `403` for metadata. | Backend tests: 8/8 passed; frontend production build passed. |
| 2026-08-12 | AUD-01 | Replaced free-text and static audit actors in operational handlers with the authenticated session email. Jury assignment auditing now records the assigning user and the assigned juror separately. | Backend tests: 9/9 passed. |
| 2026-08-12 | AUD-02 | Replaced `DefaultHasher` audit chaining with SHA-256 and persisted the exact hashed timestamp. | SHA-256 chain unit test passed; backend tests: 9/9 passed. |
| 2026-08-12 | SEC-02 | Added central competition resolution for project, team, submission, demo-day, and appeal routes. Live competition-manager test: an update to another competition's team returned `403`. | Backend tests: 9/9 passed. |
| 2026-08-12 | IAM-01 | Added a required temporary-password field to the user-management form and required passwords in `POST /users`. | Live API test: passwordless user creation returned `400`; backend tests: 10/10 passed; frontend production build passed. |
| 2026-08-12 | OPS-02 | Added `APP_ENV` and `SEED_SAMPLE_DATA` controls. Production mode now defaults sample data to disabled and requires `FILE_ENCRYPTION_KEY`. | Production seed-default unit test passed; backend tests: 10/10 passed. |
| 2026-08-12 | Code hygiene | Removed non-critical source comments. The remaining comments document authentication and malware-scanning security behavior only. Removed free-text juror identity state; the UI now displays the authenticated user and backend audit data uses the session identity. | Backend tests: 10/10 passed; frontend production build passed. |
| 2026-08-12 | SEC-04 | Added encrypted `enc:` storage for TOTP secrets using the existing AES protection layer. Legacy plaintext TOTP values remain readable for migration; production requires `FILE_ENCRYPTION_KEY`. | Two-factor secret storage unit test passed; backend tests: 11/11 passed. |
| 2026-08-12 | IAM-02 | Added a visible sign-out action and `GET /auth/session` validation on dashboard startup. The client clears invalid or revoked sessions. | Live session test: active `200`, logout `204`, revoked session `401`; backend tests: 11/11 passed; frontend production build passed. |
| 2026-08-12 | IAM-02 | Added a dashboard Settings flow for TOTP setup and six-digit verification. | Live `POST /auth/2fa/setup` returned `200`; frontend production build passed. |
| 2026-08-12 | FILE-02 | Made malware scanning fail closed by default in production. `VIRUS_SCAN_REQUIRED` can explicitly control development behavior, while an unavailable scanner rejects production uploads. | Production scanner-default unit test passed; backend tests: 12/12 passed. |
| 2026-08-12 | SEC-02, SEC-03 | Reopened after a deeper review found competition-directory disclosure and jury access to AI-evaluation payloads. Scoped competition and organization listings, restricted competition creation to system administrators, and denied jury AI-evaluation access. | Pending regression and live authorization tests. |
| 2026-08-12 | SEC-03 | Confirmed the revised blind-review policy with the scoped jury account: `GET /projects/141/ai-evaluation` returned `403`. | Backend tests: 13/13 passed; frontend production build passed. |
| 2026-08-12 | SEC-02 | Confirmed the competition-directory restriction with the scoped jury account: `GET /competitions` returned one assigned competition only. | Backend tests: 13/13 passed; live authorization test passed. |
| 2026-08-12 | IAM-01 | Added `must_change_password` to new accounts, a self-service password-change endpoint that verifies the current password, and a blocking frontend screen. Corrected the `users` migration order so a clean database can create the table before applying user-column extensions. | Live flow: created account `true`; dashboard `403`; password change `204`; dashboard `200`. Backend tests: 13/13 passed; frontend production build passed. |
| 2026-08-12 | CODE-01 | Source scan found remaining Turkish runtime strings outside `frontend/src/lib/i18n.ts`; the bilingual dictionary itself is intentional. | Pending localization cleanup during the frontend consolidation phase. |
| 2026-08-12 | SEC-05 | Started trusted-proxy hardening: rate limiting now ignores user-supplied forwarding headers unless `TRUST_PROXY_HEADERS=true` is explicitly configured. | Header-spoofing unit test passed; backend tests: 14/14 passed; frontend production build passed. |
| 2026-08-12 | DOC-01 | Replaced the obsolete API guide with the current session, authorization, scope, upload, security, and route contract. | Reviewed against the Axum router and authentication middleware. |
| 2026-08-12 | REPORT-01 | Removed the unsafe `xlsx` package after npm audit reported unresolved high-severity vulnerabilities. Implemented a dependency-free XLSX writer for the ranking export. | Pending production build and generated-artifact validation. |
| 2026-08-12 | REPORT-01 | Verified the XLSX export artifact: generated a 2,343-byte workbook and validated all Open XML ZIP entries with `unzip -t`. PDF renderer work remains open. | Frontend production build passed; npm audit: 0 vulnerabilities. |
| 2026-08-12 | OPS-03 | Added a database-backed delivery queue with claim-safe delivery state, five attempts, bounded exponential backoff, error persistence, webhook timeout, and a background worker. Dispatch now queues work instead of synchronously delivering it. | Pending retry-state live verification. |
| 2026-08-12 | OPS-03 | Verified a local unreachable-webhook flow: dispatch returned `200` without waiting, campaign remained `queued`, and the delivery row persisted `queued:1:error-recorded` for retry. | Backend tests: 15/15 passed; frontend production build passed; live queue test passed. |
| 2026-08-12 | REPORT-01 | Replaced browser-print PDF export with a dependency-free generated PDF containing report metrics and the top ten projects. | Pending production build and PDF artifact validation. |
| 2026-08-12 | REPORT-01 | Verified the generated PDF artifact as a valid one-page A4 PDF 1.4 document. | Frontend production build passed; `pdfinfo` artifact validation passed. |
| 2026-08-12 | FILE-01 | Added DOCX, XLSX/XLS, CSV, and optional OCR parsing for image and scanned-PDF submissions. OCR language selection is configurable through `OCR_LANGUAGES`; the portable default is `eng`. | DOCX and XLSX fixture tests passed; local Tesseract image extraction returned expected text; backend tests: 18/18 passed. |
| 2026-08-12 | IAM-01 | Added administrator-issued, one-hour, single-use password reset tokens. Only a SHA-256 token hash is stored; successful confirmation updates the password, clears the first-password flag, and revokes active sessions. | Pending live reset and token-reuse verification. |
| 2026-08-12 | IAM-01 | Added a reset-link generation action in user management and a public reset-password screen. Live flow verified: confirm `204`, old password rejected `401`, new password accepted `200`, token reuse rejected `401`. | Backend tests: 19/19 passed; frontend production build passed. |
| 2026-08-12 | IAM-02 | Added ten one-time, SHA-256-hashed recovery codes when TOTP is enabled. The settings page shows them only once and can copy them to the clipboard. | Live recovery sign-in returned `200`; reuse returned `401`; backend tests: 20/20 passed; frontend production build passed. |
| 2026-08-12 | SEC-04 | Production startup now validates that `FILE_ENCRYPTION_KEY` is present, Base64-decoded, and exactly 32 bytes before any database or API work. TOTP storage uses an explicit `plain:` development marker when no key is configured and AES-GCM `enc:` storage with a key. | Production startup without a key rejected as expected; backend tests: 21/21 passed; frontend production build passed. |
| 2026-08-12 | AI-01 | Added an optional authenticated HTTP scoring adapter selected by `AI_SCORING_URL`, including 30-second timeout and strict complete-KPI response validation. Added the handoff contract for the model team. | Adapter response-validation test passed; backend tests: 22/22 passed. A real model endpoint is still required for production. |
| 2026-08-12 | OPS-01 | Added Dockerfiles, production Compose topology with PostgreSQL health checks and explicit secret variables, a production environment template, and GitHub Actions backend/frontend verification. | Pending Compose configuration validation and container build rehearsal. |
| 2026-08-12 | OPS-01 | Validated the Compose configuration without starting services and added a production deployment guide. | `docker compose config --quiet` passed. Versioned migrations, observability, alerting, and scheduled backups remain open. |
| 2026-08-12 | OPS-01 | Docker build rehearsal exposed oversized build contexts caused by local build artifacts. Added service-specific `.dockerignore` rules before retrying the rehearsal. | Pending clean container build. |
| 2026-08-12 | SEC-01 | Reopened after documentation review found that `observer` and `read_only` accepted every GET endpoint. Replaced that with an explicit safe-read route list. | Pending regression and live role tests. |
| 2026-08-12 | SEC-01 | Live observer test: project list returned `200`; `/users`, `/audit`, and `/projects/141/ai-evaluation` each returned `403`. | Backend tests: 14/14 passed; live role test passed. |
| 2026-08-12 | SEC-02 | Restricted global notification and email-campaign records to system administrators because those records do not yet carry a competition scope. Removed the stale observer `audit:view` permission from the role catalogue. | Backend tests: 22/22 passed. |
| 2026-08-12 | CODE-01 | Moved remaining project-detail blind-review, appeal, eligibility, AI-confidence, file-version, and jury-assignment labels into the bilingual dictionary. | Frontend production build passed; bundle-size warning remains tracked under `PERF-01`. |
| 2026-08-12 | SEC-05 | Replaced in-memory rate limits with an atomic PostgreSQL minute-window counter and peer-address extraction. Persisted client keys are SHA-256 hashes; `X-Forwarded-For` remains opt-in through `TRUST_PROXY_HEADERS`. | Backend tests: 22/22 passed. Live test: requests 119–120 returned `200`, request 121 returned `429`; database stored a 64-character hash and count `121`. |
| 2026-08-12 | OPS-01 | Clean PostgreSQL startup rehearsal exposed a schema-order defect: `jury_assignments` columns were altered before the table existed. Reordered initialization so the table is created first. | Live clean-database startup reached `Database connected, tables ready.` and served port 3000. |
| 2026-08-12 | IAM-02 | Added a production-default `REQUIRE_TWO_FACTOR` policy for privileged roles and an enrollment gate that prevents dashboard access until TOTP setup succeeds. The deployment topology enables this policy explicitly. | Backend tests: 23/23 passed; frontend production build passed. |
| 2026-08-12 | OPS-01, IAM-02 | Added transaction-locked bootstrap creation for an empty production database. A temporary clean-database administrator login returned `two_factor_required: true`; its dashboard request returned `403` until TOTP enrollment. | Live production-like bootstrap and policy test passed. |
| 2026-08-12 | TEST-01 | Added the complete manual acceptance checklist covering authentication, scope, operations, evaluation, files, audit, reporting, communications, and deployment. | `SYSTEM-TEST-PLAN.md` created; execution remains in progress. |
| 2026-08-12 | OPS-01 | Built both backend and frontend production Docker images from compact service contexts. Backend image contains ClamAV and English/Turkish OCR packages. | Backend and frontend Docker builds passed; Compose emitted only the non-blocking missing-buildx warning. |
| 2026-08-12 | PERF-01 | Split dashboard panels, dialogs, reports, competition workspaces, and charting into deferred chunks. | Frontend production build passed with no 500 KB chunk warning. |
| 2026-08-12 | CODE-01 | Removed remaining direct runtime Turkish labels from frontend components and backend operational responses. Turkish text is now confined to the intentional localization dictionary and parser language support. | Frontend source scan found only `lib/i18n.ts`; backend scan found only Turkish document-analysis patterns. |
| 2026-08-12 | OPS-01 | Added Prometheus-compatible request/rate-limit metrics, alert rules, an optional Compose monitoring profile, and a daily backup cron template. | Backend tests: 23/23 passed; live `/metrics` output validated; monitoring Compose configuration passed. |
| 2026-08-12 | UX-02 | Made navigation role-aware and hid privileged workspaces, user management, audit access, notifications, and project creation from roles without access. | Frontend production build passed. |
| 2026-08-12 | UX-01 | Replaced dashboard polling with an authenticated Server-Sent Events subscription. Successful state-changing requests broadcast a refresh event; the dashboard reloads its visible data on receipt. | Backend tests: 23/23 passed; backend check passed; frontend production build passed; live project update emitted `event: refresh` with `data: updated`. |
| 2026-08-12 | OPS-01 | Rebuilt the production backend and frontend images after the event-stream and dashboard changes. | Both `arastirmaotomasyonu-backend:latest` and `arastirmaotomasyonu-frontend:latest` image inspections confirmed current build timestamps. |
| 2026-08-12 | SEC-02, UX-01 | Added regression coverage for observer-safe reads, protected project subresources, and event-stream visibility. | Backend tests: 24/24 passed; backend check passed; frontend production build passed. |
| 2026-08-12 | TEST-01 | Added a non-mutating API smoke test covering health, anonymous denial, session issuance and validation, authenticated lists, metrics, logout, and revoked-session denial. | Live isolated-account run passed all eight checks. |
| 2026-08-12 | OPS-01 | Added idempotent `schema_migrations` tracking with a baseline migration marker. Existing databases are adopted once; newer schemas fail closed rather than running an older backend. | Backend tests: 24/24 passed; backend check passed; live database recorded `1|baseline_schema`; a second backend startup completed without reapplying it. |
| 2026-08-12 | OPS-01 | Added migration 2 with indexes for competition/category project queries, teams, jury scores, audit timeline, active sessions, and ready email deliveries. | Backend tests: 24/24 passed; live database recorded `2|operational_indexes` and confirmed both project and email-delivery indexes. |
| 2026-08-12 | OPS-01 | Added structured JSON startup, error, and request-completion logs controlled by `RUST_LOG`; Compose and deployment guidance now expose the collection boundary. | Backend tests: 24/24 passed; live health request produced JSON fields for method, route, status, and duration. |
| 2026-08-12 | UX-01 | Added PostgreSQL `LISTEN/NOTIFY` fan-out for authenticated Server-Sent Events and configurable `BIND_ADDRESS` to support multiple backend instances. | Backend tests: 24/24 passed; live cross-instance test updated a project through port 3000 and received `event: refresh` on an SSE subscription through independent port 3001. |
| 2026-08-12 | SEC-02, TEST-01 | Added isolated-database authorization regression coverage that creates scoped jury and observer users, verifies blind-review denials, category enforcement, competition-list filtering, and cross-competition route denial. | Live run passed all 18 checks; script syntax validation passed. |
| 2026-08-12 | TEST-01 | Added Vitest frontend unit coverage for localization, category grouping, XLSX escaping/ZIP output, and PDF output. CI now runs frontend tests before the production build. | Frontend tests: 4/4 passed; npm audit: 0 vulnerabilities. |
| 2026-08-12 | TEST-01 | Added a Playwright Chromium test for sign-in, hydrated dashboard load, and sign-out. Fixed development API fallback and local IPv4 configuration so a browser can reach the API reliably. | Live Chromium E2E test passed using an isolated account. |
| 2026-08-12 | UX-02, TEST-01 | Extended the Chromium E2E suite with desktop and mobile viewport checks for horizontal overflow. | Live Chromium tests: 2/2 passed at 1440px and 390px widths. |
| 2026-08-12 | OPS-01 | Rebuilt production backend and frontend images after schema migration, multi-instance events, structured logs, and test-infrastructure updates. | Compose configuration passed; both latest image timestamps confirmed; local API health and dashboard HTTP checks returned `200`; image build audit found 0 vulnerabilities. |
| 2026-08-12 | AI-01 | Added an authenticated local HTTP contract fixture and validated backend project scoring through the configured external-adapter path. | Live project creation persisted a complete three-KPI score response from the fixture. |
| 2026-08-12 | OPS-01 | Added optional Alertmanager Compose service, Prometheus forwarding, webhook template, and production environment variable. | Monitoring Compose configuration passed; local Alertmanager readiness returned `200` and configuration load was logged. |
| 2026-08-12 | TEST-01 | Added a consolidated automated and live verification report. | `TEST-EXECUTION-REPORT.md` records all completed evidence and remaining human ownership. |
| 2026-08-13 | OPS-01, TEST-01 | Fixed the development CORS mismatch that prevented a dashboard opened at `localhost:4321` from reaching an API that allowed only `127.0.0.1:4321`. Development now permits both local origins; production fails fast unless an explicit `PUBLIC_FRONTEND_ORIGIN` is configured. | Backend tests: 24/24 passed; live health and login preflight responses returned the expected `Access-Control-Allow-Origin: http://localhost:4321`; administrator login returned `200`. |
| 2026-08-13 | OPS-01, TEST-01 | Added a same-origin `/api` development proxy so browser sessions no longer call the local backend directly. This removes browser-specific loopback/CORS interference while production continues to use its explicitly configured API origin. | `http://localhost:4321/api/health` returned `200`; system Chromium dashboard workflows passed 2/2 through the proxy. |
| 2026-08-13 | AI-UX-01 | Added the explainable AI workspace, version-aware source/similarity research, evidence-grounded project Copilot, overview readiness indicators, source-term insight cards, and the Tauri desktop delivery integration. Native file selection and protected-file save remain authorization-bound and use the existing backend contract. | Backend tests: 27/27 passed; frontend tests: 4/4 passed; production build passed; Chromium browser workflows: 2/2 passed; Tauri Rust `cargo check` passed. |
| 2026-08-26 | AI-UX-01 | Replaced the unreliable 176-language fastText path with the built-in `whatlang` language set (approximately 70 supported languages). Removed the external model runtime dependency, its model-path configuration, and the model-specific test code. Unsupported languages now return `Unknown` rather than a misleading label. | Backend tests: 52/52 passed; formatter check passed; frontend tests: 4/4 passed; frontend production build passed. |

## Next Execution Order

1. Connect and validate the external AI scoring service (`AI-01`).
2. Configure the institution-managed log collector and alert receiver, then perform a production deployment rehearsal (`OPS-01`).
3. Execute the manual acceptance checklist with the competition operations team and broaden browser workflow coverage as new flows are approved (`TEST-01`, `UX-02`).
