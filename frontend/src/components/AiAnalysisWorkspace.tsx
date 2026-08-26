import { useEffect, useMemo, useState } from 'react';
import { BrainCircuit, ExternalLink, RefreshCw, ShieldAlert, Sparkles } from 'lucide-react';

import { getProjectResearch, runCriterionEvaluation, runProjectResearch, type AiEvaluation, type JuryScore, type ProjectResearchAnalysis } from '@/lib/api';
import { useLocale } from '@/lib/locale-context';
import { useToast } from '@/lib/toast-context';
import { Button } from '@/components/ui/button';

function ScoreMeter({ value }: { value: number }) {
  const color = value >= 75 ? 'bg-emerald-500' : value >= 55 ? 'bg-amber-500' : 'bg-rose-500';
  return <div className="mt-2 h-1.5 overflow-hidden rounded-full bg-muted"><div className={`h-full rounded-full ${color}`} style={{ width: `${Math.max(0, Math.min(100, value))}%` }} /></div>;
}

function Findings({ title, items, tone }: { title: string; items: string[]; tone: 'positive' | 'warning' | 'neutral' }) {
  if (items.length === 0) return null;
  const toneClass = tone === 'positive' ? 'border-emerald-500/20 bg-emerald-500/[0.05]' : tone === 'warning' ? 'border-amber-500/25 bg-amber-500/[0.05]' : 'border-border bg-muted/30';
  return <section className={`rounded-xl border p-3 ${toneClass}`}><p className="text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">{title}</p><ul className="mt-2 space-y-1 text-xs leading-relaxed">{items.slice(0, 5).map((item) => <li key={item} className="flex gap-2"><span className="text-muted-foreground">•</span><span>{item}</span></li>)}</ul></section>;
}

function ResearchInsights({ research }: { research: ProjectResearchAnalysis }) {
  const { t } = useLocale();
  const highMatches = research.sources.filter((source) => source.similarity >= 0.4).length;

  return (
    <div className="grid gap-2 sm:grid-cols-[1fr_auto]">
      <div className="rounded-lg border bg-muted/[0.22] p-3">
        <p className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">{t('researchTerms')}</p>
        <div className="mt-2 flex flex-wrap gap-1.5">
          {research.query_terms.length
            ? research.query_terms.slice(0, 12).map((term) => <span key={term} className="rounded-full border bg-background px-2 py-1 text-[11px]">{term}</span>)
            : <span className="text-xs text-muted-foreground">{t('researchTermsUnavailable')}</span>}
        </div>
      </div>
      <div className="grid grid-cols-2 gap-2">
        <ResearchStat value={research.sources.length} label={t('researchSources')} />
        <ResearchStat value={highMatches} label={t('researchMatches')} />
      </div>
    </div>
  );
}

function ResearchStat({ value, label }: { value: number; label: string }) {
  return <div className="rounded-lg border p-3 text-center"><p className="font-data text-xl font-semibold">{value}</p><p className="mt-1 text-[10px] uppercase tracking-wide text-muted-foreground">{label}</p></div>;
}

export function AiAnalysisWorkspace({ projectId, evaluation, juryScores, canRunResearch, onEvaluationRun }: { projectId: number; evaluation: AiEvaluation | null | undefined; juryScores: JuryScore[]; canRunResearch: boolean; onEvaluationRun?: () => Promise<void> }) {
  const { t } = useLocale();
  const { showToast } = useToast();
  const [research, setResearch] = useState<ProjectResearchAnalysis | null>(null);
  const [loadingResearch, setLoadingResearch] = useState(false);
  const [runningEvaluation, setRunningEvaluation] = useState(false);
  const juryAverage = useMemo(() => juryScores.length ? juryScores.reduce((total, score) => total + score.total_score, 0) / juryScores.length : null, [juryScores]);

  useEffect(() => {
    if (!canRunResearch) return;
    getProjectResearch(projectId).then(setResearch).catch(() => setResearch(null));
  }, [canRunResearch, projectId]);

  async function refreshResearch() {
    setLoadingResearch(true);
    try {
      setResearch(await runProjectResearch(projectId, Boolean(research)));
      showToast(t('researchCompleted'), 'success');
    } catch (error) {
      showToast((error as Error).message, 'error');
    } finally {
      setLoadingResearch(false);
    }
  }

  async function runEvaluation() {
    if (!onEvaluationRun) return;
    setRunningEvaluation(true);
    try {
      await runCriterionEvaluation(projectId);
      await onEvaluationRun();
      showToast(t('aiEvaluationCompleted'), 'success');
    } catch (error) {
      showToast((error as Error).message, 'error');
    } finally {
      setRunningEvaluation(false);
    }
  }

  if (evaluation === undefined) return <div className="rounded-xl border border-dashed p-4 text-sm text-muted-foreground">{t('loading')}</div>;
  // No stored evaluation yet. For a role that may start one this is an action,
  // not a dead end — the judge should not have to leave the AI panel to run it.
  if (!evaluation) return <div className="space-y-3 rounded-xl border border-dashed p-4">
    <p className="text-sm text-muted-foreground">{t('aiAnalysisUnavailable')}</p>
    {onEvaluationRun && <Button size="sm" variant="outline" disabled={runningEvaluation} onClick={() => void runEvaluation()}>
      <BrainCircuit className={`mr-1.5 size-3.5 ${runningEvaluation ? 'animate-pulse' : ''}`} />
      {runningEvaluation ? t('aiEvaluationRunning') : t('runAiEvaluation')}
    </Button>}
  </div>;

  return (
    <section className="space-y-4 rounded-2xl border border-primary/20 bg-gradient-to-br from-primary/[0.07] via-background to-background p-4 shadow-sm">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="flex items-start gap-3"><span className="rounded-xl bg-primary/10 p-2 text-primary"><BrainCircuit className="size-5" /></span><div><p className="font-semibold">{t('aiAnalysisWorkspace')}</p><p className="mt-0.5 text-xs text-muted-foreground">{evaluation.model_version} · {t('aiEvaluatedAt')} {new Date(evaluation.evaluated_at).toLocaleString()}</p></div></div>
        <div className="flex items-center gap-2"><div className="rounded-lg border bg-background px-3 py-1.5 text-right"><p className="text-[10px] uppercase tracking-wide text-muted-foreground">{t('aiConfidence')}</p><p className="font-data text-sm font-semibold">{Math.round(evaluation.confidence * 100)}%</p></div>{onEvaluationRun && <Button size="sm" variant="outline" disabled={runningEvaluation} onClick={() => void runEvaluation()}><RefreshCw className={`mr-1.5 size-3.5 ${runningEvaluation ? 'animate-spin' : ''}`} />{runningEvaluation ? t('aiEvaluationRunning') : t('rerunAiEvaluation')}</Button>}</div>
      </div>

      <div className="grid gap-3 sm:grid-cols-3">
        <div className="rounded-xl border bg-background p-3"><p className="text-[10px] font-medium uppercase tracking-wide text-muted-foreground">{t('aiScoreLabel')}</p><p className="font-data mt-1 text-3xl font-semibold tracking-tight">{evaluation.total_score.toFixed(1)}<span className="ml-1 text-sm text-muted-foreground">/100</span></p><ScoreMeter value={evaluation.total_score} /></div>
        <div className="rounded-xl border bg-background p-3"><p className="text-[10px] font-medium uppercase tracking-wide text-muted-foreground">{t('juryAverageLabel')}</p><p className="font-data mt-1 text-3xl font-semibold tracking-tight">{juryAverage?.toFixed(1) ?? '—'}</p><p className="mt-2 text-xs text-muted-foreground">{juryAverage === null ? t('noJuryScoreYet') : `${t('aiJuryDifference')}: ${(evaluation.total_score - juryAverage).toFixed(1)}`}</p></div>
        <div className="rounded-xl border bg-background p-3"><p className="text-[10px] font-medium uppercase tracking-wide text-muted-foreground">{t('aiEvidenceReadiness')}</p><p className="font-data mt-1 text-3xl font-semibold tracking-tight">{evaluation.kpi_scores.filter((item) => item.evidence.length > 0).length}<span className="ml-1 text-sm text-muted-foreground">/{evaluation.kpi_scores.length}</span></p><p className="mt-2 text-xs text-muted-foreground">{t('kpiEvidenceAvailable')}</p></div>
      </div>

      {evaluation.confidence < 0.6 && <div className="flex gap-2 rounded-xl border border-amber-500/30 bg-amber-500/[0.08] p-3 text-xs text-amber-900 dark:text-amber-200"><ShieldAlert className="mt-0.5 size-4 shrink-0" />{t('lowAiConfidence')}</div>}

      <div className="grid gap-2 md:grid-cols-2">
        {evaluation.kpi_scores.map((kpi) => <article key={kpi.name} className="rounded-xl border bg-background p-3"><div className="flex items-start justify-between gap-3"><div><p className="text-sm font-medium">{kpi.name}</p><p className="mt-1 text-xs leading-relaxed text-muted-foreground">{kpi.reason}</p></div><span className="font-data shrink-0 rounded-md bg-muted px-2 py-1 text-xs font-semibold">{kpi.score.toFixed(0)}</span></div><ScoreMeter value={kpi.score} />{kpi.evidence.length > 0 && <p className="mt-2 text-[11px] text-primary">{t('aiEvidence')}: {kpi.evidence.join(' · ')}</p>}</article>)}
      </div>

      <div className="grid gap-3 md:grid-cols-3"><Findings title={t('aiStrengths')} items={evaluation.strengths} tone="positive" /><Findings title={t('aiWeaknessesRisks')} items={[...evaluation.weaknesses, ...evaluation.risks]} tone="warning" /><Findings title={t('aiMissingInformation')} items={evaluation.missing_information} tone="neutral" /></div>

      {canRunResearch && <section className="rounded-xl border bg-background p-3">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div><p className="flex items-center gap-2 text-sm font-semibold"><Sparkles className="size-4 text-primary" />{t('sourceSimilarityTitle')}</p><p className="mt-1 text-xs text-muted-foreground">{t('sourceSimilarityDescription')}</p></div>
          <Button size="sm" variant="outline" onClick={() => void refreshResearch()} disabled={loadingResearch}><RefreshCw className={`mr-1.5 size-3.5 ${loadingResearch ? 'animate-spin' : ''}`} />{research ? t('refreshResearch') : t('runResearch')}</Button>
        </div>
        {research && <div className="mt-3 space-y-3">
          <div className="flex flex-wrap items-center justify-between gap-2 rounded-lg bg-muted/50 px-3 py-2 text-xs"><span>{t('originalityIndicator')}: <strong className="font-data">{research.originality_score.toFixed(0)}%</strong></span><span className="text-muted-foreground">{research.originality_label}</span></div>
          <ResearchInsights research={research} />
          {research.sources.slice(0, 6).map((source) => <article key={`${source.url ?? source.title}-${source.source_type}`} className="rounded-lg border p-2.5">
            <div className="flex items-start justify-between gap-3"><div><p className="text-xs font-medium">{source.title}</p><p className="mt-1 text-[11px] text-muted-foreground">{source.explanation}</p></div><span className="font-data text-xs text-muted-foreground">{Math.round(source.similarity * 100)}%</span></div>
            {source.matched_terms.length > 0 && <div className="mt-2 flex flex-wrap gap-1">{source.matched_terms.slice(0, 6).map((term) => <span key={term} className="rounded bg-primary/[0.08] px-1.5 py-0.5 text-[10px] text-primary">{term}</span>)}</div>}
            {source.url && <a className="mt-2 inline-flex items-center gap-1 text-[11px] text-primary hover:underline" href={source.url} target="_blank" rel="noreferrer"><ExternalLink className="size-3" />{source.source_type}</a>}
          </article>)}
        </div>}
      </section>}
    </section>
  );
}
