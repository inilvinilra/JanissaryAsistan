import { CheckCircle2, CircleAlert, CircleX, FileCheck2 } from 'lucide-react';
import { useLocale } from '@/lib/locale-context';
import type { SectionFinding, TemplateCompliance } from '@/lib/api';

const STATUS_STYLES: Record<SectionFinding['status'], { icon: typeof CheckCircle2; tone: string; labelKey: string }> = {
  present: { icon: CheckCircle2, tone: 'text-primary', labelKey: 'templateSectionPresent' },
  thin: { icon: CircleAlert, tone: 'text-amber-600 dark:text-amber-500', labelKey: 'templateSectionThin' },
  missing: { icon: CircleX, tone: 'text-destructive', labelKey: 'templateSectionMissing' },
};

export function TemplateCompliancePanel({ compliance }: { compliance: TemplateCompliance | null }) {
  const { t } = useLocale();

  if (!compliance) {
    return (
      <div className="space-y-2 rounded-lg border p-3">
        <p className="text-xs font-semibold tracking-wide uppercase text-muted-foreground">{t('templateComplianceTitle')}</p>
        <p className="text-xs text-muted-foreground">{t('templateNoneDefined')}</p>
      </div>
    );
  }

  return (
    <div className="space-y-3 rounded-lg border p-3">
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div>
          <p className="flex items-center gap-2 text-xs font-semibold tracking-wide uppercase text-muted-foreground">
            <FileCheck2 className="size-3.5" />
            {t('templateComplianceTitle')}
          </p>
          <p className="mt-1 text-[11px] text-muted-foreground">
            {t('templateVersionLabel', { name: compliance.template_name, version: String(compliance.template_version) })}
          </p>
        </div>
        <span className={`text-xs font-semibold ${compliance.compliant ? 'text-primary' : 'text-destructive'}`}>
          {compliance.compliant ? t('templateCompliant') : t('templateNonCompliant')}
        </span>
      </div>

      <div className="space-y-1">
        <div className="flex items-baseline justify-between text-xs">
          <span className="text-muted-foreground">{t('templateSectionScore')}</span>
          <span className="font-data font-semibold tabular-nums">{compliance.section_score.toFixed(1)}%</span>
        </div>
        <div className="h-1.5 overflow-hidden rounded-full bg-muted">
          <div
            className={`h-full rounded-full ${compliance.compliant ? 'bg-primary' : 'bg-destructive'}`}
            style={{ width: `${Math.max(compliance.section_score, 2)}%` }}
          />
        </div>
      </div>

      <div className="grid gap-1 text-xs sm:grid-cols-2">
        <div className="flex items-center justify-between gap-2 rounded-md bg-muted/50 px-2 py-1.5">
          <span className="text-muted-foreground">{t('templateLanguageRow')}</span>
          <span className={compliance.language_matches ? '' : 'text-destructive'}>
            {compliance.language_detected}
            {!compliance.language_matches && <span className="text-muted-foreground"> · {t('templateExpected')} {compliance.language_expected}</span>}
          </span>
        </div>
        <div className="flex items-center justify-between gap-2 rounded-md bg-muted/50 px-2 py-1.5">
          <span className="text-muted-foreground">{t('templateWordCountRow')}</span>
          <span className={`font-data tabular-nums ${compliance.word_count_within_range ? '' : 'text-destructive'}`}>
            {compliance.word_count.toLocaleString()}
            <span className="text-muted-foreground"> / {compliance.min_words.toLocaleString()}–{compliance.max_words > 0 ? compliance.max_words.toLocaleString() : '∞'}</span>
          </span>
        </div>
      </div>

      <ul className="space-y-1.5">
        {compliance.sections.map((section) => {
          const style = STATUS_STYLES[section.status];
          const Icon = style.icon;
          return (
            <li key={section.key} className="flex items-start gap-2 text-xs">
              <Icon className={`mt-0.5 size-3.5 shrink-0 ${style.tone}`} />
              <span className="min-w-0 flex-1">
                <span className="font-medium">{section.title}</span>
                {!section.required && <span className="text-muted-foreground"> · {t('templateOptional')}</span>}
                <span className="mt-0.5 block break-words text-muted-foreground">{section.detail}</span>
              </span>
              <span className={`shrink-0 ${style.tone}`}>{t(style.labelKey)}</span>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
