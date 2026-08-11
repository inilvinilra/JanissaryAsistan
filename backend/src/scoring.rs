use crate::models::{Document, KpiScore, KpiTemplate};
use anyhow::Result;

// Swap in a real LLM-backed variant once an API key exists — everything that
// calls score_project only depends on this enum, not on Mock directly.
pub enum Scorer {
    Mock,
}

pub async fn score_project(scorer: &Scorer, document: &Document, kpis: &[KpiTemplate]) -> Result<Vec<KpiScore>> {
    match scorer {
        Scorer::Mock => Ok(mock_score(document, kpis)),
    }
}

// Placeholder until a real LLM call replaces it — these are document-quality
// signals, not actual judgment of the project's merit.
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
            KpiTemplate { name: "A".into(), weight: 50.0, description: String::new() },
            KpiTemplate { name: "B".into(), weight: 50.0, description: String::new() },
        ];

        let scores = score_project(&Scorer::Mock, &document, &kpis).await.unwrap();

        assert_eq!(scores.len(), 2);
        assert!(scores.iter().all(|s| s.score >= 0.0 && s.score <= 100.0));
    }
}
