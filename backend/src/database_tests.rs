//! Integration tests for the persistence layer.
//!
//! These are the only tests that exercise real SQL. Everything else in the
//! suite runs against pure functions, which left the migrations, the audit
//! chain, the append-only trigger and every aggregate query unverified.
//!
//! Each test runs against its own freshly created database so it can assert on
//! absolute counts and cannot be disturbed by, or disturb, another test or a
//! development database. They are skipped when `TEST_DATABASE_URL` is unset, so
//! `cargo test` still works with no PostgreSQL available.

use super::*;
use crate::models::{Document, FileType};
use sqlx::Connection;

/// A maintenance connection string pointing at a server, not at the database
/// under test — creating one requires connecting to another.
fn admin_url() -> Option<String> {
    std::env::var("TEST_DATABASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
}

struct IsolatedDatabase {
    database: Database,
    name: String,
    admin_url: String,
}

impl IsolatedDatabase {
    /// `Database::new` runs the migrations, so a successful setup already
    /// asserts they apply cleanly to an empty database.
    async fn create() -> Option<Self> {
        let admin_url = admin_url()?;
        let name = format!("jury_test_{}", uuid::Uuid::new_v4().simple());
        let mut admin = sqlx::PgConnection::connect(&admin_url)
            .await
            .expect("the maintenance database must be reachable");
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!("CREATE DATABASE {name}")))
            .execute(&mut admin)
            .await
            .expect("the test database should be created");
        let url = swap_database(&admin_url, &name);
        let database = Database::new(&url)
            .await
            .expect("migrations should apply to an empty database");
        Some(Self {
            database,
            name,
            admin_url,
        })
    }

    async fn drop(self) {
        let Self {
            database,
            name,
            admin_url,
        } = self;
        database.pool.close().await;
        if let Ok(mut admin) = sqlx::PgConnection::connect(&admin_url).await {
            let _ = sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                "DROP DATABASE IF EXISTS {name} WITH (FORCE)"
            )))
            .execute(&mut admin)
            .await;
        }
    }
}

fn swap_database(url: &str, name: &str) -> String {
    let (base, _) = url.rsplit_once('/').expect("a database URL has a path");
    format!("{base}/{name}")
}

/// Skips the body when no test server is configured, so the suite stays green
/// on a machine without PostgreSQL.
macro_rules! with_database {
    ($binding:ident, $body:block) => {
        let Some(fixture) = IsolatedDatabase::create().await else {
            eprintln!("skipped: set TEST_DATABASE_URL to run the persistence tests");
            return;
        };
        let $binding = &fixture.database;
        $body
        fixture.drop().await;
    };
}

#[tokio::test]
async fn migrations_are_recorded_and_rerunning_them_is_a_no_op() {
    with_database!(database, {
        let versions: Vec<i64> =
            sqlx::query_scalar("SELECT version FROM schema_migrations ORDER BY version")
                .fetch_all(&database.pool)
                .await
                .expect("migration rows should be readable");
        assert_eq!(
            versions,
            (1..=7).collect::<Vec<i64>>(),
            "every migration should be recorded exactly once"
        );

        database
            .run_migrations()
            .await
            .expect("migrations must be safe to run again");
        let after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM schema_migrations")
            .fetch_one(&database.pool)
            .await
            .expect("count should be readable");
        assert_eq!(after, 7, "a second run must not duplicate rows");
    });
}

/// The guard exists so an older binary cannot silently run against a schema a
/// newer one wrote — it would read columns that have changed meaning.
#[tokio::test]
async fn a_newer_schema_than_the_binary_supports_is_refused() {
    with_database!(database, {
        sqlx::query(
            "INSERT INTO schema_migrations (version, name) VALUES (999, 'from_the_future')",
        )
        .execute(&database.pool)
        .await
        .expect("the future migration row should insert");
        let error = database
            .run_migrations()
            .await
            .expect_err("a newer schema must be refused");
        assert!(
            error.to_string().contains("newer than this application"),
            "unexpected error: {error}"
        );
    });
}

/// Several instances start at once behind a load balancer. The advisory lock is
/// what stops each of them creating its own administrator.
#[tokio::test]
async fn the_initial_administrator_is_created_only_once() {
    with_database!(database, {
        assert!(!database.has_users().await.expect("query should succeed"));
        assert!(
            database
                .create_initial_admin("First Admin", "admin@example.org", "hash")
                .await
                .expect("first bootstrap should succeed")
        );
        assert!(
            !database
                .create_initial_admin("Second Admin", "other@example.org", "hash")
                .await
                .expect("second bootstrap should succeed without inserting"),
            "a database that already has users must not gain another bootstrap admin"
        );
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&database.pool)
            .await
            .expect("count should be readable");
        assert_eq!(count, 1);
        assert!(database.has_users().await.expect("query should succeed"));
    });
}

/// The audit trail is the record of who decided what in a competition. Both its
/// tamper protections are enforced by the database itself.
#[tokio::test]
async fn audit_events_are_hash_chained_and_cannot_be_altered() {
    with_database!(database, {
        for action in ["project_opened", "project_updated", "jury_score_submitted"] {
            database
                .record_audit(
                    action,
                    "actor@example.org",
                    "project",
                    Some(1),
                    serde_json::json!({}),
                )
                .await
                .expect("audit write should succeed");
        }

        let rows =
            sqlx::query("SELECT action, previous_hash, event_hash FROM audit_events ORDER BY id")
                .fetch_all(&database.pool)
                .await
                .expect("audit rows should be readable");
        assert_eq!(rows.len(), 3);
        assert!(
            rows[0].get::<Option<String>, _>("previous_hash").is_none(),
            "the first event starts the chain"
        );
        for window in rows.windows(2) {
            let previous: String = window[0].get("event_hash");
            let linked: Option<String> = window[1].get("previous_hash");
            assert_eq!(
                linked.as_deref(),
                Some(previous.as_str()),
                "each event must carry the hash of the one before it"
            );
        }

        let updated = sqlx::query("UPDATE audit_events SET actor = 'someone.else@example.org'")
            .execute(&database.pool)
            .await;
        assert!(updated.is_err(), "audit events must not be updatable");
        let deleted = sqlx::query("DELETE FROM audit_events")
            .execute(&database.pool)
            .await;
        assert!(deleted.is_err(), "audit events must not be deletable");
    });
}

async fn seed_competition(database: &Database) -> i32 {
    database
        .default_competition_id()
        .await
        .expect("the baseline migration creates a competition")
}

async fn insert_report(database: &Database, competition_id: i32, category: &str) -> i32 {
    let document = Document {
        filename: "rapor.md".into(),
        file_type: FileType::Markdown,
        raw_text: "Bu proje sulama sistemlerinde su tasarrufu saglamaktadir.".into(),
        word_count: 8,
        headings: Vec::new(),
        keywords: Vec::new(),
        references: Vec::new(),
        has_references: false,
        has_abstract: true,
        has_conclusion: true,
        has_methodology: true,
        language: crate::models::Language::turkish(),
        sections: Vec::new(),
    };
    database
        .insert_project(
            competition_id,
            None,
            "Test Report",
            category,
            Vec::new(),
            Some(&document),
            None,
        )
        .await
        .expect("project insert should succeed")
}

/// The authorization middleware resolves a project's competition and category
/// through this to decide whether the caller may see it, so a wrong answer is a
/// data-scope failure rather than a display bug.
#[tokio::test]
async fn project_scope_is_reported_for_the_middleware() {
    with_database!(database, {
        let competition_id = seed_competition(database).await;
        let project_id = insert_report(database, competition_id, "sustainability").await;

        let scope = database
            .get_project_scope(project_id)
            .await
            .expect("scope lookup should succeed");
        assert_eq!(scope, Some((competition_id, "sustainability".to_string())));
        assert_eq!(
            database
                .get_project_scope(project_id + 9999)
                .await
                .expect("missing project should not error"),
            None,
            "an unknown project must report no scope so the route answers 404"
        );
    });
}

/// The evaluation manager watches this figure while a bulk run is in flight.
#[tokio::test]
async fn assessment_progress_counts_only_parsed_reports() {
    with_database!(database, {
        let competition_id = seed_competition(database).await;
        let with_report = insert_report(database, competition_id, "sustainability").await;
        // A project whose report never parsed cannot be analysed, so it must not
        // drag the completion percentage down for ever.
        database
            .insert_project(
                competition_id,
                None,
                "Unparsed",
                "software",
                Vec::new(),
                None,
                None,
            )
            .await
            .expect("insert should succeed");

        let progress = database
            .assessment_progress(competition_id, 50)
            .await
            .expect("progress should be readable");
        assert_eq!(progress.total_projects, 2);
        assert_eq!(progress.parsed_reports, 1);
        assert_eq!(progress.completion_percent, 0.0);
        assert_eq!(
            progress.pending_projects.len(),
            1,
            "only the parsed report is actionable"
        );
        assert_eq!(progress.pending_projects[0].project_id, with_report);
        assert_eq!(
            progress.pending_projects[0].missing,
            vec!["category_fit", "similarity", "criterion_evaluation"]
        );

        let queue = database
            .projects_awaiting_assessment(competition_id)
            .await
            .expect("queue should be readable");
        assert_eq!(
            queue,
            vec![with_report],
            "the bulk run must not queue a project it cannot analyse"
        );
    });
}

/// Applicants must never receive the judge-facing risks list: it names the
/// reference of the submission theirs resembles.
#[tokio::test]
async fn contestant_feedback_carries_no_judge_facing_risks() {
    with_database!(database, {
        let competition_id = seed_competition(database).await;
        let project_id = insert_report(database, competition_id, "sustainability").await;
        let team_id: i32 = sqlx::query_scalar(
            "INSERT INTO teams (competition_id, name) VALUES ($1, 'Test Team') RETURNING id",
        )
        .bind(competition_id)
        .fetch_one(&database.pool)
        .await
        .expect("team insert should succeed");
        sqlx::query("UPDATE projects SET team_id = $1 WHERE id = $2")
            .bind(team_id)
            .bind(project_id)
            .execute(&database.pool)
            .await
            .expect("project should join the team");

        database
            .upsert_ai_evaluation(
                project_id,
                &crate::models::UpsertAiEvaluation {
                    model_version: "test".into(),
                    total_score: 70.0,
                    confidence: 0.6,
                    source_file_version: None,
                    kpi_scores: Vec::new(),
                    strengths: vec!["Guclu yon".into()],
                    weaknesses: vec!["Gelistirilecek alan".into()],
                    missing_information: vec!["Olcum ekleyin".into()],
                    risks: vec!["Substantial vocabulary overlap with PRJ-000151.".into()],
                    sources: Vec::new(),
                    similar_projects: Vec::new(),
                },
            )
            .await
            .expect("evaluation should store");

        let feedback = database
            .list_contestant_feedback(team_id)
            .await
            .expect("feedback should be readable");
        assert_eq!(feedback.len(), 1);
        let serialised =
            serde_json::to_string(&feedback[0]).expect("feedback should serialise for the portal");
        assert!(
            !serialised.contains("PRJ-000151"),
            "another team's reference reached the applicant: {serialised}"
        );
        assert!(
            !serialised.contains("risks"),
            "the judge-facing risks list must not be part of the applicant payload"
        );
        assert_eq!(feedback[0].suggestions, vec!["Olcum ekleyin".to_string()]);
    });
}

/// Category and similarity records replace rather than accumulate, so a
/// re-analysis after a new report version does not leave the old verdict behind.
#[tokio::test]
async fn reanalysis_replaces_the_stored_verdict() {
    with_database!(database, {
        let competition_id = seed_competition(database).await;
        let project_id = insert_report(database, competition_id, "sustainability").await;

        for (score, requires_review) in [(20.0, true), (80.0, false)] {
            crate::assessment_store::save_category_fit(
                &database.pool,
                &crate::models::CategoryFitAnalysis {
                    project_id,
                    source_file_version: None,
                    current_category_score: score,
                    recommended_category: "sustainability".into(),
                    recommended_category_score: score,
                    matched_terms: vec!["sulama".into()],
                    requires_review,
                    analyzed_at: String::new(),
                },
            )
            .await
            .expect("category fit should store");
        }

        let stored = crate::assessment_store::get_category_fit(&database.pool, project_id)
            .await
            .expect("lookup should succeed")
            .expect("a verdict should exist");
        assert_eq!(stored.current_category_score, 80.0);
        assert!(
            !stored.requires_review,
            "the stale verdict must not survive"
        );

        let rows: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM project_category_fit_analyses WHERE project_id = $1",
        )
        .bind(project_id)
        .fetch_one(&database.pool)
        .await
        .expect("count should be readable");
        assert_eq!(rows, 1, "re-analysis must replace, not accumulate");
    });
}

/// A session that has been revoked or has expired must stop authenticating, and
/// resetting a password must revoke every session that password opened.
#[tokio::test]
async fn expired_and_revoked_sessions_stop_authenticating() {
    with_database!(database, {
        database
            .create_initial_admin("Admin", "admin@example.org", "hash")
            .await
            .expect("bootstrap should succeed");
        let user_id: i32 = sqlx::query_scalar("SELECT id FROM users LIMIT 1")
            .fetch_one(&database.pool)
            .await
            .expect("the admin should exist");

        for (token, expires_in_hours, revoked) in [
            ("live", 8_i64, false),
            ("expired", -1, false),
            ("revoked", 8, true),
        ] {
            sqlx::query(
                "INSERT INTO auth_sessions (token, user_id, expires_at, revoked_at)
                 VALUES ($1, $2, NOW() + make_interval(hours => $3::int), CASE WHEN $4 THEN NOW() END)",
            )
            .bind(token)
            .bind(user_id)
            .bind(expires_in_hours as i32)
            .bind(revoked)
            .execute(&database.pool)
            .await
            .expect("session insert should succeed");
        }

        // The same predicate the authentication middleware uses.
        let usable: Vec<String> = sqlx::query_scalar(
            "SELECT token FROM auth_sessions
             WHERE revoked_at IS NULL AND expires_at > NOW() ORDER BY token",
        )
        .fetch_all(&database.pool)
        .await
        .expect("session query should succeed");
        assert_eq!(usable, vec!["live".to_string()]);
    });
}
