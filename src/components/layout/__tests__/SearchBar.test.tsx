import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi, beforeEach } from 'vitest';
import { MemoryRouter } from 'react-router-dom';
import { SearchBar } from '../SearchBar';
import { RootStoreContext } from '../../../stores/RootStore';
import { createMockRootStore } from '../../../test/test-utils';
import { api } from '../../../services/api';
import '../../../i18n';

const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return { ...actual, useNavigate: () => mockNavigate };
});

vi.mock('../../SpellCheckDialog', () => ({
  SpellCheckDialog: ({ open, onSelectWord, onCancel, spellCheckData }: any) => (
    <div data-testid="spell-dialog" data-open={open ? 'yes' : 'no'}>
      <button type="button" onClick={() => onSelectWord(spellCheckData?.suggestedWord ?? 'fallback')}>
        choose-suggestion
      </button>
      <button type="button" onClick={onCancel}>cancel-spell</button>
    </div>
  ),
}));

const renderSearchBar = (route: string, storeOverrides: Record<string, unknown> = {}) => {
  const store = createMockRootStore(storeOverrides);
  const view = render(
    <MemoryRouter initialEntries={[route]}>
      <RootStoreContext.Provider value={store}>
        <SearchBar />
      </RootStoreContext.Provider>
    </MemoryRouter>
  );
  return { ...view, store };
};

describe('SearchBar', () => {
  beforeEach(() => {
    mockNavigate.mockReset();
  });

  it('submits word search with spell-check when on word routes', async () => {
    const search = vi.fn().mockResolvedValue(undefined);
    const checkSpelling = vi.fn().mockResolvedValue(null);
    const { store } = renderSearchBar('/explore', {
      progressiveWordStore: {
        currentWord: '',
        isLoading: false,
        clearCurrentWord: vi.fn(),
      },
      searchCoordinator: {
        search,
        checkSpelling,
      },
    });

    const input = screen.getByRole('textbox');
    fireEvent.change(input, { target: { value: '  eloquent  ' } });
    fireEvent.submit(input.closest('form')!);

    await waitFor(() => {
      expect(checkSpelling).toHaveBeenCalledWith('eloquent');
      expect(search).toHaveBeenCalledWith('eloquent', { source: 'search-bar' });
    });

    expect(mockNavigate).toHaveBeenCalledWith('/');
    expect((store.progressiveWordStore.clearCurrentWord as any)).not.toHaveBeenCalled();
  });

  it('opens spell-check dialog when suggestion is returned and searches selected suggestion', async () => {
    const search = vi.fn().mockResolvedValue(undefined);
    const checkSpelling = vi.fn().mockResolvedValue({
      originalWord: 'tset',
      isCorrect: false,
      suggestedWord: 'test',
      alternatives: ['test'],
    });

    renderSearchBar('/', {
      progressiveWordStore: {
        currentWord: '',
        isLoading: false,
        clearCurrentWord: vi.fn(),
      },
      searchCoordinator: {
        search,
        checkSpelling,
      },
    });

    const input = screen.getByRole('textbox');
    fireEvent.change(input, { target: { value: 'tset' } });
    fireEvent.submit(input.closest('form')!);

    await waitFor(() => {
      expect(screen.getByTestId('spell-dialog')).toHaveAttribute('data-open', 'yes');
    });

    fireEvent.click(screen.getByRole('button', { name: 'choose-suggestion' }));

    await waitFor(() => {
      expect(search).toHaveBeenCalledWith('test', { source: 'search-bar' });
    });
  });

  it('uses history strategy on history route', async () => {
    const setSearchQuery = vi.fn();
    const checkSpelling = vi.fn();

    renderSearchBar('/history', {
      historyStore: {
        searchQuery: '',
        loadingState: 'idle',
        setSearchQuery,
      },
      searchCoordinator: {
        checkSpelling,
      },
    });

    const input = screen.getByRole('textbox');
    fireEvent.change(input, { target: { value: '  hello  ' } });
    fireEvent.submit(input.closest('form')!);

    await waitFor(() => {
      expect(setSearchQuery).toHaveBeenCalledWith('hello');
    });

    expect(checkSpelling).not.toHaveBeenCalled();
  });

  it('clears current word when input becomes empty on word strategy', () => {
    const clearCurrentWord = vi.fn();

    renderSearchBar('/', {
      progressiveWordStore: {
        currentWord: 'existing',
        isLoading: false,
        clearCurrentWord,
      },
    });

    const input = screen.getByRole('textbox');
    fireEvent.change(input, { target: { value: '' } });

    expect(clearCurrentWord).toHaveBeenCalled();
  });

  it('disables search input on settings page', () => {
    renderSearchBar('/settings');
    expect(screen.getByRole('textbox')).toBeDisabled();
  });

  it('toggles technical query in settings', async () => {
    const updateSettings = vi.fn().mockResolvedValue(undefined);
    renderSearchBar('/', {
      settingsStore: {
        settings: {
          aiProvider: 'openai',
          theme: 'light',
          enableGlobalLookup: false,
          technicalQuery: false,
          exportSettings: {},
        },
        updateSettings,
      },
    });

    fireEvent.click(screen.getByRole('button', { name: /computer science, control engineering, and robotics/i }));

    await waitFor(() => {
      expect(updateSettings).toHaveBeenCalledWith({ technicalQuery: true });
    });
  });

  it('skips spell-check for sentences routed to translation', async () => {
    const search = vi.fn().mockResolvedValue(undefined);
    const checkSpelling = vi.fn().mockResolvedValue(null);

    renderSearchBar('/', {
      progressiveWordStore: {
        currentWord: '',
        isLoading: false,
        clearCurrentWord: vi.fn(),
      },
      searchCoordinator: {
        search,
        checkSpelling,
      },
    });

    const input = screen.getByRole('textbox');
    fireEvent.change(input, { target: { value: 'Hello, world' } });
    fireEvent.submit(input.closest('form')!);

    await waitFor(() => {
      expect(search).toHaveBeenCalledWith('Hello, world', { source: 'search-bar' });
    });
    expect(checkSpelling).not.toHaveBeenCalled();
  });

  it('skips spell-check for hyphenated technical words', async () => {
    const search = vi.fn().mockResolvedValue(undefined);
    const checkSpelling = vi.fn().mockResolvedValue(null);

    renderSearchBar('/', {
      progressiveWordStore: {
        currentWord: '',
        isLoading: false,
        clearCurrentWord: vi.fn(),
      },
      searchCoordinator: {
        search,
        checkSpelling,
      },
    });

    const input = screen.getByRole('textbox');
    fireEvent.change(input, { target: { value: 'anti-communativity' } });
    fireEvent.submit(input.closest('form')!);

    await waitFor(() => {
      expect(search).toHaveBeenCalledWith('anti-communativity', { source: 'search-bar' });
    });
    expect(checkSpelling).not.toHaveBeenCalled();
  });

  it('selects a local suggestion with arrow keys and enter', async () => {
    const search = vi.fn().mockResolvedValue(undefined);
    const suggestWords = vi.spyOn(api, 'suggestWords').mockResolvedValue(['captain', 'caption']);

    renderSearchBar('/', {
      progressiveWordStore: {
        currentWord: '',
        isLoading: false,
        clearCurrentWord: vi.fn(),
      },
      searchCoordinator: {
        search,
        checkSpelling: vi.fn(),
      },
    });

    const input = screen.getByRole('textbox');
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: 'capt' } });

    expect(await screen.findByRole('option', { name: 'captain' })).toBeInTheDocument();
    fireEvent.keyDown(input, { key: 'ArrowDown' });
    fireEvent.keyDown(input, { key: 'Enter' });

    await waitFor(() => {
      expect(search).toHaveBeenCalledWith('captain', { source: 'search-bar' });
    });
    expect(screen.queryByRole('listbox')).not.toBeInTheDocument();
    expect(suggestWords).not.toHaveBeenCalledWith('captain');
    suggestWords.mockRestore();
  });

  it('does not reopen suggestions after submit while the box stays focused', async () => {
    const search = vi.fn().mockResolvedValue(undefined);
    const suggestWords = vi.spyOn(api, 'suggestWords').mockResolvedValue(['captain', 'caption']);

    renderSearchBar('/', {
      progressiveWordStore: {
        currentWord: '',
        isLoading: false,
        clearCurrentWord: vi.fn(),
      },
      searchCoordinator: {
        search,
        checkSpelling: vi.fn().mockResolvedValue(null),
      },
    });

    const input = screen.getByRole('textbox');
    fireEvent.focus(input);
    fireEvent.change(input, { target: { value: 'captain' } });
    expect(await screen.findByRole('option', { name: 'captain' })).toBeInTheDocument();

    fireEvent.submit(input.closest('form')!);

    await waitFor(() => {
      expect(search).toHaveBeenCalledWith('captain', { source: 'search-bar' });
    });
    expect(screen.queryByRole('listbox')).not.toBeInTheDocument();

    await new Promise((resolve) => setTimeout(resolve, 200));
    expect(screen.queryByRole('listbox')).not.toBeInTheDocument();
    expect(suggestWords).toHaveBeenCalledTimes(1);
    suggestWords.mockRestore();
  });
});
