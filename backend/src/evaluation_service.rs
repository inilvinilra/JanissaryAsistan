//! Orchestration for MVP gate 06.
//!
//! Collects what the earlier gates already established about a submission,
//! produces the deterministic criterion evaluation, optionally lets a language
//! model refine the wording, and persists the result through the same record
//! the judge dashboard, the jury summary and the applicant portal already read.

use anyhow::Result;

use crate::{
    assessment_store,
    database::Database,
    evaluation::{self, EvaluationContext},
    evaluation_llm,
    models::{AiEvaluation, Project, UpsertAiEvaluation},
    template,
};

/// Everything the earlier gates concluded, in the shape gate 06 consumes.
async fn gather_context(database: &Database, project: &Project) -> Result<EvaluationContext> {
    let mut context = EvaluationContext::default();

    if let Some(fit) = assessment_store::get_category_fit(&database.pool, project.id).await?
        && fit.requires_review
    {
        context.category_mismatch = Some(fit.recommended_category);
    }

    if let Some(similarity) = assessment_store::get_similarity(&database.pool, project.id).await?
        && similarity.requires_review
        && let Some(closest) = similarity.matches.first()
    {
        context.high_similarity_with = Some(closest.project_reference.clone());
    }

    if let (Some(report_template), Some(document)) = (
        database.get_report_template(project.competition_id).await?,
        database.get_project_document(project.id).await?,
    ) {
        let compliance = template::evaluate(project.id, &report_template, &document);
        for finding in compliance
            .sections
            .iter()
            .filter(|finding| finding.required)
        {
            match finding.status.as_str() {
                "missing" => context.missing_sections.push(finding.title.clone()),
                "thin" => context.thin_sections.push(finding.title.clone()),
                _ => {}
            }
        }
    }

    Ok(context)
}

/// Runs gate 06 for one project and stores the result.
///
/// The deterministic evaluation is produced first and is what gets stored if
/// anything downstream fails, so a missing key, an exhausted quota or a network
/// timeout costs explanation quality rather than the gate itself.
pub async fn run_criterion_evaluation(
    database: &Database,
    project: &Project,
) -> Result<AiEvaluation> {
    let document = database
        .get_project_document(project.id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("A parsed project report is required"))?;
    let template = database
        .list_categories()
        .await?
        .into_iter()
        .find(|item| item.category == project.category)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No KPI template is configured for the \"{}\" category",
                project.category
            )
        })?;

    let context = gather_context(database, project).await?;
    let mut evaluation = evaluation::heuristic_evaluation(&document, &template.kpis, &context);
    evaluation.source_file_version = database.latest_project_file_version(project.id).await?;

    match evaluation_llm::refine(&document, &template.kpis, &evaluation).await {
        Ok(Some(refined)) => evaluation = refined,
        Ok(None) => {}
        Err(error) => {
            tracing::warn!(
                %error,
                project_id = project.id,
                "language-model refinement failed; storing the deterministic evaluation"
            );
        }
    }

    validate(&evaluation)?;
    database.upsert_ai_evaluation(project.id, &evaluation).await
}

/// The same bounds the public `PUT /projects/{id}/ai-evaluation` route enforces.
/// A stored evaluation that violates them would fail the readiness gate later
/// with no indication of where it came from, so it is caught at the source.
fn validate(evaluation: &UpsertAiEvaluation) -> Result<()> {
    if evaluation.model_version.trim().is_empty() {
        anyhow::bail!("The evaluation is missing a model version");
    }
    if !(0.0..=100.0).contains(&evaluation.total_score) || !evaluation.total_score.is_finite() {
        anyhow::bail!("Total score {} is out of range", evaluation.total_score);
    }
    if !(0.0..=1.0).contains(&evaluation.confidence) {
        anyhow::bail!("Confidence {} is out of range", evaluation.confidence);
    }
    for score in &evaluation.kpi_scores {
        if score.name.trim().is_empty()
            || !(0.0..=100.0).contains(&score.score)
            || !(0.0..=1.0).contains(&score.confidence)
        {
            anyhow::bail!("Criterion \"{}\" carries an invalid score", score.name);
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "evaluation_service_tests.rs"]
mod tests;
