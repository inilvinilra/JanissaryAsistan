import { useEffect, useState } from 'react';
import { ThemeProvider } from '@/lib/theme-context';
import { LocaleProvider } from '@/lib/locale-context';
import { ToastProvider } from '@/lib/toast-context';
import { JuryDashboard } from '@/components/JuryDashboard';
import { ContestantPortal } from '@/components/ContestantPortal';
import { AuthGate } from '@/components/AuthGate';
import { ForcePasswordChange } from '@/components/ForcePasswordChange';
import { RequireTwoFactor } from '@/components/RequireTwoFactor';
import { ResetPasswordGate } from '@/components/ResetPasswordGate';
import { getCurrentUser, logout, type AuthSession } from '@/lib/api';

export function App() {
  const [resetToken, setResetToken] = useState(() => typeof window === 'undefined' ? null : new URLSearchParams(window.location.search).get('reset_token'));
  const [hydrated, setHydrated] = useState(false);
  const [session, setSession] = useState<AuthSession | null>(() => { if (typeof localStorage === 'undefined') return null; const token = localStorage.getItem('jury-auth-token'); const user = localStorage.getItem('jury-auth-user'); return token && user ? { token, expires_at: '', user: JSON.parse(user) } : null; });
  async function signOut() {
    const token = localStorage.getItem('jury-auth-token');
    if (token) await logout(token);
    localStorage.removeItem('jury-auth-token');
    localStorage.removeItem('jury-auth-user');
    setSession(null);
  }
  useEffect(() => {
    setHydrated(true);
  }, []);
  useEffect(() => {
    if (!session) return;
    getCurrentUser()
      .then((user) => {
        localStorage.setItem('jury-auth-user', JSON.stringify(user));
        setSession((current) => current ? { ...current, user } : null);
      })
      .catch(() => { void signOut(); });
  }, [session?.token]);
  async function refreshSessionUser() {
    const user = await getCurrentUser();
    localStorage.setItem('jury-auth-user', JSON.stringify(user));
    setSession((current) => current ? { ...current, user } : null);
  }
  function resetCompleted() {
    window.history.replaceState({}, '', window.location.pathname);
    setResetToken(null);
  }
  return (
    <ThemeProvider>
      <LocaleProvider>
        <ToastProvider>
          <div className="min-h-screen" data-hydrated={hydrated ? 'true' : 'false'}>{resetToken ? <ResetPasswordGate token={resetToken} onCompleted={resetCompleted} /> : session ? (session.user.must_change_password ? <ForcePasswordChange onCompleted={refreshSessionUser} /> : session.user.two_factor_required && !session.user.two_factor_enabled ? <RequireTwoFactor onCompleted={refreshSessionUser} /> : session.user.role === 'contestant' ? <ContestantPortal onSignOut={signOut} /> : <JuryDashboard onSignOut={signOut} />) : <AuthGate onAuthenticated={setSession} />}</div>
        </ToastProvider>
      </LocaleProvider>
    </ThemeProvider>
  );
}
