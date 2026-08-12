// JanissaryAsistan Engine — Axum REST API Sunucusu
// Tüm ağır işler burada: PDF analiz, web kazıma, benzerlik hesabı

use kys_engine::analysis;
use kys_engine::database::{Database, WeeklyWordPoint, DailyProjectPoint};
use kys_engine::parser;
use kys_engine::research;

use axum::{
    extract::{Multipart, Path, State, DefaultBodyLimit},
    http::{StatusCode, header},
    response::{Json, IntoResponse},
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};
use anyhow::Result;

// ─── Paylaşılan Uygulama Durumu ──────────────────────────────────

struct AppState {
    db: Database,
}

// ─── Ortak Yanıt Tipleri ─────────────────────────────────────────

#[derive(Serialize)]
struct ApiResponse<T: Serialize> {
    status: String,
    data: Option<T>,
    message: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    fn ok(data: T) -> Json<Self> {
        Json(Self { status: "success".into(), data: Some(data), message: None })
    }
    fn err(msg: &str) -> Json<ApiResponse<()>> {
        Json(ApiResponse { status: "error".into(), data: None, message: Some(msg.into()) })
    }
}

#[derive(Serialize)]
struct ProjectItem {
    id: String,
    title: String,
    category: String,
    score: Option<f64>,
    grade: String,
    status: String,
    word_count: i32,
}

#[derive(Serialize)]
struct ChartDataResponse {
    weekly_words: Vec<WeeklyWordPoint>,
    daily_projects: Vec<DailyProjectPoint>,
}

#[derive(Serialize)]
struct DashboardStats {
    total_projects: String,
    avg_score: String,
    risk_projects: String,
}

#[derive(Serialize)]
struct ScoreDetail {
    total: u32,
    grade: String,
    category_fit: u32,
    pub completeness: u32,
    pub reference_quality: u32,
    pub technical_depth: u32,
    pub ai_probability: f64,
}

#[derive(Serialize)]
struct SimilarityMatch {
    title: String,
    url: Option<String>,
    source_type: String,
    similarity_score: f32,
}

#[derive(Serialize)]
struct SimilarityDetail {
    pub overall_score: f64,
    pub originality_label: String,
    pub matches: Vec<SimilarityMatch>,
}

#[derive(Serialize)]
struct ProjectDetail {
    id: String,
    title: String,
    category: String,
    author: String,
    submit_date: String,
    status: String,
    score: ScoreDetail,
    similarity: SimilarityDetail,
    pdf_url: Option<String>,
}

#[derive(Serialize)]
struct AnalyzeResult {
    id: String,
    title: String,
    grade: String,
    total_score: u32,
    status: String,
}

#[derive(Deserialize)]
struct UpdateCategoryRequest {
    category: String,
}

#[derive(Deserialize)]
struct ApiKeyPayload {
    api_key: String,
}

#[derive(Deserialize)]
struct ReorderRequest {
    ranks: Vec<ProjectRank>,
}

#[derive(Deserialize)]
struct ProjectRank {
    id: String,
    rank: i32,
}

// ─── Main ─────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    info!("JanissaryAsistan Axum API başlatılıyor...");

    // Supabase / PostgreSQL bağlantısı
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/janissary".to_string());

    let db = Database::new(&database_url).await.unwrap_or_else(|e| {
        error!("Veritabanı bağlantısı başarısız: {}. Bazı özellikler çalışmayabilir.", e);
        panic!("Veritabanı bağlantısı başarısız: {}", e);
    });

    let state = Arc::new(AppState { db });

    // CORS — Geliştirme için her yerden erişime açık
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Router
    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/projects", get(get_projects))
        .route("/api/projects/:id", get(get_project_detail))
        .route("/api/stats", get(get_dashboard_stats))
        .route("/api/chart-data", get(get_chart_data))
        .route("/api/analyze", post(analyze_project))
        .route("/api/projects/reorder", put(reorder_projects))
        .route("/api/projects/:id/category", put(update_project_category))
        .route("/api/projects/:id/pdf", get(serve_pdf))
        .route("/api/settings/openai-key", get(get_openai_key).put(set_openai_key))
        .route("/api/settings/serper-key", get(get_serper_key).put(set_serper_key))
        .route("/api/settings/system-prompt", get(get_system_prompt).put(set_system_prompt))
        .with_state(state)
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("API hazır → http://{}", addr);
    info!("Endpoints:");
    info!("  GET  /api/health");
    info!("  GET  /api/projects");
    info!("  GET  /api/projects/:id");
    info!("  GET  /api/stats");
    info!("  POST /api/analyze  (multipart form-data: file)");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// ─── Endpoint Handler'ları ─────────────────────────────────────────

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "app": "JanissaryAsistan API",
        "version": "0.1.0"
    }))
}

async fn get_projects(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.db.list_projects().await {
        Ok(rows) => {
            let projects: Vec<ProjectItem> = rows.into_iter().map(|p| {
                let status = if p.total_score == 0.0 { "İnceleniyor".to_string() }
                    else if p.total_score >= 50.0 { "Tamamlandı".to_string() }
                    else { "Kopya İhtimali".to_string() };

                ProjectItem {
                    id: format!("PRJ-{}", p.id),
                    title: p.filename.replace(".pdf", "").replace(".PDF", ""),
                    category: p.category,
                    score: if p.total_score > 0.0 { Some(p.total_score) } else { None },
                    grade: if p.grade == "?" { "-".to_string() } else { p.grade },
                    status,
                    word_count: p.word_count,
                }
            }).collect();

            Ok(Json(serde_json::json!({ "status": "success", "data": projects })))
        }
        Err(e) => {
            error!("Projeler çekilemedi: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_chart_data(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.db.get_chart_data().await {
        Ok((weekly_words, daily_projects)) => {
            let chart = ChartDataResponse { weekly_words, daily_projects };
            Ok(Json(serde_json::json!({ "status": "success", "data": chart })))
        }
        Err(e) => {
            error!("Grafik verisi çekilemedi: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_dashboard_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.db.list_projects().await {
        Ok(rows) => {
            let total = rows.len();
            let scored: Vec<f64> = rows.iter().filter(|r| r.total_score > 0.0).map(|r| r.total_score).collect();
            let avg = if scored.is_empty() { 0.0 } else { scored.iter().sum::<f64>() / scored.len() as f64 };
            let risk = rows.iter().filter(|r| r.total_score > 0.0 && r.total_score < 50.0).count();

            let stats = DashboardStats {
                total_projects: if total > 0 { total.to_string() } else { "—".to_string() },
                avg_score: if avg > 0.0 { format!("{:.0}", avg) } else { "—".to_string() },
                risk_projects: risk.to_string(),
            };

            Ok(Json(serde_json::json!({ "status": "success", "data": stats })))
        }
        Err(e) => {
            error!("İstatistik hesaplanamadı: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_project_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // PRJ-123 → 123
    let numeric_id = id.replace("PRJ-", "").parse::<i64>().unwrap_or(0);

    match state.db.get_project_details_full(numeric_id).await {
        Ok(Some((proj, score, matches))) => {
            let s = score.unwrap_or_else(|| kys_engine::database::ScoreRecord {
                category_fit: 0.0, completeness: 0.0, reference_quality: 0.0,
                technical_depth: 0.0, originality: 0.0, total_score: 0.0,
                grade: "-".to_string(),
                ai_probability: 0.0,
            });

            let originality_label = if s.originality > 85.0 { "Çok Özgün" }
                else if s.originality > 65.0 { "Özgünlük Kabul Edilebilir" }
                else if s.originality > 45.0 { "Uyarı: Yüksek Benzerlik" }
                else { "Kopya / Çok Yüksek Benzerlik" };

            let detail = ProjectDetail {
                id: format!("PRJ-{}", proj.id),
                title: proj.filename.replace(".pdf", "").replace(".PDF", ""),
                category: "Genel".to_string(),
                author: proj.author,
                submit_date: proj.created_at,
                status: if s.total_score >= 50.0 { "Tamamlandı".to_string() } else { "Kopya İhtimali".to_string() },
                score: ScoreDetail {
                    total: s.total_score as u32,
                    grade: s.grade.clone(),
                    category_fit: s.category_fit as u32,
                    completeness: s.completeness as u32,
                    reference_quality: s.reference_quality as u32,
                    technical_depth: s.technical_depth as u32,
                    ai_probability: s.ai_probability as f64,
                },
                similarity: SimilarityDetail {
                    overall_score: (100.0 - s.originality as f64) / 100.0,
                    originality_label: originality_label.to_string(),
                    matches: matches.into_iter().map(|m| SimilarityMatch {
                        title: m.title.clone(),
                        url: m.url.clone(),
                        source_type: m.source_type.clone(),
                        similarity_score: m.similarity_score,
                    }).collect(),
                },
                pdf_url: Some(format!("http://localhost:8080/api/projects/{}/pdf", numeric_id)),
            };

            Ok(Json(serde_json::json!({ "status": "success", "data": detail })))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Proje detayı çekilemedi: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_project_category(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateCategoryRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let numeric_id = id.replace("PRJ-", "").parse::<i64>().unwrap_or(0);
    if numeric_id == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    match state.db.update_project_category(numeric_id, &payload.category).await {
        Ok(_) => Ok(Json(serde_json::json!({ "status": "success", "message": "Kategori güncellendi" }))),
        Err(e) => {
            error!("Kategori güncellenemedi: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn reorder_projects(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ReorderRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut ranks = Vec::new();
    for p in payload.ranks {
        let numeric_id = p.id.replace("PRJ-", "").parse::<i64>().unwrap_or(0);
        if numeric_id > 0 {
            ranks.push((numeric_id, p.rank));
        }
    }

    match state.db.update_project_ranks(&ranks).await {
        Ok(_) => Ok(Json(serde_json::json!({ "status": "success", "message": "Sıralama güncellendi" }))),
        Err(e) => {
            error!("Sıralama güncellenemedi: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn serve_pdf(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let numeric_id = id.replace("PRJ-", "").parse::<i64>().unwrap_or(0);
    if numeric_id == 0 {
        return (StatusCode::BAD_REQUEST, [(header::CONTENT_TYPE, "text/plain")], b"Gecersiz ID".to_vec());
    }

    match state.db.get_file_path(numeric_id).await {
        Ok(Some(path)) => {
            match std::fs::read(&path) {
                Ok(bytes) => (StatusCode::OK, [(header::CONTENT_TYPE, "application/pdf")], bytes),
                Err(_) => (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "text/plain")], b"Dosya diskte bulunamadi".to_vec()),
            }
        }
        _ => (StatusCode::NOT_FOUND, [(header::CONTENT_TYPE, "text/plain")], b"Proje veritabaninda bulunamadi".to_vec()),
    }
}

async fn analyze_project(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Dosyayı al
    let mut file_bytes: Vec<u8> = Vec::new();
    let mut filename = String::from("proje.pdf");
    let mut author = String::from("Bilinmeyen Kullanıcı");

    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        if let Some(name) = field.name() {
            let name_str = name.to_string(); // lifetime bypass
            if name_str == "file" {
                if let Some(fname) = field.file_name() {
                    filename = fname.to_string();
                }
                file_bytes = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?.to_vec();
            } else if name_str == "author" {
                author = field.text().await.unwrap_or_else(|_| "Bilinmeyen Kullanıcı".to_string());
            }
        }
    }

    if file_bytes.is_empty() {
        return Ok(Json(serde_json::json!({ "status": "error", "message": "Dosya bulunamadı" })));
    }
    
    // Aynı isimde proje var mı kontrol et
    let exists = state.db.check_project_exists(&filename).await.unwrap_or(false);
    if exists {
        return Ok(Json(serde_json::json!({ "status": "error", "message": "Bu proje daha önce yüklenmiş!" })));
    }

    // `uploads` klasörünün olduğundan emin ol
    let _ = std::fs::create_dir_all("uploads");

    // Dosyayı uploads klasörüne kalıcı olarak yaz
    let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
    let file_path = format!("uploads/{}_{}", timestamp, filename);
    std::fs::write(&file_path, &file_bytes).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Parser çalıştır
    let file_path_clone = file_path.clone();
    let mut document = match tokio::task::spawn_blocking(move || parser::parse_file(&file_path_clone)).await.unwrap() {
        Ok(d) => d,
        Err(e) => {
            let _ = std::fs::remove_file(&file_path);
            error!("Parse hatası: {}", e);
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }
    };

    // Web araştırması + benzerlik analizi (async)
    let search_results = research::search_related_sources(&document.keywords, "").await
        .unwrap_or_default();
    let similarity = analysis::compute_similarity(&document, &search_results);

    // Skor hesapla
    let categories = state.db.list_evaluation_categories().await.unwrap_or_default();
    let (category_fit, technical_depth, ai_probability, cat, semantic_reason) = analysis::evaluate_with_ai(&document, &categories).await;

    document.classified_category = cat.clone();
    let mut score = analysis::score_document(&document, category_fit, technical_depth, cat, semantic_reason);
    score.ai_probability = ai_probability;

    // Veritabanına kaydet
    let project_id = state.db.save_project(&document, &author, &file_path).await
        .map_err(|e| { error!("Proje kaydedilemedi: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    state.db.save_score(project_id, &score).await
        .map_err(|e| { error!("Skor kaydedilemedi: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    state.db.save_similarity(project_id, &similarity).await
        .map_err(|e| { error!("Benzerlik kaydedilemedi: {}", e); StatusCode::INTERNAL_SERVER_ERROR })?;

    let result = AnalyzeResult {
        id: format!("PRJ-{}", project_id),
        title: filename.replace(".pdf", "").replace(".PDF", ""),
        grade: score.grade().to_string(),
        total_score: score.total() as u32,
        status: "Tamamlandı".to_string(),
    };

    info!("Analiz tamamlandı: {} → {}/100 ({})", filename, score.total(), score.grade());

    Ok(Json(serde_json::json!({ "status": "success", "data": result })))
}

async fn get_openai_key() -> Json<serde_json::Value> {
    let key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
    let masked = if key.len() > 8 {
        format!("{}...{}", &key[0..4], &key[key.len()-4..])
    } else {
        String::new()
    };
    Json(serde_json::json!({ "status": "success", "data": { "masked_key": masked, "has_key": !key.is_empty() } }))
}

async fn set_openai_key(Json(payload): Json<ApiKeyPayload>) -> Json<serde_json::Value> {
    let key = payload.api_key;
    std::env::set_var("OPENAI_API_KEY", &key);
    update_env_file("OPENAI_API_KEY", &key);
    Json(serde_json::json!({ "status": "success", "message": "API key kaydedildi" }))
}

#[derive(Deserialize)]
struct PromptPayload {
    prompt: String,
}

async fn get_serper_key() -> Json<serde_json::Value> {
    let key = std::env::var("SERPER_API_KEY").unwrap_or_default();
    let masked = if key.len() > 8 {
        format!("{}...{}", &key[0..4], &key[key.len()-4..])
    } else {
        String::new()
    };
    Json(serde_json::json!({ "status": "success", "data": { "masked_key": masked, "has_key": !key.is_empty() } }))
}

async fn set_serper_key(Json(payload): Json<ApiKeyPayload>) -> Json<serde_json::Value> {
    std::env::set_var("SERPER_API_KEY", &payload.api_key);
    update_env_file("SERPER_API_KEY", &payload.api_key);
    Json(serde_json::json!({ "status": "success" }))
}

async fn get_system_prompt() -> Json<serde_json::Value> {
    let prompt = std::env::var("SYSTEM_PROMPT").unwrap_or_default();
    Json(serde_json::json!({ "status": "success", "data": { "prompt": prompt } }))
}

async fn set_system_prompt(Json(payload): Json<PromptPayload>) -> Json<serde_json::Value> {
    std::env::set_var("SYSTEM_PROMPT", &payload.prompt);
    update_env_file("SYSTEM_PROMPT", &payload.prompt);
    Json(serde_json::json!({ "status": "success" }))
}

fn update_env_file(key: &str, value: &str) {
    let mut env_content = std::fs::read_to_string(".env").unwrap_or_default();
    let prefix = format!("{}=", key);
    if env_content.contains(&prefix) {
        let lines: Vec<&str> = env_content.lines().collect();
        let mut new_lines = Vec::new();
        for line in lines {
            if line.starts_with(&prefix) {
                new_lines.push(format!("{}={}", key, value));
            } else {
                new_lines.push(line.to_string());
            }
        }
        std::fs::write(".env", new_lines.join("\n")).unwrap_or_default();
    } else {
        env_content.push_str(&format!("\n{}={}", key, value));
        std::fs::write(".env", env_content).unwrap_or_default();
    }
}
