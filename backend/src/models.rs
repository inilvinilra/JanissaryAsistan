use serde::{Deserialize, Serialize};

/// A parsed document (a project write-up)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub filename: String,
    pub file_type: FileType,
    pub raw_text: String,
    pub word_count: usize,
    pub headings: Vec<String>,
    pub keywords: Vec<String>,
    pub references: Vec<String>,
    pub has_references: bool,
    pub has_abstract: bool,
    pub has_conclusion: bool,
    pub has_methodology: bool,
    pub language: Language,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileType {
    Pdf,
    Txt,
    Markdown,
    Docx,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Language {
    Turkish,
    English,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub title: String,
    pub content: String,
    pub word_count: usize,
}

/// A single web search result (from Brave Search)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub source_type: String, // "academic", "github", "documentation", "web"
    pub fetched_content: Option<String>,
    pub http_status: u16,
}

// --- Jury Assistant's own domain model (project / KPI / ranking) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiScore {
    pub name: String,
    pub score: f64, // 0-100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i32,
    pub name: String,
    pub category: String,
    pub kpi_scores: Vec<KpiScore>,
    pub ai_score: f64,
    // Set when a juror drags this project to a new position; None until then,
    // in which case ranking falls back to ai_score.
    pub manual_rank: Option<i32>,
    pub notes: String,
    pub status: ProjectStatus,
    pub has_file: bool,
    pub review_completed: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub project_id: i32,
    pub institution: String,
    pub keywords: Vec<String>,
    pub github_url: Option<String>,
    pub demo_url: Option<String>,
    pub prototype_description: String,
    pub team_name: String,
    pub team_members: Vec<String>,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectMetadata {
    pub institution: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub github_url: Option<String>,
    pub demo_url: Option<String>,
    pub prototype_description: Option<String>,
    pub team_name: Option<String>,
    pub team_members: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFile {
    pub id: i32,
    pub project_id: i32,
    pub version: i32,
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub file_path: String,
    pub uploaded_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    New,
    Reviewing,
    Finalist,
    Rejected,
}

impl ProjectStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "new" => ProjectStatus::New,
            "finalist" => ProjectStatus::Finalist,
            "rejected" => ProjectStatus::Rejected,
            _ => ProjectStatus::Reviewing,
        }
    }
}

/// Body sent by the frontend after a drag-and-drop reorder:
/// { "category": "software", "order": [4, 3], "changed_by": "Ayşe" }
#[derive(Debug, Deserialize)]
pub struct RankingUpdate {
    pub category: String,
    pub order: Vec<i32>,
    pub changed_by: Option<String>,
}

/// PATCH /projects/{id} body — every field optional, only provided ones are applied.
#[derive(Debug, Deserialize)]
pub struct ProjectUpdate {
    pub notes: Option<String>,
    pub status: Option<String>,
    pub review_completed: Option<bool>,
    pub tags: Option<Vec<String>>,
}

/// One KPI definition within a category's scoring template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiTemplate {
    pub name: String,
    pub weight: f64,
    pub description: String,
}

/// The full KPI set a jury field (category) scores projects against
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryTemplate {
    pub category: String,
    pub kpis: Vec<KpiTemplate>,
}

/// One recorded manual ranking change, for the activity feed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEntry {
    pub project_id: i32,
    pub project_name: String,
    pub category: String,
    pub previous_rank: Option<i32>,
    pub new_rank: i32,
    pub changed_by: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: i32,
    pub competition_id: i32,
    pub name: String,
    pub status: String,
    pub members: Vec<TeamMember>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub id: i32,
    pub team_id: i32,
    pub full_name: String,
    pub email: String,
    pub role: String,
    pub is_scholar: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Submission {
    pub id: i32,
    pub team_id: i32,
    pub stage_id: i32,
    pub title: String,
    pub file_name: String,
    pub status: String,
    pub submitted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionVersion {
    pub id: i32,
    pub submission_id: i32,
    pub version: i32,
    pub file_name: String,
    pub file_path: String,
    pub uploaded_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTeam {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct AddTeamMember {
    pub full_name: String,
    pub email: String,
    pub role: Option<String>,
    pub is_scholar: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTeamStatus {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct FinalistSelection {
    pub team_ids: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoDaySlot {
    pub id: i32,
    pub competition_id: i32,
    pub team_id: i32,
    pub slot_order: i32,
    pub room: String,
    pub starts_at: String,
    pub duration_minutes: i32,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateDemoDaySlot {
    pub team_id: i32,
    pub slot_order: i32,
    pub room: String,
    pub starts_at: String,
    pub duration_minutes: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionReport {
    pub competition_id: i32,
    pub total_teams: i64,
    pub finalist_teams: i64,
    pub rejected_teams: i64,
    pub submitted_deliverables: i64,
    pub total_stages: i64,
    pub demo_day_slots: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateSubmission {
    pub stage_id: i32,
    pub title: String,
    pub file_name: String,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiKpiEvaluation {
    pub name: String,
    pub score: f64,
    pub reason: String,
    pub evidence: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarProject {
    pub project_id: Option<i32>,
    pub name: String,
    pub similarity: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiEvaluation {
    pub project_id: i32,
    pub model_version: String,
    pub total_score: f64,
    pub confidence: f64,
    pub kpi_scores: Vec<AiKpiEvaluation>,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub missing_information: Vec<String>,
    pub risks: Vec<String>,
    pub sources: Vec<String>,
    pub similar_projects: Vec<SimilarProject>,
    pub evaluated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UpsertAiEvaluation {
    pub model_version: String,
    pub total_score: f64,
    pub confidence: f64,
    pub kpi_scores: Vec<AiKpiEvaluation>,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub missing_information: Vec<String>,
    pub risks: Vec<String>,
    pub sources: Vec<String>,
    pub similar_projects: Vec<SimilarProject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JuryScore {
    pub id: i32,
    pub project_id: i32,
    pub juror_name: String,
    pub total_score: f64,
    pub kpi_scores: Vec<KpiScore>,
    pub notes: String,
    pub submitted_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateJuryScore {
    pub juror_name: String,
    pub total_score: f64,
    pub kpi_scores: Vec<KpiScore>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JuryAssignment {
    pub id: i32,
    pub project_id: i32,
    pub juror_name: String,
    pub role: String,
    pub status: String,
    pub conflict_declared: bool,
    pub assigned_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateJuryAssignment {
    pub juror_name: String,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: i32,
    pub action: String,
    pub actor: String,
    pub entity_type: String,
    pub entity_id: Option<i32>,
    pub details: serde_json::Value,
    pub created_at: String,
    pub previous_hash: Option<String>,
    pub event_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub full_name: String,
    pub email: String,
    pub role: String,
    pub active: bool,
    pub competition_id: Option<i32>,
    pub category: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub full_name: String,
    pub email: String,
    pub role: String,
    pub competition_id: Option<i32>,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub role: Option<String>,
    pub active: Option<bool>,
    pub competition_id: Option<i32>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleDefinition {
    pub role: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub kind: String,
    pub audience: String,
    pub category: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateNotification {
    pub title: String,
    pub body: String,
    pub kind: String,
    pub audience: String,
    pub category: Option<String>,
}

/// A competition container. Categories, stages and projects can be scoped to it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Competition {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub application_start: Option<String>,
    pub application_end: Option<String>,
    pub status: CompetitionStatus,
    pub organization: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompetitionStatus {
    Draft,
    Active,
    Archived,
}

impl CompetitionStatus {
    pub fn from_str(value: &str) -> Self {
        match value {
            "active" => Self::Active,
            "archived" => Self::Archived,
            _ => Self::Draft,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionStage {
    pub id: i32,
    pub competition_id: i32,
    pub name: String,
    pub stage_type: String,
    pub position: i32,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
    pub passing_score: f64,
    pub finalist_limit: Option<i32>,
    pub results_at: Option<String>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStageStatus { pub status: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionCategory {
    pub id: i32,
    pub competition_id: i32,
    pub parent_id: Option<i32>,
    pub name: String,
    pub slug: String,
    pub kpi_category: Option<String>,
}
