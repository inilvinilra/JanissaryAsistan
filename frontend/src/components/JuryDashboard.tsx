import { useEffect, useMemo, useState } from 'react';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip as ChartTooltip, ResponsiveContainer } from 'recharts';
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

import { getCategories, getProjects, updateRanking, type CategoryTemplate, type Project } from '@/lib/api';
import { useLocale } from '@/lib/locale-context';
import { useToast } from '@/lib/toast-context';
import { useJuror } from '@/lib/juror-context';
import { exportProjectsCsv } from '@/lib/export-csv';
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card';
import { Table, TableHeader, TableBody, TableRow, TableHead } from '@/components/ui/table';
import { SortableProjectRow } from '@/components/SortableProjectRow';
import { Sidebar } from '@/components/Sidebar';
import { Topbar } from '@/components/Topbar';
import { Podium } from '@/components/Podium';
import { KpiDonutPanel } from '@/components/KpiDonutPanel';
import { ActivityPanel } from '@/components/ActivityPanel';
import { MockScoringBanner } from '@/components/MockScoringBanner';
import { ProjectDetailDialog } from '@/components/ProjectDetailDialog';
import { CompareDialog } from '@/components/CompareDialog';
import { PresentationMode } from '@/components/PresentationMode';
import { Overview } from '@/components/Overview';
import { CompetitionManager } from '@/components/CompetitionManager';
import { UserManager } from '@/components/UserManager';
import { AuditLogPanel } from '@/components/AuditLogPanel';
import { ReportsPanel } from '@/components/ReportsPanel';
import { SettingsPanel } from '@/components/SettingsPanel';
import { NotificationCenter } from '@/components/NotificationCenter';
import { CompetitionOperationsPanel } from '@/components/CompetitionOperationsPanel';
import { StatTile } from '@/components/StatTile';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';

const POLL_INTERVAL_MS = 20000;

export function JuryDashboard() {
  const { t, categoryLabel } = useLocale();
  const { showToast } = useToast();
  const { jurorName } = useJuror();
  const [categories, setCategories] = useState<CategoryTemplate[]>([]);
  const [category, setCategory] = useState<string>('');
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

  const sensors = useSensors(
    useSensor(PointerSensor, { activationConstraint: { distance: 4 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );

  // Category starts unset — the app opens on the Overview (all fields), not a
  // pre-picked one.
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

  // Lightweight "live sync": periodically re-fetch so a change made by another
  // juror on another tab/device eventually shows up here too, without a
  // WebSocket server. Skipped while a project's detail dialog is open so an
  // in-progress edit isn't clobbered mid-type.
  useEffect(() => {
    if (detailProject) return;
    const id = setInterval(() => {
      if (category) getProjects(category).then(setProjects).catch(() => {});
      else getProjects().then(setAllProjects).catch(() => {});
    }, POLL_INTERVAL_MS);
    return () => clearInterval(id);
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
    // Reordering only makes sense against the actual rank order; a name-sorted
    // view is a read-only convenience, not a place to drag from.
    if (sortBy !== 'score') return;

    const { active, over } = event;
    if (!over || active.id === over.id) return;

    const oldIndex = projects.findIndex((p) => p.id === active.id);
    const newIndex = projects.findIndex((p) => p.id === over.id);
    const reordered = arrayMove(projects, oldIndex, newIndex);
    setProjects(reordered);

    try {
      await updateRanking(category, reordered.map((p) => p.id), jurorName || 'jury');
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
          onProjectCreated={(project) => {
            if (project.category === category) setProjects((prev) => [...prev, project]);
            setAllProjects((prev) => [...prev, project]);
            showToast(t('toastProjectAdded', { name: project.name }), 'success');
          }}
        />}

        <main className="mx-auto w-full max-w-7xl space-y-6 px-4 py-6 sm:px-6 lg:px-8">
          {view === 'competitions' ? (
            <CompetitionManager />
          ) : view === 'users' ? (
            <UserManager />
          ) : view === 'audit' ? (
            <AuditLogPanel />
          ) : view === 'reports' ? (
            <ReportsPanel />
          ) : view === 'settings' ? (
            <SettingsPanel />
          ) : view === 'notifications' ? (
            <NotificationCenter />
          ) : category === '' ? (
            <>
              <CompetitionOperationsPanel />
              <Overview categories={categories} allProjects={allProjects} onSelectCategory={setCategory} />
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
            <Card>
              <CardHeader>
                <CardTitle>{t('chartTitle')}</CardTitle>
                <CardDescription>{t('chartDescription', { category: categoryLabel(category) })}</CardDescription>
              </CardHeader>
              <CardContent className="h-64 w-full pl-0">
                <ResponsiveContainer>
                  <BarChart data={projects} margin={{ top: 8, right: 16, left: 0, bottom: 8 }} barCategoryGap="28%">
                    <defs>
                      <linearGradient id="scoreBarFill" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stopColor="var(--chart-1)" stopOpacity={0.95} />
                        <stop offset="100%" stopColor="var(--chart-1)" stopOpacity={0.55} />
                      </linearGradient>
                    </defs>
                    <CartesianGrid strokeDasharray="3 3" stroke="var(--border)" vertical={false} />
                    <XAxis
                      dataKey="name"
                      tick={{ fontSize: 12, fill: 'var(--muted-foreground)' }}
                      interval={0}
                      angle={-15}
                      textAnchor="end"
                      height={50}
                    />
                    <YAxis domain={[0, 100]} tick={{ fontSize: 12, fill: 'var(--muted-foreground)' }} width={32} />
                    <ChartTooltip
                      cursor={{ fill: 'var(--muted)' }}
                      contentStyle={{
                        background: 'var(--popover)',
                        color: 'var(--popover-foreground)',
                        border: '1px solid var(--border)',
                        borderRadius: 'var(--radius-md)',
                        fontSize: 12,
                      }}
                    />
                    <Bar
                      dataKey="ai_score"
                      fill="url(#scoreBarFill)"
                      radius={[8, 8, 0, 0]}
                      maxBarSize={56}
                      animationDuration={900}
                      animationEasing="ease-out"
                    />
                  </BarChart>
                </ResponsiveContainer>
              </CardContent>
            </Card>
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
                <ActivityPanel category={category} refreshKey={activityRefreshKey} />
              </div>
            </div>
          )}
            </>
          )}
        </main>
      </div>

      <ProjectDetailDialog
        project={detailProject}
        open={detailProject !== null}
        onOpenChange={(open) => !open && setDetailProject(null)}
        onProjectUpdated={applyProjectUpdate}
      />

      <CompareDialog projects={selectedProjects} open={compareOpen} onOpenChange={setCompareOpen} />

      {presentationOpen && (
        <PresentationMode projects={projects} category={category} onClose={() => setPresentationOpen(false)} />
      )}
    </div>
  );
}
