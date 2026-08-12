import { Download, Menu, Search } from 'lucide-react';

import type { CategoryTemplate, Project } from '@/lib/api';
import { useLocale } from '@/lib/locale-context';
import { Input } from '@/components/ui/input';
import { AddProjectDialog } from '@/components/AddProjectDialog';
import { NotificationBell } from '@/components/NotificationBell';

export function Topbar({
  category,
  categories,
  activeTemplate,
  search,
  onSearchChange,
  onProjectCreated,
  onExportCsv,
  onOpenMobileSidebar,
  canCreateProjects,
  canViewActivity,
}: {
  category: string;
  categories: CategoryTemplate[];
  activeTemplate: CategoryTemplate | undefined;
  search: string;
  onSearchChange: (value: string) => void;
  onProjectCreated: (project: Project) => void;
  onExportCsv: () => void;
  onOpenMobileSidebar: () => void;
  canCreateProjects: boolean;
  canViewActivity: boolean;
}) {
  const { t, categoryLabel } = useLocale();
  const kpiNames = activeTemplate?.kpis.map((k) => k.name).join(', ');

  return (
    <header className="sticky top-0 z-20 flex flex-col gap-3 border-b bg-background/95 px-6 py-4 backdrop-blur sm:flex-row sm:items-center sm:justify-between">
      <div className="flex items-center gap-3">
        <button
          type="button"
          onClick={onOpenMobileSidebar}
          aria-label={t('mobileMenuLabel')}
          className="text-muted-foreground hover:text-foreground md:hidden"
        >
          <Menu className="size-5" />
        </button>
        <div>
          <h1 className="text-lg font-semibold tracking-tight">
            {category ? categoryLabel(category) : t('appName')}
          </h1>
          <p className="text-muted-foreground text-xs">
            {kpiNames ? t('evaluatedOn', { kpis: kpiNames }) : t('topbarSubtitle')}
          </p>
        </div>
      </div>

      <div className="flex flex-wrap items-center gap-2">
        <div className="relative">
          <Search className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground" />
          <Input
            value={search}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder={t('searchPlaceholder')}
            className="h-8 w-40 pl-8 text-xs"
          />
        </div>

        <button
          type="button"
          onClick={onExportCsv}
          className="flex h-8 items-center gap-1.5 rounded-md border px-2.5 text-xs font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
        >
          <Download className="size-3.5" />
          {t('exportCsv')}
        </button>

        {canCreateProjects && categories.length > 0 && (
          <AddProjectDialog categories={categories} defaultCategory={category} onCreated={onProjectCreated} />
        )}

        <NotificationBell visible={canViewActivity} />
      </div>
    </header>
  );
}
