// JanissaryAsistan - Analiz ve Puanlama Motoru
// Benzerlik hesaplama + kural tabanlı değerlendirme

use crate::models::*;
use crate::parser::extract_keywords;
use std::collections::HashSet;
use reqwest::Client;
use serde_json::Value;

pub mod taxonomy;

/// Proje belgesi ile internet kaynakları arasındaki benzerliği hesaplar
pub fn compute_similarity(
    document: &Document,
    search_results: &[SearchResult],
) -> SimilarityReport {
    let doc_keywords: HashSet<String> = document.keywords.iter().cloned().collect();
    let mut matches = Vec::new();

    for result in search_results {
        if let Some(content) = &result.fetched_content {
            let source_keywords: HashSet<String> = extract_keywords(content).into_iter().collect();

            // Jaccard benzerliği: kesişim / birleşim
            let intersection: HashSet<&String> = doc_keywords.intersection(&source_keywords).collect();
            let union: HashSet<&String> = doc_keywords.union(&source_keywords).collect();

            let mut similarity_score = if union.is_empty() {
                0.0
            } else {
                intersection.len() as f64 / union.len() as f64
            };

            let matched_keywords: Vec<String> = intersection
                .into_iter()
                .take(5)
                .cloned()
                .collect();

            let mut explanation = generate_similarity_explanation(similarity_score, &matched_keywords);
            
            // Eğer PDF veya GitHub reposu ise ve kelimelerden bağımsız bir link bulunmuşsa (kullanıcının istediği gibi)
            // Özel bir nihai link uyarısı ekle
            if result.source_type == "pdf" {
                explanation = format!("{} (Nihai Link Uyarısı: Bu bir PDF dokümanıdır. Lütfen içerik benzerliği ihtimaline karşı bağlantıyı bizzat kontrol edin.)", explanation);
                if similarity_score < 0.20 { similarity_score += 0.30; } // PDF'ler her zaman potansiyel risktir
            } else if result.source_type == "github" && similarity_score > 0.15 {
                explanation = format!("{} (Nihai Link Uyarısı: Bu GitHub reposundaki kod ve mimari ile ciddi benzerlikler olabilir. Lütfen kaynak kodları kıyaslayın.)", explanation);
            }

            matches.push(SimilarityMatch {
                title: result.title.clone(),
                url: result.url.clone(),
                source_type: result.source_type.clone(),
                similarity_score,
                matched_keywords,
                explanation,
            });
        }
    }

    // En yüksek benzerlikten düşüğe sırala
    matches.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap());

    let overall_score = if matches.is_empty() {
        0.0
    } else {
        let top_scores: Vec<f64> = matches.iter().take(3).map(|m| m.similarity_score).collect();
        top_scores.iter().sum::<f64>() / top_scores.len() as f64
    };

    SimilarityReport {
        overall_score,
        matches,
    }
}

/// Benzerlik açıklaması üretir (template tabanlı)
fn generate_similarity_explanation(score: f64, matched_keywords: &[String]) -> String {
    let kw_str = matched_keywords.join(", ");

    match score {
        s if s < 0.10 => format!(
            "Düşük teknik örtüşme. Ortak terimler: {}",
            if kw_str.is_empty() { "bulunamadı".to_string() } else { kw_str }
        ),
        s if s < 0.25 => format!(
            "Benzer teknik yaklaşımlar bulundu. Örtüşen kavramlar: {}",
            kw_str
        ),
        s if s < 0.45 => format!(
            "Benzer yöntem kullanan çalışma tespit edildi. Ortak alanlar: {}",
            kw_str
        ),
        s if s < 0.65 => format!(
            "Bu problem daha önce benzer şekilde ele alınmış. İlgili kavramlar: {}",
            kw_str
        ),
        _ => format!(
            "Yüksek teknik benzerlik. Ortak terimler: {}. Jüri incelemesi önerilir.",
            kw_str
        ),
    }
}

/// Kural tabanlı belge puanlaması (AI olmadan)
pub fn score_document(document: &Document, category_fit: f64, technical_depth: f64, classified_category: Option<String>, semantic_reason: Option<String>) -> ScoreCard {
    ScoreCard {
        category_fit,
        classified_category,
        completeness: score_completeness(document),
        reference_quality: score_references(document),
        technical_depth,
        originality: 75.0, // Varsayılan - benzerlik analizi sonrası güncellenir
        ai_probability: 0.0, // Varsayılan - benzerlik ve AI analizi sonrası güncellenir
        semantic_reason,
    }
}

/// OpenAI/OpenRouter ile dinamik kategori uyumu ve teknik derinlik hesaplaması
pub async fn evaluate_with_ai(
    document: &Document,
    categories: &[(i32, String, String)],
) -> (f64, f64, f64, Option<String>, Option<String>) {
    use tracing::{info, warn, error};

    let api_key = match std::env::var("OPENAI_API_KEY") {
        Ok(key) if !key.is_empty() => key,
        _ => {
            warn!("OPENAI_API_KEY bulunamadı — AI analizi atlanıyor");
            return (75.0, score_technical_depth(document), 0.0, Some("Genel (AI Kapalı)".to_string()), None);
        }
    };

    let categories_json = serde_json::to_string(&categories).unwrap_or_default();

    let default_prompt = format!(
        "Aşağıdaki proje metnini dikkatle oku ve incele:\n---\n{}\n---\n\n\
        Kategorilerden bu projeye en uygun olanı seç: {}\n\n\
        DİKKAT: Sadece ve SADECE aşağıdaki JSON formatında yanıt ver, başka hiçbir şey yazma:\n\
        {{\"category_name\": \"...\", \"category_fit\": 80, \"technical_depth\": 75, \"ai_probability\": 15.5, \"reason\": \"...\"}}",
        document.raw_text.chars().take(2000).collect::<String>(),
        categories_json
    );

    let system_prompt_env = std::env::var("SYSTEM_PROMPT").unwrap_or_default();

    let prompt = if !system_prompt_env.is_empty() {
        format!(
            "{}\n\nProje Metni (İlk 2000 karakter):\n{}\n\nKategoriler: {}\n\n\
            SADECE şu JSON formatında yanıt ver:\n\
            {{\"category_name\": \"...\", \"category_fit\": 80, \"technical_depth\": 75, \"ai_probability\": 15.5, \"reason\": \"Açıklama...\"}} ",
            system_prompt_env,
            document.raw_text.chars().take(2000).collect::<String>(),
            categories_json
        )
    } else {
        default_prompt
    };

    let model_name = std::env::var("AI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
    info!("AI analizi başlatılıyor: model={}", model_name);

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .unwrap_or_default();

    // response_format kaldırıldı — tüm modellerle uyumlu olması için
    let request_body = serde_json::json!({
        "model": model_name,
        "messages": [
            {"role": "system", "content": "Sen bir proje değerlendirme asistanısın. Yanıtını SADECE geçerli JSON formatında ver, başka metin ekleme."},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.2,
        "max_tokens": 500
    });

    let api_url = if api_key.starts_with("sk-or-") {
        "https://openrouter.ai/api/v1/chat/completions"
    } else {
        "https://api.openai.com/v1/chat/completions"
    };

    info!("API isteği gönderiliyor: {}", api_url);

    let resp_result = client.post(api_url)
        .bearer_auth(&api_key)
        .header("HTTP-Referer", "http://localhost:8080")
        .header("X-Title", "JanissaryAsistan")
        .json(&request_body)
        .send()
        .await;

    match resp_result {
        Err(e) => {
            error!("AI API bağlantı hatası: {}", e);
            return (75.0, score_technical_depth(document), 0.0, Some("Genel (Bağlantı Hatası)".to_string()), None);
        }
        Ok(resp) => {
            let status = resp.status();
            info!("AI API yanıt kodu: {}", status);

            match resp.text().await {
                Err(e) => {
                    error!("AI API yanıt okunamadı: {}", e);
                }
                Ok(raw_text) => {
                    info!("AI API ham yanıt (ilk 500 karakter): {}", &raw_text.chars().take(500).collect::<String>());

                    // Önce tam JSON parse dene
                    let content_opt: Option<String> = serde_json::from_str::<Value>(&raw_text)
                        .ok()
                        .and_then(|j| j["choices"][0]["message"]["content"].as_str().map(|s| s.to_string()));

                    // Yanıt içinden JSON bloğu çıkar (```json...``` veya {...} formatı)
                    let json_str = content_opt.as_deref().unwrap_or(&raw_text);

                    // JSON bloğunu bul — önce ```json ... ``` ara, sonra { ... }
                    let extracted = extract_json_from_text(json_str);

                    if let Ok(parsed) = serde_json::from_str::<Value>(&extracted) {
                        let cat_name = parsed["category_name"].as_str().unwrap_or("Genel").to_string();
                        let cat_fit = parsed["category_fit"].as_f64().unwrap_or(75.0);
                        let tech_depth = parsed["technical_depth"].as_f64().unwrap_or(score_technical_depth(document));
                        let ai_prob = parsed["ai_probability"].as_f64().unwrap_or(0.0);
                        let reason = parsed["reason"].as_str().map(|s| s.to_string());
                        info!("AI analizi başarılı: kategori={}, uyum={}, ai_ihtimal={}", cat_name, cat_fit, ai_prob);
                        return (cat_fit, tech_depth, ai_prob, Some(cat_name), reason);
                    } else {
                        error!("AI yanıtı JSON olarak parse edilemedi. Ham: {}", &extracted.chars().take(300).collect::<String>());
                    }
                }
            }
        }
    }

    (75.0, score_technical_depth(document), 0.0, Some("Genel (Parse Hatası)".to_string()), None)
}

/// Metinden JSON bloğunu çıkarır (```json...``` veya ilk { ... } bloğu)
fn extract_json_from_text(text: &str) -> String {
    // Önce ```json ... ``` bloğu ara
    if let Some(start) = text.find("```json") {
        let after = &text[start + 7..];
        if let Some(end) = after.find("```") {
            return after[..end].trim().to_string();
        }
    }
    // Sonra ``` ... ``` bloğu ara
    if let Some(start) = text.find("```") {
        let after = &text[start + 3..];
        if let Some(end) = after.find("```") {
            return after[..end].trim().to_string();
        }
    }
    // Son olarak { ... } bloğu ara
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            if end > start {
                return text[start..=end].to_string();
            }
        }
    }
    text.to_string()
}



/// Bölüm tamlığı: özet, giriş, sonuç, yöntem var mı?
fn score_completeness(doc: &Document) -> f64 {
    let mut score: f64 = 0.0;
    let max: f64 = 100.0;

    // Her kritik bölüm +25 puan
    if doc.has_abstract {
        score += 25.0;
    }
    if doc.has_conclusion {
        score += 25.0;
    }
    if doc.has_methodology {
        score += 25.0;
    }
    if doc.has_references {
        score += 25.0;
    }

    // Kelime sayısı bonusu
    let word_bonus: f64 = match doc.word_count {
        0..=500 => -10.0,
        501..=1000 => 0.0,
        1001..=3000 => 5.0,
        _ => 10.0,
    };

    (score + word_bonus).max(0.0_f64).min(max)
}

/// Kaynak kalitesi: kaynakça dolu mu?
fn score_references(doc: &Document) -> f64 {
    if !doc.has_references {
        return 20.0; // Kaynakça yok = çok düşük puan
    }

    let ref_count = doc.references.len();
    
    // DOI içeren referanslar daha değerli
    let doi_count = doc.references.iter()
        .filter(|r| r.starts_with("DOI:"))
        .count();

    let base = match ref_count {
        0 => 20.0,
        1..=3 => 45.0,
        4..=7 => 65.0,
        8..=15 => 80.0,
        _ => 90.0,
    };

    let doi_bonus = (doi_count as f64 * 5.0).min(10.0);
    (base + doi_bonus).min(100.0)
}

/// Teknik derinlik: kelime sayısı + bölüm sayısı + referans yoğunluğu
fn score_technical_depth(doc: &Document) -> f64 {
    let word_score: f64 = match doc.word_count {
        0..=300 => 20.0,
        301..=800 => 40.0,
        801..=2000 => 60.0,
        2001..=5000 => 75.0,
        _ => 85.0,
    };

    let section_score: f64 = match doc.sections.len() {
        0..=1 => 0.0,
        2..=3 => 10.0,
        4..=6 => 15.0,
        _ => 20.0,
    };

    // Ortalama bölüm uzunluğu bonusu
    let avg_section_words = if doc.sections.is_empty() {
        0
    } else {
        doc.sections.iter().map(|s| s.word_count).sum::<usize>() / doc.sections.len()
    };

    let depth_bonus: f64 = match avg_section_words {
        0..=50 => -5.0,
        51..=150 => 0.0,
        151..=400 => 5.0,
        _ => 10.0,
    };

    (word_score + section_score + depth_bonus).max(0.0_f64).min(100.0_f64)
}

/// Benzerlik raporundan özgünlük puanı üretir (0-100, yüksek = özgün)
pub fn originality_from_similarity(similarity: &SimilarityReport) -> f64 {
    // Benzerlik yüksekse özgünlük düşük
    let base = (1.0 - similarity.overall_score) * 100.0;
    base.max(0.0).min(100.0)
}

/// Puanı günceller - benzerlik analizi tamamlandıktan sonra çağrılır
pub fn update_score_with_similarity(score: &mut ScoreCard, similarity: &SimilarityReport) {
    score.originality = originality_from_similarity(similarity);
}
