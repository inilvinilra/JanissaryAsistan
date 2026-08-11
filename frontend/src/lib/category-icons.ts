import {
  Atom,
  Bot,
  Brain,
  ClipboardCheck,
  Code2,
  Cpu,
  Database,
  FileSearch,
  FlaskConical,
  GraduationCap,
  HeartPulse,
  LayoutGrid,
  Leaf,
  ShieldCheck,
  Sigma,
  type LucideIcon,
} from 'lucide-react';

export const CATEGORY_ICONS: Record<string, LucideIcon> = {
  software: Code2,
  technology: Cpu,
  science: FlaskConical,
  mathematics: Sigma,
  physics: Atom,
  ai: Brain,
  'data-science': Database,
  'health-tech': HeartPulse,
  sustainability: Leaf,
  edtech: GraduationCap,
  robotics: Bot,
  cybersecurity: ShieldCheck,
  odr: ClipboardCheck,
  ktr: FileSearch,
};

export const DEFAULT_CATEGORY_ICON = LayoutGrid;
