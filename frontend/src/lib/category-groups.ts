// Competition review phases (T3/TEKNOFEST-style) — a different axis from subject
// fields, so the sidebar groups them separately even though they share the same
// category/KPI-template mechanism on the backend.
export const PHASE_CATEGORIES = new Set(['odr', 'ktr']);

export function isPhaseCategory(category: string): boolean {
  return PHASE_CATEGORIES.has(category);
}
