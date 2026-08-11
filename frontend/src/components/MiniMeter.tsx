export function MiniMeter({ value }: { value: number }) {
  return (
    <div className="flex min-w-20 items-center gap-2">
      <span className="font-data w-8 shrink-0 text-xs tabular-nums text-muted-foreground">
        {value.toFixed(0)}
      </span>
      <div className="h-1.5 w-full min-w-10 overflow-hidden rounded-full bg-muted">
        <div className="h-full rounded-full bg-primary/70" style={{ width: `${Math.min(value, 100)}%` }} />
      </div>
    </div>
  );
}
