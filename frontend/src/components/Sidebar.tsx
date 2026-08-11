import { BellRing, CalendarRange, ClipboardList, FileBarChart, Gavel, LayoutDashboard, Moon, Settings, ShieldCheck, Sun, User, X } from 'lucide-react';

import type { CategoryTemplate } from '@/lib/api';
import { useLocale } from '@/lib/locale-context';
import { useTheme } from '@/lib/theme-context';
import { useJuror } from '@/lib/juror-context';
import { CATEGORY_ICONS, DEFAULT_CATEGORY_ICON } from '@/lib/category-icons';
import { isPhaseCategory } from '@/lib/category-groups';
import { cn } from '@/lib/utils';
import { Input } from '@/components/ui/input';

export function Sidebar({
  categories,
  category,
  categoryCounts,
  onSelect,
  onOpenCompetitions,
  onOpenUsers,
  onOpenAudit,
  onOpenReports,
  onOpenSettings,
  onOpenNotifications,
  mobileOpen,
  onCloseMobile,
}: {
  categories: CategoryTemplate[];
  category: string;
  categoryCounts: Record<string, number>;
  onSelect: (category: string) => void;
  onOpenCompetitions: () => void;
  onOpenUsers: () => void;
  onOpenAudit: () => void;
  onOpenReports: () => void;
  onOpenSettings: () => void;
  onOpenNotifications: () => void;
  mobileOpen: boolean;
  onCloseMobile: () => void;
}) {
  const { locale, setLocale, t, categoryLabel } = useLocale();
  const { theme, toggleTheme } = useTheme();
  const { jurorName, setJurorName } = useJuror();
  const organization = typeof window !== 'undefined' ? localStorage.getItem('jury-organization') || 'T3 Vakfı' : 'T3 Vakfı';
  const competitionIdentity = typeof window !== 'undefined' ? localStorage.getItem('jury-competition-identity') || 'Creathon 2026' : 'Creathon 2026';
  const logoUrl = typeof window !== 'undefined' ? localStorage.getItem('jury-logo-url') : null;

  const totalProjects = Object.values(categoryCounts).reduce((sum, n) => sum + n, 0);
  const fieldCategories = categories.filter((c) => !isPhaseCategory(c.category));
  const phaseCategories = categories.filter((c) => isPhaseCategory(c.category));

  function NavItem(cat: CategoryTemplate) {
    const Icon = CATEGORY_ICONS[cat.category] ?? DEFAULT_CATEGORY_ICON;
    const active = cat.category === category;
    const count = categoryCounts[cat.category] ?? 0;
    return (
      <button
        key={cat.category}
        type="button"
        onClick={() => {
          onSelect(cat.category);
          onCloseMobile();
        }}
        className={cn(
          'flex w-full items-center gap-2.5 rounded-lg px-3 py-2 text-sm font-medium transition-colors',
          active
            ? 'bg-accent text-accent-foreground'
            : 'text-muted-foreground hover:bg-secondary/60 hover:text-foreground',
        )}
      >
        <Icon className="size-4 shrink-0" />
        <span className="flex-1 truncate text-left">{categoryLabel(cat.category)}</span>
        <span
          className={cn(
            'font-data rounded-full px-1.5 py-0.5 text-[10px] tabular-nums',
            active ? 'bg-background text-foreground' : 'bg-muted text-muted-foreground',
          )}
        >
          {count}
        </span>
      </button>
    );
  }

  const content = (
    <>
      <div className="flex items-center justify-between gap-2.5 border-b px-5 py-5">
        <div className="flex items-center gap-2.5">
          <span className="flex size-8 items-center justify-center overflow-hidden rounded-lg bg-primary text-primary-foreground">
            {logoUrl ? <img src={logoUrl} alt={organization} className="size-full object-cover" /> : <Gavel className="size-4" />}
          </span>
          <div>
            <p className="text-sm leading-tight font-semibold">{t('appName')}</p>
            <p className="text-muted-foreground text-[10px] tracking-wide uppercase">{organization}</p>
            <p className="text-muted-foreground truncate text-[10px]">{competitionIdentity}</p>
          </div>
        </div>
        <button
          type="button"
          onClick={onCloseMobile}
          aria-label={t('dismiss')}
          className="text-muted-foreground hover:text-foreground md:hidden"
        >
          <X className="size-5" />
        </button>

      </div>

      <div className="border-b px-3 py-3">
        <button
          type="button"
          onClick={() => {
            onOpenCompetitions();
            onCloseMobile();
          }}
          className="flex w-full items-center gap-2.5 rounded-lg bg-secondary/70 px-3 py-2.5 text-sm font-medium text-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
        >
          <CalendarRange className="size-4 shrink-0" />
          {t('competitionsLabel')}
        </button>
        <button type="button" onClick={() => { onOpenUsers(); onCloseMobile(); }} className="mt-2 flex w-full items-center gap-2.5 rounded-lg px-3 py-2.5 text-sm font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground">
          <ShieldCheck className="size-4 shrink-0" />{t('usersLabel')}
        </button>
        <button type="button" onClick={() => { onOpenAudit(); onCloseMobile(); }} className="mt-2 flex w-full items-center gap-2.5 rounded-lg px-3 py-2.5 text-sm font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground">
          <ClipboardList className="size-4 shrink-0" />{t('auditLabel')}
        </button>
        <button type="button" onClick={() => { onOpenReports(); onCloseMobile(); }} className="mt-2 flex w-full items-center gap-2.5 rounded-lg px-3 py-2.5 text-sm font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground">
          <FileBarChart className="size-4 shrink-0" />{t('reportsLabel')}
        </button>
        <button type="button" onClick={() => { onOpenSettings(); onCloseMobile(); }} className="mt-2 flex w-full items-center gap-2.5 rounded-lg px-3 py-2.5 text-sm font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground">
          <Settings className="size-4 shrink-0" />{t('settingsLabel')}
        </button>
        <button type="button" onClick={() => { onOpenNotifications(); onCloseMobile(); }} className="mt-2 flex w-full items-center gap-2.5 rounded-lg px-3 py-2.5 text-sm font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground">
          <BellRing className="size-4 shrink-0" />{t('notificationCenterLabel')}
        </button>
      </div>

      <div className="border-b px-5 py-3">
        <p className="font-data text-xl font-bold tabular-nums">{totalProjects}</p>
        <p className="text-muted-foreground text-[11px]">
          {t('statCount')} · {fieldCategories.length} {t('fieldsLabel').toLowerCase()}
        </p>
      </div>

      <nav className="flex-1 space-y-1 overflow-y-auto px-3 py-4">
        <button
          type="button"
          onClick={() => {
            onSelect('');
            onCloseMobile();
          }}
          className={cn(
            'mb-2 flex w-full items-center gap-2.5 rounded-lg px-3 py-2 text-sm font-medium transition-colors',
            category === ''
              ? 'bg-accent text-accent-foreground'
              : 'text-muted-foreground hover:bg-secondary/60 hover:text-foreground',
          )}
        >
          <LayoutDashboard className="size-4 shrink-0" />
          {t('overviewLabel')}
        </button>

        <p className="text-muted-foreground px-2 pb-2 text-[10px] font-semibold tracking-widest uppercase">
          {t('fieldsLabel')}
        </p>
        {fieldCategories.map(NavItem)}

        {phaseCategories.length > 0 && (
          <>
            <p className="text-muted-foreground mt-4 px-2 pb-2 text-[10px] font-semibold tracking-widest uppercase">
              {t('evaluationLabel')}
            </p>
            {phaseCategories.map(NavItem)}
          </>
        )}
      </nav>

      <div className="space-y-2.5 border-t px-3 py-3">
        <div className="relative">
          <User className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground" />
          <Input
            value={jurorName}
            onChange={(e) => setJurorName(e.target.value)}
            placeholder={t('jurorNamePlaceholder')}
            title={t('jurorNameLabel')}
            className="h-8 pl-8 text-xs"
          />
        </div>

        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={toggleTheme}
            aria-label="Toggle theme"
            className="flex h-8 flex-1 items-center justify-center gap-1.5 rounded-md border text-xs text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
          >
            {theme === 'dark' ? <Sun className="size-3.5" /> : <Moon className="size-3.5" />}
          </button>

          <div className="relative flex w-[72px] overflow-hidden rounded-md border text-xs font-medium">
            <div
              className="absolute inset-y-0 w-9 rounded-md bg-primary transition-transform duration-300 ease-out"
              style={{ transform: locale === 'en' ? 'translateX(36px)' : 'translateX(0px)' }}
            />
            {(['tr', 'en'] as const).map((code) => (
              <button
                key={code}
                type="button"
                onClick={() => setLocale(code)}
                className={cn(
                  'relative z-10 w-9 py-1.5 transition-colors',
                  locale === code ? 'text-primary-foreground' : 'text-muted-foreground hover:text-foreground',
                )}
              >
                {code.toUpperCase()}
              </button>
            ))}
          </div>
        </div>
      </div>
    </>
  );

  return (
    <>
      <aside className="hidden w-64 shrink-0 flex-col border-r bg-card md:flex">{content}</aside>

      {mobileOpen && (
        <div className="fixed inset-0 z-40 md:hidden">
          <div className="absolute inset-0 bg-black/50" onClick={onCloseMobile} />
          <aside className="animate-in slide-in-from-left fixed inset-y-0 left-0 flex w-64 flex-col border-r bg-card duration-200">
            {content}
          </aside>
        </div>
      )}
    </>
  );
}
