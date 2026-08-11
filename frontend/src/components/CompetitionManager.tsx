import { useEffect, useState, type FormEvent } from 'react';
import { CalendarDays, Plus, Trash2, Trophy } from 'lucide-react';

import {
  createCompetition,
  createCompetitionCategory,
  createCompetitionStage,
  addTeamMember,
  getCompetitionCategories,
  getCompetitionStages,
  updateStageStatus,
  getCompetitionTeams,
  getCompetitions,
  createTeam,
  getCategories,
  updateKpiTemplate,
  updateTeamStatus,
  type Competition,
  type CompetitionCategory,
  type CompetitionStage,
  type CategoryTemplate,
  type Team,
} from '@/lib/api';
import { useLocale } from '@/lib/locale-context';
import { useToast } from '@/lib/toast-context';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';

export function CompetitionManager() {
  const { t } = useLocale();
  const { showToast } = useToast();
  const [competitions, setCompetitions] = useState<Competition[]>([]);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [stages, setStages] = useState<CompetitionStage[]>([]);
  const [categories, setCategories] = useState<CompetitionCategory[]>([]);
  const [teams, setTeams] = useState<Team[]>([]);
  const [kpiTemplates, setKpiTemplates] = useState<CategoryTemplate[]>([]);
  const [selectedKpiCategory, setSelectedKpiCategory] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [newCompetition, setNewCompetition] = useState({ name: '', description: '', application_start: '', application_end: '', organization: 'T3 Vakfı' });
  const [newStage, setNewStage] = useState({ name: '', stage_type: 'review', position: 1, starts_at: '', ends_at: '', passing_score: 0, finalist_limit: 0, results_at: '' });
  const [newCategory, setNewCategory] = useState({ name: '', slug: '', parent_id: '', kpi_category: '' });
  const [newTeamName, setNewTeamName] = useState('');
  const [teamDialogOpen, setTeamDialogOpen] = useState(false);
  const [teamCompetitionId, setTeamCompetitionId] = useState<number | null>(null);
  const [selectedTeamId, setSelectedTeamId] = useState<number | null>(null);
  const [newMember, setNewMember] = useState({ full_name: '', email: '', role: '', is_scholar: false });
  const [newKpiCategory, setNewKpiCategory] = useState('');

  const selected = competitions.find((competition) => competition.id === selectedId) ?? null;
  const selectedTeam = teams.find((team) => team.id === selectedTeamId) ?? null;

  useEffect(() => {
    getCompetitions()
      .then((items) => {
        setCompetitions(items);
        setSelectedId(items[0]?.id ?? null);
        setTeamCompetitionId(items[0]?.id ?? null);
      })
      .catch((e) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    getCategories().then((items) => {
      setKpiTemplates(items);
      setSelectedKpiCategory(items[0]?.category ?? '');
    }).catch((e) => setError(e.message));
  }, []);

  useEffect(() => {
    if (!selectedId) {
      setStages([]);
      setCategories([]);
      return;
    }
    Promise.all([getCompetitionStages(selectedId), getCompetitionCategories(selectedId), getCompetitionTeams(selectedId)])
      .then(([stageItems, categoryItems, teamItems]) => {
        setStages(stageItems);
        setCategories(categoryItems);
        setTeams(teamItems);
        setSelectedTeamId(teamItems[0]?.id ?? null);
      })
      .catch((e) => setError(e.message));
  }, [selectedId]);

  useEffect(() => {
    if (selectedId !== null) setTeamCompetitionId(selectedId);
  }, [selectedId]);

  async function handleCreateCompetition(event: FormEvent) {
    event.preventDefault();
    if (!newCompetition.name.trim()) return;
    try {
      const created = await createCompetition({
        name: newCompetition.name.trim(),
        description: newCompetition.description.trim(),
        application_start: newCompetition.application_start || undefined,
        application_end: newCompetition.application_end || undefined,
        organization: newCompetition.organization.trim() || 'T3 Vakfı',
      });
      setCompetitions((items) => [created, ...items]);
      setSelectedId(created.id);
      setNewCompetition({ name: '', description: '', application_start: '', application_end: '', organization: 'T3 Vakfı' });
      showToast(t('competitionCreated'), 'success');
    } catch (e) {
      showToast((e as Error).message, 'error');
    }
  }

  async function handleCreateStage(event: FormEvent) {
    event.preventDefault();
    if (!selectedId || !newStage.name.trim()) return;
    try {
      const created = await createCompetitionStage(selectedId, {
        ...newStage,
        name: newStage.name.trim(),
        starts_at: newStage.starts_at || null,
        ends_at: newStage.ends_at || null,
      });
      setStages((items) => [...items, created].sort((a, b) => a.position - b.position));
      setNewStage({ name: '', stage_type: 'review', position: stages.length + 2, starts_at: '', ends_at: '', passing_score: 0, finalist_limit: 0, results_at: '' });
      showToast(t('stageCreated'), 'success');
    } catch (e) {
      showToast((e as Error).message, 'error');
    }
  }

  async function handleCreateCategory(event: FormEvent) {
    event.preventDefault();
    if (!selectedId || !newCategory.name.trim() || !newCategory.slug.trim()) return;
    try {
      const created = await createCompetitionCategory(selectedId, {
        name: newCategory.name.trim(), slug: newCategory.slug.trim(),
        parent_id: newCategory.parent_id ? Number(newCategory.parent_id) : null,
        kpi_category: newCategory.kpi_category.trim() || null,
      });
      setCategories((items) => [...items, created]);
      setNewCategory({ name: '', slug: '', parent_id: '', kpi_category: '' });
      showToast(t('categoryCreated'), 'success');
    } catch (e) {
      showToast((e as Error).message, 'error');
    }
  }

  async function handleSaveKpis() {
    const template = kpiTemplates.find((item) => item.category === selectedKpiCategory);
    if (!template) return;
    const total = template.kpis.reduce((sum, kpi) => sum + kpi.weight, 0);
    if (Math.abs(total - 100) > 0.01) {
      showToast(t('kpiWeightsMustTotal'), 'error');
      return;
    }
    try {
      const updated = await updateKpiTemplate(template.category, template.kpis);
      setKpiTemplates((items) => items.map((item) => item.category === updated.category ? updated : item));
      showToast(t('kpiSaved'), 'success');
    } catch (e) {
      showToast((e as Error).message, 'error');
    }
  }

  async function handleCreateTeam(event: FormEvent) {
    event.preventDefault();
    if (!selectedId || !newTeamName.trim()) return;
    try {
      const created = await createTeam(selectedId, newTeamName.trim());
      setTeams((items) => [created, ...items]);
      setNewTeamName('');
      showToast(t('teamCreated'), 'success');
    } catch (e) {
      showToast((e as Error).message, 'error');
    }
  }

  async function handleQuickCreateTeam(event: FormEvent) {
    event.preventDefault();
    if (!teamCompetitionId || !newTeamName.trim()) return;
    try {
      const created = await createTeam(teamCompetitionId, newTeamName.trim());
      if (teamCompetitionId === selectedId) {
        setTeams((items) => [created, ...items]);
        setSelectedTeamId(created.id);
      }
      setNewTeamName('');
      setTeamDialogOpen(false);
      showToast(t('teamCreated'), 'success');
    } catch (e) {
      showToast((e as Error).message, 'error');
    }
  }

  async function handleTeamStatusChange(status: string) {
    if (!selectedTeam) return;
    try {
      await updateTeamStatus(selectedTeam.id, status);
      setTeams((items) => items.map((team) => team.id === selectedTeam.id ? { ...team, status } : team));
      showToast(t('teamStatusUpdated'), 'success');
    } catch (e) {
      showToast((e as Error).message, 'error');
    }
  }

  async function handleAddMember(event: FormEvent) {
    event.preventDefault();
    if (!selectedTeam || !newMember.full_name.trim() || !newMember.email.trim()) return;
    try {
      const member = await addTeamMember(selectedTeam.id, {
        full_name: newMember.full_name.trim(),
        email: newMember.email.trim(),
        role: newMember.role.trim() || undefined,
        is_scholar: newMember.is_scholar,
      });
      setTeams((items) => items.map((team) => team.id === selectedTeam.id ? { ...team, members: [...team.members, member] } : team));
      setNewMember({ full_name: '', email: '', role: '', is_scholar: false });
      showToast(t('memberAdded'), 'success');
    } catch (e) {
      showToast((e as Error).message, 'error');
    }
  }

  function addKpi() {
    if (!selectedKpiCategory) return;
    setKpiTemplates((items) => items.map((item) => item.category === selectedKpiCategory
      ? { ...item, kpis: [...item.kpis, { name: '', weight: 0, description: '' }] }
      : item));
  }

  function addKpiCategory() {
    const category = newKpiCategory.trim().toLowerCase().replace(/\s+/g, '-');
    if (!category || kpiTemplates.some((item) => item.category === category)) return;
    setKpiTemplates((items) => [...items, { category, kpis: [{ name: '', weight: 100, description: '' }] }]);
    setSelectedKpiCategory(category);
    setNewKpiCategory('');
  }

  function removeKpi(index: number) {
    setKpiTemplates((items) => items.map((item) => item.category === selectedKpiCategory
      ? { ...item, kpis: item.kpis.filter((_, entryIndex) => entryIndex !== index) }
      : item));
  }

  if (error) return <p className="text-destructive text-sm">{t('errorPrefix')} {error}</p>;
  if (loading) return <p className="text-muted-foreground text-sm">{t('loading')}</p>;

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h2 className="text-xl font-semibold tracking-tight">{t('competitionsTitle')}</h2>
          <p className="text-muted-foreground mt-1 text-sm">{t('competitionsDescription')}</p>
        </div>
        <Button type="button" onClick={() => setTeamDialogOpen(true)} disabled={competitions.length === 0}>
          <Plus className="size-4" />{t('quickAddTeam')}
        </Button>
      </div>

      <Dialog open={teamDialogOpen} onOpenChange={setTeamDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('quickAddTeam')}</DialogTitle>
            <DialogDescription>{t('quickAddTeamDescription')}</DialogDescription>
          </DialogHeader>
          <form onSubmit={handleQuickCreateTeam} className="space-y-4">
            <select value={teamCompetitionId ?? ''} onChange={(e) => setTeamCompetitionId(Number(e.target.value))} className="h-9 w-full rounded-md border bg-background px-3 text-sm" required>
              {competitions.map((competition) => <option key={competition.id} value={competition.id}>{competition.name}</option>)}
            </select>
            <Input value={newTeamName} onChange={(e) => setNewTeamName(e.target.value)} placeholder={t('teamName')} required />
            <DialogFooter><Button type="submit">{t('addTeam')}</Button></DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <div className="grid gap-4 lg:grid-cols-[280px_1fr]">
        <Card>
          <CardHeader><CardTitle className="text-sm">{t('competitionList')}</CardTitle></CardHeader>
          <CardContent className="space-y-2">
            {competitions.length === 0 && <p className="text-muted-foreground text-sm">{t('noCompetitions')}</p>}
            {competitions.map((competition) => (
              <button key={competition.id} type="button" onClick={() => setSelectedId(competition.id)}
                className={`flex w-full items-center gap-2 rounded-md border px-3 py-2 text-left text-sm ${selectedId === competition.id ? 'border-primary bg-accent' : 'hover:bg-accent/50'}`}>
                <Trophy className="size-4 shrink-0" />
                <span className="truncate">{competition.name}</span>
              </button>
            ))}
            <form onSubmit={handleCreateCompetition} className="space-y-2 border-t pt-3">
              <Input placeholder={t('competitionName')} value={newCompetition.name} onChange={(e) => setNewCompetition({ ...newCompetition, name: e.target.value })} required />
              <Input placeholder={t('competitionDescription')} value={newCompetition.description} onChange={(e) => setNewCompetition({ ...newCompetition, description: e.target.value })} />
              <Input placeholder={t('organizationName')} value={newCompetition.organization} onChange={(e) => setNewCompetition({ ...newCompetition, organization: e.target.value })} />
              <div className="grid grid-cols-2 gap-2">
                <Input type="date" value={newCompetition.application_start} onChange={(e) => setNewCompetition({ ...newCompetition, application_start: e.target.value })} />
                <Input type="date" value={newCompetition.application_end} onChange={(e) => setNewCompetition({ ...newCompetition, application_end: e.target.value })} />
              </div>
              <Button type="submit" size="sm" className="w-full"><Plus className="size-3.5" />{t('createCompetition')}</Button>
            </form>
          </CardContent>
        </Card>

        {selected ? (
          <div className="space-y-4">
            <Card>
              <CardHeader>
                <CardTitle>{selected.name}</CardTitle>
                <CardDescription>{selected.description || t('noCompetitionDescription')}</CardDescription>
                <p className="text-muted-foreground text-xs">{t('organizationName')}: {selected.organization}</p>
              </CardHeader>
              <CardContent className="flex items-center gap-2 text-xs text-muted-foreground">
                <CalendarDays className="size-4" />
                {selected.application_start || '—'} → {selected.application_end || '—'}
              </CardContent>
            </Card>

            <div className="grid gap-4 xl:grid-cols-2">
              <Card>
                <CardHeader><CardTitle className="text-sm">{t('stagesTitle')}</CardTitle></CardHeader>
                <CardContent className="space-y-2">
                  {stages.map((stage) => <div key={stage.id} className="flex items-center justify-between gap-2 rounded-md border px-3 py-2 text-sm"><span>{stage.position}. {stage.name}<span className="text-muted-foreground ml-2 text-xs">{stage.passing_score > 0 ? `≥ ${stage.passing_score}` : ''}{stage.finalist_limit ? ` · ${stage.finalist_limit} finalist` : ''}{stage.results_at ? ` · ${stage.results_at}` : ''}</span></span><div className="flex items-center gap-2"><select value={stage.status} onChange={async (e) => { try { const updated = await updateStageStatus(selectedId!, stage.id, e.target.value as CompetitionStage['status']); setStages((items) => items.map((item) => item.id === updated.id ? updated : item)); showToast(t('stageStatusUpdated'), 'success'); } catch (error) { showToast((error as Error).message, 'error'); } }} className="h-7 rounded-md border bg-background px-1.5 text-xs"><option value="planned">planned</option><option value="active">active</option><option value="completed">completed</option><option value="locked">locked</option></select><span className="text-muted-foreground text-xs">{stage.stage_type}</span></div></div>)}
                  <form onSubmit={handleCreateStage} className="space-y-2 border-t pt-3">
                    <div className="grid grid-cols-[1fr_92px] gap-2"><Input placeholder={t('stageName')} value={newStage.name} onChange={(e) => setNewStage({ ...newStage, name: e.target.value })} required /><Input type="number" min="1" value={newStage.position} onChange={(e) => setNewStage({ ...newStage, position: Number(e.target.value) })} /></div>
                    <Input placeholder={t('stageType')} value={newStage.stage_type} onChange={(e) => setNewStage({ ...newStage, stage_type: e.target.value })} required />
                    <div className="grid grid-cols-3 gap-2"><Input type="number" min="0" max="100" placeholder={t('passingScore')} value={newStage.passing_score || ''} onChange={(e) => setNewStage({ ...newStage, passing_score: Number(e.target.value) })} /><Input type="number" min="0" placeholder={t('finalistLimit')} value={newStage.finalist_limit || ''} onChange={(e) => setNewStage({ ...newStage, finalist_limit: Number(e.target.value) })} /><Input type="datetime-local" value={newStage.results_at} onChange={(e) => setNewStage({ ...newStage, results_at: e.target.value })} /></div>
                    <Button type="submit" size="sm"><Plus className="size-3.5" />{t('addStage')}</Button>
                  </form>
                </CardContent>
              </Card>

              <Card>
                <CardHeader><CardTitle className="text-sm">{t('competitionCategoriesTitle')}</CardTitle></CardHeader>
                <CardContent className="space-y-2">
                  {categories.map((category) => <div key={category.id} className="flex items-center justify-between rounded-md border px-3 py-2 text-sm"><span>{category.parent_id ? '↳ ' : ''}{category.name}</span><span className="text-muted-foreground text-xs">{category.kpi_category || category.slug}</span></div>)}
                  <form onSubmit={handleCreateCategory} className="space-y-2 border-t pt-3">
                    <Input placeholder={t('categoryName')} value={newCategory.name} onChange={(e) => setNewCategory({ ...newCategory, name: e.target.value })} required />
                    <div className="grid grid-cols-2 gap-2"><Input placeholder={t('categorySlug')} value={newCategory.slug} onChange={(e) => setNewCategory({ ...newCategory, slug: e.target.value })} required /><Input placeholder={t('kpiCategory')} value={newCategory.kpi_category} onChange={(e) => setNewCategory({ ...newCategory, kpi_category: e.target.value })} /></div>
                    <Button type="submit" size="sm"><Plus className="size-3.5" />{t('addCategory')}</Button>
                  </form>
                </CardContent>
              </Card>
            </div>

            <Card>
              <CardHeader><CardTitle className="text-sm">{t('teamManageTitle')}</CardTitle><CardDescription>{t('teamManageDescription')}</CardDescription></CardHeader>
              <CardContent className="space-y-3">
                {teams.length === 0 && <p className="text-muted-foreground text-sm">{t('noTeams')}</p>}
                {teams.length > 0 && <div className="grid gap-2 sm:grid-cols-2">{teams.map((team) => <button key={team.id} type="button" onClick={() => setSelectedTeamId(team.id)} className={`flex items-center justify-between rounded-md border px-3 py-3 text-left text-sm transition-colors ${selectedTeamId === team.id ? 'border-primary bg-accent' : 'hover:bg-accent/50'}`}><span><span className="block font-medium">{team.name}</span><span className="text-muted-foreground text-xs">{team.members.length} {t('teamMembers').toLowerCase()}</span></span><span className="rounded-full bg-muted px-2 py-1 text-xs">{team.status}</span></button>)}</div>}

                {selectedTeam && <div className="grid gap-4 border-t pt-4 lg:grid-cols-[1fr_1.2fr]">
                  <div className="space-y-3">
                    <div className="flex items-center justify-between gap-2"><div><p className="font-medium">{selectedTeam.name}</p><p className="text-muted-foreground text-xs">ID: {selectedTeam.id}</p></div><select value={selectedTeam.status} onChange={(e) => handleTeamStatusChange(e.target.value)} aria-label={t('teamStatus')} className="h-9 rounded-md border bg-background px-2 text-sm"><option value="new">new</option><option value="reviewing">reviewing</option><option value="finalist">finalist</option><option value="rejected">rejected</option><option value="winner">winner</option></select></div>
                    <div><p className="mb-2 text-sm font-medium">{t('teamMembers')}</p>{selectedTeam.members.length === 0 ? <p className="text-muted-foreground text-xs">{t('noMembers')}</p> : <div className="space-y-2">{selectedTeam.members.map((member) => <div key={member.id} className="rounded-md border px-3 py-2 text-sm"><div className="flex items-center justify-between gap-2"><span className="font-medium">{member.full_name}</span>{member.is_scholar && <span className="text-xs text-muted-foreground">{t('scholar')}</span>}</div><p className="text-muted-foreground text-xs">{member.email}{member.role ? ` · ${member.role}` : ''}</p></div>)}</div>}</div>
                  </div>
                  <form onSubmit={handleAddMember} className="space-y-2 rounded-md border p-3"><p className="text-sm font-medium">{t('addMember')}</p><div className="grid gap-2 sm:grid-cols-2"><Input value={newMember.full_name} onChange={(e) => setNewMember({ ...newMember, full_name: e.target.value })} placeholder={t('memberName')} required /><Input type="email" value={newMember.email} onChange={(e) => setNewMember({ ...newMember, email: e.target.value })} placeholder={t('memberEmail')} required /></div><Input value={newMember.role} onChange={(e) => setNewMember({ ...newMember, role: e.target.value })} placeholder={t('memberRole')} /><label className="flex items-center gap-2 text-xs text-muted-foreground"><input type="checkbox" checked={newMember.is_scholar} onChange={(e) => setNewMember({ ...newMember, is_scholar: e.target.checked })} />{t('memberScholar')}</label><Button type="submit" size="sm"><Plus className="size-3.5" />{t('addMember')}</Button></form>
                </div>}
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="text-sm">{t('kpiManagementTitle')}</CardTitle>
                <CardDescription>{t('kpiManagementDescription')}</CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="flex flex-wrap gap-2">
                  <select value={selectedKpiCategory} onChange={(e) => setSelectedKpiCategory(e.target.value)} className="h-9 min-w-48 flex-1 rounded-md border bg-background px-3 text-sm">
                    {kpiTemplates.map((template) => <option key={template.category} value={template.category}>{template.category}</option>)}
                  </select>
                  <Input value={newKpiCategory} onChange={(e) => setNewKpiCategory(e.target.value)} placeholder={t('newKpiCategory')} className="h-9 w-44" />
                  <Button type="button" size="sm" variant="outline" onClick={addKpiCategory}><Plus className="size-3.5" />{t('addKpiCategory')}</Button>
                </div>
                {kpiTemplates.find((template) => template.category === selectedKpiCategory)?.kpis.map((kpi, index) => (
                  <div key={`${selectedKpiCategory}-${index}`} className="grid gap-2 rounded-md border p-3 sm:grid-cols-[1fr_1fr_84px_32px] sm:items-center">
                    <Input value={kpi.name} placeholder={t('kpiName')} onChange={(e) => setKpiTemplates((items) => items.map((item) => item.category === selectedKpiCategory ? { ...item, kpis: item.kpis.map((entry, entryIndex) => entryIndex === index ? { ...entry, name: e.target.value } : entry) } : item))} />
                    <Input value={kpi.description} placeholder={t('kpiDescription')} onChange={(e) => setKpiTemplates((items) => items.map((item) => item.category === selectedKpiCategory ? { ...item, kpis: item.kpis.map((entry, entryIndex) => entryIndex === index ? { ...entry, description: e.target.value } : entry) } : item))} />
                    <Input type="number" min="0" max="100" step="1" value={kpi.weight} aria-label={t('kpiWeight')} onChange={(e) => setKpiTemplates((items) => items.map((item) => item.category === selectedKpiCategory ? { ...item, kpis: item.kpis.map((entry, entryIndex) => entryIndex === index ? { ...entry, weight: Number(e.target.value) } : entry) } : item))} />
                    <Button type="button" size="icon" variant="ghost" aria-label={t('removeKpi')} onClick={() => removeKpi(index)}><Trash2 className="size-4 text-destructive" /></Button>
                  </div>
                ))}
                <div className="flex flex-wrap items-center justify-between gap-2 border-t pt-3"><span className="text-muted-foreground text-xs">{t('totalLabel')}</span><span className="font-data text-sm font-semibold">{(kpiTemplates.find((template) => template.category === selectedKpiCategory)?.kpis.reduce((sum, kpi) => sum + kpi.weight, 0) ?? 0).toFixed(0)}%</span></div>
                <div className="flex gap-2"><Button type="button" size="sm" variant="outline" onClick={addKpi}><Plus className="size-3.5" />{t('addKpi')}</Button><Button type="button" size="sm" onClick={handleSaveKpis}>{t('saveKpis')}</Button></div>
              </CardContent>
            </Card>
          </div>
        ) : (
          <Card><CardContent className="text-muted-foreground py-10 text-sm">{t('selectCompetition')}</CardContent></Card>
        )}
      </div>
    </div>
  );
}
