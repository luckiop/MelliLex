import { screen, fireEvent } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { Section1HeaderView } from '../sections/Section1HeaderView';
import { renderWithProviders, createMockRootStore } from '../../test/test-utils';
import type { WordSection1Header } from '../../types';
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

const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return { ...actual, useNavigate: () => mockNavigate };
});

const sampleHeader: WordSection1Header = {
  word: 'eloquent',
  pronunciation: '/ˈel.ə.kwənt/',
  syllables: 'el·o·quent',
  origin: 'Latin eloquentem',
  formality: { level: 'Formal', percentage: 75 },
  domains: ['Literature', 'Public Speaking'],
  tldr: 'Fluent or persuasive in speaking or writing.',
};

describe('Section1HeaderView', () => {
  beforeEach(() => {
    mockNavigate.mockClear();
    vi.mocked(pronunciation.play).mockClear();
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('renders core header metadata', () => {
    renderWithProviders(<Section1HeaderView data={sampleHeader} />);
    expect(screen.getByText('eloquent')).toBeInTheDocument();
    expect(screen.getByText('/ˈel.ə.kwənt/')).toBeInTheDocument();
    expect(screen.getByText('el·o·quent')).toBeInTheDocument();
    expect(screen.getByText('Latin')).toBeInTheDocument();
    const originEl = screen.getByTitle('Latin eloquentem');
    expect(originEl).toBeInTheDocument();
    expect(screen.getByText('Formal · 75%')).toBeInTheDocument();
    expect(screen.getByText('Literature')).toBeInTheDocument();
    expect(screen.getByText('Public Speaking')).toBeInTheDocument();
    expect(screen.getByText(/Fluent or persuasive/)).toBeInTheDocument();
  });

  it('hides domains section when no domains', () => {
    renderWithProviders(
      <Section1HeaderView data={{ ...sampleHeader, domains: [] }} />
    );
    expect(screen.queryByText('Literature')).toBeNull();
  });

  it('navigates to explore page when explore button is clicked', () => {
    renderWithProviders(<Section1HeaderView data={sampleHeader} />);
    fireEvent.click(screen.getByRole('button', { name: /explore/i }));
    expect(mockNavigate).toHaveBeenCalledWith('/explore');
  });

  it('plays the word when pronounce is clicked', () => {
    renderWithProviders(<Section1HeaderView data={sampleHeader} />);
    fireEvent.click(screen.getByLabelText(/pronounce/i));
    expect(pronunciation.play).toHaveBeenCalledWith('eloquent');
  });

  it('adds the word to the vocab notebook from the header', () => {
    const cycleStars = vi.fn().mockResolvedValue(undefined);
    renderWithProviders(<Section1HeaderView data={sampleHeader} />, {
      store: createMockRootStore({
        vocabStore: { cycleStars },
      }),
    });

    fireEvent.click(screen.getByRole('button', { name: /add/i }));
    expect(cycleStars).toHaveBeenCalledWith('eloquent', [
      { partOfSpeech: '', definition: 'Fluent or persuasive in speaking or writing.' },
    ]);
  });

  it('shows two stars when the word is already two-star vocab', () => {
    renderWithProviders(<Section1HeaderView data={sampleHeader} />, {
      store: createMockRootStore({
        vocabStore: {
          currentEntry: {
            word: 'eloquent',
            stars: 2,
            category: 'daily',
            addedAt: 0,
            updatedAt: 0,
          },
          activeCategory: 'daily',
        },
      }),
    });

    expect(screen.getByRole('button', { name: /2 stars/i })).toBeInTheDocument();
  });
});
