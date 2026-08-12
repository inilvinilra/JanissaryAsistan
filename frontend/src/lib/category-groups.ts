export const PHASE_CATEGORIES = new Set(['odr', 'ktr']);

export function isPhaseCategory(category: string): boolean {
  return PHASE_CATEGORIES.has(category);
}
