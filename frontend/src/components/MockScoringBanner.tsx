import { useEffect, useState } from 'react';
import { Info, X } from 'lucide-react';

import { useLocale } from '@/lib/locale-context';

const STORAGE_KEY = 'jury-assistant-mock-banner-dismissed';

export function MockScoringBanner() {
  const { t } = useLocale();
  const [dismissed, setDismissed] = useState(true);

  useEffect(() => {
    setDismissed(localStorage.getItem(STORAGE_KEY) === '1');
  }, []);

  if (dismissed) return null;

  return (
    <div className="animate-in fade-in flex items-start gap-2.5 rounded-lg border border-primary/30 bg-accent px-4 py-3 text-sm text-accent-foreground duration-300">
      <Info className="mt-0.5 size-4 shrink-0" />
      <p className="flex-1">{t('mockBannerText')}</p>
      <button
        type="button"
        onClick={() => {
          localStorage.setItem(STORAGE_KEY, '1');
          setDismissed(true);
        }}
        aria-label={t('dismiss')}
        className="shrink-0 opacity-70 hover:opacity-100"
      >
        <X className="size-4" />
      </button>
    </div>
  );
}
