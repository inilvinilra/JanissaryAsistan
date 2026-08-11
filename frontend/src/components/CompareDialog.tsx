import type { Project } from '@/lib/api';
import { useLocale } from '@/lib/locale-context';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog';

export function CompareDialog({
  projects,
  open,
  onOpenChange,
}: {
  projects: Project[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const { t } = useLocale();
  const kpiNames = [...new Set(projects.flatMap((p) => p.kpi_scores.map((k) => k.name)))];

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[85vh] overflow-y-auto sm:max-w-2xl">
        <DialogHeader>
          <DialogTitle>{t('compareTitle')}</DialogTitle>
        </DialogHeader>

        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b">
                <th className="p-2 text-left font-medium text-muted-foreground">{t('colProject')}</th>
                {projects.map((p) => (
                  <th key={p.id} className="max-w-32 truncate p-2 text-left font-medium">
                    {p.name}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              <tr className="border-b">
                <td className="p-2 font-medium text-muted-foreground">{t('colScore')}</td>
                {projects.map((p) => (
                  <td key={p.id} className="font-data p-2 font-bold tabular-nums">
                    {p.ai_score.toFixed(1)}
                  </td>
                ))}
              </tr>
              {kpiNames.map((kpiName) => (
                <tr key={kpiName} className="border-b">
                  <td className="p-2 text-muted-foreground">{kpiName}</td>
                  {projects.map((p) => {
                    const score = p.kpi_scores.find((k) => k.name === kpiName)?.score;
                    return (
                      <td key={p.id} className="font-data p-2 tabular-nums">
                        {score !== undefined ? score.toFixed(0) : '—'}
                      </td>
                    );
                  })}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </DialogContent>
    </Dialog>
  );
}
