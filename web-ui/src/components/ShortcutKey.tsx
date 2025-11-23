import { cn } from '../lib/utils';

type ShortcutKeyProps = {
  keys: string[];
  className?: string;
};

export function ShortcutKey({ keys, className }: ShortcutKeyProps) {
  return (
    <div className={cn('flex items-center gap-1 text-xs text-muted-foreground', className)}>
      {keys.map((key) => (
        <span
          key={key}
          className="rounded-lg border border-border/60 bg-white/5 px-2 py-1 font-semibold uppercase tracking-wide"
        >
          {key}
        </span>
      ))}
    </div>
  );
}
