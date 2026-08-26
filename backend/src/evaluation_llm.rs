//! Optional language-model refinement for gate 06.
//!
//! The deterministic evaluation already produces scores and quoted evidence.
//! What it cannot do is judge whether a section actually argues what it claims
//! to — the semantic half of the brief's "başlık ve içerik kontrolü". That is
//! what this layer adds.
//!
//! It is deliberately a *refinement* and never a replacement. The model is
//! given the report and the deterministic result, and everything it returns is
//! bounded before use:
//!
//! - Criterion names come from the competition's template, not from the model;
//!   an unrecognised name is discarded rather than creating a criterion.
//! - Every quotation is checked against the report and dropped if absent, so a
//!   fabricated citation cannot reach a judge.
//! - Scores and confidences are clamped, and a criterion whose evidence did not
//!   survive verification loses the confidence that evidence would have earned.
//!
//! When `MISTRAL_API_KEY` is unset the module reports "not configured" and the
//! caller keeps the deterministic evaluation.

use std::collections::BTreeMap;
use std::time::Duration;

use anyhow::Result;
use serde::Deserialize;

use crate::evaluation;
use crate::models::{AiKpiEvaluation, Document, KpiTemplate, UpsertAiEvaluation};

const DEFAULT_MODEL: &str = "mistral-small-latest";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
/// Reports run to tens of thousands of characters; the prompt keeps the opening
/// portion, which is where the abstract, problem and method sections live.
const MAX_REPORT_CHARS: usize = 12_000;

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct ModelEvaluation {
    #[serde(default)]
    criteria: Vec<ModelCriterion>,
    #[serde(default)]
    strengths: Vec<String>,
    #[serde(default)]
    weaknesses: Vec<String>,
    #[serde(default)]
    missing_information: Vec<String>,
    #[serde(default)]
    risks: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ModelCriterion {
    name: String,
    #[serde(default)]
    score: f64,
    #[serde(default)]
    reason: String,
    #[serde(default)]
    evidence: Vec<String>,
    #[serde(default)]
    confidence: f64,
}

fn configured() -> Option<(String, String)> {
    let key = std::env::var("MISTRAL_API_KEY")
        .ok()
        .filter(|value| !value.trim().is_empty())?;
    let model = std::env::var("MISTRAL_MODEL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_MODEL.to_string());
    Some((key, model))
}

fn prompt(document: &Document, kpis: &[KpiTemplate], baseline: &UpsertAiEvaluation) -> String {
    let criteria = kpis
        .iter()
        .map(|kpi| {
            format!(
                "- {} (weight {:.0}%): {}",
                kpi.name, kpi.weight, kpi.description
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let baseline_scores = baseline
        .kpi_scores
        .iter()
        .map(|score| format!("- {}: {:.0}/100", score.name, score.score))
        .collect::<Vec<_>>()
        .join("\n");
    let report: String = document.raw_text.chars().take(MAX_REPORT_CHARS).collect();

    format!(
        "You assess competition project reports for a jury. You do not decide the outcome; a human judge does. \
Assess only what the report states.\n\n\
Evaluate the report against each criterion. For every criterion return:\n\
- score: 0-100\n\
- reason: one or two sentences, in the language of the report\n\
- evidence: sentences copied EXACTLY from the report, word for word. Never write a sentence that is not in the report. \
If the report says nothing about a criterion, return an empty evidence list and say so in the reason.\n\
- confidence: 0.0-1.0, reflecting how directly the report supports your score\n\n\
Also return strengths, weaknesses, missing_information and risks as lists of short sentences addressed to the applicant, \
in the language of the report.\n\n\
Reply with JSON only, in this shape:\n\
{{\"criteria\":[{{\"name\":\"...\",\"score\":0,\"reason\":\"...\",\"evidence\":[\"...\"],\"confidence\":0.0}}],\
\"strengths\":[\"...\"],\"weaknesses\":[\"...\"],\"missing_information\":[\"...\"],\"risks\":[\"...\"]}}\n\n\
Use exactly these criterion names:\n{criteria}\n\n\
A rule-based pass scored the report as follows. Correct it where the text disagrees:\n{baseline_scores}\n\n\
REPORT:\n{report}"
    )
}

/// Models often wrap JSON in prose or a code fence; take the outermost object.
fn extract_json(content: &str) -> Option<&str> {
    let start = content.find('{')?;
    let end = content.rfind('}')?;
    (end > start).then(|| &content[start..=end])
}

async fn request(key: &str, model: &str, prompt: String) -> Result<ModelEvaluation> {
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()?;
    let response = client
        .post("https://api.mistral.ai/v1/chat/completions")
        .bearer_auth(key)
        .json(&serde_json::json!({
            "model": model,
            "messages": [{ "role": "user", "content": prompt }],
            "temperature": 0.2,
            "response_format": { "type": "json_object" },
        }))
        .send()
        .await?;
    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("Mistral returned {status}");
    }
    let body: ChatResponse = response.json().await?;
    let content = body
        .choices
        .first()
        .map(|choice| choice.message.content.as_str())
        .ok_or_else(|| anyhow::anyhow!("Mistral returned no choices"))?;
    let json =
        extract_json(content).ok_or_else(|| anyhow::anyhow!("Mistral returned no JSON object"))?;
    Ok(serde_json::from_str(json)?)
}

/// Folds the model's output onto the deterministic evaluation.
///
/// The baseline decides which criteria exist and remains the fallback for any
/// the model omitted or named differently, so the stored evaluation always
/// covers the competition's full criterion set.
fn merge(
    document: &Document,
    baseline: &UpsertAiEvaluation,
    model: ModelEvaluation,
    model_version: String,
    kpis: &[KpiTemplate],
) -> UpsertAiEvaluation {
    let mut returned: BTreeMap<String, ModelCriterion> = model
        .criteria
        .into_iter()
        .map(|criterion| (criterion.name.trim().to_lowercase(), criterion))
        .collect();

    let kpi_scores: Vec<AiKpiEvaluation> = baseline
        .kpi_scores
        .iter()
        .map(|fallback| {
            let Some(criterion) = returned.remove(&fallback.name.trim().to_lowercase()) else {
                return fallback.clone();
            };
            let (evidence, discarded) =
                evaluation::ground_evidence(document, criterion.evidence.clone());
            // An unverifiable quotation is the clearest signal that the model
            // is reasoning past the text, so the score keeps its reasoning but
            // not the confidence the evidence would have justified.
            let penalty = if discarded > 0 { 0.35 } else { 1.0 };
            let confidence = if evidence.is_empty() {
                fallback.confidence.min(0.4)
            } else {
                (criterion.confidence.clamp(0.0, 1.0) * penalty).clamp(0.0, 0.95)
            };
            AiKpiEvaluation {
                name: fallback.name.clone(),
                score: if criterion.score.is_finite() {
                    criterion.score.clamp(0.0, 100.0)
                } else {
                    fallback.score
                },
                reason: if criterion.reason.trim().is_empty() {
                    fallback.reason.clone()
                } else {
                    criterion.reason.trim().to_string()
                },
                evidence,
                confidence,
            }
        })
        .collect();

    let stale_evidence_risk = evaluation::unevidenced_criteria_risk(&baseline.kpi_scores);

    let total_weight: f64 = kpis.iter().map(|kpi| kpi.weight).sum();
    let total_score = if total_weight > 0.0 {
        kpi_scores
            .iter()
            .zip(kpis.iter())
            .map(|(score, kpi)| score.score * kpi.weight)
            .sum::<f64>()
            / total_weight
    } else {
        baseline.total_score
    };
    let confidence = if kpi_scores.is_empty() {
        baseline.confidence
    } else {
        kpi_scores.iter().map(|score| score.confidence).sum::<f64>() / kpi_scores.len() as f64
    };

    fn prefer(model: Vec<String>, fallback: &[String]) -> Vec<String> {
        let cleaned: Vec<String> = model
            .into_iter()
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .take(8)
            .collect();
        if cleaned.is_empty() {
            fallback.to_vec()
        } else {
            cleaned
        }
    }

    // Risks raised by the earlier gates are factual findings, not opinions, so
    // the model's list is appended to them rather than replacing them. The one
    // exception is the count of unevidenced criteria: it describes the scores
    // being stored, and the model normally finds quotations the deterministic
    // pass missed, so the stale count is dropped and recomputed from the merge.
    let risks: Vec<String> = baseline
        .risks
        .iter()
        .filter(|risk| Some(risk.as_str()) != stale_evidence_risk.as_deref())
        .cloned()
        .chain(evaluation::unevidenced_criteria_risk(&kpi_scores))
        .chain(
            model
                .risks
                .into_iter()
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty()),
        )
        .take(10)
        .collect();

    UpsertAiEvaluation {
        model_version,
        total_score,
        confidence,
        source_file_version: baseline.source_file_version,
        kpi_scores,
        strengths: prefer(model.strengths, &baseline.strengths),
        weaknesses: prefer(model.weaknesses, &baseline.weaknesses),
        missing_information: prefer(model.missing_information, &baseline.missing_information),
        risks,
        sources: baseline.sources.clone(),
        similar_projects: baseline.similar_projects.clone(),
    }
}

/// `Ok(None)` means no model is configured, which is a normal state rather than
/// a failure: the caller keeps the deterministic evaluation.
pub async fn refine(
    document: &Document,
    kpis: &[KpiTemplate],
    baseline: &UpsertAiEvaluation,
) -> Result<Option<UpsertAiEvaluation>> {
    let Some((key, model)) = configured() else {
        return Ok(None);
    };
    let response = request(&key, &model, prompt(document, kpis, baseline)).await?;
    Ok(Some(merge(
        document,
        baseline,
        response,
        format!("mistral:{model}"),
        kpis,
    )))
}

#[cfg(test)]
#[path = "evaluation_llm_tests.rs"]
mod tests;
