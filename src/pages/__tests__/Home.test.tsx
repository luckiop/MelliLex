import { fireEvent, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { Home } from '../../pages/Home';
import { PageHintService } from '../../services/PageHintService';
import { renderWithProviders, createMockRootStore } from '../../test/test-utils';
import type { WordSection1Header, WordSection2Meanings, WordSection3Related, PhraseSection1Overview } from '../../types';

// Mock sonner
vi.mock('sonner', () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}));

// Mock ExportWordDialog since it depends on store internals
vi.mock('../../components/ExportWordDialog', () => ({
  ExportWordDialog: () => <div data-testid="export-dialog" />,
}));

// Mock HighlightedText to avoid async API calls
vi.mock('../../utils/textHighlight', () => ({
  HighlightedText: ({ text }: { text: string }) => <span>{text}</span>,
}));

// Mock ExportPhraseDialog
vi.mock('../../components/phrase/ExportPhraseDialog', () => ({
  ExportPhraseDialog: () => <div data-testid="export-phrase-dialog" />,
}));

vi.mock('@tauri-apps/api/app', () => ({
  getVersion: () => Promise.resolve('0.13.0'),
}));

const sampleHeader: WordSection1Header = {
  word: 'serendipity',
  pronunciation: '/ˌser.ənˈdɪp.ɪ.ti/',
  syllables: 'ser·en·dip·i·ty',
  origin: 'English coined by Horace Walpole',
  formality: { level: 'Neutral', percentage: 50 },
  domains: ['Literature'],
  tldr: 'Finding good things by chance.',
};

const sampleMeanings: WordSection2Meanings = {
  meanings: [{
    number: 1,
    partOfSpeech: 'noun',
    definition: 'The occurrence of events by chance in a happy way.',
    memoryTip: 'Think of a happy accident.',
    examples: ['A lucky act of serendipity.'],
  }],
};

const sampleRelated: WordSection3Related = {
  synonyms: ['luck', 'fortune'],
  antonyms: ['misfortune'],
  collocations: [],
};

const samplePhraseOverview: PhraseSection1Overview = {
  phrase: 'break the ice',
  phraseType: 'idiom',
  tldr: 'Start a conversation in a tense setting.',
  actualMeaning: 'To make people feel comfortable and begin talking.',
  formality: { level: 'Informal', percentage: 30 },
  region: 'universal',
};

describe('Home page', () => {
  beforeEach(() => {
    PageHintService.resetForTests();
  });

  afterEach(() => {
    PageHintService.resetForTests();
  });

  it('shows welcome screen when no data and not loading', async () => {
    renderWithProviders(<Home />, { route: '/' });
    expect(screen.getByText('MelliLex')).toBeInTheDocument();
    expect(screen.getByText(/AI-powered word/)).toBeInTheDocument();
    expect(await screen.findByText('v0.13.0 beta')).toBeInTheDocument();
    expect(screen.getByText(/Type a word, phrase, Chinese/)).toBeInTheDocument();
    expect(screen.getByText('Definitions')).toBeInTheDocument();
    expect(screen.getByText('Vocab notebook')).toBeInTheDocument();
    expect(screen.getByText('Markdown notes')).toBeInTheDocument();
    expect(screen.getByText('Technical lookup')).toBeInTheDocument();
    expect(screen.getByText('History')).toBeInTheDocument();
    expect(screen.getByText('Settings')).toBeInTheDocument();
    const apiGuide = screen.getByRole('button', { name: 'API' });
    fireEvent.mouseEnter(apiGuide);
    expect(screen.getByRole('button', { name: 'Open the DeepSeek platform' })).toBeInTheDocument();
    expect(screen.getByText(/Create new API key/)).toBeInTheDocument();
    fireEvent.mouseLeave(apiGuide.closest('div')!);
    expect(screen.queryByRole('button', { name: 'Open the DeepSeek platform' })).not.toBeInTheDocument();
    expect(screen.getByText(/Meanings are generated as you search/)).toBeInTheDocument();
  });

  it('hides the page hint after it is dismissed', () => {
    renderWithProviders(<Home />, { route: '/' });
    fireEvent.click(screen.getByRole('button', { name: 'Close' }));
    expect(screen.queryByText(/Meanings are generated as you search/)).not.toBeInTheDocument();
  });

  it('shows ProviderErrorAlert when word store has error', () => {
    const store = createMockRootStore({
      progressiveWordStore: {
        headerSection: null,
        meaningsSection: null,
        relatedSection: null,
        isLoading: false,
        hasError: true,
        error: 'Something went wrong with the request',
        currentWord: 'test',
        hasHeaderSection: false,
        searchWord: vi.fn(),
        clearCurrentWord: vi.fn(),
        cleanup: vi.fn(),
      },
    });
    renderWithProviders(<Home />, { store, route: '/' });
    expect(screen.getByText('Something went wrong with the request')).toBeInTheDocument();
  });

  it('shows ProviderErrorAlert when phrase store has error', () => {
    const store = createMockRootStore({
      phraseStore: {
        overviewSection: null,
        contextSection: null,
        relatedSection: null,
        isLoading: false,
        hasError: true,
        error: 'Phrase lookup failed',
        currentPhrase: 'break the ice',
        hasOverviewSection: false,
        hasContextSection: false,
        hasRelatedSection: false,
        searchPhrase: vi.fn(),
        cleanup: vi.fn(),
      },
    });
    renderWithProviders(<Home />, { store, route: '/' });
    expect(screen.getByText('Phrase lookup failed')).toBeInTheDocument();
  });

  it('renders word definition sections when data is available', () => {
    const store = createMockRootStore({
      progressiveWordStore: {
        headerSection: sampleHeader,
        meaningsSection: sampleMeanings,
        relatedSection: sampleRelated,
        isLoading: false,
        hasError: false,
        error: null,
        currentWord: 'serendipity',
        hasHeaderSection: true,
        searchWord: vi.fn(),
        clearCurrentWord: vi.fn(),
        cleanup: vi.fn(),
      },
    });
    renderWithProviders(<Home />, { store, route: '/' });
    expect(screen.getByText('serendipity')).toBeInTheDocument();
    expect(screen.getByText('Meanings')).toBeInTheDocument();
    expect(screen.getByText('Related Words')).toBeInTheDocument();
  });

  it('sets last page on mount', () => {
    const store = createMockRootStore();
    renderWithProviders(<Home />, { store, route: '/' });
    expect(store.lastPageStore.setLastPage).toHaveBeenCalledWith('/');
  });

  it('renders phrase UI when phrase data exists', () => {
    const store = createMockRootStore({
      phraseStore: {
        overviewSection: samplePhraseOverview,
        contextSection: null,
        relatedSection: null,
        isLoading: false,
        hasError: false,
        error: null,
        currentPhrase: 'break the ice',
        hasOverviewSection: true,
        hasContextSection: false,
        hasRelatedSection: false,
        searchPhrase: vi.fn(),
        cleanup: vi.fn(),
      },
    });
    renderWithProviders(<Home />, { store, route: '/' });
    expect(screen.getByText('break the ice')).toBeInTheDocument();
  });

  it('prioritizes phrase UI when both phrase and word data exist', () => {
    const store = createMockRootStore({
      progressiveWordStore: {
        headerSection: sampleHeader,
        meaningsSection: sampleMeanings,
        relatedSection: sampleRelated,
        isLoading: false,
        hasError: false,
        error: null,
        currentWord: 'serendipity',
        hasHeaderSection: true,
        searchWord: vi.fn(),
        clearCurrentWord: vi.fn(),
        cleanup: vi.fn(),
      },
      phraseStore: {
        overviewSection: samplePhraseOverview,
        contextSection: null,
        relatedSection: null,
        isLoading: false,
        hasError: false,
        error: null,
        currentPhrase: 'break the ice',
        hasOverviewSection: true,
        hasContextSection: false,
        hasRelatedSection: false,
        searchPhrase: vi.fn(),
        cleanup: vi.fn(),
      },
    });
    renderWithProviders(<Home />, { store, route: '/' });
    expect(screen.getByText('break the ice')).toBeInTheDocument();
    expect(screen.queryByText('serendipity')).toBeNull();
  });

  it('renders zh-en results instead of the English header', () => {
    const store = createMockRootStore({
      progressiveWordStore: {
        headerSection: sampleHeader,
        meaningsSection: sampleMeanings,
        relatedSection: sampleRelated,
        isLoading: false,
        hasError: false,
        error: null,
        currentWord: 'serendipity',
        hasHeaderSection: true,
        searchWord: vi.fn(),
        clearCurrentWord: vi.fn(),
        cleanup: vi.fn(),
      },
      zhEnStore: {
        currentQuery: '餐桌',
        result: {
          query: '餐桌',
          equivalents: [
            {
              expression: 'dining table',
              partOfSpeech: 'n.',
              gloss: '用餐的桌子',
              example: 'They sat around the dining table.',
              occasion: '家居/请客',
              contrast: '比 table 更具体。',
            },
          ],
        },
        isLoading: false,
        hasError: false,
        error: null,
      },
      searchCoordinator: {
        search: vi.fn(),
        checkSpelling: vi.fn(),
        currentInputType: 'zh-en' as const,
        isBusy: false,
      },
    });
    renderWithProviders(<Home />, { store, route: '/' });
    expect(screen.getByText('餐桌')).toBeInTheDocument();
    expect(screen.getByText('dining table')).toBeInTheDocument();
    expect(screen.queryByText('serendipity')).toBeNull();
  });

  it('renders translation results instead of the English header', () => {
    const source =
      'Bai Yu (白宇) is a lecturer in the College of Education Sciences at HKUST(GZ).';
    const store = createMockRootStore({
      progressiveWordStore: {
        headerSection: sampleHeader,
        meaningsSection: sampleMeanings,
        relatedSection: sampleRelated,
        isLoading: false,
        hasError: false,
        error: null,
        currentWord: 'serendipity',
        hasHeaderSection: true,
        searchWord: vi.fn(),
        clearCurrentWord: vi.fn(),
        cleanup: vi.fn(),
      },
      translateStore: {
        currentSource: source,
        result: {
          source,
          translation: '白宇是香港科技大学（广州）教育科学学院的讲师。',
          remark: '这是一段学术简历。',
        },
        isLoading: false,
        hasError: false,
        error: null,
      },
      searchCoordinator: {
        search: vi.fn(),
        checkSpelling: vi.fn(),
        currentInputType: 'translate' as const,
        isBusy: false,
      },
    });
    renderWithProviders(<Home />, { store, route: '/' });
    expect(screen.getByText('Translation')).toBeInTheDocument();
    expect(screen.getByText(source)).toBeInTheDocument();
    expect(screen.getByText('白宇是香港科技大学（广州）教育科学学院的讲师。')).toBeInTheDocument();
    expect(screen.getByText('这是一段学术简历。')).toBeInTheDocument();
    expect(screen.queryByText('serendipity')).toBeNull();
  });
});
