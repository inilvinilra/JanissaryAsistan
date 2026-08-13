# Emirhan Branch Feature Inventory and Integration Plan

## Purpose

This document records the useful capabilities found in the `emirhan-api` branch and defines how they should be integrated into the current enterprise dashboard.

The current dashboard already has stronger competition operations, jury workflows, authorization, auditability, security, reporting, and deployment foundations. The integration target is to strengthen the product's original value proposition: explainable AI-assisted project analysis.

## Architecture Findings

### `emirhan-api` branch

- Astro and React frontend under `kys-app`.
- Rust Axum analysis API under `kys-engine`.
- Tauri desktop application support.
- Supabase Auth and Supabase database integration.
- Direct OpenAI/OpenRouter scoring path.
- Serper-oriented related-source search settings.
- Frontend API calls currently depend on `localhost:8080` in several areas.

### Current dashboard

- Astro and React frontend under `frontend`.
- Rust Axum backend under `backend`.
- PostgreSQL-backed enterprise data model.
- Competition, category, stage, team, jury, appeal, calibration, audit, reporting, and operations workflows.
- Authenticated external AI adapter through `AI_SCORING_URL` and `AI_SCORING_TOKEN`.
- Docker, migrations, backups, monitoring, rate limiting, file security, and CI support.

## Features Found in Emirhan Branch

### AI analysis experience

- Category classification based on project content.
- Category-fit score.
- Technical-depth score.
- Completeness score.
- Reference-quality score.
- Originality score.
- AI-generated-content probability indicator.
- Overall score and letter grade.
- Score radar chart and metric bars.
- Natural-language reasoning for the score.
- Configurable system prompt.
- Configurable OpenAI/OpenRouter model settings.

### Similarity and source analysis

- Related-source search.
- Academic, PDF, GitHub, and web source classification.
- Keyword-based similarity calculation.
- Matched-keyword display.
- Similarity percentage and originality label.
- Source links in the project detail view.
- GitHub and PDF source warnings.
- Project-level similarity records.

### AI Copilot

- Chat-style project assistant in the project detail view.
- Project-specific context passed to the assistant.
- Questions about strengths, weaknesses, and project quality.
- Separate PDF analysis and Copilot tabs.

### Project review workspace

- PDF viewer.
- PDF word/term analysis.
- Project detail page.
- Recent projects table.
- Search and category filters.
- Score, grade, status, title, and word-count sorting.
- Manual drag-and-drop ranking.
- Bulk selection and deletion.
- XLSX export.
- Upload queue with staged files.
- Bulk PDF upload.
- Per-file category and title selection.
- Import and uploads pages.

### Dashboard and settings

- Total project count.
- Average score.
- Risk project count.
- Recent-period trend values.
- Daily project chart.
- Daily word-count chart.
- Category and AI criteria management.
- User profile management.
- Password update.
- Theme settings.
- OpenAI API key settings.
- Serper API key settings.
- System prompt settings.
- Evaluation category prompt management.

### Authentication and delivery model

- Supabase email/password registration.
- Email verification flow.
- Login and logout.
- Password reset.
- Supabase session guard.
- Tauri desktop packaging.
- Native desktop file selection and opening support.

## Features Already Present in Current Dashboard

The following Emirhan capabilities already exist in the current system, either directly or through a stronger implementation:

- Explainable AI evaluation fields.
- KPI-level scores and reasons.
- Evidence and source links.
- AI confidence score.
- Strengths, weaknesses, missing information, and risks.
- Similar-project records.
- File versions connected to AI evaluations.
- PDF, DOCX, XLSX, CSV, image, and OCR parsing support.
- Project search, filtering, score sorting, and manual ranking.
- XLSX and generated PDF reports.
- Category and KPI template management.
- Project detail evaluation workspace.
- Secure authentication, password reset, 2FA, and recovery codes.

## Integration Gaps to Close

### Highest priority: AI analysis visibility

- Make AI analysis the primary project-detail experience rather than a secondary panel.
- Add a clear AI summary card with total score, confidence, model version, and evaluation date.
- Add a KPI score breakdown with reason and evidence per KPI.
- Add prominent low-confidence and missing-information warnings.
- Add strengths, weaknesses, risks, and missing-information sections.
- Add source links and similar-project cards.
- Add an originality/similarity panel with explainable matched terms.

### AI Copilot

- Add a project-scoped Copilot panel to `ProjectDetailDialog`.
- Define a backend `/projects/{id}/copilot` contract instead of exposing provider keys to the browser.
- Include project text, KPI scores, evidence, risks, and jury notes in the controlled context.
- Log Copilot requests without storing unnecessary confidential content.
- Enforce the same competition, category, blind-review, and jury permissions as project details.

### Source and similarity analysis

- Reuse `backend/src/research.rs` as the source-search foundation.
- Add a versioned research result model tied to the evaluated document version.
- Store source URL, source type, matched terms, similarity score, and explanation.
- Keep source retrieval and external API calls server-side.
- Add timeout, rate limit, caching, and failure-state handling.
- Ensure similarity is advisory and never silently changes the jury score.

### Tauri desktop support

- Keep the current web dashboard as the primary product surface.
- Reuse or migrate the existing `kys-app/src-tauri` shell rather than maintaining a second business-logic implementation.
- Point the Tauri shell to the current frontend build and current backend/API contract.
- Add native file picker support for project uploads.
- Add native protected-file opening only through authorized backend responses.
- Define desktop update, configuration, logging, and offline behavior before release.
- Do not duplicate auth, scoring, authorization, or database logic inside Tauri.

## Recommended Implementation Order for Tomorrow

1. Confirm the current AI response contract and map every Emirhan score field to the current AI evaluation model.
2. Improve `ProjectDetailDialog` into the main AI analysis workspace.
3. Add the source/similarity view and document-version linkage.
4. Add the project-scoped AI Copilot backend contract.
5. Add the Copilot frontend panel with role and blind-review checks.
6. Add AI analysis overview cards and confidence/risk warnings to the dashboard overview.
7. Reconnect the existing Tauri shell to the current frontend and backend.
8. Add native upload and protected-file opening flows.
9. Run authorization, blind-review, file-security, AI-contract, and responsive browser tests.
10. Update the AI integration contract, test report, and critical-improvements register.

## Security and Architecture Rules

- Never expose OpenAI, OpenRouter, Serper, or model-service keys in frontend code.
- Never call third-party AI or search services directly from the browser.
- Every Copilot and research request must pass current authenticated-user and competition-scope checks.
- Blind-review users must not receive hidden project identity through AI context, source metadata, or Copilot responses.
- AI output is advisory; jury overrides remain explicit, visible, and auditable.
- AI model version, prompt/template version, source file version, and evaluation timestamp must be stored.
- Similarity scores must not be treated as plagiarism findings without jury review.
- Tauri must remain a delivery shell and must not become a second backend.

## Product Direction

The desired final product is not a choice between the two dashboards:

> Current dashboard enterprise foundation + Emirhan branch AI analysis experience + optional Tauri desktop shell.

The current implementation is ahead on institutional operations. The next development focus should make the AI analysis experience equally prominent and easier for jurors to understand and act on.

## Implementation Checklist

- [x] Feature inventory and architecture comparison completed.
- [x] AI analysis workspace added to the current project detail view.
- [x] AI score, confidence, KPI evidence, strengths, weaknesses, risks, missing information, and jury-score comparison made prominent.
- [x] Version-aware source and similarity analysis model, database migration, API routes, audit events, and frontend workspace added.
- [x] Similarity results kept advisory and labelled for jury review; no automatic plagiarism or score decision is made.
- [x] Project-scoped Copilot API and frontend panel added without exposing provider credentials.
- [x] Copilot and research access restricted to system administrators, competition managers, and chief judges to preserve blind-review policy.
- [x] Backend unit, frontend unit/build, migration, and live API verification completed for this implementation slice.
- [x] Add AI analysis summary and risk indicators to the dashboard overview without altering the existing navigation.
- [x] Add richer visual analysis cards: source totals, high-match count, document terms, and per-source matched terms, without changing the existing navigation structure.
- [x] Connect a least-privilege Tauri desktop shell to the current frontend/API architecture.
- [x] Add native upload and authorized protected-file opening flows through Tauri dialog and filesystem plugins.
- [x] Run the authorization, browser, responsive, frontend, backend, and desktop compilation verification suite for this completed slice.
- [ ] Validate live third-party source search with a configured research provider and an explicitly approved non-sensitive test submission.

## Verification Log

| Date | Slice | Result |
| --- | --- | --- |
| 2026-08-13 | AI workspace, source/similarity, and Copilot | Backend tests: 27/27 passed. Frontend tests: 4/4 passed. Production build passed. Schema migration `3|ai_analysis_workspace` applied. Live API test created an isolated parsed project, persisted research analysis, and returned a Copilot response. |
| 2026-08-13 | Overview AI readiness and Tauri shell | Frontend tests: 4/4 passed. Frontend production build passed. Tauri desktop build dependency resolution and Rust `cargo check` passed. The desktop shell uses the same Astro build and API contract, with narrowly scoped open/save dialog and file read/write capabilities. |
| 2026-08-13 | Visual analysis and native desktop files | Frontend tests: 4/4 passed. Backend tests: 27/27 passed. Browser workflow: 2/2 passed with login, logout, 1440px and 390px checks. Tauri `cargo check` passed with file-open/save-only dialog permissions and filesystem file operations. |
| 2026-08-13 | Desktop packaging and runtime recovery | The `Jury Assistant_0.1.0_amd64.deb` package and desktop executable were produced successfully. After package installation, a stale Vite optimized-dependency cache delayed React hydration; a clean dev-server restart resolved it. API and frontend health checks then returned `200`, and Playwright passed 2/2. |
