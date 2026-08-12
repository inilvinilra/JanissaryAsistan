// JanissaryAsistan - İnternet Araştırma Motoru
// Serper.dev (Google tabanlı) ile akademik PDF, GitHub ve web araması yapar

use crate::models::SearchResult;
use anyhow::Result;
use reqwest::Client;
use std::time::Duration;
use tracing::{info, warn};

/// Anahtar kelimelere göre Serper.dev üzerinden araştırma yapar
pub async fn search_related_sources(
    keywords: &[String],
    _api_key: &str,
) -> Result<Vec<SearchResult>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .build()?;

    // En önemli 3-5 keyword
    let base_query = keywords.iter().take(4).cloned().collect::<Vec<_>>().join(" ");

    let serper_key = std::env::var("SERPER_API_KEY").unwrap_or_default();
    
    let mut all_results: Vec<SearchResult> = Vec::new();

    if !serper_key.is_empty() {
        info!("Serper.dev arama başlatılıyor: '{}'", base_query);

        // 1. PDF araması — filetype:pdf
        let pdf_query = format!("{} filetype:pdf", base_query);
        match search_serper(&client, &pdf_query, &serper_key, "pdf").await {
            Ok(results) => {
                info!("Serper PDF: {} sonuç bulundu", results.len());
                all_results.extend(results);
            }
            Err(e) => warn!("Serper PDF arama hatası: {}", e),
        }

        // 2. GitHub araması
        let gh_query = format!("{} site:github.com", base_query);
        match search_serper(&client, &gh_query, &serper_key, "github").await {
            Ok(results) => {
                info!("Serper GitHub: {} sonuç bulundu", results.len());
                all_results.extend(results);
            }
            Err(e) => warn!("Serper GitHub arama hatası: {}", e),
        }

        // 3. Genel akademik arama
        match search_serper(&client, &base_query, &serper_key, "web").await {
            Ok(results) => {
                info!("Serper Genel: {} sonuç bulundu", results.len());
                all_results.extend(results);
            }
            Err(e) => warn!("Serper genel arama hatası: {}", e),
        }

    } else {
        warn!("SERPER_API_KEY bulunamadı. DuckDuckGo fallback başlatılıyor.");
        match search_duckduckgo_fallback(&client, &base_query).await {
            Ok(results) => all_results.extend(results),
            Err(e) => warn!("DuckDuckGo hatası: {}", e),
        }
    }

    // Tekrar eden URL'leri kaldır
    all_results.dedup_by(|a, b| a.url == b.url);

    info!("Toplam {} tekil kaynak bulundu", all_results.len());

    // İlk 8 sonucu paralel olarak içerik çek
    let fetch_tasks: Vec<_> = all_results.into_iter().take(8).collect();

    let fetch_futures: Vec<_> = fetch_tasks.iter().map(|result| {
        let client = client.clone();
        let url = result.url.clone();
        let source_type = result.source_type.clone();
        async move {
            fetch_content(&client, &url, &source_type).await
        }
    }).collect();

    let fetch_results = futures::future::join_all(fetch_futures).await;

    let mut fetched = Vec::new();
    for (mut result, fetch_out) in fetch_tasks.into_iter().zip(fetch_results.into_iter()) {
        match fetch_out {
            Ok((status, content)) => {
                result.http_status = status;
                if status == 200 && !content.is_empty() {
                    result.fetched_content = Some(content);
                    fetched.push(result);
                } else if status == 200 && result.source_type == "pdf" {
                    // PDF'ler için snippet'i içerik olarak kullan
                    result.fetched_content = Some(result.snippet.clone());
                    fetched.push(result);
                } else {
                    warn!("HTTP {} veya boş içerik: {} - atlanıyor", status, result.url);
                }
            }
            Err(e) => warn!("Fetch hatası: {} — {}", result.url, e),
        }
    }

    Ok(fetched)
}

/// Serper.dev API üzerinden arama yapar
async fn search_serper(
    client: &Client,
    query: &str,
    api_key: &str,
    source_hint: &str,
) -> Result<Vec<SearchResult>> {
    let body = serde_json::json!({
        "q": query,
        "num": 5,
        "gl": "tr",
        "hl": "tr"
    });

    let response = client
        .post("https://google.serper.dev/search")
        .header("X-API-KEY", api_key)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Serper HTTP {}", response.status()));
    }

    let json: serde_json::Value = response.json().await?;
    let mut results = Vec::new();

    // Organik sonuçları parse et
    if let Some(organic) = json.get("organic").and_then(|o| o.as_array()) {
        for item in organic.iter().take(5) {
            let url = item.get("link").and_then(|u| u.as_str()).unwrap_or_default().to_string();
            let title = item.get("title").and_then(|t| t.as_str()).unwrap_or_default().to_string();
            let snippet = item.get("snippet").and_then(|s| s.as_str()).unwrap_or_default().to_string();

            if url.is_empty() { continue; }

            // Kaynak tipini belirle
            let source_type = if source_hint == "pdf" || url.ends_with(".pdf") {
                "pdf"
            } else if url.contains("github.com") || url.contains("githubusercontent.com") {
                "github"
            } else if url.contains("arxiv.org") || url.contains("semanticscholar.org") || url.contains("researchgate.net") {
                "academic"
            } else {
                "web"
            }.to_string();

            results.push(SearchResult {
                title,
                url,
                snippet,
                source_type,
                fetched_content: None,
                http_status: 0,
            });
        }
    }

    Ok(results)
}

/// DuckDuckGo fallback (Serper key yoksa)
async fn search_duckduckgo_fallback(client: &Client, query: &str) -> Result<Vec<SearchResult>> {
    use scraper::{Html, Selector};
    use urlencoding::encode;

    let url = format!("https://html.duckduckgo.com/html/?q={}", encode(query));
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("DuckDuckGo HTTP {}", response.status()));
    }

    let html_content = response.text().await?;
    let document = Html::parse_document(&html_content);
    let result_selector = Selector::parse(".result").unwrap();
    let title_selector = Selector::parse(".result__title > a.result__url").unwrap();
    let snippet_selector = Selector::parse(".result__snippet").unwrap();

    let mut results = Vec::new();
    for element in document.select(&result_selector).take(5) {
        if let Some(t_el) = element.select(&title_selector).next() {
            let raw_url = t_el.value().attr("href").unwrap_or("").to_string();
            let title = t_el.text().collect::<Vec<_>>().join(" ");
            let snippet = element.select(&snippet_selector).next()
                .map(|s| s.text().collect::<Vec<_>>().join(" "))
                .unwrap_or_default();

            let clean_url = if raw_url.contains("uddg=") {
                raw_url.split("uddg=").nth(1)
                    .and_then(|s| s.split('&').next())
                    .map(|s| urlencoding::decode(s).unwrap_or_default().to_string())
                    .unwrap_or(raw_url)
            } else {
                raw_url
            };

            if !clean_url.is_empty() {
                results.push(SearchResult {
                    title: title.trim().to_string(),
                    url: clean_url,
                    snippet: snippet.trim().to_string(),
                    source_type: "web".to_string(),
                    fetched_content: None,
                    http_status: 0,
                });
            }
        }
    }
    Ok(results)
}

/// URL'den içerik çeker
async fn fetch_content(client: &Client, url: &str, source_type: &str) -> Result<(u16, String)> {
    // PDF — binary, snippet ile yetiniyoruz (ayrıca PDF URL'ini kaydediyoruz)
    if source_type == "pdf" || url.ends_with(".pdf") {
        return Ok((200, format!(
            "[PDF KAYNAĞI] Bu PDF projende benzer içerik barındırıyor olabilir. Link: {}",
            url
        )));
    }

    // GitHub — README çek (master veya main)
    let fetch_url = if url.contains("github.com") && !url.contains("raw.githubusercontent.com") {
        let parts: Vec<&str> = url.split("github.com/").collect();
        if parts.len() == 2 {
            let repo_path = parts[1].split('/').take(2).collect::<Vec<_>>().join("/");
            format!("https://raw.githubusercontent.com/{}/master/README.md", repo_path)
        } else {
            url.to_string()
        }
    } else {
        url.to_string()
    };

    let response = client
        .get(&fetch_url)
        .timeout(Duration::from_secs(8))
        .send()
        .await?;

    let status = response.status().as_u16();

    match status {
        200 => {
            let text = response.text().await?;
            let clean = clean_html(&text);
            Ok((200, clean.chars().take(3000).collect()))
        }
        404 if fetch_url.contains("/master/README.md") => {
            // main branch dene
            let main_url = fetch_url.replace("/master/README.md", "/main/README.md");
            if let Ok(resp2) = client.get(&main_url).timeout(Duration::from_secs(6)).send().await {
                if resp2.status().is_success() {
                    let t = resp2.text().await?;
                    return Ok((200, clean_html(&t).chars().take(3000).collect()));
                }
            }
            Ok((404, String::new()))
        }
        code => {
            warn!("HTTP {}: {}", code, url);
            Ok((code, String::new()))
        }
    }
}

/// HTML tag'lerini temizler
fn clean_html(html: &str) -> String {
    let tag_re = regex::Regex::new(r"<[^>]+>").unwrap();
    let ws_re = regex::Regex::new(r"\s+").unwrap();
    let no_tags = tag_re.replace_all(html, " ");
    ws_re.replace_all(&no_tags, " ").trim().to_string()
}

/// URL kaynak tipini sınıflandırır
fn classify_source(url: &str) -> String {
    if url.ends_with(".pdf") { return "pdf".to_string(); }
    if url.contains("github.com") || url.contains("githubusercontent.com") { return "github".to_string(); }
    if url.contains("arxiv.org") || url.contains("semanticscholar.org") || url.contains("researchgate.net") { return "academic".to_string(); }
    "web".to_string()
}
