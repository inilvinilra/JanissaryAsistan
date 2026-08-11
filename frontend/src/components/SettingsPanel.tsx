import { useState } from 'react';
import { Save, Settings2 } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { useLocale } from '@/lib/locale-context';
import { useToast } from '@/lib/toast-context';

export function SettingsPanel() {
  const { t } = useLocale(); const { showToast } = useToast();
  const [organization, setOrganization] = useState(() => localStorage.getItem('jury-organization') ?? '');
  const [competitionIdentity, setCompetitionIdentity] = useState(() => localStorage.getItem('jury-competition-identity') ?? '');
  const [logoUrl, setLogoUrl] = useState(() => localStorage.getItem('jury-logo-url') ?? '');
  function save() { localStorage.setItem('jury-organization', organization); localStorage.setItem('jury-competition-identity', competitionIdentity); localStorage.setItem('jury-logo-url', logoUrl); showToast(t('settingsSaved'), 'success'); }
  return <div className="space-y-6"><div><h1 className="text-xl font-semibold">{t('settingsTitle')}</h1><p className="text-muted-foreground mt-1 text-sm">{t('settingsDescription')}</p></div><Card><CardHeader><CardTitle className="flex items-center gap-2"><Settings2 className="size-4" />{t('identitySettings')}</CardTitle></CardHeader><CardContent className="space-y-4"><div><label className="mb-1.5 block text-xs font-medium">{t('organizationName')}</label><Input value={organization} onChange={(e) => setOrganization(e.target.value)} placeholder="T3 Vakfı" /></div><div><label className="mb-1.5 block text-xs font-medium">{t('competitionIdentity')}</label><Input value={competitionIdentity} onChange={(e) => setCompetitionIdentity(e.target.value)} placeholder="TEKNOFEST 2026" /></div><div><label className="mb-1.5 block text-xs font-medium">{t('logoUrl')}</label><Input type="url" value={logoUrl} onChange={(e) => setLogoUrl(e.target.value)} placeholder="https://.../logo.png" /></div><div className="flex justify-end"><Button size="sm" onClick={save}><Save className="mr-1.5 size-3.5" />{t('save')}</Button></div></CardContent></Card></div>;
}
