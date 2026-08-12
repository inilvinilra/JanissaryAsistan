use crate::models::{Document, KpiScore, KpiTemplate};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub enum Scorer {
    Mock,
    Http {
        endpoint: String,
        bearer_token: Option<String>,
    },
}

#[derive(Serialize)]
struct ExternalScoringRequest<'a> {
    document: &'a Document,
    kpis: &'a [KpiTemplate],
}

#[derive(Deserialize)]
struct ExternalScoringResponse {
    kpi_scores: Vec<KpiScore>,
}

pub fn configured_scorer() -> Scorer {
    match std::env::var("AI_SCORING_URL") {
        Ok(endpoint) if !endpoint.trim().is_empty() => Scorer::Http {
            endpoint,
            bearer_token: std::env::var("AI_SCORING_TOKEN")
                .ok()
                .filter(|value| !value.trim().is_empty()),
        },
        _ => Scorer::Mock,
    }
}

pub async fn score_project(
    scorer: &Scorer,
    document: &Document,
    kpis: &[KpiTemplate],
) -> Result<Vec<KpiScore>> {
    match scorer {
        Scorer::Mock => Ok(mock_score(document, kpis)),
        Scorer::Http {
            endpoint,
            bearer_token,
        } => score_external(endpoint, bearer_token.as_deref(), document, kpis).await,
    }
}

async fn score_external(
    endpoint: &str,
    bearer_token: Option<&str>,
    document: &Document,
    kpis: &[KpiTemplate],
) -> Result<Vec<KpiScore>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;
    let mut request = client
        .post(endpoint)
        .json(&ExternalScoringRequest { document, kpis });
    if let Some(token) = bearer_token {
        request = request.bearer_auth(token);
    }
    let response = request.send().await?.error_for_status()?;
    let body: ExternalScoringResponse = response.json().await?;
    validate_external_scores(body.kpi_scores, kpis)
}

fn validate_external_scores(scores: Vec<KpiScore>, kpis: &[KpiTemplate]) -> Result<Vec<KpiScore>> {
    if scores.len() != kpis.len() {
        anyhow::bail!("AI service returned an incomplete KPI score set");
    }
    let expected = kpis
        .iter()
        .map(|kpi| kpi.name.as_str())
        .collect::<std::collections::HashSet<_>>();
    let returned = scores
        .iter()
        .map(|score| score.name.as_str())
        .collect::<std::collections::HashSet<_>>();
    if expected != returned || returned.len() != scores.len() {
        anyhow::bail!("AI service returned unexpected or duplicate KPI names");
    }
    if scores
        .iter()
        .any(|score| !score.score.is_finite() || !(0.0..=100.0).contains(&score.score))
    {
        anyhow::bail!("AI service returned an invalid KPI score");
    }
    Ok(scores)
}

fn mock_score(document: &Document, kpis: &[KpiTemplate]) -> Vec<KpiScore> {
    let mut base = 55.0;
    if document.has_abstract {
        base += 8.0;
    }
    if document.has_methodology {
        base += 8.0;
    }
    if document.has_conclusion {
        base += 7.0;
    }
    if document.has_references {
        base += 7.0;
    }
    if document.word_count > 300 {
        base += 5.0;
    }

    kpis.iter()
        .enumerate()
        .map(|(i, kpi)| {
            let variation = (i as f64 * 3.7) % 10.0 - 5.0;
            KpiScore {
                name: kpi.name.clone(),
                score: (base + variation).clamp(0.0, 100.0),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FileType;

    #[tokio::test]
    async fn mock_score_returns_one_score_per_kpi() {
        let document = Document {
            filename: "test.md".into(),
            file_type: FileType::Markdown,
            raw_text: String::new(),
            word_count: 400,
            headings: vec![],
            keywords: vec![],
            references: vec![],
            has_references: true,
            has_abstract: true,
            has_conclusion: true,
            has_methodology: true,
            language: crate::models::Language::English,
            sections: vec![],
        };
        let kpis = vec![
            KpiTemplate {
                name: "A".into(),
                weight: 50.0,
                description: String::new(),
            },
            KpiTemplate {
                name: "B".into(),
                weight: 50.0,
                description: String::new(),
            },
        ];

        let scores = score_project(&Scorer::Mock, &document, &kpis)
            .await
            .unwrap();

        assert_eq!(scores.len(), 2);
        assert!(scores.iter().all(|s| s.score >= 0.0 && s.score <= 100.0));
    }

    #[test]
    fn external_scores_require_the_expected_unique_kpis() {
        let kpis = vec![
            KpiTemplate {
                name: "Innovation".into(),
                weight: 50.0,
                description: String::new(),
            },
            KpiTemplate {
                name: "Impact".into(),
                weight: 50.0,
                description: String::new(),
            },
        ];
        let valid = vec![
            KpiScore {
                name: "Innovation".into(),
                score: 85.0,
            },
            KpiScore {
                name: "Impact".into(),
                score: 70.0,
            },
        ];
        assert!(validate_external_scores(valid, &kpis).is_ok());
        let invalid = vec![
            KpiScore {
                name: "Innovation".into(),
                score: 85.0,
            },
            KpiScore {
                name: "Innovation".into(),
                score: 70.0,
            },
        ];
        assert!(validate_external_scores(invalid, &kpis).is_err());
    }
}
