import { useEffect, useMemo, useState } from 'react';
import { CalendarDays, ClipboardList, Plus, Users } from 'lucide-react';

import {
  createCompetition,
  getCompetitionStages,
  getCompetitionTeams,
  getDemoDaySlots,
  getCompetitionReport,
  getCompetitions,
  type Competition,
  type CompetitionStage,
  type Team,
  type DemoDaySlot,
  type CompetitionReport,
} from '@/lib/api';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';

export function CompetitionOperationsPanel() {
  const [competitions, setCompetitions] = useState<Competition[]>([]);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [stages, setStages] = useState<CompetitionStage[]>([]);
  const [teams, setTeams] = useState<Team[]>([]);
  const [demoSlots, setDemoSlots] = useState<DemoDaySlot[]>([]);
  const [report, setReport] = useState<CompetitionReport | null>(null);
  const [name, setName] = useState('');
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getCompetitions()
      .then((items) => {
        setCompetitions(items);
        if (items[0]) setSelectedId(items[0].id);
      })
      .catch((e) => setError(e.message));
  }, []);

  useEffect(() => {
    if (selectedId === null) return;
    Promise.all([getCompetitionStages(selectedId), getCompetitionTeams(selectedId), getDemoDaySlots(selectedId), getCompetitionReport(selectedId)])
      .then(([stageItems, teamItems, slotItems, reportData]) => {
        setStages(stageItems);
        setTeams(teamItems);
        setDemoSlots(slotItems);
        setReport(reportData);
      })
      .catch((e) => setError(e.message));
  }, [selectedId]);

  const selected = useMemo(() => competitions.find((item) => item.id === selectedId), [competitions, selectedId]);

  async function handleCreate() {
    if (!name.trim()) return;
    setCreating(true);
    setError(null);
    try {
      const created = await createCompetition({ name: name.trim() });
      setCompetitions((items) => [created, ...items]);
      setSelectedId(created.id);
      setName('');
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setCreating(false);
    }
  }

  return (
    <Card className="border-primary/20">
      <CardHeader className="flex flex-row items-center justify-between gap-3">
        <div>
          <CardTitle className="text-base">Yarışma operasyonları</CardTitle>
          <p className="text-muted-foreground mt-1 text-xs">Başvuru, aşama ve takım akışının merkezi görünümü</p>
        </div>
        <div className="flex items-center gap-2">
          <Input
            value={name}
            onChange={(event) => setName(event.target.value)}
            placeholder="Yeni yarışma adı"
            className="h-8 w-44 text-xs"
            onKeyDown={(event) => event.key === 'Enter' && handleCreate()}
          />
          <Button size="sm" onClick={handleCreate} disabled={creating || !name.trim()}>
            <Plus className="mr-1 size-3.5" /> Oluştur
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {error && <p className="text-destructive text-xs">{error}</p>}
        {competitions.length === 0 ? (
          <p className="text-muted-foreground text-sm">Henüz yarışma oluşturulmadı.</p>
        ) : (
          <>
            <select
              value={selectedId ?? ''}
              onChange={(event) => setSelectedId(Number(event.target.value))}
              className="h-9 w-full rounded-md border bg-background px-3 text-sm"
            >
              {competitions.map((competition) => (
                <option key={competition.id} value={competition.id}>{competition.name}</option>
              ))}
            </select>
            {selected && (
              <>
              <div className="grid gap-3 sm:grid-cols-3">
                <div className="rounded-lg border p-3">
                  <div className="text-muted-foreground flex items-center gap-2 text-xs"><CalendarDays className="size-3.5" />Başvuru dönemi</div>
                  <p className="mt-2 text-sm font-medium">{selected.application_start ?? 'Tanımlanmadı'} — {selected.application_end ?? 'Tanımlanmadı'}</p>
                </div>
                <div className="rounded-lg border p-3">
                  <div className="text-muted-foreground flex items-center gap-2 text-xs"><ClipboardList className="size-3.5" />Değerlendirme aşaması</div>
                  <p className="mt-2 text-sm font-medium">{stages.length} aşama</p>
                </div>
                <div className="rounded-lg border p-3">
                  <div className="text-muted-foreground flex items-center gap-2 text-xs"><Users className="size-3.5" />Takımlar</div>
                  <p className="mt-2 text-sm font-medium">{teams.length} takım</p>
                </div>
              </div>
              {stages.length > 0 && (
                <div className="rounded-lg border p-3">
                  <p className="mb-2 text-xs font-semibold tracking-wide uppercase text-muted-foreground">Değerlendirme aşamaları</p>
                  <div className="flex flex-wrap gap-2">
                    {stages.map((stage) => <span key={stage.id} className="rounded-full bg-secondary px-2.5 py-1 text-xs">{stage.position}. {stage.name}</span>)}
                  </div>
                </div>
              )}
              {teams.length > 0 && (
                <div className="rounded-lg border p-3">
                  <p className="mb-2 text-xs font-semibold tracking-wide uppercase text-muted-foreground">Takımlar ve finalist durumu</p>
                  <div className="space-y-1.5">
                    {teams.map((team) => <div key={team.id} className="flex items-center justify-between text-xs"><span className="font-medium">{team.name}</span><span className={team.status === 'finalist' ? 'text-primary font-semibold' : 'text-muted-foreground'}>{team.status}</span></div>)}
                  </div>
                </div>
              )}
              {demoSlots.length > 0 && (
                <div className="rounded-lg border p-3">
                  <p className="mb-2 text-xs font-semibold tracking-wide uppercase text-muted-foreground">Demo Day programı</p>
                  <div className="space-y-1.5">{demoSlots.map((slot) => <div key={slot.id} className="flex items-center justify-between text-xs"><span>#{slot.slot_order} · Takım {slot.team_id}</span><span className="text-muted-foreground">{slot.room} · {slot.starts_at}</span></div>)}</div>
                </div>
              )}
              {report && (
                <div className="rounded-lg border p-3">
                  <p className="mb-2 text-xs font-semibold tracking-wide uppercase text-muted-foreground">Yarışma rapor özeti</p>
                  <div className="grid grid-cols-2 gap-2 text-xs sm:grid-cols-4">
                    <span><strong>{report.total_teams}</strong> takım</span>
                    <span><strong>{report.finalist_teams}</strong> finalist</span>
                    <span><strong>{report.submitted_deliverables}</strong> teslim</span>
                    <span><strong>{report.demo_day_slots}</strong> sunum slotu</span>
                  </div>
                </div>
              )}
              </>
            )}
          </>
        )}
      </CardContent>
    </Card>
  );
}
