use serde::{Deserialize, Serialize};

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
    Spreadsheet,
    Image,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub source_type: String,
    pub fetched_content: Option<String>,
    pub http_status: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiScore {
    pub name: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i32,
    pub competition_id: i32,
    pub team_id: Option<i32>,
    pub name: String,
    pub category: String,
    pub kpi_scores: Vec<KpiScore>,
    pub ai_score: f64,
    pub manual_rank: Option<i32>,
    pub notes: String,
    pub status: ProjectStatus,
    pub has_file: bool,
    pub review_completed: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BlindProject {
    pub reference: String,
    pub category: String,
    pub kpi_scores: Vec<KpiScore>,
    pub ai_score: f64,
    pub manual_rank: Option<i32>,
    pub status: ProjectStatus,
    pub review_completed: bool,
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

#[derive(Debug, Deserialize)]
pub struct RankingUpdate {
    pub category: String,
    pub order: Vec<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectUpdate {
    pub notes: Option<String>,
    pub status: Option<String>,
    pub review_completed: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiTemplate {
    pub name: String,
    pub weight: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryTemplate {
    pub category: String,
    pub kpis: Vec<KpiTemplate>,
}

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
    pub birth_year: Option<i32>,
    pub education_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Submission {
    pub id: i32,
    pub team_id: i32,
    pub stage_id: i32,
    pub title: String,
    pub file_name: String,
    pub status: String,
    pub is_late: bool,
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
    pub birth_year: Option<i32>,
    pub education_level: Option<String>,
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
    pub checked_in_at: Option<String>,
    pub evidence_urls: Vec<String>,
    pub field_score: Option<f64>,
    pub jury_signature: Option<String>,
    pub check_in_token: String,
    pub prototype_checklist: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDemoDaySlot {
    pub team_id: i32,
    pub slot_order: i32,
    pub room: String,
    pub starts_at: String,
    pub duration_minutes: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDemoDaySlot {
    pub status: Option<String>,
    pub check_in: Option<bool>,
    pub evidence_urls: Option<Vec<String>>,
    pub field_score: Option<f64>,
    pub jury_signature: Option<String>,
    pub prototype_checklist: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct FinalizeCompetition {
    pub minutes: String,
    pub signed_by: String,
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
    pub source_file_version: Option<i32>,
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
    #[serde(default)]
    pub source_file_version: Option<i32>,
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
    pub stage_id: Option<i32>,
    pub juror_name: String,
    pub total_score: f64,
    pub kpi_scores: Vec<KpiScore>,
    pub notes: String,
    pub submitted_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateJuryScore {
    pub stage_id: Option<i32>,
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
    pub conflict_reason: String,
    pub assigned_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateJuryAssignment {
    pub juror_name: String,
    pub role: Option<String>,
    pub category: Option<String>,
    pub conflict_declared: Option<bool>,
    pub conflict_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct JuryReadiness {
    pub project_id: i32,
    pub assigned_count: i64,
    pub minimum_required: i64,
    pub ready: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct JurorProfile {
    pub user_id: i32,
    pub full_name: String,
    pub email: String,
    pub expertise: Vec<String>,
    pub institution: String,
    pub max_assignments: i32,
    pub active_assignments: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateJurorProfile {
    pub expertise: Vec<String>,
    pub institution: String,
    pub max_assignments: i32,
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
    pub must_change_password: bool,
    pub two_factor_enabled: bool,
    pub two_factor_required: bool,
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
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub role: Option<String>,
    pub active: Option<bool>,
    pub competition_id: Option<i32>,
    pub category: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub totp_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmPasswordResetRequest {
    pub token: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct PasswordResetToken {
    pub token: String,
    pub expires_at: String,
}

#[derive(Debug, Serialize)]
pub struct AuthSession {
    pub token: String,
    pub expires_at: String,
    pub user: User,
}

#[derive(Debug, Serialize)]
pub struct TwoFactorSetup {
    pub secret: String,
    pub otpauth_url: String,
}

#[derive(Debug, Deserialize)]
pub struct TwoFactorConfirm {
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct TwoFactorConfirmation {
    pub recovery_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Appeal {
    pub id: i32,
    pub project_id: i32,
    pub submitted_by: String,
    pub reason: String,
    pub deadline: Option<String>,
    pub committee: Vec<String>,
    pub status: String,
    pub decision_reason: String,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAppeal {
    pub submitted_by: String,
    pub reason: String,
    pub deadline: Option<String>,
    pub committee: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResolveAppeal {
    pub status: String,
    pub decision_reason: String,
    pub new_score: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct EligibilityCheck {
    pub key: String,
    pub label: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct EligibilityReport {
    pub project_id: i32,
    pub eligible: bool,
    pub checks: Vec<EligibilityCheck>,
}

#[derive(Debug, Serialize)]
pub struct JurorCalibration {
    pub juror_name: String,
    pub score_count: i64,
    pub average_score: f64,
    pub deviation_from_overall: f64,
    pub alert: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CalibrationSummary {
    pub overall_average: f64,
    pub jurors: Vec<JurorCalibration>,
    pub kpi_comment_variance: Vec<KpiCommentVariance>,
}

#[derive(Debug, Serialize)]
pub struct KpiCommentVariance {
    pub project_id: i32,
    pub comment_count: i64,
    pub distinct_comment_count: i64,
    pub alert: bool,
}

#[derive(Debug, Serialize)]
pub struct CalibrationCase {
    pub id: i32,
    pub project_id: i32,
    pub expected_score: f64,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateCalibrationCase {
    pub project_id: i32,
    pub expected_score: f64,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailCampaign {
    pub id: i32,
    pub subject: String,
    pub body: String,
    pub audience: String,
    pub category: Option<String>,
    pub recipient_count: i64,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateEmailCampaign {
    pub subject: String,
    pub body: String,
    pub audience: String,
    pub category: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EmailDeliveryTarget {
    pub id: i32,
    pub email: String,
    pub attempt_count: i32,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationSummary {
    pub organization: String,
    pub competition_count: i64,
    pub archived_count: i64,
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
pub struct UpdateStageStatus {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionCategory {
    pub id: i32,
    pub competition_id: i32,
    pub parent_id: Option<i32>,
    pub name: String,
    pub slug: String,
    pub kpi_category: Option<String>,
}
