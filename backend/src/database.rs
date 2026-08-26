use crate::models::{
    ActivityEntry, CategoryTemplate, Competition, CompetitionCategory, CompetitionStage,
    CompetitionStatus, Document, KpiScore, KpiTemplate, Project, ProjectStatus,
};
use anyhow::Result;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row, postgres::PgRow};

const BASELINE_MIGRATION_VERSION: i64 = 1;
const BASELINE_MIGRATION_NAME: &str = "baseline_schema";
const PERFORMANCE_INDEX_MIGRATION_VERSION: i64 = 2;
const PERFORMANCE_INDEX_MIGRATION_NAME: &str = "operational_indexes";
const AI_ANALYSIS_MIGRATION_VERSION: i64 = 3;
const AI_ANALYSIS_MIGRATION_NAME: &str = "ai_analysis_workspace";
const REPORT_TEMPLATE_MIGRATION_VERSION: i64 = 4;
const REPORT_TEMPLATE_MIGRATION_NAME: &str = "report_templates";
const ASSESSMENT_MIGRATION_VERSION: i64 = 5;
const ASSESSMENT_MIGRATION_NAME: &str = "project_assessment_analyses";
const PARTICIPANT_ACCESS_MIGRATION_VERSION: i64 = 6;
const PARTICIPANT_ACCESS_MIGRATION_NAME: &str = "participant_feedback_access";
const TWO_FACTOR_EXEMPTION_MIGRATION_VERSION: i64 = 7;
const TWO_FACTOR_EXEMPTION_MIGRATION_NAME: &str = "per_user_two_factor_exemption";

pub struct Database {
    pub pool: PgPool,
}

fn audit_event_hash(
    previous_hash: Option<&str>,
    action: &str,
    actor: &str,
    entity_id: Option<i32>,
    details: &str,
    created_at: &str,
) -> String {
    let payload = format!(
        "{}|{}|{}|{:?}|{}|{}",
        previous_hash.unwrap_or(""),
        action,
        actor,
        entity_id,
        details,
        created_at
    );
    Sha256::digest(payload.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await.map_err(|e| {
            anyhow::anyhow!("Database connection error: {}. Check DATABASE_URL.", e)
        })?;

        let db = Database { pool };
        db.run_migrations().await?;
        Ok(db)
    }

    async fn run_migrations(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version BIGINT PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
        )
        .execute(&self.pool)
        .await?;

        let applied_version: Option<i64> =
            sqlx::query_scalar("SELECT MAX(version) FROM schema_migrations")
                .fetch_one(&self.pool)
                .await?;
        if applied_version.is_some_and(|version| version > TWO_FACTOR_EXEMPTION_MIGRATION_VERSION) {
            anyhow::bail!("Database schema version is newer than this application supports");
        }
        let applied_version = applied_version.unwrap_or_default();
        if applied_version < BASELINE_MIGRATION_VERSION {
            self.apply_baseline_migration().await?;
            self.record_migration(BASELINE_MIGRATION_VERSION, BASELINE_MIGRATION_NAME)
                .await?;
        }
        if applied_version < PERFORMANCE_INDEX_MIGRATION_VERSION {
            self.apply_operational_indexes().await?;
            self.record_migration(
                PERFORMANCE_INDEX_MIGRATION_VERSION,
                PERFORMANCE_INDEX_MIGRATION_NAME,
            )
            .await?;
        }
        if applied_version < AI_ANALYSIS_MIGRATION_VERSION {
            self.apply_ai_analysis_workspace().await?;
            self.record_migration(AI_ANALYSIS_MIGRATION_VERSION, AI_ANALYSIS_MIGRATION_NAME)
                .await?;
        }
        if applied_version < REPORT_TEMPLATE_MIGRATION_VERSION {
            self.apply_report_templates().await?;
            self.record_migration(
                REPORT_TEMPLATE_MIGRATION_VERSION,
                REPORT_TEMPLATE_MIGRATION_NAME,
            )
            .await?;
        }
        if applied_version < ASSESSMENT_MIGRATION_VERSION {
            self.apply_project_assessment_analyses().await?;
            self.record_migration(ASSESSMENT_MIGRATION_VERSION, ASSESSMENT_MIGRATION_NAME)
                .await?;
        }
        if applied_version < PARTICIPANT_ACCESS_MIGRATION_VERSION {
            self.apply_participant_feedback_access().await?;
            self.record_migration(
                PARTICIPANT_ACCESS_MIGRATION_VERSION,
                PARTICIPANT_ACCESS_MIGRATION_NAME,
            )
            .await?;
        }
        if applied_version < TWO_FACTOR_EXEMPTION_MIGRATION_VERSION {
            self.apply_two_factor_exemptions().await?;
            self.record_migration(
                TWO_FACTOR_EXEMPTION_MIGRATION_VERSION,
                TWO_FACTOR_EXEMPTION_MIGRATION_NAME,
            )
            .await?;
        }
        Ok(())
    }

    async fn apply_two_factor_exemptions(&self) -> Result<()> {
        sqlx::query(
            "ALTER TABLE users ADD COLUMN IF NOT EXISTS two_factor_exempt BOOLEAN NOT NULL DEFAULT FALSE",
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn apply_project_assessment_analyses(&self) -> Result<()> {
        for statement in [
            "CREATE TABLE IF NOT EXISTS project_category_fit_analyses (
                project_id INTEGER PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
                source_file_version INTEGER,
                current_category_score DOUBLE PRECISION NOT NULL,
                recommended_category TEXT NOT NULL,
                recommended_category_score DOUBLE PRECISION NOT NULL,
                matched_terms JSONB NOT NULL DEFAULT '[]'::jsonb,
                requires_review BOOLEAN NOT NULL DEFAULT FALSE,
                analyzed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
            "CREATE TABLE IF NOT EXISTS project_similarity_analyses (
                project_id INTEGER PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
                source_file_version INTEGER,
                highest_similarity DOUBLE PRECISION NOT NULL,
                requires_review BOOLEAN NOT NULL DEFAULT FALSE,
                matches JSONB NOT NULL DEFAULT '[]'::jsonb,
                analyzed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )",
            "CREATE INDEX IF NOT EXISTS project_category_fit_review_idx ON project_category_fit_analyses (requires_review) WHERE requires_review = TRUE",
            "CREATE INDEX IF NOT EXISTS project_similarity_review_idx ON project_similarity_analyses (requires_review) WHERE requires_review = TRUE",
        ] {
            sqlx::query(statement).execute(&self.pool).await?;
        }
        Ok(())
    }

    async fn apply_participant_feedback_access(&self) -> Result<()> {
        for statement in [
            "ALTER TABLE users ADD COLUMN IF NOT EXISTS team_id INTEGER REFERENCES teams(id) ON DELETE SET NULL",
            "CREATE INDEX IF NOT EXISTS users_team_id_idx ON users (team_id) WHERE team_id IS NOT NULL",
        ] {
            sqlx::query(statement).execute(&self.pool).await?;
        }
        Ok(())
    }

    async fn apply_report_templates(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS report_templates (
                competition_id INTEGER PRIMARY KEY REFERENCES competitions(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                version INTEGER NOT NULL DEFAULT 1,
                expected_language TEXT NOT NULL DEFAULT 'Any',
                min_words BIGINT NOT NULL DEFAULT 0,
                max_words BIGINT NOT NULL DEFAULT 0,
                sections JSONB NOT NULL DEFAULT '[]'::jsonb,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_by TEXT NOT NULL DEFAULT ''
            )",
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn record_migration(&self, version: i64, name: &str) -> Result<()> {
        sqlx::query(
            "INSERT INTO schema_migrations (version, name) VALUES ($1, $2)
             ON CONFLICT (version) DO NOTHING",
        )
        .bind(version)
        .bind(name)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn apply_operational_indexes(&self) -> Result<()> {
        for statement in [
            "CREATE INDEX IF NOT EXISTS projects_competition_category_idx ON projects (competition_id, category)",
            "CREATE INDEX IF NOT EXISTS teams_competition_idx ON teams (competition_id)",
            "CREATE INDEX IF NOT EXISTS jury_scores_project_stage_idx ON jury_scores (project_id, stage_id)",
            "CREATE INDEX IF NOT EXISTS audit_events_created_at_idx ON audit_events (created_at DESC)",
            "CREATE INDEX IF NOT EXISTS auth_sessions_user_active_idx ON auth_sessions (user_id, expires_at) WHERE revoked_at IS NULL",
            "CREATE INDEX IF NOT EXISTS email_deliveries_ready_idx ON email_deliveries (status, next_attempt_at)",
        ] {
            sqlx::query(statement).execute(&self.pool).await?;
        }
        Ok(())
    }

    async fn apply_ai_analysis_workspace(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS project_research_analyses (
                project_id INTEGER PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
                source_file_version INTEGER,
                originality_score DOUBLE PRECISION NOT NULL,
                originality_label TEXT NOT NULL,
                query_terms JSONB NOT NULL DEFAULT '[]'::jsonb,
                sources JSONB NOT NULL DEFAULT '[]'::jsonb,
                analyzed_at TIMESTAMPTZ DEFAULT NOW()
            )",
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn apply_baseline_migration(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS projects (
                id            SERIAL PRIMARY KEY,
                name          TEXT NOT NULL,
                category      TEXT NOT NULL,
                manual_rank   INTEGER,
                created_at    TIMESTAMPTZ DEFAULT NOW()
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS kpi_scores (
                id          SERIAL PRIMARY KEY,
                project_id  INTEGER REFERENCES projects(id) ON DELETE CASCADE,
                name        TEXT NOT NULL,
                score       DOUBLE PRECISION NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS ranking_history (
                id              SERIAL PRIMARY KEY,
                project_id      INTEGER REFERENCES projects(id) ON DELETE CASCADE,
                previous_rank   INTEGER,
                new_rank        INTEGER NOT NULL,
                changed_by      TEXT,
                timestamp       TIMESTAMPTZ DEFAULT NOW()
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS activity_log (
                id          SERIAL PRIMARY KEY,
                action      TEXT NOT NULL,
                project_id  INTEGER,
                details     TEXT,
                timestamp   TIMESTAMPTZ DEFAULT NOW()
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS category_kpi_templates (
                id            SERIAL PRIMARY KEY,
                category      TEXT NOT NULL,
                kpi_name      TEXT NOT NULL,
                weight        DOUBLE PRECISION NOT NULL,
                description   TEXT NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS competitions (
                id                  SERIAL PRIMARY KEY,
                name                TEXT NOT NULL,
                description         TEXT NOT NULL DEFAULT '',
                application_start   TEXT,
                application_end     TEXT,
                status              TEXT NOT NULL DEFAULT 'draft',
                created_at          TIMESTAMPTZ DEFAULT NOW()
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("ALTER TABLE competitions ADD COLUMN IF NOT EXISTS organization TEXT NOT NULL DEFAULT 'T3 Foundation'").execute(&self.pool).await?;
        let default_competition_id: i32 = match sqlx::query_scalar("SELECT id FROM competitions ORDER BY id LIMIT 1")
            .fetch_optional(&self.pool)
            .await?
        {
            Some(id) => id,
            None => sqlx::query_scalar(
                "INSERT INTO competitions (name, description, organization) VALUES ('Legacy Intake', 'Backfilled projects from the initial deployment.', 'System') RETURNING id",
            )
            .fetch_one(&self.pool)
            .await?,
        };

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS competition_stages (
                id              SERIAL PRIMARY KEY,
                competition_id  INTEGER NOT NULL REFERENCES competitions(id) ON DELETE CASCADE,
                name            TEXT NOT NULL,
                stage_type      TEXT NOT NULL,
                position        INTEGER NOT NULL,
                starts_at       TEXT,
                ends_at         TEXT
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS competition_categories (
                id              SERIAL PRIMARY KEY,
                competition_id  INTEGER NOT NULL REFERENCES competitions(id) ON DELETE CASCADE,
                parent_id       INTEGER REFERENCES competition_categories(id) ON DELETE SET NULL,
                name            TEXT NOT NULL,
                slug            TEXT NOT NULL,
                kpi_category    TEXT,
                UNIQUE (competition_id, slug)
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("ALTER TABLE competition_stages ADD COLUMN IF NOT EXISTS passing_score DOUBLE PRECISION NOT NULL DEFAULT 0").execute(&self.pool).await?;
        sqlx::query(
            "ALTER TABLE competition_stages ADD COLUMN IF NOT EXISTS finalist_limit INTEGER",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("ALTER TABLE competition_stages ADD COLUMN IF NOT EXISTS results_at TEXT")
            .execute(&self.pool)
            .await?;
        sqlx::query("ALTER TABLE competition_stages ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'planned'").execute(&self.pool).await?;

        sqlx::query("ALTER TABLE projects ADD COLUMN IF NOT EXISTS document JSONB")
            .execute(&self.pool)
            .await?;
        sqlx::query("ALTER TABLE projects ADD COLUMN IF NOT EXISTS competition_id INTEGER REFERENCES competitions(id) ON DELETE RESTRICT")
            .execute(&self.pool)
            .await?;
        sqlx::query("UPDATE projects SET competition_id=$1 WHERE competition_id IS NULL")
            .bind(default_competition_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("ALTER TABLE projects ALTER COLUMN competition_id SET NOT NULL")
            .execute(&self.pool)
            .await?;
        sqlx::query("ALTER TABLE projects ADD COLUMN IF NOT EXISTS notes TEXT NOT NULL DEFAULT ''")
            .execute(&self.pool)
            .await?;
        sqlx::query("ALTER TABLE projects ADD COLUMN IF NOT EXISTS status TEXT NOT NULL DEFAULT 'reviewing'")
            .execute(&self.pool)
            .await?;
        sqlx::query("ALTER TABLE projects ADD COLUMN IF NOT EXISTS file_path TEXT")
            .execute(&self.pool)
            .await?;
        sqlx::query("ALTER TABLE projects ADD COLUMN IF NOT EXISTS review_completed BOOLEAN NOT NULL DEFAULT FALSE")
            .execute(&self.pool)
            .await?;
        sqlx::query(
            "ALTER TABLE projects ADD COLUMN IF NOT EXISTS tags JSONB NOT NULL DEFAULT '[]'::jsonb",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS project_metadata (
                project_id INTEGER PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
                institution TEXT NOT NULL DEFAULT '',
                keywords JSONB NOT NULL DEFAULT '[]'::jsonb,
                github_url TEXT,
                demo_url TEXT,
                prototype_description TEXT NOT NULL DEFAULT '',
                team_name TEXT NOT NULL DEFAULT '',
                team_members JSONB NOT NULL DEFAULT '[]'::jsonb,
                updated_at TIMESTAMPTZ DEFAULT NOW()
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("ALTER TABLE project_metadata ADD COLUMN IF NOT EXISTS team_name TEXT NOT NULL DEFAULT ''").execute(&self.pool).await?;
        sqlx::query("ALTER TABLE project_metadata ADD COLUMN IF NOT EXISTS team_members JSONB NOT NULL DEFAULT '[]'::jsonb").execute(&self.pool).await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS project_files (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                version INTEGER NOT NULL,
                file_name TEXT NOT NULL,
                mime_type TEXT NOT NULL DEFAULT 'application/octet-stream',
                size_bytes BIGINT NOT NULL,
                file_path TEXT NOT NULL,
                uploaded_at TIMESTAMPTZ DEFAULT NOW(),
                UNIQUE(project_id, version)
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS teams (
                id SERIAL PRIMARY KEY,
                competition_id INTEGER NOT NULL REFERENCES competitions(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'new',
                created_at TIMESTAMPTZ DEFAULT NOW()
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("ALTER TABLE projects ADD COLUMN IF NOT EXISTS team_id INTEGER REFERENCES teams(id) ON DELETE SET NULL")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS team_members (
                id SERIAL PRIMARY KEY,
                team_id INTEGER NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
                full_name TEXT NOT NULL,
                email TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'member',
                is_scholar BOOLEAN NOT NULL DEFAULT FALSE,
                birth_year INTEGER,
                education_level TEXT NOT NULL DEFAULT ''
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("ALTER TABLE team_members ADD COLUMN IF NOT EXISTS birth_year INTEGER")
            .execute(&self.pool)
            .await?;
        sqlx::query("ALTER TABLE team_members ADD COLUMN IF NOT EXISTS education_level TEXT NOT NULL DEFAULT ''").execute(&self.pool).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS submissions (
                id SERIAL PRIMARY KEY,
                team_id INTEGER NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
                stage_id INTEGER NOT NULL REFERENCES competition_stages(id) ON DELETE CASCADE,
                title TEXT NOT NULL,
                file_name TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'submitted',
                is_late BOOLEAN NOT NULL DEFAULT FALSE,
                submitted_at TIMESTAMPTZ DEFAULT NOW()
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("ALTER TABLE submissions ADD COLUMN IF NOT EXISTS is_late BOOLEAN NOT NULL DEFAULT FALSE")
            .execute(&self.pool).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS ai_evaluations (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL UNIQUE REFERENCES projects(id) ON DELETE CASCADE,
                model_version TEXT NOT NULL,
                total_score DOUBLE PRECISION NOT NULL,
                confidence DOUBLE PRECISION NOT NULL,
                source_file_version INTEGER,
                kpi_scores JSONB NOT NULL DEFAULT '[]'::jsonb,
                strengths JSONB NOT NULL DEFAULT '[]'::jsonb,
                weaknesses JSONB NOT NULL DEFAULT '[]'::jsonb,
                missing_information JSONB NOT NULL DEFAULT '[]'::jsonb,
                risks JSONB NOT NULL DEFAULT '[]'::jsonb,
                sources JSONB NOT NULL DEFAULT '[]'::jsonb,
                similar_projects JSONB NOT NULL DEFAULT '[]'::jsonb,
                evaluated_at TIMESTAMPTZ DEFAULT NOW()
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("ALTER TABLE ai_evaluations ADD COLUMN IF NOT EXISTS similar_projects JSONB NOT NULL DEFAULT '[]'::jsonb")
            .execute(&self.pool).await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS jury_scores (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                stage_id INTEGER REFERENCES competition_stages(id) ON DELETE SET NULL,
                juror_name TEXT NOT NULL,
                total_score DOUBLE PRECISION NOT NULL,
                kpi_scores JSONB NOT NULL DEFAULT '[]'::jsonb,
                notes TEXT NOT NULL DEFAULT '',
                submitted_at TIMESTAMPTZ DEFAULT NOW()
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("ALTER TABLE jury_scores ADD COLUMN IF NOT EXISTS stage_id INTEGER REFERENCES competition_stages(id) ON DELETE SET NULL")
            .execute(&self.pool).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS jury_assignments (
                id SERIAL PRIMARY KEY,
                project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                juror_name TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'juror',
                status TEXT NOT NULL DEFAULT 'assigned',
                conflict_declared BOOLEAN NOT NULL DEFAULT FALSE,
                assigned_at TIMESTAMPTZ DEFAULT NOW(),
                UNIQUE (project_id, juror_name)
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("ALTER TABLE jury_assignments ADD COLUMN IF NOT EXISTS category TEXT")
            .execute(&self.pool)
            .await?;
        sqlx::query("ALTER TABLE jury_assignments ADD COLUMN IF NOT EXISTS conflict_reason TEXT NOT NULL DEFAULT ''")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS audit_events (
                id SERIAL PRIMARY KEY,
                action TEXT NOT NULL,
                actor TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                entity_id INTEGER,
                details JSONB NOT NULL DEFAULT '{}'::jsonb,
                created_at TIMESTAMPTZ DEFAULT NOW()
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("ALTER TABLE audit_events ADD COLUMN IF NOT EXISTS previous_hash TEXT")
            .execute(&self.pool)
            .await?;
        sqlx::query(
            "ALTER TABLE ai_evaluations ADD COLUMN IF NOT EXISTS source_file_version INTEGER",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "ALTER TABLE audit_events ADD COLUMN IF NOT EXISTS event_hash TEXT NOT NULL DEFAULT ''",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE OR REPLACE FUNCTION prevent_audit_event_mutation() RETURNS trigger AS $$
             BEGIN RAISE EXCEPTION 'audit_events are append-only'; END; $$ LANGUAGE plpgsql",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("DROP TRIGGER IF EXISTS audit_events_append_only ON audit_events")
            .execute(&self.pool)
            .await?;
        sqlx::query(
            "CREATE TRIGGER audit_events_append_only BEFORE UPDATE OR DELETE ON audit_events
             FOR EACH ROW EXECUTE FUNCTION prevent_audit_event_mutation()",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS users (
                id SERIAL PRIMARY KEY,
                full_name TEXT NOT NULL,
                email TEXT NOT NULL UNIQUE,
                role TEXT NOT NULL DEFAULT 'observer',
                active BOOLEAN NOT NULL DEFAULT TRUE,
                competition_id INTEGER REFERENCES competitions(id) ON DELETE SET NULL,
                category TEXT,
                created_at TIMESTAMPTZ DEFAULT NOW()
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("ALTER TABLE users ADD COLUMN IF NOT EXISTS password_hash TEXT")
            .execute(&self.pool)
            .await?;
        sqlx::query("ALTER TABLE users ADD COLUMN IF NOT EXISTS two_factor_secret TEXT")
            .execute(&self.pool)
            .await?;
        sqlx::query("ALTER TABLE users ADD COLUMN IF NOT EXISTS two_factor_enabled BOOLEAN NOT NULL DEFAULT FALSE")
            .execute(&self.pool)
            .await?;
        sqlx::query("ALTER TABLE users ADD COLUMN IF NOT EXISTS must_change_password BOOLEAN NOT NULL DEFAULT FALSE")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS auth_sessions (token TEXT PRIMARY KEY, user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE, expires_at TIMESTAMPTZ NOT NULL, revoked_at TIMESTAMPTZ, created_at TIMESTAMPTZ DEFAULT NOW(), last_active_at TIMESTAMPTZ DEFAULT NOW())")
            .execute(&self.pool)
            .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS api_rate_limit_windows (
                client_key TEXT NOT NULL,
                window_started_at TIMESTAMPTZ NOT NULL,
                request_count INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (client_key, window_started_at)
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS password_reset_tokens (id SERIAL PRIMARY KEY, user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE, token_hash TEXT NOT NULL UNIQUE, expires_at TIMESTAMPTZ NOT NULL, used_at TIMESTAMPTZ, created_at TIMESTAMPTZ DEFAULT NOW())")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS two_factor_recovery_codes (id SERIAL PRIMARY KEY, user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE, code_hash TEXT NOT NULL, used_at TIMESTAMPTZ, created_at TIMESTAMPTZ DEFAULT NOW(), UNIQUE(user_id, code_hash))")
            .execute(&self.pool)
            .await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS juror_profiles (user_id INTEGER PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE, expertise JSONB NOT NULL DEFAULT '[]'::jsonb, institution TEXT NOT NULL DEFAULT '', max_assignments INTEGER NOT NULL DEFAULT 20)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS notifications (
                id SERIAL PRIMARY KEY,
                title TEXT NOT NULL,
                body TEXT NOT NULL,
                kind TEXT NOT NULL,
                audience TEXT NOT NULL DEFAULT 'all',
                category TEXT,
                created_at TIMESTAMPTZ DEFAULT NOW()
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS email_campaigns (
                id SERIAL PRIMARY KEY,
                subject TEXT NOT NULL,
                body TEXT NOT NULL,
                audience TEXT NOT NULL DEFAULT 'all',
                category TEXT,
                status TEXT NOT NULL DEFAULT 'queued',
                created_at TIMESTAMPTZ DEFAULT NOW()
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS email_deliveries (
                id SERIAL PRIMARY KEY,
                campaign_id INTEGER NOT NULL REFERENCES email_campaigns(id) ON DELETE CASCADE,
                user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                email TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'queued',
                attempt_count INTEGER NOT NULL DEFAULT 0,
                next_attempt_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                last_error TEXT,
                sent_at TIMESTAMPTZ,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                UNIQUE(campaign_id, user_id)
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("ALTER TABLE email_deliveries ADD COLUMN IF NOT EXISTS attempt_count INTEGER NOT NULL DEFAULT 0")
            .execute(&self.pool)
            .await?;
        sqlx::query("ALTER TABLE email_deliveries ADD COLUMN IF NOT EXISTS next_attempt_at TIMESTAMPTZ NOT NULL DEFAULT NOW()")
            .execute(&self.pool)
            .await?;
        sqlx::query("ALTER TABLE email_deliveries ADD COLUMN IF NOT EXISTS last_error TEXT")
            .execute(&self.pool)
            .await?;
        sqlx::query("ALTER TABLE email_deliveries ADD COLUMN IF NOT EXISTS sent_at TIMESTAMPTZ")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS demo_day_slots (
                id SERIAL PRIMARY KEY,
                competition_id INTEGER NOT NULL REFERENCES competitions(id) ON DELETE CASCADE,
                team_id INTEGER NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
                slot_order INTEGER NOT NULL,
                room TEXT NOT NULL DEFAULT '',
                starts_at TEXT NOT NULL,
                duration_minutes INTEGER NOT NULL DEFAULT 10,
                status TEXT NOT NULL DEFAULT 'scheduled', checked_in_at TIMESTAMPTZ, evidence_urls JSONB NOT NULL DEFAULT '[]'::jsonb, field_score DOUBLE PRECISION, jury_signature TEXT, check_in_token TEXT NOT NULL DEFAULT '', prototype_checklist JSONB NOT NULL DEFAULT '[]'::jsonb,
                UNIQUE (competition_id, slot_order)
            )",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query(
            "ALTER TABLE demo_day_slots ADD COLUMN IF NOT EXISTS checked_in_at TIMESTAMPTZ",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("ALTER TABLE demo_day_slots ADD COLUMN IF NOT EXISTS evidence_urls JSONB NOT NULL DEFAULT '[]'::jsonb").execute(&self.pool).await?;
        sqlx::query(
            "ALTER TABLE demo_day_slots ADD COLUMN IF NOT EXISTS field_score DOUBLE PRECISION",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("ALTER TABLE demo_day_slots ADD COLUMN IF NOT EXISTS jury_signature TEXT")
            .execute(&self.pool)
            .await?;
        sqlx::query("ALTER TABLE demo_day_slots ADD COLUMN IF NOT EXISTS check_in_token TEXT NOT NULL DEFAULT ''").execute(&self.pool).await?;
        sqlx::query("ALTER TABLE demo_day_slots ADD COLUMN IF NOT EXISTS prototype_checklist JSONB NOT NULL DEFAULT '[]'::jsonb").execute(&self.pool).await?;
        sqlx::query("ALTER TABLE competitions ADD COLUMN IF NOT EXISTS final_minutes TEXT NOT NULL DEFAULT ''").execute(&self.pool).await?;
        sqlx::query(
            "ALTER TABLE competitions ADD COLUMN IF NOT EXISTS results_locked_at TIMESTAMPTZ",
        )
        .execute(&self.pool)
        .await?;
        sqlx::query("ALTER TABLE competitions ADD COLUMN IF NOT EXISTS final_signed_by TEXT NOT NULL DEFAULT ''").execute(&self.pool).await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS submission_versions (
                id SERIAL PRIMARY KEY,
                submission_id INTEGER NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
                version INTEGER NOT NULL,
                file_name TEXT NOT NULL,
                file_path TEXT NOT NULL,
                uploaded_at TIMESTAMPTZ DEFAULT NOW(),
                UNIQUE (submission_id, version)
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query("CREATE TABLE IF NOT EXISTS appeals (id SERIAL PRIMARY KEY, project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE, submitted_by TEXT NOT NULL, reason TEXT NOT NULL, deadline TEXT, committee JSONB NOT NULL DEFAULT '[]'::jsonb, status TEXT NOT NULL DEFAULT 'submitted', decision_reason TEXT NOT NULL DEFAULT '', old_score DOUBLE PRECISION, new_score DOUBLE PRECISION, created_at TIMESTAMPTZ DEFAULT NOW(), resolved_at TIMESTAMPTZ)").execute(&self.pool).await?;
        sqlx::query("CREATE TABLE IF NOT EXISTS calibration_cases (id SERIAL PRIMARY KEY, project_id INTEGER NOT NULL UNIQUE REFERENCES projects(id) ON DELETE CASCADE, expected_score DOUBLE PRECISION NOT NULL, active BOOLEAN NOT NULL DEFAULT TRUE)").execute(&self.pool).await?;

        Ok(())
    }

    pub async fn list_competitions(&self) -> Result<Vec<Competition>> {
        let rows = sqlx::query(
            "SELECT id, name, description, application_start, application_end, status, organization
             FROM competitions ORDER BY id DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| Competition {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                application_start: row.get("application_start"),
                application_end: row.get("application_end"),
                status: CompetitionStatus::from_str(row.get::<String, _>("status").as_str()),
                organization: row.get("organization"),
            })
            .collect())
    }

    pub async fn create_competition(
        &self,
        name: &str,
        description: &str,
        application_start: Option<&str>,
        application_end: Option<&str>,
        organization: &str,
    ) -> Result<i32> {
        let row = sqlx::query(
            "INSERT INTO competitions (name, description, application_start, application_end, organization)
             VALUES ($1, $2, $3, $4, $5) RETURNING id",
        )
        .bind(name)
        .bind(description)
        .bind(application_start)
        .bind(application_end).bind(organization)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.get("id"))
    }

    pub async fn add_competition_stage(
        &self,
        competition_id: i32,
        name: &str,
        stage_type: &str,
        position: i32,
        starts_at: Option<&str>,
        ends_at: Option<&str>,
        passing_score: f64,
        finalist_limit: Option<i32>,
        results_at: Option<&str>,
    ) -> Result<CompetitionStage> {
        let row = sqlx::query(
            "INSERT INTO competition_stages
             (competition_id, name, stage_type, position, starts_at, ends_at, passing_score, finalist_limit, results_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             RETURNING id, competition_id, name, stage_type, position, starts_at, ends_at, passing_score, finalist_limit, results_at, status",
        )
        .bind(competition_id).bind(name).bind(stage_type).bind(position)
        .bind(starts_at).bind(ends_at).bind(passing_score).bind(finalist_limit).bind(results_at).fetch_one(&self.pool).await?;
        Ok(CompetitionStage {
            id: row.get("id"),
            competition_id: row.get("competition_id"),
            name: row.get("name"),
            stage_type: row.get("stage_type"),
            position: row.get("position"),
            starts_at: row.get("starts_at"),
            ends_at: row.get("ends_at"),
            passing_score: row.get("passing_score"),
            finalist_limit: row.get("finalist_limit"),
            results_at: row.get("results_at"),
            status: row.get("status"),
        })
    }

    pub async fn add_competition_category(
        &self,
        competition_id: i32,
        parent_id: Option<i32>,
        name: &str,
        slug: &str,
        kpi_category: Option<&str>,
    ) -> Result<CompetitionCategory> {
        let row = sqlx::query(
            "INSERT INTO competition_categories
             (competition_id, parent_id, name, slug, kpi_category)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, competition_id, parent_id, name, slug, kpi_category",
        )
        .bind(competition_id)
        .bind(parent_id)
        .bind(name)
        .bind(slug)
        .bind(kpi_category)
        .fetch_one(&self.pool)
        .await?;
        Ok(CompetitionCategory {
            id: row.get("id"),
            competition_id: row.get("competition_id"),
            parent_id: row.get("parent_id"),
            name: row.get("name"),
            slug: row.get("slug"),
            kpi_category: row.get("kpi_category"),
        })
    }

    pub async fn list_competition_stages(
        &self,
        competition_id: i32,
    ) -> Result<Vec<CompetitionStage>> {
        let rows = sqlx::query(
            "SELECT id, competition_id, name, stage_type, position, starts_at, ends_at, passing_score, finalist_limit, results_at, status
             FROM competition_stages WHERE competition_id = $1 ORDER BY position, id",
        ).bind(competition_id).fetch_all(&self.pool).await?;
        Ok(rows
            .into_iter()
            .map(|row| CompetitionStage {
                id: row.get("id"),
                competition_id: row.get("competition_id"),
                name: row.get("name"),
                stage_type: row.get("stage_type"),
                position: row.get("position"),
                starts_at: row.get("starts_at"),
                ends_at: row.get("ends_at"),
                passing_score: row.get("passing_score"),
                finalist_limit: row.get("finalist_limit"),
                results_at: row.get("results_at"),
                status: row.get("status"),
            })
            .collect())
    }

    pub async fn update_stage_status(
        &self,
        stage_id: i32,
        status: &str,
    ) -> Result<CompetitionStage> {
        let row = sqlx::query("UPDATE competition_stages SET status = $2 WHERE id = $1 RETURNING id, competition_id, name, stage_type, position, starts_at, ends_at, passing_score, finalist_limit, results_at, status")
            .bind(stage_id).bind(status).fetch_one(&self.pool).await?;
        Ok(CompetitionStage {
            id: row.get("id"),
            competition_id: row.get("competition_id"),
            name: row.get("name"),
            stage_type: row.get("stage_type"),
            position: row.get("position"),
            starts_at: row.get("starts_at"),
            ends_at: row.get("ends_at"),
            passing_score: row.get("passing_score"),
            finalist_limit: row.get("finalist_limit"),
            results_at: row.get("results_at"),
            status: row.get("status"),
        })
    }

    pub async fn list_competition_categories(
        &self,
        competition_id: i32,
    ) -> Result<Vec<CompetitionCategory>> {
        let rows = sqlx::query(
            "SELECT id, competition_id, parent_id, name, slug, kpi_category
             FROM competition_categories WHERE competition_id = $1 ORDER BY parent_id NULLS FIRST, name",
        ).bind(competition_id).fetch_all(&self.pool).await?;
        Ok(rows
            .into_iter()
            .map(|row| CompetitionCategory {
                id: row.get("id"),
                competition_id: row.get("competition_id"),
                parent_id: row.get("parent_id"),
                name: row.get("name"),
                slug: row.get("slug"),
                kpi_category: row.get("kpi_category"),
            })
            .collect())
    }

    pub async fn seed_kpi_templates(&self) -> Result<()> {
        let templates: Vec<(&str, Vec<(&str, f64, &str)>)> = vec![
            (
                "software",
                vec![
                    (
                        "Innovation",
                        30.0,
                        "Novelty of the approach compared to existing solutions",
                    ),
                    (
                        "Functionality",
                        35.0,
                        "Does the software work as intended, end to end",
                    ),
                    (
                        "Code Quality",
                        35.0,
                        "Structure, readability, and maintainability of the codebase",
                    ),
                ],
            ),
            (
                "technology",
                vec![
                    (
                        "Feasibility",
                        35.0,
                        "Can this be built and deployed with realistic resources",
                    ),
                    (
                        "Innovation",
                        30.0,
                        "Novelty of the approach compared to existing solutions",
                    ),
                    (
                        "Sustainability",
                        35.0,
                        "Long-term viability, cost, and environmental impact",
                    ),
                ],
            ),
            (
                "science",
                vec![
                    (
                        "Scientific Rigor",
                        40.0,
                        "Soundness of methodology and data analysis",
                    ),
                    (
                        "Originality",
                        30.0,
                        "Contribution beyond existing published work",
                    ),
                    ("Impact", 30.0, "Potential significance of the findings"),
                ],
            ),
            (
                "mathematics",
                vec![
                    (
                        "Rigor",
                        40.0,
                        "Correctness and completeness of proofs or derivations",
                    ),
                    ("Originality", 30.0, "Novelty of the approach or result"),
                    ("Clarity", 30.0, "How clearly the reasoning is presented"),
                ],
            ),
            (
                "physics",
                vec![
                    (
                        "Theoretical Soundness",
                        35.0,
                        "Consistency with established physical principles",
                    ),
                    (
                        "Experimental Validation",
                        35.0,
                        "Quality of supporting experiments or simulations",
                    ),
                    ("Originality", 30.0, "Novelty of the approach or result"),
                ],
            ),
            (
                "ai",
                vec![
                    (
                        "Model Performance",
                        40.0,
                        "Accuracy, robustness, and generalization of the model",
                    ),
                    (
                        "Data Quality & Ethics",
                        30.0,
                        "Soundness and ethical handling of training/evaluation data",
                    ),
                    (
                        "Innovation",
                        30.0,
                        "Novelty of the approach compared to existing solutions",
                    ),
                ],
            ),
            (
                "data-science",
                vec![
                    (
                        "Analytical Depth",
                        35.0,
                        "Rigor and thoroughness of the analysis",
                    ),
                    (
                        "Visualization & Communication",
                        30.0,
                        "How clearly findings are presented",
                    ),
                    (
                        "Methodology",
                        35.0,
                        "Soundness of the data pipeline and statistical methods",
                    ),
                ],
            ),
            (
                "health-tech",
                vec![
                    (
                        "Clinical Applicability",
                        40.0,
                        "Real-world usefulness in a healthcare setting",
                    ),
                    (
                        "Safety & Compliance",
                        35.0,
                        "Handling of patient data and regulatory considerations",
                    ),
                    (
                        "Innovation",
                        25.0,
                        "Novelty of the approach compared to existing solutions",
                    ),
                ],
            ),
            (
                "sustainability",
                vec![
                    (
                        "Environmental Impact",
                        40.0,
                        "Measurable positive effect on sustainability",
                    ),
                    (
                        "Feasibility",
                        30.0,
                        "Can this be built and deployed with realistic resources",
                    ),
                    (
                        "Innovation",
                        30.0,
                        "Novelty of the approach compared to existing solutions",
                    ),
                ],
            ),
            (
                "edtech",
                vec![
                    (
                        "Pedagogical Value",
                        40.0,
                        "Actual contribution to learning outcomes",
                    ),
                    (
                        "Accessibility",
                        30.0,
                        "Usability across different learners and contexts",
                    ),
                    (
                        "Innovation",
                        30.0,
                        "Novelty of the approach compared to existing solutions",
                    ),
                ],
            ),
            (
                "robotics",
                vec![
                    (
                        "Hardware Integration",
                        35.0,
                        "Quality of the physical/software integration",
                    ),
                    ("Autonomy", 35.0, "Degree of independent operation achieved"),
                    (
                        "Innovation",
                        30.0,
                        "Novelty of the approach compared to existing solutions",
                    ),
                ],
            ),
            (
                "cybersecurity",
                vec![
                    (
                        "Security Robustness",
                        45.0,
                        "Resistance to realistic attack scenarios",
                    ),
                    (
                        "Feasibility",
                        25.0,
                        "Can this be built and deployed with realistic resources",
                    ),
                    (
                        "Innovation",
                        30.0,
                        "Novelty of the approach compared to existing solutions",
                    ),
                ],
            ),
            (
                "odr",
                vec![
                    (
                        "Problem Definition",
                        35.0,
                        "Clarity and depth of the problem statement and needs analysis",
                    ),
                    (
                        "Solution Originality",
                        35.0,
                        "Novelty of the proposed approach at this early stage",
                    ),
                    (
                        "Team Readiness",
                        30.0,
                        "Team competency and planning maturity",
                    ),
                ],
            ),
            (
                "ktr",
                vec![
                    (
                        "Technical Design Maturity",
                        40.0,
                        "Depth and completeness of the technical design",
                    ),
                    (
                        "System Architecture",
                        30.0,
                        "Soundness of system architecture and component integration",
                    ),
                    (
                        "Validation Plan",
                        30.0,
                        "Quality of the test and verification plan",
                    ),
                ],
            ),
        ];

        for (category, kpis) in templates {
            let row = sqlx::query(
                "SELECT COUNT(*) as count FROM category_kpi_templates WHERE category = $1",
            )
            .bind(category)
            .fetch_one(&self.pool)
            .await?;
            let count: i64 = row.get("count");
            if count > 0 {
                continue;
            }

            for (kpi_name, weight, description) in kpis {
                sqlx::query(
                    "INSERT INTO category_kpi_templates (category, kpi_name, weight, description) VALUES ($1, $2, $3, $4)",
                )
                .bind(category)
                .bind(kpi_name)
                .bind(weight)
                .bind(description)
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }

    pub async fn list_categories(&self) -> Result<Vec<CategoryTemplate>> {
        let rows = sqlx::query(
            "SELECT category, kpi_name, weight, description
             FROM category_kpi_templates
             ORDER BY category, id",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut templates: Vec<CategoryTemplate> = Vec::new();

        for row in rows {
            let category: String = row.get("category");

            if templates.last().map(|t| &t.category) != Some(&category) {
                templates.push(CategoryTemplate {
                    category,
                    kpis: Vec::new(),
                });
            }

            templates.last_mut().unwrap().kpis.push(KpiTemplate {
                name: row.get("kpi_name"),
                weight: row.get("weight"),
                description: row.get("description"),
            });
        }

        Ok(templates)
    }

    pub async fn replace_kpi_template(&self, category: &str, kpis: &[KpiTemplate]) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM category_kpi_templates WHERE category = $1")
            .bind(category)
            .execute(&mut *tx)
            .await?;
        for kpi in kpis {
            sqlx::query(
                "INSERT INTO category_kpi_templates (category, kpi_name, weight, description)
                 VALUES ($1, $2, $3, $4)",
            )
            .bind(category)
            .bind(&kpi.name)
            .bind(kpi.weight)
            .bind(&kpi.description)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn seed_sample_data(&self) -> Result<()> {
        const TARGET: i64 = 10;

        let rows =
            sqlx::query("SELECT category, COUNT(*) as count FROM projects GROUP BY category")
                .fetch_all(&self.pool)
                .await?;
        let mut counts: std::collections::HashMap<String, i64> = rows
            .into_iter()
            .map(|r| (r.get::<String, _>("category"), r.get::<i64, _>("count")))
            .collect();

        let samples: Vec<(&str, &str, Vec<KpiScore>)> = vec![
            (
                "Smart Irrigation System",
                "technology",
                vec![
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 88.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 75.0,
                    },
                    KpiScore {
                        name: "Sustainability".into(),
                        score: 84.0,
                    },
                ],
            ),
            (
                "Cancer Cell Detection Model",
                "science",
                vec![
                    KpiScore {
                        name: "Scientific Rigor".into(),
                        score: 93.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 89.0,
                    },
                    KpiScore {
                        name: "Impact".into(),
                        score: 91.0,
                    },
                ],
            ),
            (
                "Blockchain-Based Voting",
                "software",
                vec![
                    KpiScore {
                        name: "Innovation".into(),
                        score: 70.0,
                    },
                    KpiScore {
                        name: "Functionality".into(),
                        score: 80.0,
                    },
                    KpiScore {
                        name: "Code Quality".into(),
                        score: 72.0,
                    },
                ],
            ),
            (
                "NLP-Based Summarization",
                "software",
                vec![
                    KpiScore {
                        name: "Innovation".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Functionality".into(),
                        score: 85.0,
                    },
                    KpiScore {
                        name: "Code Quality".into(),
                        score: 89.0,
                    },
                ],
            ),
            (
                "Prime Gap Distribution Analysis",
                "mathematics",
                vec![
                    KpiScore {
                        name: "Rigor".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 78.0,
                    },
                    KpiScore {
                        name: "Clarity".into(),
                        score: 82.0,
                    },
                ],
            ),
            (
                "Topological Data Analysis Toolkit",
                "mathematics",
                vec![
                    KpiScore {
                        name: "Rigor".into(),
                        score: 85.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 88.0,
                    },
                    KpiScore {
                        name: "Clarity".into(),
                        score: 75.0,
                    },
                ],
            ),
            (
                "Quantum Dot Solar Cell Efficiency Model",
                "physics",
                vec![
                    KpiScore {
                        name: "Theoretical Soundness".into(),
                        score: 88.0,
                    },
                    KpiScore {
                        name: "Experimental Validation".into(),
                        score: 82.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 79.0,
                    },
                ],
            ),
            (
                "Low-Cost Cosmic Ray Detector",
                "physics",
                vec![
                    KpiScore {
                        name: "Theoretical Soundness".into(),
                        score: 80.0,
                    },
                    KpiScore {
                        name: "Experimental Validation".into(),
                        score: 91.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 85.0,
                    },
                ],
            ),
            (
                "Turkish Sign Language Recognition",
                "ai",
                vec![
                    KpiScore {
                        name: "Model Performance".into(),
                        score: 87.0,
                    },
                    KpiScore {
                        name: "Data Quality & Ethics".into(),
                        score: 80.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 90.0,
                    },
                ],
            ),
            (
                "Crop Disease Detection via Drone Imagery",
                "ai",
                vec![
                    KpiScore {
                        name: "Model Performance".into(),
                        score: 91.0,
                    },
                    KpiScore {
                        name: "Data Quality & Ethics".into(),
                        score: 85.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 78.0,
                    },
                ],
            ),
            (
                "Urban Traffic Pattern Analysis",
                "data-science",
                vec![
                    KpiScore {
                        name: "Analytical Depth".into(),
                        score: 84.0,
                    },
                    KpiScore {
                        name: "Visualization & Communication".into(),
                        score: 89.0,
                    },
                    KpiScore {
                        name: "Methodology".into(),
                        score: 86.0,
                    },
                ],
            ),
            (
                "Public Health Dashboard for Regional Outbreaks",
                "data-science",
                vec![
                    KpiScore {
                        name: "Analytical Depth".into(),
                        score: 88.0,
                    },
                    KpiScore {
                        name: "Visualization & Communication".into(),
                        score: 92.0,
                    },
                    KpiScore {
                        name: "Methodology".into(),
                        score: 80.0,
                    },
                ],
            ),
            (
                "Wearable ECG Anomaly Alert System",
                "health-tech",
                vec![
                    KpiScore {
                        name: "Clinical Applicability".into(),
                        score: 85.0,
                    },
                    KpiScore {
                        name: "Safety & Compliance".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 83.0,
                    },
                ],
            ),
            (
                "AI-Assisted Diabetic Retinopathy Screening",
                "health-tech",
                vec![
                    KpiScore {
                        name: "Clinical Applicability".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Safety & Compliance".into(),
                        score: 87.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 88.0,
                    },
                ],
            ),
            (
                "Smart Water Recycling for Apartments",
                "sustainability",
                vec![
                    KpiScore {
                        name: "Environmental Impact".into(),
                        score: 89.0,
                    },
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 82.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 80.0,
                    },
                ],
            ),
            (
                "Biodegradable Packaging from Agricultural Waste",
                "sustainability",
                vec![
                    KpiScore {
                        name: "Environmental Impact".into(),
                        score: 92.0,
                    },
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 78.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 86.0,
                    },
                ],
            ),
            (
                "Adaptive Math Tutor for Middle Schoolers",
                "edtech",
                vec![
                    KpiScore {
                        name: "Pedagogical Value".into(),
                        score: 88.0,
                    },
                    KpiScore {
                        name: "Accessibility".into(),
                        score: 85.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 82.0,
                    },
                ],
            ),
            (
                "Sign Language Learning Game",
                "edtech",
                vec![
                    KpiScore {
                        name: "Pedagogical Value".into(),
                        score: 84.0,
                    },
                    KpiScore {
                        name: "Accessibility".into(),
                        score: 91.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 87.0,
                    },
                ],
            ),
            (
                "Autonomous Greenhouse Monitoring Rover",
                "robotics",
                vec![
                    KpiScore {
                        name: "Hardware Integration".into(),
                        score: 86.0,
                    },
                    KpiScore {
                        name: "Autonomy".into(),
                        score: 83.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 81.0,
                    },
                ],
            ),
            (
                "Modular Search-and-Rescue Robot Arm",
                "robotics",
                vec![
                    KpiScore {
                        name: "Hardware Integration".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Autonomy".into(),
                        score: 79.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 88.0,
                    },
                ],
            ),
            (
                "Phishing Detection Browser Extension",
                "cybersecurity",
                vec![
                    KpiScore {
                        name: "Security Robustness".into(),
                        score: 87.0,
                    },
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 79.0,
                    },
                ],
            ),
            (
                "IoT Device Firmware Integrity Checker",
                "cybersecurity",
                vec![
                    KpiScore {
                        name: "Security Robustness".into(),
                        score: 91.0,
                    },
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 84.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 85.0,
                    },
                ],
            ),
            (
                "Real-Time Collaborative Code Editor",
                "software",
                vec![
                    KpiScore {
                        name: "Innovation".into(),
                        score: 82.0,
                    },
                    KpiScore {
                        name: "Functionality".into(),
                        score: 78.0,
                    },
                    KpiScore {
                        name: "Code Quality".into(),
                        score: 85.0,
                    },
                ],
            ),
            (
                "Offline-First Note Taking App",
                "software",
                vec![
                    KpiScore {
                        name: "Innovation".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Functionality".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Code Quality".into(),
                        score: 80.0,
                    },
                ],
            ),
            (
                "Automated API Testing Framework",
                "software",
                vec![
                    KpiScore {
                        name: "Innovation".into(),
                        score: 88.0,
                    },
                    KpiScore {
                        name: "Functionality".into(),
                        score: 84.0,
                    },
                    KpiScore {
                        name: "Code Quality".into(),
                        score: 77.0,
                    },
                ],
            ),
            (
                "Peer-to-Peer File Sharing Client",
                "software",
                vec![
                    KpiScore {
                        name: "Innovation".into(),
                        score: 91.0,
                    },
                    KpiScore {
                        name: "Functionality".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Code Quality".into(),
                        score: 86.0,
                    },
                ],
            ),
            (
                "Voice-Controlled Task Manager",
                "software",
                vec![
                    KpiScore {
                        name: "Innovation".into(),
                        score: 79.0,
                    },
                    KpiScore {
                        name: "Functionality".into(),
                        score: 87.0,
                    },
                    KpiScore {
                        name: "Code Quality".into(),
                        score: 92.0,
                    },
                ],
            ),
            (
                "Static Site Generator for Blogs",
                "software",
                vec![
                    KpiScore {
                        name: "Innovation".into(),
                        score: 85.0,
                    },
                    KpiScore {
                        name: "Functionality".into(),
                        score: 81.0,
                    },
                    KpiScore {
                        name: "Code Quality".into(),
                        score: 89.0,
                    },
                ],
            ),
            (
                "Encrypted Messaging Client",
                "software",
                vec![
                    KpiScore {
                        name: "Innovation".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Functionality".into(),
                        score: 92.0,
                    },
                    KpiScore {
                        name: "Code Quality".into(),
                        score: 78.0,
                    },
                ],
            ),
            (
                "Open-Source Expense Tracker",
                "software",
                vec![
                    KpiScore {
                        name: "Innovation".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Functionality".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Code Quality".into(),
                        score: 83.0,
                    },
                ],
            ),
            (
                "Solar-Powered Water Purifier",
                "technology",
                vec![
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 82.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 78.0,
                    },
                    KpiScore {
                        name: "Sustainability".into(),
                        score: 85.0,
                    },
                ],
            ),
            (
                "Smart Traffic Light Controller",
                "technology",
                vec![
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Sustainability".into(),
                        score: 80.0,
                    },
                ],
            ),
            (
                "Home Energy Usage Monitor",
                "technology",
                vec![
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 88.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 84.0,
                    },
                    KpiScore {
                        name: "Sustainability".into(),
                        score: 77.0,
                    },
                ],
            ),
            (
                "Modular E-Bike Conversion Kit",
                "technology",
                vec![
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 91.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Sustainability".into(),
                        score: 86.0,
                    },
                ],
            ),
            (
                "Low-Power Mesh Network for Rural Areas",
                "technology",
                vec![
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 79.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 87.0,
                    },
                    KpiScore {
                        name: "Sustainability".into(),
                        score: 92.0,
                    },
                ],
            ),
            (
                "Automated Greenhouse Climate Control",
                "technology",
                vec![
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 85.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 81.0,
                    },
                    KpiScore {
                        name: "Sustainability".into(),
                        score: 89.0,
                    },
                ],
            ),
            (
                "Portable Air Quality Sensor",
                "technology",
                vec![
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 92.0,
                    },
                    KpiScore {
                        name: "Sustainability".into(),
                        score: 78.0,
                    },
                ],
            ),
            (
                "Smart Parking Space Finder",
                "technology",
                vec![
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Sustainability".into(),
                        score: 83.0,
                    },
                ],
            ),
            (
                "Microplastic Detection in Freshwater Samples",
                "science",
                vec![
                    KpiScore {
                        name: "Scientific Rigor".into(),
                        score: 82.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 78.0,
                    },
                    KpiScore {
                        name: "Impact".into(),
                        score: 85.0,
                    },
                ],
            ),
            (
                "Gut Microbiome Diversity in Urban Populations",
                "science",
                vec![
                    KpiScore {
                        name: "Scientific Rigor".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Impact".into(),
                        score: 80.0,
                    },
                ],
            ),
            (
                "Photocatalytic Degradation of Textile Dyes",
                "science",
                vec![
                    KpiScore {
                        name: "Scientific Rigor".into(),
                        score: 88.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 84.0,
                    },
                    KpiScore {
                        name: "Impact".into(),
                        score: 77.0,
                    },
                ],
            ),
            (
                "Machine Learning for Protein Folding Prediction",
                "science",
                vec![
                    KpiScore {
                        name: "Scientific Rigor".into(),
                        score: 91.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Impact".into(),
                        score: 86.0,
                    },
                ],
            ),
            (
                "Soil Nitrogen Fixation Rate Modeling",
                "science",
                vec![
                    KpiScore {
                        name: "Scientific Rigor".into(),
                        score: 79.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 87.0,
                    },
                    KpiScore {
                        name: "Impact".into(),
                        score: 92.0,
                    },
                ],
            ),
            (
                "Coral Bleaching Early Warning System",
                "science",
                vec![
                    KpiScore {
                        name: "Scientific Rigor".into(),
                        score: 85.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 81.0,
                    },
                    KpiScore {
                        name: "Impact".into(),
                        score: 89.0,
                    },
                ],
            ),
            (
                "Novel Antibiotic Resistance Gene Screening",
                "science",
                vec![
                    KpiScore {
                        name: "Scientific Rigor".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 92.0,
                    },
                    KpiScore {
                        name: "Impact".into(),
                        score: 78.0,
                    },
                ],
            ),
            (
                "Atmospheric Aerosol Impact on Cloud Formation",
                "science",
                vec![
                    KpiScore {
                        name: "Scientific Rigor".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Impact".into(),
                        score: 83.0,
                    },
                ],
            ),
            (
                "Biodegradable Enzyme-Based Water Filter",
                "science",
                vec![
                    KpiScore {
                        name: "Scientific Rigor".into(),
                        score: 82.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 78.0,
                    },
                    KpiScore {
                        name: "Impact".into(),
                        score: 85.0,
                    },
                ],
            ),
            (
                "Graph Coloring Algorithm for Scheduling",
                "mathematics",
                vec![
                    KpiScore {
                        name: "Rigor".into(),
                        score: 82.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 78.0,
                    },
                    KpiScore {
                        name: "Clarity".into(),
                        score: 85.0,
                    },
                ],
            ),
            (
                "Statistical Model for Epidemic Spread",
                "mathematics",
                vec![
                    KpiScore {
                        name: "Rigor".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Clarity".into(),
                        score: 80.0,
                    },
                ],
            ),
            (
                "Fractal Geometry in Urban Growth Patterns",
                "mathematics",
                vec![
                    KpiScore {
                        name: "Rigor".into(),
                        score: 88.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 84.0,
                    },
                    KpiScore {
                        name: "Clarity".into(),
                        score: 77.0,
                    },
                ],
            ),
            (
                "Optimization of Traffic Flow Networks",
                "mathematics",
                vec![
                    KpiScore {
                        name: "Rigor".into(),
                        score: 91.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Clarity".into(),
                        score: 86.0,
                    },
                ],
            ),
            (
                "Game-Theoretic Model of Resource Allocation",
                "mathematics",
                vec![
                    KpiScore {
                        name: "Rigor".into(),
                        score: 79.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 87.0,
                    },
                    KpiScore {
                        name: "Clarity".into(),
                        score: 92.0,
                    },
                ],
            ),
            (
                "Number-Theoretic Cryptographic Hash Function",
                "mathematics",
                vec![
                    KpiScore {
                        name: "Rigor".into(),
                        score: 85.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 81.0,
                    },
                    KpiScore {
                        name: "Clarity".into(),
                        score: 89.0,
                    },
                ],
            ),
            (
                "Machine Learning Bias in Statistical Sampling",
                "mathematics",
                vec![
                    KpiScore {
                        name: "Rigor".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 92.0,
                    },
                    KpiScore {
                        name: "Clarity".into(),
                        score: 78.0,
                    },
                ],
            ),
            (
                "Combinatorial Design for Tournament Scheduling",
                "mathematics",
                vec![
                    KpiScore {
                        name: "Rigor".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Clarity".into(),
                        score: 83.0,
                    },
                ],
            ),
            (
                "Acoustic Levitation for Material Handling",
                "physics",
                vec![
                    KpiScore {
                        name: "Theoretical Soundness".into(),
                        score: 82.0,
                    },
                    KpiScore {
                        name: "Experimental Validation".into(),
                        score: 78.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 85.0,
                    },
                ],
            ),
            (
                "Piezoelectric Energy Harvesting from Footsteps",
                "physics",
                vec![
                    KpiScore {
                        name: "Theoretical Soundness".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Experimental Validation".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 80.0,
                    },
                ],
            ),
            (
                "Magnetic Levitation Train Prototype",
                "physics",
                vec![
                    KpiScore {
                        name: "Theoretical Soundness".into(),
                        score: 88.0,
                    },
                    KpiScore {
                        name: "Experimental Validation".into(),
                        score: 84.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 77.0,
                    },
                ],
            ),
            (
                "Thin-Film Superconductor Characterization",
                "physics",
                vec![
                    KpiScore {
                        name: "Theoretical Soundness".into(),
                        score: 91.0,
                    },
                    KpiScore {
                        name: "Experimental Validation".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 86.0,
                    },
                ],
            ),
            (
                "Laser-Based Distance Measurement System",
                "physics",
                vec![
                    KpiScore {
                        name: "Theoretical Soundness".into(),
                        score: 79.0,
                    },
                    KpiScore {
                        name: "Experimental Validation".into(),
                        score: 87.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 92.0,
                    },
                ],
            ),
            (
                "Thermoelectric Generator for Waste Heat Recovery",
                "physics",
                vec![
                    KpiScore {
                        name: "Theoretical Soundness".into(),
                        score: 85.0,
                    },
                    KpiScore {
                        name: "Experimental Validation".into(),
                        score: 81.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 89.0,
                    },
                ],
            ),
            (
                "Optical Tweezers for Cell Manipulation",
                "physics",
                vec![
                    KpiScore {
                        name: "Theoretical Soundness".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Experimental Validation".into(),
                        score: 92.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 78.0,
                    },
                ],
            ),
            (
                "Low-Frequency Gravitational Wave Simulation",
                "physics",
                vec![
                    KpiScore {
                        name: "Theoretical Soundness".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Experimental Validation".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Originality".into(),
                        score: 83.0,
                    },
                ],
            ),
            (
                "Real-Time Traffic Sign Recognition",
                "ai",
                vec![
                    KpiScore {
                        name: "Model Performance".into(),
                        score: 82.0,
                    },
                    KpiScore {
                        name: "Data Quality & Ethics".into(),
                        score: 78.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 85.0,
                    },
                ],
            ),
            (
                "AI-Generated Turkish Poetry Model",
                "ai",
                vec![
                    KpiScore {
                        name: "Model Performance".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Data Quality & Ethics".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 80.0,
                    },
                ],
            ),
            (
                "Fraud Detection in Mobile Payments",
                "ai",
                vec![
                    KpiScore {
                        name: "Model Performance".into(),
                        score: 88.0,
                    },
                    KpiScore {
                        name: "Data Quality & Ethics".into(),
                        score: 84.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 77.0,
                    },
                ],
            ),
            (
                "Speech Emotion Recognition System",
                "ai",
                vec![
                    KpiScore {
                        name: "Model Performance".into(),
                        score: 91.0,
                    },
                    KpiScore {
                        name: "Data Quality & Ethics".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 86.0,
                    },
                ],
            ),
            (
                "Automated Resume Screening Tool",
                "ai",
                vec![
                    KpiScore {
                        name: "Model Performance".into(),
                        score: 79.0,
                    },
                    KpiScore {
                        name: "Data Quality & Ethics".into(),
                        score: 87.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 92.0,
                    },
                ],
            ),
            (
                "Wildlife Species Classification from Camera Traps",
                "ai",
                vec![
                    KpiScore {
                        name: "Model Performance".into(),
                        score: 85.0,
                    },
                    KpiScore {
                        name: "Data Quality & Ethics".into(),
                        score: 81.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 89.0,
                    },
                ],
            ),
            (
                "Predictive Maintenance for Industrial Equipment",
                "ai",
                vec![
                    KpiScore {
                        name: "Model Performance".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Data Quality & Ethics".into(),
                        score: 92.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 78.0,
                    },
                ],
            ),
            (
                "AI-Powered Turkish Grammar Checker",
                "ai",
                vec![
                    KpiScore {
                        name: "Model Performance".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Data Quality & Ethics".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 83.0,
                    },
                ],
            ),
            (
                "E-Commerce Customer Churn Prediction",
                "data-science",
                vec![
                    KpiScore {
                        name: "Analytical Depth".into(),
                        score: 82.0,
                    },
                    KpiScore {
                        name: "Visualization & Communication".into(),
                        score: 78.0,
                    },
                    KpiScore {
                        name: "Methodology".into(),
                        score: 85.0,
                    },
                ],
            ),
            (
                "Social Media Sentiment During Elections",
                "data-science",
                vec![
                    KpiScore {
                        name: "Analytical Depth".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Visualization & Communication".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Methodology".into(),
                        score: 80.0,
                    },
                ],
            ),
            (
                "Energy Consumption Forecasting Dashboard",
                "data-science",
                vec![
                    KpiScore {
                        name: "Analytical Depth".into(),
                        score: 88.0,
                    },
                    KpiScore {
                        name: "Visualization & Communication".into(),
                        score: 84.0,
                    },
                    KpiScore {
                        name: "Methodology".into(),
                        score: 77.0,
                    },
                ],
            ),
            (
                "Sports Performance Analytics Platform",
                "data-science",
                vec![
                    KpiScore {
                        name: "Analytical Depth".into(),
                        score: 91.0,
                    },
                    KpiScore {
                        name: "Visualization & Communication".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Methodology".into(),
                        score: 86.0,
                    },
                ],
            ),
            (
                "Real Estate Price Trend Analysis",
                "data-science",
                vec![
                    KpiScore {
                        name: "Analytical Depth".into(),
                        score: 79.0,
                    },
                    KpiScore {
                        name: "Visualization & Communication".into(),
                        score: 87.0,
                    },
                    KpiScore {
                        name: "Methodology".into(),
                        score: 92.0,
                    },
                ],
            ),
            (
                "Air Pollution Source Attribution Model",
                "data-science",
                vec![
                    KpiScore {
                        name: "Analytical Depth".into(),
                        score: 85.0,
                    },
                    KpiScore {
                        name: "Visualization & Communication".into(),
                        score: 81.0,
                    },
                    KpiScore {
                        name: "Methodology".into(),
                        score: 89.0,
                    },
                ],
            ),
            (
                "Student Performance Prediction System",
                "data-science",
                vec![
                    KpiScore {
                        name: "Analytical Depth".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Visualization & Communication".into(),
                        score: 92.0,
                    },
                    KpiScore {
                        name: "Methodology".into(),
                        score: 78.0,
                    },
                ],
            ),
            (
                "Supply Chain Bottleneck Visualization",
                "data-science",
                vec![
                    KpiScore {
                        name: "Analytical Depth".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Visualization & Communication".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Methodology".into(),
                        score: 83.0,
                    },
                ],
            ),
            (
                "Remote Physical Therapy Monitoring App",
                "health-tech",
                vec![
                    KpiScore {
                        name: "Clinical Applicability".into(),
                        score: 82.0,
                    },
                    KpiScore {
                        name: "Safety & Compliance".into(),
                        score: 78.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 85.0,
                    },
                ],
            ),
            (
                "AI Chatbot for Mental Health Support",
                "health-tech",
                vec![
                    KpiScore {
                        name: "Clinical Applicability".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Safety & Compliance".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 80.0,
                    },
                ],
            ),
            (
                "Portable Ultrasound Image Enhancement",
                "health-tech",
                vec![
                    KpiScore {
                        name: "Clinical Applicability".into(),
                        score: 88.0,
                    },
                    KpiScore {
                        name: "Safety & Compliance".into(),
                        score: 84.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 77.0,
                    },
                ],
            ),
            (
                "Medication Adherence Reminder System",
                "health-tech",
                vec![
                    KpiScore {
                        name: "Clinical Applicability".into(),
                        score: 91.0,
                    },
                    KpiScore {
                        name: "Safety & Compliance".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 86.0,
                    },
                ],
            ),
            (
                "Sleep Apnea Detection Wearable",
                "health-tech",
                vec![
                    KpiScore {
                        name: "Clinical Applicability".into(),
                        score: 79.0,
                    },
                    KpiScore {
                        name: "Safety & Compliance".into(),
                        score: 87.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 92.0,
                    },
                ],
            ),
            (
                "Telemedicine Triage Assistant",
                "health-tech",
                vec![
                    KpiScore {
                        name: "Clinical Applicability".into(),
                        score: 85.0,
                    },
                    KpiScore {
                        name: "Safety & Compliance".into(),
                        score: 81.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 89.0,
                    },
                ],
            ),
            (
                "Fall Detection System for Elderly",
                "health-tech",
                vec![
                    KpiScore {
                        name: "Clinical Applicability".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Safety & Compliance".into(),
                        score: 92.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 78.0,
                    },
                ],
            ),
            (
                "Personalized Nutrition Recommendation Engine",
                "health-tech",
                vec![
                    KpiScore {
                        name: "Clinical Applicability".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Safety & Compliance".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 83.0,
                    },
                ],
            ),
            (
                "Vertical Farming Nutrient Optimization",
                "sustainability",
                vec![
                    KpiScore {
                        name: "Environmental Impact".into(),
                        score: 82.0,
                    },
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 78.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 85.0,
                    },
                ],
            ),
            (
                "Community Solar Sharing Platform",
                "sustainability",
                vec![
                    KpiScore {
                        name: "Environmental Impact".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 80.0,
                    },
                ],
            ),
            (
                "Plastic Waste Sorting Robot",
                "sustainability",
                vec![
                    KpiScore {
                        name: "Environmental Impact".into(),
                        score: 88.0,
                    },
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 84.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 77.0,
                    },
                ],
            ),
            (
                "Rainwater Harvesting Smart Controller",
                "sustainability",
                vec![
                    KpiScore {
                        name: "Environmental Impact".into(),
                        score: 91.0,
                    },
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 86.0,
                    },
                ],
            ),
            (
                "Carbon Footprint Tracking App",
                "sustainability",
                vec![
                    KpiScore {
                        name: "Environmental Impact".into(),
                        score: 79.0,
                    },
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 87.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 92.0,
                    },
                ],
            ),
            (
                "Upcycled Construction Material from Waste",
                "sustainability",
                vec![
                    KpiScore {
                        name: "Environmental Impact".into(),
                        score: 85.0,
                    },
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 81.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 89.0,
                    },
                ],
            ),
            (
                "Electric Vehicle Charging Load Balancer",
                "sustainability",
                vec![
                    KpiScore {
                        name: "Environmental Impact".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 92.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 78.0,
                    },
                ],
            ),
            (
                "Reforestation Drone Seed Planter",
                "sustainability",
                vec![
                    KpiScore {
                        name: "Environmental Impact".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 83.0,
                    },
                ],
            ),
            (
                "Gamified Coding Curriculum for Kids",
                "edtech",
                vec![
                    KpiScore {
                        name: "Pedagogical Value".into(),
                        score: 82.0,
                    },
                    KpiScore {
                        name: "Accessibility".into(),
                        score: 78.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 85.0,
                    },
                ],
            ),
            (
                "AI-Powered Essay Feedback Tool",
                "edtech",
                vec![
                    KpiScore {
                        name: "Pedagogical Value".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Accessibility".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 80.0,
                    },
                ],
            ),
            (
                "Virtual Science Lab Simulator",
                "edtech",
                vec![
                    KpiScore {
                        name: "Pedagogical Value".into(),
                        score: 88.0,
                    },
                    KpiScore {
                        name: "Accessibility".into(),
                        score: 84.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 77.0,
                    },
                ],
            ),
            (
                "Peer Tutoring Matchmaking Platform",
                "edtech",
                vec![
                    KpiScore {
                        name: "Pedagogical Value".into(),
                        score: 91.0,
                    },
                    KpiScore {
                        name: "Accessibility".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 86.0,
                    },
                ],
            ),
            (
                "Speech Therapy Practice App",
                "edtech",
                vec![
                    KpiScore {
                        name: "Pedagogical Value".into(),
                        score: 79.0,
                    },
                    KpiScore {
                        name: "Accessibility".into(),
                        score: 87.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 92.0,
                    },
                ],
            ),
            (
                "Interactive History Timeline Builder",
                "edtech",
                vec![
                    KpiScore {
                        name: "Pedagogical Value".into(),
                        score: 85.0,
                    },
                    KpiScore {
                        name: "Accessibility".into(),
                        score: 81.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 89.0,
                    },
                ],
            ),
            (
                "Accessible Braille Learning Device",
                "edtech",
                vec![
                    KpiScore {
                        name: "Pedagogical Value".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Accessibility".into(),
                        score: 92.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 78.0,
                    },
                ],
            ),
            (
                "Classroom Engagement Analytics Tool",
                "edtech",
                vec![
                    KpiScore {
                        name: "Pedagogical Value".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Accessibility".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 83.0,
                    },
                ],
            ),
            (
                "Warehouse Inventory Scanning Robot",
                "robotics",
                vec![
                    KpiScore {
                        name: "Hardware Integration".into(),
                        score: 82.0,
                    },
                    KpiScore {
                        name: "Autonomy".into(),
                        score: 78.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 85.0,
                    },
                ],
            ),
            (
                "Bipedal Balance Control Algorithm",
                "robotics",
                vec![
                    KpiScore {
                        name: "Hardware Integration".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Autonomy".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 80.0,
                    },
                ],
            ),
            (
                "Underwater Pipeline Inspection Robot",
                "robotics",
                vec![
                    KpiScore {
                        name: "Hardware Integration".into(),
                        score: 88.0,
                    },
                    KpiScore {
                        name: "Autonomy".into(),
                        score: 84.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 77.0,
                    },
                ],
            ),
            (
                "Robotic Arm for Assistive Feeding",
                "robotics",
                vec![
                    KpiScore {
                        name: "Hardware Integration".into(),
                        score: 91.0,
                    },
                    KpiScore {
                        name: "Autonomy".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 86.0,
                    },
                ],
            ),
            (
                "Swarm Robotics for Crop Monitoring",
                "robotics",
                vec![
                    KpiScore {
                        name: "Hardware Integration".into(),
                        score: 79.0,
                    },
                    KpiScore {
                        name: "Autonomy".into(),
                        score: 87.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 92.0,
                    },
                ],
            ),
            (
                "Autonomous Lawn Mowing Robot",
                "robotics",
                vec![
                    KpiScore {
                        name: "Hardware Integration".into(),
                        score: 85.0,
                    },
                    KpiScore {
                        name: "Autonomy".into(),
                        score: 81.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 89.0,
                    },
                ],
            ),
            (
                "Robotic Exoskeleton for Rehabilitation",
                "robotics",
                vec![
                    KpiScore {
                        name: "Hardware Integration".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Autonomy".into(),
                        score: 92.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 78.0,
                    },
                ],
            ),
            (
                "Drone-Based Package Delivery System",
                "robotics",
                vec![
                    KpiScore {
                        name: "Hardware Integration".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Autonomy".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 83.0,
                    },
                ],
            ),
            (
                "Ransomware Behavior Detection System",
                "cybersecurity",
                vec![
                    KpiScore {
                        name: "Security Robustness".into(),
                        score: 82.0,
                    },
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 78.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 85.0,
                    },
                ],
            ),
            (
                "Secure Password Manager with Biometrics",
                "cybersecurity",
                vec![
                    KpiScore {
                        name: "Security Robustness".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 80.0,
                    },
                ],
            ),
            (
                "Network Intrusion Detection Dashboard",
                "cybersecurity",
                vec![
                    KpiScore {
                        name: "Security Robustness".into(),
                        score: 88.0,
                    },
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 84.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 77.0,
                    },
                ],
            ),
            (
                "Smart Contract Vulnerability Scanner",
                "cybersecurity",
                vec![
                    KpiScore {
                        name: "Security Robustness".into(),
                        score: 91.0,
                    },
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 86.0,
                    },
                ],
            ),
            (
                "Deepfake Detection Tool",
                "cybersecurity",
                vec![
                    KpiScore {
                        name: "Security Robustness".into(),
                        score: 79.0,
                    },
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 87.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 92.0,
                    },
                ],
            ),
            (
                "Zero-Trust Access Control Prototype",
                "cybersecurity",
                vec![
                    KpiScore {
                        name: "Security Robustness".into(),
                        score: 85.0,
                    },
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 81.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 89.0,
                    },
                ],
            ),
            (
                "Encrypted File Sharing for Teams",
                "cybersecurity",
                vec![
                    KpiScore {
                        name: "Security Robustness".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 92.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 78.0,
                    },
                ],
            ),
            (
                "Social Engineering Awareness Training Platform",
                "cybersecurity",
                vec![
                    KpiScore {
                        name: "Security Robustness".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Feasibility".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Innovation".into(),
                        score: 83.0,
                    },
                ],
            ),
            (
                "Autonomous Delivery Drone Concept",
                "odr",
                vec![
                    KpiScore {
                        name: "Problem Definition".into(),
                        score: 82.0,
                    },
                    KpiScore {
                        name: "Solution Originality".into(),
                        score: 78.0,
                    },
                    KpiScore {
                        name: "Team Readiness".into(),
                        score: 85.0,
                    },
                ],
            ),
            (
                "Smart Prosthetic Hand Proposal",
                "odr",
                vec![
                    KpiScore {
                        name: "Problem Definition".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Solution Originality".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Team Readiness".into(),
                        score: 80.0,
                    },
                ],
            ),
            (
                "Flood Early Warning Network",
                "odr",
                vec![
                    KpiScore {
                        name: "Problem Definition".into(),
                        score: 88.0,
                    },
                    KpiScore {
                        name: "Solution Originality".into(),
                        score: 84.0,
                    },
                    KpiScore {
                        name: "Team Readiness".into(),
                        score: 77.0,
                    },
                ],
            ),
            (
                "Campus Waste Sorting Initiative",
                "odr",
                vec![
                    KpiScore {
                        name: "Problem Definition".into(),
                        score: 91.0,
                    },
                    KpiScore {
                        name: "Solution Originality".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Team Readiness".into(),
                        score: 86.0,
                    },
                ],
            ),
            (
                "Assistive Reading Device for the Visually Impaired",
                "odr",
                vec![
                    KpiScore {
                        name: "Problem Definition".into(),
                        score: 79.0,
                    },
                    KpiScore {
                        name: "Solution Originality".into(),
                        score: 87.0,
                    },
                    KpiScore {
                        name: "Team Readiness".into(),
                        score: 92.0,
                    },
                ],
            ),
            (
                "Rural Telehealth Kiosk",
                "odr",
                vec![
                    KpiScore {
                        name: "Problem Definition".into(),
                        score: 85.0,
                    },
                    KpiScore {
                        name: "Solution Originality".into(),
                        score: 81.0,
                    },
                    KpiScore {
                        name: "Team Readiness".into(),
                        score: 89.0,
                    },
                ],
            ),
            (
                "Wildfire Detection Balloon System",
                "odr",
                vec![
                    KpiScore {
                        name: "Problem Definition".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Solution Originality".into(),
                        score: 92.0,
                    },
                    KpiScore {
                        name: "Team Readiness".into(),
                        score: 78.0,
                    },
                ],
            ),
            (
                "Low-Cost Water Desalination Unit",
                "odr",
                vec![
                    KpiScore {
                        name: "Problem Definition".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Solution Originality".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Team Readiness".into(),
                        score: 83.0,
                    },
                ],
            ),
            (
                "Emergency Response Coordination App",
                "odr",
                vec![
                    KpiScore {
                        name: "Problem Definition".into(),
                        score: 84.0,
                    },
                    KpiScore {
                        name: "Solution Originality".into(),
                        score: 88.0,
                    },
                    KpiScore {
                        name: "Team Readiness".into(),
                        score: 79.0,
                    },
                ],
            ),
            (
                "Solar-Powered Irrigation Drone",
                "odr",
                vec![
                    KpiScore {
                        name: "Problem Definition".into(),
                        score: 87.0,
                    },
                    KpiScore {
                        name: "Solution Originality".into(),
                        score: 82.0,
                    },
                    KpiScore {
                        name: "Team Readiness".into(),
                        score: 91.0,
                    },
                ],
            ),
            (
                "UAV Flight Control System — Design Review",
                "ktr",
                vec![
                    KpiScore {
                        name: "Technical Design Maturity".into(),
                        score: 82.0,
                    },
                    KpiScore {
                        name: "System Architecture".into(),
                        score: 78.0,
                    },
                    KpiScore {
                        name: "Validation Plan".into(),
                        score: 85.0,
                    },
                ],
            ),
            (
                "Underwater ROV Structural Design",
                "ktr",
                vec![
                    KpiScore {
                        name: "Technical Design Maturity".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "System Architecture".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "Validation Plan".into(),
                        score: 80.0,
                    },
                ],
            ),
            (
                "Prosthetic Hand Actuator Architecture",
                "ktr",
                vec![
                    KpiScore {
                        name: "Technical Design Maturity".into(),
                        score: 88.0,
                    },
                    KpiScore {
                        name: "System Architecture".into(),
                        score: 84.0,
                    },
                    KpiScore {
                        name: "Validation Plan".into(),
                        score: 77.0,
                    },
                ],
            ),
            (
                "Autonomous Ground Vehicle Sensor Fusion",
                "ktr",
                vec![
                    KpiScore {
                        name: "Technical Design Maturity".into(),
                        score: 91.0,
                    },
                    KpiScore {
                        name: "System Architecture".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "Validation Plan".into(),
                        score: 86.0,
                    },
                ],
            ),
            (
                "Satellite Payload Thermal Design",
                "ktr",
                vec![
                    KpiScore {
                        name: "Technical Design Maturity".into(),
                        score: 79.0,
                    },
                    KpiScore {
                        name: "System Architecture".into(),
                        score: 87.0,
                    },
                    KpiScore {
                        name: "Validation Plan".into(),
                        score: 92.0,
                    },
                ],
            ),
            (
                "Firefighting Robot Mechanical Design",
                "ktr",
                vec![
                    KpiScore {
                        name: "Technical Design Maturity".into(),
                        score: 85.0,
                    },
                    KpiScore {
                        name: "System Architecture".into(),
                        score: 81.0,
                    },
                    KpiScore {
                        name: "Validation Plan".into(),
                        score: 89.0,
                    },
                ],
            ),
            (
                "Exoskeleton Control System Architecture",
                "ktr",
                vec![
                    KpiScore {
                        name: "Technical Design Maturity".into(),
                        score: 73.0,
                    },
                    KpiScore {
                        name: "System Architecture".into(),
                        score: 92.0,
                    },
                    KpiScore {
                        name: "Validation Plan".into(),
                        score: 78.0,
                    },
                ],
            ),
            (
                "Agricultural Drone Swarm Coordination Design",
                "ktr",
                vec![
                    KpiScore {
                        name: "Technical Design Maturity".into(),
                        score: 90.0,
                    },
                    KpiScore {
                        name: "System Architecture".into(),
                        score: 76.0,
                    },
                    KpiScore {
                        name: "Validation Plan".into(),
                        score: 83.0,
                    },
                ],
            ),
            (
                "Search-and-Rescue Robot Power System",
                "ktr",
                vec![
                    KpiScore {
                        name: "Technical Design Maturity".into(),
                        score: 84.0,
                    },
                    KpiScore {
                        name: "System Architecture".into(),
                        score: 88.0,
                    },
                    KpiScore {
                        name: "Validation Plan".into(),
                        score: 79.0,
                    },
                ],
            ),
            (
                "Hybrid Rocket Engine Design Review",
                "ktr",
                vec![
                    KpiScore {
                        name: "Technical Design Maturity".into(),
                        score: 87.0,
                    },
                    KpiScore {
                        name: "System Architecture".into(),
                        score: 82.0,
                    },
                    KpiScore {
                        name: "Validation Plan".into(),
                        score: 91.0,
                    },
                ],
            ),
        ];

        for (name, category, kpi_scores) in samples {
            let already = *counts.get(category).unwrap_or(&0);
            if already >= TARGET {
                continue;
            }
            self.insert_project(
                self.default_competition_id().await?,
                None,
                name,
                category,
                kpi_scores,
                None,
                None,
            )
            .await?;
            *counts.entry(category.to_string()).or_insert(0) += 1;
        }

        Ok(())
    }

    pub async fn insert_project(
        &self,
        competition_id: i32,
        team_id: Option<i32>,
        name: &str,
        category: &str,
        kpi_scores: Vec<KpiScore>,
        document: Option<&Document>,
        file_path: Option<&str>,
    ) -> Result<i32> {
        let mut tx = self.pool.begin().await?;

        let document_json = document.map(serde_json::to_value).transpose()?;

        let row = sqlx::query(
            "INSERT INTO projects (competition_id, team_id, name, category, status, document, file_path) VALUES ($1, $2, $3, $4, 'new', $5, $6) RETURNING id",
        )
        .bind(competition_id)
        .bind(team_id)
        .bind(name)
        .bind(category)
        .bind(document_json)
        .bind(file_path)
        .fetch_one(&mut *tx)
        .await?;
        let project_id: i32 = row.get("id");

        for kpi in &kpi_scores {
            sqlx::query("INSERT INTO kpi_scores (project_id, name, score) VALUES ($1, $2, $3)")
                .bind(project_id)
                .bind(&kpi.name)
                .bind(kpi.score)
                .execute(&mut *tx)
                .await?;
        }

        sqlx::query("INSERT INTO activity_log (action, project_id, details) VALUES ($1, $2, $3)")
            .bind("project_created")
            .bind(project_id)
            .bind(name)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(project_id)
    }

    pub async fn list_projects(
        &self,
        category: Option<&str>,
        competition_id: Option<i32>,
    ) -> Result<Vec<Project>> {
        let rows = sqlx::query(
            "SELECT p.id, p.competition_id, p.team_id, p.name, p.category, p.manual_rank, p.notes, p.status, p.review_completed, p.tags,
                    (p.file_path IS NOT NULL) AS has_file,
                    k.name AS kpi_name, k.score AS kpi_score, t.weight AS kpi_weight
             FROM projects p
             LEFT JOIN kpi_scores k ON k.project_id = p.id
             LEFT JOIN category_kpi_templates t ON t.category = p.category AND t.kpi_name = k.name
             WHERE ($1::TEXT IS NULL OR p.category = $1)
               AND ($2::INTEGER IS NULL OR p.competition_id = $2)
             ORDER BY p.id",
        )
        .bind(category)
        .bind(competition_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(build_projects_from_rows(rows))
    }

    pub async fn get_project(&self, id: i32) -> Result<Option<Project>> {
        let rows = sqlx::query(
            "SELECT p.id, p.competition_id, p.team_id, p.name, p.category, p.manual_rank, p.notes, p.status, p.review_completed, p.tags,
                    (p.file_path IS NOT NULL) AS has_file,
                    k.name AS kpi_name, k.score AS kpi_score, t.weight AS kpi_weight
             FROM projects p
             LEFT JOIN kpi_scores k ON k.project_id = p.id
             LEFT JOIN category_kpi_templates t ON t.category = p.category AND t.kpi_name = k.name
             WHERE p.id = $1",
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;

        Ok(build_projects_from_rows(rows).into_iter().next())
    }

    pub async fn get_project_scope(&self, id: i32) -> Result<Option<(i32, String)>> {
        let row = sqlx::query("SELECT competition_id, category FROM projects WHERE id=$1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|row| (row.get("competition_id"), row.get("category"))))
    }

    pub async fn get_project_document(&self, id: i32) -> Result<Option<Document>> {
        let row = sqlx::query("SELECT document FROM projects WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        let Some(row) = row else { return Ok(None) };
        let value: Option<serde_json::Value> = row.get("document");
        Ok(value.map(serde_json::from_value).transpose()?)
    }

    /// Attaches a parsed report to a project created without one, or replaces an
    /// outdated one. Only the analysable content changes; KPI scores are left
    /// untouched so a jury's existing evaluation is never silently overwritten.
    pub async fn update_project_document(
        &self,
        id: i32,
        document: &Document,
        file_path: &str,
    ) -> Result<bool> {
        let result = sqlx::query("UPDATE projects SET document = $1, file_path = $2 WHERE id = $3")
            .bind(serde_json::to_value(document)?)
            .bind(file_path)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_project_file_path(&self, id: i32) -> Result<Option<String>> {
        let row = sqlx::query("SELECT file_path FROM projects WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.and_then(|r| r.get::<Option<String>, _>("file_path")))
    }

    pub async fn default_competition_id(&self) -> Result<i32> {
        sqlx::query_scalar("SELECT id FROM competitions ORDER BY id LIMIT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn team_belongs_to_competition(
        &self,
        team_id: i32,
        competition_id: i32,
    ) -> Result<bool> {
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM teams WHERE id=$1 AND competition_id=$2)")
            .bind(team_id)
            .bind(competition_id)
            .fetch_one(&self.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn team_competition_id(&self, id: i32) -> Result<Option<i32>> {
        sqlx::query_scalar("SELECT competition_id FROM teams WHERE id=$1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn submission_competition_id(&self, id: i32) -> Result<Option<i32>> {
        sqlx::query_scalar(
            "SELECT t.competition_id FROM submissions s JOIN teams t ON t.id=s.team_id WHERE s.id=$1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn demo_day_competition_id(&self, id: i32) -> Result<Option<i32>> {
        sqlx::query_scalar("SELECT competition_id FROM demo_day_slots WHERE id=$1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn appeal_competition_id(&self, id: i32) -> Result<Option<i32>> {
        sqlx::query_scalar(
            "SELECT p.competition_id FROM appeals a JOIN projects p ON p.id=a.project_id WHERE a.id=$1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn update_project(
        &self,
        id: i32,
        notes: Option<&str>,
        status: Option<&str>,
        review_completed: Option<bool>,
        tags: Option<&[String]>,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE projects SET notes = COALESCE($2, notes), status = COALESCE($3, status), review_completed = COALESCE($4, review_completed), tags = COALESCE($5, tags) WHERE id = $1",
        )
        .bind(id)
        .bind(notes)
        .bind(status)
        .bind(review_completed)
        .bind(tags.map(serde_json::to_value).transpose()?)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_project_metadata(
        &self,
        project_id: i32,
    ) -> Result<crate::models::ProjectMetadata> {
        let row = sqlx::query("INSERT INTO project_metadata (project_id) VALUES ($1) ON CONFLICT (project_id) DO NOTHING RETURNING project_id, institution, keywords, github_url, demo_url, prototype_description, team_name, team_members, updated_at")
            .bind(project_id).fetch_optional(&self.pool).await?;
        let row = match row { Some(row) => row, None => sqlx::query("SELECT project_id, institution, keywords, github_url, demo_url, prototype_description, team_name, team_members, updated_at FROM project_metadata WHERE project_id = $1").bind(project_id).fetch_one(&self.pool).await? };
        let keywords =
            serde_json::from_value::<Vec<String>>(row.get("keywords")).unwrap_or_default();
        let team_members =
            serde_json::from_value::<Vec<String>>(row.get("team_members")).unwrap_or_default();
        Ok(crate::models::ProjectMetadata {
            project_id: row.get("project_id"),
            institution: row.get("institution"),
            keywords,
            github_url: row.get("github_url"),
            demo_url: row.get("demo_url"),
            prototype_description: row.get("prototype_description"),
            team_name: row.get("team_name"),
            team_members,
            updated_at: timestamp_text(&row, "updated_at"),
        })
    }

    pub async fn update_project_metadata(
        &self,
        project_id: i32,
        input: &crate::models::UpdateProjectMetadata,
    ) -> Result<crate::models::ProjectMetadata> {
        sqlx::query("INSERT INTO project_metadata (project_id) VALUES ($1) ON CONFLICT (project_id) DO NOTHING").bind(project_id).execute(&self.pool).await?;
        sqlx::query("UPDATE project_metadata SET institution = COALESCE($2, institution), keywords = COALESCE($3, keywords), github_url = COALESCE($4, github_url), demo_url = COALESCE($5, demo_url), prototype_description = COALESCE($6, prototype_description), team_name = COALESCE($7, team_name), team_members = COALESCE($8, team_members), updated_at = NOW() WHERE project_id = $1")
            .bind(project_id).bind(&input.institution).bind(input.keywords.as_ref().map(serde_json::to_value).transpose()?).bind(&input.github_url).bind(&input.demo_url).bind(&input.prototype_description).bind(&input.team_name).bind(input.team_members.as_ref().map(serde_json::to_value).transpose()?).execute(&self.pool).await?;
        self.get_project_metadata(project_id).await
    }

    pub async fn list_project_files(
        &self,
        project_id: i32,
    ) -> Result<Vec<crate::models::ProjectFile>> {
        let rows = sqlx::query("SELECT id, project_id, version, file_name, mime_type, size_bytes, file_path, uploaded_at FROM project_files WHERE project_id = $1 ORDER BY version DESC")
            .bind(project_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(project_file_from_row).collect())
    }

    pub async fn add_project_file(
        &self,
        project_id: i32,
        file_name: &str,
        mime_type: &str,
        size_bytes: i64,
        file_path: &str,
    ) -> Result<crate::models::ProjectFile> {
        let version: i32 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(version), 0) + 1 FROM project_files WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_one(&self.pool)
        .await?;
        let row = sqlx::query("INSERT INTO project_files (project_id, version, file_name, mime_type, size_bytes, file_path) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id, project_id, version, file_name, mime_type, size_bytes, file_path, uploaded_at")
            .bind(project_id).bind(version).bind(file_name).bind(mime_type).bind(size_bytes).bind(file_path).fetch_one(&self.pool).await?;
        Ok(project_file_from_row(&row))
    }

    pub async fn get_project_file_record(
        &self,
        project_id: i32,
        file_id: i32,
    ) -> Result<Option<crate::models::ProjectFile>> {
        let row = sqlx::query("SELECT id, project_id, version, file_name, mime_type, size_bytes, file_path, uploaded_at FROM project_files WHERE project_id = $1 AND id = $2")
            .bind(project_id).bind(file_id).fetch_optional(&self.pool).await?;
        Ok(row.as_ref().map(project_file_from_row))
    }

    pub async fn list_activity(
        &self,
        category: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ActivityEntry>> {
        let rows = match category {
            Some(c) => {
                sqlx::query(
                    "SELECT h.project_id, p.name AS project_name, p.category, h.previous_rank, h.new_rank,
                            h.changed_by, h.timestamp
                     FROM ranking_history h
                     JOIN projects p ON p.id = h.project_id
                     WHERE p.category = $1
                     ORDER BY h.timestamp DESC
                     LIMIT $2",
                )
                .bind(c)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query(
                    "SELECT h.project_id, p.name AS project_name, p.category, h.previous_rank, h.new_rank,
                            h.changed_by, h.timestamp
                     FROM ranking_history h
                     JOIN projects p ON p.id = h.project_id
                     ORDER BY h.timestamp DESC
                     LIMIT $1",
                )
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(rows
            .into_iter()
            .map(|row| {
                let timestamp: chrono::DateTime<chrono::Utc> = row.get("timestamp");
                ActivityEntry {
                    project_id: row.get("project_id"),
                    project_name: row.get("project_name"),
                    category: row.get("category"),
                    previous_rank: row.get("previous_rank"),
                    new_rank: row.get("new_rank"),
                    changed_by: row.get("changed_by"),
                    timestamp: timestamp.to_rfc3339(),
                }
            })
            .collect())
    }

    pub async fn update_ranking(
        &self,
        category: &str,
        order: &[i32],
        changed_by: &str,
    ) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        for (index, project_id) in order.iter().enumerate() {
            let new_rank = (index + 1) as i32;

            let row = sqlx::query(
                "UPDATE projects AS p
                 SET manual_rank = $1
                 FROM (SELECT manual_rank FROM projects WHERE id = $2) AS previous
                 WHERE p.id = $2 AND p.category = $3
                 RETURNING previous.manual_rank AS previous_rank",
            )
            .bind(new_rank)
            .bind(project_id)
            .bind(category)
            .fetch_optional(&mut *tx)
            .await?;

            let Some(row) = row else { continue };
            let previous_rank: Option<i32> = row.get("previous_rank");

            sqlx::query(
                "INSERT INTO ranking_history (project_id, previous_rank, new_rank, changed_by) VALUES ($1, $2, $3, $4)",
            )
            .bind(project_id)
            .bind(previous_rank)
            .bind(new_rank)
            .bind(changed_by)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn create_team(
        &self,
        competition_id: i32,
        input: &crate::models::CreateTeam,
    ) -> Result<crate::models::Team> {
        let row = sqlx::query(
            "INSERT INTO teams (competition_id, name) VALUES ($1, $2)
             RETURNING id, competition_id, name, status, created_at",
        )
        .bind(competition_id)
        .bind(&input.name)
        .fetch_one(&self.pool)
        .await?;
        Ok(team_from_row(&self.pool, &row).await?)
    }

    pub async fn list_teams(&self, competition_id: i32) -> Result<Vec<crate::models::Team>> {
        let rows = sqlx::query(
            "SELECT id, competition_id, name, status, created_at FROM teams
             WHERE competition_id = $1 ORDER BY created_at DESC",
        )
        .bind(competition_id)
        .fetch_all(&self.pool)
        .await?;
        let mut teams = Vec::with_capacity(rows.len());
        for row in &rows {
            teams.push(team_from_row(&self.pool, row).await?);
        }
        Ok(teams)
    }

    pub async fn add_team_member(
        &self,
        team_id: i32,
        input: &crate::models::AddTeamMember,
    ) -> Result<crate::models::TeamMember> {
        let row = sqlx::query(
            "INSERT INTO team_members (team_id, full_name, email, role, is_scholar, birth_year, education_level)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, team_id, full_name, email, role, is_scholar, birth_year, education_level",
        )
        .bind(team_id)
        .bind(&input.full_name)
        .bind(&input.email)
        .bind(input.role.as_deref().unwrap_or("member"))
        .bind(input.is_scholar)
        .bind(input.birth_year)
        .bind(input.education_level.as_deref().unwrap_or(""))
        .fetch_one(&self.pool)
        .await?;
        Ok(member_from_row(&row))
    }

    pub async fn create_submission(
        &self,
        team_id: i32,
        input: &crate::models::CreateSubmission,
    ) -> Result<crate::models::Submission> {
        let ends_at = sqlx::query_scalar::<_, Option<String>>(
            "SELECT ends_at FROM competition_stages WHERE id = $1",
        )
        .bind(input.stage_id)
        .fetch_optional(&self.pool)
        .await?
        .flatten();
        let is_late = ends_at
            .as_deref()
            .and_then(|deadline| deadline.get(0..10))
            .and_then(|date| chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").ok())
            .map(|date| chrono::Utc::now().date_naive() > date)
            .unwrap_or(false);
        let row = sqlx::query(
            "INSERT INTO submissions (team_id, stage_id, title, file_name, status, is_late)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, team_id, stage_id, title, file_name, status, is_late, submitted_at",
        )
        .bind(team_id)
        .bind(input.stage_id)
        .bind(&input.title)
        .bind(&input.file_name)
        .bind(input.status.as_deref().unwrap_or("submitted"))
        .bind(is_late)
        .fetch_one(&self.pool)
        .await?;
        Ok(submission_from_row(&row))
    }

    pub async fn list_submissions(&self, team_id: i32) -> Result<Vec<crate::models::Submission>> {
        let rows = sqlx::query(
            "SELECT id, team_id, stage_id, title, file_name, status, is_late, submitted_at
             FROM submissions WHERE team_id = $1 ORDER BY submitted_at DESC, id DESC",
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(submission_from_row).collect())
    }

    pub async fn upsert_ai_evaluation(
        &self,
        project_id: i32,
        input: &crate::models::UpsertAiEvaluation,
    ) -> Result<crate::models::AiEvaluation> {
        let row = sqlx::query(
            "INSERT INTO ai_evaluations
             (project_id, model_version, total_score, confidence, source_file_version, kpi_scores, strengths,
              weaknesses, missing_information, risks, sources, similar_projects)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
             ON CONFLICT (project_id) DO UPDATE SET
               model_version = EXCLUDED.model_version,
               total_score = EXCLUDED.total_score,
               confidence = EXCLUDED.confidence,
               source_file_version = EXCLUDED.source_file_version,
               kpi_scores = EXCLUDED.kpi_scores,
               strengths = EXCLUDED.strengths,
               weaknesses = EXCLUDED.weaknesses,
               missing_information = EXCLUDED.missing_information,
               risks = EXCLUDED.risks,
               sources = EXCLUDED.sources,
               similar_projects = EXCLUDED.similar_projects,
               evaluated_at = NOW()
             RETURNING project_id, model_version, total_score, confidence, source_file_version, kpi_scores,
                       strengths, weaknesses, missing_information, risks, sources, similar_projects, evaluated_at",
        )
        .bind(project_id)
        .bind(&input.model_version)
        .bind(input.total_score)
        .bind(input.confidence)
        .bind(input.source_file_version)
        .bind(serde_json::to_value(&input.kpi_scores)?)
        .bind(serde_json::to_value(&input.strengths)?)
        .bind(serde_json::to_value(&input.weaknesses)?)
        .bind(serde_json::to_value(&input.missing_information)?)
        .bind(serde_json::to_value(&input.risks)?)
        .bind(serde_json::to_value(&input.sources)?)
        .bind(serde_json::to_value(&input.similar_projects)?)
        .fetch_one(&self.pool).await?;
        Ok(ai_evaluation_from_row(&row)?)
    }

    pub async fn get_ai_evaluation(
        &self,
        project_id: i32,
    ) -> Result<Option<crate::models::AiEvaluation>> {
        let row = sqlx::query(
            "SELECT project_id, model_version, total_score, confidence, source_file_version, kpi_scores,
                    strengths, weaknesses, missing_information, risks, sources, similar_projects, evaluated_at
             FROM ai_evaluations WHERE project_id = $1",
        ).bind(project_id).fetch_optional(&self.pool).await?;
        row.map(|value| ai_evaluation_from_row(&value)).transpose()
    }

    pub async fn get_report_template(
        &self,
        competition_id: i32,
    ) -> Result<Option<crate::models::ReportTemplate>> {
        let row = sqlx::query(
            "SELECT competition_id, name, version, expected_language, min_words, max_words,
                    sections, updated_at, updated_by
             FROM report_templates WHERE competition_id = $1",
        )
        .bind(competition_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(|value| report_template_from_row(&value))
            .transpose()
    }

    /// Every saved edit bumps the version so a compliance result always names
    /// the template revision it was produced against.
    pub async fn upsert_report_template(
        &self,
        competition_id: i32,
        input: &crate::models::UpsertReportTemplate,
        updated_by: &str,
    ) -> Result<crate::models::ReportTemplate> {
        let row = sqlx::query(
            "INSERT INTO report_templates
                (competition_id, name, version, expected_language, min_words, max_words, sections, updated_at, updated_by)
             VALUES ($1, $2, 1, $3, $4, $5, $6, NOW(), $7)
             ON CONFLICT (competition_id) DO UPDATE SET
                name = EXCLUDED.name,
                version = report_templates.version + 1,
                expected_language = EXCLUDED.expected_language,
                min_words = EXCLUDED.min_words,
                max_words = EXCLUDED.max_words,
                sections = EXCLUDED.sections,
                updated_at = NOW(),
                updated_by = EXCLUDED.updated_by
             RETURNING competition_id, name, version, expected_language, min_words, max_words,
                       sections, updated_at, updated_by",
        )
        .bind(competition_id)
        .bind(&input.name)
        .bind(&input.expected_language)
        .bind(input.min_words)
        .bind(input.max_words)
        .bind(serde_json::to_value(&input.sections)?)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await?;
        report_template_from_row(&row)
    }

    pub async fn seed_default_report_template(&self, competition_id: i32) -> Result<()> {
        sqlx::query(
            "INSERT INTO report_templates
                (competition_id, name, version, expected_language, min_words, max_words, sections, updated_by)
             VALUES ($1, $2, 1, 'Turkish', 500, 10000, $3, 'system')
             ON CONFLICT (competition_id) DO NOTHING",
        )
        .bind(competition_id)
        .bind("TEKNOFEST Proje Detay Raporu")
        .bind(serde_json::to_value(crate::template::default_sections())?)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn latest_project_file_version(&self, project_id: i32) -> Result<Option<i32>> {
        sqlx::query_scalar("SELECT MAX(version) FROM project_files WHERE project_id = $1")
            .bind(project_id)
            .fetch_one(&self.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn get_project_research(
        &self,
        project_id: i32,
    ) -> Result<Option<crate::models::ProjectResearchAnalysis>> {
        let row = sqlx::query(
            "SELECT project_id, source_file_version, originality_score, originality_label,
                    query_terms, sources, analyzed_at
             FROM project_research_analyses WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(|value| project_research_from_row(&value))
            .transpose()
    }

    pub async fn upsert_project_research(
        &self,
        analysis: &crate::models::ProjectResearchAnalysis,
    ) -> Result<crate::models::ProjectResearchAnalysis> {
        let row = sqlx::query(
            "INSERT INTO project_research_analyses
             (project_id, source_file_version, originality_score, originality_label, query_terms, sources)
             VALUES ($1,$2,$3,$4,$5,$6)
             ON CONFLICT (project_id) DO UPDATE SET
               source_file_version = EXCLUDED.source_file_version,
               originality_score = EXCLUDED.originality_score,
               originality_label = EXCLUDED.originality_label,
               query_terms = EXCLUDED.query_terms,
               sources = EXCLUDED.sources,
               analyzed_at = NOW()
             RETURNING project_id, source_file_version, originality_score, originality_label,
                       query_terms, sources, analyzed_at",
        )
        .bind(analysis.project_id)
        .bind(analysis.source_file_version)
        .bind(analysis.originality_score)
        .bind(&analysis.originality_label)
        .bind(serde_json::to_value(&analysis.query_terms)?)
        .bind(serde_json::to_value(&analysis.sources)?)
        .fetch_one(&self.pool)
        .await?;
        project_research_from_row(&row)
    }

    pub async fn add_jury_score(
        &self,
        project_id: i32,
        input: &crate::models::CreateJuryScore,
    ) -> Result<crate::models::JuryScore> {
        let row = sqlx::query(
            "INSERT INTO jury_scores (project_id, stage_id, juror_name, total_score, kpi_scores, notes)
             VALUES ($1,$2,$3,$4,$5,$6)
             RETURNING id, project_id, stage_id, juror_name, total_score, kpi_scores, notes, submitted_at",
        )
        .bind(project_id)
        .bind(input.stage_id)
        .bind(&input.juror_name)
        .bind(input.total_score)
        .bind(serde_json::to_value(&input.kpi_scores)?)
        .bind(input.notes.as_deref().unwrap_or(""))
        .fetch_one(&self.pool)
        .await?;
        jury_score_from_row(&row)
    }

    pub async fn list_jury_scores(&self, project_id: i32) -> Result<Vec<crate::models::JuryScore>> {
        let rows = sqlx::query(
            "SELECT id, project_id, stage_id, juror_name, total_score, kpi_scores, notes, submitted_at
             FROM jury_scores WHERE project_id = $1 ORDER BY submitted_at DESC, id DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(jury_score_from_row).collect()
    }

    pub async fn add_jury_assignment(
        &self,
        project_id: i32,
        input: &crate::models::CreateJuryAssignment,
    ) -> Result<crate::models::JuryAssignment> {
        let row = sqlx::query(
            "INSERT INTO jury_assignments (project_id, juror_name, role, category, conflict_declared, conflict_reason)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (project_id, juror_name) DO UPDATE SET role = EXCLUDED.role, category = EXCLUDED.category, conflict_declared = EXCLUDED.conflict_declared, conflict_reason = EXCLUDED.conflict_reason
             RETURNING id, project_id, juror_name, role, status, conflict_declared, conflict_reason, assigned_at",
        )
        .bind(project_id)
        .bind(&input.juror_name)
        .bind(input.role.as_deref().unwrap_or("juror"))
        .bind(&input.category)
        .bind(input.conflict_declared.unwrap_or(false))
        .bind(input.conflict_reason.as_deref().unwrap_or(""))
        .fetch_one(&self.pool)
        .await?;
        Ok(assignment_from_row(&row))
    }

    pub async fn list_jury_assignments(
        &self,
        project_id: i32,
    ) -> Result<Vec<crate::models::JuryAssignment>> {
        let rows = sqlx::query(
            "SELECT id, project_id, juror_name, role, status, conflict_declared, conflict_reason, assigned_at
             FROM jury_assignments WHERE project_id = $1 ORDER BY assigned_at DESC, id DESC",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(assignment_from_row).collect())
    }

    pub async fn record_audit(
        &self,
        action: &str,
        actor: &str,
        entity_type: &str,
        entity_id: Option<i32>,
        details: serde_json::Value,
    ) -> Result<()> {
        let previous_hash: Option<String> =
            sqlx::query_scalar("SELECT event_hash FROM audit_events ORDER BY id DESC LIMIT 1")
                .fetch_optional(&self.pool)
                .await?
                .flatten();
        let details_text = serde_json::to_string(&details)?;
        let created_at = chrono::Utc::now();
        let event_hash = audit_event_hash(
            previous_hash.as_deref(),
            action,
            actor,
            entity_id,
            &details_text,
            &created_at.to_rfc3339(),
        );
        sqlx::query("INSERT INTO audit_events (action, actor, entity_type, entity_id, details, previous_hash, event_hash, created_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)")
            .bind(action).bind(actor).bind(entity_type).bind(entity_id).bind(details).bind(previous_hash).bind(event_hash).bind(created_at)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn list_audit(&self, limit: i64) -> Result<Vec<crate::models::AuditEvent>> {
        let rows = sqlx::query(
            "SELECT id, action, actor, entity_type, entity_id, details, created_at, previous_hash, event_hash
             FROM audit_events ORDER BY created_at DESC, id DESC LIMIT $1",
        ).bind(limit.clamp(1, 200)).fetch_all(&self.pool).await?;
        Ok(rows
            .iter()
            .map(|row| crate::models::AuditEvent {
                id: row.get("id"),
                action: row.get("action"),
                actor: row.get("actor"),
                entity_type: row.get("entity_type"),
                entity_id: row.get("entity_id"),
                details: row.get("details"),
                created_at: timestamp_text(row, "created_at"),
                previous_hash: row.get("previous_hash"),
                event_hash: row.get("event_hash"),
            })
            .collect())
    }

    pub async fn list_users(&self) -> Result<Vec<crate::models::User>> {
        let rows = sqlx::query("SELECT id, full_name, email, role, active, must_change_password, two_factor_enabled, competition_id, category, team_id, created_at FROM users ORDER BY full_name, id")
            .fetch_all(&self.pool).await?;
        Ok(rows
            .iter()
            .map(|row| crate::models::User {
                id: row.get("id"),
                full_name: row.get("full_name"),
                email: row.get("email"),
                role: row.get("role"),
                active: row.get("active"),
                must_change_password: row.get("must_change_password"),
                two_factor_enabled: row.get("two_factor_enabled"),
                two_factor_required: false,
                competition_id: row.get("competition_id"),
                category: row.get("category"),
                team_id: row.get("team_id"),
                created_at: timestamp_text(row, "created_at"),
            })
            .collect())
    }

    pub async fn list_contestant_feedback(
        &self,
        team_id: i32,
    ) -> Result<Vec<crate::models::ContestantFeedback>> {
        let rows = sqlx::query(
            "SELECT p.id, p.name, p.category, p.status, a.total_score, a.strengths, a.weaknesses,
                    a.missing_information, a.risks, a.evaluated_at,
                    c.current_category_score, c.recommended_category,
                    c.recommended_category_score, c.requires_review AS category_requires_review
             FROM projects p
             JOIN ai_evaluations a ON a.project_id = p.id
             LEFT JOIN project_category_fit_analyses c ON c.project_id = p.id
             WHERE p.team_id = $1
             ORDER BY a.evaluated_at DESC, p.id DESC",
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await?;
        rows.iter()
            .map(|row| {
                // The category-fit analysis is optional — a manager may not have
                // run it yet — so its columns come back NULL via the LEFT JOIN.
                let category_fit = row.get::<Option<String>, _>("recommended_category").map(
                    |recommended_category| crate::models::CategoryFitSummary {
                        current_category_score: row.get("current_category_score"),
                        recommended_category,
                        recommended_category_score: row.get("recommended_category_score"),
                        requires_review: row.get("category_requires_review"),
                    },
                );
                Ok(crate::models::ContestantFeedback {
                    project_id: row.get("id"),
                    project_name: row.get("name"),
                    category: row.get("category"),
                    status: crate::models::ProjectStatus::from_str(row.get("status")),
                    total_score: row.get("total_score"),
                    strengths: json_array(row, "strengths")?,
                    weaknesses: json_array(row, "weaknesses")?,
                    missing_information: json_array(row, "missing_information")?,
                    risks: json_array(row, "risks")?,
                    evaluated_at: timestamp_text(row, "evaluated_at"),
                    category_fit,
                })
            })
            .collect()
    }

    pub async fn list_notifications(&self, limit: i64) -> Result<Vec<crate::models::Notification>> {
        let rows = sqlx::query("SELECT id, title, body, kind, audience, category, created_at FROM notifications ORDER BY created_at DESC, id DESC LIMIT $1").bind(limit.clamp(1, 200)).fetch_all(&self.pool).await?;
        Ok(rows
            .iter()
            .map(|row| crate::models::Notification {
                id: row.get("id"),
                title: row.get("title"),
                body: row.get("body"),
                kind: row.get("kind"),
                audience: row.get("audience"),
                category: row.get("category"),
                created_at: timestamp_text(row, "created_at"),
            })
            .collect())
    }

    pub async fn create_notification(
        &self,
        input: &crate::models::CreateNotification,
    ) -> Result<crate::models::Notification> {
        let row = sqlx::query("INSERT INTO notifications (title, body, kind, audience, category) VALUES ($1,$2,$3,$4,$5) RETURNING id, title, body, kind, audience, category, created_at")
            .bind(&input.title).bind(&input.body).bind(&input.kind).bind(&input.audience).bind(&input.category).fetch_one(&self.pool).await?;
        Ok(crate::models::Notification {
            id: row.get("id"),
            title: row.get("title"),
            body: row.get("body"),
            kind: row.get("kind"),
            audience: row.get("audience"),
            category: row.get("category"),
            created_at: timestamp_text(&row, "created_at"),
        })
    }

    pub async fn create_email_campaign(
        &self,
        input: &crate::models::CreateEmailCampaign,
    ) -> Result<crate::models::EmailCampaign> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query("INSERT INTO email_campaigns (subject, body, audience, category) VALUES ($1,$2,$3,$4) RETURNING id, subject, body, audience, category, status, created_at")
            .bind(&input.subject).bind(&input.body).bind(&input.audience).bind(&input.category).fetch_one(&mut *tx).await?;
        let campaign_id: i32 = row.get("id");
        sqlx::query("INSERT INTO email_deliveries (campaign_id, user_id, email) SELECT $1, id, email FROM users WHERE active = TRUE AND ($2 = 'all' OR role = $2) AND ($3::TEXT IS NULL OR category IS NULL OR category = $3)")
            .bind(campaign_id).bind(&input.audience).bind(&input.category).execute(&mut *tx).await?;
        let recipient_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM email_deliveries WHERE campaign_id = $1")
                .bind(campaign_id)
                .fetch_one(&mut *tx)
                .await?;
        tx.commit().await?;
        Ok(crate::models::EmailCampaign {
            id: campaign_id,
            subject: row.get("subject"),
            body: row.get("body"),
            audience: row.get("audience"),
            category: row.get("category"),
            recipient_count,
            status: row.get("status"),
            created_at: timestamp_text(&row, "created_at"),
        })
    }

    pub async fn list_email_campaigns(
        &self,
        limit: i64,
    ) -> Result<Vec<crate::models::EmailCampaign>> {
        let rows = sqlx::query("SELECT c.id, c.subject, c.body, c.audience, c.category, c.status, c.created_at, COUNT(d.id) AS recipient_count FROM email_campaigns c LEFT JOIN email_deliveries d ON d.campaign_id = c.id GROUP BY c.id ORDER BY c.created_at DESC, c.id DESC LIMIT $1")
            .bind(limit.clamp(1, 200)).fetch_all(&self.pool).await?;
        Ok(rows
            .iter()
            .map(|row| crate::models::EmailCampaign {
                id: row.get("id"),
                subject: row.get("subject"),
                body: row.get("body"),
                audience: row.get("audience"),
                category: row.get("category"),
                recipient_count: row.get("recipient_count"),
                status: row.get("status"),
                created_at: timestamp_text(row, "created_at"),
            })
            .collect())
    }

    pub async fn get_email_campaign(
        &self,
        campaign_id: i32,
    ) -> Result<Option<crate::models::EmailCampaign>> {
        let row = sqlx::query("SELECT c.id, c.subject, c.body, c.audience, c.category, c.status, c.created_at, COUNT(d.id) AS recipient_count FROM email_campaigns c LEFT JOIN email_deliveries d ON d.campaign_id = c.id WHERE c.id = $1 GROUP BY c.id")
            .bind(campaign_id).fetch_optional(&self.pool).await?;
        Ok(row.map(|row| crate::models::EmailCampaign {
            id: row.get("id"),
            subject: row.get("subject"),
            body: row.get("body"),
            audience: row.get("audience"),
            category: row.get("category"),
            recipient_count: row.get("recipient_count"),
            status: row.get("status"),
            created_at: timestamp_text(&row, "created_at"),
        }))
    }

    pub async fn claim_queued_email_deliveries(
        &self,
        campaign_id: i32,
        limit: i64,
    ) -> Result<Vec<crate::models::EmailDeliveryTarget>> {
        let rows = sqlx::query("UPDATE email_deliveries SET status = 'sending', attempt_count = attempt_count + 1 WHERE id IN (SELECT id FROM email_deliveries WHERE campaign_id = $1 AND status = 'queued' AND next_attempt_at <= NOW() ORDER BY id LIMIT $2 FOR UPDATE SKIP LOCKED) RETURNING id, email, attempt_count")
            .bind(campaign_id)
            .bind(limit.clamp(1, 100))
            .fetch_all(&self.pool)
            .await?;
        Ok(rows
            .iter()
            .map(|row| crate::models::EmailDeliveryTarget {
                id: row.get("id"),
                email: row.get("email"),
                attempt_count: row.get("attempt_count"),
            })
            .collect())
    }

    pub async fn complete_email_delivery(
        &self,
        delivery: &crate::models::EmailDeliveryTarget,
        error: Option<&str>,
        next_attempt_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<()> {
        let sent = error.is_none();
        let status = if sent {
            "sent"
        } else if delivery.attempt_count >= 5 {
            "failed"
        } else {
            "queued"
        };
        sqlx::query("UPDATE email_deliveries SET status = $2, last_error = $3, next_attempt_at = COALESCE($4, next_attempt_at), sent_at = CASE WHEN $5 THEN NOW() ELSE sent_at END WHERE id = $1")
            .bind(delivery.id)
            .bind(status)
            .bind(error)
            .bind(next_attempt_at)
            .bind(sent)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn refresh_email_campaign_status(&self, campaign_id: i32) -> Result<()> {
        sqlx::query("UPDATE email_campaigns SET status = CASE WHEN NOT EXISTS (SELECT 1 FROM email_deliveries WHERE campaign_id = $1 AND status IN ('queued', 'sending')) THEN CASE WHEN EXISTS (SELECT 1 FROM email_deliveries WHERE campaign_id = $1 AND status = 'failed') THEN 'partial' ELSE 'sent' END ELSE 'queued' END WHERE id = $1")
            .bind(campaign_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn set_email_campaign_status(&self, campaign_id: i32, status: &str) -> Result<()> {
        sqlx::query("UPDATE email_campaigns SET status = $2 WHERE id = $1")
            .bind(campaign_id)
            .bind(status)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn create_user(
        &self,
        input: &crate::models::CreateUser,
    ) -> Result<crate::models::User> {
        let row = sqlx::query("INSERT INTO users (full_name, email, role, competition_id, category, team_id, password_hash, must_change_password) VALUES ($1,$2,$3,$4,$5,$6,$7,TRUE) RETURNING id, full_name, email, role, active, must_change_password, two_factor_enabled, competition_id, category, team_id, created_at")
            .bind(&input.full_name).bind(&input.email).bind(&input.role).bind(input.competition_id).bind(&input.category).bind(input.team_id).bind(&input.password)
            .fetch_one(&self.pool).await?;
        self.user_from_row(&row)
    }

    pub async fn create_initial_admin(
        &self,
        full_name: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<bool> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query("SELECT pg_advisory_xact_lock(20260812)")
            .execute(&mut *transaction)
            .await?;
        let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&mut *transaction)
            .await?;
        if user_count > 0 {
            transaction.commit().await?;
            return Ok(false);
        }
        sqlx::query("INSERT INTO users (full_name, email, role, password_hash, must_change_password) VALUES ($1,$2,'system_admin',$3,FALSE)")
            .bind(full_name)
            .bind(email)
            .bind(password_hash)
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await?;
        Ok(true)
    }

    pub async fn has_users(&self) -> Result<bool> {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await
            .map(|count| count > 0)
            .map_err(Into::into)
    }

    pub async fn update_user(
        &self,
        id: i32,
        input: &crate::models::UpdateUser,
    ) -> Result<crate::models::User> {
        let row = sqlx::query("UPDATE users SET role = COALESCE($2, role), active = COALESCE($3, active), competition_id = COALESCE($4, competition_id), category = COALESCE($5, category), team_id = COALESCE($6, team_id), password_hash = COALESCE($7, password_hash) WHERE id = $1 RETURNING id, full_name, email, role, active, must_change_password, two_factor_enabled, competition_id, category, team_id, created_at")
            .bind(id).bind(&input.role).bind(input.active).bind(input.competition_id).bind(&input.category).bind(input.team_id).bind(&input.password)
            .fetch_one(&self.pool).await?;
        self.user_from_row(&row)
    }

    fn user_from_row(&self, row: &PgRow) -> Result<crate::models::User> {
        Ok(crate::models::User {
            id: row.get("id"),
            full_name: row.get("full_name"),
            email: row.get("email"),
            role: row.get("role"),
            active: row.get("active"),
            must_change_password: row.get("must_change_password"),
            two_factor_enabled: row.get("two_factor_enabled"),
            two_factor_required: false,
            competition_id: row.get("competition_id"),
            category: row.get("category"),
            team_id: row.get("team_id"),
            created_at: timestamp_text(row, "created_at"),
        })
    }

    pub async fn list_juror_profiles(&self) -> Result<Vec<crate::models::JurorProfile>> {
        let rows = sqlx::query("SELECT u.id AS user_id, u.full_name, u.email, COALESCE(p.expertise, '[]'::jsonb) AS expertise, COALESCE(p.institution, '') AS institution, COALESCE(p.max_assignments, 20) AS max_assignments, COUNT(a.id) FILTER (WHERE a.status IN ('assigned','in_progress')) AS active_assignments FROM users u LEFT JOIN juror_profiles p ON p.user_id=u.id LEFT JOIN jury_assignments a ON a.juror_name=u.full_name WHERE u.role IN ('jury_member','chief_judge') AND u.active=TRUE GROUP BY u.id, p.expertise, p.institution, p.max_assignments ORDER BY u.full_name")
            .fetch_all(&self.pool).await?;
        rows.iter()
            .map(|row| {
                Ok(crate::models::JurorProfile {
                    user_id: row.get("user_id"),
                    full_name: row.get("full_name"),
                    email: row.get("email"),
                    expertise: json_array(row, "expertise")?,
                    institution: row.get("institution"),
                    max_assignments: row.get("max_assignments"),
                    active_assignments: row.get("active_assignments"),
                })
            })
            .collect()
    }

    pub async fn calibration_summary(&self) -> Result<crate::models::CalibrationSummary> {
        let overall: Option<f64> = sqlx::query_scalar("SELECT AVG(total_score) FROM jury_scores")
            .fetch_one(&self.pool)
            .await?;
        let overall = overall.unwrap_or(0.0);
        let rows = sqlx::query("SELECT juror_name, COUNT(*) AS score_count, AVG(total_score) AS average_score FROM jury_scores GROUP BY juror_name ORDER BY juror_name").fetch_all(&self.pool).await?;
        let jurors = rows
            .iter()
            .map(|row| {
                let average: f64 = row.get("average_score");
                let deviation = average - overall;
                crate::models::JurorCalibration {
                    juror_name: row.get("juror_name"),
                    score_count: row.get("score_count"),
                    average_score: average,
                    deviation_from_overall: deviation,
                    alert: if deviation.abs() >= 15.0 {
                        Some(if deviation > 0.0 {
                            "Unusually high scoring tendency".into()
                        } else {
                            "Unusually low scoring tendency".into()
                        })
                    } else {
                        None
                    },
                }
            })
            .collect();
        let variance_rows = sqlx::query("SELECT project_id, COUNT(*) AS comment_count, COUNT(DISTINCT NULLIF(trim(notes), '')) AS distinct_comment_count FROM jury_scores WHERE trim(notes) <> '' GROUP BY project_id").fetch_all(&self.pool).await?;
        let kpi_comment_variance = variance_rows
            .iter()
            .map(|row| {
                let count: i64 = row.get("comment_count");
                let distinct: i64 = row.get("distinct_comment_count");
                crate::models::KpiCommentVariance {
                    project_id: row.get("project_id"),
                    comment_count: count,
                    distinct_comment_count: distinct,
                    alert: count >= 2 && distinct >= 2,
                }
            })
            .collect();
        Ok(crate::models::CalibrationSummary {
            overall_average: overall,
            jurors,
            kpi_comment_variance,
        })
    }

    pub async fn create_calibration_case(
        &self,
        input: &crate::models::CreateCalibrationCase,
    ) -> Result<crate::models::CalibrationCase> {
        let row = sqlx::query("INSERT INTO calibration_cases (project_id, expected_score) VALUES ($1,$2) ON CONFLICT (project_id) DO UPDATE SET expected_score=EXCLUDED.expected_score, active=TRUE RETURNING id, project_id, expected_score, active").bind(input.project_id).bind(input.expected_score).fetch_one(&self.pool).await?;
        Ok(crate::models::CalibrationCase {
            id: row.get("id"),
            project_id: row.get("project_id"),
            expected_score: row.get("expected_score"),
            active: row.get("active"),
        })
    }

    pub async fn update_juror_profile(
        &self,
        user_id: i32,
        input: &crate::models::UpdateJurorProfile,
    ) -> Result<()> {
        sqlx::query("INSERT INTO juror_profiles (user_id, expertise, institution, max_assignments) VALUES ($1,$2,$3,$4) ON CONFLICT (user_id) DO UPDATE SET expertise=EXCLUDED.expertise, institution=EXCLUDED.institution, max_assignments=EXCLUDED.max_assignments")
            .bind(user_id).bind(serde_json::to_value(&input.expertise)?).bind(&input.institution).bind(input.max_assignments).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn create_appeal(
        &self,
        project_id: i32,
        input: &crate::models::CreateAppeal,
    ) -> Result<crate::models::Appeal> {
        let old_score: Option<f64> = sqlx::query_scalar("SELECT COALESCE(SUM(k.score * t.weight) / NULLIF(SUM(t.weight), 0), 0) FROM kpi_scores k JOIN category_kpi_templates t ON t.category=(SELECT category FROM projects WHERE id=$1) AND t.kpi_name=k.name WHERE k.project_id=$1").bind(project_id).fetch_one(&self.pool).await?;
        let row = sqlx::query("INSERT INTO appeals (project_id, submitted_by, reason, deadline, committee, old_score) VALUES ($1,$2,$3,$4,$5,$6) RETURNING id, project_id, submitted_by, reason, deadline, committee, status, decision_reason, created_at, resolved_at")
            .bind(project_id).bind(&input.submitted_by).bind(&input.reason).bind(&input.deadline).bind(serde_json::to_value(&input.committee)?).bind(old_score).fetch_one(&self.pool).await?;
        appeal_from_row(&row)
    }

    pub async fn list_appeals(&self, project_id: i32) -> Result<Vec<crate::models::Appeal>> {
        let rows = sqlx::query("SELECT id, project_id, submitted_by, reason, deadline, committee, status, decision_reason, created_at, resolved_at FROM appeals WHERE project_id=$1 ORDER BY created_at DESC").bind(project_id).fetch_all(&self.pool).await?;
        rows.iter().map(appeal_from_row).collect()
    }

    pub async fn resolve_appeal(
        &self,
        id: i32,
        input: &crate::models::ResolveAppeal,
    ) -> Result<crate::models::Appeal> {
        let row = sqlx::query("UPDATE appeals SET status=$2, decision_reason=$3, new_score=$4, resolved_at=NOW() WHERE id=$1 RETURNING id, project_id, submitted_by, reason, deadline, committee, status, decision_reason, created_at, resolved_at")
            .bind(id).bind(&input.status).bind(&input.decision_reason).bind(input.new_score).fetch_one(&self.pool).await?;
        appeal_from_row(&row)
    }

    pub async fn update_team_status(&self, team_id: i32, status: &str) -> Result<()> {
        sqlx::query("UPDATE teams SET status = $2 WHERE id = $1")
            .bind(team_id)
            .bind(status)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn select_finalists(&self, competition_id: i32, team_ids: &[i32]) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("UPDATE teams SET status = 'reviewing' WHERE competition_id = $1")
            .bind(competition_id)
            .execute(&mut *tx)
            .await?;
        for team_id in team_ids {
            sqlx::query(
                "UPDATE teams SET status = 'finalist' WHERE id = $1 AND competition_id = $2",
            )
            .bind(team_id)
            .bind(competition_id)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn add_demo_day_slot(
        &self,
        competition_id: i32,
        input: &crate::models::CreateDemoDaySlot,
    ) -> Result<crate::models::DemoDaySlot> {
        let row = sqlx::query(
            "INSERT INTO demo_day_slots (competition_id, team_id, slot_order, room, starts_at, duration_minutes, check_in_token)
             VALUES ($1,$2,$3,$4,$5,$6,$7)
             RETURNING id, competition_id, team_id, slot_order, room, starts_at, duration_minutes, status, checked_in_at, evidence_urls, field_score, jury_signature, check_in_token, prototype_checklist",
        ).bind(competition_id).bind(input.team_id).bind(input.slot_order).bind(&input.room)
        .bind(&input.starts_at).bind(input.duration_minutes.unwrap_or(10)).bind(uuid::Uuid::new_v4().to_string())
        .fetch_one(&self.pool).await?;
        Ok(demo_slot_from_row(&row))
    }

    pub async fn list_demo_day_slots(
        &self,
        competition_id: i32,
    ) -> Result<Vec<crate::models::DemoDaySlot>> {
        let rows = sqlx::query(
            "SELECT id, competition_id, team_id, slot_order, room, starts_at, duration_minutes, status, checked_in_at, evidence_urls, field_score, jury_signature, check_in_token, prototype_checklist
             FROM demo_day_slots WHERE competition_id = $1 ORDER BY slot_order",
        ).bind(competition_id).fetch_all(&self.pool).await?;
        Ok(rows.iter().map(demo_slot_from_row).collect())
    }

    pub async fn update_demo_day_slot(
        &self,
        id: i32,
        input: &crate::models::UpdateDemoDaySlot,
    ) -> Result<crate::models::DemoDaySlot> {
        let row = sqlx::query("UPDATE demo_day_slots SET status=COALESCE($2,status), checked_in_at=CASE WHEN $3::BOOLEAN THEN NOW() WHEN $3::BOOLEAN=FALSE THEN NULL ELSE checked_in_at END, evidence_urls=COALESCE($4,evidence_urls), field_score=COALESCE($5,field_score), jury_signature=COALESCE($6,jury_signature), prototype_checklist=COALESCE($7,prototype_checklist) WHERE id=$1 RETURNING id, competition_id, team_id, slot_order, room, starts_at, duration_minutes, status, checked_in_at, evidence_urls, field_score, jury_signature, check_in_token, prototype_checklist")
            .bind(id).bind(&input.status).bind(input.check_in).bind(input.evidence_urls.as_ref().map(serde_json::to_value).transpose()?).bind(input.field_score).bind(&input.jury_signature).bind(input.prototype_checklist.as_ref().map(serde_json::to_value).transpose()?).fetch_one(&self.pool).await?;
        Ok(demo_slot_from_row(&row))
    }

    /// Analysis progress across a competition, computed in one pass over the
    /// stored analyses rather than by asking each project in turn — the
    /// evaluation manager watches this while a bulk run is in flight, so it has
    /// to stay cheap with a few hundred submissions.
    pub async fn assessment_progress(
        &self,
        competition_id: i32,
        pending_limit: i64,
    ) -> Result<crate::models::AssessmentProgress> {
        let totals = sqlx::query(
            "SELECT
                COUNT(*) AS total_projects,
                COUNT(*) FILTER (WHERE p.document IS NOT NULL) AS parsed_reports,
                COUNT(cf.project_id) AS category_fit_completed,
                COUNT(sa.project_id) AS similarity_completed,
                COUNT(ae.project_id) AS criterion_evaluation_completed,
                COUNT(*) FILTER (
                    WHERE cf.requires_review IS TRUE OR sa.requires_review IS TRUE
                ) AS flagged_for_review
             FROM projects p
             LEFT JOIN project_category_fit_analyses cf ON cf.project_id = p.id
             LEFT JOIN project_similarity_analyses sa ON sa.project_id = p.id
             LEFT JOIN ai_evaluations ae ON ae.project_id = p.id
             WHERE p.competition_id = $1",
        )
        .bind(competition_id)
        .fetch_one(&self.pool)
        .await?;

        let parsed_reports: i64 = totals.get("parsed_reports");
        let category_fit_completed: i64 = totals.get("category_fit_completed");
        let similarity_completed: i64 = totals.get("similarity_completed");
        let criterion_evaluation_completed: i64 = totals.get("criterion_evaluation_completed");

        // Three analyses per parsed report. A competition with nothing parsed
        // yet is reported as 0% rather than dividing by zero.
        let expected = parsed_reports * 3;
        let completion_percent = if expected == 0 {
            0.0
        } else {
            (category_fit_completed + similarity_completed + criterion_evaluation_completed) as f64
                / expected as f64
                * 100.0
        };

        let rows = sqlx::query(
            "SELECT p.id, p.category,
                    cf.project_id IS NOT NULL AS has_category_fit,
                    sa.project_id IS NOT NULL AS has_similarity,
                    ae.project_id IS NOT NULL AS has_evaluation
             FROM projects p
             LEFT JOIN project_category_fit_analyses cf ON cf.project_id = p.id
             LEFT JOIN project_similarity_analyses sa ON sa.project_id = p.id
             LEFT JOIN ai_evaluations ae ON ae.project_id = p.id
             WHERE p.competition_id = $1
               AND p.document IS NOT NULL
               AND (cf.project_id IS NULL OR sa.project_id IS NULL OR ae.project_id IS NULL)
             ORDER BY p.id DESC
             LIMIT $2",
        )
        .bind(competition_id)
        .bind(pending_limit)
        .fetch_all(&self.pool)
        .await?;

        let pending_projects = rows
            .into_iter()
            .map(|row| {
                let id: i32 = row.get("id");
                let mut missing = Vec::new();
                if !row.get::<bool, _>("has_category_fit") {
                    missing.push("category_fit".to_string());
                }
                if !row.get::<bool, _>("has_similarity") {
                    missing.push("similarity".to_string());
                }
                if !row.get::<bool, _>("has_evaluation") {
                    missing.push("criterion_evaluation".to_string());
                }
                crate::models::PendingAssessment {
                    project_id: id,
                    project_reference: format!("PRJ-{id:06}"),
                    category: row.get("category"),
                    missing,
                }
            })
            .collect();

        Ok(crate::models::AssessmentProgress {
            competition_id,
            total_projects: totals.get("total_projects"),
            parsed_reports,
            category_fit_completed,
            similarity_completed,
            criterion_evaluation_completed,
            flagged_for_review: totals.get("flagged_for_review"),
            completion_percent,
            pending_projects,
        })
    }

    /// Projects in this competition that still need at least one analysis. The
    /// bulk run walks this list, so it resumes naturally after an interruption
    /// instead of repeating work already stored.
    pub async fn projects_awaiting_assessment(&self, competition_id: i32) -> Result<Vec<i32>> {
        let rows = sqlx::query(
            "SELECT p.id FROM projects p
             LEFT JOIN project_category_fit_analyses cf ON cf.project_id = p.id
             LEFT JOIN project_similarity_analyses sa ON sa.project_id = p.id
             LEFT JOIN ai_evaluations ae ON ae.project_id = p.id
             WHERE p.competition_id = $1
               AND p.document IS NOT NULL
               AND (cf.project_id IS NULL OR sa.project_id IS NULL OR ae.project_id IS NULL)
             ORDER BY p.id",
        )
        .bind(competition_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(|row| row.get("id")).collect())
    }

    pub async fn competition_report(
        &self,
        competition_id: i32,
    ) -> Result<crate::models::CompetitionReport> {
        let row = sqlx::query(
            "SELECT
               (SELECT COUNT(*) FROM teams WHERE competition_id = $1) AS total_teams,
               (SELECT COUNT(*) FROM teams WHERE competition_id = $1 AND status = 'finalist') AS finalist_teams,
               (SELECT COUNT(*) FROM teams WHERE competition_id = $1 AND status = 'rejected') AS rejected_teams,
               (SELECT COUNT(*) FROM submissions s JOIN teams t ON t.id = s.team_id WHERE t.competition_id = $1) AS submitted_deliverables,
               (SELECT COUNT(*) FROM competition_stages WHERE competition_id = $1) AS total_stages,
               (SELECT COUNT(*) FROM demo_day_slots WHERE competition_id = $1) AS demo_day_slots",
        ).bind(competition_id).fetch_one(&self.pool).await?;
        Ok(crate::models::CompetitionReport {
            competition_id,
            total_teams: row.get("total_teams"),
            finalist_teams: row.get("finalist_teams"),
            rejected_teams: row.get("rejected_teams"),
            submitted_deliverables: row.get("submitted_deliverables"),
            total_stages: row.get("total_stages"),
            demo_day_slots: row.get("demo_day_slots"),
        })
    }

    pub async fn finalize_competition(
        &self,
        id: i32,
        minutes: &str,
        signed_by: &str,
    ) -> Result<()> {
        let result = sqlx::query("UPDATE competitions SET final_minutes=$2, final_signed_by=$3, results_locked_at=NOW(), status='archived' WHERE id=$1 AND results_locked_at IS NULL")
            .bind(id).bind(minutes).bind(signed_by).execute(&self.pool).await?;
        if result.rows_affected() == 0 {
            anyhow::bail!("Competition results are already locked or competition does not exist");
        }
        Ok(())
    }

    pub async fn add_submission_version(
        &self,
        submission_id: i32,
        file_name: &str,
        file_path: &str,
    ) -> Result<crate::models::SubmissionVersion> {
        let next: i32 = sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) + 1 FROM submission_versions WHERE submission_id = $1")
            .bind(submission_id).fetch_one(&self.pool).await?;
        let row = sqlx::query(
            "INSERT INTO submission_versions (submission_id, version, file_name, file_path)
             VALUES ($1,$2,$3,$4)
             RETURNING id, submission_id, version, file_name, file_path, uploaded_at",
        )
        .bind(submission_id)
        .bind(next)
        .bind(file_name)
        .bind(file_path)
        .fetch_one(&self.pool)
        .await?;
        Ok(submission_version_from_row(&row))
    }

    pub async fn list_submission_versions(
        &self,
        submission_id: i32,
    ) -> Result<Vec<crate::models::SubmissionVersion>> {
        let rows = sqlx::query(
            "SELECT id, submission_id, version, file_name, file_path, uploaded_at
             FROM submission_versions WHERE submission_id = $1 ORDER BY version DESC",
        )
        .bind(submission_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(submission_version_from_row).collect())
    }
}

fn optional_text(row: &PgRow, key: &str) -> Option<String> {
    row.try_get(key).ok().flatten()
}
fn timestamp_text(row: &PgRow, key: &str) -> String {
    row.try_get::<chrono::DateTime<chrono::Utc>, _>(key)
        .map(|v| v.to_rfc3339())
        .unwrap_or_default()
}

fn project_file_from_row(row: &PgRow) -> crate::models::ProjectFile {
    crate::models::ProjectFile {
        id: row.get("id"),
        project_id: row.get("project_id"),
        version: row.get("version"),
        file_name: row.get("file_name"),
        mime_type: row.get("mime_type"),
        size_bytes: row.get("size_bytes"),
        file_path: row.get("file_path"),
        uploaded_at: timestamp_text(row, "uploaded_at"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_hash_is_sha256_and_changes_with_the_payload() {
        let first = audit_event_hash(
            None,
            "project_updated",
            "admin@example.org",
            Some(4),
            "{}",
            "2026-08-12T12:00:00+00:00",
        );
        let second = audit_event_hash(
            Some(&first),
            "project_updated",
            "admin@example.org",
            Some(4),
            "{}",
            "2026-08-12T12:00:00+00:00",
        );

        assert_eq!(first.len(), 64);
        assert_ne!(first, second);
    }
}

fn member_from_row(row: &PgRow) -> crate::models::TeamMember {
    crate::models::TeamMember {
        id: row.get("id"),
        team_id: row.get("team_id"),
        full_name: row.get("full_name"),
        email: row.get("email"),
        role: row.get("role"),
        is_scholar: row.get("is_scholar"),
        birth_year: row.get("birth_year"),
        education_level: row.get("education_level"),
    }
}
async fn team_from_row(pool: &PgPool, row: &PgRow) -> Result<crate::models::Team> {
    let member_rows = sqlx::query("SELECT id, team_id, full_name, email, role, is_scholar, birth_year, education_level FROM team_members WHERE team_id = $1 ORDER BY id")
    .bind(row.get::<i32, _>("id")).fetch_all(pool).await?;
    Ok(crate::models::Team {
        id: row.get("id"),
        competition_id: row.get("competition_id"),
        name: row.get("name"),
        status: row.get("status"),
        members: member_rows.iter().map(member_from_row).collect(),
        created_at: timestamp_text(row, "created_at"),
    })
}
fn submission_from_row(row: &PgRow) -> crate::models::Submission {
    crate::models::Submission {
        id: row.get("id"),
        team_id: row.get("team_id"),
        stage_id: row.get("stage_id"),
        title: row.get("title"),
        file_name: row.get("file_name"),
        status: row.get("status"),
        is_late: row.get("is_late"),
        submitted_at: optional_text(row, "submitted_at").or_else(|| {
            row.try_get::<chrono::DateTime<chrono::Utc>, _>("submitted_at")
                .ok()
                .map(|v| v.to_rfc3339())
        }),
    }
}
fn json_array<T: serde::de::DeserializeOwned>(row: &PgRow, key: &str) -> Result<T> {
    Ok(serde_json::from_value(row.get(key))?)
}
fn report_template_from_row(row: &PgRow) -> Result<crate::models::ReportTemplate> {
    let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
    Ok(crate::models::ReportTemplate {
        competition_id: row.get("competition_id"),
        name: row.get("name"),
        version: row.get("version"),
        expected_language: row.get("expected_language"),
        min_words: row.get("min_words"),
        max_words: row.get("max_words"),
        sections: json_array(row, "sections")?,
        updated_at: updated_at.to_rfc3339(),
        updated_by: row.get("updated_by"),
    })
}

fn ai_evaluation_from_row(row: &PgRow) -> Result<crate::models::AiEvaluation> {
    Ok(crate::models::AiEvaluation {
        project_id: row.get("project_id"),
        model_version: row.get("model_version"),
        total_score: row.get("total_score"),
        confidence: row.get("confidence"),
        source_file_version: row.get("source_file_version"),
        kpi_scores: json_array(row, "kpi_scores")?,
        strengths: json_array(row, "strengths")?,
        weaknesses: json_array(row, "weaknesses")?,
        missing_information: json_array(row, "missing_information")?,
        risks: json_array(row, "risks")?,
        sources: json_array(row, "sources")?,
        similar_projects: json_array(row, "similar_projects")?,
        evaluated_at: timestamp_text(row, "evaluated_at"),
    })
}
fn project_research_from_row(row: &PgRow) -> Result<crate::models::ProjectResearchAnalysis> {
    Ok(crate::models::ProjectResearchAnalysis {
        project_id: row.get("project_id"),
        source_file_version: row.get("source_file_version"),
        originality_score: row.get("originality_score"),
        originality_label: row.get("originality_label"),
        query_terms: json_array(row, "query_terms")?,
        sources: json_array(row, "sources")?,
        analyzed_at: timestamp_text(row, "analyzed_at"),
    })
}
fn jury_score_from_row(row: &PgRow) -> Result<crate::models::JuryScore> {
    Ok(crate::models::JuryScore {
        id: row.get("id"),
        project_id: row.get("project_id"),
        stage_id: row.get("stage_id"),
        juror_name: row.get("juror_name"),
        total_score: row.get("total_score"),
        kpi_scores: json_array(row, "kpi_scores")?,
        notes: row.get("notes"),
        submitted_at: timestamp_text(row, "submitted_at"),
    })
}
fn assignment_from_row(row: &PgRow) -> crate::models::JuryAssignment {
    crate::models::JuryAssignment {
        id: row.get("id"),
        project_id: row.get("project_id"),
        juror_name: row.get("juror_name"),
        role: row.get("role"),
        status: row.get("status"),
        conflict_declared: row.get("conflict_declared"),
        conflict_reason: row.get("conflict_reason"),
        assigned_at: timestamp_text(row, "assigned_at"),
    }
}
fn appeal_from_row(row: &PgRow) -> Result<crate::models::Appeal> {
    Ok(crate::models::Appeal {
        id: row.get("id"),
        project_id: row.get("project_id"),
        submitted_by: row.get("submitted_by"),
        reason: row.get("reason"),
        deadline: row.get("deadline"),
        committee: json_array(row, "committee")?,
        status: row.get("status"),
        decision_reason: row.get("decision_reason"),
        created_at: timestamp_text(row, "created_at"),
        resolved_at: optional_text(row, "resolved_at").or_else(|| {
            row.try_get::<chrono::DateTime<chrono::Utc>, _>("resolved_at")
                .ok()
                .map(|v| v.to_rfc3339())
        }),
    })
}
fn demo_slot_from_row(row: &PgRow) -> crate::models::DemoDaySlot {
    crate::models::DemoDaySlot {
        id: row.get("id"),
        competition_id: row.get("competition_id"),
        team_id: row.get("team_id"),
        slot_order: row.get("slot_order"),
        room: row.get("room"),
        starts_at: row.get("starts_at"),
        duration_minutes: row.get("duration_minutes"),
        status: row.get("status"),
        checked_in_at: optional_text(row, "checked_in_at").or_else(|| {
            row.try_get::<chrono::DateTime<chrono::Utc>, _>("checked_in_at")
                .ok()
                .map(|v| v.to_rfc3339())
        }),
        evidence_urls: json_array(row, "evidence_urls").unwrap_or_default(),
        field_score: row.get("field_score"),
        jury_signature: row.get("jury_signature"),
        check_in_token: row.get("check_in_token"),
        prototype_checklist: json_array(row, "prototype_checklist").unwrap_or_default(),
    }
}
fn submission_version_from_row(row: &PgRow) -> crate::models::SubmissionVersion {
    crate::models::SubmissionVersion {
        id: row.get("id"),
        submission_id: row.get("submission_id"),
        version: row.get("version"),
        file_name: row.get("file_name"),
        file_path: row.get("file_path"),
        uploaded_at: timestamp_text(row, "uploaded_at"),
    }
}

fn build_projects_from_rows(rows: Vec<PgRow>) -> Vec<Project> {
    let mut projects: Vec<Project> = Vec::new();
    let mut weights: Vec<Vec<f64>> = Vec::new();

    for row in rows {
        let id: i32 = row.get("id");

        if projects.last().map(|p| p.id) != Some(id) {
            let status_str: String = row.get("status");
            projects.push(Project {
                id,
                competition_id: row.get("competition_id"),
                team_id: row.get("team_id"),
                name: row.get("name"),
                category: row.get("category"),
                kpi_scores: Vec::new(),
                ai_score: 0.0,
                manual_rank: row.get("manual_rank"),
                notes: row.get("notes"),
                status: ProjectStatus::from_str(&status_str),
                has_file: row.get("has_file"),
                review_completed: row.get("review_completed"),
                tags: serde_json::from_value(row.get("tags")).unwrap_or_default(),
            });
            weights.push(Vec::new());
        }

        if let Ok(kpi_name) = row.try_get::<String, _>("kpi_name") {
            let score: f64 = row.get("kpi_score");
            let weight: f64 = row.try_get("kpi_weight").unwrap_or(1.0);
            projects.last_mut().unwrap().kpi_scores.push(KpiScore {
                name: kpi_name,
                score,
            });
            weights.last_mut().unwrap().push(weight);
        }
    }

    for (project, kpi_weights) in projects.iter_mut().zip(weights.iter()) {
        let total_weight: f64 = kpi_weights.iter().sum();
        if total_weight > 0.0 {
            let weighted_sum: f64 = project
                .kpi_scores
                .iter()
                .zip(kpi_weights.iter())
                .map(|(kpi, weight)| kpi.score * weight)
                .sum();
            project.ai_score = weighted_sum / total_weight;
        }
    }

    projects
}
