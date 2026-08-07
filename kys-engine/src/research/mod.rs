// KYS - İnternet Araştırma Motoru
// Brave Search API ile kaynak bulur, HTTP hatalarını yönetir

use crate::models::SearchResult;
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use tracing::{info, warn};

/// Brave Search API yanıt yapısı
#[derive(Debug, Deserialize)]
struct BraveResponse {
    web: Option<BraveWebResults>,
}

#[derive(Debug, Deserialize)]
struct BraveWebResults {
    results: Vec<BraveResult>,
}

#[derive(Debug, Deserialize)]
struct BraveResult {
    title: String,
    url: String,
    description: Option<String>,
}

/// Anahtar kelimelere göre Brave Search ile kaynak bulur
pub async fn search_related_sources(
    keywords: &[String],
    api_key: &str,
) -> Result<Vec<SearchResult>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("KYS-Research-Engine/0.1")
        .build()?;

    let mut all_results: Vec<SearchResult> = Vec::new();

    // En önemli 5 keyword ile arama yap (kota tasarrufu)
    let search_terms: Vec<String> = keywords
        .iter()
        .take(5)
        .map(|k| k.clone())
        .collect();

    for keyword in &search_terms {
        info!("Brave Search: '{}'", keyword);
        
        match search_brave(&client, keyword, api_key).await {
            Ok(results) => {
                all_results.extend(results);
                // Rate limit - her istek arasında bekle
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Err(e) => {
                warn!("Brave Search hatası ('{}'): {}", keyword, e);
            }
        }
    }

    // Tekrar eden URL'leri kaldır
    all_results.dedup_by(|a, b| a.url == b.url);

    info!("Toplam {} kaynak bulundu", all_results.len());

    // Bulunan kaynakları içerik için fetch et
    let mut fetched_results = Vec::new();
    for mut result in all_results.into_iter().take(10) {
        match fetch_content(&client, &result.url).await {
            Ok((status, content)) => {
                result.http_status = status;
                if status == 200 {
                    result.fetched_content = Some(content);
                    result.source_type = classify_source(&result.url);
                    fetched_results.push(result);
                } else {
                    warn!("HTTP {}: {} - atlanıyor", status, result.url);
                }
            }
            Err(e) => {
                warn!("Fetch hatası ({}): {} - atlanıyor", result.url, e);
            }
        }
    }

    Ok(fetched_results)
}

/// Brave Search API'ye tek sorgu atar
async fn search_brave(
    client: &Client,
    query: &str,
    api_key: &str,
) -> Result<Vec<SearchResult>> {
    // Akademik kaynaklara öncelik ver
    let academic_query = format!(
        "{} site:arxiv.org OR site:github.com OR site:semanticscholar.org OR filetype:pdf",
        query
    );

    let response = client
        .get("https://api.search.brave.com/res/v1/web/search")
        .header("X-Subscription-Token", api_key)
        .header("Accept", "application/json")
        .query(&[
            ("q", academic_query.as_str()),
            ("count", "5"),
            ("safesearch", "off"),
        ])
        .send()
        .await?;

    let status = response.status().as_u16();

    match status {
        200 => {
            let brave_resp: BraveResponse = response.json().await?;
            let results = brave_resp
                .web
                .map(|w| w.results)
                .unwrap_or_default()
                .into_iter()
                .map(|r| SearchResult {
                    title: r.title,
                    url: r.url.clone(),
                    snippet: r.description.unwrap_or_default(),
                    source_type: classify_source(&r.url),
                    fetched_content: None,
                    http_status: 0,
                })
                .collect();
            Ok(results)
        }
        429 => {
            warn!("Rate limit! 5 saniye bekleniyor...");
            tokio::time::sleep(Duration::from_secs(5)).await;
            Err(anyhow::anyhow!("Rate limited (429)"))
        }
        403 => {
            warn!("API erişim engeli (403) - API anahtarını kontrol edin");
            Err(anyhow::anyhow!("Forbidden (403)"))
        }
        _ => Err(anyhow::anyhow!("Beklenmedik HTTP durumu: {}", status)),
    }
}

/// URL'den içerik çeker - hata yönetimiyle
async fn fetch_content(client: &Client, url: &str) -> Result<(u16, String)> {
    // PDF'ler için binary fetch - şimdilik atla
    if url.ends_with(".pdf") {
        return Ok((200, format!("[PDF kaynağı: {}]", url)));
    }

    let response = client
        .get(url)
        .timeout(Duration::from_secs(10))
        .header("User-Agent", "Mozilla/5.0 KYS-Research/0.1")
        .send()
        .await?;

    let status = response.status().as_u16();

    match status {
        200 => {
            let text = response.text().await?;
            // HTML taglerini temizle, ilk 2000 karakter al
            let clean = clean_html(&text);
            Ok((200, clean[..clean.len().min(2000)].to_string()))
        }
        404 => {
            warn!("404 Not Found: {}", url);
            Ok((404, String::new()))
        }
        403 => {
            warn!("403 Forbidden: {} - log tutuldu", url);
            Ok((403, String::new()))
        }
        429 => {
            warn!("429 Too Many Requests: {} - 3s bekleniyor", url);
            tokio::time::sleep(Duration::from_secs(3)).await;
            Ok((429, String::new()))
        }
        code => {
            warn!("Bilinmeyen HTTP {}: {}", code, url);
            Ok((code, String::new()))
        }
    }
}

/// HTML taglerini temizler
fn clean_html(html: &str) -> String {
    let tag_re = regex::Regex::new(r"<[^>]+>").unwrap();
    let whitespace_re = regex::Regex::new(r"\s+").unwrap();
    let no_tags = tag_re.replace_all(html, " ");
    whitespace_re.replace_all(&no_tags, " ").trim().to_string()
}

/// URL'e göre kaynak tipini sınıflandırır
fn classify_source(url: &str) -> String {
    if url.contains("arxiv.org") {
        "academic".to_string()
    } else if url.contains("github.com") {
        "github".to_string()
    } else if url.contains("semanticscholar.org") || url.contains("researchgate.net") {
        "academic".to_string()
    } else if url.contains("wikipedia.org") {
        "encyclopedia".to_string()
    } else if url.ends_with(".pdf") {
        "pdf".to_string()
    } else if url.contains("docs.") || url.contains("/docs/") || url.contains("documentation") {
        "documentation".to_string()
    } else {
        "web".to_string()
    }
}
