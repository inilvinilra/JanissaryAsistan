import { BrainCircuit, CircleAlert } from 'lucide-react';

import type { JuryAiSummary } from '@/lib/api';

export function JuryAiSummaryPanel({ summary }: { summary: JuryAiSummary | null | undefined }) {
  if (!summary) return null;
  return <section className="space-y-3 rounded-lg border bg-card p-3">
    <div className="flex items-start justify-between gap-3">
      <div><p className="flex items-center gap-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground"><BrainCircuit className="size-3.5" /> AI pre-assessment</p><p className="mt-1 text-xs text-muted-foreground">Decision support only; the final decision belongs to the jury.</p></div>
      <div className="text-right"><p className="font-data text-lg font-semibold tabular-nums">{summary.total_score.toFixed(1)}</p><p className="text-[11px] text-muted-foreground">{Math.round(summary.confidence * 100)}% confidence</p></div>
    </div>
    <div className="grid gap-2 sm:grid-cols-2">{summary.kpi_scores.map((kpi) => <div key={kpi.name} className="rounded-md bg-muted/50 p-2 text-xs"><div className="flex justify-between gap-2"><span className="font-medium">{kpi.name}</span><span className="font-data">{kpi.score.toFixed(1)}</span></div><p className="mt-1 text-muted-foreground">{kpi.reason}</p></div>)}</div>
    <div className="grid gap-2 sm:grid-cols-2"><FeedbackList title="Strengths" items={summary.strengths} /><FeedbackList title="Needs improvement" items={summary.weaknesses} /><FeedbackList title="Missing information" items={summary.missing_information} /><FeedbackList title="Risks" items={summary.risks} warning /></div>
  </section>;
}

function FeedbackList({ title, items, warning = false }: { title: string; items: string[]; warning?: boolean }) {
  return <div className="rounded-md border p-2 text-xs"><p className={`flex items-center gap-1 font-medium ${warning ? 'text-amber-600 dark:text-amber-500' : ''}`}>{warning && <CircleAlert className="size-3" />}{title}</p>{items.length ? <ul className="mt-1.5 space-y-1 text-muted-foreground">{items.map((item) => <li key={item}>• {item}</li>)}</ul> : <p className="mt-1 text-muted-foreground">No findings.</p>}</div>;
}
