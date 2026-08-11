import { Trophy } from 'lucide-react';

import type { CategoryTemplate, Project } from '@/lib/api';
import { useLocale } from '@/lib/locale-context';
import { CATEGORY_ICONS, DEFAULT_CATEGORY_ICON } from '@/lib/category-icons';
import { isPhaseCategory } from '@/lib/category-groups';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/card';
import { StatTile } from '@/components/StatTile';
import { RankMedal } from '@/components/RankMedal';

export function Overview({
  categories,
  allProjects,
  onSelectCategory,
}: {
  categories: CategoryTemplate[];
  allProjects: Project[];
  onSelectCategory: (category: string) => void;
}) {
  const { t, categoryLabel } = useLocale();

  const fieldCategories = categories.filter((c) => !isPhaseCategory(c.category));
  const fieldProjects = allProjects.filter((p) => !isPhaseCategory(p.category));

  const overallAvg = fieldProjects.length
    ? fieldProjects.reduce((sum, p) => sum + p.ai_score, 0) / fieldProjects.length
    : 0;

  const topAcrossFields = [...fieldProjects].sort((a, b) => b.ai_score - a.ai_score).slice(0, 8);

  return (
    <div className="space-y-6">
      <div className="grid grid-cols-3 gap-3">
        <StatTile label={t('statCount')} value={fieldProjects.length} delay={0} />
        <StatTile label={t('overviewFields')} value={fieldCategories.length} delay={80} />
        <StatTile label={t('statAvg')} value={overallAvg} decimals={1} delay={160} />
      </div>

      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
        {fieldCategories.map((cat, i) => {
          const Icon = CATEGORY_ICONS[cat.category] ?? DEFAULT_CATEGORY_ICON;
          const projects = fieldProjects.filter((p) => p.category === cat.category);
          const avg = projects.length ? projects.reduce((s, p) => s + p.ai_score, 0) / projects.length : 0;

          return (
            <button
              key={cat.category}
              type="button"
              onClick={() => onSelectCategory(cat.category)}
              className="surface-elevated animate-in fade-in slide-in-from-bottom-2 fill-mode-both group rounded-xl border bg-card p-4 text-left duration-500 hover:border-primary/50"
              style={{ animationDelay: `${Math.min(i, 8) * 60}ms` }}
            >
              <div className="mb-3 flex items-center gap-2.5">
                <span className="flex size-8 items-center justify-center rounded-lg bg-secondary text-foreground group-hover:bg-primary group-hover:text-primary-foreground">
                  <Icon className="size-4" />
                </span>
                <span className="flex-1 truncate text-sm font-semibold">{categoryLabel(cat.category)}</span>
              </div>
              <div className="flex items-end justify-between">
                <div>
                  <p className="text-muted-foreground text-[10px] uppercase">{t('statCount')}</p>
                  <p className="font-data text-lg font-bold tabular-nums">{projects.length}</p>
                </div>
                <div className="text-right">
                  <p className="text-muted-foreground text-[10px] uppercase">{t('statAvg')}</p>
                  <p className="font-data text-lg font-bold tabular-nums">
                    {projects.length ? avg.toFixed(1) : '—'}
                  </p>
                </div>
              </div>
            </button>
          );
        })}
      </div>

      {topAcrossFields.length > 0 && (
        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <Trophy className="size-4 text-primary" />
              <CardTitle>{t('overviewTopAcrossFields')}</CardTitle>
            </div>
          </CardHeader>
          <CardContent className="divide-y p-0">
            {topAcrossFields.map((project, i) => (
              <button
                key={project.id}
                type="button"
                onClick={() => onSelectCategory(project.category)}
                className="flex w-full items-center gap-3 px-5 py-3 text-left transition-colors hover:bg-accent/40"
              >
                <RankMedal rank={i + 1} />
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-medium">{project.name}</p>
                  <p className="text-muted-foreground text-xs">{categoryLabel(project.category)}</p>
                </div>
                <p className="font-data text-base font-bold tabular-nums">{project.ai_score.toFixed(1)}</p>
              </button>
            ))}
          </CardContent>
        </Card>
      )}
    </div>
  );
}
