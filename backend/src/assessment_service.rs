use anyhow::Result;

use crate::{
    assessment, assessment_store,
    database::Database,
    models::{CategoryFitAnalysis, Project, ProjectSimilarityAnalysis, ProjectSimilarityMatch},
};

/// Shared vocabulary over combined vocabulary. Two independently written
/// reports in the same language and subject area measure roughly 0.25–0.34 on
/// this scale, while a near-duplicate measures 0.75 and above, so the two
/// populations separate cleanly here.
const JACCARD_REVIEW_THRESHOLD: f64 = 0.45;

/// Shared vocabulary over the smaller report's own vocabulary. This is the
/// measure that catches a copied section buried inside a padded document, where
/// it approaches 1.0 while Jaccard collapses. Its baseline for unrelated
/// reports sits near 0.55, so it needs a far higher bar than Jaccard: judging
/// both against 0.45 marked every submission for review.
const CONTAINMENT_REVIEW_THRESHOLD: f64 = 0.80;

const MAX_SIMILARITY_MATCHES: usize = 10;

/// A match earns human attention when either measure clears its own bar.
fn needs_review(match_: &ProjectSimilarityMatch) -> bool {
    match_.jaccard >= JACCARD_REVIEW_THRESHOLD || match_.containment >= CONTAINMENT_REVIEW_THRESHOLD
}

pub async fn run_category_fit(
    database: &Database,
    project: &Project,
) -> Result<CategoryFitAnalysis> {
    let document = database
        .get_project_document(project.id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("A parsed project report is required"))?;
    let categories = database.list_categories().await?;
    let result = assessment::analyze_category_fit(&document, &project.category, &categories)
        .ok_or_else(|| anyhow::anyhow!("No KPI category template is configured"))?;
    let analysis = CategoryFitAnalysis {
        project_id: project.id,
        source_file_version: database.latest_project_file_version(project.id).await?,
        current_category_score: result.current_category_score,
        recommended_category: result.recommended_category,
        recommended_category_score: result.recommended_category_score,
        matched_terms: result.matched_terms,
        requires_review: result.requires_review,
        analyzed_at: String::new(),
    };
    assessment_store::save_category_fit(&database.pool, &analysis).await
}

pub async fn run_similarity(
    database: &Database,
    project: &Project,
) -> Result<ProjectSimilarityAnalysis> {
    let document = database
        .get_project_document(project.id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("A parsed project report is required"))?;
    let comparable =
        assessment_store::comparable_projects(&database.pool, project.competition_id, project.id)
            .await?;
    // Tokenised once here rather than inside every pairwise comparison; with a
    // few hundred submissions the repeated tokenisation dominated the request.
    let document_terms = assessment::similarity_tokens(&document);
    let mut matches = comparable
        .iter()
        .map(|other| {
            let result = assessment::compare_similarity_tokens(
                &document_terms,
                &assessment::similarity_tokens(&other.document),
            );
            ProjectSimilarityMatch {
                project_id: other.id,
                project_reference: format!("PRJ-{:06}", other.id),
                category: other.category.clone(),
                similarity: result.similarity,
                jaccard: result.jaccard,
                containment: result.containment,
                matched_terms: result.matched_terms.into_iter().take(25).collect(),
            }
        })
        .collect::<Vec<_>>();
    matches.sort_by(|left, right| right.similarity.total_cmp(&left.similarity));
    matches.truncate(MAX_SIMILARITY_MATCHES);
    let highest_similarity = matches.first().map(|item| item.similarity).unwrap_or(0.0);
    let analysis = ProjectSimilarityAnalysis {
        project_id: project.id,
        source_file_version: database.latest_project_file_version(project.id).await?,
        highest_similarity,
        requires_review: matches.iter().any(needs_review),
        matches: matches.clone(),
        analyzed_at: String::new(),
    };
    let saved = assessment_store::save_similarity(&database.pool, &analysis).await?;
    propagate_matches(database, project, &matches).await?;
    Ok(saved)
}

/// Similarity is symmetric, but each project's record is only written when that
/// project is analysed. Without this the first of a duplicated pair keeps a
/// clean record for ever while the second shows the match, so a jury opening
/// the earlier submission sees no warning at all.
async fn propagate_matches(
    database: &Database,
    project: &Project,
    matches: &[ProjectSimilarityMatch],
) -> Result<()> {
    for entry in matches {
        let Some(mut other) =
            assessment_store::get_similarity(&database.pool, entry.project_id).await?
        else {
            continue;
        };
        let mirrored = ProjectSimilarityMatch {
            project_id: project.id,
            project_reference: format!("PRJ-{:06}", project.id),
            category: project.category.clone(),
            similarity: entry.similarity,
            jaccard: entry.jaccard,
            containment: entry.containment,
            matched_terms: entry.matched_terms.clone(),
        };
        other.matches.retain(|item| item.project_id != project.id);
        other.matches.push(mirrored);
        other
            .matches
            .sort_by(|left, right| right.similarity.total_cmp(&left.similarity));
        other.matches.truncate(MAX_SIMILARITY_MATCHES);
        other.highest_similarity = other
            .matches
            .first()
            .map(|item| item.similarity)
            .unwrap_or(0.0);
        other.requires_review = other.matches.iter().any(needs_review);
        assessment_store::save_similarity(&database.pool, &other).await?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "assessment_service_tests.rs"]
mod tests;
