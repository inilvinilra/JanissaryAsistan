// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod auth;

use serde::Serialize;

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

use std::collections::HashMap;
use std::sync::Mutex;
use kys_engine::parser;
use kys_engine::research;
use kys_engine::analysis;

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

pub struct AppState {
    pub projects: Mutex<HashMap<String, ProjectDetail>>,
}

#[tauri::command]
fn get_recent_projects() -> Vec<Project> {
    // Şimdilik test verisi dönüyoruz, ileride kys_engine::database üzerinden çekilecek
    vec![
        Project { id: "PRJ-2041".into(), title: "Görüntü İşleme ile Yüz Tanıma".into(), category: "Yapay Zeka".into(), score: Some(92), grade: "A".into(), status: "Tamamlandı".into() },
        Project { id: "PRJ-2042".into(), title: "Otonom Tarım Robotu".into(), category: "Robotik".into(), score: Some(85), grade: "B+".into(), status: "Tamamlandı".into() },
        Project { id: "PRJ-2043".into(), title: "Akıllı Ev Güvenlik Sistemi".into(), category: "Nesnelerin İnterneti".into(), score: Some(72), grade: "C".into(), status: "Uyarı: Benzerlik".into() },
        Project { id: "PRJ-2044".into(), title: "Güneş Paneli Verimlilik Analizi".into(), category: "Enerji".into(), score: Some(45), grade: "F".into(), status: "Kopya İhtimali".into() },
        Project { id: "PRJ-2045".into(), title: "Deprem Erken Uyarı Ağı".into(), category: "Afet Yönetimi".into(), score: None, grade: "-".into(), status: "İnceleniyor".into() },
    ]
}

#[tauri::command]
async fn analyze_project_pdf(file_path: String, state: tauri::State<'_, AppState>) -> Result<ProjectDetail, String> {
    // 1. KYS-Engine ile PDF'i parse et
    let mut document = parser::parse_file(&file_path)
        .map_err(|e| format!("PDF okuma hatası: {}", e))?;
    
    // 2. İnternet araştırması (Async)
    let search_results = research::search_related_sources(&document.keywords, "")
        .await
        .unwrap_or_default(); // Hata olursa boş geç
        
    // 3. Benzerlik analizi
    let similarity_result = analysis::compute_similarity(&document, &search_results);
    
    // 4. Taksonomi ve Skorlama
    // Rust tarafındaki assets yolu masaüstünde farklı olabilir, şimdilik statik bir fit veriyoruz
    let category_fit = 85.0; 
    let classified_category = Some("Yapay Zeka / Teknoloji".to_string());
    document.classified_category = classified_category.clone();
    
    let score = analysis::score_document(&document, category_fit, classified_category);
    
    // Tauri Structlarına Mapleme
    let matches: Vec<SimilarityMatch> = similarity_result.matches.into_iter().map(|m| {
        SimilarityMatch {
            title: m.title,
            source_type: m.source_type,
            similarity_score: m.similarity_score,
        }
    }).collect();
    
    // Yeni ID üret
    let id = format!("PRJ-{}", rand::random::<u16>() % 9000 + 1000);
    let mut filename = std::path::Path::new(&file_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    if filename.is_empty() { filename = "Yeni_Proje".into(); }
    
    let detail = ProjectDetail {
        id: id.clone(),
        title: filename,
        category: document.classified_category.unwrap_or_else(|| "Bilinmiyor".into()),
        author: "KYS Kullanıcısı".into(), // DB'den alınacak
        submit_date: "Şu An".into(), // chrono kullanılabilir
        status: "Yeni Analiz".into(),
        score: ScoreDetail {
            total: score.total() as u32,
            grade: score.grade(),
            category_fit: score.category_fit as u32,
            completeness: score.completeness as u32,
            reference_quality: score.reference_quality as u32,
            technical_depth: score.technical_depth as u32,
        },
        similarity: SimilarityDetail {
            overall_score: similarity_result.overall_score,
            originality_label: similarity_result.originality_label().into(),
            matches,
        },
        // tauri:// protokolü üzerinden okumak için
        pdf_url: Some(format!("https://asset.localhost/{}", file_path.replace("\\", "/"))),
    };
    
    // State'e kaydet
    state.projects.lock().unwrap().insert(id.clone(), detail.clone());
    
    Ok(detail)
}

#[tauri::command]
fn get_project_details(id: String, state: tauri::State<'_, AppState>) -> Result<ProjectDetail, String> {
    let projects = state.projects.lock().unwrap();
    if let Some(project) = projects.get(&id) {
        Ok(project.clone())
    } else {
        // Bulunamazsa Mock veri dön, geliştirme için kolaylık
        Ok(ProjectDetail {
            id: id.clone(),
            title: "Görüntü İşleme ile Yüz Tanıma (Örnek)".into(),
            category: "Yapay Zeka".into(),
            author: "Ahmet Yılmaz (Takım Kaptanı)".into(),
            submit_date: "14 Mayıs 2026".into(),
            status: "Tamamlandı".into(),
            score: ScoreDetail {
                total: 92,
                grade: "A".into(),
                category_fit: 95,
                completeness: 88,
                reference_quality: 90,
                technical_depth: 96,
            },
            similarity: SimilarityDetail {
                overall_score: 0.12,
                originality_label: "Özgünlük Yüksek (Özgün)".into(),
                matches: vec![
                    SimilarityMatch { title: "Yüz Tanıma Sistemleri".into(), source_type: "Akademik Makale".into(), similarity_score: 0.08 },
                ],
            },
            pdf_url: None,
        })
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
        .manage(AppState {
            projects: std::sync::Mutex::new(std::collections::HashMap::new()),
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
