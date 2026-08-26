import type { Project } from '@/lib/api';

/**
 * Project and KPI names are entered by applicants and administrators, so a
 * value starting with a formula prefix would be executed by Excel and Sheets
 * when the export is opened. Prefixing with an apostrophe forces text.
 */
function neutralizeFormula(value: string): string {
  return /^[=+\-@\t\r]/.test(value) ? `'${value}` : value;
}

function escapeCsv(value: string): string {
  const safe = neutralizeFormula(value);
  if (safe.includes(',') || safe.includes('"') || safe.includes('\n')) {
    return `"${safe.replace(/"/g, '""')}"`;
  }
  return safe;
}

export function exportProjectsCsv(projects: Project[], category: string) {
  const kpiNames = [...new Set(projects.flatMap((p) => p.kpi_scores.map((k) => k.name)))];
  const header = ['rank', 'name', 'ai_score', ...kpiNames.map(escapeCsv)];
  const rows = projects.map((p, i) => {
    const scoreByKpi = new Map(p.kpi_scores.map((k) => [k.name, k.score]));
    return [
      String(p.manual_rank ?? i + 1),
      escapeCsv(p.name),
      p.ai_score.toFixed(1),
      ...kpiNames.map((name) => String(scoreByKpi.get(name) ?? '')),
    ];
  });

  const csv = [header.join(','), ...rows.map((r) => r.join(','))].join('\n');
  const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${category || 'projects'}-ranking.csv`;
  a.click();
  URL.revokeObjectURL(url);
}
