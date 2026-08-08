// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod auth;

use serde::Serialize;
use tauri::Manager;
use std::sync::Arc;
use kys_engine::database::Database;
use kys_engine::parser;
use kys_engine::research;
use kys_engine::analysis;

#[derive(Serialize)]
pub struct Project {
    id: String,
    title: String,
    category: String,
    score: Option<u32>,
    grade: String,
    status: String,
}

#[derive(Serialize)]
pub struct DashboardStats {
    total_projects: String,
    total_projects_trend: String,
    avg_score: String,
    avg_score_trend: String,
    risk_projects: String,
    risk_projects_trend: String,
}

#[derive(Serialize, Clone)]
pub struct ScoreDetail {
    total: u32,
    grade: String,
    category_fit: u32,
    completeness: u32,
    reference_quality: u32,
    technical_depth: u32,
}

#[derive(Serialize, Clone)]
pub struct SimilarityMatch {
    title: String,
    source_type: String,
    similarity_score: f64,
}

#[derive(Serialize, Clone)]
pub struct SimilarityDetail {
    overall_score: f64,
    originality_label: String,
    matches: Vec<SimilarityMatch>,
}

#[derive(Serialize, Clone)]
pub struct ProjectDetail {
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

#[tauri::command]
async fn get_recent_projects(db: tauri::State<'_, Arc<Database>>) -> Result<Vec<Project>, String> {
    let rows = db.list_projects().await.map_err(|e| e.to_string())?;
    
    let mut projects = Vec::new();
    for (id, filename, score, grade) in rows {
        projects.push(Project {
            id: id.to_string(),
            title: filename,
            category: "Genel".into(), 
            score: if score > 0.0 { Some(score as u32) } else { None },
            grade: grade.clone(),
            status: if score > 0.0 { "Tamamlandı".into() } else { "İnceleniyor".into() },
        });
    }
    
    Ok(projects)
}

#[tauri::command]
async fn analyze_project_pdf(file_path: String, db: tauri::State<'_, Arc<Database>>) -> Result<ProjectDetail, String> {
    // 1. KYS-Engine ile PDF'i parse et
    let mut document = parser::parse_file(&file_path)
        .map_err(|e| format!("PDF okuma hatası: {}", e))?;
    
    // 2. DB'ye boş haliyle ekle (İnceleniyor durumuna geçmesi için)
    let project_id = db.save_project(&document).await.map_err(|e| e.to_string())?;

    // 3. İnternet araştırması (Async)
    let search_results = research::search_related_sources(&document.keywords, "")
        .await
        .unwrap_or_default(); 
        
    // 4. Benzerlik analizi
    let similarity_result = analysis::compute_similarity(&document, &search_results);
    
    // 5. Taksonomi ve Skorlama
    let category_fit = 85.0; 
    let classified_category = Some("Yapay Zeka / Teknoloji".to_string());
    document.classified_category = classified_category.clone();
    
    let score = analysis::score_document(&document, category_fit, classified_category);
    
    // 6. Sonuçları veritabanına kaydet
    db.save_score(project_id, &score).await.map_err(|e| e.to_string())?;
    db.save_similarity(project_id, &similarity_result).await.map_err(|e| e.to_string())?;
    
    // 7. Oluşturulan kaydı tam detaylı modeliyle arayüze dön
    get_project_details(project_id.to_string(), db).await
}

#[tauri::command]
async fn get_project_details(id: String, db: tauri::State<'_, Arc<Database>>) -> Result<ProjectDetail, String> {
    let project_id = id.replace("PRJ-", "").parse::<i64>().map_err(|_| "Geçersiz ID formatı".to_string())?;
    
    let result = db.get_project_details_full(project_id).await.map_err(|e| e.to_string())?;
    
    if let Some((proj, score_opt, matches)) = result {
        let score = score_opt.unwrap_or(kys_engine::database::ScoreRecord {
            category_fit: 0.0, completeness: 0.0, reference_quality: 0.0,
            technical_depth: 0.0, originality: 0.0, total_score: 0.0, grade: "-".into()
        });
        
        let detail = ProjectDetail {
            id: proj.id.to_string(),
            title: proj.filename,
            category: "Sistem".into(),
            author: "KYS Kullanıcısı".into(), 
            submit_date: proj.created_at,
            status: if score.total_score > 0.0 { "Tamamlandı".into() } else { "İnceleniyor".into() },
            score: ScoreDetail {
                total: score.total_score as u32,
                grade: score.grade.clone(),
                category_fit: score.category_fit as u32,
                completeness: score.completeness as u32,
                reference_quality: score.reference_quality as u32,
                technical_depth: score.technical_depth as u32,
            },
            similarity: SimilarityDetail {
                overall_score: score.originality as f64 / 100.0, 
                originality_label: "Analiz Edildi".into(), 
                matches: matches.into_iter().map(|m| SimilarityMatch {
                    title: m.title,
                    source_type: m.source_type,
                    similarity_score: m.similarity_score as f64,
                }).collect()
            },
            pdf_url: None, // Tauri asset loading ile alınabilir
        };
        Ok(detail)
    } else {
        Err("Proje bulunamadı".to_string())
    }
}

#[tauri::command]
fn get_dashboard_stats() -> DashboardStats {
    DashboardStats {
        total_projects: "14k".into(),
        total_projects_trend: "+25%".into(),
        avg_score: "325".into(),
        avg_score_trend: "-25%".into(),
        risk_projects: "200k".into(),
        risk_projects_trend: "+5%".into(),
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                // Engine klasöründeki env dosyasını yüklemeyi dene
                let _ = dotenvy::from_path("../../kys-engine/.env"); 
                
                // Yoksa Supabase env variable fallback
                let db_url = std::env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgresql://postgres.fgwctkxmsaczqzvlbyol:Rick3429!%3F31@aws-0-eu-central-1.pooler.supabase.com:6543/postgres".into());
                
                if let Ok(db) = Database::new(&db_url).await {
                    app_handle.manage(Arc::new(db));
                    println!(">>> Veritabanı Tauri'ye başarıyla bağlandı! <<<");
                } else {
                    println!("!!! Veritabanı bağlantı hatası !!!");
                }
            });
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            auth::register_user,
            auth::verify_otp,
            get_recent_projects,
            get_dashboard_stats,
            get_project_details,
            analyze_project_pdf
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
