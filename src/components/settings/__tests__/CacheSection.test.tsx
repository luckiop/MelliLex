import { screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { CacheSection } from '../CacheSection';
import { api } from '../../../services/api';
import { renderWithProviders } from '../../../test/test-utils';
import type { StorageReport } from '../../../types';

vi.mock('../../../services/api', () => ({
  api: {
    storageReport: vi.fn(),
    storageClear: vi.fn(),
  },
}));

const report: StorageReport = {
  totalBytes: 120_000_000,
  lookups: { bytes: 2_000_000, count: 12, oldestUnix: 1_700_000_000, older7: 1000, older30: 500_000, older90: 0 },
  voice: { bytes: 8_000_000, count: 4, oldestUnix: 1_700_000_000, older7: 0, older30: 0, older90: 0 },
  logs: { bytes: 50_000, count: 2, oldestUnix: null, older7: 0, older30: 0, older90: 0 },
  modelBytes: 80_000_000,
  runtimeBytes: 22_000_000,
  databaseBytes: 10_000_000,
};

describe('CacheSection', () => {
  beforeEach(() => {
    vi.mocked(api.storageReport).mockResolvedValue(report);
  });

  it('shows lookup, voice, and log usage with separate age controls', async () => {
    renderWithProviders(<CacheSection />);

    expect(await screen.findByText('Lookup results')).toBeInTheDocument();
    expect(screen.getByText(/does not erase the others/)).toBeInTheDocument();
    expect(screen.getAllByText('Logs').length).toBeGreaterThan(0);
    expect(screen.getAllByText('Voice model').length).toBeGreaterThan(0);
    expect(screen.getAllByRole('combobox')).toHaveLength(3);
    expect(screen.getAllByRole('button', { name: 'Delete older' })).toHaveLength(3);
  });
});
