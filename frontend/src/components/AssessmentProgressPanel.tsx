import { useCallback, useEffect, useRef, useState } from 'react';
import { Activity, CircleAlert, Play } from 'lucide-react';

import {
  getAssessmentProgress,
  runCompetitionAssessments,
  type AssessmentProgress,
} from '@/lib/api';
import { Button } from '@/components/ui/button';
import { useLocale } from '@/lib/locale-context';
import { useToast } from '@/lib/toast-context';

/** How often the panel refreshes while a run is still working through the queue. */
const POLL_INTERVAL_MS = 5000;

const MISSING_LABELS: Record<string, string> = {
  category_fit: 'Category fit',
  similarity: 'Similarity',
  criterion_evaluation: 'AI criteria',
};

function Stat({ label, value, total }: { label: string; value: number; total?: number }) {
  return (
    <div className="rounded-lg border bg-background p-3">
      <p className="text-[10px] font-medium uppercase tracking-wide text-muted-foreground">{label}</p>
      <p className="font-data mt-1 text-2xl font-semibold tracking-tight">
        {value}
        {total !== undefined && <span className="ml-1 text-sm text-muted-foreground">/{total}</span>}
      </p>
    </div>
  );
}

export function AssessmentProgressPanel({ competitionId, canRun }: { competitionId: number; canRun: boolean }) {
  const { t } = useLocale();
  const { showToast } = useToast();
  const [progress, setProgress] = useState<AssessmentProgress | null>(null);
  const [starting, setStarting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  // Held in a ref so the polling effect does not restart on every refresh.
  const pendingCount = useRef(0);

  const refresh = useCallback(async () => {
    const next = await getAssessmentProgress(competitionId);
    pendingCount.current = next.pending_projects.length;
    setProgress(next);
    return next;
  }, [competitionId]);

  useEffect(() => {
    setProgress(null);
    setError(null);
    refresh().catch((reason) => setError((reason as Error).message));
  }, [competitionId, refresh]);

  // The run is queued on the server and takes seconds per project, so the panel
  // follows it rather than leaving the manager to reload the page. Polling
  // stops once nothing is outstanding.
  useEffect(() => {
    const timer = window.setInterval(() => {
      if (pendingCount.current === 0) return;
      refresh().catch(() => {});
    }, POLL_INTERVAL_MS);
    return () => window.clearInterval(timer);
  }, [refresh]);

  async function startRun() {
    setStarting(true);
    try {
      const next = await runCompetitionAssessments(competitionId);
      pendingCount.current = next.pending_projects.length;
      setProgress(next);
      showToast(
        next.pending_projects.length === 0
          ? t('assessmentRunNothingPending')
          : t('assessmentRunQueued', { count: String(next.pending_projects.length) }),
        'success',
      );
    } catch (reason) {
      showToast((reason as Error).message, 'error');
    } finally {
      setStarting(false);
    }
  }

  if (error) {
    return <div className="rounded-lg border border-destructive/40 p-3 text-xs text-destructive">{error}</div>;
  }

  const percent = progress ? Math.round(progress.completion_percent) : 0;
  const running = (progress?.pending_projects.length ?? 0) > 0;

  return (
    <div className="rounded-lg border p-3">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <p className="flex items-center gap-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
          <Activity className="size-3.5" />
          {t('assessmentProgressTitle')}
        </p>
        {canRun && progress && (
          <Button size="sm" variant="outline" disabled={starting || !running} onClick={() => void startRun()}>
            <Play className="mr-1.5 size-3.5" />
            {starting ? t('assessmentRunStarting') : running ? t('assessmentRunPending', { count: String(progress.pending_projects.length) }) : t('assessmentRunComplete')}
          </Button>
        )}
      </div>
      <div className="mt-3 space-y-4">
        {!progress ? (
          <p className="text-sm text-muted-foreground">{t('loading')}</p>
        ) : (
          <>
            <div>
              <div className="flex items-baseline justify-between">
                <p className="text-xs text-muted-foreground">{t('assessmentCompletion')}</p>
                <p className="font-data text-sm font-semibold">{percent}%</p>
              </div>
              <div
                className="mt-1.5 h-2 overflow-hidden rounded-full bg-muted"
                role="progressbar"
                aria-valuenow={percent}
                aria-valuemin={0}
                aria-valuemax={100}
                aria-label={t('assessmentCompletion')}
              >
                <div className="h-full rounded-full bg-primary transition-[width] duration-500" style={{ width: `${percent}%` }} />
              </div>
              {running && <p className="mt-1.5 text-[11px] text-muted-foreground">{t('assessmentRunInFlight')}</p>}
            </div>

            <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
              <Stat label={t('assessmentParsedReports')} value={progress.parsed_reports} total={progress.total_projects} />
              <Stat label={t('assessmentCategoryFit')} value={progress.category_fit_completed} total={progress.parsed_reports} />
              <Stat label={t('assessmentSimilarity')} value={progress.similarity_completed} total={progress.parsed_reports} />
              <Stat label={t('assessmentCriteria')} value={progress.criterion_evaluation_completed} total={progress.parsed_reports} />
            </div>

            {progress.flagged_for_review > 0 && (
              <p className="flex items-center gap-2 rounded-lg border border-amber-500/30 bg-amber-500/[0.08] p-2.5 text-xs text-amber-900 dark:text-amber-200">
                <CircleAlert className="size-3.5 shrink-0" />
                {t('assessmentFlagged', { count: String(progress.flagged_for_review) })}
              </p>
            )}

            {progress.pending_projects.length > 0 && (
              <div>
                <p className="text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">{t('assessmentPendingList')}</p>
                <ul className="mt-2 space-y-1">
                  {progress.pending_projects.slice(0, 8).map((item) => (
                    <li key={item.project_id} className="flex flex-wrap items-center gap-2 rounded-md border px-2.5 py-1.5 text-xs">
                      <span className="font-data font-medium">{item.project_reference}</span>
                      <span className="text-muted-foreground">{item.category}</span>
                      <span className="ml-auto text-muted-foreground">
                        {item.missing.map((key) => MISSING_LABELS[key] ?? key).join(' · ')}
                      </span>
                    </li>
                  ))}
                </ul>
                {progress.pending_projects.length > 8 && (
                  <p className="mt-1.5 text-[11px] text-muted-foreground">
                    {t('assessmentPendingMore', { count: String(progress.pending_projects.length - 8) })}
                  </p>
                )}
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}
