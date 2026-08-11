import { Trophy } from 'lucide-react';

import type { Project } from '@/lib/api';
import { useLocale } from '@/lib/locale-context';
import { cn } from '@/lib/utils';
import { Card } from '@/components/ui/card';

export function Podium({ projects }: { projects: Project[] }) {
  const { t } = useLocale();
  const top3 = projects.slice(0, 3);
  if (top3.length === 0) return null;

  return (
    <Card className="p-5">
      <div className="mb-4 flex items-center gap-2">
        <Trophy className="size-4 text-primary" />
        <h3 className="text-sm font-semibold">{t('podiumTitle')}</h3>
      </div>

      <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
        {top3.map((project, i) => (
          <div
            key={project.id}
            className={cn(
              'animate-in fade-in slide-in-from-bottom-2 fill-mode-both rounded-lg border p-4 text-center duration-500',
              i === 0 ? 'border-primary/60 bg-primary/5' : 'border-border',
            )}
            style={{ animationDelay: `${i * 90}ms` }}
          >
            <div
              className={cn(
                'font-data mx-auto mb-2 flex size-10 items-center justify-center rounded-full text-sm font-bold',
                i === 0 && 'bg-primary text-primary-foreground',
                i === 1 && 'border-2 border-primary/50 text-primary',
                i === 2 && 'border-2 border-primary/30 text-primary/80',
              )}
            >
              {i + 1}
            </div>
            <p className="truncate text-sm font-medium">{project.name}</p>
            <p className="font-data mt-1 text-2xl font-bold tabular-nums">{project.ai_score.toFixed(1)}</p>
          </div>
        ))}
      </div>
    </Card>
  );
}
