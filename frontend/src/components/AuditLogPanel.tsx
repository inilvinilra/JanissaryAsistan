import { useEffect, useState } from 'react';
import { ClipboardList, RefreshCw } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { getAuditEvents, type AuditEvent } from '@/lib/api';
import { useLocale } from '@/lib/locale-context';

function actionLabel(action: string, t: (key: string) => string) {
  const key = `audit_${action}`;
  const value = t(key);
  return value === key ? action.replaceAll('_', ' ') : value;
}

export function AuditLogPanel() {
  const { t } = useLocale();
  const [events, setEvents] = useState<AuditEvent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  async function load() { setLoading(true); try { setEvents(await getAuditEvents()); setError(''); } catch (e) { setError((e as Error).message); } finally { setLoading(false); } }
  useEffect(() => { void load(); }, []);
  return <div className="space-y-6">
    <div className="flex items-start justify-between gap-3"><div><h1 className="text-xl font-semibold">{t('auditTitle')}</h1><p className="text-muted-foreground mt-1 text-sm">{t('auditDescription')}</p></div><button type="button" onClick={() => void load()} className="flex items-center gap-2 rounded-md border px-3 py-2 text-xs font-medium hover:bg-accent"><RefreshCw className="size-3.5" />{t('refresh')}</button></div>
    <Card><CardHeader><CardTitle className="flex items-center gap-2"><ClipboardList className="size-4" />{t('auditRecent')}</CardTitle></CardHeader><CardContent className="p-0">{loading ? <p className="text-muted-foreground px-5 py-6 text-sm">{t('loading')}</p> : error ? <p className="text-destructive px-5 py-6 text-sm">{error}</p> : <div className="divide-y">{events.map((event) => <div key={event.id} className="grid gap-2 px-5 py-4 md:grid-cols-[1fr_180px_160px]"><div><p className="text-sm font-medium">{actionLabel(event.action, t)}</p><p className="text-muted-foreground mt-1 text-xs">{event.entity_type}{event.entity_id ? ` #${event.entity_id}` : ''}</p><p className="text-muted-foreground mt-1 truncate text-[10px]">{t('auditHash')}: {event.event_hash}</p></div><p className="text-xs"><span className="text-muted-foreground">{t('auditActor')}: </span>{event.actor}</p><p className="text-muted-foreground text-xs md:text-right">{new Date(event.created_at).toLocaleString()}</p></div>)}{events.length === 0 && <p className="text-muted-foreground px-5 py-6 text-sm">{t('auditEmpty')}</p>}</div>}</CardContent></Card>
  </div>;
}
