import { useEffect, useMemo, useState } from 'react';
import { CalendarDays, ClipboardList, Plus, Users } from 'lucide-react';

import {
  createCompetition,
  getCompetitionStages,
  getCompetitionTeams,
  getDemoDaySlots,
  getCompetitionReport,
  getCompetitions,
  updateDemoDaySlot,
  finalizeCompetition,
  type Competition,
  type CompetitionStage,
  type Team,
  type DemoDaySlot,
  type CompetitionReport,
} from '@/lib/api';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { useLocale } from '@/lib/locale-context';
import { AssessmentProgressPanel } from '@/components/AssessmentProgressPanel';

export function CompetitionOperationsPanel() {
  const { t } = useLocale();
  const canRunAssessments = ['system_admin', 'competition_manager', 'chief_judge', 'evaluation_manager'].includes(
    typeof localStorage === 'undefined'
      ? ''
      : (() => { try { return JSON.parse(localStorage.getItem('jury-auth-user') ?? '{}').role ?? ''; } catch { return ''; } })(),
  );
  const [competitions, setCompetitions] = useState<Competition[]>([]);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [stages, setStages] = useState<CompetitionStage[]>([]);
  const [teams, setTeams] = useState<Team[]>([]);
  const [demoSlots, setDemoSlots] = useState<DemoDaySlot[]>([]);
  const [report, setReport] = useState<CompetitionReport | null>(null);
  const [name, setName] = useState('');
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [checklistText, setChecklistText] = useState<Record<number, string>>({});
  const [finalMinutes, setFinalMinutes] = useState(''); const [finalSigner, setFinalSigner] = useState('');

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
          <CardTitle className="text-base">{t('competitionOperationsTitle')}</CardTitle>
          <p className="text-muted-foreground mt-1 text-xs">{t('competitionOperationsDescription')}</p>
        </div>
        <div className="flex items-center gap-2">
          <Input
            value={name}
            onChange={(event) => setName(event.target.value)}
            placeholder={t('newCompetitionName')}
            className="h-8 w-44 text-xs"
            onKeyDown={(event) => event.key === 'Enter' && handleCreate()}
          />
          <Button size="sm" onClick={handleCreate} disabled={creating || !name.trim()}>
            <Plus className="mr-1 size-3.5" /> {t('create')}
          </Button>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {error && <p className="text-destructive text-xs">{error}</p>}
        {competitions.length === 0 ? (
          <p className="text-muted-foreground text-sm">{t('noCompetitionCreated')}</p>
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
                  <div className="text-muted-foreground flex items-center gap-2 text-xs"><CalendarDays className="size-3.5" />{t('applicationPeriod')}</div>
                  <p className="mt-2 text-sm font-medium">{selected.application_start ?? t('notDefined')} — {selected.application_end ?? t('notDefined')}</p>
                </div>
                <div className="rounded-lg border p-3">
                  <div className="text-muted-foreground flex items-center gap-2 text-xs"><ClipboardList className="size-3.5" />{t('evaluationStage')}</div>
                  <p className="mt-2 text-sm font-medium">{t('stageCount', { count: String(stages.length) })}</p>
                </div>
                <div className="rounded-lg border p-3">
                  <div className="text-muted-foreground flex items-center gap-2 text-xs"><Users className="size-3.5" />{t('teamsLabel')}</div>
                  <p className="mt-2 text-sm font-medium">{t('teamCount', { count: String(teams.length) })}</p>
                </div>
              </div>
              {stages.length > 0 && (
                <div className="rounded-lg border p-3">
                  <p className="mb-2 text-xs font-semibold tracking-wide uppercase text-muted-foreground">{t('evaluationStages')}</p>
                  <div className="flex flex-wrap gap-2">
                    {stages.map((stage) => <span key={stage.id} className="rounded-full bg-secondary px-2.5 py-1 text-xs">{stage.position}. {stage.name}</span>)}
                  </div>
                </div>
              )}
              {teams.length > 0 && (
                <div className="rounded-lg border p-3">
                  <p className="mb-2 text-xs font-semibold tracking-wide uppercase text-muted-foreground">{t('teamsAndFinalists')}</p>
                  <div className="space-y-1.5">
                    {teams.map((team) => <div key={team.id} className="flex items-center justify-between text-xs"><span className="font-medium">{team.name}</span><span className={team.status === 'finalist' ? 'text-primary font-semibold' : 'text-muted-foreground'}>{team.status}</span></div>)}
                  </div>
                </div>
              )}
              {demoSlots.length > 0 && (
                <div className="rounded-lg border p-3">
                  <p className="mb-2 text-xs font-semibold tracking-wide uppercase text-muted-foreground">{t('demoDaySchedule')}</p>
                  <div className="space-y-2">{demoSlots.map((slot) => <div key={slot.id} className="rounded-md border p-2 text-xs"><div className="flex items-center justify-between"><span>#{slot.slot_order} · {t('teamSlot', { id: String(slot.team_id) })}</span><span className="flex items-center gap-2 text-muted-foreground">{slot.room} · {slot.starts_at}<button type="button" className={slot.checked_in_at ? 'text-primary' : 'underline'} onClick={async () => { const updated = await updateDemoDaySlot(slot.id, { check_in: !slot.checked_in_at }); setDemoSlots((items) => items.map((item) => item.id === updated.id ? updated : item)); }}>{slot.checked_in_at ? t('checkedIn') : t('checkIn')}</button></span></div><p className="mt-1 text-muted-foreground">{t('qrCheckInToken')}: <code className="select-all">{slot.check_in_token}</code></p><div className="mt-2 flex gap-2"><Input className="h-7 text-xs" value={checklistText[slot.id] ?? slot.prototype_checklist.join(', ')} onChange={(event) => setChecklistText((items) => ({ ...items, [slot.id]: event.target.value }))} placeholder={t('prototypeChecklist')} /><button type="button" className="rounded border px-2" onClick={async () => { const updated = await updateDemoDaySlot(slot.id, { prototype_checklist: (checklistText[slot.id] ?? '').split(',').map((item) => item.trim()).filter(Boolean) }); setDemoSlots((items) => items.map((item) => item.id === updated.id ? updated : item)); }}>{t('save')}</button></div></div>)}</div>
                </div>
              )}
              {report && (
                <div className="rounded-lg border p-3">
                  <p className="mb-2 text-xs font-semibold tracking-wide uppercase text-muted-foreground">{t('competitionReportSummary')}</p>
                  <div className="grid grid-cols-2 gap-2 text-xs sm:grid-cols-4">
                    <span>{t('teamCount', { count: String(report.total_teams) })}</span>
                    <span>{t('finalistCount', { count: String(report.finalist_teams) })}</span>
                    <span>{t('deliverableCount', { count: String(report.submitted_deliverables) })}</span>
                    <span>{t('presentationSlotCount', { count: String(report.demo_day_slots) })}</span>
                  </div>
                </div>
              )}
              <AssessmentProgressPanel competitionId={selectedId!} canRun={canRunAssessments} />
              <div className="rounded-lg border p-3"><p className="mb-2 text-xs font-semibold tracking-wide uppercase text-muted-foreground">{t('finalMinutesAndLock')}</p><textarea className="min-h-16 w-full rounded-md border bg-background p-2 text-xs" value={finalMinutes} onChange={(event) => setFinalMinutes(event.target.value)} placeholder={t('finalMinutesPlaceholder')} /><div className="mt-2 flex gap-2"><Input className="h-8 text-xs" value={finalSigner} onChange={(event) => setFinalSigner(event.target.value)} placeholder={t('authorizedSigner')} /><Button size="sm" disabled={!finalMinutes.trim() || !finalSigner.trim()} onClick={async () => { try { await finalizeCompetition(selectedId!, { minutes: finalMinutes, signed_by: finalSigner }); setFinalMinutes(''); setFinalSigner(''); setError(null); } catch (error) { setError((error as Error).message); } }}>{t('lockResults')}</Button></div></div>
              </>
            )}
          </>
        )}
      </CardContent>
    </Card>
  );
}
