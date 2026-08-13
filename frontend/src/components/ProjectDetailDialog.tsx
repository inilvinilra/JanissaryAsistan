import { useEffect, useState } from 'react';
import { Download, ExternalLink, FileText, EyeOff } from 'lucide-react';

import {
  getAiEvaluation,
  addJuryScore,
  getProjectDocument,
  getJuryScores,
  getJuryAssignments,
  getProjectMetadata,
  updateProjectMetadata,
  getProjectFiles,
  getCompetitions,
  getCompetitionStages,
  getAppeals,
  createAppeal,
  getEligibilityReport,
  uploadProjectFile,
  fetchProtectedFile,
  projectVersionFileUrl,
  updateProject,
  projectFileUrl,
  type AiEvaluation,
  type Document,
  type JuryScore,
  type JuryAssignment,
  type Project,
  type ProjectStatus,
  type ProjectMetadata,
  type ProjectFile,
  type CompetitionStage,
  type Appeal,
  type EligibilityReport,
} from '@/lib/api';
import { useLocale } from '@/lib/locale-context';
import { useToast } from '@/lib/toast-context';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Textarea } from '@/components/ui/textarea';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { AiAnalysisWorkspace } from '@/components/AiAnalysisWorkspace';
import { ProjectCopilotPanel } from '@/components/ProjectCopilotPanel';
import { chooseDesktopProjectFile, isDesktopApp, saveDesktopFile } from '@/lib/desktop';

const STATUSES: ProjectStatus[] = ['new', 'reviewing', 'finalist', 'rejected'];
const STATUS_KEYS: Record<ProjectStatus, string> = {
  new: 'statusNew',
  reviewing: 'statusReviewing',
  finalist: 'statusFinalist',
  rejected: 'statusRejected',
};

export function ProjectDetailDialog({
  project,
  open,
  onOpenChange,
  onProjectUpdated,
}: {
  project: Project | null;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onProjectUpdated: (project: Project) => void;
}) {
  const { t } = useLocale();
  const { showToast } = useToast();
  const currentRole = typeof localStorage === 'undefined' ? 'read_only' : (() => { try { return JSON.parse(localStorage.getItem('jury-auth-user') ?? '{}').role || 'read_only'; } catch { return 'read_only'; } })();
  const canViewAiAnalysis = currentRole !== 'jury_member' && currentRole !== 'observer' && currentRole !== 'read_only';
  const canRunResearch = ['system_admin', 'competition_manager', 'chief_judge'].includes(currentRole);
  const canUseCopilot = ['system_admin', 'competition_manager', 'chief_judge'].includes(currentRole);
  const authenticatedName = typeof localStorage === 'undefined' ? 'Authenticated user' : (() => { try { return JSON.parse(localStorage.getItem('jury-auth-user') ?? '{}').full_name || 'Authenticated user'; } catch { return 'Authenticated user'; } })();
  const [document, setDocument] = useState<Document | null | undefined>(undefined);
  const [aiEvaluation, setAiEvaluation] = useState<AiEvaluation | null | undefined>(undefined);
  const [juryScores, setJuryScores] = useState<JuryScore[]>([]);
  const [juryAssignments, setJuryAssignments] = useState<JuryAssignment[]>([]);
  const [notes, setNotes] = useState('');
  const [savingNotes, setSavingNotes] = useState(false);
  const [juryScore, setJuryScore] = useState('');
  const [blindReview, setBlindReview] = useState(false);
  const [appeals, setAppeals] = useState<Appeal[]>([]);
  const [appealReason, setAppealReason] = useState('');
  const [eligibility, setEligibility] = useState<EligibilityReport | null>(null);
  const [juryStageId, setJuryStageId] = useState('');
  const [evaluationStages, setEvaluationStages] = useState<CompetitionStage[]>([]);
  const [juryComment, setJuryComment] = useState('');
  const [reviewCompleted, setReviewCompleted] = useState(false);
  const [tagText, setTagText] = useState('');
  const [savingReview, setSavingReview] = useState(false);
  const [metadata, setMetadata] = useState<ProjectMetadata | null>(null);
  const [institution, setInstitution] = useState('');
  const [keywords, setKeywords] = useState('');
  const [githubUrl, setGithubUrl] = useState('');
  const [demoUrl, setDemoUrl] = useState('');
  const [prototypeDescription, setPrototypeDescription] = useState('');
  const [savingMetadata, setSavingMetadata] = useState(false);
  const [projectFiles, setProjectFiles] = useState<ProjectFile[]>([]);
  const [uploadingFile, setUploadingFile] = useState(false);
  const [versionComparison, setVersionComparison] = useState<string | null>(null);

  useEffect(() => {
    if (!open || !project) return;
    setDocument(undefined);
    setAiEvaluation(undefined);
    setNotes(project.notes);
    setReviewCompleted(project.review_completed);
    setTagText(project.tags.join(', '));
    Promise.all([getProjectDocument(project.id).catch(() => null), canViewAiAnalysis ? getAiEvaluation(project.id).catch(() => null) : Promise.resolve(null), getJuryScores(project.id).catch(() => []), getJuryAssignments(project.id).catch(() => []), getProjectMetadata(project.id).catch(() => null), getProjectFiles(project.id).catch(() => []), getCompetitions().catch(() => []), getAppeals(project.id).catch(() => []), getEligibilityReport(project.id).catch(() => null)])
      .then(async ([nextDocument, nextAiEvaluation, nextJuryScores, nextJuryAssignments, nextMetadata, nextFiles, competitions, nextAppeals, nextEligibility]) => {
        setDocument(nextDocument);
        setAiEvaluation(nextAiEvaluation);
        setJuryScores(nextJuryScores);
        setJuryAssignments(nextJuryAssignments);
        setMetadata(nextMetadata); setInstitution(nextMetadata?.institution ?? ''); setKeywords(nextMetadata?.keywords.join(', ') ?? ''); setGithubUrl(nextMetadata?.github_url ?? ''); setDemoUrl(nextMetadata?.demo_url ?? ''); setPrototypeDescription(nextMetadata?.prototype_description ?? '');
        setProjectFiles(nextFiles);
        setAppeals(nextAppeals);
        setEligibility(nextEligibility);
        const stageLists = await Promise.all(competitions.map((competition) => getCompetitionStages(competition.id)));
        setEvaluationStages(stageLists.flat());
        setVersionComparison(null);
      })
      .catch(() => {
        setDocument(null);
        setAiEvaluation(null);
      });
  }, [open, project, canViewAiAnalysis]);

  if (!project) return null;

  async function saveNotes() {
    setSavingNotes(true);
    try {
      const updated = await updateProject(project!.id, { notes });
      onProjectUpdated(updated);
      showToast(t('notesSaved'), 'success');
    } catch (e) {
      showToast((e as Error).message, 'error');
    } finally {
      setSavingNotes(false);
    }
  }

  async function changeStatus(status: ProjectStatus) {
    const updated = await updateProject(project!.id, { status });
    onProjectUpdated(updated);
  }

  async function saveReview() {
    if (!juryScore) return;
    setSavingReview(true);
    try {
      const score = await addJuryScore(project.id, {
        stage_id: juryStageId ? Number(juryStageId) : null,
        juror_name: authenticatedName,
        total_score: Number(juryScore),
        kpi_scores: [],
        notes: juryComment,
      });
      setJuryScores((items) => [score, ...items]);
      await updateProject(project.id, {
        review_completed: reviewCompleted,
        tags: tagText.split(',').map((tag) => tag.trim()).filter(Boolean),
      });
      showToast(t('juryReviewSaved'), 'success');
    } catch (e) {
      showToast((e as Error).message, 'error');
    } finally {
      setSavingReview(false);
    }
  }

  async function saveMetadata() {
    setSavingMetadata(true);
    try { const updated = await updateProjectMetadata(project.id, { institution, keywords: keywords.split(',').map((item) => item.trim()).filter(Boolean), github_url: githubUrl || null, demo_url: demoUrl || null, prototype_description: prototypeDescription, team_name: metadata?.team_name ?? '', team_members: metadata?.team_members ?? [] }); setMetadata(updated); showToast(t('metadataSaved'), 'success'); }
    catch (e) { showToast((e as Error).message, 'error'); }
    finally { setSavingMetadata(false); }
  }

  async function submitAppeal() { if (!appealReason.trim()) return; try { const appeal = await createAppeal(project.id, { submitted_by: authenticatedName, reason: appealReason.trim(), committee: [] }); setAppeals((items) => [appeal, ...items]); setAppealReason(''); showToast('Appeal recorded.', 'success'); } catch (error) { showToast((error as Error).message, 'error'); } }

  async function uploadFile(file: File | undefined) {
    if (!file) return;
    setUploadingFile(true);
    try { const uploaded = await uploadProjectFile(project.id, file); setProjectFiles((items) => [uploaded, ...items]); showToast(t('fileUploaded'), 'success'); }
    catch (e) { showToast((e as Error).message, 'error'); }
    finally { setUploadingFile(false); }
  }

  async function chooseAndUploadFile() {
    const file = await chooseDesktopProjectFile();
    if (file) await uploadFile(file);
  }

  async function compareLatestVersions() {
    if (projectFiles.length < 2) return;
    const [latest, previous] = projectFiles;
    const textLike = /\.(txt|md|markdown|csv)$/i.test(latest.file_name) && /\.(txt|md|markdown|csv)$/i.test(previous.file_name);
    if (!textLike) { setVersionComparison(`${t('binaryVersionComparison')}: v${previous.version} (${Math.ceil(previous.size_bytes / 1024)} KB) → v${latest.version} (${Math.ceil(latest.size_bytes / 1024)} KB)`); return; }
    try { const [newText, oldText] = await Promise.all([fetchProtectedFile(projectVersionFileUrl(project.id, latest.id)).then((response) => response.text()), fetchProtectedFile(projectVersionFileUrl(project.id, previous.id)).then((response) => response.text())]); const oldLines = oldText.split(/\r?\n/); const newLines = newText.split(/\r?\n/); const changed = newLines.reduce((count, line, index) => count + (line !== oldLines[index] ? 1 : 0), 0) + Math.max(0, oldLines.length - newLines.length); setVersionComparison(t('textVersionComparison', { old: String(previous.version), next: String(latest.version), changed: String(changed) })); } catch { setVersionComparison(t('versionComparisonFailed')); }
  }

  async function openProtectedFile(url: string, download = false) {
    try {
      const response = await fetchProtectedFile(url);
      const blob = await response.blob();
      if (download && await saveDesktopFile(blob, 'project-file')) return;
      const blobUrl = URL.createObjectURL(blob);
      const link = window.document.createElement('a');
      link.href = blobUrl;
      if (download) link.download = 'proje-dosyasi'; else link.target = '_blank';
      link.click();
      window.setTimeout(() => URL.revokeObjectURL(blobUrl), 60_000);
    } catch (error) { showToast((error as Error).message, 'error'); }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[88vh] overflow-y-auto sm:max-w-4xl">
        <DialogHeader>
          <div className="flex items-center justify-between gap-3"><DialogTitle>{blindReview ? `PRJ-${String(project.id).padStart(6, '0')}` : project.name}</DialogTitle><Button type="button" size="sm" variant="outline" onClick={() => setBlindReview((value) => !value)}><EyeOff className="mr-1.5 size-3.5" />{blindReview ? t('showIdentity') : t('enableBlindReview')}</Button></div>
          <DialogDescription>{t('detailTitle')}</DialogDescription>
        </DialogHeader>

        <div className="space-y-4 text-sm">
          <div className="flex flex-wrap items-center gap-3">
            <Select value={project.status} onValueChange={(v) => changeStatus(v as ProjectStatus)}>
              <SelectTrigger className="h-8 w-36 text-xs">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {STATUSES.map((s) => (
                  <SelectItem key={s} value={s}>
                    {t(STATUS_KEYS[s])}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            {project.has_file ? (
              <div className="flex items-center gap-2">
                <button type="button" onClick={() => void openProtectedFile(projectFileUrl(project.id))} className="text-primary flex items-center gap-1 text-xs hover:underline">
                  <ExternalLink className="size-3.5" />
                  {t('viewFile')}
                </button>
                <button type="button" onClick={() => void openProtectedFile(projectFileUrl(project.id), true)} className="text-muted-foreground flex items-center gap-1 text-xs hover:text-foreground">
                  <Download className="size-3.5" />
                  {t('downloadFile')}
                </button>
              </div>
            ) : (
              <span className="text-muted-foreground text-xs">{t('noFile')}</span>
            )}
          </div>

          {canViewAiAnalysis && <AiAnalysisWorkspace projectId={project.id} evaluation={aiEvaluation} juryScores={juryScores} canRunResearch={canRunResearch} />}
          {canUseCopilot && <ProjectCopilotPanel projectId={project.id} />}

          <div>
            <p className="mb-1.5 text-xs font-semibold tracking-wide uppercase text-muted-foreground">
              {t('notesLabel')}
            </p>
            <Textarea
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              placeholder={t('notesPlaceholder')}
              className="min-h-20"
            />
            <div className="mt-2 flex justify-end">
              <Button size="sm" onClick={saveNotes} disabled={savingNotes || notes === project.notes}>
                {savingNotes ? t('saving') : t('save')}
              </Button>
            </div>
          </div>

          {!blindReview && <div className="space-y-3 rounded-lg border p-3">
            <p className="text-xs font-semibold tracking-wide uppercase text-muted-foreground">{t('projectMetadata')}</p>
            <div className="grid gap-2 sm:grid-cols-2"><Input value={institution} onChange={(e) => setInstitution(e.target.value)} placeholder={t('institutionPlaceholder')} /><Input value={keywords} onChange={(e) => setKeywords(e.target.value)} placeholder={t('keywordsPlaceholder')} /></div>
            <div className="grid gap-2 sm:grid-cols-2"><Input type="url" value={githubUrl} onChange={(e) => setGithubUrl(e.target.value)} placeholder="GitHub URL" /><Input type="url" value={demoUrl} onChange={(e) => setDemoUrl(e.target.value)} placeholder={t('demoUrlPlaceholder')} /></div>
            <div className="grid gap-2 sm:grid-cols-2"><Input value={metadata?.team_name ?? ''} onChange={(e) => setMetadata((prev) => prev ? { ...prev, team_name: e.target.value } : prev)} placeholder={t('teamNamePlaceholder')} /><Input value={(metadata?.team_members ?? []).join(', ')} onChange={(e) => setMetadata((prev) => prev ? { ...prev, team_members: e.target.value.split(',').map((item) => item.trim()).filter(Boolean) } : prev)} placeholder={t('teamMembersPlaceholder')} /></div>
            <Textarea value={prototypeDescription} onChange={(e) => setPrototypeDescription(e.target.value)} placeholder={t('prototypePlaceholder')} className="min-h-16" />
            <div className="flex justify-end"><Button size="sm" onClick={saveMetadata} disabled={savingMetadata}>{savingMetadata ? t('saving') : t('saveMetadata')}</Button></div>
          </div>}

          <div className="space-y-2 rounded-lg border p-3"><p className="text-xs font-semibold tracking-wide uppercase text-muted-foreground">{t('appealsTitle')}</p><Textarea value={appealReason} onChange={(e) => setAppealReason(e.target.value)} placeholder={t('appealReasonPlaceholder')} className="min-h-16" /><Button size="sm" variant="outline" onClick={() => void submitAppeal()} disabled={!appealReason.trim()}>{t('saveAppeal')}</Button>{appeals.map((appeal) => <div key={appeal.id} className="rounded-md border bg-background p-2 text-xs"><div className="flex justify-between"><span className="font-medium">{appeal.status}</span><span>{new Date(appeal.created_at).toLocaleDateString()}</span></div><p className="mt-1 text-muted-foreground">{appeal.reason}</p>{appeal.decision_reason && <p className="mt-1 text-primary">{t('decisionLabel')}: {appeal.decision_reason}</p>}</div>)}</div>

          {eligibility && <div className="space-y-2 rounded-lg border p-3"><p className="text-xs font-semibold tracking-wide uppercase text-muted-foreground">{t('eligibilityTitle')} · <span className={eligibility.eligible ? 'text-primary' : 'text-destructive'}>{eligibility.eligible ? t('eligible') : t('reviewRequired')}</span></p>{eligibility.checks.map((check) => <div key={check.key} className="flex items-start justify-between gap-3 text-xs"><span>{check.label}<span className="mt-0.5 block text-muted-foreground">{check.detail}</span></span><span className={check.passed ? 'text-primary' : 'text-destructive'}>{check.passed ? t('passed') : t('missing')}</span></div>)}</div>}

          <div className="space-y-3 rounded-lg border p-3">
            <div className="flex items-center justify-between"><p className="text-xs font-semibold tracking-wide uppercase text-muted-foreground">{t('projectFilesTitle')}</p>{isDesktopApp() ? <button type="button" className="cursor-pointer rounded-md border px-3 py-1.5 text-xs hover:bg-accent disabled:cursor-not-allowed" disabled={uploadingFile} onClick={() => void chooseAndUploadFile()}>{uploadingFile ? t('uploading') : t('uploadFile')}</button> : <label className="cursor-pointer rounded-md border px-3 py-1.5 text-xs hover:bg-accent">{uploadingFile ? t('uploading') : t('uploadFile')}<input type="file" className="hidden" accept=".pdf,.txt,.md,.markdown,.doc,.docx,.xls,.xlsx,.csv,.png,.jpg,.jpeg,.webp" disabled={uploadingFile} onChange={(e) => { void uploadFile(e.target.files?.[0]); e.currentTarget.value = ''; }} /></label>}</div>
            <p className="text-muted-foreground text-[11px]">{t('fileVersionDescription')}</p>
            <div className="space-y-1.5">{projectFiles.map((file) => <div key={file.id} className="flex items-center justify-between gap-2 rounded-md border bg-background px-3 py-2 text-xs"><span className="truncate"><strong>v{file.version}</strong> · {file.file_name} <span className="text-muted-foreground">({Math.ceil(file.size_bytes / 1024)} KB · {new Date(file.uploaded_at).toLocaleString()})</span></span><button type="button" className="text-primary shrink-0 hover:underline" onClick={() => void openProtectedFile(projectVersionFileUrl(project.id, file.id))}>{t('viewFile')}</button></div>)}{projectFiles.length === 0 && <p className="text-muted-foreground text-xs">{t('noProjectFiles')}</p>}</div>
            {projectFiles.length > 1 && <><Button variant="outline" size="sm" onClick={() => void compareLatestVersions()}>{t('compareVersions')}</Button>{versionComparison && <p className="rounded-md bg-secondary/60 p-2 text-xs">{versionComparison}</p>}</>}
          </div>

          <div className="space-y-3 rounded-lg border p-3">
            <div className="flex items-center justify-between">
              <p className="text-xs font-semibold tracking-wide uppercase text-muted-foreground">{t('juryReviewTitle')}</p>
              <label className="flex items-center gap-2 text-xs text-muted-foreground"><input type="checkbox" checked={reviewCompleted} onChange={(e) => setReviewCompleted(e.target.checked)} />{t('reviewCompleted')}</label>
            </div>
            <div className="grid gap-2 sm:grid-cols-[120px_1fr]">
              <Input type="number" min="0" max="100" step="0.1" value={juryScore} onChange={(e) => setJuryScore(e.target.value)} placeholder={t('juryScorePlaceholder')} />
              <Textarea value={juryComment} onChange={(e) => setJuryComment(e.target.value)} placeholder={t('juryCommentPlaceholder')} className="min-h-16" />
            </div>
            <select value={juryStageId} onChange={(e) => setJuryStageId(e.target.value)} className="h-9 w-full rounded-md border bg-background px-3 text-sm"><option value="">{t('generalScore')}</option>{evaluationStages.map((stage) => <option key={stage.id} value={stage.id}>{t('stageScore')}: {stage.name}</option>)}</select>
            <Input value={tagText} onChange={(e) => setTagText(e.target.value)} placeholder={t('tagsPlaceholder')} />
            <Button size="sm" onClick={saveReview} disabled={savingReview || !juryScore}>{savingReview ? t('saving') : t('submitJuryReview')}</Button>
          </div>

          {document === undefined && <p className="text-muted-foreground text-sm">{t('loading')}</p>}

          {juryAssignments.length > 0 && (
            <div className="rounded-lg border p-3">
              <p className="mb-2 text-xs font-semibold tracking-wide uppercase text-muted-foreground">{t('assignedJurors')}</p>
              <div className="space-y-1.5">
                {juryAssignments.map((assignment) => (
                  <div key={assignment.id} className="flex items-center justify-between text-xs">
                    <span className="font-medium">{assignment.juror_name}</span>
                    <span className="text-muted-foreground">{assignment.role} · {assignment.status}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {juryScores.length > 0 && (
            <div className="rounded-lg border p-3">
              <p className="mb-2 text-xs font-semibold tracking-wide uppercase text-muted-foreground">{t('juryScoresTitle')}</p>
              <div className="space-y-2">
                {juryScores.map((score, index) => (
                  <div key={score.id} className="rounded-md border bg-background p-2 text-xs">
                    <div className="flex items-center justify-between"><span className="font-medium">{blindReview ? t('blindJuror', { number: String(index + 1) }) : score.juror_name}</span><span className="font-data font-semibold">{score.total_score.toFixed(1)}</span></div>
                    {score.stage_id && <p className="text-primary mt-1">{t('stageScore')}: {evaluationStages.find((stage) => stage.id === score.stage_id)?.name ?? `#${score.stage_id}`}</p>}
                    <p className="text-muted-foreground mt-1">{score.notes || t('noJuryComment')}</p>
                  </div>
                ))}
              </div>
              {juryScores.length > 1 && <p className="text-muted-foreground mt-2 text-xs">{t('jurySpread')}: <span className="font-data font-semibold">{(Math.max(...juryScores.map((score) => score.total_score)) - Math.min(...juryScores.map((score) => score.total_score))).toFixed(1)}</span></p>}
            </div>
          )}

          {document === null && (
            <p className="text-muted-foreground flex items-center gap-2 text-sm">
              <FileText className="size-4" />
              {t('detailNoDocument')}
            </p>
          )}

          {document && (
            <>
              <div className="flex flex-wrap gap-4 text-xs">
                <span className="text-muted-foreground">
                  {t('detailLanguage')}: <span className="font-medium text-foreground">{document.language}</span>
                </span>
                <span className="text-muted-foreground">
                  {t('detailWordCount')}:{' '}
                  <span className="font-data font-medium text-foreground">{document.word_count}</span>
                </span>
              </div>

              {document.headings.length > 0 && (
                <div>
                  <p className="mb-1.5 text-xs font-semibold tracking-wide uppercase text-muted-foreground">
                    {t('detailHeadings')}
                  </p>
                  <ul className="list-inside list-disc space-y-0.5">
                    {document.headings.map((h) => (
                      <li key={h}>{h}</li>
                    ))}
                  </ul>
                </div>
              )}

              {document.keywords.length > 0 && (
                <div>
                  <p className="mb-1.5 text-xs font-semibold tracking-wide uppercase text-muted-foreground">
                    {t('detailKeywords')}
                  </p>
                  <div className="flex flex-wrap gap-1.5">
                    {document.keywords.slice(0, 15).map((k) => (
                      <span
                        key={k}
                        className="rounded-full bg-secondary px-2 py-0.5 text-xs text-secondary-foreground"
                      >
                        {k}
                      </span>
                    ))}
                  </div>
                </div>
              )}

              {document.sections.length > 0 && (
                <div>
                  <p className="mb-1.5 text-xs font-semibold tracking-wide uppercase text-muted-foreground">
                    {t('detailSections')}
                  </p>
                  <div className="space-y-2">
                    {document.sections.map((s) => (
                      <div key={s.title} className="rounded-md border p-2.5">
                        <p className="font-medium">{s.title.replace(/^#+\s*/, '')}</p>
                        <p className="text-muted-foreground mt-0.5 line-clamp-2 text-xs">{s.content}</p>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {document.references.length > 0 && (
                <div>
                  <p className="mb-1.5 text-xs font-semibold tracking-wide uppercase text-muted-foreground">
                    {t('detailReferences')}
                  </p>
                  <ul className="font-data list-inside list-disc space-y-0.5 text-xs">
                    {document.references.slice(0, 10).map((r) => (
                      <li key={r} className="truncate">
                        {r}
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
