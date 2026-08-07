// KYS - İnternet Araştırma Motoru (Federated Search)
// GitHub ve Semantic Scholar API'lerini doğrudan kullanarak (API Key gerektirmez) kaynak bulur.

use crate::models::SearchResult;
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use tracing::{info, warn};

// Github API Response
#[derive(Debug, Deserialize)]
struct GithubResponse {
    items: Option<Vec<GithubItem>>,
}

#[derive(Debug, Deserialize)]
struct GithubItem {
    html_url: String,
    description: Option<String>,
    name: String,
}

/// Anahtar kelimelere göre açık kaynak API'lerden araştırma yapar
pub async fn search_related_sources(
    keywords: &[String],
    _api_key: &str, // Artık API key kullanmıyoruz
) -> Result<Vec<SearchResult>> {
    // Jüri / Akademik proje olduğumuzu belirten özel User-Agent
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("KYS-Research-Engine/0.1 (T3/TUBITAK Academic Project Evaluation Bot)")
        .build()?;

    let mut all_results: Vec<SearchResult> = Vec::new();

    // En önemli 3 keyword ile arama yap (Rate limitlere takılmamak için)
    let search_terms: Vec<String> = keywords
        .iter()
        .take(3)
        .cloned()
        .collect();

    for keyword in &search_terms {
        info!("Federated Search başlatılıyor: '{}'", keyword);
        
        // 1. GitHub Araması (Açık kaynak yazılım benzerlikleri için)
        match search_github(&client, keyword).await {
            Ok(results) => {
                all_results.extend(results);
            }
            Err(e) => {
                warn!("GitHub Search hatası ('{}'): {}", keyword, e);
            }
        }
        // Hızlı istek atmamak için bekle
        tokio::time::sleep(Duration::from_millis(1500)).await;

        // 2. Semantic Scholar Araması (Akademik makale benzerlikleri için)
        match search_scholar(&client, keyword).await {
            Ok(results) => {
                all_results.extend(results);
            }
            Err(e) => {
                warn!("Semantic Scholar Search hatası ('{}'): {}", keyword, e);
            }
        }
        tokio::time::sleep(Duration::from_millis(1500)).await;
    }

    // Tekrar eden URL'leri kaldır
    all_results.dedup_by(|a, b| a.url == b.url);

    info!("Toplam {} tekil akademik/yazılım kaynağı bulundu", all_results.len());

    // Bulunan kaynakların (örneğin GitHub Readme veya Makale Özeti) içeriklerini topla
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

/// GitHub API'sine şifresiz sorgu atar
async fn search_github(client: &Client, query: &str) -> Result<Vec<SearchResult>> {
    let url = "https://api.github.com/search/repositories";
    let response = client
        .get(url)
        .query(&[("q", query), ("per_page", "3")])
        .send()
        .await?;

    let status = response.status();
    if status.is_success() {
        let gh_resp: GithubResponse = response.json().await?;
        let results = gh_resp.items.unwrap_or_default().into_iter().map(|item| {
            SearchResult {
                title: item.name,
                url: item.html_url,
                snippet: item.description.unwrap_or_default(),
                source_type: "github".to_string(),
                fetched_content: None,
                http_status: 0,
            }
        }).collect();
        Ok(results)
    } else {
        Err(anyhow::anyhow!("HTTP {}", status))
    }
}

/// Semantic Scholar API'sine şifresiz akademik makale sorgusu atar
async fn search_scholar(client: &Client, query: &str) -> Result<Vec<SearchResult>> {
    let url = "https://api.semanticscholar.org/graph/v1/paper/search";
    let response = client
        .get(url)
        .query(&[("query", query), ("limit", "3"), ("fields", "title,url,abstract")])
        .send()
        .await?;

    let status = response.status();
    if status.is_success() {
        // Dinamik JSON parse
        let parsed: serde_json::Value = response.json().await?;
        let mut results = Vec::new();
        
        if let Some(data) = parsed.get("data").and_then(|d| d.as_array()) {
            for item in data {
                if let Some(url) = item.get("url").and_then(|u| u.as_str()) {
                    let title = item.get("title").and_then(|t| t.as_str()).unwrap_or("Bilinmeyen Makale").to_string();
                    let snippet = item.get("abstract").and_then(|a| a.as_str()).unwrap_or("").to_string();
                    results.push(SearchResult {
                        title,
                        url: url.to_string(),
                        snippet,
                        source_type: "academic".to_string(),
                        fetched_content: None,
                        http_status: 0,
                    });
                }
            }
        }
        Ok(results)
    } else {
        Err(anyhow::anyhow!("HTTP {}", status))
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
        .send()
        .await?;

    let status = response.status().as_u16();

    match status {
        200 => {
            let text = response.text().await?;
            let clean = clean_html(&text);
            let truncated: String = clean.chars().take(2000).collect();
            Ok((200, truncated))
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
