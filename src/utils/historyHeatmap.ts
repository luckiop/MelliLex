const DAY_MS = 24 * 60 * 60 * 1000;

export function dayKey(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

export function startOfLocalDay(date: Date): Date {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate());
}

export function countsByDay(items: Array<{ timestamp: number }>): Map<string, number> {
  const counts = new Map<string, number>();
  for (const item of items) {
    const key = dayKey(new Date(item.timestamp * 1000));
    counts.set(key, (counts.get(key) ?? 0) + 1);
  }
  return counts;
}

/** GitHub-style weeks: Sunday-first columns covering `weekCount` weeks ending today. */
export function buildHeatmapDays(today = new Date(), weekCount = 53): Date[] {
  const end = startOfLocalDay(today);
  const weekday = end.getDay();
  const start = new Date(end.getTime() - ((weekCount - 1) * 7 + weekday) * DAY_MS);
  const days: Date[] = [];
  for (let cursor = start.getTime(); cursor <= end.getTime(); cursor += DAY_MS) {
    days.push(new Date(cursor));
  }
  return days;
}

export function heatmapLevel(count: number, max: number): 0 | 1 | 2 | 3 | 4 {
  if (count <= 0 || max <= 0) {
    return 0;
  }
  const ratio = count / max;
  if (ratio <= 0.25) {
    return 1;
  }
  if (ratio <= 0.5) {
    return 2;
  }
  if (ratio <= 0.75) {
    return 3;
  }
  return 4;
}
