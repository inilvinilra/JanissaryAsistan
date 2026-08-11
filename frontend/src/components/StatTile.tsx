import { useCountUp } from '@/lib/use-count-up';

export function StatTile({
  label,
  value,
  decimals = 0,
  delay = 0,
}: {
  label: string;
  value: number;
  decimals?: number;
  delay?: number;
}) {
  const animated = useCountUp(value);
  return (
    <div
      className="surface-elevated animate-in fade-in slide-in-from-bottom-2 rounded-xl border bg-card px-5 py-4 fill-mode-both duration-500"
      style={{ animationDelay: `${delay}ms` }}
    >
      <p className="text-muted-foreground text-[11px] font-medium tracking-wide uppercase">{label}</p>
      <p className="font-data mt-1 text-3xl font-bold tabular-nums">{animated.toFixed(decimals)}</p>
    </div>
  );
}
