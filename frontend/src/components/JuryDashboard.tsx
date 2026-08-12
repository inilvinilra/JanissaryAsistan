import { lazy, Suspense, useEffect, useMemo, useState } from 'react';
import { Info, Presentation, Scale } from 'lucide-react';
import {
  DndContext,
  closestCenter,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
  type DragEndEvent,
} from '@dnd-kit/core';
import { SortableContext, verticalListSortingStrategy, sortableKeyboardCoordinates, arrayMove } from '@dnd-kit/sortable';

import { getCategories, getProjects, subscribeToUpdates, updateRanking, type CategoryTemplate, type Project } from '@/lib/api';
import { useLocale } from '@/lib/locale-context';
import { useToast } from '@/lib/toast-context';
import { exportProjectsCsv } from '@/lib/export-csv';
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card';
import { Table, TableHeader, TableBody, TableRow, TableHead } from '@/components/ui/table';
import { SortableProjectRow } from '@/components/SortableProjectRow';
import { Sidebar } from '@/components/Sidebar';
import { Topbar } from '@/components/Topbar';
import { Podium } from '@/components/Podium';
import { KpiDonutPanel } from '@/components/KpiDonutPanel';
import { MockScoringBanner } from '@/components/MockScoringBanner';
import { StatTile } from '@/components/StatTile';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';

const ActivityPanel = lazy(() => import('@/components/ActivityPanel').then((module) => ({ default: module.ActivityPanel })));
const ProjectDetailDialog = lazy(() => import('@/components/ProjectDetailDialog').then((module) => ({ default: module.ProjectDetailDialog })));
const CompareDialog = lazy(() => import('@/components/CompareDialog').then((module) => ({ default: module.CompareDialog })));
const PresentationMode = lazy(() => import('@/components/PresentationMode').then((module) => ({ default: module.PresentationMode })));
const Overview = lazy(() => import('@/components/Overview').then((module) => ({ default: module.Overview })));
const CompetitionManager = lazy(() => import('@/components/CompetitionManager').then((module) => ({ default: module.CompetitionManager })));
const UserManager = lazy(() => import('@/components/UserManager').then((module) => ({ default: module.UserManager })));
const AuditLogPanel = lazy(() => import('@/components/AuditLogPanel').then((module) => ({ default: module.AuditLogPanel })));
const ReportsPanel = lazy(() => import('@/components/ReportsPanel').then((module) => ({ default: module.ReportsPanel })));
const SettingsPanel = lazy(() => import('@/components/SettingsPanel').then((module) => ({ default: module.SettingsPanel })));
const NotificationCenter = lazy(() => import('@/components/NotificationCenter').then((module) => ({ default: module.NotificationCenter })));
const CompetitionOperationsPanel = lazy(() => import('@/components/CompetitionOperationsPanel').then((module) => ({ default: module.CompetitionOperationsPanel })));
const ScoreChart = lazy(() => import('@/components/ScoreChart').then((module) => ({ default: module.ScoreChart })));

function DeferredPanel({ children }: { children: React.ReactNode }) {
  return <Suspense fallback={<p className="text-muted-foreground text-sm">Loading workspace…</p>}>{children}</Suspense>;
}

export function JuryDashboard({ onSignOut }: { onSignOut: () => Promise<void> }) {
  const { t, categoryLabel } = useLocale();
  const { showToast } = useToast();
  const [categories, setCategories] = useState<CategoryTemplate[]>([]);
  const [category, setCategory] = useState<string>(() => {
    if (typeof localStorage === 'undefined') return '';
    try { return JSON.parse(localStorage.getItem('jury-auth-user') ?? '{}').category ?? ''; }
    catch { return ''; }
  });
  const [projects, setProjects] = useState<Project[]>([]);
  const [allProjects, setAllProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [search, setSearch] = useState('');
  const [mobileSidebarOpen, setMobileSidebarOpen] = useState(false);
  const [detailProject, setDetailProject] = useState<Project | null>(null);
  const [activityRefreshKey, setActivityRefreshKey] = useState(0);
  const [selectedIds, setSelectedIds] = useState<Set<number>>(new Set());
  const [compareOpen, setCompareOpen] = useState(false);
  const [presentationOpen, setPresentationOpen] = useState(false);
  const [statusFilter, setStatusFilter] = useState<'all' | Project['status']>('all');
  const [sortBy, setSortBy] = useState<'score' | 'name'>('score');
  const [view, setView] = useState<'dashboard' | 'competitions' | 'users' | 'audit' | 'reports' | 'settings' | 'notifications'>('dashboard');
  const currentRole = typeof localStorage === 'undefined' ? 'read_only' : (() => { try { return JSON.parse(localStorage.getItem('jury-auth-user') ?? '{}').role || 'read_only'; } catch { return 'read_only'; } })();
  const canCreateProjects = ['system_admin', 'competition_manager', 'chief_judge'].includes(currentRole);
  const canViewActivity = currentRole === 'system_admin';

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 4 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );

  useEffect(() => {
    getCategories()
      .then(setCategories)
      .catch((e) => setError(e.message));

    getProjects()
      .then(setAllProjects)
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    if (!category) return;
    setLoading(true);
    setSearch('');
    setSelectedIds(new Set());
    getProjects(category)
      .then(setProjects)
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, [category]);

  useEffect(() => {
    if (detailProject) return;
    const controller = new AbortController();
    async function listenForUpdates() {
      while (!controller.signal.aborted) {
        try {
          await subscribeToUpdates(controller.signal, () => {
            if (category) getProjects(category).then(setProjects).catch(() => {});
            else getProjects().then(setAllProjects).catch(() => {});
          });
        } catch {
          if (!controller.signal.aborted) await new Promise((resolve) => window.setTimeout(resolve, 3000));
        }
      }
    }
    void listenForUpdates();
    return () => controller.abort();
  }, [category, detailProject]);

  const categoryCounts = useMemo(() => {
    const counts: Record<string, number> = {};
    for (const p of allProjects) counts[p.category] = (counts[p.category] ?? 0) + 1;
    return counts;
  }, [allProjects]);

  const stats = useMemo(() => {
    if (projects.length === 0) return null;
    const avg = projects.reduce((sum, p) => sum + p.ai_score, 0) / projects.length;
    const top = projects.reduce((best, p) => (p.ai_score > best.ai_score ? p : best), projects[0]);
    return { count: projects.length, avg, top };
  }, [projects]);

  const activeTemplate = categories.find((c) => c.category === category);
  const kpiOrder = activeTemplate?.kpis.map((k) => k.name) ?? [];

  let filteredProjects = search.trim()
    ? projects.filter((p) => p.name.toLowerCase().includes(search.trim().toLowerCase()))
    : projects;
  if (statusFilter !== 'all') filteredProjects = filteredProjects.filter((p) => p.status === statusFilter);
  if (sortBy === 'name') filteredProjects = [...filteredProjects].sort((a, b) => a.name.localeCompare(b.name));

  const selectedProjects = projects.filter((p) => selectedIds.has(p.id));

  function toggleSelect(id: number) {
    setSelectedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  function applyProjectUpdate(updated: Project) {
    setProjects((prev) => prev.map((p) => (p.id === updated.id ? updated : p)));
    setDetailProject(updated);
  }

  async function handleDragEnd(event: DragEndEvent) {
    if (sortBy !== 'score') return;

    const { active, over } = event;
    if (!over || active.id === over.id) return;

    const oldIndex = projects.findIndex((p) => p.id === active.id);
    const newIndex = projects.findIndex((p) => p.id === over.id);
    const reordered = arrayMove(projects, oldIndex, newIndex);
    setProjects(reordered);

    try {
      await updateRanking(category, reordered.map((p) => p.id));
      const fresh = await getProjects(category);
      setProjects(fresh);
      showToast(t('toastRankingSaved'), 'success');
      setActivityRefreshKey((k) => k + 1);
    } catch (e) {
      setError((e as Error).message);
      showToast(t('toastRankingFailed'), 'error');
    }
  }

  if (error) {
    return (
      <div className="flex min-h-screen items-center justify-center p-6">
        <Card className="max-w-md border-destructive/40">
          <CardContent className="text-destructive text-sm">
            {t('errorPrefix')} {error}. {t('errorBackendCheck', { url: import.meta.env.PUBLIC_API_URL })}
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="flex min-h-screen">
      <Sidebar
        categories={categories}
        category={category}
        categoryCounts={categoryCounts}
        onSelect={(nextCategory) => {
          setView('dashboard');
          setCategory(nextCategory);
        }}
        onOpenCompetitions={() => setView('competitions')}
        onOpenUsers={() => setView('users')}
        onOpenAudit={() => setView('audit')}
        onOpenReports={() => setView('reports')}
        onOpenSettings={() => setView('settings')}
        onOpenNotifications={() => setView('notifications')}
        onSignOut={onSignOut}
        mobileOpen={mobileSidebarOpen}
        onCloseMobile={() => setMobileSidebarOpen(false)}
      />

      <div className="flex-1">
        {view === 'dashboard' && <Topbar
          category={category}
          categories={categories}
          activeTemplate={activeTemplate}
          search={search}
          onSearchChange={setSearch}
          onExportCsv={() => exportProjectsCsv(filteredProjects, category)}
          onOpenMobileSidebar={() => setMobileSidebarOpen(true)}
          canCreateProjects={canCreateProjects}
          canViewActivity={canViewActivity}
          onProjectCreated={(project) => {
            if (project.category === category) setProjects((prev) => [...prev, project]);
            setAllProjects((prev) => [...prev, project]);
            showToast(t('toastProjectAdded', { name: project.name }), 'success');
          }}
        />}

        <main className="mx-auto w-full max-w-7xl space-y-6 px-4 py-6 sm:px-6 lg:px-8">
          {view === 'competitions' ? (
            <DeferredPanel><CompetitionManager /></DeferredPanel>
          ) : view === 'users' ? (
            <DeferredPanel><UserManager /></DeferredPanel>
          ) : view === 'audit' ? (
            <DeferredPanel><AuditLogPanel /></DeferredPanel>
          ) : view === 'reports' ? (
            <DeferredPanel><ReportsPanel /></DeferredPanel>
          ) : view === 'settings' ? (
            <DeferredPanel><SettingsPanel /></DeferredPanel>
          ) : view === 'notifications' ? (
            <DeferredPanel><NotificationCenter /></DeferredPanel>
          ) : category === '' ? (
            <>
              <DeferredPanel><CompetitionOperationsPanel /></DeferredPanel>
              <DeferredPanel><Overview categories={categories} allProjects={allProjects} onSelectCategory={setCategory} /></DeferredPanel>
            </>
          ) : (
            <>
          <MockScoringBanner />

          {loading && <p className="text-muted-foreground text-sm">{t('loading')}</p>}

          {!loading && projects.length === 0 && category && (
            <Card>
              <CardContent className="text-muted-foreground text-sm">
                {t('emptyCategory', { category: categoryLabel(category) })}
              </CardContent>
            </Card>
          )}

          {stats && (
            <div className="grid grid-cols-3 gap-3">
              <StatTile label={t('statCount')} value={stats.count} delay={0} />
              <StatTile label={t('statAvg')} value={stats.avg} decimals={1} delay={80} />
              <StatTile label={t('statTop')} value={stats.top.ai_score} decimals={1} delay={160} />
            </div>
          )}

          {projects.length > 0 && <Podium projects={projects} />}

          {projects.length > 0 && (
            <DeferredPanel><ScoreChart title={t('chartTitle')} description={t('chartDescription', { category: categoryLabel(category) })} projects={projects} /></DeferredPanel>
          )}

          {projects.length > 0 && (
            <div className="grid grid-cols-1 gap-4 lg:grid-cols-[1fr_280px]">
              <Card className="overflow-hidden py-0">
                <div className="flex flex-wrap items-center justify-between gap-2 border-b px-5 py-4">
                  <h3 className="text-sm font-semibold">{t('colProject')}</h3>
                  <div className="flex flex-wrap items-center gap-3">
                    <span className="text-muted-foreground text-xs">
                      {selectedIds.size > 0
                        ? t('compareSelected', { count: String(selectedIds.size) })
                        : `${filteredProjects.length} ${t('statCount').toLowerCase()}`}
                    </span>

                    <select
                      value={statusFilter}
                      onChange={(e) => setStatusFilter(e.target.value as typeof statusFilter)}
                      className="h-7 rounded-md border bg-background px-1.5 text-xs text-muted-foreground"
                    >
                      <option value="all">{t('filterAll')}</option>
                      <option value="new">{t('statusNew')}</option>
                      <option value="reviewing">{t('statusReviewing')}</option>
                      <option value="finalist">{t('statusFinalist')}</option>
                      <option value="rejected">{t('statusRejected')}</option>
                    </select>

                    <select
                      value={sortBy}
                      onChange={(e) => setSortBy(e.target.value as typeof sortBy)}
                      className="h-7 rounded-md border bg-background px-1.5 text-xs text-muted-foreground"
                    >
                      <option value="score">{t('sortByScore')}</option>
                      <option value="name">{t('sortByName')}</option>
                    </select>

                    {selectedIds.size >= 2 && (
                      <button
                        type="button"
                        onClick={() => setCompareOpen(true)}
                        className="text-primary flex items-center gap-1 text-xs font-medium hover:underline"
                      >
                        <Scale className="size-3.5" />
                        {t('compare')}
                      </button>
                    )}
                    <button
                      type="button"
                      onClick={() => setPresentationOpen(true)}
                      className="text-muted-foreground flex items-center gap-1 text-xs font-medium hover:text-foreground"
                    >
                      <Presentation className="size-3.5" />
                      {t('presentationMode')}
                    </button>
                  </div>
                </div>

                {filteredProjects.length === 0 ? (
                  <p className="text-muted-foreground px-5 py-6 text-sm">
                    {t('noSearchResults', { query: search })}
                  </p>
                ) : (
                  <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEnd}>
                    <Table>
                      <TableHeader>
                        <TableRow>
                          <TableHead />
                          <TableHead />
                          <TableHead>{t('colRank')}</TableHead>
                          <TableHead>{t('colProject')}</TableHead>
                          {(activeTemplate?.kpis ?? []).map((kpi) => (
                            <TableHead key={kpi.name}>
                              <Tooltip>
                                <TooltipTrigger className="flex items-center gap-1">
                                  {kpi.name}
                                  <Info className="size-3 text-muted-foreground" />
                                </TooltipTrigger>
                                <TooltipContent>
                                  {kpi.description} ({kpi.weight.toFixed(0)}%)
                                </TooltipContent>
                              </Tooltip>
                            </TableHead>
                          ))}
                          <TableHead className="text-right">{t('colScore')}</TableHead>
                        </TableRow>
                      </TableHeader>
                      <SortableContext items={filteredProjects.map((p) => p.id)} strategy={verticalListSortingStrategy}>
                        <TableBody>
                          {filteredProjects.map((project, i) => (
                            <SortableProjectRow
                              key={project.id}
                              project={project}
                              rank={project.manual_rank ?? i + 1}
                              index={i}
                              kpiOrder={kpiOrder}
                              onOpenDetail={setDetailProject}
                              selected={selectedIds.has(project.id)}
                              onToggleSelect={toggleSelect}
                              dragDisabled={sortBy !== 'score'}
                            />
                          ))}
                        </TableBody>
                      </SortableContext>
                    </Table>
                  </DndContext>
                )}
              </Card>

              <div className="flex flex-col gap-4">
                <KpiDonutPanel template={activeTemplate} />
                <DeferredPanel><ActivityPanel category={category} refreshKey={activityRefreshKey} /></DeferredPanel>
              </div>
            </div>
          )}
            </>
          )}
        </main>
      </div>

      <Suspense fallback={null}><ProjectDetailDialog
        project={detailProject}
        open={detailProject !== null}
        onOpenChange={(open) => !open && setDetailProject(null)}
        onProjectUpdated={applyProjectUpdate}
      /></Suspense>

      <Suspense fallback={null}><CompareDialog projects={selectedProjects} open={compareOpen} onOpenChange={setCompareOpen} /></Suspense>

      {presentationOpen && (
        <Suspense fallback={null}><PresentationMode projects={projects} category={category} onClose={() => setPresentationOpen(false)} /></Suspense>
      )}
    </div>
  );
}
