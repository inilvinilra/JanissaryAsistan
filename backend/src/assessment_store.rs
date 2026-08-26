use anyhow::Result;
use sqlx::{PgPool, Row, postgres::PgRow};

use crate::models::{CategoryFitAnalysis, Document, ProjectSimilarityAnalysis};

#[derive(Debug, Clone)]
pub struct ComparableProject {
    pub id: i32,
    pub category: String,
    pub document: Document,
}

pub async fn get_category_fit(
    pool: &PgPool,
    project_id: i32,
) -> Result<Option<CategoryFitAnalysis>> {
    let row = sqlx::query(
        "SELECT project_id, source_file_version, current_category_score, recommended_category,
                recommended_category_score, matched_terms, requires_review, analyzed_at
         FROM project_category_fit_analyses WHERE project_id = $1",
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await?;
    row.map(|value| category_fit_from_row(&value)).transpose()
}

pub async fn save_category_fit(
    pool: &PgPool,
    analysis: &CategoryFitAnalysis,
) -> Result<CategoryFitAnalysis> {
    let row = sqlx::query(
        "INSERT INTO project_category_fit_analyses
            (project_id, source_file_version, current_category_score, recommended_category,
             recommended_category_score, matched_terms, requires_review)
         VALUES ($1,$2,$3,$4,$5,$6,$7)
         ON CONFLICT (project_id) DO UPDATE SET
            source_file_version = EXCLUDED.source_file_version,
            current_category_score = EXCLUDED.current_category_score,
            recommended_category = EXCLUDED.recommended_category,
            recommended_category_score = EXCLUDED.recommended_category_score,
            matched_terms = EXCLUDED.matched_terms,
            requires_review = EXCLUDED.requires_review,
            analyzed_at = NOW()
         RETURNING project_id, source_file_version, current_category_score, recommended_category,
                   recommended_category_score, matched_terms, requires_review, analyzed_at",
    )
    .bind(analysis.project_id)
    .bind(analysis.source_file_version)
    .bind(analysis.current_category_score)
    .bind(&analysis.recommended_category)
    .bind(analysis.recommended_category_score)
    .bind(serde_json::to_value(&analysis.matched_terms)?)
    .bind(analysis.requires_review)
    .fetch_one(pool)
    .await?;
    category_fit_from_row(&row)
}

pub async fn get_similarity(
    pool: &PgPool,
    project_id: i32,
) -> Result<Option<ProjectSimilarityAnalysis>> {
    let row = sqlx::query(
        "SELECT project_id, source_file_version, highest_similarity, requires_review, matches, analyzed_at
         FROM project_similarity_analyses WHERE project_id = $1",
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await?;
    row.map(|value| similarity_from_row(&value)).transpose()
}

pub async fn save_similarity(
    pool: &PgPool,
    analysis: &ProjectSimilarityAnalysis,
) -> Result<ProjectSimilarityAnalysis> {
    let row = sqlx::query(
        "INSERT INTO project_similarity_analyses
            (project_id, source_file_version, highest_similarity, requires_review, matches)
         VALUES ($1,$2,$3,$4,$5)
         ON CONFLICT (project_id) DO UPDATE SET
            source_file_version = EXCLUDED.source_file_version,
            highest_similarity = EXCLUDED.highest_similarity,
            requires_review = EXCLUDED.requires_review,
            matches = EXCLUDED.matches,
            analyzed_at = NOW()
         RETURNING project_id, source_file_version, highest_similarity, requires_review, matches, analyzed_at",
    )
    .bind(analysis.project_id)
    .bind(analysis.source_file_version)
    .bind(analysis.highest_similarity)
    .bind(analysis.requires_review)
    .bind(serde_json::to_value(&analysis.matches)?)
    .fetch_one(pool)
    .await?;
    similarity_from_row(&row)
}

pub async fn comparable_projects(
    pool: &PgPool,
    competition_id: i32,
    excluded_project_id: i32,
) -> Result<Vec<ComparableProject>> {
    let rows = sqlx::query(
        "SELECT id, category, document FROM projects
         WHERE competition_id = $1 AND id <> $2 AND document IS NOT NULL",
    )
    .bind(competition_id)
    .bind(excluded_project_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            let value: serde_json::Value = row.get("document");
            Ok(ComparableProject {
                id: row.get("id"),
                category: row.get("category"),
                document: serde_json::from_value(value)?,
            })
        })
        .collect()
}

fn json_array<T: serde::de::DeserializeOwned>(row: &PgRow, key: &str) -> Result<T> {
    Ok(serde_json::from_value(row.get(key))?)
}

fn timestamp(row: &PgRow, key: &str) -> String {
    row.get::<chrono::DateTime<chrono::Utc>, _>(key)
        .to_rfc3339()
}

fn category_fit_from_row(row: &PgRow) -> Result<CategoryFitAnalysis> {
    Ok(CategoryFitAnalysis {
        project_id: row.get("project_id"),
        source_file_version: row.get("source_file_version"),
        current_category_score: row.get("current_category_score"),
        recommended_category: row.get("recommended_category"),
        recommended_category_score: row.get("recommended_category_score"),
        matched_terms: json_array(row, "matched_terms")?,
        requires_review: row.get("requires_review"),
        analyzed_at: timestamp(row, "analyzed_at"),
    })
}

fn similarity_from_row(row: &PgRow) -> Result<ProjectSimilarityAnalysis> {
    Ok(ProjectSimilarityAnalysis {
        project_id: row.get("project_id"),
        source_file_version: row.get("source_file_version"),
        highest_similarity: row.get("highest_similarity"),
        requires_review: row.get("requires_review"),
        matches: json_array(row, "matches")?,
        analyzed_at: timestamp(row, "analyzed_at"),
    })
}

#[cfg(test)]
#[path = "assessment_store_tests.rs"]
mod tests;
