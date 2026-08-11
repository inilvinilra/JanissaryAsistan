import { useEffect, useState } from 'react';
import { Download, ExternalLink, FileText } from 'lucide-react';

import {
  getAiEvaluation,
  addJuryScore,
  getProjectDocument,
  getJuryScores,
  getJuryAssignments,
  getProjectMetadata,
  updateProjectMetadata,
  getProjectFiles,
  uploadProjectFile,
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
} from '@/lib/api';
import { useLocale } from '@/lib/locale-context';
import { useToast } from '@/lib/toast-context';
import { useJuror } from '@/lib/juror-context';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Textarea } from '@/components/ui/textarea';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';

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
  const { jurorName } = useJuror();
  const [document, setDocument] = useState<Document | null | undefined>(undefined);
  const [aiEvaluation, setAiEvaluation] = useState<AiEvaluation | null | undefined>(undefined);
  const [juryScores, setJuryScores] = useState<JuryScore[]>([]);
  const [juryAssignments, setJuryAssignments] = useState<JuryAssignment[]>([]);
  const [notes, setNotes] = useState('');
  const [savingNotes, setSavingNotes] = useState(false);
  const [juryScore, setJuryScore] = useState('');
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
    Promise.all([getProjectDocument(project.id), getAiEvaluation(project.id), getJuryScores(project.id), getJuryAssignments(project.id), getProjectMetadata(project.id), getProjectFiles(project.id)])
      .then(([nextDocument, nextAiEvaluation, nextJuryScores, nextJuryAssignments, nextMetadata, nextFiles]) => {
        setDocument(nextDocument);
        setAiEvaluation(nextAiEvaluation);
        setJuryScores(nextJuryScores);
        setJuryAssignments(nextJuryAssignments);
        setMetadata(nextMetadata); setInstitution(nextMetadata.institution); setKeywords(nextMetadata.keywords.join(', ')); setGithubUrl(nextMetadata.github_url ?? ''); setDemoUrl(nextMetadata.demo_url ?? ''); setPrototypeDescription(nextMetadata.prototype_description);
        setProjectFiles(nextFiles);
        setVersionComparison(null);
      })
      .catch(() => {
        setDocument(null);
        setAiEvaluation(null);
      });
  }, [open, project]);

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
    if (!jurorName.trim() || !juryScore) return;
    setSavingReview(true);
    try {
      const score = await addJuryScore(project.id, {
        juror_name: jurorName.trim(),
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

  async function uploadFile(file: File | undefined) {
    if (!file) return;
    setUploadingFile(true);
    try { const uploaded = await uploadProjectFile(project.id, file); setProjectFiles((items) => [uploaded, ...items]); showToast(t('fileUploaded'), 'success'); }
    catch (e) { showToast((e as Error).message, 'error'); }
    finally { setUploadingFile(false); }
  }

  async function compareLatestVersions() {
    if (projectFiles.length < 2) return;
    const [latest, previous] = projectFiles;
    const textLike = /\.(txt|md|markdown|csv)$/i.test(latest.file_name) && /\.(txt|md|markdown|csv)$/i.test(previous.file_name);
    if (!textLike) { setVersionComparison(`${t('binaryVersionComparison')}: v${previous.version} (${Math.ceil(previous.size_bytes / 1024)} KB) → v${latest.version} (${Math.ceil(latest.size_bytes / 1024)} KB)`); return; }
    try { const [newText, oldText] = await Promise.all([fetch(projectVersionFileUrl(project.id, latest.id)).then((response) => response.text()), fetch(projectVersionFileUrl(project.id, previous.id)).then((response) => response.text())]); const oldLines = oldText.split(/\r?\n/); const newLines = newText.split(/\r?\n/); const changed = newLines.reduce((count, line, index) => count + (line !== oldLines[index] ? 1 : 0), 0) + Math.max(0, oldLines.length - newLines.length); setVersionComparison(t('textVersionComparison', { old: String(previous.version), next: String(latest.version), changed: String(changed) })); } catch { setVersionComparison(t('versionComparisonFailed')); }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[85vh] overflow-y-auto sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>{project.name}</DialogTitle>
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
                <a
                  href={projectFileUrl(project.id)}
                  target="_blank"
                  rel="noreferrer"
                  className="text-primary flex items-center gap-1 text-xs hover:underline"
                >
                  <ExternalLink className="size-3.5" />
                  {t('viewFile')}
                </a>
                <a
                  href={projectFileUrl(project.id)}
                  download
                  className="text-muted-foreground flex items-center gap-1 text-xs hover:text-foreground"
                >
                  <Download className="size-3.5" />
                  {t('downloadFile')}
                </a>
              </div>
            ) : (
              <span className="text-muted-foreground text-xs">{t('noFile')}</span>
            )}
          </div>

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

          <div className="space-y-3 rounded-lg border p-3">
            <p className="text-xs font-semibold tracking-wide uppercase text-muted-foreground">{t('projectMetadata')}</p>
            <div className="grid gap-2 sm:grid-cols-2"><Input value={institution} onChange={(e) => setInstitution(e.target.value)} placeholder={t('institutionPlaceholder')} /><Input value={keywords} onChange={(e) => setKeywords(e.target.value)} placeholder={t('keywordsPlaceholder')} /></div>
            <div className="grid gap-2 sm:grid-cols-2"><Input type="url" value={githubUrl} onChange={(e) => setGithubUrl(e.target.value)} placeholder="GitHub URL" /><Input type="url" value={demoUrl} onChange={(e) => setDemoUrl(e.target.value)} placeholder={t('demoUrlPlaceholder')} /></div>
            <div className="grid gap-2 sm:grid-cols-2"><Input value={metadata?.team_name ?? ''} onChange={(e) => setMetadata((prev) => prev ? { ...prev, team_name: e.target.value } : prev)} placeholder={t('teamNamePlaceholder')} /><Input value={(metadata?.team_members ?? []).join(', ')} onChange={(e) => setMetadata((prev) => prev ? { ...prev, team_members: e.target.value.split(',').map((item) => item.trim()).filter(Boolean) } : prev)} placeholder={t('teamMembersPlaceholder')} /></div>
            <Textarea value={prototypeDescription} onChange={(e) => setPrototypeDescription(e.target.value)} placeholder={t('prototypePlaceholder')} className="min-h-16" />
            <div className="flex justify-end"><Button size="sm" onClick={saveMetadata} disabled={savingMetadata}>{savingMetadata ? t('saving') : t('saveMetadata')}</Button></div>
          </div>

          <div className="space-y-3 rounded-lg border p-3">
            <div className="flex items-center justify-between"><p className="text-xs font-semibold tracking-wide uppercase text-muted-foreground">{t('projectFilesTitle')}</p><label className="cursor-pointer rounded-md border px-3 py-1.5 text-xs hover:bg-accent">{uploadingFile ? t('uploading') : t('uploadFile')}<input type="file" className="hidden" accept=".pdf,.txt,.md,.markdown,.doc,.docx,.xls,.xlsx,.csv,.png,.jpg,.jpeg,.webp" disabled={uploadingFile} onChange={(e) => { void uploadFile(e.target.files?.[0]); e.currentTarget.value = ''; }} /></label></div>
            <p className="text-muted-foreground text-[11px]">{t('fileVersionDescription')}</p>
            <div className="space-y-1.5">{projectFiles.map((file) => <div key={file.id} className="flex items-center justify-between gap-2 rounded-md border bg-background px-3 py-2 text-xs"><span className="truncate"><strong>v{file.version}</strong> · {file.file_name} <span className="text-muted-foreground">({Math.ceil(file.size_bytes / 1024)} KB · {new Date(file.uploaded_at).toLocaleString()})</span></span><a className="text-primary shrink-0 hover:underline" href={projectVersionFileUrl(project.id, file.id)} target="_blank" rel="noreferrer">{t('viewFile')}</a></div>)}{projectFiles.length === 0 && <p className="text-muted-foreground text-xs">{t('noProjectFiles')}</p>}</div>
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
            <Input value={tagText} onChange={(e) => setTagText(e.target.value)} placeholder={t('tagsPlaceholder')} />
            <Button size="sm" onClick={saveReview} disabled={savingReview || !jurorName.trim() || !juryScore}>{savingReview ? t('saving') : t('submitJuryReview')}</Button>
          </div>

          {document === undefined && <p className="text-muted-foreground text-sm">{t('loading')}</p>}

          {aiEvaluation && (
            <div className="space-y-3 rounded-lg border border-primary/20 bg-primary/[0.03] p-3">
              <div className="flex items-center justify-between">
                <p className="text-xs font-semibold tracking-wide uppercase text-muted-foreground">{t('aiEvaluationTitle')}</p>
                <span className="text-muted-foreground text-[11px]">{aiEvaluation.model_version} · {t('aiConfidence')} %{Math.round(aiEvaluation.confidence * 100)}</span>
              </div>
              <div className="grid grid-cols-2 gap-2">
                <div className="rounded-md bg-background p-2"><p className="text-muted-foreground text-[10px] uppercase">{t('aiScoreLabel')}</p><p className="font-data text-xl font-bold">{aiEvaluation.total_score.toFixed(1)}</p></div>
                <div className="rounded-md bg-background p-2"><p className="text-muted-foreground text-[10px] uppercase">{t('juryAverageLabel')}</p><p className="font-data text-xl font-bold">{juryScores.length ? (juryScores.reduce((sum, score) => sum + score.total_score, 0) / juryScores.length).toFixed(1) : '—'}</p></div>
              </div>
              {juryScores.length > 0 && <p className="text-muted-foreground text-xs">{t('aiJuryDifference')}: <span className="font-data font-semibold">{(aiEvaluation.total_score - juryScores.reduce((sum, score) => sum + score.total_score, 0) / juryScores.length).toFixed(1)}</span></p>}
              {aiEvaluation.kpi_scores.map((kpi) => (
                <div key={kpi.name} className="rounded-md border bg-background p-2.5">
                  <div className="flex items-center justify-between text-xs font-medium"><span>{kpi.name}</span><span className="font-data">{kpi.score.toFixed(0)}/100</span></div>
                  <p className="text-muted-foreground mt-1 text-xs">{kpi.reason}</p>
                  {kpi.evidence.length > 0 && <p className="text-primary mt-1 text-[11px]">{t('aiEvidence')}: {kpi.evidence.join(', ')}</p>}
                </div>
              ))}
              <div className="grid gap-2 text-xs sm:grid-cols-2">
                <div><p className="font-semibold text-emerald-700">{t('aiStrengths')}</p><ul className="mt-1 list-inside list-disc">{aiEvaluation.strengths.map((item) => <li key={item}>{item}</li>)}</ul></div>
                <div><p className="font-semibold text-destructive">{t('aiWeaknessesRisks')}</p><ul className="mt-1 list-inside list-disc">{[...aiEvaluation.weaknesses, ...aiEvaluation.risks].map((item) => <li key={item}>{item}</li>)}</ul></div>
              </div>
              {aiEvaluation.missing_information.length > 0 && <div className="rounded-md border border-amber-500/30 bg-amber-500/5 p-2.5 text-xs"><p className="font-semibold text-amber-700">{t('aiMissingInformation')}</p><ul className="mt-1 list-inside list-disc">{aiEvaluation.missing_information.map((item) => <li key={item}>{item}</li>)}</ul></div>}
              {aiEvaluation.sources.length > 0 && <div className="text-xs"><p className="font-semibold">{t('aiSources')}</p><ul className="text-primary mt-1 list-inside list-disc">{aiEvaluation.sources.map((source) => <li key={source} className="truncate">{source}</li>)}</ul></div>}
              {aiEvaluation.similar_projects.length > 0 && <div className="text-xs"><p className="font-semibold">{t('aiSimilarProjects')}</p><div className="mt-1 space-y-1.5">{aiEvaluation.similar_projects.map((similar) => <div key={`${similar.project_id ?? similar.name}-${similar.similarity}`} className="rounded-md border bg-background p-2"><div className="flex items-center justify-between"><span className="font-medium">{similar.name}</span><span className="font-data text-muted-foreground">{Math.round(similar.similarity * 100)}%</span></div><p className="text-muted-foreground mt-0.5">{similar.reason}</p></div>)}</div></div>}
              <p className="text-muted-foreground text-[11px]">{t('aiEvaluatedAt')}: {new Date(aiEvaluation.evaluated_at).toLocaleString()}</p>
            </div>
          )}

          {juryAssignments.length > 0 && (
            <div className="rounded-lg border p-3">
              <p className="mb-2 text-xs font-semibold tracking-wide uppercase text-muted-foreground">Atanan jüri üyeleri</p>
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
                {juryScores.map((score) => (
                  <div key={score.id} className="rounded-md border bg-background p-2 text-xs">
                    <div className="flex items-center justify-between"><span className="font-medium">{score.juror_name}</span><span className="font-data font-semibold">{score.total_score.toFixed(1)}</span></div>
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
