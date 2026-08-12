import { useState } from 'react';
import { KeyRound } from 'lucide-react';
import { changePassword } from '@/lib/api';

export function ForcePasswordChange({ onCompleted }: { onCompleted: () => Promise<void> }) {
  const [currentPassword, setCurrentPassword] = useState('');
  const [newPassword, setNewPassword] = useState('');
  const [confirmation, setConfirmation] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  async function submit(event: React.FormEvent) {
    event.preventDefault();
    if (newPassword.length < 12) { setError('Your new password must contain at least 12 characters.'); return; }
    if (newPassword !== confirmation) { setError('The new password confirmation does not match.'); return; }
    setLoading(true); setError('');
    try { await changePassword({ current_password: currentPassword, new_password: newPassword }); await onCompleted(); }
    catch { setError('The current password is invalid or the password could not be updated.'); }
    finally { setLoading(false); }
  }
  return <main className="flex min-h-screen items-center justify-center bg-muted/40 p-4"><form onSubmit={submit} className="w-full max-w-sm space-y-4 rounded-xl border bg-background p-6 shadow-sm"><div className="flex items-center gap-3"><span className="rounded-lg bg-primary/10 p-2 text-primary"><KeyRound className="size-5" /></span><div><h1 className="font-semibold">Change temporary password</h1><p className="text-sm text-muted-foreground">Set a personal password to continue.</p></div></div><label className="block text-sm">Current temporary password<input className="mt-1 h-10 w-full rounded-md border bg-background px-3" type="password" value={currentPassword} onChange={(event) => setCurrentPassword(event.target.value)} required /></label><label className="block text-sm">New password<input className="mt-1 h-10 w-full rounded-md border bg-background px-3" type="password" minLength={12} value={newPassword} onChange={(event) => setNewPassword(event.target.value)} required /></label><label className="block text-sm">Confirm new password<input className="mt-1 h-10 w-full rounded-md border bg-background px-3" type="password" minLength={12} value={confirmation} onChange={(event) => setConfirmation(event.target.value)} required /></label>{error && <p className="text-sm text-destructive">{error}</p>}<button className="h-10 w-full rounded-md bg-primary text-sm font-medium text-primary-foreground disabled:opacity-50" disabled={loading}>{loading ? 'Updating password…' : 'Continue'}</button></form></main>;
}
