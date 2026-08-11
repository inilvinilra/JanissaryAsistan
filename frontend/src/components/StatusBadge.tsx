import type { ProjectStatus } from '@/lib/api';
import { useLocale } from '@/lib/locale-context';
import { cn } from '@/lib/utils';

const STYLES: Record<ProjectStatus, string> = {
  new: 'bg-secondary text-secondary-foreground',
  reviewing: 'bg-muted text-muted-foreground',
  finalist: 'bg-primary/15 text-primary',
  rejected: 'bg-destructive/15 text-destructive',
};

const LABEL_KEYS: Record<ProjectStatus, string> = {
  new: 'statusNew',
  reviewing: 'statusReviewing',
  finalist: 'statusFinalist',
  rejected: 'statusRejected',
};

export function StatusBadge({ status }: { status: ProjectStatus }) {
  const { t } = useLocale();
  return (
    <span className={cn('rounded-full px-2 py-0.5 text-[11px] font-medium whitespace-nowrap', STYLES[status])}>
      {t(LABEL_KEYS[status])}
    </span>
  );
}
