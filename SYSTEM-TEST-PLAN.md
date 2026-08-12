# System Test Plan

## Purpose

This checklist is the manual acceptance test plan for the Jury Assistant dashboard and API. Record the date, tester, environment, evidence link, and result for every executed row.

## Test Environments

| Environment | Required configuration |
| --- | --- |
| Development | API on `http://localhost:3000`, dashboard on `http://localhost:4321`, isolated PostgreSQL database. |
| Production-like | `APP_ENV=production`, valid 32-byte Base64 `FILE_ENCRYPTION_KEY`, `REQUIRE_TWO_FACTOR=true`, malware scanner available, sample data disabled. |

## Fast API Smoke Test

For an isolated account without mandatory two-factor enrollment, run:

```bash
API_URL=http://127.0.0.1:3000 \
JURY_TEST_EMAIL=your-test-account@example.test \
JURY_TEST_PASSWORD='your-test-password' \
bash backend/scripts/api-smoke-test.sh
```

The script checks health, anonymous denial, login, session validation, authenticated lists, metrics, logout, and session revocation. It does not mutate competition or project data.

For API authorization regression on an isolated development database, run:

```bash
API_URL=http://127.0.0.1:3000 \
JURY_TEST_ADMIN_EMAIL=admin@example.test \
JURY_TEST_ADMIN_PASSWORD='admin-test-password' \
bash backend/scripts/authorization-regression-test.sh
```

The script creates temporary jury and observer accounts, changes their required temporary passwords, and verifies both allowed and denied API routes. It intentionally leaves the accounts in the isolated test database for auditability; never run it against production.

For the browser sign-in/dashboard/sign-out test with a local Chromium binary, run:

```bash
cd frontend
E2E_BASE_URL=http://127.0.0.1:4321 \
PLAYWRIGHT_CHROMIUM_EXECUTABLE=/usr/bin/chromium \
JURY_E2E_EMAIL=your-test-account@example.test \
JURY_E2E_PASSWORD='your-test-password' \
npm run test:e2e
```

Start the API and dashboard before running this test. It verifies sign-in, hydrated dashboard load, sign-out, and the absence of horizontal overflow at 1440px and 390px viewport widths. Use an isolated account without mandatory TOTP enrollment, or extend the test with a test-only authenticator flow.

## Test Accounts

Create separate accounts for every role. Do not use real applicant data or production passwords.

| Role | Required scope |
| --- | --- |
| System administrator | Global access, two-factor authentication enabled. |
| Competition manager | Competition A only. |
| Chief judge | Competition A only. |
| Jury member | Competition A and a single category. |
| Observer | Competition A only. |
| Read-only user | Competition A only. |

## Authentication and Session Lifecycle

| ID | Method | Expected result | Result |
| --- | --- | --- | --- |
| AUTH-01 | Sign in with a valid email and password. | A server-side session is created and the dashboard opens. | ⬜ |
| AUTH-02 | Sign in with an invalid password. | `401`; no session is created. | ⬜ |
| AUTH-03 | Sign in as an inactive user. | `401`; no session is created. | ⬜ |
| AUTH-04 | Sign out from the sidebar and reload the page. | Session is revoked and the sign-in page remains visible. | ⬜ |
| AUTH-05 | Reuse a copied token after sign-out. | Protected API returns `401`. | ⬜ |
| AUTH-06 | Create a user with a temporary password and sign in. | Password-change screen blocks dashboard access. | ⬜ |
| AUTH-07 | Change the temporary password with an invalid current password. | Request fails; password remains unchanged. | ⬜ |
| AUTH-08 | Change the temporary password correctly. | Dashboard access is restored. | ⬜ |
| AUTH-09 | Issue a reset link as system administrator. | One-hour, one-time link is returned once. | ⬜ |
| AUTH-10 | Complete password reset, then try old password and reused link. | New password works; old password and reused link fail. | ⬜ |
| AUTH-11 | Enable TOTP and sign in using authenticator code. | Sign-in succeeds only with a valid code. | ⬜ |
| AUTH-12 | Sign in using a recovery code, then reuse it. | First sign-in succeeds; reuse fails. | ⬜ |
| AUTH-13 | In production-like mode, sign in as a privileged user without TOTP. | Enrollment screen is shown; dashboard API requests return `403`. | ⬜ |
| AUTH-14 | Complete mandatory TOTP enrollment and continue. | Recovery codes are shown once; dashboard opens. | ⬜ |

## Authorization, Scope, and Blind Review

| ID | Method | Expected result | Result |
| --- | --- | --- | --- |
| ACL-01 | Use jury account to open `/users`, `/audit`, and `/email-campaigns`. | Every request returns `403`. | ⬜ |
| ACL-02 | Use observer account to call the same endpoints. | Every request returns `403`. | ⬜ |
| ACL-03 | Use competition manager A to list Competition B or update its team/project. | Request returns `403`. | ⬜ |
| ACL-04 | Use category-scoped jury member to list projects in another category. | Request returns `403` or no out-of-scope data. | ⬜ |
| ACL-05 | Use jury member to read project metadata, documents, files, appeals, assignments, and AI payload. | All protected identity-bearing resources return `403`. | ⬜ |
| ACL-06 | Use jury member to view assigned project list and submit own score. | Anonymous project identity is returned; score submission succeeds. | ⬜ |
| ACL-07 | Use read-only user to attempt any POST, PATCH, or PUT request. | Request returns `403`. | ⬜ |
| ACL-08 | Use non-administrator to create a competition. | Request returns `403`. | ⬜ |
| ACL-09 | Use administrator to access notifications and email campaigns. | Access succeeds. | ⬜ |
| ACL-10 | Enable blind review in project details. | Team identity, project name, and juror identities are hidden. | ⬜ |

## Competition and Application Operations

| ID | Method | Expected result | Result |
| --- | --- | --- | --- |
| OPS-01 | Create a competition with application dates and organization. | Competition appears in the correct organization. | ⬜ |
| OPS-02 | Create parent and child competition categories. | Hierarchy and category KPI binding persist after reload. | ⬜ |
| OPS-03 | Add preliminary, technical, and final stages. | Position, dates, passing score, finalist limit, and status persist. | ⬜ |
| OPS-04 | Create a team, add members, and update team status. | Team and members appear only in the selected competition. | ⬜ |
| OPS-05 | Create a submission and upload a new submission version. | Version number increments and previous version remains available. | ⬜ |
| OPS-06 | Select finalists at a configured stage. | Only eligible projects within the limit become finalists. | ⬜ |
| OPS-07 | Create, check in, and update a Demo Day slot. | Check-in timestamp, checklist, evidence, score, and signature persist. | ⬜ |
| OPS-08 | Finalize a competition with signed minutes. | Results lock and the audit event records the action. | ⬜ |

## KPI, AI Contract, and Jury Evaluation

| ID | Method | Expected result | Result |
| --- | --- | --- | --- |
| SCORE-01 | Create KPI template whose weights total 100. | Save succeeds. | ⬜ |
| SCORE-02 | Attempt to save KPI weights not totaling 100. | Validation error is shown; existing template is unchanged. | ⬜ |
| SCORE-03 | Upload a project report with sufficient text. | Document extraction, KPI scoring, and project ranking are created. | ⬜ |
| SCORE-04 | Configure `AI_SCORING_URL` with a contract-compatible test service. | Complete 0–100 KPI response is stored. | ⬜ |
| SCORE-05 | Return an AI response with missing, duplicate, or out-of-range KPI values. | Evaluation is rejected; no partial score is stored. | ⬜ |
| SCORE-06 | Review AI strengths, risks, evidence, sources, similar projects, and confidence. | All fields render correctly; low confidence is visibly highlighted. | ⬜ |
| SCORE-07 | Submit jury scores from two jury members. | Average and score spread update; each score keeps its juror audit trail. | ⬜ |
| SCORE-08 | Drag projects to reorder ranking as chief judge. | New ordering persists after reload and appears in audit history. | ⬜ |
| SCORE-09 | Submit an appeal and resolve it as authorized staff. | Status, decision reason, old/new score, and audit entries persist. | ⬜ |
| SCORE-10 | Add calibration case and compare model/jury score. | Calibration summary reflects expected versus actual score. | ⬜ |

## Files, Privacy, and Security Controls

| ID | Method | Expected result | Result |
| --- | --- | --- | --- |
| FILE-01 | Upload PDF, DOCX, XLSX, CSV, PNG/JPG, and scanned PDF test fixtures. | Supported files are stored as versioned attachments and parsed where applicable. | ⬜ |
| FILE-02 | Upload unsupported extension and file exceeding configured limit. | Upload is rejected with a clear validation error. | ⬜ |
| FILE-03 | Run with scanner unavailable and `VIRUS_SCAN_REQUIRED=true`. | Upload is rejected (fail closed). | ⬜ |
| FILE-04 | Download a project file as authorized manager and as jury member. | Manager succeeds; jury member receives `403`. | ⬜ |
| FILE-05 | Verify database values for encrypted file/TOTP data in production-like mode. | Plain file bytes and raw TOTP secret are not stored. | ⬜ |
| FILE-06 | Send spoofed `X-Forwarded-For` header while proxy trust is disabled. | Rate-limit key uses direct peer address, not spoofed header. | ⬜ |
| FILE-07 | Send more than 120 requests in one minute from one client. | First 120 requests succeed as appropriate; next request returns `429`. | ⬜ |
| FILE-08 | Restart one backend instance after rate-limit activity. | Limit counter remains because it is stored in PostgreSQL. | ⬜ |

## Audit, Reporting, Notifications, and Delivery

| ID | Method | Expected result | Result |
| --- | --- | --- | --- |
| AUDIT-01 | Create/update project, score, assignment, ranking, user, password reset, and finalization. | Every action has actor, timestamp, entity, details, previous hash, and event hash. | ⬜ |
| AUDIT-02 | Attempt direct UPDATE or DELETE against `audit_events` with a database test account. | Database trigger rejects the mutation. | ⬜ |
| REPORT-01 | Export ranking report as XLSX. | Spreadsheet opens in Excel/LibreOffice with expected fields. | ⬜ |
| REPORT-02 | Export report as PDF. | Valid PDF contains metrics and ranked projects. | ⬜ |
| REPORT-03 | Verify competition summary, finalist count, category result, and jury consistency sections. | Values match dashboard source data. | ⬜ |
| COMMS-01 | Create notification as system administrator. | Notification is visible in the notification center. | ⬜ |
| COMMS-02 | Create and dispatch email campaign with unavailable webhook. | Campaign remains queued; retry count and error are persisted. | ⬜ |
| COMMS-03 | Restore webhook and process queue. | Delivery is marked sent once; no duplicate delivery appears. | ⬜ |
| UX-01 | Open the same dashboard data in two browser sessions, change a project or ranking in one session. | The second session refreshes visible data without a manual reload or polling delay. | ⬜ |
| UX-02 | Review every workspace at desktop, tablet, and mobile widths with each role. | Navigation, actions, focus order, and empty states remain understandable and no forbidden control is shown. | ⬜ |

## Deployment and Resilience

| ID | Method | Expected result | Result |
| --- | --- | --- | --- |
| DEPLOY-01 | Run `docker compose config --quiet` with all required variables. | Configuration succeeds. | ⬜ |
| DEPLOY-02 | Build backend and frontend Docker images. | Both images build without source-context bloat. | ⬜ |
| DEPLOY-03 | Start production stack with an empty database and bootstrap variables. | Exactly one initial administrator is created. | ⬜ |
| DEPLOY-04 | Restart production stack with bootstrap variables unchanged. | No duplicate administrator is created. | ⬜ |
| DEPLOY-05 | Start production backend without a valid encryption key. | Startup fails before serving requests. | ⬜ |
| DEPLOY-06 | Run CI verification workflow. | Backend tests and frontend build complete successfully. | ⬜ |
| DEPLOY-07 | Restore a backup to an isolated database and run health check. | Database is usable and audit chain remains readable. | ⬜ |
| DEPLOY-08 | Run two backend instances behind a load balancer and change a project through one instance. | Every subscribed dashboard receives the refresh event through PostgreSQL `LISTEN/NOTIFY`. | ⬜ |

## Evidence Template

```text
Test ID:
Date and environment:
Tester:
Result: Pass / Fail / Blocked
Evidence: Screenshot, API response, log excerpt, or report file path
Notes:
```
