import { useEffect, useState } from 'react';
import { Copy, Save, Settings2 } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { useLocale } from '@/lib/locale-context';
import { useToast } from '@/lib/toast-context';
import { confirmTwoFactor, getOrganizations, setupTwoFactor, type OrganizationSummary, type TwoFactorSetup } from '@/lib/api';

export function SettingsPanel() {
  const { t } = useLocale();
  const { showToast } = useToast();
  const [organization, setOrganization] = useState(() => localStorage.getItem('jury-organization') ?? '');
  const [competitionIdentity, setCompetitionIdentity] = useState(() => localStorage.getItem('jury-competition-identity') ?? '');
  const [logoUrl, setLogoUrl] = useState(() => localStorage.getItem('jury-logo-url') ?? '');
  const [organizations, setOrganizations] = useState<OrganizationSummary[]>([]);
  const [twoFactorSetup, setTwoFactorSetup] = useState<TwoFactorSetup | null>(null);
  const [twoFactorCode, setTwoFactorCode] = useState('');
  const [recoveryCodes, setRecoveryCodes] = useState<string[]>([]);

  useEffect(() => { getOrganizations().then(setOrganizations).catch(() => {}); }, []);

  function save() {
    localStorage.setItem('jury-organization', organization);
    localStorage.setItem('jury-competition-identity', competitionIdentity);
    localStorage.setItem('jury-logo-url', logoUrl);
    showToast(t('settingsSaved'), 'success');
  }

  async function beginTwoFactor() {
    try { setRecoveryCodes([]); setTwoFactorSetup(await setupTwoFactor()); }
    catch (error) { showToast((error as Error).message, 'error'); }
  }

  async function enableTwoFactor() {
    try {
      const confirmation = await confirmTwoFactor(twoFactorCode);
      setRecoveryCodes(confirmation.recovery_codes);
      setTwoFactorSetup(null);
      setTwoFactorCode('');
      showToast('Two-factor authentication enabled. Save your recovery codes now.', 'success');
    } catch (error) { showToast((error as Error).message, 'error'); }
  }

  async function copyRecoveryCodes() {
    await navigator.clipboard?.writeText(recoveryCodes.join('\n'));
    showToast('Recovery codes copied to clipboard.', 'success');
  }

  return <div className="space-y-6">
    <div><h1 className="text-xl font-semibold">{t('settingsTitle')}</h1><p className="text-muted-foreground mt-1 text-sm">{t('settingsDescription')}</p></div>
    <Card><CardHeader><CardTitle className="flex items-center gap-2"><Settings2 className="size-4" />{t('identitySettings')}</CardTitle></CardHeader><CardContent className="space-y-4"><div><label className="mb-1.5 block text-xs font-medium">{t('organizationName')}</label><Input value={organization} onChange={(event) => setOrganization(event.target.value)} placeholder="T3 Foundation" /></div><div><label className="mb-1.5 block text-xs font-medium">{t('competitionIdentity')}</label><Input value={competitionIdentity} onChange={(event) => setCompetitionIdentity(event.target.value)} placeholder="TEKNOFEST 2026" /></div><div><label className="mb-1.5 block text-xs font-medium">{t('logoUrl')}</label><Input type="url" value={logoUrl} onChange={(event) => setLogoUrl(event.target.value)} placeholder="https://example.org/logo.png" /></div><div className="flex justify-end"><Button size="sm" onClick={save}><Save className="mr-1.5 size-3.5" />{t('save')}</Button></div></CardContent></Card>
    <Card><CardHeader><CardTitle>Two-factor authentication</CardTitle></CardHeader><CardContent className="space-y-3"><p className="text-sm text-muted-foreground">Protect this account with an authenticator app.</p>{!twoFactorSetup ? <Button size="sm" variant="outline" onClick={() => void beginTwoFactor()}>Set up two-factor authentication</Button> : <><div className="rounded-md border bg-secondary/30 p-3 text-xs"><p className="font-medium">Authenticator secret</p><code className="mt-1 block break-all select-all">{twoFactorSetup.secret}</code><p className="mt-2 text-muted-foreground">You can also use this setup URL in a compatible authenticator: <code className="break-all">{twoFactorSetup.otpauth_url}</code></p></div><div className="flex flex-wrap gap-2"><Input className="max-w-56" inputMode="numeric" maxLength={6} value={twoFactorCode} onChange={(event) => setTwoFactorCode(event.target.value.replace(/\D/g, ''))} placeholder="Six-digit verification code" /><Button size="sm" disabled={twoFactorCode.length !== 6} onClick={() => void enableTwoFactor()}>Enable</Button></div></>}</CardContent></Card>
    {recoveryCodes.length > 0 && <Card className="border-amber-500/50"><CardHeader><CardTitle>Save recovery codes</CardTitle></CardHeader><CardContent className="space-y-3"><p className="text-sm text-muted-foreground">These codes are shown only once. Store them in a password manager. Each code can be used for one sign-in if your authenticator is unavailable.</p><div className="grid grid-cols-2 gap-2 rounded-md border bg-secondary/30 p-3 font-mono text-sm">{recoveryCodes.map((code) => <code key={code}>{code}</code>)}</div><Button size="sm" variant="outline" onClick={() => void copyRecoveryCodes()}><Copy className="mr-1.5 size-3.5" />Copy recovery codes</Button></CardContent></Card>}
    <Card><CardHeader><CardTitle>{t('organizationManagerTitle')}</CardTitle></CardHeader><CardContent className="space-y-2">{organizations.map((item) => <div key={item.organization} className="flex items-center justify-between rounded-md border px-3 py-2 text-sm"><span className="font-medium">{item.organization}</span><span className="text-muted-foreground text-xs">{item.competition_count} {t('organizationCompetitions')} · {item.archived_count} {t('organizationArchived')}</span></div>)}{organizations.length === 0 && <p className="text-muted-foreground text-sm">{t('organizationEmpty')}</p>}</CardContent></Card>
  </div>;
}
