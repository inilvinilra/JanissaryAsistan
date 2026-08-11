import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { GripVertical } from 'lucide-react';

import type { Project } from '@/lib/api';
import { useLocale } from '@/lib/locale-context';
import { cn } from '@/lib/utils';
import { TableRow, TableCell } from '@/components/ui/table';
import { RankMedal } from '@/components/RankMedal';
import { MiniMeter } from '@/components/MiniMeter';
import { StatusBadge } from '@/components/StatusBadge';

export function SortableProjectRow({
  project,
  rank,
  index,
  kpiOrder,
  onOpenDetail,
  selected,
  onToggleSelect,
  dragDisabled = false,
}: {
  project: Project;
  rank: number;
  index: number;
  kpiOrder: string[];
  onOpenDetail: (project: Project) => void;
  selected: boolean;
  onToggleSelect: (id: number) => void;
  dragDisabled?: boolean;
}) {
  const { t } = useLocale();
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: project.id,
  });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.6 : 1,
    animationDelay: `${Math.min(index, 8) * 45}ms`,
  };

  const isMedal = rank <= 3;
  const scoreByKpi = new Map(project.kpi_scores.map((k) => [k.name, k.score]));

  return (
    <TableRow
      ref={setNodeRef}
      style={style}
      className={cn(
        'animate-in fade-in slide-in-from-left-1 fill-mode-both duration-300',
        isMedal && 'bg-accent/40 hover:bg-accent/60',
      )}
    >
      <TableCell className="w-8">
        <input
          type="checkbox"
          checked={selected}
          onChange={() => onToggleSelect(project.id)}
          className="accent-primary size-3.5"
        />
      </TableCell>
      <TableCell className="w-8">
        {!dragDisabled && (
          <button
            type="button"
            {...attributes}
            {...listeners}
            className="cursor-grab touch-none text-muted-foreground transition-colors hover:text-foreground active:cursor-grabbing"
            aria-label={t('reorderAria', { name: project.name })}
          >
            <GripVertical className="size-4" />
          </button>
        )}
      </TableCell>
      <TableCell>
        <RankMedal rank={rank} />
      </TableCell>
      <TableCell className="max-w-48">
        <button
          type="button"
          onClick={() => onOpenDetail(project)}
          className="block truncate text-left font-medium hover:text-primary hover:underline"
        >
          {project.name}
        </button>
        <StatusBadge status={project.status} />
      </TableCell>
      {kpiOrder.map((kpiName) => (
        <TableCell key={kpiName}>
          <MiniMeter value={scoreByKpi.get(kpiName) ?? 0} />
        </TableCell>
      ))}
      <TableCell className="font-data text-right text-base font-bold tabular-nums">
        {project.ai_score.toFixed(1)}
      </TableCell>
    </TableRow>
  );
}
