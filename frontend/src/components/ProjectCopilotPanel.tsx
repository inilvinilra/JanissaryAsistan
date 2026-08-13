import { useState } from 'react';
import { Bot, Send } from 'lucide-react';

import { askProjectCopilot, type CopilotResponse } from '@/lib/api';
import { useLocale } from '@/lib/locale-context';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';

export function ProjectCopilotPanel({ projectId }: { projectId: number }) {
  const { t } = useLocale();
  const [question, setQuestion] = useState('');
  const [response, setResponse] = useState<CopilotResponse | null>(null);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  async function submit(nextQuestion = question) {
    if (nextQuestion.trim().length < 3) return;
    setLoading(true); setError('');
    try { setResponse(await askProjectCopilot(projectId, nextQuestion.trim())); setQuestion(''); }
    catch (requestError) { setError((requestError as Error).message); }
    finally { setLoading(false); }
  }

  return <section className="rounded-2xl border bg-card p-4 shadow-sm"><div className="flex items-start gap-3"><span className="rounded-xl bg-primary/10 p-2 text-primary"><Bot className="size-5" /></span><div><p className="font-semibold">{t('projectCopilotTitle')}</p><p className="mt-0.5 text-xs text-muted-foreground">{t('projectCopilotDescription')}</p></div></div><div className="mt-3 flex flex-wrap gap-2">{['What are the strongest KPI findings?', 'Which risks require jury verification?', 'What information is missing from this submission?'].map((item) => <button key={item} type="button" onClick={() => void submit(item)} className="rounded-full border px-2.5 py-1 text-[11px] text-muted-foreground hover:bg-muted">{item}</button>)}</div><div className="mt-3 flex gap-2"><Textarea value={question} onChange={(event) => setQuestion(event.target.value)} placeholder={t('projectCopilotPlaceholder')} className="min-h-20" /><Button type="button" onClick={() => void submit()} disabled={loading || question.trim().length < 3}><Send className="size-4" /></Button></div>{error && <p className="mt-2 text-xs text-destructive">{error}</p>}{response && <div className="mt-3 rounded-xl border bg-muted/30 p-3"><p className="text-sm leading-relaxed">{response.answer}</p>{response.citations.length > 0 && <div className="mt-3 border-t pt-2"><p className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">{t('aiEvidence')}</p>{response.citations.map((citation) => <p key={citation} className="mt-1 truncate text-xs text-primary">{citation}</p>)}</div>}</div>}</section>;
}
