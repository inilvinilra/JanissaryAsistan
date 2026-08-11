import { X } from 'lucide-react';

import type { Project } from '@/lib/api';
import { useLocale } from '@/lib/locale-context';
import { RankMedal } from '@/components/RankMedal';

export function PresentationMode({
  projects,
  category,
  onClose,
}: {
  projects: Project[];
  category: string;
  onClose: () => void;
}) {
  const { t, categoryLabel } = useLocale();
  const top3 = projects.slice(0, 3);

  return (
    <div className="hero-glow fixed inset-0 z-50 flex flex-col items-center justify-center bg-background p-8">
      <button
        type="button"
        onClick={onClose}
        aria-label={t('exitPresentation')}
        className="absolute top-6 right-6 flex size-10 items-center justify-center rounded-full border text-muted-foreground hover:text-foreground"
      >
        <X className="size-5" />
      </button>

      <p className="text-muted-foreground mb-2 text-sm font-medium tracking-widest uppercase">Creathon 2026</p>
      <h1 className="mb-12 text-3xl font-bold tracking-tight sm:text-5xl">
        {t('winnersTitle', { category: categoryLabel(category) })}
      </h1>

      <div className="flex w-full max-w-3xl flex-col items-center gap-6 sm:flex-row sm:items-end sm:justify-center">
        {[1, 0, 2].map((i) => {
          const project = top3[i];
          if (!project) return null;
          const rank = i + 1;
          return (
            <div
              key={project.id}
              className="animate-in fade-in slide-in-from-bottom-4 fill-mode-both flex flex-col items-center gap-3 duration-700"
              style={{ animationDelay: `${i * 150}ms`, order: rank === 1 ? 0 : rank }}
            >
              <RankMedal rank={rank} />
              <p className="max-w-[220px] text-center text-lg font-semibold">{project.name}</p>
              <p className="font-data text-4xl font-bold tabular-nums text-primary">
                {project.ai_score.toFixed(1)}
              </p>
            </div>
          );
        })}
      </div>
    </div>
  );
}
