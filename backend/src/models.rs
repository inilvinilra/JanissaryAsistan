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

/// Serialised as a bare string ("Turkish", "German", "Unknown"), so documents
/// stored before multi-language detection still deserialise unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Language(String);

impl Language {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn unknown() -> Self {
        Self("Unknown".into())
    }

    pub fn turkish() -> Self {
        Self("Turkish".into())
    }

    pub fn english() -> Self {
        Self("English".into())
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
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
pub struct ResearchSource {
    pub title: String,
    pub url: Option<String>,
    pub source_type: String,
    pub snippet: String,
    pub matched_terms: Vec<String>,
    pub similarity: f64,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectResearchAnalysis {
    pub project_id: i32,
    pub source_file_version: Option<i32>,
    pub originality_score: f64,
    pub originality_label: String,
    pub query_terms: Vec<String>,
    pub sources: Vec<ResearchSource>,
    pub analyzed_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ResearchRequest {
    #[serde(default)]
    pub refresh: bool,
}

#[derive(Debug, Deserialize)]
pub struct CopilotRequest {
    pub question: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CopilotResponse {
    pub answer: String,
    pub mode: String,
    pub citations: Vec<String>,
    pub suggested_questions: Vec<String>,
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

#[derive(Debug, Clone, Serialize)]
pub struct JuryAiSummary {
    pub project_id: i32,
    pub total_score: f64,
    pub confidence: f64,
    pub kpi_scores: Vec<AiKpiEvaluation>,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub missing_information: Vec<String>,
    pub risks: Vec<String>,
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
    pub team_id: Option<i32>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub full_name: String,
    pub email: String,
    pub role: String,
    pub competition_id: Option<i32>,
    pub category: Option<String>,
    pub team_id: Option<i32>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub role: Option<String>,
    pub active: Option<bool>,
    pub competition_id: Option<i32>,
    pub category: Option<String>,
    pub team_id: Option<i32>,
    pub password: Option<String>,
}

/// What an applicant is shown: the three things the brief promises them —
/// strengths, areas to improve, and suggestions.
///
/// Deliberately not the whole evaluation. The `risks` list is written for the
/// judge and carries signals that must not cross to an applicant: it names the
/// reference of the submission theirs resembles, which would disclose another
/// team's entry, and it discusses how far the AI's own scores can be trusted,
/// which is a question for the judge rather than the applicant. Leaving the
/// field out of this type keeps that separation structural rather than relying
/// on every future caller to remember to filter it.
#[derive(Debug, Clone, Serialize)]
pub struct ContestantFeedback {
    pub project_id: i32,
    pub project_name: String,
    pub category: String,
    pub status: ProjectStatus,
    pub total_score: f64,
    pub strengths: Vec<String>,
    /// Areas to improve.
    pub weaknesses: Vec<String>,
    /// Concrete actions, drawn from what the evaluation found missing.
    pub suggestions: Vec<String>,
    pub evaluated_at: String,
    /// `None` until a manager runs the category-fit analysis for this project.
    pub category_fit: Option<CategoryFitSummary>,
}

/// A contestant-safe view of `CategoryFitAnalysis`: no matched terms or file
/// version, since those are internal review evidence, not applicant-facing.
#[derive(Debug, Clone, Serialize)]
pub struct CategoryFitSummary {
    pub current_category_score: f64,
    pub recommended_category: String,
    pub recommended_category_score: f64,
    pub requires_review: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateSection {
    pub key: String,
    pub title: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub min_words: i64,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTemplate {
    pub competition_id: i32,
    pub name: String,
    pub version: i32,
    pub expected_language: String,
    pub min_words: i64,
    pub max_words: i64,
    pub sections: Vec<TemplateSection>,
    pub updated_at: String,
    pub updated_by: String,
}

#[derive(Debug, Deserialize)]
pub struct UpsertReportTemplate {
    pub name: String,
    pub expected_language: String,
    pub min_words: i64,
    pub max_words: i64,
    pub sections: Vec<TemplateSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionFinding {
    pub key: String,
    pub title: String,
    pub required: bool,
    pub status: String,
    pub matched_heading: Option<String>,
    pub word_count: i64,
    pub min_words: i64,
    pub detail: String,
}

impl SectionFinding {
    /// "present" is the only state that satisfies a required section. "thin"
    /// means the heading exists but the section is shorter than the template
    /// demands; "off_topic" means it is long enough but discusses something
    /// else. Callers must go through this instead of comparing the string,
    /// which is how the readiness gate once tested for a status that is never
    /// produced.
    pub fn is_satisfied(&self) -> bool {
        self.status == "present"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateCompliance {
    pub project_id: i32,
    pub template_name: String,
    pub template_version: i32,
    pub compliant: bool,
    pub section_score: f64,
    pub sections: Vec<SectionFinding>,
    pub language_expected: String,
    pub language_detected: String,
    pub language_matches: bool,
    pub word_count: i64,
    pub min_words: i64,
    pub max_words: i64,
    pub word_count_within_range: bool,
    pub summary: String,
    pub evaluated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryFitAnalysis {
    pub project_id: i32,
    pub source_file_version: Option<i32>,
    pub current_category_score: f64,
    pub recommended_category: String,
    pub recommended_category_score: f64,
    pub matched_terms: Vec<String>,
    pub requires_review: bool,
    pub analyzed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSimilarityMatch {
    pub project_id: i32,
    pub project_reference: String,
    pub category: String,
    /// Headline figure shown to the jury: the stronger of the two measures.
    pub similarity: f64,
    /// Shared vocabulary over combined vocabulary. Separates genuine overlap
    /// from two reports that merely share a language and a subject area.
    #[serde(default)]
    pub jaccard: f64,
    /// Shared vocabulary over the *smaller* report's own vocabulary. Stays high
    /// when a copied section is buried inside a much longer padded document,
    /// which is exactly the case Jaccard dilutes away.
    #[serde(default)]
    pub containment: f64,
    pub matched_terms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSimilarityAnalysis {
    pub project_id: i32,
    pub source_file_version: Option<i32>,
    pub highest_similarity: f64,
    pub requires_review: bool,
    pub matches: Vec<ProjectSimilarityMatch>,
    pub analyzed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentGate {
    pub key: String,
    pub label: String,
    pub status: String,
    pub detail: String,
    pub requires_human_review: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAssessmentReadiness {
    pub project_id: i32,
    pub ready_for_evaluation: bool,
    pub checks: Vec<AssessmentGate>,
}

/// Competition-wide analysis progress for the evaluation manager, whose role in
/// the brief is to watch completion rates rather than individual reports.
///
/// Derived entirely from the stored analyses rather than from a job record, so
/// it survives a restart and reports the same figures whichever instance is
/// asked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentProgress {
    pub competition_id: i32,
    pub total_projects: i64,
    /// Projects whose report parsed. The analyses can only run on these, so
    /// they are the denominator for the percentages below.
    pub parsed_reports: i64,
    pub category_fit_completed: i64,
    pub similarity_completed: i64,
    pub criterion_evaluation_completed: i64,
    /// Projects an earlier gate marked for human attention.
    pub flagged_for_review: i64,
    /// Share of the three analyses completed across all parsed reports.
    pub completion_percent: f64,
    /// Projects still missing at least one analysis, newest first, capped for
    /// the response size. This is the manager's work list.
    pub pending_projects: Vec<PendingAssessment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingAssessment {
    pub project_id: i32,
    pub project_reference: String,
    pub category: String,
    pub missing: Vec<String>,
}
