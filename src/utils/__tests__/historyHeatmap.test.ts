import { describe, expect, it } from 'vitest';
import { buildHeatmapDays, countsByDay, dayKey, heatmapLevel } from '../historyHeatmap';

describe('historyHeatmap', () => {
  it('groups unix timestamps by local day', () => {
    const noon = Math.floor(new Date(2026, 8, 3, 12).getTime() / 1000);
    const evening = Math.floor(new Date(2026, 8, 3, 20).getTime() / 1000);
    const counts = countsByDay([{ timestamp: noon }, { timestamp: evening }]);
    expect(counts.get(dayKey(new Date(2026, 8, 3)))).toBe(2);
  });

  it('builds Sunday-aligned days ending today', () => {
    const friday = new Date(2026, 8, 4);
    const days = buildHeatmapDays(friday, 2);
    expect(days[0].getDay()).toBe(0);
    expect(dayKey(days[days.length - 1])).toBe(dayKey(friday));
  });

  it('maps counts onto five intensity levels', () => {
    expect(heatmapLevel(0, 8)).toBe(0);
    expect(heatmapLevel(1, 8)).toBe(1);
    expect(heatmapLevel(8, 8)).toBe(4);
  });
});
