import { useEffect, useState } from 'react';
import { History } from 'lucide-react';

import { getActivity, type ActivityEntry } from '@/lib/api';
import { useLocale } from '@/lib/locale-context';
import { Card } from '@/components/ui/card';

function relativeTime(iso: string, locale: 'tr' | 'en'): string {
  const diffMs = Date.now() - new Date(iso).getTime();
  const minutes = Math.max(1, Math.round(diffMs / 60000));
  const formatter = new Intl.RelativeTimeFormat(locale === 'tr' ? 'tr-TR' : 'en-US', { numeric: 'auto', style: 'short' });
  if (minutes < 60) return formatter.format(-minutes, 'minute');
  const hours = Math.round(minutes / 60);
  if (hours < 24) return formatter.format(-hours, 'hour');
  const days = Math.round(hours / 24);
  return formatter.format(-days, 'day');
}

export function ActivityPanel({ category, refreshKey }: { category: string; refreshKey: number }) {
  const { t, locale } = useLocale();
  const [entries, setEntries] = useState<ActivityEntry[]>([]);

  useEffect(() => {
    if (!category) return;
    getActivity(category, 8)
      .then(setEntries)
      .catch(() => setEntries([]));
  }, [category, refreshKey]);

  return (
    <Card className="p-5">
      <div className="mb-4 flex items-center gap-2">
        <History className="size-4 text-primary" />
        <h3 className="text-sm font-semibold">{t('activityTitle')}</h3>
      </div>

      {entries.length === 0 ? (
        <p className="text-muted-foreground text-xs">{t('activityEmpty')}</p>
      ) : (
        <ul className="space-y-3">
          {entries.map((entry, i) => (
            <li key={`${entry.project_id}-${entry.timestamp}-${i}`} className="text-xs">
              <p className="truncate font-medium">
                {t('activityMoved', {
                  name: entry.project_name,
                  from: entry.previous_rank ? String(entry.previous_rank) : '—',
                  to: String(entry.new_rank),
                })}
              </p>
              <p className="text-muted-foreground mt-0.5">{relativeTime(entry.timestamp, locale)}</p>
            </li>
          ))}
        </ul>
      )}
    </Card>
  );
}
