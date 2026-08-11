const API_URL = import.meta.env.PUBLIC_API_URL;

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
  submitted_at: string | null;
}

export interface AiKpiEvaluation {
  name: string;
  score: number;
  reason: string;
  evidence: string[];
  confidence: number;
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

export interface JuryScore {
  id: number;
  project_id: number;
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
  assigned_at: string;
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
}

export interface CompetitionReport {
  competition_id: number;
  total_teams: number;
  finalist_teams: number;
  rejected_teams: number;
  submitted_deliverables: number;
  total_stages: number;
  demo_day_slots: number;
}

export type UserRole = 'system_admin' | 'competition_manager' | 'chief_judge' | 'jury_member' | 'observer' | 'read_only';
export interface RoleDefinition { role: UserRole; permissions: string[]; }
export interface User {
  id: number;
  full_name: string;
  email: string;
  role: UserRole;
  active: boolean;
  competition_id: number | null;
  category: string | null;
  created_at: string;
}

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
export function createUser(input: Omit<User, 'id' | 'active' | 'created_at'>): Promise<User> {
  return jsonRequest(`${API_URL}/users`, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(input) });
}
export function updateUser(id: number, input: Partial<Pick<User, 'role' | 'active' | 'competition_id' | 'category'>>): Promise<User> {
  return jsonRequest(`${API_URL}/users/${id}`, { method: 'PATCH', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(input) });
}

export interface Project {
  id: number;
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
export function uploadProjectFile(id: number, file: File): Promise<ProjectFile> {
  const body = new FormData(); body.append('file', file);
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
  language: 'Turkish' | 'English' | 'Unknown';
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
  const res = await fetch(`${API_URL}/categories`);
  if (!res.ok) throw new Error(`Failed to load categories: ${res.status}`);
  return res.json();
}

async function jsonRequest<T>(url: string, init?: RequestInit): Promise<T> {
  const res = await fetch(url, init);
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

export function createCompetitionStage(
  competitionId: number,
  input: Omit<CompetitionStage, 'id' | 'competition_id'>,
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
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ status }),
  });
  if (!res.ok) throw new Error(`Failed to update team status: ${res.status} ${await res.text()}`);
}

export function addTeamMember(
  teamId: number,
  input: { full_name: string; email: string; role?: string; is_scholar: boolean },
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

export function getTeamSubmissions(teamId: number): Promise<Submission[]> {
  return jsonRequest(`${API_URL}/teams/${teamId}/submissions`);
}

export async function getAiEvaluation(projectId: number): Promise<AiEvaluation | null> {
  const res = await fetch(`${API_URL}/projects/${projectId}/ai-evaluation`);
  if (res.status === 404) return null;
  if (!res.ok) throw new Error(`AI evaluation request failed: ${res.status}`);
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
  const res = await fetch(url);
  if (!res.ok) throw new Error(`Failed to load projects: ${res.status}`);
  return res.json();
}

export async function uploadProject(name: string, category: string, file: File): Promise<Project> {
  const formData = new FormData();
  formData.append('name', name);
  formData.append('category', category);
  formData.append('file', file);

  const res = await fetch(`${API_URL}/projects/upload`, { method: 'POST', body: formData });
  if (!res.ok) throw new Error(`Failed to upload project: ${res.status} ${await res.text()}`);
  return res.json();
}

export async function updateRanking(category: string, order: number[], changedBy: string): Promise<void> {
  const res = await fetch(`${API_URL}/ranking`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ category, order, changed_by: changedBy }),
  });
  if (!res.ok) throw new Error(`Failed to update ranking: ${res.status}`);
}

export async function updateProject(
  id: number,
  update: { notes?: string; status?: ProjectStatus; review_completed?: boolean; tags?: string[] },
): Promise<Project> {
  const res = await fetch(`${API_URL}/projects/${id}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(update),
  });
  if (!res.ok) throw new Error(`Failed to update project: ${res.status}`);
  return res.json();
}

export function projectFileUrl(id: number): string {
  return `${API_URL}/projects/${id}/file`;
}

export async function getProjectDocument(id: number): Promise<Document | null> {
  const res = await fetch(`${API_URL}/projects/${id}/document`);
  if (res.status === 404) return null;
  if (!res.ok) throw new Error(`Failed to load document: ${res.status}`);
  return res.json();
}

export async function getActivity(category?: string, limit = 10): Promise<ActivityEntry[]> {
  const params = new URLSearchParams({ limit: String(limit) });
  if (category) params.set('category', category);
  const res = await fetch(`${API_URL}/activity?${params}`);
  if (!res.ok) throw new Error(`Failed to load activity: ${res.status}`);
  return res.json();
}
