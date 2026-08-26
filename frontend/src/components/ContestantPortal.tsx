import { useEffect, useState } from 'react';
import { LogOut, Sparkles, Target } from 'lucide-react';

import { getMyFeedback, type CategoryFitSummary, type ContestantFeedback } from '@/lib/api';
import { Button } from '@/components/ui/button';

export function ContestantPortal({ onSignOut }: { onSignOut: () => Promise<void> }) {
  const [feedback, setFeedback] = useState<ContestantFeedback[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => { getMyFeedback().then(setFeedback).catch((reason) => setError((reason as Error).message)); }, []);
  return <main className="mx-auto min-h-screen max-w-4xl space-y-6 px-4 py-8 sm:px-6"><header className="flex items-center justify-between border-b pb-5"><div><p className="text-sm font-semibold text-primary">Jury Assistant</p><h1 className="mt-1 text-2xl font-semibold tracking-tight">Project feedback</h1><p className="mt-1 text-sm text-muted-foreground">AI findings support your project’s improvement; they do not replace the official jury decision.</p></div><Button variant="outline" size="sm" onClick={() => void onSignOut()}><LogOut className="size-3.5" />Sign out</Button></header>{error ? <p className="rounded-md border border-destructive/30 p-4 text-sm text-destructive">{error}</p> : feedback === null ? <p className="text-sm text-muted-foreground">Loading feedback…</p> : feedback.length === 0 ? <p className="rounded-lg border p-5 text-sm text-muted-foreground">Your feedback will appear here after the evaluation is published.</p> : feedback.map((item) => <article key={item.project_id} className="space-y-4 rounded-xl border bg-card p-5 shadow-sm"><div className="flex flex-wrap items-start justify-between gap-3"><div><h2 className="text-lg font-semibold">{item.project_name}</h2><p className="mt-1 text-xs text-muted-foreground">{item.category} · Evaluated {new Date(item.evaluated_at).toLocaleDateString()}</p></div><div className="rounded-lg bg-primary/10 px-3 py-2 text-right"><p className="font-data text-xl font-semibold">{item.total_score.toFixed(1)}</p><p className="text-[11px] text-muted-foreground">Pre-assessment score</p></div></div><CategoryFit category={item.category} fit={item.category_fit} /><div className="grid gap-3 md:grid-cols-3"><Feedback title="Strengths" values={item.strengths} /><Feedback title="Areas to improve" values={item.weaknesses} /><Feedback title="Suggestions" values={item.suggestions} /></div></article>)}</main>;
}

function CategoryFit({ category, fit }: { category: string; fit: CategoryFitSummary | null }) {
  if (!fit) {
    return <section className="rounded-lg bg-muted/50 p-3"><p className="flex items-center gap-1.5 text-xs font-semibold"><Target className="size-3.5 text-primary" />Category fit</p><p className="mt-2 text-sm text-muted-foreground">Category-fit analysis has not been run for this submission yet.</p></section>;
  }
  return (
    <section className="rounded-lg bg-muted/50 p-3">
      <p className="flex items-center gap-1.5 text-xs font-semibold"><Target className="size-3.5 text-primary" />Category fit</p>
      <p className="mt-2 text-sm text-muted-foreground">
        Your submission matches <strong className="text-foreground">{category}</strong> at {fit.current_category_score.toFixed(0)}%.
      </p>
      {fit.requires_review && (
        <p className="mt-1.5 text-sm text-muted-foreground">
          The content also shows strong overlap with <strong className="text-foreground">{fit.recommended_category}</strong> ({fit.recommended_category_score.toFixed(0)}%). Consider whether that category fits your project better.
        </p>
      )}
    </section>
  );
}

function Feedback({ title, values }: { title: string; values: string[] }) { return <section className="rounded-lg bg-muted/50 p-3"><p className="flex items-center gap-1.5 text-xs font-semibold"><Sparkles className="size-3.5 text-primary" />{title}</p>{values.length ? <ul className="mt-2 space-y-1.5 text-sm text-muted-foreground">{values.map((value) => <li key={value}>• {value}</li>)}</ul> : <p className="mt-2 text-sm text-muted-foreground">No feedback was generated in this area.</p>}</section>; }
