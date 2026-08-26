const API_URL = import.meta.env.PUBLIC_API_URL ?? 'http://127.0.0.1:3000';

function authHeaders(headers?: HeadersInit): Headers {
  const next = new Headers(headers);
  const token = typeof localStorage === 'undefined' ? null : localStorage.getItem('jury-auth-token');
  if (token) next.set('Authorization', `Bearer ${token}`);
  return next;
}

export interface AuthUser { id: number; full_name: string; email: string; role: string; active: boolean; must_change_password: boolean; two_factor_enabled: boolean; two_factor_required: boolean; competition_id: number | null; category: string | null; team_id: number | null; created_at: string; }
export interface AuthSession { token: string; expires_at: string; user: AuthUser; }
export function login(input: { email: string; password: string; totp_code?: string }): Promise<AuthSession> { return jsonRequest(`${API_URL}/auth/login`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(input) }); }
export async function logout(token: string): Promise<void> { await fetch(`${API_URL}/auth/logout`, { method: 'POST', headers: { Authorization: `Bearer ${token}` } }); }
export function getCurrentUser(): Promise<AuthUser> { return jsonRequest(`${API_URL}/auth/session`); }
export async function subscribeToUpdates(signal: AbortSignal, onRefresh: () => void): Promise<void> {
  const response = await fetch(`${API_URL}/events`, { headers: authHeaders(), signal });
  if (!response.ok || !response.body) throw new Error(`Live update subscription failed: ${response.status}`);
  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let buffer = '';
  while (!signal.aborted) {
    const { done, value } = await reader.read();
    if (done) return;
    buffer += decoder.decode(value, { stream: true });
    let boundary = buffer.indexOf('\n\n');
    while (boundary >= 0) {
      const message = buffer.slice(0, boundary);
      buffer = buffer.slice(boundary + 2);
      if (message.includes('event: refresh')) onRefresh();
      boundary = buffer.indexOf('\n\n');
    }
  }
}
export async function changePassword(input: { current_password: string; new_password: string }): Promise<void> { const response = await fetch(`${API_URL}/auth/password`, { method: 'PUT', headers: authHeaders({ 'Content-Type': 'application/json' }), body: JSON.stringify(input) }); if (!response.ok) throw new Error(`Password change failed: ${response.status}`); }
export interface TwoFactorSetup { secret: string; otpauth_url: string; }
export interface TwoFactorConfirmation { recovery_codes: string[]; }
export function setupTwoFactor(): Promise<TwoFactorSetup> { return jsonRequest(`${API_URL}/auth/2fa/setup`, { method: 'POST' }); }
export async function confirmTwoFactor(code: string): Promise<TwoFactorConfirmation> { return jsonRequest(`${API_URL}/auth/2fa/confirm`, { method: 'POST', headers: authHeaders({ 'Content-Type': 'application/json' }), body: JSON.stringify({ code }) }); }

export interface KpiScore {
  name: string;
  score: number;
}

export interface KpiTemplate {
  name: string;
  weight: number;
  description: string;
}

export interface CategoryTemplate {
  category: string;
  kpis: KpiTemplate[];
}

export function updateKpiTemplate(category: string, kpis: KpiTemplate[]): Promise<CategoryTemplate> {
  return jsonRequest(`${API_URL}/categories/${encodeURIComponent(category)}/kpis`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ kpis }),
  });
}

export type ProjectStatus = 'new' | 'reviewing' | 'finalist' | 'rejected';

export interface Competition {
  id: number;
  name: string;
  description: string;
  application_start: string | null;
  application_end: string | null;
  status: 'draft' | 'active' | 'archived';
  organization: string;
}
export interface OrganizationSummary { organization: string; competition_count: number; archived_count: number; }
export function getOrganizations(): Promise<OrganizationSummary[]> { return jsonRequest(`${API_URL}/organizations`); }

export interface CompetitionStage {
  id: number;
  competition_id: number;
  name: string;
  stage_type: string;
  position: number;
  starts_at: string | null;
  ends_at: string | null;
  passing_score: number;
  finalist_limit: number | null;
  results_at: string | null;
  status: 'planned' | 'active' | 'completed' | 'locked';
}

export interface TeamMember {
  id: number;
  team_id: number;
  full_name: string;
  email: string;
  role: string;
  is_scholar: boolean;
  birth_year: number | null;
  education_level: string;
}

export interface Team {
  id: number;
  competition_id: number;
  name: string;
  status: string;
  members: TeamMember[];
  created_at: string;
}

export interface Submission {
  id: number;
  team_id: number;
  stage_id: number;
  title: string;
  file_name: string;
  status: string;
  is_late: boolean;
  submitted_at: string | null;
}

export interface AiKpiEvaluation {
  name: string;
  score: number;
  reason: string;
  evidence: string[];
  confidence: number;
  source_file_version: number | null;
}

export interface SimilarProject {
  project_id: number | null;
  name: string;
  similarity: number;
  reason: string;
}

export interface AiEvaluation {
  project_id: number;
  model_version: string;
  total_score: number;
  confidence: number;
  kpi_scores: AiKpiEvaluation[];
  strengths: string[];
  weaknesses: string[];
  missing_information: string[];
  risks: string[];
  sources: string[];
  similar_projects: SimilarProject[];
  evaluated_at: string;
}
export interface JuryAiSummary {
  project_id: number;
  total_score: number;
  confidence: number;
  kpi_scores: AiKpiEvaluation[];
  strengths: string[];
  weaknesses: string[];
  missing_information: string[];
  risks: string[];
  evaluated_at: string;
}

export interface ResearchSource {
  title: string;
  url: string | null;
  source_type: string;
  snippet: string;
  matched_terms: string[];
  similarity: number;
  explanation: string;
}

export interface ProjectResearchAnalysis {
  project_id: number;
  source_file_version: number | null;
  originality_score: number;
  originality_label: string;
  query_terms: string[];
  sources: ResearchSource[];
  analyzed_at: string;
}

export interface CopilotResponse {
  answer: string;
  mode: string;
  citations: string[];
  suggested_questions: string[];
}

export interface JuryScore {
  id: number;
  project_id: number;
  stage_id: number | null;
  juror_name: string;
  total_score: number;
  kpi_scores: KpiScore[];
  notes: string;
  submitted_at: string;
}

export interface JuryAssignment {
  id: number;
  project_id: number;
  juror_name: string;
  role: string;
  status: string;
  conflict_declared: boolean;
  conflict_reason: string;
  assigned_at: string;
}

export interface JurorProfile { user_id: number; full_name: string; email: string; expertise: string[]; institution: string; max_assignments: number; active_assignments: number; }
export function getJurors(): Promise<JurorProfile[]> { return jsonRequest(`${API_URL}/jurors`); }
export async function updateJurorProfile(userId: number, input: { expertise: string[]; institution: string; max_assignments: number }): Promise<void> { const res = await fetch(`${API_URL}/jurors/${userId}/profile`, { method: 'PUT', headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${localStorage.getItem('jury-auth-token') ?? ''}` }, body: JSON.stringify(input) }); if (!res.ok) throw new Error(`Jury profile update failed: ${res.status}`); }
export interface Appeal { id: number; project_id: number; submitted_by: string; reason: string; deadline: string | null; committee: string[]; status: string; decision_reason: string; created_at: string; resolved_at: string | null; }
export function getAppeals(projectId: number): Promise<Appeal[]> { return jsonRequest(`${API_URL}/projects/${projectId}/appeals`); }
export function createAppeal(projectId: number, input: { submitted_by: string; reason: string; deadline?: string | null; committee: string[] }): Promise<Appeal> { return jsonRequest(`${API_URL}/projects/${projectId}/appeals`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(input) }); }
export interface EligibilityReport { project_id: number; eligible: boolean; checks: { key: string; label: string; passed: boolean; detail: string }[]; }
export function getEligibilityReport(projectId: number): Promise<EligibilityReport> { return jsonRequest(`${API_URL}/projects/${projectId}/eligibility`); }

export interface TemplateSection { key: string; title: string; aliases: string[]; min_words: number; required: boolean; }
export interface ReportTemplate { competition_id: number; name: string; version: number; expected_language: string; min_words: number; max_words: number; sections: TemplateSection[]; updated_at: string; updated_by: string; }
export interface SectionFinding { key: string; title: string; required: boolean; status: 'present' | 'thin' | 'missing'; matched_heading: string | null; word_count: number; min_words: number; detail: string; }
export interface TemplateCompliance {
  project_id: number;
  template_name: string;
  template_version: number;
  compliant: boolean;
  section_score: number;
  sections: SectionFinding[];
  language_expected: string;
  language_detected: string;
  language_matches: boolean;
  word_count: number;
  min_words: number;
  max_words: number;
  word_count_within_range: boolean;
  summary: string;
  evaluated_at: string;
}
export interface CategoryFitAnalysis {
  project_id: number;
  source_file_version: number | null;
  current_category_score: number;
  recommended_category: string;
  recommended_category_score: number;
  matched_terms: string[];
  requires_review: boolean;
  analyzed_at: string;
}
export interface ProjectSimilarityMatch {
  project_id: number;
  project_reference: string;
  category: string;
  similarity: number;
  matched_terms: string[];
}
export interface ProjectSimilarityAnalysis {
  project_id: number;
  source_file_version: number | null;
  highest_similarity: number;
  requires_review: boolean;
  matches: ProjectSimilarityMatch[];
  analyzed_at: string;
}
export interface AssessmentGate {
  key: string;
  label: string;
  status: 'passed' | 'failed' | 'pending';
  detail: string;
  requires_human_review: boolean;
}
export interface ProjectAssessmentReadiness {
  project_id: number;
  ready_for_evaluation: boolean;
  checks: AssessmentGate[];
}
export function getSupportedLanguages(): Promise<string[]> { return jsonRequest(`${API_URL}/languages`); }
export function getTemplateCompliance(projectId: number): Promise<TemplateCompliance> { return jsonRequest(`${API_URL}/projects/${projectId}/template-compliance`); }
export async function getCategoryFitAnalysis(projectId: number): Promise<CategoryFitAnalysis | null> { const response = await fetch(`${API_URL}/projects/${projectId}/category-fit`, { headers: authHeaders() }); if (response.status === 404) return null; if (!response.ok) throw new Error(`Category-fit request failed: ${response.status}`); return response.json(); }
export function runCategoryFitAnalysis(projectId: number): Promise<CategoryFitAnalysis> { return jsonRequest(`${API_URL}/projects/${projectId}/category-fit`, { method: 'POST' }); }
export async function getProjectSimilarityAnalysis(projectId: number): Promise<ProjectSimilarityAnalysis | null> { const response = await fetch(`${API_URL}/projects/${projectId}/similarity`, { headers: authHeaders() }); if (response.status === 404) return null; if (!response.ok) throw new Error(`Similarity request failed: ${response.status}`); return response.json(); }
export function runProjectSimilarityAnalysis(projectId: number): Promise<ProjectSimilarityAnalysis> { return jsonRequest(`${API_URL}/projects/${projectId}/similarity`, { method: 'POST' }); }
export function getProjectAssessmentReadiness(projectId: number): Promise<ProjectAssessmentReadiness> { return jsonRequest(`${API_URL}/projects/${projectId}/assessment-readiness`); }
export function getReportTemplate(competitionId: number): Promise<ReportTemplate> { return jsonRequest(`${API_URL}/competitions/${competitionId}/report-template`); }
export function saveReportTemplate(competitionId: number, input: { name: string; expected_language: string; min_words: number; max_words: number; sections: TemplateSection[] }): Promise<ReportTemplate> {
  return jsonRequest(`${API_URL}/competitions/${competitionId}/report-template`, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(input) });
}

export interface DemoDaySlot {
  id: number;
  competition_id: number;
  team_id: number;
  slot_order: number;
  room: string;
  starts_at: string;
  duration_minutes: number;
  status: string;
  checked_in_at: string | null;
  evidence_urls: string[];
  field_score: number | null;
  jury_signature: string | null;
  check_in_token: string;
  prototype_checklist: string[];
}
export function updateDemoDaySlot(slotId: number, input: { status?: string; check_in?: boolean; evidence_urls?: string[]; field_score?: number; jury_signature?: string; prototype_checklist?: string[] }): Promise<DemoDaySlot> { return jsonRequest(`${API_URL}/demo-day/${slotId}`, { method: 'PATCH', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(input) }); }

export interface CompetitionReport {
  competition_id: number;
  total_teams: number;
  finalist_teams: number;
  rejected_teams: number;
  submitted_deliverables: number;
  total_stages: number;
  demo_day_slots: number;
}

/** Mirrors `USER_ROLES` in backend/src/main.rs — keep both lists in step. */
export type UserRole = 'system_admin' | 'competition_manager' | 'chief_judge' | 'evaluation_manager' | 'jury_member' | 'contestant' | 'observer' | 'read_only';
export interface RoleDefinition { role: UserRole; permissions: string[]; }
export interface User {
  id: number;
  full_name: string;
  email: string;
  role: UserRole;
  active: boolean;
  competition_id: number | null;
  category: string | null;
  team_id: number | null;
  created_at: string;
}
export interface CategoryFitSummary { current_category_score: number; recommended_category: string; recommended_category_score: number; requires_review: boolean; }
export interface ContestantFeedback { project_id: number; project_name: string; category: string; status: ProjectStatus; total_score: number; strengths: string[]; weaknesses: string[]; missing_information: string[]; risks: string[]; evaluated_at: string; category_fit: CategoryFitSummary | null; }
export function getMyFeedback(): Promise<ContestantFeedback[]> { return jsonRequest(`${API_URL}/my-feedback`); }

export interface AuditEvent {
  id: number;
  action: string;
  actor: string;
  entity_type: string;
  entity_id: number | null;
  details: Record<string, unknown>;
  created_at: string;
  previous_hash: string | null;
  event_hash: string;
}

export function getAuditEvents(limit = 50): Promise<AuditEvent[]> { return jsonRequest(`${API_URL}/audit?limit=${limit}`); }

export function getUsers(): Promise<User[]> { return jsonRequest(`${API_URL}/users`); }
export function getRoles(): Promise<RoleDefinition[]> { return jsonRequest(`${API_URL}/roles`); }

export type NotificationKind = 'announcement' | 'missing_document' | 'deadline' | 'review_task' | 'result' | 'question' | 'faq';
export interface Notification { id: number; title: string; body: string; kind: NotificationKind; audience: string; category: string | null; created_at: string; }
export function getNotifications(limit = 50): Promise<Notification[]> { return jsonRequest(`${API_URL}/notifications?limit=${limit}`); }
export function createNotification(input: Omit<Notification, 'id' | 'created_at'>): Promise<Notification> { return jsonRequest(`${API_URL}/notifications`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(input) }); }

export interface EmailCampaign { id: number; subject: string; body: string; audience: string; category: string | null; recipient_count: number; status: string; created_at: string; }
export function getEmailCampaigns(limit = 50): Promise<EmailCampaign[]> { return jsonRequest(`${API_URL}/email-campaigns?limit=${limit}`); }
export function createEmailCampaign(input: Omit<EmailCampaign, 'id' | 'recipient_count' | 'status' | 'created_at'>): Promise<EmailCampaign> { return jsonRequest(`${API_URL}/email-campaigns`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(input) }); }
export function dispatchEmailCampaign(id: number): Promise<EmailCampaign> { return jsonRequest(`${API_URL}/email-campaigns/${id}/dispatch`, { method: 'POST' }); }
export function createUser(input: Omit<User, 'id' | 'active' | 'must_change_password' | 'created_at'> & { password: string }): Promise<User> {
  return jsonRequest(`${API_URL}/users`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(input) });
}
export function updateUser(id: number, input: Partial<Pick<User, 'role' | 'active' | 'competition_id' | 'category' | 'team_id'>>): Promise<User> {
  return jsonRequest(`${API_URL}/users/${id}`, { method: 'PATCH', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(input) });
}
export interface PasswordResetToken { token: string; expires_at: string; }
export function issuePasswordReset(id: number): Promise<PasswordResetToken> { return jsonRequest(`${API_URL}/users/${id}/password-reset`, { method: 'POST' }); }
export async function confirmPasswordReset(input: { token: string; new_password: string }): Promise<void> { const response = await fetch(`${API_URL}/auth/password-reset/confirm`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(input) }); if (!response.ok) throw new Error(`Password reset failed: ${response.status}`); }

export interface Project {
  id: number;
  competition_id: number;
  team_id: number | null;
  name: string;
  category: string;
  kpi_scores: KpiScore[];
  ai_score: number;
  manual_rank: number | null;
  notes: string;
  status: ProjectStatus;
  has_file: boolean;
  review_completed: boolean;
  tags: string[];
}

export interface ProjectMetadata {
  project_id: number;
  institution: string;
  keywords: string[];
  github_url: string | null;
  demo_url: string | null;
  prototype_description: string;
  team_name: string;
  team_members: string[];
  updated_at: string;
}

export function getProjectMetadata(id: number): Promise<ProjectMetadata> { return jsonRequest(`${API_URL}/projects/${id}/metadata`); }
export function updateProjectMetadata(id: number, input: Partial<Omit<ProjectMetadata, 'project_id' | 'updated_at'>>): Promise<ProjectMetadata> {
  return jsonRequest(`${API_URL}/projects/${id}/metadata`, { method: 'PATCH', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(input) });
}

export interface ProjectFile { id: number; project_id: number; version: number; file_name: string; mime_type: string; size_bytes: number; file_path: string; uploaded_at: string; }
export function getProjectFiles(id: number): Promise<ProjectFile[]> { return jsonRequest(`${API_URL}/projects/${id}/files`); }
export function uploadProjectFile(id: number, file: File, setAsReport = false): Promise<ProjectFile> {
  const body = new FormData();
  body.append('file', file);
  if (setAsReport) body.append('set_as_report', 'true');
  return jsonRequest(`${API_URL}/projects/${id}/files`, { method: 'POST', body });
}
export function projectVersionFileUrl(projectId: number, fileId: number): string { return `${API_URL}/projects/${projectId}/files/${fileId}`; }

export interface Section {
  title: string;
  content: string;
  word_count: number;
}

export interface Document {
  filename: string;
  file_type: 'Pdf' | 'Txt' | 'Markdown' | 'Docx';
  raw_text: string;
  word_count: number;
  headings: string[];
  keywords: string[];
  references: string[];
  has_references: boolean;
  has_abstract: boolean;
  has_conclusion: boolean;
  has_methodology: boolean;
  language: string;
  sections: Section[];
}

export interface ActivityEntry {
  project_id: number;
  project_name: string;
  category: string;
  previous_rank: number | null;
  new_rank: number;
  changed_by: string | null;
  timestamp: string;
}

export async function getCategories(): Promise<CategoryTemplate[]> {
  const res = await fetch(`${API_URL}/categories`, { headers: authHeaders() });
  if (!res.ok) throw new Error(`Failed to load categories: ${res.status}`);
  return res.json();
}

async function jsonRequest<T>(url: string, init?: RequestInit): Promise<T> {
  const headers = authHeaders(init?.headers);
  const res = await fetch(url, { ...init, headers });
  if (!res.ok) throw new Error(`API request failed: ${res.status} ${await res.text()}`);
  return res.json();
}

export function getCompetitions(): Promise<Competition[]> {
  return jsonRequest(`${API_URL}/competitions`);
}

export function createCompetition(input: {
  name: string;
  description?: string;
  application_start?: string;
  application_end?: string;
  organization?: string;
}): Promise<Competition> {
  return jsonRequest(`${API_URL}/competitions`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(input),
  });
}

export function getCompetitionStages(competitionId: number): Promise<CompetitionStage[]> {
  return jsonRequest(`${API_URL}/competitions/${competitionId}/stages`);
}
export function updateStageStatus(competitionId: number, stageId: number, status: CompetitionStage['status']): Promise<CompetitionStage> { return jsonRequest(`${API_URL}/competitions/${competitionId}/stages/${stageId}/status`, { method: 'PATCH', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ status }) }); }

export interface CompetitionCategory {
  id: number;
  competition_id: number;
  parent_id: number | null;
  name: string;
  slug: string;
  kpi_category: string | null;
}

/** `status` is assigned by the backend ('planned'); it is not part of the request. */
export function createCompetitionStage(
  competitionId: number,
  input: Omit<CompetitionStage, 'id' | 'competition_id' | 'status'>,
): Promise<CompetitionStage> {
  return jsonRequest(`${API_URL}/competitions/${competitionId}/stages`, {
    method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(input),
  });
}

export function getCompetitionCategories(competitionId: number): Promise<CompetitionCategory[]> {
  return jsonRequest(`${API_URL}/competitions/${competitionId}/categories`);
}

export function createCompetitionCategory(
  competitionId: number,
  input: Omit<CompetitionCategory, 'id' | 'competition_id'>,
): Promise<CompetitionCategory> {
  return jsonRequest(`${API_URL}/competitions/${competitionId}/categories`, {
    method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(input),
  });
}

export function getCompetitionTeams(competitionId: number): Promise<Team[]> {
  return jsonRequest(`${API_URL}/competitions/${competitionId}/teams`);
}

export function createTeam(competitionId: number, name: string): Promise<Team> {
  return jsonRequest(`${API_URL}/competitions/${competitionId}/teams`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name }),
  });
}

export async function updateTeamStatus(teamId: number, status: string): Promise<void> {
  const res = await fetch(`${API_URL}/teams/${teamId}`, {
    method: 'PATCH',
    headers: authHeaders({ 'Content-Type': 'application/json' }),
    body: JSON.stringify({ status }),
  });
  if (!res.ok) throw new Error(`Failed to update team status: ${res.status} ${await res.text()}`);
}

export function addTeamMember(
  teamId: number,
  input: { full_name: string; email: string; role?: string; is_scholar: boolean; birth_year?: number | null; education_level?: string },
): Promise<TeamMember> {
  return jsonRequest(`${API_URL}/teams/${teamId}/members`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(input),
  });
}

export function getDemoDaySlots(competitionId: number): Promise<DemoDaySlot[]> {
  return jsonRequest(`${API_URL}/competitions/${competitionId}/demo-day`);
}

export function getCompetitionReport(competitionId: number): Promise<CompetitionReport> {
  return jsonRequest(`${API_URL}/competitions/${competitionId}/report`);
}
export async function finalizeCompetition(competitionId: number, input: { minutes: string; signed_by: string }): Promise<void> { const res = await fetch(`${API_URL}/competitions/${competitionId}/finalize`, { method: 'POST', headers: authHeaders({ 'Content-Type': 'application/json' }), body: JSON.stringify(input) }); if (!res.ok) throw new Error(`Finalization failed: ${res.status} ${await res.text()}`); }

export function getTeamSubmissions(teamId: number): Promise<Submission[]> {
  return jsonRequest(`${API_URL}/teams/${teamId}/submissions`);
}

export async function getAiEvaluation(projectId: number): Promise<AiEvaluation | null> {
  const res = await fetch(`${API_URL}/projects/${projectId}/ai-evaluation`, { headers: authHeaders() });
  if (res.status === 404) return null;
  if (!res.ok) throw new Error(`AI evaluation request failed: ${res.status}`);
  return res.json();
}
export async function getJuryAiSummary(projectId: number): Promise<JuryAiSummary | null> {
  const res = await fetch(`${API_URL}/projects/${projectId}/jury-ai-summary`, { headers: authHeaders() });
  if (res.status === 404) return null;
  if (!res.ok) throw new Error(`Jury AI summary request failed: ${res.status}`);
  return res.json();
}

/** Adapter contract for the external AI team: the dashboard stores and renders this payload. */
export function upsertAiEvaluation(
  projectId: number,
  evaluation: Omit<AiEvaluation, 'project_id' | 'evaluated_at'>,
): Promise<AiEvaluation> {
  return jsonRequest(`${API_URL}/projects/${projectId}/ai-evaluation`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(evaluation),
  });
}

export async function getProjectResearch(projectId: number): Promise<ProjectResearchAnalysis | null> {
  const response = await fetch(`${API_URL}/projects/${projectId}/research`, { headers: authHeaders() });
  if (response.status === 404) return null;
  if (!response.ok) throw new Error(`Research analysis request failed: ${response.status}`);
  return response.json();
}

export function runProjectResearch(projectId: number, refresh = false): Promise<ProjectResearchAnalysis> {
  return jsonRequest(`${API_URL}/projects/${projectId}/research`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ refresh }),
  });
}

export function askProjectCopilot(projectId: number, question: string): Promise<CopilotResponse> {
  return jsonRequest(`${API_URL}/projects/${projectId}/copilot`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ question }),
  });
}

export function getJuryScores(projectId: number): Promise<JuryScore[]> {
  return jsonRequest(`${API_URL}/projects/${projectId}/jury-scores`);
}

export function addJuryScore(projectId: number, input: Omit<JuryScore, 'id' | 'project_id' | 'submitted_at'>): Promise<JuryScore> {
  return jsonRequest(`${API_URL}/projects/${projectId}/jury-scores`, {
    method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(input),
  });
}

export function getJuryAssignments(projectId: number): Promise<JuryAssignment[]> {
  return jsonRequest(`${API_URL}/projects/${projectId}/jury-assignments`);
}

export async function getProjects(category?: string): Promise<Project[]> {
  const url = category ? `${API_URL}/projects?category=${encodeURIComponent(category)}` : `${API_URL}/projects`;
  const res = await fetch(url, { headers: authHeaders() });
  if (!res.ok) throw new Error(`Failed to load projects: ${res.status}`);
  return res.json();
}

export async function uploadProject(name: string, category: string, competitionId: number, file: File, teamId?: number): Promise<Project> {
  const formData = new FormData();
  formData.append('name', name);
  formData.append('category', category);
  formData.append('competition_id', String(competitionId));
  if (teamId) formData.append('team_id', String(teamId));
  formData.append('file', file);

  const res = await fetch(`${API_URL}/projects/upload`, { method: 'POST', headers: authHeaders(), body: formData });
  if (!res.ok) throw new Error(`Failed to upload project: ${res.status} ${await res.text()}`);
  return res.json();
}

export async function updateRanking(category: string, order: number[]): Promise<void> {
  const res = await fetch(`${API_URL}/ranking`, {
    method: 'PATCH',
    headers: authHeaders({ 'Content-Type': 'application/json' }),
    body: JSON.stringify({ category, order }),
  });
  if (!res.ok) throw new Error(`Failed to update ranking: ${res.status}`);
}

export async function updateProject(
  id: number,
  update: { notes?: string; status?: ProjectStatus; review_completed?: boolean; tags?: string[] },
): Promise<Project> {
  const res = await fetch(`${API_URL}/projects/${id}`, {
    method: 'PATCH',
    headers: authHeaders({ 'Content-Type': 'application/json' }),
    body: JSON.stringify(update),
  });
  if (!res.ok) throw new Error(`Failed to update project: ${res.status}`);
  return res.json();
}

export function projectFileUrl(id: number): string {
  return `${API_URL}/projects/${id}/file`;
}

export async function fetchProtectedFile(url: string): Promise<Response> {
  const response = await fetch(url, { headers: authHeaders() });
  if (!response.ok) throw new Error(`File download failed: ${response.status}`);
  return response;
}

export async function getProjectDocument(id: number): Promise<Document | null> {
  const res = await fetch(`${API_URL}/projects/${id}/document`, { headers: authHeaders() });
  if (res.status === 404) return null;
  if (!res.ok) throw new Error(`Failed to load document: ${res.status}`);
  return res.json();
}

export async function getActivity(category?: string, limit = 10): Promise<ActivityEntry[]> {
  const params = new URLSearchParams({ limit: String(limit) });
  if (category) params.set('category', category);
  const res = await fetch(`${API_URL}/activity?${params}`, { headers: authHeaders() });
  if (!res.ok) throw new Error(`Failed to load activity: ${res.status}`);
  return res.json();
}
