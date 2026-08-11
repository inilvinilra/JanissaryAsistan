import { useEffect, useRef, useState } from 'react';
import { Bell } from 'lucide-react';

import { getAuditEvents, type AuditEvent } from '@/lib/api';
import { useLocale } from '@/lib/locale-context';
import { cn } from '@/lib/utils';

function relativeTime(iso: string, locale: 'tr' | 'en'): string {
  const diffMs = Date.now() - new Date(iso).getTime();
  const minutes = Math.max(1, Math.round(diffMs / 60000));
  if (minutes < 60) return locale === 'tr' ? `${minutes} dk önce` : `${minutes}m ago`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return locale === 'tr' ? `${hours} sa önce` : `${hours}h ago`;
  const days = Math.round(hours / 24);
  return locale === 'tr' ? `${days} gün önce` : `${days}d ago`;
}

export function NotificationBell() {
  const { t, locale } = useLocale();
  const [open, setOpen] = useState(false);
  const [entries, setEntries] = useState<AuditEvent[]>([]);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    getAuditEvents(8)
      .then(setEntries)
      .catch(() => {});
    const id = setInterval(() => {
      getAuditEvents(8)
        .then(setEntries)
        .catch(() => {});
    }, 30000);
    return () => clearInterval(id);
  }, []);

  useEffect(() => {
    function onClickOutside(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false);
    }
    document.addEventListener('mousedown', onClickOutside);
    return () => document.removeEventListener('mousedown', onClickOutside);
  }, []);

  return (
    <div ref={ref} className="relative">
      <button
        type="button"
        onClick={() => setOpen((o) => !o)}
        aria-label={t('activityTitle')}
        className={cn(
          'flex size-8 items-center justify-center rounded-md border text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground',
          open && 'bg-accent text-accent-foreground',
        )}
      >
        <Bell className="size-4" />
      </button>

      {open && (
        <div className="surface-elevated animate-in fade-in zoom-in-95 fill-mode-both absolute right-0 z-50 mt-2 w-72 rounded-lg border bg-popover p-3 text-popover-foreground duration-150">
          <p className="mb-2 text-xs font-semibold tracking-wide uppercase text-muted-foreground">
            {t('activityTitle')}
          </p>
          {entries.length === 0 ? (
            <p className="text-muted-foreground text-xs">{t('activityEmpty')}</p>
          ) : (
            <ul className="space-y-2.5">
              {entries.map((entry) => (
                <li key={entry.id} className="text-xs">
                  <p className="truncate"><span className="font-medium">{t(`audit_${entry.action}`)}</span> <span className="text-muted-foreground">· {entry.entity_type}{entry.entity_id ? ` #${entry.entity_id}` : ''}</span></p>
                  <p className="text-muted-foreground">{t('auditActor')}: {entry.actor} · {relativeTime(entry.created_at, locale)}</p>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </div>
  );
}
