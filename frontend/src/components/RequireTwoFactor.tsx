import { useState } from 'react';
import { Copy, ShieldCheck } from 'lucide-react';
import { confirmTwoFactor, setupTwoFactor, type TwoFactorSetup } from '@/lib/api';

export function RequireTwoFactor({ onCompleted }: { onCompleted: () => Promise<void> }) {
  const [setup, setSetup] = useState<TwoFactorSetup | null>(null);
  const [code, setCode] = useState('');
  const [recoveryCodes, setRecoveryCodes] = useState<string[]>([]);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  async function beginEnrollment() {
    setLoading(true);
    setError('');
    try { setSetup(await setupTwoFactor()); }
    catch (reason) { setError((reason as Error).message); }
    finally { setLoading(false); }
  }

  async function completeEnrollment() {
    setLoading(true);
    setError('');
    try {
      const confirmation = await confirmTwoFactor(code);
      setRecoveryCodes(confirmation.recovery_codes);
    } catch (reason) { setError((reason as Error).message); }
    finally { setLoading(false); }
  }

  async function copyRecoveryCodes() {
    await navigator.clipboard?.writeText(recoveryCodes.join('\n'));
  }

  return <main className="flex min-h-screen items-center justify-center bg-muted/40 p-4"><section className="w-full max-w-lg space-y-4 rounded-xl border bg-background p-6 shadow-sm"><div className="flex items-center gap-3"><span className="rounded-lg bg-primary/10 p-2 text-primary"><ShieldCheck className="size-5" /></span><div><h1 className="font-semibold">Two-factor authentication required</h1><p className="text-sm text-muted-foreground">Set up an authenticator app before entering the dashboard.</p></div></div>{!setup && recoveryCodes.length === 0 && <button type="button" className="h-10 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground disabled:opacity-50" disabled={loading} onClick={() => void beginEnrollment()}>{loading ? 'Preparing…' : 'Set up two-factor authentication'}</button>}{setup && recoveryCodes.length === 0 && <div className="space-y-3"><div className="rounded-md border bg-secondary/30 p-3 text-xs"><p className="font-medium">Authenticator secret</p><code className="mt-1 block break-all select-all">{setup.secret}</code><p className="mt-2 text-muted-foreground">Add this secret or setup URL to your authenticator app.</p><code className="mt-1 block break-all select-all">{setup.otpauth_url}</code></div><label className="block text-sm">Verification code<input className="mt-1 h-10 w-full rounded-md border bg-background px-3" inputMode="numeric" maxLength={6} value={code} onChange={(event) => setCode(event.target.value.replace(/\D/g, ''))} /></label><button type="button" className="h-10 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground disabled:opacity-50" disabled={loading || code.length !== 6} onClick={() => void completeEnrollment()}>{loading ? 'Verifying…' : 'Enable two-factor authentication'}</button></div>}{recoveryCodes.length > 0 && <div className="space-y-3"><p className="text-sm text-muted-foreground">Save these one-time recovery codes in a password manager before continuing.</p><div className="grid grid-cols-2 gap-2 rounded-md border bg-secondary/30 p-3 font-mono text-sm">{recoveryCodes.map((recoveryCode) => <code key={recoveryCode}>{recoveryCode}</code>)}</div><button type="button" className="inline-flex h-9 items-center rounded-md border px-3 text-sm" onClick={() => void copyRecoveryCodes()}><Copy className="mr-1.5 size-3.5" />Copy recovery codes</button><button type="button" className="ml-2 h-9 rounded-md bg-primary px-3 text-sm text-primary-foreground" onClick={() => void onCompleted()}>Continue to dashboard</button></div>}{error && <p className="text-sm text-destructive">{error}</p>}</section></main>;
}
