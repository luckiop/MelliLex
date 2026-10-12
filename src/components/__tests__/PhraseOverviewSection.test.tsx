import { screen, fireEvent } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { PhraseOverviewSection } from '../phrase/PhraseOverviewSection';
import { renderWithProviders } from '../../test/test-utils';
import type { PhraseSection1Overview } from '../../types';
import { pronunciation } from '../../services/pronunciation';

vi.mock('../../services/pronunciation', () => ({
  pronunciation: {
    play: vi.fn(),
    stop: vi.fn(),
  },
}));

// Mock sonner toast
vi.mock('sonner', () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

// Mock ExportPhraseDialog to isolate tests
vi.mock('../phrase/ExportPhraseDialog', () => ({
  ExportPhraseDialog: () => <button>Export</button>,
}));

const sampleOverview: PhraseSection1Overview = {
  phrase: 'break the ice',
  phraseType: 'idiom',
  tldr: 'To initiate conversation in an awkward social situation.',
  literalMeaning: 'To physically break frozen water',
  actualMeaning: 'To relieve tension or get conversation started in a social setting.',
  formality: { level: 'Informal', percentage: 30 },
  region: 'universal',
};

describe('PhraseOverviewSection', () => {
  beforeEach(() => {
    vi.mocked(pronunciation.play).mockClear();
  });

  it('renders key phrase content and badges', () => {
    renderWithProviders(<PhraseOverviewSection data={sampleOverview} />);
    expect(screen.getByText('break the ice')).toBeInTheDocument();
    expect(screen.getByText(/initiate conversation/)).toBeInTheDocument();
    expect(screen.getByText(/physically break frozen/)).toBeInTheDocument();
    expect(screen.getByText(/relieve tension/)).toBeInTheDocument();
    expect(screen.getByText('Idiom')).toBeInTheDocument();
    expect(screen.getByText(/pronounce/i)).toBeInTheDocument();
    expect(screen.getByText(/share/i)).toBeInTheDocument();
  });

  it('plays the phrase when pronounce is clicked', () => {
    renderWithProviders(<PhraseOverviewSection data={sampleOverview} />);
    fireEvent.click(screen.getByRole('button', { name: /pronounce/i }));
    expect(pronunciation.play).toHaveBeenCalledWith('break the ice');
  });

  it('shows only meaning section when no literalMeaning', () => {
    const noLiteral = { ...sampleOverview, literalMeaning: undefined };
    renderWithProviders(<PhraseOverviewSection data={noLiteral} />);
    expect(screen.getByText(/relieve tension/)).toBeInTheDocument();
    expect(screen.queryByText(/Literal/i)).toBeNull();
  });
});
