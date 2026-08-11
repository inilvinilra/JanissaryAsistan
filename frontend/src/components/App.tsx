import { ThemeProvider } from '@/lib/theme-context';
import { LocaleProvider } from '@/lib/locale-context';
import { ToastProvider } from '@/lib/toast-context';
import { JurorProvider } from '@/lib/juror-context';
import { JuryDashboard } from '@/components/JuryDashboard';

export function App() {
  return (
    <ThemeProvider>
      <LocaleProvider>
        <ToastProvider>
          <JurorProvider>
            <div className="min-h-screen">
              <JuryDashboard />
            </div>
          </JurorProvider>
        </ToastProvider>
      </LocaleProvider>
    </ThemeProvider>
  );
}
