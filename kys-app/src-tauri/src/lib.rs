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
    word_count: u32,
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
    for p in rows {
        projects.push(Project {
            id: p.id.to_string(),
            title: p.filename,
            category: p.category, 
            score: if p.total_score > 0.0 { Some(p.total_score as u32) } else { None },
            grade: p.grade,
            status: p.status,
            word_count: p.word_count as u32,
        });
    }
    
    Ok(projects)
}

#[tauri::command]
async fn analyze_project_pdf(file_path: String, db: tauri::State<'_, Arc<Database>>) -> Result<ProjectDetail, String> {
    // 1. JanissaryAsistan-Engine ile PDF'i parse et
    let mut document = parser::parse_file(&file_path)
        .map_err(|e| format!("PDF okuma hatası: {}", e))?;
    
    // 2. DB'ye boş haliyle ekle (İnceleniyor durumuna geçmesi için)
    let project_id = db.save_project(&document, "Masaüstü Kullanıcısı", &file_path).await.map_err(|e| e.to_string())?;

    // 3. İnternet araştırması (Async)
    let search_results = research::search_related_sources(&document.keywords, "")
        .await
        .unwrap_or_default(); 
        
    // 4. Benzerlik analizi
    let similarity_result = analysis::compute_similarity(&document, &search_results);
    
    // 5. Taksonomi ve Skorlama
    let categories = db.list_evaluation_categories().await.unwrap_or_default();
    let (category_fit, technical_depth, classified_category) = analysis::evaluate_with_ai(&document, &categories).await;
    document.classified_category = classified_category.clone();
    
    let score = analysis::score_document(&document, category_fit, technical_depth, classified_category);
    
    // 6. Sonuçları veritabanına kaydet
    db.save_score(project_id, &score).await.map_err(|e| e.to_string())?;
    db.save_similarity(project_id, &similarity_result).await.map_err(|e| e.to_string())?;
    
    // 7. Oluşturulan kaydı tam detaylı modeliyle arayüze dön
    get_project_details(project_id.to_string(), db).await
}

#[tauri::command]
async fn upload_project_only(
    file_path: String,
    title: Option<String>,
    category: Option<String>,
    db: tauri::State<'_, Arc<Database>>
) -> Result<ProjectDetail, String> {
    // Sadece parse edip DB'ye kaydediyoruz, analiz etmiyoruz.
    let mut document = parser::parse_file(&file_path)
        .map_err(|e| format!("PDF okuma hatası: {}", e))?;
        
    if let Some(t) = title {
        if !t.trim().is_empty() {
            document.filename = t;
        }
    }
    
    let author = document.author.clone().unwrap_or_else(|| "Bilinmeyen Kullanıcı".to_string());
    let project_id = db.save_project(&document, &author, &file_path).await.map_err(|e| e.to_string())?;
    
    if let Some(c) = category {
        if !c.trim().is_empty() {
            let _ = db.update_project_category(project_id, &c).await;
        } else {
            let _ = db.update_project_category(project_id, "Belirtilmedi").await;
        }
    } else {
        let _ = db.update_project_category(project_id, "Belirtilmedi").await;
    }
    
    get_project_details(project_id.to_string(), db).await
}

#[tauri::command]
async fn analyze_existing_project(id: String, db: tauri::State<'_, Arc<Database>>) -> Result<ProjectDetail, String> {
    let project_id = id.replace("PRJ-", "").parse::<i64>().map_err(|_| "Geçersiz ID".to_string())?;

    let file_path = db.get_file_path(project_id).await
        .map_err(|e| format!("Veritabanı hatası: {}", e))?
        .ok_or_else(|| "Projenin dosya yolu bulunamadı".to_string())?;

    // Analiz için tekrar parse et
    let mut document = parser::parse_file(&file_path)
        .map_err(|e| format!("PDF okuma hatası: {}", e))?;

    let search_results = research::search_related_sources(&document.keywords, "")
        .await
        .unwrap_or_default(); 
        
    let similarity_result = analysis::compute_similarity(&document, &search_results);
    
    let taxonomy = kys_engine::analysis::taxonomy::Taxonomy::load_from_file("C:/Users/emirh/Desktop/t3proje/kaynak_tarama_taxonomy.md");
    let (tax_category, tax_score) = taxonomy.classify(&document.keywords);
    let final_category = tax_category.unwrap_or_else(|| "Genel (Kategori Bulunamadı)".to_string());

    let categories = db.list_evaluation_categories().await.unwrap_or_default();
    let (mut category_fit, technical_depth, _) = analysis::evaluate_with_ai(&document, &categories).await;
    
    // Taxonomy ve AI skorlarını birleştir
    category_fit = (category_fit + tax_score) / 2.0;
    document.classified_category = Some(final_category.clone());
    
    let mut score = analysis::score_document(&document, category_fit, technical_depth, Some(final_category.clone()));
    
    // Özgünlük (originality) değerini benzerlik raporundan çekerek güncelle
    analysis::update_score_with_similarity(&mut score, &similarity_result);
    
    // Projenin kategorisini DB'de güncelle
    db.update_project_category(project_id, &final_category).await.map_err(|e| e.to_string())?;
    
    db.save_score(project_id, &score).await.map_err(|e| e.to_string())?;
    db.save_similarity(project_id, &similarity_result).await.map_err(|e| e.to_string())?;
    
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
            category: proj.category,
            author: proj.author, 
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
                overall_score: 1.0 - (score.originality as f64 / 100.0), 
                originality_label: match score.originality as f64 {
                    s if s > 80.0 => "Tamamen Özgün".into(),
                    s if s > 50.0 => "Düşük Risk".into(),
                    s if s > 30.0 => "Riskli (İnceleme Gerekli)".into(),
                    _ => "Kopya Uyarısı".into(),
                },
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
async fn update_project_category(id: String, category: String, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    let project_id = id.replace("PRJ-", "").parse::<i64>().map_err(|_| "Geçersiz ID formatı".to_string())?;
    db.update_project_category(project_id, &category).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_dashboard_stats(db: tauri::State<'_, Arc<Database>>) -> Result<DashboardStats, String> {
    let rows = db.list_projects().await.map_err(|e| e.to_string())?;
    
    let total = rows.len();
    let scored: Vec<f64> = rows.iter().filter(|r| r.total_score > 0.0).map(|r| r.total_score).collect();
    let avg = if scored.is_empty() { 0.0 } else { scored.iter().sum::<f64>() / scored.len() as f64 };
    let risk = rows.iter().filter(|r| r.total_score > 0.0 && r.total_score < 50.0).count();

    Ok(DashboardStats {
        total_projects: total.to_string(),
        total_projects_trend: "+0%".into(), // Real trend requires time-based SQL which we skip for now
        avg_score: format!("{:.0}", avg),
        avg_score_trend: "+0%".into(),
        risk_projects: risk.to_string(),
        risk_projects_trend: "+0%".into(),
    })
}

#[derive(Serialize)]
pub struct ChartData {
    area_data: Vec<serde_json::Value>,
    bar_data: Vec<serde_json::Value>,
}

#[tauri::command]
async fn get_chart_data(db: tauri::State<'_, Arc<Database>>) -> Result<ChartData, String> {
    let rows = db.list_projects().await.map_err(|e| e.to_string())?;
    
    // Basit bir gruplama yapalım. Gerçek projelerin isimlerini ve kelime sayılarını area chart'a koyalım.
    // 6 öğe ile sınırlayalım.
    let mut area_data = Vec::new();
    for p in rows.iter().take(6) {
        area_data.push(serde_json::json!({
            "name": p.filename.chars().take(10).collect::<String>(),
            "val1": p.word_count, // Kelime Sayısı
            "val2": 0,
            "val3": 0,
        }));
    }
    // Eğer boşsa fallback veri koyalım grafiğin boş görünmemesi için
    if area_data.is_empty() {
        area_data.push(serde_json::json!({ "name": "Boş", "val1": 0, "val2": 0, "val3": 0 }));
    }

    // Bar chart için durumları sayalım (İncelendi, Beklemede, Hata)
    let mut incelendi = 0;
    let mut beklemede = 0;
    let mut hata = 0;
    
    for p in &rows {
        if p.status == "Tamamlandı" || p.status == "İncelendi" {
            incelendi += 1;
        } else if p.status == "Hata" {
            hata += 1;
        } else {
            beklemede += 1;
        }
    }

    let bar_data = vec![
        serde_json::json!({
            "name": "Projeler",
            "val1": incelendi,
            "val2": beklemede,
            "val3": hata,
        })
    ];

    Ok(ChartData {
        area_data,
        bar_data,
    })
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn ask_copilot(project_id: String, project_title: String, message: String, context: Option<String>) -> Result<String, String> {
    // Try to load .env from the app root (where the binary runs or dev server is)
    let _ = dotenvy::from_path("../../kys-app/.env").or_else(|_| dotenvy::dotenv());

    let api_key = std::env::var("OPENROUTER_API_KEY").map_err(|_| "OPENROUTER_API_KEY bulunamadı. Lütfen .env dosyasını kontrol edin.".to_string())?;
    
    let client = reqwest::Client::new();
    
    let system_prompt = format!(
        "Sen Janissary Copilot'sun. Tek hedefin, '{}' (ID: {}) projesi hakkında kullanıcıya yardımcı olmaktır. \
        Görev Sınırların ve Kuralların: \
        1. Sadece yüklenen PDF içeriği, Rust sisteminin sağladığı analiz bulguları, skorlar ve intihal linkleri hakkında konuş. \
        2. Proje kapsamı dışındaki genel sohbetleri veya alakasız soruları reddet. \
        3. Sistemin sana aktardığı analiz sonuçlarını ve PDF'den çıkarılan kelime/cümle bağlamlarını detaylıca tartış, açıkla ve değerlendir. \
        Lütfen kısa, net, profesyonel ve Türkçe yanıt ver.",
        project_title, project_id
    );
    
    let mut messages = vec![
        serde_json::json!({
            "role": "system",
            "content": system_prompt
        })
    ];
    
    if let Some(ctx) = context {
        if !ctx.trim().is_empty() {
            messages.push(serde_json::json!({
                "role": "system",
                "content": format!("Kullanıcı şu an analiz edilen şu içeriğe odaklanıyor: '{}'", ctx)
            }));
        }
    }
    
    messages.push(serde_json::json!({
        "role": "user",
        "content": message
    }));
    
    let request_body = serde_json::json!({
        "model": "nvidia/nemotron-3-nano-omni-30b-a3b-reasoning:free", // User requested model
        "messages": messages
    });

    let res = client.post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("API İstek hatası: {}", e))?;
        
    let response_json: serde_json::Value = res.json().await.map_err(|e| format!("JSON parse hatası: {}", e))?;
    
    if let Some(error) = response_json.get("error") {
        return Err(format!("OpenRouter API Hatası: {}", error));
    }

    let content = response_json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("Yanıt alınamadı.")
        .to_string();
        
    Ok(content)
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
                    .unwrap_or_else(|_| "postgresql://postgres.fgwctkxmsaczqzvlbyol:Rick3429%21%3F31@aws-0-eu-central-1.pooler.supabase.com:6543/postgres".into());
                
                match Database::new(&db_url).await {
                    Ok(db) => {
                        app_handle.manage(Arc::new(db));
                        println!(">>> Veritabanı Tauri'ye başarıyla bağlandı! <<<");
                    }
                    Err(e) => {
                        println!("!!! Veritabanı bağlantı hatası !!!\nHata detayı: {:?}", e);
                    }
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
            analyze_project_pdf,
            upload_project_only,
            analyze_existing_project,
            get_chart_data,
            update_project_category,
            ask_copilot
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
