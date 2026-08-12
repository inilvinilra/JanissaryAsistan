mod database;
mod models;
mod parser;
mod research;
mod scoring;

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use argon2::{
    Argon2, PasswordHasher, PasswordVerifier,
    password_hash::{PasswordHash, SaltString},
};
use axum::{
    Json, Router,
    extract::{ConnectInfo, Extension, Multipart, Path, Query, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    middleware,
    response::{
        IntoResponse,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{get, patch},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use chrono::Datelike;
use models::{
    AddTeamMember, AiEvaluation, Appeal, AuditEvent, AuthSession, BlindProject, CalibrationCase,
    CalibrationSummary, ChangePasswordRequest, Competition, CompetitionCategory, CompetitionReport,
    CompetitionStage, ConfirmPasswordResetRequest, CreateAppeal, CreateCalibrationCase,
    CreateDemoDaySlot, CreateEmailCampaign, CreateJuryAssignment, CreateJuryScore,
    CreateNotification, CreateSubmission, CreateTeam, CreateUser, DemoDaySlot, EligibilityCheck,
    EligibilityReport, EmailCampaign, FinalistSelection, FinalizeCompetition, JurorProfile,
    JuryAssignment, JuryReadiness, JuryScore, LoginRequest, Notification, OrganizationSummary,
    PasswordResetToken, Project, ProjectFile, ProjectMetadata, ProjectUpdate, RankingUpdate,
    ResolveAppeal, RoleDefinition, Submission, SubmissionVersion, Team, TeamMember,
    TwoFactorConfirm, TwoFactorConfirmation, TwoFactorSetup, UpdateDemoDaySlot, UpdateJurorProfile,
    UpdateProjectMetadata, UpdateStageStatus, UpdateTeamStatus, UpdateUser, UpsertAiEvaluation,
    User,
};
use rand_core::{OsRng, RngCore};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::{Row, postgres::PgListener};
use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    db: Arc<database::Database>,
    request_count: Arc<AtomicU64>,
    rate_limit_rejections: Arc<AtomicU64>,
    update_events: broadcast::Sender<()>,
    instance_id: String,
}

#[derive(Clone)]
struct AuthenticatedUser {
    id: i32,
    email: String,
    role: String,
    competition_id: Option<i32>,
    category: Option<String>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "backend=info,tower_http=info".into()),
        )
        .init();
    let production_mode =
        std::env::var("APP_ENV").is_ok_and(|value| value.eq_ignore_ascii_case("production"));
    if production_mode && file_encryption_key().ok().flatten().is_none() {
        panic!("A valid FILE_ENCRYPTION_KEY is required when APP_ENV=production");
    }

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not found in .env file");
    let db = database::Database::new(&database_url)
        .await
        .expect("Could not connect to database");
    tracing::info!("database connected and schema migrations completed");
    bootstrap_initial_admin(&db, production_mode)
        .await
        .expect("Could not bootstrap the initial system administrator");

    db.seed_kpi_templates()
        .await
        .expect("Failed to seed KPI templates");
    if sample_data_enabled(
        production_mode,
        std::env::var("SEED_SAMPLE_DATA").ok().as_deref(),
    ) {
        db.seed_sample_data()
            .await
            .expect("Failed to seed sample data");
    }

    let (update_events, _) = broadcast::channel(256);
    let state = AppState {
        db: Arc::new(db),
        request_count: Arc::new(AtomicU64::new(0)),
        rate_limit_rejections: Arc::new(AtomicU64::new(0)),
        update_events,
        instance_id: Uuid::new_v4().to_string(),
    };
    tokio::spawn(database_update_listener(
        state.clone(),
        database_url.clone(),
    ));
    if std::env::var("EMAIL_WEBHOOK_URL").is_ok() {
        tokio::spawn(email_delivery_worker(state.clone()));
    }
    let allowed_origin: HeaderValue = std::env::var("PUBLIC_FRONTEND_ORIGIN")
        .unwrap_or_else(|_| "http://127.0.0.1:4321".into())
        .parse()
        .expect("PUBLIC_FRONTEND_ORIGIN must be a valid origin");

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/metrics", get(metrics))
        .route("/events", get(event_stream))
        .route("/auth/login", axum::routing::post(login))
        .route("/auth/session", get(current_session))
        .route("/auth/logout", axum::routing::post(logout))
        .route("/auth/password", axum::routing::put(change_password))
        .route(
            "/auth/password-reset/confirm",
            axum::routing::post(confirm_password_reset),
        )
        .route("/auth/2fa/setup", axum::routing::post(setup_two_factor))
        .route("/auth/2fa/confirm", axum::routing::post(confirm_two_factor))
        .route("/projects", get(list_projects).post(create_project))
        .route("/projects/upload", axum::routing::post(upload_project))
        .route("/projects/{id}", get(get_project).patch(update_project))
        .route("/projects/{id}/blind", get(get_blind_project))
        .route("/projects/{id}/eligibility", get(get_eligibility_report))
        .route(
            "/projects/{id}/appeals",
            get(list_appeals).post(create_appeal),
        )
        .route("/appeals/{id}/resolve", axum::routing::post(resolve_appeal))
        .route(
            "/projects/{id}/metadata",
            get(get_project_metadata).patch(update_project_metadata),
        )
        .route(
            "/projects/{id}/files",
            get(list_project_files).post(upload_project_file),
        )
        .route("/projects/{id}/files/{file_id}", get(download_project_file))
        .route("/projects/{id}/document", get(get_project_document))
        .route("/projects/{id}/file", get(get_project_file))
        .route("/categories", get(list_categories))
        .route(
            "/categories/{category}/kpis",
            axum::routing::put(update_kpi_template),
        )
        .route(
            "/competitions",
            get(list_competitions).post(create_competition),
        )
        .route("/organizations", get(list_organizations))
        .route(
            "/competitions/{id}/stages",
            get(list_competition_stages).post(create_competition_stage),
        )
        .route(
            "/competitions/{id}/stages/{stage_id}/status",
            patch(update_stage_status),
        )
        .route(
            "/competitions/{id}/categories",
            get(list_competition_categories).post(create_competition_category),
        )
        .route(
            "/competitions/{id}/teams",
            get(list_teams).post(create_team),
        )
        .route(
            "/competitions/{id}/finalists",
            axum::routing::post(select_finalists),
        )
        .route(
            "/competitions/{id}/demo-day",
            get(list_demo_day_slots).post(create_demo_day_slot),
        )
        .route("/demo-day/{id}", patch(update_demo_day_slot))
        .route("/competitions/{id}/report", get(get_competition_report))
        .route(
            "/competitions/{id}/finalize",
            axum::routing::post(finalize_competition),
        )
        .route("/teams/{id}", patch(update_team_status))
        .route("/teams/{id}/members", axum::routing::post(add_team_member))
        .route(
            "/teams/{id}/submissions",
            get(list_submissions).post(create_submission),
        )
        .route(
            "/submissions/{id}/versions",
            get(list_submission_versions).post(upload_submission_version),
        )
        .route(
            "/projects/{id}/ai-evaluation",
            get(get_ai_evaluation).put(upsert_ai_evaluation),
        )
        .route(
            "/projects/{id}/jury-scores",
            get(list_jury_scores).post(add_jury_score),
        )
        .route(
            "/projects/{id}/jury-assignments",
            get(list_jury_assignments).post(add_jury_assignment),
        )
        .route("/projects/{id}/jury-readiness", get(get_jury_readiness))
        .route("/audit", get(list_audit))
        .route("/users", get(list_users).post(create_user))
        .route("/users/{id}", patch(update_user))
        .route(
            "/users/{id}/password-reset",
            axum::routing::post(issue_password_reset),
        )
        .route("/jurors", get(list_jurors))
        .route("/calibration", get(get_calibration_summary))
        .route(
            "/calibration/cases",
            axum::routing::post(create_calibration_case),
        )
        .route(
            "/jurors/{id}/profile",
            axum::routing::put(update_juror_profile),
        )
        .route("/roles", get(list_roles))
        .route(
            "/notifications",
            get(list_notifications).post(create_notification),
        )
        .route(
            "/email-campaigns",
            get(list_email_campaigns).post(create_email_campaign),
        )
        .route(
            "/email-campaigns/{id}/dispatch",
            axum::routing::post(dispatch_email_campaign),
        )
        .route("/ranking", patch(update_ranking))
        .route("/activity", get(list_activity))
        .route("/test/parse", get(test_parse))
        .route("/test/search", get(test_search))
        .layer(
            CorsLayer::new()
                .allow_origin(allowed_origin)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(middleware::from_fn_with_state(state.clone(), rate_limit))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            require_authenticated_request,
        ))
        .layer(middleware::from_fn(request_logging))
        .with_state(state);

    let bind_address = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:3000".into());
    let listener = tokio::net::TcpListener::bind(&bind_address).await.unwrap();
    tracing::info!(%bind_address, "server started");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

fn sample_data_enabled(production_mode: bool, configured: Option<&str>) -> bool {
    configured
        .map(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(!production_mode)
}

async fn bootstrap_initial_admin(
    db: &database::Database,
    production_mode: bool,
) -> anyhow::Result<()> {
    let email = std::env::var("BOOTSTRAP_ADMIN_EMAIL").ok();
    let password = std::env::var("BOOTSTRAP_ADMIN_PASSWORD").ok();
    match (email, password) {
        (None, None) => {
            if production_mode && !db.has_users().await? {
                anyhow::bail!(
                    "A bootstrap administrator is required for an empty production database"
                );
            }
        }
        (Some(email), Some(password)) => {
            let normalized_email = email.trim().to_lowercase();
            if normalized_email.is_empty() {
                anyhow::bail!("BOOTSTRAP_ADMIN_EMAIL must not be empty");
            }
            let password_hash = hash_password(&password).map_err(|_| {
                anyhow::anyhow!("BOOTSTRAP_ADMIN_PASSWORD must contain at least 12 characters")
            })?;
            let full_name = std::env::var("BOOTSTRAP_ADMIN_NAME")
                .unwrap_or_else(|_| "Initial System Administrator".into());
            if db
                .create_initial_admin(full_name.trim(), &normalized_email, &password_hash)
                .await?
            {
                tracing::info!("initial system administrator created");
            }
        }
        _ => {
            anyhow::bail!("BOOTSTRAP_ADMIN_EMAIL and BOOTSTRAP_ADMIN_PASSWORD must be set together")
        }
    }
    Ok(())
}

fn two_factor_is_required(role: &str) -> bool {
    let production_mode =
        std::env::var("APP_ENV").is_ok_and(|value| value.eq_ignore_ascii_case("production"));
    let policy_enabled = two_factor_policy_enabled(
        production_mode,
        std::env::var("REQUIRE_TWO_FACTOR").ok().as_deref(),
    );
    policy_enabled
        && matches!(
            role,
            "system_admin" | "competition_manager" | "chief_judge" | "jury_member"
        )
}

fn two_factor_policy_enabled(production_mode: bool, configured: Option<&str>) -> bool {
    configured
        .map(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(production_mode)
}

async fn rate_limit(
    State(state): State<AppState>,
    request: axum::extract::Request,
    next: middleware::Next,
) -> axum::response::Response {
    state.request_count.fetch_add(1, Ordering::Relaxed);
    let trust_proxy_headers = std::env::var("TRUST_PROXY_HEADERS")
        .is_ok_and(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes"));
    let peer_address = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|address| address.0.ip().to_string());
    let client_key = rate_limit_key(rate_limit_client(
        request.headers(),
        peer_address.as_deref(),
        trust_proxy_headers,
    ));
    let request_count: i32 = match sqlx::query_scalar(
        "INSERT INTO api_rate_limit_windows (client_key, window_started_at, request_count)
         VALUES ($1, date_trunc('minute', NOW()), 1)
         ON CONFLICT (client_key, window_started_at)
         DO UPDATE SET request_count = api_rate_limit_windows.request_count + 1
         RETURNING request_count",
    )
    .bind(client_key)
    .fetch_one(&state.db.pool)
    .await
    {
        Ok(count) => count,
        Err(error) => {
            tracing::error!(%error, "rate-limit database query failed");
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
    };
    if request_count > 120 {
        state.rate_limit_rejections.fetch_add(1, Ordering::Relaxed);
        return (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded").into_response();
    }
    next.run(request).await
}

async fn request_logging(
    request: axum::extract::Request,
    next: middleware::Next,
) -> axum::response::Response {
    let method = request.method().clone();
    let path = request.uri().path().to_owned();
    let started_at = Instant::now();
    let response = next.run(request).await;
    tracing::info!(
        method = %method,
        path,
        status = response.status().as_u16(),
        duration_ms = started_at.elapsed().as_millis(),
        "request completed"
    );
    response
}

async fn database_update_listener(state: AppState, database_url: String) {
    loop {
        let mut listener = match PgListener::connect(&database_url).await {
            Ok(listener) => listener,
            Err(error) => {
                tracing::error!(%error, "update listener database connection failed");
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        };
        if let Err(error) = listener.listen("jury_assistant_updates").await {
            tracing::error!(%error, "update listener subscription failed");
            tokio::time::sleep(Duration::from_secs(5)).await;
            continue;
        }
        tracing::info!("database update listener connected");
        loop {
            match listener.recv().await {
                Ok(notification) if notification.payload() != state.instance_id => {
                    let _ = state.update_events.send(());
                }
                Ok(_) => {}
                Err(error) => {
                    tracing::error!(%error, "update listener receive failed");
                    break;
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

fn rate_limit_client(
    headers: &HeaderMap,
    peer_address: Option<&str>,
    trust_proxy_headers: bool,
) -> String {
    if trust_proxy_headers {
        if let Some(client) = headers
            .get("x-forwarded-for")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(',').next())
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return client.to_string();
        }
    }
    peer_address.unwrap_or("direct").to_string()
}

fn rate_limit_key(client: String) -> String {
    Sha256::digest(client.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Every operational API route requires an active server-side session. Health
/// checks and authentication endpoints are intentionally exempt.
async fn require_authenticated_request(
    State(state): State<AppState>,
    mut request: axum::extract::Request,
    next: middleware::Next,
) -> axum::response::Response {
    if matches!(request.method(), &Method::OPTIONS)
        || matches!(request.uri().path(), "/" | "/health" | "/metrics")
        || request.uri().path().starts_with("/auth/")
    {
        return next.run(request).await;
    }
    let user_id = match authenticated_user_id(&state, request.headers()).await {
        Ok(user_id) => user_id,
        Err((status, message)) => return (status, message).into_response(),
    };
    let user = match sqlx::query(
        "SELECT email, role, competition_id, category, must_change_password, two_factor_enabled FROM users WHERE id=$1 AND active=TRUE",
    )
    .bind(user_id)
    .fetch_optional(&state.db.pool)
    .await
    {
        Ok(Some(user)) => user,
        Ok(None) => return (StatusCode::UNAUTHORIZED, "User is inactive").into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let current_user = AuthenticatedUser {
        id: user_id,
        email: user.get("email"),
        role: user.get("role"),
        competition_id: user.get("competition_id"),
        category: user.get("category"),
    };
    if user.get::<bool, _>("must_change_password") && request.uri().path() != "/auth/password" {
        return (
            StatusCode::FORBIDDEN,
            "Password change is required before accessing the dashboard",
        )
            .into_response();
    }
    if two_factor_is_required(&current_user.role) && !user.get::<bool, _>("two_factor_enabled") {
        return (
            StatusCode::FORBIDDEN,
            "Two-factor enrollment is required before accessing the dashboard",
        )
            .into_response();
    }
    let role = current_user.role.as_str();
    let scoped_competition = current_user.competition_id;
    let read_only_role = matches!(role, "observer" | "read_only");
    let writes = !matches!(
        request.method(),
        &Method::GET | &Method::HEAD | &Method::OPTIONS
    );
    if read_only_role && writes {
        return (StatusCode::FORBIDDEN, "This role has read-only access").into_response();
    }
    if role != "system_admin"
        && let (Some(allowed), Some(requested)) = (
            scoped_competition,
            competition_id_from_path(request.uri().path()),
        )
        && allowed != requested
    {
        return (
            StatusCode::FORBIDDEN,
            "You do not have access to this competition",
        )
            .into_response();
    }
    if !role_allows_request(&current_user, request.method(), request.uri().path()) {
        return (
            StatusCode::FORBIDDEN,
            "This role cannot access this resource",
        )
            .into_response();
    }
    if let Some(project_id) = project_id_from_path(request.uri().path()) {
        let scope = match state.db.get_project_scope(project_id).await {
            Ok(scope) => scope,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
        let Some((competition_id, category)) = scope else {
            return StatusCode::NOT_FOUND.into_response();
        };
        if current_user.role != "system_admin"
            && (current_user
                .competition_id
                .is_some_and(|allowed| allowed != competition_id)
                || current_user
                    .category
                    .as_deref()
                    .is_some_and(|allowed| allowed != category))
        {
            return (
                StatusCode::FORBIDDEN,
                "You do not have access to this project",
            )
                .into_response();
        }
    }
    if current_user.role != "system_admin" {
        let competition_id = match resource_competition_id(&state, request.uri().path()).await {
            Ok(competition_id) => competition_id,
            Err(status) => return status.into_response(),
        };
        if current_user
            .competition_id
            .is_some_and(|allowed| competition_id.is_some_and(|actual| actual != allowed))
        {
            return (
                StatusCode::FORBIDDEN,
                "You do not have access to this competition",
            )
                .into_response();
        }
    }
    request.extensions_mut().insert(current_user);
    let response = next.run(request).await;
    if writes && response.status().is_success() {
        let _ = state.update_events.send(());
        if let Err(error) = sqlx::query("SELECT pg_notify($1, $2)")
            .bind("jury_assistant_updates")
            .bind(&state.instance_id)
            .execute(&state.db.pool)
            .await
        {
            tracing::error!(%error, "database update notification failed");
        }
    }
    response
}

fn role_allows_request(user: &AuthenticatedUser, method: &Method, path: &str) -> bool {
    if user.role == "system_admin" {
        return true;
    }
    if path == "/events" && matches!(method, &Method::GET | &Method::HEAD) {
        return true;
    }
    if path.starts_with("/email-campaigns") || path.starts_with("/notifications") {
        return false;
    }
    let read_request = matches!(method, &Method::GET | &Method::HEAD);
    if matches!(user.role.as_str(), "observer" | "read_only") {
        return read_request && observer_can_read_path(path);
    }
    if user.role == "competition_manager" {
        return !path.starts_with("/audit")
            && !path.starts_with("/roles")
            && !path.starts_with("/users");
    }
    if user.role == "chief_judge" {
        return !path.starts_with("/users")
            && !path.starts_with("/roles")
            && !path.starts_with("/audit")
            && !path.starts_with("/email-campaigns")
            && !path.starts_with("/notifications");
    }
    if user.role == "jury_member" {
        if read_request {
            if path == "/categories" || path == "/competitions" || path == "/jurors" {
                return true;
            }
            if path.starts_with("/projects/") {
                return !(path.ends_with("/metadata")
                    || path.ends_with("/document")
                    || path.ends_with("/file")
                    || path.contains("/files/")
                    || path.ends_with("/files")
                    || path.ends_with("/appeals")
                    || path.ends_with("/jury-assignments")
                    || path.ends_with("/ai-evaluation"));
            }
            return path == "/projects";
        }
        return path.ends_with("/jury-scores")
            || (path.starts_with("/jurors/")
                && path.ends_with("/profile")
                && path_segment_id(path, "jurors") == Some(user.id));
    }
    false
}

fn observer_can_read_path(path: &str) -> bool {
    if matches!(
        path,
        "/categories" | "/competitions" | "/organizations" | "/projects"
    ) {
        return true;
    }
    path.starts_with("/competitions/") && (path.ends_with("/report") || path.ends_with("/stages"))
        || (path.starts_with("/projects/")
            && !(path.ends_with("/metadata")
                || path.ends_with("/document")
                || path.ends_with("/file")
                || path.contains("/files/")
                || path.ends_with("/files")
                || path.ends_with("/appeals")
                || path.ends_with("/jury-assignments")
                || path.ends_with("/jury-scores")
                || path.ends_with("/ai-evaluation")))
}

async fn resource_competition_id(state: &AppState, path: &str) -> Result<Option<i32>, StatusCode> {
    let result = if let Some(id) = path_segment_id(path, "teams") {
        state.db.team_competition_id(id).await
    } else if let Some(id) = path_segment_id(path, "submissions") {
        state.db.submission_competition_id(id).await
    } else if let Some(id) = path_segment_id(path, "demo-day") {
        state.db.demo_day_competition_id(id).await
    } else if let Some(id) = path_segment_id(path, "appeals") {
        state.db.appeal_competition_id(id).await
    } else {
        return Ok(None);
    };
    result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn blind_project_view(mut project: Project) -> Project {
    project.name = format!("PRJ-{:06}", project.id);
    project.team_id = None;
    project.notes.clear();
    project.tags.clear();
    project
}

fn competition_is_visible_to(user: &AuthenticatedUser, competition_id: i32) -> bool {
    user.role == "system_admin" || user.competition_id == Some(competition_id)
}

fn visible_competitions(
    user: &AuthenticatedUser,
    competitions: Vec<Competition>,
) -> Vec<Competition> {
    competitions
        .into_iter()
        .filter(|competition| competition_is_visible_to(user, competition.id))
        .collect()
}

fn competition_id_from_path(path: &str) -> Option<i32> {
    let mut segments = path.trim_matches('/').split('/');
    while let Some(segment) = segments.next() {
        if segment == "competitions" {
            return segments.next()?.parse().ok();
        }
    }
    None
}

fn path_segment_id(path: &str, prefix: &str) -> Option<i32> {
    let mut segments = path.trim_matches('/').split('/');
    while let Some(segment) = segments.next() {
        if segment == prefix {
            return segments.next()?.parse().ok();
        }
    }
    None
}

fn project_id_from_path(path: &str) -> Option<i32> {
    path_segment_id(path, "projects")
}

async fn root() -> &'static str {
    "Hello! Backend is running."
}

async fn health(State(state): State<AppState>) -> (StatusCode, &'static str) {
    match sqlx::query("SELECT 1").execute(&state.db.pool).await {
        Ok(_) => (StatusCode::OK, "OK - API and database are running"),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            "API is up but database is unreachable",
        ),
    }
}

async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    let body = format!(
        "# TYPE jury_assistant_http_requests_total counter\njury_assistant_http_requests_total {}\n# TYPE jury_assistant_rate_limit_rejections_total counter\njury_assistant_rate_limit_rejections_total {}\n",
        state.request_count.load(Ordering::Relaxed),
        state.rate_limit_rejections.load(Ordering::Relaxed),
    );
    ([(header::CONTENT_TYPE, "text/plain; version=0.0.4")], body)
}

async fn event_stream(
    Extension(_user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let stream = BroadcastStream::new(state.update_events.subscribe()).filter_map(|event| {
        event
            .ok()
            .map(|_| Ok(Event::default().event("refresh").data("updated")))
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginRequest>,
) -> Result<Json<AuthSession>, (StatusCode, String)> {
    let row = sqlx::query("SELECT id, full_name, email, role, active, must_change_password, competition_id, category, created_at, password_hash, two_factor_enabled, two_factor_secret FROM users WHERE email = $1")
        .bind(input.email.trim().to_lowercase()).fetch_optional(&state.db.pool).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid credentials".into()))?;
    let hash: Option<String> = row.get("password_hash");
    let valid = hash
        .as_deref()
        .and_then(|value| PasswordHash::new(value).ok())
        .map(|value| {
            Argon2::default()
                .verify_password(input.password.as_bytes(), &value)
                .is_ok()
        })
        .unwrap_or(false);
    if !valid || !row.get::<bool, _>("active") {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".into()));
    }
    if row.get::<bool, _>("two_factor_enabled") {
        let email: String = row.get("email");
        let secret: Option<String> = row.get("two_factor_secret");
        let valid_totp = input
            .totp_code
            .as_deref()
            .and_then(|code| {
                secret
                    .as_deref()
                    .and_then(|secret| unprotect_totp_secret(secret).ok())
                    .as_deref()
                    .and_then(|secret| totp_for_secret(secret, &email).ok())
                    .and_then(|totp| totp.check_current(code.trim()))
            })
            .is_some();
        let valid_recovery_code = if valid_totp {
            false
        } else if let Some(code) = input.totp_code.as_deref() {
            consume_recovery_code(&state, row.get::<i32, _>("id"), code).await?
        } else {
            false
        };
        if !valid_totp && !valid_recovery_code {
            return Err((
                StatusCode::UNAUTHORIZED,
                "Invalid or missing two-factor code".into(),
            ));
        }
    }
    let token = uuid::Uuid::new_v4().to_string();
    let expires = chrono::Utc::now() + chrono::Duration::hours(8);
    sqlx::query("INSERT INTO auth_sessions (token, user_id, expires_at) VALUES ($1,$2,$3)")
        .bind(&token)
        .bind(row.get::<i32, _>("id"))
        .bind(expires)
        .execute(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(AuthSession {
        token,
        expires_at: expires.to_rfc3339(),
        user: User {
            id: row.get("id"),
            full_name: row.get("full_name"),
            email: row.get("email"),
            role: row.get("role"),
            active: row.get("active"),
            must_change_password: row.get("must_change_password"),
            two_factor_enabled: row.get("two_factor_enabled"),
            two_factor_required: two_factor_is_required(&row.get::<String, _>("role")),
            competition_id: row.get("competition_id"),
            category: row.get("category"),
            created_at: row
                .try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                .map(|v| v.to_rfc3339())
                .unwrap_or_default(),
        },
    }))
}

async fn logout(State(state): State<AppState>, headers: HeaderMap) -> StatusCode {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    match token {
        Some(token) => {
            let _ = sqlx::query("UPDATE auth_sessions SET revoked_at=NOW() WHERE token=$1")
                .bind(token)
                .execute(&state.db.pool)
                .await;
            StatusCode::NO_CONTENT
        }
        None => StatusCode::UNAUTHORIZED,
    }
}

async fn current_session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<User>, (StatusCode, String)> {
    let user_id = authenticated_user_id(&state, &headers).await?;
    let row = sqlx::query("SELECT id, full_name, email, role, active, must_change_password, two_factor_enabled, competition_id, category, created_at FROM users WHERE id=$1 AND active=TRUE")
        .bind(user_id)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "Session user is inactive".into()))?;
    Ok(Json(User {
        id: row.get("id"),
        full_name: row.get("full_name"),
        email: row.get("email"),
        role: row.get("role"),
        active: row.get("active"),
        must_change_password: row.get("must_change_password"),
        two_factor_enabled: row.get("two_factor_enabled"),
        two_factor_required: two_factor_is_required(&row.get::<String, _>("role")),
        competition_id: row.get("competition_id"),
        category: row.get("category"),
        created_at: row
            .try_get::<chrono::DateTime<chrono::Utc>, _>("created_at")
            .map(|value| value.to_rfc3339())
            .unwrap_or_default(),
    }))
}

async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<ChangePasswordRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let user_id = authenticated_user_id(&state, &headers).await?;
    let row = sqlx::query("SELECT password_hash, email FROM users WHERE id=$1 AND active=TRUE")
        .bind(user_id)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "Session user is inactive".into()))?;
    let valid_current_password = row
        .get::<Option<String>, _>("password_hash")
        .as_deref()
        .and_then(|value| PasswordHash::new(value).ok())
        .map(|hash| {
            Argon2::default()
                .verify_password(input.current_password.as_bytes(), &hash)
                .is_ok()
        })
        .unwrap_or(false);
    if !valid_current_password {
        return Err((
            StatusCode::UNAUTHORIZED,
            "Current password is invalid".into(),
        ));
    }
    let password_hash = hash_password(&input.new_password).map_err(|status| {
        (
            status,
            "New password must contain at least 12 characters".into(),
        )
    })?;
    sqlx::query("UPDATE users SET password_hash=$2, must_change_password=FALSE WHERE id=$1")
        .bind(user_id)
        .bind(password_hash)
        .execute(&state.db.pool)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    state
        .db
        .record_audit(
            "password_changed",
            &row.get::<String, _>("email"),
            "user",
            Some(user_id),
            serde_json::json!({ "source": "self_service" }),
        )
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn issue_password_reset(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<PasswordResetToken>, (StatusCode, String)> {
    if actor.role != "system_admin" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only system administrators can issue password resets".into(),
        ));
    }
    let target = sqlx::query("SELECT email FROM users WHERE id=$1 AND active=TRUE")
        .bind(id)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Active user not found".into()))?;
    let token = uuid::Uuid::new_v4().to_string();
    let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
    sqlx::query(
        "UPDATE password_reset_tokens SET used_at=NOW() WHERE user_id=$1 AND used_at IS NULL",
    )
    .bind(id)
    .execute(&state.db.pool)
    .await
    .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    sqlx::query(
        "INSERT INTO password_reset_tokens (user_id, token_hash, expires_at) VALUES ($1,$2,$3)",
    )
    .bind(id)
    .bind(password_reset_token_hash(&token))
    .bind(expires_at)
    .execute(&state.db.pool)
    .await
    .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    state.db.record_audit("password_reset_issued", &actor.email, "user", Some(id), serde_json::json!({ "target_email": target.get::<String, _>("email"), "expires_at": expires_at.to_rfc3339() })).await.map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    Ok(Json(PasswordResetToken {
        token,
        expires_at: expires_at.to_rfc3339(),
    }))
}

async fn confirm_password_reset(
    State(state): State<AppState>,
    Json(input): Json<ConfirmPasswordResetRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let password_hash = hash_password(&input.new_password).map_err(|status| {
        (
            status,
            "New password must contain at least 12 characters".into(),
        )
    })?;
    let mut transaction = state
        .db
        .pool
        .begin()
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    let reset = sqlx::query("SELECT r.id, r.user_id, u.email FROM password_reset_tokens r JOIN users u ON u.id=r.user_id WHERE r.token_hash=$1 AND r.used_at IS NULL AND r.expires_at > NOW() AND u.active=TRUE FOR UPDATE")
        .bind(password_reset_token_hash(input.token.trim()))
        .fetch_optional(&mut *transaction)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "Password reset token is invalid or expired".into()))?;
    let reset_id: i32 = reset.get("id");
    let user_id: i32 = reset.get("user_id");
    let email: String = reset.get("email");
    sqlx::query("UPDATE users SET password_hash=$2, must_change_password=FALSE WHERE id=$1")
        .bind(user_id)
        .bind(password_hash)
        .execute(&mut *transaction)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    sqlx::query("UPDATE password_reset_tokens SET used_at=NOW() WHERE id=$1")
        .bind(reset_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    sqlx::query(
        "UPDATE auth_sessions SET revoked_at=NOW() WHERE user_id=$1 AND revoked_at IS NULL",
    )
    .bind(user_id)
    .execute(&mut *transaction)
    .await
    .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    transaction
        .commit()
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    state
        .db
        .record_audit(
            "password_reset_completed",
            &email,
            "user",
            Some(user_id),
            serde_json::json!({ "source": "reset_token" }),
        )
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn authenticated_user_id(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<i32, (StatusCode, String)> {
    let token = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or((StatusCode::UNAUTHORIZED, "Authentication required".into()))?;
    let row = sqlx::query("SELECT user_id FROM auth_sessions WHERE token=$1 AND revoked_at IS NULL AND expires_at > NOW()")
        .bind(token).fetch_optional(&state.db.pool).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "Session expired or revoked".into()))?;
    let _ = sqlx::query("UPDATE auth_sessions SET last_active_at=NOW() WHERE token=$1")
        .bind(token)
        .execute(&state.db.pool)
        .await;
    Ok(row.get("user_id"))
}

fn totp_for_secret(secret: &str, account: &str) -> Result<totp_rs::Totp, (StatusCode, String)> {
    let secret = totp_rs::Secret::try_from_base32(secret)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid two-factor secret".into()))?;
    totp_rs::Builder::new()
        .with_secret(secret)
        .with_account_name(account)
        .with_issuer(Some("Jury Assistant"))
        .build()
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Could not initialize two-factor verification".into(),
            )
        })
}

async fn setup_two_factor(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<TwoFactorSetup>, (StatusCode, String)> {
    let user_id = authenticated_user_id(&state, &headers).await?;
    let email: String = sqlx::query_scalar("SELECT email FROM users WHERE id=$1")
        .bind(user_id)
        .fetch_one(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    let secret = totp_rs::Secret::generate().to_base32();
    let url = totp_for_secret(&secret, &email)?.to_url().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not create two-factor setup URL".into(),
        )
    })?;
    let protected_secret = protect_totp_secret(&secret)?;
    sqlx::query("UPDATE users SET two_factor_secret=$2, two_factor_enabled=FALSE WHERE id=$1")
        .bind(user_id)
        .bind(&protected_secret)
        .execute(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(TwoFactorSetup {
        secret,
        otpauth_url: url,
    }))
}

async fn confirm_two_factor(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<TwoFactorConfirm>,
) -> Result<Json<TwoFactorConfirmation>, (StatusCode, String)> {
    let user_id = authenticated_user_id(&state, &headers).await?;
    let row = sqlx::query("SELECT email, two_factor_secret FROM users WHERE id=$1")
        .bind(user_id)
        .fetch_one(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    let email: String = row.get("email");
    let secret: Option<String> = row.get("two_factor_secret");
    let valid = secret
        .as_deref()
        .and_then(|value| unprotect_totp_secret(value).ok())
        .as_deref()
        .and_then(|value| totp_for_secret(value, &email).ok())
        .and_then(|totp| totp.check_current(input.code.trim()))
        .is_some();
    if !valid {
        return Err((StatusCode::UNAUTHORIZED, "Invalid two-factor code".into()));
    }
    let recovery_codes = generate_recovery_codes();
    let mut transaction = state
        .db
        .pool
        .begin()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    sqlx::query("UPDATE users SET two_factor_enabled=TRUE WHERE id=$1")
        .bind(user_id)
        .execute(&mut *transaction)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    sqlx::query("DELETE FROM two_factor_recovery_codes WHERE user_id=$1")
        .bind(user_id)
        .execute(&mut *transaction)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    for code in &recovery_codes {
        sqlx::query("INSERT INTO two_factor_recovery_codes (user_id, code_hash) VALUES ($1,$2)")
            .bind(user_id)
            .bind(recovery_code_hash(code))
            .execute(&mut *transaction)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }
    transaction
        .commit()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    state
        .db
        .record_audit(
            "two_factor_enabled",
            &email,
            "user",
            Some(user_id),
            serde_json::json!({}),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(TwoFactorConfirmation { recovery_codes }))
}

fn generate_recovery_codes() -> Vec<String> {
    (0..10)
        .map(|_| {
            let mut bytes = [0_u8; 5];
            OsRng.fill_bytes(&mut bytes);
            let value = bytes
                .iter()
                .map(|byte| format!("{byte:02X}"))
                .collect::<String>();
            format!("{}-{}", &value[..5], &value[5..])
        })
        .collect()
}

fn normalized_recovery_code(code: &str) -> String {
    code.trim().to_ascii_uppercase().replace('-', "")
}

fn recovery_code_hash(code: &str) -> String {
    password_reset_token_hash(&normalized_recovery_code(code))
}

async fn consume_recovery_code(
    state: &AppState,
    user_id: i32,
    code: &str,
) -> Result<bool, (StatusCode, String)> {
    let result = sqlx::query("UPDATE two_factor_recovery_codes SET used_at=NOW() WHERE id = (SELECT id FROM two_factor_recovery_codes WHERE user_id=$1 AND code_hash=$2 AND used_at IS NULL FOR UPDATE SKIP LOCKED LIMIT 1) RETURNING id")
        .bind(user_id)
        .bind(recovery_code_hash(code))
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    Ok(result.is_some())
}

async fn get_blind_project(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<BlindProject>, StatusCode> {
    let project = state
        .db
        .get_project(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(BlindProject {
        reference: format!("PRJ-{:06}", project.id),
        category: project.category,
        kpi_scores: project.kpi_scores,
        ai_score: project.ai_score,
        manual_rank: project.manual_rank,
        status: project.status,
        review_completed: project.review_completed,
    }))
}

async fn get_eligibility_report(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<EligibilityReport>, StatusCode> {
    let project = state
        .db
        .get_project(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let document = state
        .db
        .get_project_document(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let duplicate_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM projects WHERE category=$1 AND lower(trim(name))=lower(trim($2)) AND id <> $3").bind(&project.category).bind(&project.name).bind(id).fetch_one(&state.db.pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut checks = vec![
        EligibilityCheck {
            key: "file".into(),
            label: "Application file".into(),
            passed: project.has_file,
            detail: if project.has_file {
                "File available".into()
            } else {
                "Application file is missing".into()
            },
        },
        EligibilityCheck {
            key: "duplicate".into(),
            label: "Duplicate project check".into(),
            passed: duplicate_count == 0,
            detail: if duplicate_count == 0 {
                "No other project with the same category and name was found".into()
            } else {
                format!("{duplicate_count} matching project records found")
            },
        },
    ];
    match document {
        Some(document) => {
            checks.push(EligibilityCheck {
                key: "word_limit".into(),
                label: "Word limit".into(),
                passed: document.word_count <= 10_000,
                detail: format!("{} / 10,000 words", document.word_count),
            });
            checks.push(EligibilityCheck {
                key: "required_sections".into(),
                label: "Required sections".into(),
                passed: document.has_abstract
                    && document.has_methodology
                    && document.has_conclusion,
                detail: format!(
                    "Abstract: {}, methodology: {}, conclusion: {}",
                    document.has_abstract, document.has_methodology, document.has_conclusion
                ),
            });
        }
        None => checks.push(EligibilityCheck {
            key: "document_parse".into(),
            label: "Document readability".into(),
            passed: false,
            detail: "Document could not be parsed".into(),
        }),
    }
    let eligible = checks.iter().all(|check| check.passed);
    Ok(Json(EligibilityReport {
        project_id: id,
        eligible,
        checks,
    }))
}

async fn list_appeals(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<Appeal>>, StatusCode> {
    state
        .db
        .list_appeals(id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_appeal(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(input): Json<CreateAppeal>,
) -> Result<(StatusCode, Json<Appeal>), (StatusCode, String)> {
    if input.submitted_by.trim().is_empty() || input.reason.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Appeal applicant and reason are required".into(),
        ));
    }
    let appeal = state
        .db
        .create_appeal(id, &input)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    state
        .db
        .record_audit(
            "appeal_submitted",
            &actor.email,
            "appeal",
            Some(appeal.id),
            serde_json::json!({"project_id": id, "committee": appeal.committee}),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(appeal)))
}

async fn resolve_appeal(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(input): Json<ResolveAppeal>,
) -> Result<Json<Appeal>, (StatusCode, String)> {
    if !matches!(
        input.status.as_str(),
        "accepted" | "rejected" | "re_evaluation"
    ) || input.decision_reason.trim().is_empty()
        || input
            .new_score
            .is_some_and(|score| !(0.0..=100.0).contains(&score))
    {
        return Err((StatusCode::BAD_REQUEST, "Invalid appeal decision".into()));
    }
    let appeal = state
        .db
        .resolve_appeal(id, &input)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    state
        .db
        .record_audit(
            "appeal_resolved",
            &actor.email,
            "appeal",
            Some(id),
            serde_json::json!({"status": appeal.status, "new_score": input.new_score}),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(appeal))
}

async fn get_project_metadata(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<ProjectMetadata>, StatusCode> {
    state
        .db
        .get_project_metadata(id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

async fn update_project_metadata(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(input): Json<UpdateProjectMetadata>,
) -> Result<Json<ProjectMetadata>, (StatusCode, String)> {
    let metadata = state
        .db
        .update_project_metadata(id, &input)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    state.db.record_audit("project_metadata_updated", &actor.email, "project", Some(id), serde_json::json!({"institution": metadata.institution, "keyword_count": metadata.keywords.len(), "has_github": metadata.github_url.is_some(), "has_demo": metadata.demo_url.is_some()})).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(metadata))
}

async fn list_project_files(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<ProjectFile>>, StatusCode> {
    state
        .db
        .list_project_files(id)
        .await
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

async fn upload_project_file(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<ProjectFile>), (StatusCode, String)> {
    let mut file_name = None;
    let mut mime_type = "application/octet-stream".to_string();
    let mut bytes = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        if field.name().unwrap_or("") == "file" {
            file_name = field.file_name().map(str::to_string);
            if let Some(content_type) = field.content_type() {
                mime_type = content_type.to_string();
            }
            bytes = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
                    .to_vec(),
            );
        }
    }
    let file_name = file_name.ok_or((StatusCode::BAD_REQUEST, "Missing file".into()))?;
    let bytes = bytes.ok_or((StatusCode::BAD_REQUEST, "Missing file".into()))?;
    if bytes.is_empty() || bytes.len() > 25 * 1024 * 1024 {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            "File must be between 1 byte and 25 MB".into(),
        ));
    }
    let ext = std::path::Path::new(&file_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let allowed = [
        "pdf", "txt", "md", "markdown", "doc", "docx", "xls", "xlsx", "csv", "png", "jpg", "jpeg",
        "webp",
    ];
    if !allowed.contains(&ext.as_str()) {
        return Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Unsupported file type".into(),
        ));
    }
    if !valid_file_signature(&ext, &bytes) {
        return Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "File content does not match its extension".into(),
        ));
    }
    let safe_name = std::path::Path::new(&file_name)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("project-file.bin");
    let dir = format!("uploads/project-{id}");
    std::fs::create_dir_all(&dir)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = format!("{dir}/{unique}-{safe_name}");
    std::fs::write(&path, &bytes)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let virus_scan = scan_uploaded_file(&path).await?;
    std::fs::write(&path, protect_file_bytes(&bytes)?)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let file = state
        .db
        .add_project_file(id, &file_name, &mime_type, bytes.len() as i64, &path)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    state.db.record_audit("project_file_uploaded", &actor.email, "project", Some(id), serde_json::json!({"file_name": file.file_name, "version": file.version, "size_bytes": file.size_bytes, "virus_scan": virus_scan})).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(file)))
}

async fn download_project_file(
    State(state): State<AppState>,
    Path((id, file_id)): Path<(i32, i32)>,
) -> Result<impl IntoResponse, StatusCode> {
    let file = state
        .db
        .get_project_file_record(id, file_id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let bytes = tokio::fs::read(&file.file_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    Ok((
        [(header::CONTENT_TYPE, file.mime_type)],
        unprotect_file_bytes(bytes)?,
    ))
}

fn valid_file_signature(extension: &str, bytes: &[u8]) -> bool {
    match extension {
        "pdf" => bytes.starts_with(b"%PDF"),
        "png" => bytes.starts_with(&[0x89, b'P', b'N', b'G']),
        "jpg" | "jpeg" => bytes.starts_with(&[0xff, 0xd8, 0xff]),
        "webp" => bytes.len() > 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP",
        "docx" | "xlsx" | "xls" | "doc" => {
            bytes.starts_with(b"PK") || bytes.starts_with(&[0xd0, 0xcf, 0x11, 0xe0])
        }
        "txt" | "md" | "markdown" | "csv" => std::str::from_utf8(bytes).is_ok(),
        _ => false,
    }
}

const ENCRYPTED_FILE_PREFIX: &[u8] = b"JURYENC1";

fn file_encryption_key() -> Result<Option<[u8; 32]>, (StatusCode, String)> {
    parse_file_encryption_key(std::env::var("FILE_ENCRYPTION_KEY").ok().as_deref())
}

fn parse_file_encryption_key(
    encoded: Option<&str>,
) -> Result<Option<[u8; 32]>, (StatusCode, String)> {
    let Some(encoded) = encoded.filter(|value| !value.trim().is_empty()) else {
        return Ok(None);
    };
    let decoded = STANDARD.decode(encoded).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "FILE_ENCRYPTION_KEY must be base64 encoded".into(),
        )
    })?;
    let key: [u8; 32] = decoded.try_into().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "FILE_ENCRYPTION_KEY must decode to exactly 32 bytes".into(),
        )
    })?;
    Ok(Some(key))
}

fn protect_file_bytes(bytes: &[u8]) -> Result<Vec<u8>, (StatusCode, String)> {
    let Some(key) = file_encryption_key()? else {
        return Ok(bytes.to_vec());
    };
    encrypt_file_bytes(&key, bytes)
}

fn encrypt_file_bytes(key: &[u8; 32], bytes: &[u8]) -> Result<Vec<u8>, (StatusCode, String)> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not initialize file encryption".into(),
        )
    })?;
    let mut nonce_bytes = [0_u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from(nonce_bytes);
    let ciphertext = cipher.encrypt(&nonce, bytes).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Could not encrypt uploaded file".into(),
        )
    })?;
    let mut protected =
        Vec::with_capacity(ENCRYPTED_FILE_PREFIX.len() + nonce.len() + ciphertext.len());
    protected.extend_from_slice(ENCRYPTED_FILE_PREFIX);
    protected.extend_from_slice(&nonce);
    protected.extend_from_slice(&ciphertext);
    Ok(protected)
}

fn unprotect_file_bytes(bytes: Vec<u8>) -> Result<Vec<u8>, StatusCode> {
    if !bytes.starts_with(ENCRYPTED_FILE_PREFIX) {
        return Ok(bytes);
    }
    let key = file_encryption_key()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    decrypt_file_bytes(&key, bytes)
}

fn protect_totp_secret(secret: &str) -> Result<String, (StatusCode, String)> {
    let Some(key) = file_encryption_key()? else {
        return Ok(format!("plain:{secret}"));
    };
    Ok(format!(
        "enc:{}",
        STANDARD.encode(encrypt_file_bytes(&key, secret.as_bytes())?)
    ))
}

fn unprotect_totp_secret(secret: &str) -> Result<String, (StatusCode, String)> {
    if let Some(secret) = secret.strip_prefix("plain:") {
        return Ok(secret.to_string());
    }
    let Some(encoded) = secret.strip_prefix("enc:") else {
        return Ok(secret.to_string());
    };
    let bytes = STANDARD.decode(encoded).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Stored two-factor secret is invalid".into(),
        )
    })?;
    let decrypted = unprotect_file_bytes(bytes).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Stored two-factor secret cannot be decrypted".into(),
        )
    })?;
    String::from_utf8(decrypted).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Stored two-factor secret is invalid".into(),
        )
    })
}

fn decrypt_file_bytes(key: &[u8; 32], bytes: Vec<u8>) -> Result<Vec<u8>, StatusCode> {
    if bytes.len() <= ENCRYPTED_FILE_PREFIX.len() + 12 {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    let (nonce, ciphertext) = bytes[ENCRYPTED_FILE_PREFIX.len()..].split_at(12);
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let nonce = Nonce::try_from(nonce).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Uses ClamAV when available. Set VIRUS_SCAN_REQUIRED=true to reject uploads
/// while the scanner is unavailable; otherwise the audit trail records "skipped".
async fn scan_uploaded_file(path: &str) -> Result<&'static str, (StatusCode, String)> {
    let production_mode =
        std::env::var("APP_ENV").is_ok_and(|value| value.eq_ignore_ascii_case("production"));
    let required = virus_scan_required(
        production_mode,
        std::env::var("VIRUS_SCAN_REQUIRED").ok().as_deref(),
    );
    let command = std::env::var("CLAMAV_COMMAND").unwrap_or_else(|_| "clamscan".into());
    match tokio::process::Command::new(command)
        .arg("--no-summary")
        .arg(path)
        .output()
        .await
    {
        Ok(output) if output.status.code() == Some(0) => Ok("clean"),
        Ok(output) if output.status.code() == Some(1) => {
            let _ = std::fs::remove_file(path);
            Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "Upload rejected: malware detected".into(),
            ))
        }
        Ok(_) | Err(_) if required => {
            let _ = std::fs::remove_file(path);
            Err((
                StatusCode::SERVICE_UNAVAILABLE,
                "Virus scanner is unavailable; upload is blocked".into(),
            ))
        }
        Ok(_) | Err(_) => Ok("skipped"),
    }
}

fn virus_scan_required(production_mode: bool, configured: Option<&str>) -> bool {
    configured
        .filter(|value| !value.trim().is_empty())
        .map(|value| matches!(value.to_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(production_mode)
}

const USER_ROLES: [&str; 6] = [
    "system_admin",
    "competition_manager",
    "chief_judge",
    "jury_member",
    "observer",
    "read_only",
];

fn valid_role(role: &str) -> bool {
    USER_ROLES.contains(&role)
}

fn hash_password(password: &str) -> Result<String, StatusCode> {
    if password.len() < 12 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn password_reset_token_hash(token: &str) -> String {
    Sha256::digest(token.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

async fn list_roles() -> Json<Vec<RoleDefinition>> {
    Json(vec![
        RoleDefinition {
            role: "system_admin".into(),
            permissions: vec![
                "users:manage".into(),
                "competitions:manage".into(),
                "projects:manage".into(),
                "reports:view".into(),
                "audit:view".into(),
            ],
        },
        RoleDefinition {
            role: "competition_manager".into(),
            permissions: vec![
                "competitions:manage".into(),
                "projects:manage".into(),
                "jury:manage".into(),
                "reports:view".into(),
            ],
        },
        RoleDefinition {
            role: "chief_judge".into(),
            permissions: vec![
                "projects:review".into(),
                "ranking:manage".into(),
                "jury:assign".into(),
                "reports:view".into(),
            ],
        },
        RoleDefinition {
            role: "jury_member".into(),
            permissions: vec![
                "projects:review".into(),
                "jury:scores:create".into(),
                "assigned_scope:read".into(),
            ],
        },
        RoleDefinition {
            role: "observer".into(),
            permissions: vec!["projects:read".into(), "reports:view".into()],
        },
        RoleDefinition {
            role: "read_only".into(),
            permissions: vec!["projects:read".into(), "reports:view".into()],
        },
    ])
}

async fn list_users(State(state): State<AppState>) -> Result<Json<Vec<User>>, StatusCode> {
    state.db.list_users().await.map(Json).map_err(|e| {
        eprintln!("List users error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

async fn list_jurors(State(state): State<AppState>) -> Result<Json<Vec<JurorProfile>>, StatusCode> {
    state
        .db
        .list_juror_profiles()
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_calibration_summary(
    State(state): State<AppState>,
) -> Result<Json<CalibrationSummary>, StatusCode> {
    state
        .db
        .calibration_summary()
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_calibration_case(
    State(state): State<AppState>,
    Json(input): Json<CreateCalibrationCase>,
) -> Result<(StatusCode, Json<CalibrationCase>), (StatusCode, String)> {
    if !(0.0..=100.0).contains(&input.expected_score) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Expected score must be between 0 and 100".into(),
        ));
    }
    let case = state
        .db
        .create_calibration_case(&input)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(case)))
}

async fn update_juror_profile(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(input): Json<UpdateJurorProfile>,
) -> Result<StatusCode, (StatusCode, String)> {
    if input.max_assignments < 1 || input.expertise.iter().any(|item| item.trim().is_empty()) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid juror expertise or workload limit".into(),
        ));
    }
    state
        .db
        .update_juror_profile(id, &input)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    state.db.record_audit("juror_profile_updated", &actor.email, "user", Some(id), serde_json::json!({"expertise": input.expertise, "institution": input.institution, "max_assignments": input.max_assignments})).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

const NOTIFICATION_KINDS: [&str; 7] = [
    "announcement",
    "missing_document",
    "deadline",
    "review_task",
    "result",
    "question",
    "faq",
];

async fn list_notifications(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<Notification>>, StatusCode> {
    let limit = params
        .get("limit")
        .and_then(|value| value.parse().ok())
        .unwrap_or(50);
    state
        .db
        .list_notifications(limit)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_notification(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(input): Json<CreateNotification>,
) -> Result<(StatusCode, Json<Notification>), (StatusCode, String)> {
    if input.title.trim().is_empty()
        || input.body.trim().is_empty()
        || !NOTIFICATION_KINDS.contains(&input.kind.as_str())
        || input.audience.trim().is_empty()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Title, body, audience and a valid notification kind are required".into(),
        ));
    }
    let notification = state
        .db
        .create_notification(&input)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    state.db.record_audit("notification_created", &actor.email, "notification", Some(notification.id), serde_json::json!({"kind": notification.kind, "audience": notification.audience, "category": notification.category})).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(notification)))
}

async fn list_email_campaigns(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<EmailCampaign>>, StatusCode> {
    let limit = params
        .get("limit")
        .and_then(|value| value.parse().ok())
        .unwrap_or(50);
    state
        .db
        .list_email_campaigns(limit)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_email_campaign(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(input): Json<CreateEmailCampaign>,
) -> Result<(StatusCode, Json<EmailCampaign>), (StatusCode, String)> {
    if input.subject.trim().is_empty()
        || input.body.trim().is_empty()
        || input.audience.trim().is_empty()
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Subject, body and audience are required".into(),
        ));
    }
    let campaign = state
        .db
        .create_email_campaign(&input)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    state.db.record_audit("email_campaign_queued", &actor.email, "email_campaign", Some(campaign.id), serde_json::json!({"audience": campaign.audience, "recipient_count": campaign.recipient_count, "category": campaign.category})).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(campaign)))
}

async fn dispatch_email_campaign(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<EmailCampaign>, (StatusCode, String)> {
    let campaign = state
        .db
        .get_email_campaign(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Email campaign not found".into()))?;
    if std::env::var("EMAIL_WEBHOOK_URL").is_err() {
        return Err((
            StatusCode::CONFLICT,
            "EMAIL_WEBHOOK_URL is not configured; campaign remains queued".into(),
        ));
    }
    state
        .db
        .set_email_campaign_status(id, "queued")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let worker_state = state.clone();
    tokio::spawn(async move {
        let _ = process_email_campaign(&worker_state, id).await;
    });
    state
        .db
        .record_audit(
            "email_campaign_dispatched",
            &actor.email,
            "email_campaign",
            Some(id),
            serde_json::json!({"recipient_count": campaign.recipient_count, "mode": "queued"}),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    state
        .db
        .get_email_campaign(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map(Json)
        .ok_or((StatusCode::NOT_FOUND, "Email campaign not found".into()))
}

fn email_retry_delay(attempt_count: i32) -> Duration {
    Duration::from_secs(
        (30_u64.saturating_mul(2_u64.saturating_pow(attempt_count.saturating_sub(1) as u32)))
            .min(3600),
    )
}

async fn email_delivery_worker(state: AppState) {
    loop {
        if let Ok(campaigns) = state.db.list_email_campaigns(200).await {
            for campaign in campaigns
                .into_iter()
                .filter(|campaign| campaign.status == "queued")
            {
                let _ = process_email_campaign(&state, campaign.id).await;
            }
        }
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}

async fn process_email_campaign(state: &AppState, campaign_id: i32) -> anyhow::Result<()> {
    let webhook_url = std::env::var("EMAIL_WEBHOOK_URL")?;
    let campaign = state
        .db
        .get_email_campaign(campaign_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Email campaign not found"))?;
    let deliveries = state
        .db
        .claim_queued_email_deliveries(campaign_id, 100)
        .await?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    for delivery in deliveries {
        let response = client.post(&webhook_url).json(&serde_json::json!({"to": delivery.email, "subject": campaign.subject, "body": campaign.body, "campaign_id": campaign.id, "delivery_id": delivery.id})).send().await;
        let error = match response {
            Ok(response) if response.status().is_success() => None,
            Ok(response) => Some(format!("Webhook returned {}", response.status())),
            Err(error) => Some(error.to_string()),
        };
        let next_attempt_at = error
            .as_ref()
            .filter(|_| delivery.attempt_count < 5)
            .map(|_| {
                chrono::Utc::now()
                    + chrono::Duration::from_std(email_retry_delay(delivery.attempt_count))
                        .unwrap_or_else(|_| chrono::Duration::hours(1))
            });
        state
            .db
            .complete_email_delivery(&delivery, error.as_deref(), next_attempt_at)
            .await?;
    }
    state.db.refresh_email_campaign_status(campaign_id).await?;
    Ok(())
}

async fn create_user(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(mut input): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), StatusCode> {
    input.full_name = input.full_name.trim().to_string();
    input.email = input.email.trim().to_lowercase();
    if input.full_name.is_empty()
        || !input.email.contains('@')
        || !valid_role(&input.role)
        || input.password.is_none()
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    input.password = input.password.as_deref().map(hash_password).transpose()?;
    let user = state.db.create_user(&input).await.map_err(|e| {
        eprintln!("Create user error: {e}");
        StatusCode::BAD_REQUEST
    })?;
    state.db.record_audit("user_created", &actor.email, "user", Some(user.id), serde_json::json!({
        "role": user.role, "active": user.active, "competition_id": user.competition_id, "category": user.category,
    })).await.map_err(|e| { eprintln!("Audit error: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;
    Ok((StatusCode::CREATED, Json(user)))
}

async fn update_user(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(mut input): Json<UpdateUser>,
) -> Result<Json<User>, StatusCode> {
    if let Some(role) = input.role.as_deref() {
        if !valid_role(role) {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    input.password = input.password.as_deref().map(hash_password).transpose()?;
    let before = state
        .db
        .list_users()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .find(|user| user.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;
    let after = state.db.update_user(id, &input).await.map_err(|e| {
        eprintln!("Update user error: {e}");
        StatusCode::NOT_FOUND
    })?;
    state.db.record_audit("user_updated", &actor.email, "user", Some(id), serde_json::json!({
        "before": { "role": before.role, "active": before.active, "competition_id": before.competition_id, "category": before.category },
        "after": { "role": after.role, "active": after.active, "competition_id": after.competition_id, "category": after.category },
    })).await.map_err(|e| { eprintln!("Audit error: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;
    Ok(Json(after))
}

async fn list_projects(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<Project>>, StatusCode> {
    let category = params.get("category").map(|s| s.as_str());
    if actor.role != "system_admin"
        && let Some(allowed_category) = actor.category.as_deref()
        && category != Some(allowed_category)
    {
        return Err(StatusCode::FORBIDDEN);
    }

    let competition_id = if actor.role == "system_admin" {
        None
    } else {
        actor.competition_id
    };
    let mut projects = state
        .db
        .list_projects(category, competition_id)
        .await
        .map_err(|e| {
            eprintln!("List error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    projects.sort_by(|a, b| match (a.manual_rank, b.manual_rank) {
        (Some(x), Some(y)) => x.cmp(&y),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => b.ai_score.partial_cmp(&a.ai_score).unwrap(),
    });

    if actor.role == "jury_member" {
        projects = projects.into_iter().map(blind_project_view).collect();
    }

    Ok(Json(projects))
}

#[derive(Deserialize)]
struct CreateProjectRequest {
    competition_id: i32,
    team_id: Option<i32>,
    name: String,
    category: String,
    file_path: String,
}

async fn create_project(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<Project>, (StatusCode, String)> {
    validate_project_scope(
        &state,
        &actor,
        req.competition_id,
        req.team_id,
        &req.category,
    )
    .await?;
    let document = parser::parse_file(&req.file_path)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Parse error: {e}")))?;

    score_and_store(
        &state,
        &actor.email,
        req.competition_id,
        req.team_id,
        &req.name,
        &req.category,
        document,
        Some(&req.file_path),
    )
    .await
}

async fn upload_project(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Project>, (StatusCode, String)> {
    let mut name: Option<String> = None;
    let mut category: Option<String> = None;
    let mut competition_id: Option<i32> = None;
    let mut team_id: Option<i32> = None;
    let mut filename: Option<String> = None;
    let mut file_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        match field.name().unwrap_or("") {
            "name" => {
                name = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?,
                )
            }
            "category" => {
                category = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?,
                )
            }
            "competition_id" => {
                competition_id = field.text().await.ok().and_then(|value| value.parse().ok());
            }
            "team_id" => {
                team_id = field.text().await.ok().and_then(|value| value.parse().ok());
            }
            "file" => {
                filename = field.file_name().map(str::to_string);
                file_bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
                        .to_vec(),
                );
            }
            _ => {}
        }
    }

    let name = name.ok_or((StatusCode::BAD_REQUEST, "Missing 'name' field".to_string()))?;
    let category = category.ok_or((
        StatusCode::BAD_REQUEST,
        "Missing 'category' field".to_string(),
    ))?;
    let competition_id = competition_id.ok_or((
        StatusCode::BAD_REQUEST,
        "Missing competition_id field".to_string(),
    ))?;
    validate_project_scope(&state, &actor, competition_id, team_id, &category).await?;
    let filename = filename.ok_or((StatusCode::BAD_REQUEST, "Missing file".to_string()))?;
    let file_bytes = file_bytes.ok_or((StatusCode::BAD_REQUEST, "Missing file".to_string()))?;
    if file_bytes.is_empty() || file_bytes.len() > 25 * 1024 * 1024 {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            "File must be between 1 byte and 25 MB".into(),
        ));
    }

    std::fs::create_dir_all("uploads")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let ext = std::path::Path::new(&filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    if ![
        "pdf", "txt", "md", "markdown", "docx", "xlsx", "xls", "csv", "png", "jpg", "jpeg", "webp",
    ]
    .contains(&ext.as_str())
        || !valid_file_signature(&ext, &file_bytes)
    {
        return Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Only valid PDF, TXT, Markdown, DOCX, XLSX, XLS, CSV, and OCR-supported image files can be evaluated".into(),
        ));
    }
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let stored_path = format!("uploads/{unique}.{ext}");
    std::fs::write(&stored_path, &file_bytes)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    scan_uploaded_file(&stored_path).await?;

    let document = parser::parse_file(&stored_path)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Parse error: {e}")))?;
    std::fs::write(&stored_path, protect_file_bytes(&file_bytes)?)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    score_and_store(
        &state,
        &actor.email,
        competition_id,
        team_id,
        &name,
        &category,
        document,
        Some(&stored_path),
    )
    .await
}

async fn score_and_store(
    state: &AppState,
    actor: &str,
    competition_id: i32,
    team_id: Option<i32>,
    name: &str,
    category: &str,
    document: models::Document,
    file_path: Option<&str>,
) -> Result<Json<Project>, (StatusCode, String)> {
    let template = state
        .db
        .list_categories()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .into_iter()
        .find(|t| t.category == category)
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                format!("Unknown category: {}", category),
            )
        })?;

    let scorer = scoring::configured_scorer();
    let kpi_scores = scoring::score_project(&scorer, &document, &template.kpis)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let id = state
        .db
        .insert_project(
            competition_id,
            team_id,
            name,
            category,
            kpi_scores,
            Some(&document),
            file_path,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state.db.record_audit("project_uploaded", actor, "project", Some(id), serde_json::json!({
        "competition_id": competition_id, "team_id": team_id, "name": name, "category": category, "file_path": file_path, "document_filename": document.filename,
        "kpi_template": template.kpis.iter().map(|kpi| &kpi.name).collect::<Vec<_>>(),
    })).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state
        .db
        .get_project(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map(Json)
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Project not found after insert".to_string(),
        ))
}

async fn validate_project_scope(
    state: &AppState,
    actor: &AuthenticatedUser,
    competition_id: i32,
    team_id: Option<i32>,
    category: &str,
) -> Result<(), (StatusCode, String)> {
    if actor.role != "system_admin" && actor.competition_id.is_some_and(|id| id != competition_id) {
        return Err((
            StatusCode::FORBIDDEN,
            "You do not have access to this competition".into(),
        ));
    }
    if actor.role != "system_admin"
        && actor
            .category
            .as_deref()
            .is_some_and(|allowed| allowed != category)
    {
        return Err((
            StatusCode::FORBIDDEN,
            "You do not have access to this category".into(),
        ));
    }
    if let Some(team_id) = team_id {
        let belongs = state
            .db
            .team_belongs_to_competition(team_id, competition_id)
            .await
            .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
        if !belongs {
            return Err((
                StatusCode::BAD_REQUEST,
                "Team does not belong to the selected competition".into(),
            ));
        }
    }
    Ok(())
}

async fn list_categories(
    State(state): State<AppState>,
) -> Result<Json<Vec<models::CategoryTemplate>>, StatusCode> {
    state.db.list_categories().await.map(Json).map_err(|e| {
        eprintln!("Categories error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

#[derive(Debug, Deserialize)]
struct KpiTemplateUpdate {
    kpis: Vec<models::KpiTemplate>,
}

async fn update_kpi_template(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(category): Path<String>,
    Json(update): Json<KpiTemplateUpdate>,
) -> Result<Json<models::CategoryTemplate>, (StatusCode, String)> {
    if update.kpis.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "At least one KPI is required".into(),
        ));
    }
    if update
        .kpis
        .iter()
        .any(|kpi| kpi.name.trim().is_empty() || !(0.0..=100.0).contains(&kpi.weight))
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "KPI names are required and weights must be between 0 and 100".into(),
        ));
    }
    let total: f64 = update.kpis.iter().map(|kpi| kpi.weight).sum();
    if (total - 100.0).abs() > 0.01 {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("KPI weights must total 100 (received {total:.2})"),
        ));
    }
    state
        .db
        .replace_kpi_template(&category, &update.kpis)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    state
        .db
        .record_audit(
            "kpi_template_updated",
            &actor.email,
            "category",
            None,
            serde_json::json!({
                "category": category, "kpis": &update.kpis,
            }),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(models::CategoryTemplate {
        category,
        kpis: update.kpis,
    }))
}

#[derive(Debug, Deserialize)]
struct CreateCompetitionRequest {
    name: String,
    description: Option<String>,
    application_start: Option<String>,
    application_end: Option<String>,
    organization: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateStageRequest {
    name: String,
    stage_type: String,
    position: i32,
    starts_at: Option<String>,
    ends_at: Option<String>,
    passing_score: Option<f64>,
    finalist_limit: Option<i32>,
    results_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateCompetitionCategoryRequest {
    name: String,
    slug: String,
    parent_id: Option<i32>,
    kpi_category: Option<String>,
}

async fn list_competitions(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<Competition>>, StatusCode> {
    let competitions = state.db.list_competitions().await.map_err(|e| {
        eprintln!("Competitions error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(Json(visible_competitions(&actor, competitions)))
}

async fn list_organizations(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> Result<Json<Vec<OrganizationSummary>>, StatusCode> {
    let competitions = state
        .db
        .list_competitions()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut summaries = BTreeMap::<String, (i64, i64)>::new();
    for competition in visible_competitions(&actor, competitions) {
        let entry = summaries.entry(competition.organization).or_default();
        entry.0 += 1;
        entry.1 += i64::from(matches!(
            competition.status,
            models::CompetitionStatus::Archived
        ));
    }
    Ok(Json(
        summaries
            .into_iter()
            .map(
                |(organization, (competition_count, archived_count))| OrganizationSummary {
                    organization,
                    competition_count,
                    archived_count,
                },
            )
            .collect(),
    ))
}

async fn create_competition(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(req): Json<CreateCompetitionRequest>,
) -> Result<Json<Competition>, (StatusCode, String)> {
    if actor.role != "system_admin" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only system administrators can create competitions".into(),
        ));
    }
    if req.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Competition name is required".into(),
        ));
    }
    let id = state
        .db
        .create_competition(
            req.name.trim(),
            req.description.as_deref().unwrap_or(""),
            req.application_start.as_deref(),
            req.application_end.as_deref(),
            req.organization.as_deref().unwrap_or("T3 Foundation"),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let competition = state
        .db
        .list_competitions()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .into_iter()
        .find(|item| item.id == id)
        .ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Competition not found after insert".into(),
        ))?;
    Ok(Json(competition))
}

async fn list_competition_stages(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<CompetitionStage>>, StatusCode> {
    state
        .db
        .list_competition_stages(id)
        .await
        .map(Json)
        .map_err(|e| {
            eprintln!("Competition stages error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

async fn create_competition_stage(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<CreateStageRequest>,
) -> Result<Json<CompetitionStage>, (StatusCode, String)> {
    if req.name.trim().is_empty() || req.stage_type.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Stage name and type are required".into(),
        ));
    }
    if req.position < 1 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Stage position must be positive".into(),
        ));
    }
    if req.passing_score.unwrap_or(0.0) < 0.0
        || req.passing_score.unwrap_or(0.0) > 100.0
        || req.finalist_limit.unwrap_or(0) < 0
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid stage threshold or finalist limit".into(),
        ));
    }
    state
        .db
        .add_competition_stage(
            id,
            req.name.trim(),
            req.stage_type.trim(),
            req.position,
            req.starts_at.as_deref(),
            req.ends_at.as_deref(),
            req.passing_score.unwrap_or(0.0),
            req.finalist_limit.filter(|value| *value > 0),
            req.results_at.as_deref(),
        )
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

async fn update_stage_status(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path((competition_id, stage_id)): Path<(i32, i32)>,
    Json(input): Json<UpdateStageStatus>,
) -> Result<Json<CompetitionStage>, (StatusCode, String)> {
    if !matches!(
        input.status.as_str(),
        "planned" | "active" | "completed" | "locked"
    ) {
        return Err((StatusCode::BAD_REQUEST, "Invalid stage status".into()));
    }
    let stages = state
        .db
        .list_competition_stages(competition_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let current = stages
        .iter()
        .find(|stage| stage.id == stage_id)
        .ok_or((StatusCode::NOT_FOUND, "Stage not found".into()))?;
    let allowed = matches!(
        (&current.status[..], input.status.as_str()),
        ("planned", "active") | ("active", "completed") | ("completed", "locked")
    );
    if !allowed {
        return Err((
            StatusCode::CONFLICT,
            format!(
                "Invalid stage transition: {} -> {}",
                current.status, input.status
            ),
        ));
    }
    let stage = state
        .db
        .update_stage_status(stage_id, &input.status)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    state.db.record_audit("stage_status_updated", &actor.email, "competition_stage", Some(stage_id), serde_json::json!({"competition_id": competition_id, "before": current.status, "after": input.status})).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(stage))
}

async fn list_competition_categories(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<CompetitionCategory>>, StatusCode> {
    state
        .db
        .list_competition_categories(id)
        .await
        .map(Json)
        .map_err(|e| {
            eprintln!("Competition categories error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

async fn create_competition_category(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<CreateCompetitionCategoryRequest>,
) -> Result<Json<CompetitionCategory>, (StatusCode, String)> {
    if req.name.trim().is_empty() || req.slug.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Category name and slug are required".into(),
        ));
    }
    state
        .db
        .add_competition_category(
            id,
            req.parent_id,
            req.name.trim(),
            req.slug.trim(),
            req.kpi_category.as_deref(),
        )
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

async fn list_teams(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<Team>>, (StatusCode, String)> {
    state
        .db
        .list_teams(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn create_team(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(input): Json<CreateTeam>,
) -> Result<Json<Team>, (StatusCode, String)> {
    if input.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Team name is required".into()));
    }
    state
        .db
        .create_team(id, &input)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

async fn update_team_status(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(input): Json<UpdateTeamStatus>,
) -> Result<StatusCode, (StatusCode, String)> {
    const ALLOWED: &[&str] = &["new", "reviewing", "finalist", "rejected", "winner"];
    if !ALLOWED.contains(&input.status.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "Invalid team status".into()));
    }
    state
        .db
        .update_team_status(id, &input.status)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn select_finalists(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(input): Json<FinalistSelection>,
) -> Result<StatusCode, (StatusCode, String)> {
    if input.team_ids.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "At least one finalist team is required".into(),
        ));
    }
    state
        .db
        .select_finalists(id, &input.team_ids)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_demo_day_slots(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<DemoDaySlot>>, StatusCode> {
    state
        .db
        .list_demo_day_slots(id)
        .await
        .map(Json)
        .map_err(|e| {
            eprintln!("Demo Day slots error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

async fn create_demo_day_slot(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(input): Json<CreateDemoDaySlot>,
) -> Result<Json<DemoDaySlot>, (StatusCode, String)> {
    if input.slot_order < 1 || input.room.trim().is_empty() || input.starts_at.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Slot order, room and start time are required".into(),
        ));
    }
    state
        .db
        .add_demo_day_slot(id, &input)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

async fn update_demo_day_slot(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(input): Json<UpdateDemoDaySlot>,
) -> Result<Json<DemoDaySlot>, (StatusCode, String)> {
    if input
        .field_score
        .is_some_and(|score| !(0.0..=100.0).contains(&score))
        || input.evidence_urls.as_ref().is_some_and(|urls| {
            urls.iter()
                .any(|url| !url.starts_with("http://") && !url.starts_with("https://"))
        })
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid field score or evidence URL".into(),
        ));
    }
    let slot = state
        .db
        .update_demo_day_slot(id, &input)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;
    state.db.record_audit("demo_day_slot_updated", &actor.email, "demo_day_slot", Some(id), serde_json::json!({"status": slot.status, "checked_in": slot.checked_in_at.is_some(), "field_score": slot.field_score, "evidence_count": slot.evidence_urls.len(), "signed": slot.jury_signature.is_some()})).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(slot))
}

async fn get_competition_report(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<CompetitionReport>, StatusCode> {
    state
        .db
        .competition_report(id)
        .await
        .map(Json)
        .map_err(|e| {
            eprintln!("Competition report error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

async fn finalize_competition(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(input): Json<FinalizeCompetition>,
) -> Result<StatusCode, (StatusCode, String)> {
    if input.minutes.trim().is_empty() || input.signed_by.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Final minutes and signer are required".into(),
        ));
    }
    state
        .db
        .finalize_competition(id, &input.minutes, &input.signed_by)
        .await
        .map_err(|e| (StatusCode::CONFLICT, e.to_string()))?;
    state
        .db
        .record_audit(
            "competition_results_locked",
            &actor.email,
            "competition",
            Some(id),
            serde_json::json!({"final_minutes_recorded": true}),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn add_team_member(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(input): Json<AddTeamMember>,
) -> Result<Json<TeamMember>, (StatusCode, String)> {
    if input.full_name.trim().is_empty() || input.email.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Member name and email are required".into(),
        ));
    }
    if input
        .birth_year
        .is_some_and(|year| !(1900..=chrono::Utc::now().year()).contains(&year))
        || input
            .education_level
            .as_deref()
            .is_some_and(|level| level.trim().is_empty())
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid age or education information".into(),
        ));
    }
    state
        .db
        .add_team_member(id, &input)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

async fn list_submissions(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<Submission>>, (StatusCode, String)> {
    state
        .db
        .list_submissions(id)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn create_submission(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(input): Json<CreateSubmission>,
) -> Result<Json<Submission>, (StatusCode, String)> {
    if input.title.trim().is_empty() || input.file_name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Submission title and file name are required".into(),
        ));
    }
    let submission = state
        .db
        .create_submission(id, &input)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    state
        .db
        .record_audit(
            "submission_created",
            &actor.email,
            "submission",
            Some(submission.id),
            serde_json::json!({"team_id": id, "stage_id": submission.stage_id, "is_late": submission.is_late}),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(submission))
}

async fn upload_submission_version(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    mut multipart: Multipart,
) -> Result<Json<SubmissionVersion>, (StatusCode, String)> {
    let mut filename: Option<String> = None;
    let mut bytes: Option<Vec<u8>> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        if field.name().unwrap_or("") == "file" {
            filename = field.file_name().map(str::to_string);
            bytes = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
                    .to_vec(),
            );
        }
    }
    let filename = filename.ok_or((StatusCode::BAD_REQUEST, "Missing file".into()))?;
    let bytes = bytes.ok_or((StatusCode::BAD_REQUEST, "Missing file".into()))?;
    if bytes.is_empty() || bytes.len() > 25 * 1024 * 1024 {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            "File must be between 1 byte and 25 MB".into(),
        ));
    }
    let ext = std::path::Path::new(&filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let allowed = [
        "pdf", "txt", "md", "markdown", "doc", "docx", "xls", "xlsx", "csv", "png", "jpg", "jpeg",
        "webp",
    ];
    if !allowed.contains(&ext.as_str()) || !valid_file_signature(&ext, &bytes) {
        return Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "File content does not match an accepted file type".into(),
        ));
    }
    std::fs::create_dir_all("uploads/submissions")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let safe_name = std::path::Path::new(&filename)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("submission.bin");
    let stored_path = format!("uploads/submissions/{unique}-{safe_name}");
    std::fs::write(&stored_path, bytes)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    scan_uploaded_file(&stored_path).await?;
    let original = std::fs::read(&stored_path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    std::fs::write(&stored_path, protect_file_bytes(&original)?)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    state
        .db
        .add_submission_version(id, &filename, &stored_path)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

async fn list_submission_versions(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<SubmissionVersion>>, StatusCode> {
    state
        .db
        .list_submission_versions(id)
        .await
        .map(Json)
        .map_err(|e| {
            eprintln!("Submission versions error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

async fn get_ai_evaluation(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<AiEvaluation>, StatusCode> {
    state
        .db
        .get_ai_evaluation(id)
        .await
        .map_err(|e| {
            eprintln!("AI evaluation error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn upsert_ai_evaluation(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(input): Json<UpsertAiEvaluation>,
) -> Result<Json<AiEvaluation>, (StatusCode, String)> {
    if input.model_version.trim().is_empty()
        || !(0.0..=100.0).contains(&input.total_score)
        || !(0.0..=1.0).contains(&input.confidence)
        || input.kpi_scores.iter().any(|kpi| {
            kpi.name.trim().is_empty()
                || !(0.0..=100.0).contains(&kpi.score)
                || !(0.0..=1.0).contains(&kpi.confidence)
        })
        || input.similar_projects.iter().any(|project| {
            project.name.trim().is_empty() || !(0.0..=1.0).contains(&project.similarity)
        })
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Model version, scores (0-100) and confidence values (0-1) are required".into(),
        ));
    }
    let evaluation = state
        .db
        .upsert_ai_evaluation(id, &input)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    state
        .db
        .record_audit(
            "ai_evaluation_updated",
            &actor.email,
            "project",
            Some(id),
            serde_json::json!({
                "total_score": input.total_score,
                "confidence": input.confidence,
                "model_version": input.model_version,
            }),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(evaluation))
}

async fn list_jury_scores(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<JuryScore>>, StatusCode> {
    state.db.list_jury_scores(id).await.map(Json).map_err(|e| {
        eprintln!("Jury scores error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

async fn add_jury_score(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(mut input): Json<CreateJuryScore>,
) -> Result<Json<JuryScore>, (StatusCode, String)> {
    input.juror_name = actor.email.clone();
    if !(0.0..=100.0).contains(&input.total_score) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Score must be between 0 and 100".into(),
        ));
    }
    if let Some(stage_id) = input.stage_id {
        let stage_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM competition_stages WHERE id = $1)",
        )
        .bind(stage_id)
        .fetch_one(&state.db.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        if !stage_exists {
            return Err((
                StatusCode::BAD_REQUEST,
                "Selected evaluation stage does not exist".into(),
            ));
        }
    }
    let score = state
        .db
        .add_jury_score(id, &input)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    state
        .db
        .record_audit(
            "jury_score_submitted",
            &actor.email,
            "project",
            Some(id),
            serde_json::json!({
                "total_score": input.total_score,
                "stage_id": input.stage_id,
            }),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(score))
}

async fn list_jury_assignments(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<JuryAssignment>>, StatusCode> {
    state
        .db
        .list_jury_assignments(id)
        .await
        .map(Json)
        .map_err(|e| {
            eprintln!("Jury assignments error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

async fn add_jury_assignment(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(input): Json<CreateJuryAssignment>,
) -> Result<Json<JuryAssignment>, (StatusCode, String)> {
    if input.juror_name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Juror name is required".into()));
    }
    let conflict = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM users u JOIN juror_profiles jp ON jp.user_id=u.id JOIN project_metadata pm ON pm.project_id=$1 WHERE u.full_name=$2 AND ((jsonb_typeof(pm.team_members)='array' AND pm.team_members @> jsonb_build_array(u.email)) OR (jp.institution <> '' AND lower(jp.institution)=lower(pm.institution))))")
        .bind(id).bind(&input.juror_name).fetch_one(&state.db.pool).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    if conflict && !input.conflict_declared.unwrap_or(false) {
        return Err((
            StatusCode::CONFLICT,
            "Juror has an ownership or institution conflict; declare it before assignment".into(),
        ));
    }
    let assignment = state
        .db
        .add_jury_assignment(id, &input)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    state
        .db
        .record_audit(
            "jury_assigned",
            &actor.email,
            "project",
            Some(id),
            serde_json::json!({"assigned_juror": assignment.juror_name, "role": assignment.role}),
        )
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?;
    Ok(Json(assignment))
}

async fn get_jury_readiness(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<JuryReadiness>, StatusCode> {
    let assigned_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM jury_assignments WHERE project_id=$1 AND conflict_declared=FALSE",
    )
    .bind(id)
    .fetch_one(&state.db.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(JuryReadiness {
        project_id: id,
        assigned_count,
        minimum_required: 3,
        ready: assigned_count >= 3,
    }))
}

async fn list_audit(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<AuditEvent>>, StatusCode> {
    let limit = params
        .get("limit")
        .and_then(|value| value.parse().ok())
        .unwrap_or(50);
    state.db.list_audit(limit).await.map(Json).map_err(|e| {
        eprintln!("Audit error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

async fn get_project(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Project>, StatusCode> {
    let project = state
        .db
        .get_project(id)
        .await
        .map_err(|e| {
            eprintln!("Detail error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;
    if let Err(error) = state
        .db
        .record_audit(
            "project_opened",
            &actor.email,
            "project",
            Some(id),
            serde_json::json!({}),
        )
        .await
    {
        eprintln!("Audit error: {error}");
    }
    Ok(Json(if actor.role == "jury_member" {
        blind_project_view(project)
    } else {
        project
    }))
}

async fn update_project(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(update): Json<ProjectUpdate>,
) -> Result<Json<Project>, (StatusCode, String)> {
    if let Some(status) = update.status.as_deref() {
        if !matches!(status, "new" | "reviewing" | "finalist" | "rejected") {
            return Err((StatusCode::BAD_REQUEST, "Invalid project status".into()));
        }
    }
    let before = state
        .db
        .get_project(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Project not found".to_string()))?;
    state
        .db
        .update_project(
            id,
            update.notes.as_deref(),
            update.status.as_deref(),
            update.review_completed,
            update.tags.as_deref(),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let after = state
        .db
        .get_project(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Project not found".to_string()))?;
    state.db.record_audit("project_updated", &actor.email, "project", Some(id), serde_json::json!({
        "before": { "status": before.status, "notes": before.notes, "review_completed": before.review_completed, "tags": before.tags },
        "after": { "status": after.status, "notes": after.notes, "review_completed": after.review_completed, "tags": after.tags },
    })).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(after))
}

async fn get_project_file(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    let path = state.db.get_project_file_path(id).await.map_err(|e| {
        eprintln!("File path fetch error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let Some(path) = path else {
        return Err(StatusCode::NOT_FOUND);
    };

    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let bytes = unprotect_file_bytes(bytes)?;

    let content_type = match std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
    {
        Some("pdf") => "application/pdf",
        Some("md") | Some("markdown") => "text/markdown; charset=utf-8",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    };

    Ok(([(header::CONTENT_TYPE, content_type)], bytes))
}

async fn get_project_document(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<models::Document>, StatusCode> {
    state
        .db
        .get_project_document(id)
        .await
        .map_err(|e| {
            eprintln!("Document fetch error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn list_activity(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<models::ActivityEntry>>, StatusCode> {
    let category = params.get("category").map(|s| s.as_str());
    let limit = params
        .get("limit")
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);

    state
        .db
        .list_activity(category, limit)
        .await
        .map(Json)
        .map_err(|e| {
            eprintln!("Activity error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}

async fn update_ranking(
    Extension(actor): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(update): Json<RankingUpdate>,
) -> StatusCode {
    let changed_by = actor.email.as_str();
    match state
        .db
        .update_ranking(&update.category, &update.order, changed_by)
        .await
    {
        Ok(_) => {
            if let Err(e) = state
                .db
                .record_audit(
                    "ranking_updated",
                    changed_by,
                    "category",
                    None,
                    serde_json::json!({
                        "category": update.category,
                        "order": update.order,
                    }),
                )
                .await
            {
                eprintln!("Audit error: {e}");
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            StatusCode::OK
        }
        Err(e) => {
            eprintln!("Ranking update error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

async fn test_parse() -> Result<Json<models::Document>, StatusCode> {
    match parser::parse_file("samples/sample-project.md") {
        Ok(document) => Ok(Json(document)),
        Err(e) => {
            eprintln!("Parse error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn test_search(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<models::SearchResult>>, (StatusCode, String)> {
    let api_key = std::env::var("BRAVE_API_KEY").map_err(|_| {
        (
            StatusCode::PRECONDITION_FAILED,
            "BRAVE_API_KEY is not set. Add it to backend/.env.".to_string(),
        )
    })?;

    let query = params.get("q").cloned().unwrap_or_default();
    let keywords: Vec<String> = query.split(',').map(|s| s.trim().to_string()).collect();

    research::search_related_sources(&keywords, &api_key)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypted_file_round_trip_preserves_content() {
        let key = [42_u8; 32];
        let source = b"Jury project report: confidential";

        let encrypted = encrypt_file_bytes(&key, source).expect("file should encrypt");
        assert!(encrypted.starts_with(ENCRYPTED_FILE_PREFIX));
        assert_ne!(&encrypted[ENCRYPTED_FILE_PREFIX.len() + 12..], source);

        let decrypted = decrypt_file_bytes(&key, encrypted).expect("file should decrypt");
        assert_eq!(decrypted, source);
    }

    #[test]
    fn competition_scope_is_extracted_only_from_competition_routes() {
        assert_eq!(
            competition_id_from_path("/competitions/42/stages"),
            Some(42)
        );
        assert_eq!(competition_id_from_path("/projects/42"), None);
        assert_eq!(competition_id_from_path("/competitions/not-a-number"), None);
    }

    #[test]
    fn role_permissions_restrict_administrative_and_jury_routes() {
        let jury_member = AuthenticatedUser {
            id: 7,
            email: "jury@example.org".into(),
            role: "jury_member".into(),
            competition_id: Some(1),
            category: Some("software".into()),
        };
        let chief_judge = AuthenticatedUser {
            role: "chief_judge".into(),
            ..jury_member.clone()
        };

        assert!(!role_allows_request(&jury_member, &Method::GET, "/users"));
        assert!(!role_allows_request(
            &jury_member,
            &Method::PATCH,
            "/ranking"
        ));
        assert!(role_allows_request(
            &jury_member,
            &Method::POST,
            "/projects/3/jury-scores"
        ));
        assert!(!role_allows_request(
            &jury_member,
            &Method::GET,
            "/projects/3/ai-evaluation"
        ));
        assert!(role_allows_request(
            &chief_judge,
            &Method::PATCH,
            "/ranking"
        ));
        assert!(!role_allows_request(&chief_judge, &Method::GET, "/users"));
        assert!(!role_allows_request(
            &chief_judge,
            &Method::GET,
            "/notifications"
        ));
        let observer = AuthenticatedUser {
            id: 3,
            email: "observer@example.test".into(),
            role: "observer".into(),
            competition_id: Some(1),
            category: None,
        };
        assert!(role_allows_request(&observer, &Method::GET, "/projects/1"));
        assert!(!role_allows_request(&observer, &Method::GET, "/users"));
        assert!(!role_allows_request(&observer, &Method::GET, "/audit"));
        assert!(!role_allows_request(
            &observer,
            &Method::GET,
            "/email-campaigns"
        ));
    }

    #[test]
    fn event_stream_and_observer_safe_reads_follow_role_policy() {
        let observer = AuthenticatedUser {
            id: 3,
            email: "observer@example.test".into(),
            role: "observer".into(),
            competition_id: Some(1),
            category: None,
        };
        let jury_member = AuthenticatedUser {
            id: 4,
            email: "jury@example.test".into(),
            role: "jury_member".into(),
            competition_id: Some(1),
            category: Some("software".into()),
        };

        assert!(role_allows_request(&observer, &Method::GET, "/events"));
        assert!(role_allows_request(&jury_member, &Method::GET, "/events"));
        assert!(observer_can_read_path("/competitions/1/report"));
        assert!(!observer_can_read_path("/projects/1/jury-scores"));
        assert!(!observer_can_read_path("/projects/1/files"));
    }

    #[test]
    fn competition_visibility_requires_an_assigned_competition() {
        let scoped_user = AuthenticatedUser {
            id: 1,
            email: "jury@example.test".into(),
            role: "jury_member".into(),
            competition_id: Some(10),
            category: None,
        };
        let unscoped_user = AuthenticatedUser {
            competition_id: None,
            ..scoped_user.clone()
        };
        assert!(competition_is_visible_to(&scoped_user, 10));
        assert!(!competition_is_visible_to(&scoped_user, 11));
        assert!(!competition_is_visible_to(&unscoped_user, 10));
    }

    #[test]
    fn production_mode_disables_sample_data_by_default() {
        assert!(!sample_data_enabled(true, None));
        assert!(sample_data_enabled(false, None));
        assert!(sample_data_enabled(true, Some("true")));
    }

    #[test]
    fn two_factor_secret_storage_supports_encrypted_and_legacy_values() {
        let stored = protect_totp_secret("JBSWY3DPEHPK3PXP").expect("secret should be stored");
        assert!(stored.starts_with("plain:"));
        assert_eq!(
            unprotect_totp_secret(&stored).expect("stored secret should be readable"),
            "JBSWY3DPEHPK3PXP"
        );
        assert_eq!(
            unprotect_totp_secret("JBSWY3DPEHPK3PXP").expect("legacy secret should be readable"),
            "JBSWY3DPEHPK3PXP"
        );
    }

    #[test]
    fn production_requires_a_virus_scan_unless_explicitly_configured() {
        assert!(virus_scan_required(true, None));
        assert!(virus_scan_required(true, Some("")));
        assert!(!virus_scan_required(false, None));
        assert!(virus_scan_required(false, Some("true")));
    }

    #[test]
    fn two_factor_policy_defaults_to_required_in_production() {
        assert!(two_factor_policy_enabled(true, None));
        assert!(!two_factor_policy_enabled(false, None));
        assert!(two_factor_policy_enabled(false, Some("true")));
        assert!(!two_factor_policy_enabled(true, Some("false")));
    }

    #[test]
    fn rate_limit_ignores_spoofed_forwarded_headers_without_a_trusted_proxy() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("203.0.113.17, 10.0.0.1"),
        );
        assert_eq!(
            rate_limit_client(&headers, Some("127.0.0.1"), false),
            "127.0.0.1"
        );
        assert_eq!(
            rate_limit_client(&headers, Some("127.0.0.1"), true),
            "203.0.113.17"
        );
        assert_ne!(rate_limit_key("127.0.0.1".into()), "127.0.0.1");
    }

    #[test]
    fn email_retry_delay_uses_bounded_exponential_backoff() {
        assert_eq!(email_retry_delay(1), Duration::from_secs(30));
        assert_eq!(email_retry_delay(2), Duration::from_secs(60));
        assert_eq!(email_retry_delay(8), Duration::from_secs(3600));
    }

    #[test]
    fn password_reset_tokens_are_sha256_hashed() {
        let hash = password_reset_token_hash("reset-token-value");
        assert_eq!(hash.len(), 64);
        assert_ne!(hash, "reset-token-value");
        assert_eq!(hash, password_reset_token_hash("reset-token-value"));
    }

    #[test]
    fn recovery_codes_are_normalized_and_generated_as_a_set() {
        let codes = generate_recovery_codes();
        assert_eq!(codes.len(), 10);
        assert!(
            codes
                .iter()
                .all(|code| code.len() == 11 && code.as_bytes()[5] == b'-')
        );
        assert_eq!(
            recovery_code_hash("abcde-f1234"),
            recovery_code_hash("ABCDE-F1234")
        );
    }

    #[test]
    fn file_encryption_key_requires_a_base64_encoded_32_byte_value() {
        assert!(
            parse_file_encryption_key(None)
                .expect("missing key is valid outside production")
                .is_none()
        );
        assert!(
            parse_file_encryption_key(Some(""))
                .expect("empty key is treated as missing")
                .is_none()
        );
        assert!(parse_file_encryption_key(Some("invalid")).is_err());
        let encoded = STANDARD.encode([7_u8; 32]);
        assert_eq!(
            parse_file_encryption_key(Some(&encoded)).expect("valid key should parse"),
            Some([7_u8; 32])
        );
    }
}
