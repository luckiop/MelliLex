import { screen, fireEvent } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Section1HeaderView } from '../sections/Section1HeaderView';
import { renderWithProviders, createMockRootStore } from '../../test/test-utils';
import type { WordSection1Header } from '../../types';

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

  it('calls speechSynthesis when pronounce is clicked', () => {
    const mockSpeak = vi.fn();
    const mockUtterance = vi.fn();
    Object.defineProperty(window, 'speechSynthesis', {
      value: { speak: mockSpeak },
      writable: true,
      configurable: true,
    });
    vi.stubGlobal('SpeechSynthesisUtterance', mockUtterance);
    renderWithProviders(<Section1HeaderView data={sampleHeader} />);
    fireEvent.click(screen.getByLabelText(/pronounce/i));
    expect(mockSpeak).toHaveBeenCalled();
    expect(mockUtterance).toHaveBeenCalledWith('eloquent');
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
});
