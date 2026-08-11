use crate::models::SearchResult;
use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

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

pub async fn search_related_sources(
    keywords: &[String],
    api_key: &str,
) -> Result<Vec<SearchResult>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("JuryAssistant-Backend/0.1")
        .build()?;

    let mut all_results: Vec<SearchResult> = Vec::new();

    // Only the top 5 keywords, to save on API quota
    let search_terms: Vec<String> = keywords.iter().take(5).cloned().collect();

    for keyword in &search_terms {
        match search_brave(&client, keyword, api_key).await {
            Ok(results) => {
                all_results.extend(results);
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Err(e) => {
                eprintln!("Brave Search error ('{}'): {}", keyword, e);
            }
        }
    }

    all_results.dedup_by(|a, b| a.url == b.url);

    let mut fetched_results = Vec::new();
    for mut result in all_results.into_iter().take(10) {
        match fetch_content(&client, &result.url).await {
            Ok((status, content)) => {
                result.http_status = status;
                if status == 200 {
                    result.fetched_content = Some(content);
                    result.source_type = classify_source(&result.url);
                    fetched_results.push(result);
                }
            }
            Err(e) => {
                eprintln!("Fetch error ({}): {} - skipping", result.url, e);
            }
        }
    }

    Ok(fetched_results)
}

async fn search_brave(client: &Client, query: &str, api_key: &str) -> Result<Vec<SearchResult>> {
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
            tokio::time::sleep(Duration::from_secs(5)).await;
            Err(anyhow::anyhow!("Rate limited (429)"))
        }
        403 => Err(anyhow::anyhow!("Forbidden (403) - check the API key")),
        _ => Err(anyhow::anyhow!("Unexpected HTTP status: {}", status)),
    }
}

async fn fetch_content(client: &Client, url: &str) -> Result<(u16, String)> {
    if url.ends_with(".pdf") {
        return Ok((200, format!("[PDF source: {}]", url)));
    }

    let response = client
        .get(url)
        .timeout(Duration::from_secs(10))
        .header("User-Agent", "Mozilla/5.0 JuryAssistant-Backend/0.1")
        .send()
        .await?;

    let status = response.status().as_u16();

    match status {
        200 => {
            let text = response.text().await?;
            let clean = clean_html(&text);
            Ok((200, clean[..clean.len().min(2000)].to_string()))
        }
        429 => {
            tokio::time::sleep(Duration::from_secs(3)).await;
            Ok((429, String::new()))
        }
        code => Ok((code, String::new())),
    }
}

fn clean_html(html: &str) -> String {
    let tag_re = regex::Regex::new(r"<[^>]+>").unwrap();
    let whitespace_re = regex::Regex::new(r"\s+").unwrap();
    let no_tags = tag_re.replace_all(html, " ");
    whitespace_re.replace_all(&no_tags, " ").trim().to_string()
}

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

#[cfg(test)]
mod tests {
    use super::*;

    // Keep parser/scraper unit tests deterministic. Network availability is an
    // environment concern and should be covered by a separate integration test.
    #[test]
    fn clean_html_strips_tags_and_normalizes_whitespace() {
        let content = clean_html("<html><body>  Hello <strong>world</strong> </body></html>");
        assert_eq!(content, "Hello world");
        assert!(!content.contains('<'));
    }

    #[test]
    fn classify_source_detects_supported_source_types() {
        assert_eq!(classify_source("https://arxiv.org/abs/1234"), "academic");
        assert_eq!(classify_source("https://github.com/example/project"), "github");
        assert_eq!(classify_source("https://example.com/report.pdf"), "pdf");
        assert_eq!(classify_source("https://docs.example.com/guide"), "documentation");
    }
}
