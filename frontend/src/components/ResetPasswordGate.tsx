import { useState } from 'react';
import { KeyRound } from 'lucide-react';
import { confirmPasswordReset } from '@/lib/api';

export function ResetPasswordGate({ token, onCompleted }: { token: string; onCompleted: () => void }) {
  const [password, setPassword] = useState('');
  const [confirmation, setConfirmation] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  async function submit(event: React.FormEvent) {
    event.preventDefault();
    if (password.length < 12) { setError('Your new password must contain at least 12 characters.'); return; }
    if (password !== confirmation) { setError('The password confirmation does not match.'); return; }
    setLoading(true); setError('');
    try { await confirmPasswordReset({ token, new_password: password }); onCompleted(); }
    catch { setError('This reset link is invalid, expired, or has already been used.'); }
    finally { setLoading(false); }
  }
  return <main className="flex min-h-screen items-center justify-center bg-muted/40 p-4"><form onSubmit={submit} className="w-full max-w-sm space-y-4 rounded-xl border bg-background p-6 shadow-sm"><div className="flex items-center gap-3"><span className="rounded-lg bg-primary/10 p-2 text-primary"><KeyRound className="size-5" /></span><div><h1 className="font-semibold">Reset password</h1><p className="text-sm text-muted-foreground">Choose a new password to regain access.</p></div></div><label className="block text-sm">New password<input className="mt-1 h-10 w-full rounded-md border bg-background px-3" type="password" minLength={12} value={password} onChange={(event) => setPassword(event.target.value)} required /></label><label className="block text-sm">Confirm new password<input className="mt-1 h-10 w-full rounded-md border bg-background px-3" type="password" minLength={12} value={confirmation} onChange={(event) => setConfirmation(event.target.value)} required /></label>{error && <p className="text-sm text-destructive">{error}</p>}<button className="h-10 w-full rounded-md bg-primary text-sm font-medium text-primary-foreground disabled:opacity-50" disabled={loading}>{loading ? 'Resetting password…' : 'Set new password'}</button></form></main>;
}
