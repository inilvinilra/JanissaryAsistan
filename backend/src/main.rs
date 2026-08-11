mod database;
mod models;
mod parser;
mod research;
mod scoring;

use axum::{
    extract::{Multipart, Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, patch},
    Json, Router,
};
use models::{
    AddTeamMember, Competition, CompetitionCategory, CompetitionStage, CreateSubmission,
    CreateJuryScore, CreateTeam, JuryScore, Project, ProjectUpdate, RankingUpdate, Submission,
    Team, TeamMember, AiEvaluation, UpsertAiEvaluation, CreateJuryAssignment, JuryAssignment,
    AuditEvent, DemoDaySlot, CreateDemoDaySlot, FinalistSelection, UpdateTeamStatus,
    SubmissionVersion, User, CreateUser, UpdateUser, ProjectMetadata, UpdateProjectMetadata, ProjectFile, RoleDefinition, Notification, CreateNotification, UpdateStageStatus,
    CompetitionReport,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
struct AppState {
    db: Arc<database::Database>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not found in .env file");
    let db = database::Database::new(&database_url)
        .await
        .expect("Could not connect to database");
    println!("Database connected, tables ready.");

    db.seed_kpi_templates().await.expect("Failed to seed KPI templates");
    db.seed_sample_data().await.expect("Failed to seed sample data");

    let state = AppState { db: Arc::new(db) };

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/projects", get(list_projects).post(create_project))
        .route("/projects/upload", axum::routing::post(upload_project))
        .route("/projects/{id}", get(get_project).patch(update_project))
        .route("/projects/{id}/metadata", get(get_project_metadata).patch(update_project_metadata))
        .route("/projects/{id}/files", get(list_project_files).post(upload_project_file))
        .route("/projects/{id}/files/{file_id}", get(download_project_file))
        .route("/projects/{id}/document", get(get_project_document))
        .route("/projects/{id}/file", get(get_project_file))
        .route("/categories", get(list_categories))
        .route("/categories/{category}/kpis", axum::routing::put(update_kpi_template))
        .route("/competitions", get(list_competitions).post(create_competition))
        .route("/competitions/{id}/stages", get(list_competition_stages).post(create_competition_stage))
        .route("/competitions/{id}/stages/{stage_id}/status", patch(update_stage_status))
        .route("/competitions/{id}/categories", get(list_competition_categories).post(create_competition_category))
        .route("/competitions/{id}/teams", get(list_teams).post(create_team))
        .route("/competitions/{id}/finalists", axum::routing::post(select_finalists))
        .route("/competitions/{id}/demo-day", get(list_demo_day_slots).post(create_demo_day_slot))
        .route("/competitions/{id}/report", get(get_competition_report))
        .route("/teams/{id}", patch(update_team_status))
        .route("/teams/{id}/members", axum::routing::post(add_team_member))
        .route("/teams/{id}/submissions", get(list_submissions).post(create_submission))
        .route("/submissions/{id}/versions", get(list_submission_versions).post(upload_submission_version))
        .route("/projects/{id}/ai-evaluation", get(get_ai_evaluation).put(upsert_ai_evaluation))
        .route("/projects/{id}/jury-scores", get(list_jury_scores).post(add_jury_score))
        .route("/projects/{id}/jury-assignments", get(list_jury_assignments).post(add_jury_assignment))
        .route("/audit", get(list_audit))
        .route("/users", get(list_users).post(create_user))
        .route("/users/{id}", patch(update_user))
        .route("/roles", get(list_roles))
        .route("/notifications", get(list_notifications).post(create_notification))
        .route("/ranking", patch(update_ranking))
        .route("/activity", get(list_activity))
        .route("/test/parse", get(test_parse))
        .route("/test/search", get(test_search))
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Server running: http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello! Backend is running."
}

async fn health(State(state): State<AppState>) -> (StatusCode, &'static str) {
    match sqlx::query("SELECT 1").execute(&state.db.pool).await {
        Ok(_) => (StatusCode::OK, "OK - API and database are running"),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, "API is up but database is unreachable"),
    }
}

async fn get_project_metadata(State(state): State<AppState>, Path(id): Path<i32>) -> Result<Json<ProjectMetadata>, StatusCode> {
    state.db.get_project_metadata(id).await.map(Json).map_err(|_| StatusCode::NOT_FOUND)
}

async fn update_project_metadata(State(state): State<AppState>, Path(id): Path<i32>, Json(input): Json<UpdateProjectMetadata>) -> Result<Json<ProjectMetadata>, (StatusCode, String)> {
    let metadata = state.db.update_project_metadata(id, &input).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    state.db.record_audit("project_metadata_updated", "jury", "project", Some(id), serde_json::json!({"institution": metadata.institution, "keyword_count": metadata.keywords.len(), "has_github": metadata.github_url.is_some(), "has_demo": metadata.demo_url.is_some()})).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(metadata))
}

async fn list_project_files(State(state): State<AppState>, Path(id): Path<i32>) -> Result<Json<Vec<ProjectFile>>, StatusCode> {
    state.db.list_project_files(id).await.map(Json).map_err(|_| StatusCode::NOT_FOUND)
}

async fn upload_project_file(State(state): State<AppState>, Path(id): Path<i32>, mut multipart: Multipart) -> Result<(StatusCode, Json<ProjectFile>), (StatusCode, String)> {
    let mut file_name = None;
    let mut mime_type = "application/octet-stream".to_string();
    let mut bytes = None;
    while let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))? {
        if field.name().unwrap_or("") == "file" {
            file_name = field.file_name().map(str::to_string);
            if let Some(content_type) = field.content_type() { mime_type = content_type.to_string(); }
            bytes = Some(field.bytes().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?.to_vec());
        }
    }
    let file_name = file_name.ok_or((StatusCode::BAD_REQUEST, "Missing file".into()))?;
    let bytes = bytes.ok_or((StatusCode::BAD_REQUEST, "Missing file".into()))?;
    if bytes.is_empty() || bytes.len() > 25 * 1024 * 1024 { return Err((StatusCode::PAYLOAD_TOO_LARGE, "File must be between 1 byte and 25 MB".into())); }
    let ext = std::path::Path::new(&file_name).extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let allowed = ["pdf", "txt", "md", "markdown", "doc", "docx", "xls", "xlsx", "csv", "png", "jpg", "jpeg", "webp"];
    if !allowed.contains(&ext.as_str()) { return Err((StatusCode::UNSUPPORTED_MEDIA_TYPE, "Unsupported file type".into())); }
    if !valid_file_signature(&ext, &bytes) { return Err((StatusCode::UNSUPPORTED_MEDIA_TYPE, "File content does not match its extension".into())); }
    let dir = format!("uploads/project-{id}");
    std::fs::create_dir_all(&dir).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = format!("{dir}/{unique}-{file_name}");
    std::fs::write(&path, &bytes).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let file = state.db.add_project_file(id, &file_name, &mime_type, bytes.len() as i64, &path).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    state.db.record_audit("project_file_uploaded", "jury", "project", Some(id), serde_json::json!({"file_name": file.file_name, "version": file.version, "size_bytes": file.size_bytes})).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(file)))
}

async fn download_project_file(State(state): State<AppState>, Path((id, file_id)): Path<(i32, i32)>) -> Result<impl IntoResponse, StatusCode> {
    let file = state.db.get_project_file_record(id, file_id).await.map_err(|_| StatusCode::NOT_FOUND)?.ok_or(StatusCode::NOT_FOUND)?;
    let bytes = tokio::fs::read(&file.file_path).await.map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(([(header::CONTENT_TYPE, file.mime_type)], bytes))
}

fn valid_file_signature(extension: &str, bytes: &[u8]) -> bool {
    match extension {
        "pdf" => bytes.starts_with(b"%PDF"),
        "png" => bytes.starts_with(&[0x89, b'P', b'N', b'G']),
        "jpg" | "jpeg" => bytes.starts_with(&[0xff, 0xd8, 0xff]),
        "webp" => bytes.len() > 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP",
        "docx" | "xlsx" | "xls" | "doc" => bytes.starts_with(b"PK") || bytes.starts_with(&[0xd0, 0xcf, 0x11, 0xe0]),
        "txt" | "md" | "markdown" | "csv" => std::str::from_utf8(bytes).is_ok(),
        _ => false,
    }
}

const USER_ROLES: [&str; 6] = ["system_admin", "competition_manager", "chief_judge", "jury_member", "observer", "read_only"];

fn valid_role(role: &str) -> bool { USER_ROLES.contains(&role) }

async fn list_roles() -> Json<Vec<RoleDefinition>> {
    Json(vec![
        RoleDefinition { role: "system_admin".into(), permissions: vec!["users:manage".into(), "competitions:manage".into(), "projects:manage".into(), "reports:view".into(), "audit:view".into()] },
        RoleDefinition { role: "competition_manager".into(), permissions: vec!["competitions:manage".into(), "projects:manage".into(), "jury:manage".into(), "reports:view".into()] },
        RoleDefinition { role: "chief_judge".into(), permissions: vec!["projects:review".into(), "ranking:manage".into(), "jury:assign".into(), "reports:view".into()] },
        RoleDefinition { role: "jury_member".into(), permissions: vec!["projects:review".into(), "jury:scores:create".into(), "assigned_scope:read".into()] },
        RoleDefinition { role: "observer".into(), permissions: vec!["projects:read".into(), "reports:view".into(), "audit:view".into()] },
        RoleDefinition { role: "read_only".into(), permissions: vec!["projects:read".into(), "reports:view".into()] },
    ])
}

async fn list_users(State(state): State<AppState>) -> Result<Json<Vec<User>>, StatusCode> {
    state.db.list_users().await.map(Json).map_err(|e| { eprintln!("List users error: {e}"); StatusCode::INTERNAL_SERVER_ERROR })
}

const NOTIFICATION_KINDS: [&str; 7] = ["announcement", "missing_document", "deadline", "review_task", "result", "question", "faq"];

async fn list_notifications(State(state): State<AppState>, Query(params): Query<HashMap<String, String>>) -> Result<Json<Vec<Notification>>, StatusCode> {
    let limit = params.get("limit").and_then(|value| value.parse().ok()).unwrap_or(50);
    state.db.list_notifications(limit).await.map(Json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn create_notification(State(state): State<AppState>, Json(input): Json<CreateNotification>) -> Result<(StatusCode, Json<Notification>), (StatusCode, String)> {
    if input.title.trim().is_empty() || input.body.trim().is_empty() || !NOTIFICATION_KINDS.contains(&input.kind.as_str()) || input.audience.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Title, body, audience and a valid notification kind are required".into()));
    }
    let notification = state.db.create_notification(&input).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    state.db.record_audit("notification_created", "system", "notification", Some(notification.id), serde_json::json!({"kind": notification.kind, "audience": notification.audience, "category": notification.category})).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(notification)))
}

async fn create_user(State(state): State<AppState>, Json(mut input): Json<CreateUser>) -> Result<(StatusCode, Json<User>), StatusCode> {
    input.full_name = input.full_name.trim().to_string();
    input.email = input.email.trim().to_lowercase();
    if input.full_name.is_empty() || !input.email.contains('@') || !valid_role(&input.role) { return Err(StatusCode::BAD_REQUEST); }
    let user = state.db.create_user(&input).await.map_err(|e| { eprintln!("Create user error: {e}"); StatusCode::BAD_REQUEST })?;
    state.db.record_audit("user_created", &input.email, "user", Some(user.id), serde_json::json!({
        "role": user.role, "active": user.active, "competition_id": user.competition_id, "category": user.category,
    })).await.map_err(|e| { eprintln!("Audit error: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;
    Ok((StatusCode::CREATED, Json(user)))
}

async fn update_user(State(state): State<AppState>, Path(id): Path<i32>, Json(input): Json<UpdateUser>) -> Result<Json<User>, StatusCode> {
    if let Some(role) = input.role.as_deref() { if !valid_role(role) { return Err(StatusCode::BAD_REQUEST); } }
    let before = state.db.list_users().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.into_iter().find(|user| user.id == id).ok_or(StatusCode::NOT_FOUND)?;
    let after = state.db.update_user(id, &input).await.map_err(|e| { eprintln!("Update user error: {e}"); StatusCode::NOT_FOUND })?;
    state.db.record_audit("user_updated", &before.email, "user", Some(id), serde_json::json!({
        "before": { "role": before.role, "active": before.active, "competition_id": before.competition_id, "category": before.category },
        "after": { "role": after.role, "active": after.active, "competition_id": after.competition_id, "category": after.category },
    })).await.map_err(|e| { eprintln!("Audit error: {e}"); StatusCode::INTERNAL_SERVER_ERROR })?;
    Ok(Json(after))
}

// GET /projects              -> all projects, ranked
// GET /projects?category=software -> only that category
//
// A project a juror has dragged (manual_rank set) is ranked by that position first;
// untouched projects fall back to ai_score, highest first.
async fn list_projects(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<Project>>, StatusCode> {
    let category = params.get("category").map(|s| s.as_str());

    let mut projects = state.db.list_projects(category).await.map_err(|e| {
        eprintln!("List error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    projects.sort_by(|a, b| match (a.manual_rank, b.manual_rank) {
        (Some(x), Some(y)) => x.cmp(&y),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => b.ai_score.partial_cmp(&a.ai_score).unwrap(),
    });

    Ok(Json(projects))
}

#[derive(Deserialize)]
struct CreateProjectRequest {
    name: String,
    category: String,
    file_path: String,
}

// POST /projects
// Body: { "name": "...", "category": "software", "file_path": "samples/sample-project.md" }
// Parses a document already sitting on the server's disk. See POST /projects/upload
// for the version that accepts an actual file from the browser.
async fn create_project(
    State(state): State<AppState>,
    Json(req): Json<CreateProjectRequest>,
) -> Result<Json<Project>, (StatusCode, String)> {
    let document = parser::parse_file(&req.file_path)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Parse error: {e}")))?;

    score_and_store(&state, &req.name, &req.category, document, Some(&req.file_path)).await
}

// POST /projects/upload
// multipart/form-data fields: name, category, file (the actual PDF/TXT/Markdown).
// Saves the upload to uploads/, parses it, scores it, and stores the result — same
// pipeline as POST /projects, just fed by a real file instead of a server-side path.
async fn upload_project(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Project>, (StatusCode, String)> {
    let mut name: Option<String> = None;
    let mut category: Option<String> = None;
    let mut filename: Option<String> = None;
    let mut file_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))? {
        match field.name().unwrap_or("") {
            "name" => name = Some(field.text().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?),
            "category" => {
                category = Some(field.text().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?)
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
    let category = category.ok_or((StatusCode::BAD_REQUEST, "Missing 'category' field".to_string()))?;
    let filename = filename.ok_or((StatusCode::BAD_REQUEST, "Missing file".to_string()))?;
    let file_bytes = file_bytes.ok_or((StatusCode::BAD_REQUEST, "Missing file".to_string()))?;

    std::fs::create_dir_all("uploads").map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let ext = std::path::Path::new(&filename).extension().and_then(|e| e.to_str()).unwrap_or("txt");
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let stored_path = format!("uploads/{unique}.{ext}");
    std::fs::write(&stored_path, &file_bytes).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let document = parser::parse_file(&stored_path)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Parse error: {e}")))?;

    score_and_store(&state, &name, &category, document, Some(&stored_path)).await
}

async fn score_and_store(
    state: &AppState,
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
        .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("Unknown category: {}", category)))?;

    let kpi_scores = scoring::score_project(&scoring::Scorer::Mock, &document, &template.kpis)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let id = state
        .db
        .insert_project(name, category, kpi_scores, Some(&document), file_path)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state.db.record_audit("project_uploaded", "system", "project", Some(id), serde_json::json!({
        "name": name, "category": category, "file_path": file_path, "document_filename": document.filename,
        "kpi_template": template.kpis.iter().map(|kpi| &kpi.name).collect::<Vec<_>>(),
    })).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state
        .db
        .get_project(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map(Json)
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Project not found after insert".to_string()))
}

// GET /categories -> every jury field and the KPI set jurors score projects against in it
async fn list_categories(State(state): State<AppState>) -> Result<Json<Vec<models::CategoryTemplate>>, StatusCode> {
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
    State(state): State<AppState>,
    Path(category): Path<String>,
    Json(update): Json<KpiTemplateUpdate>,
) -> Result<Json<models::CategoryTemplate>, (StatusCode, String)> {
    if update.kpis.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "At least one KPI is required".into()));
    }
    if update.kpis.iter().any(|kpi| kpi.name.trim().is_empty() || !(0.0..=100.0).contains(&kpi.weight)) {
        return Err((StatusCode::BAD_REQUEST, "KPI names are required and weights must be between 0 and 100".into()));
    }
    let total: f64 = update.kpis.iter().map(|kpi| kpi.weight).sum();
    if (total - 100.0).abs() > 0.01 {
        return Err((StatusCode::BAD_REQUEST, format!("KPI weights must total 100 (received {total:.2})")));
    }
    state.db.replace_kpi_template(&category, &update.kpis).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    state.db.record_audit("kpi_template_updated", "system", "category", None, serde_json::json!({
        "category": category, "kpis": &update.kpis,
    })).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(models::CategoryTemplate { category, kpis: update.kpis }))
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

async fn list_competitions(State(state): State<AppState>) -> Result<Json<Vec<Competition>>, StatusCode> {
    state.db.list_competitions().await.map(Json).map_err(|e| {
        eprintln!("Competitions error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

async fn create_competition(
    State(state): State<AppState>,
    Json(req): Json<CreateCompetitionRequest>,
) -> Result<Json<Competition>, (StatusCode, String)> {
    if req.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Competition name is required".into()));
    }
    let id = state.db.create_competition(
        req.name.trim(), req.description.as_deref().unwrap_or(""),
        req.application_start.as_deref(), req.application_end.as_deref(), req.organization.as_deref().unwrap_or("T3 Vakfı"),
    ).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let competition = state.db.list_competitions().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .into_iter().find(|item| item.id == id)
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Competition not found after insert".into()))?;
    Ok(Json(competition))
}

async fn list_competition_stages(
    State(state): State<AppState>, Path(id): Path<i32>,
) -> Result<Json<Vec<CompetitionStage>>, StatusCode> {
    state.db.list_competition_stages(id).await.map(Json).map_err(|e| {
        eprintln!("Competition stages error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

async fn create_competition_stage(
    State(state): State<AppState>, Path(id): Path<i32>, Json(req): Json<CreateStageRequest>,
) -> Result<Json<CompetitionStage>, (StatusCode, String)> {
    if req.name.trim().is_empty() || req.stage_type.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Stage name and type are required".into()));
    }
    if req.position < 1 {
        return Err((StatusCode::BAD_REQUEST, "Stage position must be positive".into()));
    }
    if req.passing_score.unwrap_or(0.0) < 0.0 || req.passing_score.unwrap_or(0.0) > 100.0 || req.finalist_limit.unwrap_or(0) < 0 {
        return Err((StatusCode::BAD_REQUEST, "Invalid stage threshold or finalist limit".into()));
    }
    state.db.add_competition_stage(
        id, req.name.trim(), req.stage_type.trim(), req.position,
        req.starts_at.as_deref(), req.ends_at.as_deref(), req.passing_score.unwrap_or(0.0), req.finalist_limit.filter(|value| *value > 0), req.results_at.as_deref(),
    ).await.map(Json).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

async fn update_stage_status(State(state): State<AppState>, Path((competition_id, stage_id)): Path<(i32, i32)>, Json(input): Json<UpdateStageStatus>) -> Result<Json<CompetitionStage>, (StatusCode, String)> {
    if !matches!(input.status.as_str(), "planned" | "active" | "completed" | "locked") { return Err((StatusCode::BAD_REQUEST, "Invalid stage status".into())); }
    let stages = state.db.list_competition_stages(competition_id).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let current = stages.iter().find(|stage| stage.id == stage_id).ok_or((StatusCode::NOT_FOUND, "Stage not found".into()))?;
    let allowed = matches!((&current.status[..], input.status.as_str()), ("planned", "active") | ("active", "completed") | ("completed", "locked"));
    if !allowed { return Err((StatusCode::CONFLICT, format!("Invalid stage transition: {} -> {}", current.status, input.status))); }
    let stage = state.db.update_stage_status(stage_id, &input.status).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    state.db.record_audit("stage_status_updated", "system", "competition_stage", Some(stage_id), serde_json::json!({"competition_id": competition_id, "before": current.status, "after": input.status})).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(stage))
}

async fn list_competition_categories(
    State(state): State<AppState>, Path(id): Path<i32>,
) -> Result<Json<Vec<CompetitionCategory>>, StatusCode> {
    state.db.list_competition_categories(id).await.map(Json).map_err(|e| {
        eprintln!("Competition categories error: {e}");
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

async fn create_competition_category(
    State(state): State<AppState>, Path(id): Path<i32>, Json(req): Json<CreateCompetitionCategoryRequest>,
) -> Result<Json<CompetitionCategory>, (StatusCode, String)> {
    if req.name.trim().is_empty() || req.slug.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Category name and slug are required".into()));
    }
    state.db.add_competition_category(
        id, req.parent_id, req.name.trim(), req.slug.trim(), req.kpi_category.as_deref(),
    ).await.map(Json).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

async fn list_teams(
    State(state): State<AppState>, Path(id): Path<i32>,
) -> Result<Json<Vec<Team>>, (StatusCode, String)> {
    state.db.list_teams(id).await.map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn create_team(
    State(state): State<AppState>, Path(id): Path<i32>, Json(input): Json<CreateTeam>,
) -> Result<Json<Team>, (StatusCode, String)> {
    if input.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Team name is required".into()));
    }
    state.db.create_team(id, &input).await.map(Json).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

async fn update_team_status(
    State(state): State<AppState>, Path(id): Path<i32>, Json(input): Json<UpdateTeamStatus>,
) -> Result<StatusCode, (StatusCode, String)> {
    const ALLOWED: &[&str] = &["new", "reviewing", "finalist", "rejected", "winner"];
    if !ALLOWED.contains(&input.status.as_str()) {
        return Err((StatusCode::BAD_REQUEST, "Invalid team status".into()));
    }
    state.db.update_team_status(id, &input.status).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn select_finalists(
    State(state): State<AppState>, Path(id): Path<i32>, Json(input): Json<FinalistSelection>,
) -> Result<StatusCode, (StatusCode, String)> {
    if input.team_ids.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "At least one finalist team is required".into()));
    }
    state.db.select_finalists(id, &input.team_ids).await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_demo_day_slots(
    State(state): State<AppState>, Path(id): Path<i32>,
) -> Result<Json<Vec<DemoDaySlot>>, StatusCode> {
    state.db.list_demo_day_slots(id).await.map(Json).map_err(|e| {
        eprintln!("Demo Day slots error: {e}"); StatusCode::INTERNAL_SERVER_ERROR
    })
}

async fn create_demo_day_slot(
    State(state): State<AppState>, Path(id): Path<i32>, Json(input): Json<CreateDemoDaySlot>,
) -> Result<Json<DemoDaySlot>, (StatusCode, String)> {
    if input.slot_order < 1 || input.room.trim().is_empty() || input.starts_at.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Slot order, room and start time are required".into()));
    }
    state.db.add_demo_day_slot(id, &input).await.map(Json).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

async fn get_competition_report(
    State(state): State<AppState>, Path(id): Path<i32>,
) -> Result<Json<CompetitionReport>, StatusCode> {
    state.db.competition_report(id).await.map(Json).map_err(|e| {
        eprintln!("Competition report error: {e}"); StatusCode::INTERNAL_SERVER_ERROR
    })
}

async fn add_team_member(
    State(state): State<AppState>, Path(id): Path<i32>, Json(input): Json<AddTeamMember>,
) -> Result<Json<TeamMember>, (StatusCode, String)> {
    if input.full_name.trim().is_empty() || input.email.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Member name and email are required".into()));
    }
    state.db.add_team_member(id, &input).await.map(Json).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

async fn list_submissions(
    State(state): State<AppState>, Path(id): Path<i32>,
) -> Result<Json<Vec<Submission>>, (StatusCode, String)> {
    state.db.list_submissions(id).await.map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn create_submission(
    State(state): State<AppState>, Path(id): Path<i32>, Json(input): Json<CreateSubmission>,
) -> Result<Json<Submission>, (StatusCode, String)> {
    if input.title.trim().is_empty() || input.file_name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Submission title and file name are required".into()));
    }
    state.db.create_submission(id, &input).await.map(Json).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

async fn upload_submission_version(
    State(state): State<AppState>, Path(id): Path<i32>, mut multipart: Multipart,
) -> Result<Json<SubmissionVersion>, (StatusCode, String)> {
    let mut filename: Option<String> = None;
    let mut bytes: Option<Vec<u8>> = None;
    while let Some(field) = multipart.next_field().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))? {
        if field.name().unwrap_or("") == "file" {
            filename = field.file_name().map(str::to_string);
            bytes = Some(field.bytes().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?.to_vec());
        }
    }
    let filename = filename.ok_or((StatusCode::BAD_REQUEST, "Missing file".into()))?;
    let bytes = bytes.ok_or((StatusCode::BAD_REQUEST, "Missing file".into()))?;
    std::fs::create_dir_all("uploads/submissions").map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let safe_name = std::path::Path::new(&filename).file_name().and_then(|name| name.to_str()).unwrap_or("submission.bin");
    let stored_path = format!("uploads/submissions/{unique}-{safe_name}");
    std::fs::write(&stored_path, bytes).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    state.db.add_submission_version(id, &filename, &stored_path).await.map(Json).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

async fn list_submission_versions(
    State(state): State<AppState>, Path(id): Path<i32>,
) -> Result<Json<Vec<SubmissionVersion>>, StatusCode> {
    state.db.list_submission_versions(id).await.map(Json).map_err(|e| {
        eprintln!("Submission versions error: {e}"); StatusCode::INTERNAL_SERVER_ERROR
    })
}

async fn get_ai_evaluation(
    State(state): State<AppState>, Path(id): Path<i32>,
) -> Result<Json<AiEvaluation>, StatusCode> {
    state.db.get_ai_evaluation(id).await.map_err(|e| {
        eprintln!("AI evaluation error: {e}"); StatusCode::INTERNAL_SERVER_ERROR
    })?.map(Json).ok_or(StatusCode::NOT_FOUND)
}

async fn upsert_ai_evaluation(
    State(state): State<AppState>, Path(id): Path<i32>, Json(input): Json<UpsertAiEvaluation>,
) -> Result<Json<AiEvaluation>, (StatusCode, String)> {
    if input.model_version.trim().is_empty() || !(0.0..=100.0).contains(&input.total_score)
        || !(0.0..=1.0).contains(&input.confidence)
        || input.kpi_scores.iter().any(|kpi| kpi.name.trim().is_empty()
            || !(0.0..=100.0).contains(&kpi.score)
            || !(0.0..=1.0).contains(&kpi.confidence))
        || input.similar_projects.iter().any(|project| project.name.trim().is_empty()
            || !(0.0..=1.0).contains(&project.similarity))
    {
        return Err((StatusCode::BAD_REQUEST, "Model version, scores (0-100) and confidence values (0-1) are required".into()));
    }
    let evaluation = state.db.upsert_ai_evaluation(id, &input).await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    state.db.record_audit("ai_evaluation_updated", &input.model_version, "project", Some(id), serde_json::json!({
        "total_score": input.total_score,
        "confidence": input.confidence,
        "model_version": input.model_version,
    })).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(evaluation))
}

async fn list_jury_scores(
    State(state): State<AppState>, Path(id): Path<i32>,
) -> Result<Json<Vec<JuryScore>>, StatusCode> {
    state.db.list_jury_scores(id).await.map(Json).map_err(|e| {
        eprintln!("Jury scores error: {e}"); StatusCode::INTERNAL_SERVER_ERROR
    })
}

async fn add_jury_score(
    State(state): State<AppState>, Path(id): Path<i32>, Json(input): Json<CreateJuryScore>,
) -> Result<Json<JuryScore>, (StatusCode, String)> {
    if input.juror_name.trim().is_empty() || !(0.0..=100.0).contains(&input.total_score) {
        return Err((StatusCode::BAD_REQUEST, "Juror name and score (0-100) are required".into()));
    }
    let score = state.db.add_jury_score(id, &input).await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    state.db.record_audit("jury_score_submitted", &input.juror_name, "project", Some(id), serde_json::json!({
        "total_score": input.total_score,
    })).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(score))
}

async fn list_jury_assignments(
    State(state): State<AppState>, Path(id): Path<i32>,
) -> Result<Json<Vec<JuryAssignment>>, StatusCode> {
    state.db.list_jury_assignments(id).await.map(Json).map_err(|e| {
        eprintln!("Jury assignments error: {e}"); StatusCode::INTERNAL_SERVER_ERROR
    })
}

async fn add_jury_assignment(
    State(state): State<AppState>, Path(id): Path<i32>, Json(input): Json<CreateJuryAssignment>,
) -> Result<Json<JuryAssignment>, (StatusCode, String)> {
    if input.juror_name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Juror name is required".into()));
    }
    state.db.add_jury_assignment(id, &input).await.map(Json).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

async fn list_audit(
    State(state): State<AppState>, Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<AuditEvent>>, StatusCode> {
    let limit = params.get("limit").and_then(|value| value.parse().ok()).unwrap_or(50);
    state.db.list_audit(limit).await.map(Json).map_err(|e| {
        eprintln!("Audit error: {e}"); StatusCode::INTERNAL_SERVER_ERROR
    })
}

async fn get_project(State(state): State<AppState>, Path(id): Path<i32>) -> Result<Json<Project>, StatusCode> {
    let project = state
        .db
        .get_project(id)
        .await
        .map_err(|e| {
            eprintln!("Detail error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)?;
    if let Err(error) = state.db.record_audit("project_opened", "jury", "project", Some(id), serde_json::json!({})).await {
        eprintln!("Audit error: {error}");
    }
    Ok(project)
}

// PATCH /projects/{id}
// Body: { "notes": "...", "status": "finalist" } — both fields optional.
async fn update_project(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(update): Json<ProjectUpdate>,
) -> Result<Json<Project>, (StatusCode, String)> {
    if let Some(status) = update.status.as_deref() {
        if !matches!(status, "new" | "reviewing" | "finalist" | "rejected") {
            return Err((StatusCode::BAD_REQUEST, "Invalid project status".into()));
        }
    }
    let before = state.db.get_project(id).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Project not found".to_string()))?;
    state
        .db
        .update_project(id, update.notes.as_deref(), update.status.as_deref(), update.review_completed, update.tags.as_deref())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let after = state.db.get_project(id).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Project not found".to_string()))?;
    state.db.record_audit("project_updated", "jury", "project", Some(id), serde_json::json!({
        "before": { "status": before.status, "notes": before.notes, "review_completed": before.review_completed, "tags": before.tags },
        "after": { "status": after.status, "notes": after.notes, "review_completed": after.review_completed, "tags": after.tags },
    })).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(after))
}

// GET /projects/{id}/file -> the original uploaded file's bytes, for preview/download.
// 404 if the project has no stored file (seeded/legacy projects).
async fn get_project_file(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, StatusCode> {
    let path = state.db.get_project_file_path(id).await.map_err(|e| {
        eprintln!("File path fetch error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let Some(path) = path else { return Err(StatusCode::NOT_FOUND) };

    let bytes = tokio::fs::read(&path).await.map_err(|_| StatusCode::NOT_FOUND)?;

    let content_type = match std::path::Path::new(&path).extension().and_then(|e| e.to_str()) {
        Some("pdf") => "application/pdf",
        Some("md") | Some("markdown") => "text/markdown; charset=utf-8",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    };

    Ok(([(header::CONTENT_TYPE, content_type)], bytes))
}

// GET /projects/{id}/document -> the parser's full analysis for that project, if it
// was created from a real file (seeded/legacy projects have none: 404).
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

// GET /activity?category=software&limit=10 -> most recent manual ranking changes
async fn list_activity(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<models::ActivityEntry>>, StatusCode> {
    let category = params.get("category").map(|s| s.as_str());
    let limit = params.get("limit").and_then(|v| v.parse().ok()).unwrap_or(10);

    state.db.list_activity(category, limit).await.map(Json).map_err(|e| {
        eprintln!("Activity error: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })
}

// PATCH /ranking
// Body: { "category": "software", "order": [4, 3], "changed_by": "Ayşe" }
// changed_by is optional free text (a juror's name typed client-side, not an
// authenticated identity) recorded in the ranking_history audit trail.
async fn update_ranking(State(state): State<AppState>, Json(update): Json<RankingUpdate>) -> StatusCode {
    let changed_by = update.changed_by.as_deref().unwrap_or("jury");
    match state.db.update_ranking(&update.category, &update.order, changed_by).await {
        Ok(_) => {
            if let Err(e) = state.db.record_audit("ranking_updated", changed_by, "category", None, serde_json::json!({
                "category": update.category,
                "order": update.order,
            })).await {
                eprintln!("Audit error: {e}");
                return StatusCode::INTERNAL_SERVER_ERROR;
            }
            StatusCode::OK
        },
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
