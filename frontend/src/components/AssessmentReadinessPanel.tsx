import { useState } from 'react';
import { CheckCircle2, CircleAlert, CircleDashed, RefreshCw } from 'lucide-react';

import {
  runCategoryFitAnalysis,
  runCriterionEvaluation,
  runProjectSimilarityAnalysis,
  type ProjectAssessmentReadiness,
} from '@/lib/api';
import { Button } from '@/components/ui/button';
import { useToast } from '@/lib/toast-context';

const gateStyle = {
  passed: { icon: CheckCircle2, tone: 'text-primary', label: 'Passed' },
  failed: { icon: CircleAlert, tone: 'text-destructive', label: 'Needs attention' },
  pending: { icon: CircleDashed, tone: 'text-muted-foreground', label: 'Pending' },
} as const;

export function AssessmentReadinessPanel({
  projectId,
  readiness,
  canRunAnalysis,
  onUpdated,
}: {
  projectId: number;
  readiness: ProjectAssessmentReadiness | null;
  canRunAnalysis: boolean;
  onUpdated: () => Promise<void>;
}) {
  const { showToast } = useToast();
  const [stage, setStage] = useState<'idle' | 'analysing' | 'evaluating'>('idle');
  const running = stage !== 'idle';

  async function runPendingAnalyses() {
    try {
      // Category fit and similarity first: the criterion evaluation reads both
      // so it can put their findings in front of the judge as risks. Running
      // them together would evaluate against whatever the previous run left.
      setStage('analysing');
      await Promise.all([runCategoryFitAnalysis(projectId), runProjectSimilarityAnalysis(projectId)]);
      setStage('evaluating');
      await runCriterionEvaluation(projectId);
      await onUpdated();
      showToast('Category, similarity and criterion evaluation completed.', 'success');
    } catch (error) {
      // An earlier gate may already have succeeded, so the panel is refreshed
      // either way rather than leaving it showing stale results.
      await onUpdated().catch(() => {});
      showToast((error as Error).message, 'error');
    } finally {
      setStage('idle');
    }
  }

  return (
    <section className="space-y-3 rounded-lg border bg-card p-3">
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div>
          <p className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">Assessment readiness</p>
          <p className={`mt-1 text-xs font-medium ${readiness?.ready_for_evaluation ? 'text-primary' : 'text-amber-600 dark:text-amber-500'}`}>
            {readiness?.ready_for_evaluation ? 'All mandatory checks passed' : 'Complete mandatory checks before evaluation'}
          </p>
        </div>
        {canRunAnalysis && <Button size="sm" variant="outline" disabled={running} onClick={() => void runPendingAnalyses()}>
          <RefreshCw className={`size-3.5 ${running ? 'animate-spin' : ''}`} />
          {stage === 'analysing' ? 'Analyzing…' : stage === 'evaluating' ? 'Evaluating criteria…' : 'Run analyses'}
        </Button>}
      </div>
      {!readiness ? <p className="text-xs text-muted-foreground">Readiness data is loading.</p> : <ul className="space-y-2">
        {readiness.checks.map((check) => {
          const style = gateStyle[check.status];
          const Icon = style.icon;
          return <li key={check.key} className="flex gap-2 text-xs">
            <Icon className={`mt-0.5 size-3.5 shrink-0 ${style.tone}`} />
            <span className="min-w-0 flex-1"><span className="font-medium">{check.label}</span><span className="mt-0.5 block text-muted-foreground">{check.detail}</span></span>
            <span className={`shrink-0 text-[11px] ${style.tone}`}>{check.requires_human_review ? 'Review' : style.label}</span>
          </li>;
        })}
      </ul>}
    </section>
  );
}
