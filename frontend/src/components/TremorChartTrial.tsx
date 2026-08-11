import { BarChart } from '@tremor/react';

import type { Project } from '@/lib/api';

export function TremorChartTrial({ projects }: { projects: Project[] }) {
  return (
    <BarChart
      data={projects}
      index="name"
      categories={['ai_score']}
      colors={['amber']}
      valueFormatter={(v: number) => v.toFixed(1)}
      yAxisWidth={32}
      showLegend={false}
      className="h-64"
    />
  );
}
