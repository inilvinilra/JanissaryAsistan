import { useEffect, useState } from 'react';
import { ShieldCheck, UserPlus } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { getUsers, createUser, updateUser, getCompetitions, getRoles, type User, type UserRole, type Competition, type RoleDefinition } from '@/lib/api';
import { useLocale } from '@/lib/locale-context';

const roles: UserRole[] = ['system_admin', 'competition_manager', 'chief_judge', 'jury_member', 'observer', 'read_only'];

export function UserManager() {
  const { t } = useLocale();
  const [users, setUsers] = useState<User[]>([]);
  const [competitions, setCompetitions] = useState<Competition[]>([]);
  const [roleDefinitions, setRoleDefinitions] = useState<RoleDefinition[]>([]);
  const [name, setName] = useState(''); const [email, setEmail] = useState(''); const [role, setRole] = useState<UserRole>('jury_member');
  const [competitionId, setCompetitionId] = useState<number | null>(null); const [category, setCategory] = useState('');
  const [error, setError] = useState('');
  const load = () => getUsers().then(setUsers).catch((e) => setError(e.message));
  useEffect(() => { void load(); getCompetitions().then(setCompetitions).catch(() => {}); getRoles().then(setRoleDefinitions).catch(() => {}); }, []);
  async function add() {
    if (!name.trim() || !email.trim()) return;
    try { const user = await createUser({ full_name: name.trim(), email: email.trim(), role, competition_id: competitionId, category: category.trim() || null }); setUsers((prev) => [...prev, user].sort((a, b) => a.full_name.localeCompare(b.full_name))); setName(''); setEmail(''); setCompetitionId(null); setCategory(''); }
    catch (e) { setError((e as Error).message); }
  }
  async function changeRole(user: User, next: UserRole) { const updated = await updateUser(user.id, { role: next }); setUsers((prev) => prev.map((item) => item.id === updated.id ? updated : item)); }
  async function toggle(user: User) { const updated = await updateUser(user.id, { active: !user.active }); setUsers((prev) => prev.map((item) => item.id === updated.id ? updated : item)); }
  return <div className="space-y-6">
    <div><h1 className="text-xl font-semibold">{t('usersTitle')}</h1><p className="text-muted-foreground mt-1 text-sm">{t('usersDescription')}</p></div>
    <Card><CardHeader><CardTitle className="flex items-center gap-2"><UserPlus className="size-4" />{t('addUser')}</CardTitle></CardHeader><CardContent><div className="grid gap-3 md:grid-cols-[1fr_1fr_190px_190px_1fr_auto]"><Input value={name} onChange={(e) => setName(e.target.value)} placeholder={t('userNamePlaceholder')} /><Input type="email" value={email} onChange={(e) => setEmail(e.target.value)} placeholder={t('userEmailPlaceholder')} /><select value={role} onChange={(e) => setRole(e.target.value as UserRole)} className="h-9 rounded-md border bg-background px-3 text-sm">{roles.map((item) => <option key={item} value={item}>{t(`role_${item}`)}</option>)}</select><select value={competitionId ?? ''} onChange={(e) => setCompetitionId(e.target.value ? Number(e.target.value) : null)} className="h-9 rounded-md border bg-background px-3 text-sm"><option value="">{t('allCompetitions')}</option>{competitions.map((item) => <option key={item.id} value={item.id}>{item.name}</option>)}</select><Input value={category} onChange={(e) => setCategory(e.target.value)} placeholder={t('categoryScopePlaceholder')} /><button type="button" onClick={add} className="rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground">{t('add')}</button></div>{error && <p className="mt-3 text-sm text-destructive">{error}</p>}</CardContent></Card>
    <Card><CardHeader><CardTitle>{t('usersListTitle')}</CardTitle></CardHeader><CardContent className="p-0"><div className="divide-y">{users.map((user) => <div key={user.id} className="flex flex-wrap items-center gap-3 px-5 py-4"><div className="flex min-w-[220px] flex-1 items-center gap-3"><span className="flex size-9 items-center justify-center rounded-full bg-secondary"><ShieldCheck className="size-4" /></span><div><p className="text-sm font-medium">{user.full_name}</p><p className="text-muted-foreground text-xs">{user.email}</p><p className="text-muted-foreground text-[11px]">{user.competition_id ? `Competition #${user.competition_id}` : t('allCompetitions')}{user.category ? ` · ${user.category}` : ''}</p></div></div><select value={user.role} onChange={(e) => changeRole(user, e.target.value as UserRole)} className="h-8 rounded-md border bg-background px-2 text-xs">{roles.map((item) => <option key={item} value={item}>{t(`role_${item}`)}</option>)}</select><button type="button" onClick={() => toggle(user)} className={`rounded-full px-3 py-1 text-xs ${user.active ? 'bg-emerald-500/15 text-emerald-700' : 'bg-muted text-muted-foreground'}`}>{user.active ? t('active') : t('inactive')}</button></div>)}{users.length === 0 && <p className="text-muted-foreground px-5 py-6 text-sm">{t('noUsers')}</p>}</div></CardContent></Card>
    <Card><CardHeader><CardTitle>{t('rolePermissionsTitle')}</CardTitle></CardHeader><CardContent className="grid gap-3 md:grid-cols-2">{roleDefinitions.map((definition) => <div key={definition.role} className="rounded-md border p-3"><p className="text-sm font-medium">{t(`role_${definition.role}`)}</p><div className="mt-2 flex flex-wrap gap-1.5">{definition.permissions.map((permission) => <span key={permission} className="rounded-full bg-secondary px-2 py-0.5 text-[10px] text-muted-foreground">{permission}</span>)}</div></div>)}</CardContent></Card>
  </div>;
}
