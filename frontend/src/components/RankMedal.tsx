import { cn } from '@/lib/utils';

export function RankMedal({ rank }: { rank: number }) {
  if (rank === 1) {
    return (
      <span className="animate-in zoom-in-75 fade-in fill-mode-both font-data flex size-7 items-center justify-center rounded-full bg-primary text-sm font-bold tabular-nums text-primary-foreground shadow-[0_0_0_3px_color-mix(in_oklab,var(--primary)_22%,transparent)] duration-300">
        {rank}
      </span>
    );
  }

  if (rank <= 3) {
    return (
      <span
        className={cn(
          'animate-in zoom-in-75 fade-in fill-mode-both font-data flex size-7 items-center justify-center rounded-full border-2 border-primary/50 text-sm font-bold tabular-nums text-primary duration-300',
        )}
      >
        {rank}
      </span>
    );
  }

  return (
    <span className="font-data flex size-7 items-center justify-center text-sm font-bold tabular-nums text-muted-foreground">
      {rank}
    </span>
  );
}
