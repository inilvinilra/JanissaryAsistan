import { PieChart, Pie, Cell, ResponsiveContainer } from 'recharts';
import { PieChart as PieChartIcon } from 'lucide-react';

import type { CategoryTemplate } from '@/lib/api';
import { useLocale } from '@/lib/locale-context';
import { Card } from '@/components/ui/card';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';

const SLICE_COLORS = ['var(--chart-1)', 'var(--chart-2)', 'var(--chart-3)', 'var(--chart-4)', 'var(--chart-5)'];

export function KpiDonutPanel({ template }: { template: CategoryTemplate | undefined }) {
  const { t } = useLocale();
  if (!template) return null;

  const total = template.kpis.reduce((sum, k) => sum + k.weight, 0);

  return (
    <Card className="p-5">
      <div className="mb-4 flex items-center gap-2">
        <PieChartIcon className="size-4 text-primary" />
        <h3 className="text-sm font-semibold">{t('kpiWeightsTitle')}</h3>
      </div>

      <div className="relative mx-auto h-40 w-40">
        <ResponsiveContainer>
          <PieChart>
            <Pie
              data={template.kpis}
              dataKey="weight"
              nameKey="name"
              innerRadius={48}
              outerRadius={72}
              paddingAngle={2}
              strokeWidth={0}
              isAnimationActive
              animationDuration={700}
            >
              {template.kpis.map((_, i) => (
                <Cell key={i} fill={SLICE_COLORS[i % SLICE_COLORS.length]} />
              ))}
            </Pie>
          </PieChart>
        </ResponsiveContainer>
        <div className="pointer-events-none absolute inset-0 flex flex-col items-center justify-center">
          <p className="font-data text-xl font-bold tabular-nums">{total.toFixed(0)}%</p>
          <p className="text-muted-foreground text-[10px] uppercase">{t('totalLabel')}</p>
        </div>
      </div>

      <div className="mt-5 space-y-2">
        {template.kpis.map((kpi, i) => (
          <Tooltip key={kpi.name}>
            <TooltipTrigger className="flex w-full items-center gap-2 text-xs">
              <span
                className="size-2 shrink-0 rounded-full"
                style={{ backgroundColor: SLICE_COLORS[i % SLICE_COLORS.length] }}
              />
              <span className="flex-1 truncate text-left">{kpi.name}</span>
              <span className="font-data font-medium tabular-nums">{kpi.weight.toFixed(0)}%</span>
            </TooltipTrigger>
            <TooltipContent>{kpi.description}</TooltipContent>
          </Tooltip>
        ))}
      </div>
    </Card>
  );
}
