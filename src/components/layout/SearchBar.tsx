import { useState, useEffect, useMemo, useRef } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { observer } from 'mobx-react-lite';
import { useTranslation } from 'react-i18next';
import { Search, X, Cpu } from 'lucide-react';
import { useStores } from '../../stores/RootStore';
import { SpellCheckDialog } from '../SpellCheckDialog';
import type { SpellCheckResponse } from '../../types';
import { createSearchStrategy } from '../../services/search/SearchContextStrategy';
import { api } from '../../services/api';
import { fillSearchBox, isSearchBoxHeld, publishSearchQuery, subscribeSearchBox } from '../../services/searchBoxBridge';
import { detectInputType, SEARCH_BAR_DISPLAY_CHARS, truncateForSearchBar } from '../../utils/inputDetection';
import { canSuggestQuery } from '../../utils/wordSuggestions';
import { DEFAULT_TECHNICAL_DOMAIN } from '../settings/TechnicalDomainSection';

export const SearchBar = observer(() => {
  const { t } = useTranslation();
  const location = useLocation();
  const navigate = useNavigate();
  const {
    searchCoordinator,
    progressiveWordStore,
    historyStore,
    settingsStore,
    zhEnStore,
    translateStore,
  } = useStores();
  const [searchQuery, setSearchQuery] = useState('');
  const [focused, setFocused] = useState(false);
  const [suggestions, setSuggestions] = useState<string[]>([]);
  const [suggestionsEnabled, setSuggestionsEnabled] = useState(false);
  const suggestionsEnabledRef = useRef(false);
  const [activeIndex, setActiveIndex] = useState(-1);
  const [spellCheckData, setSpellCheckData] = useState<SpellCheckResponse | null>(null);
  const [showSpellCheckDialog, setShowSpellCheckDialog] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const listId = 'search-suggestions';

  const pathname = location.pathname;
  const isSettingsPage = pathname === '/settings';

  const searchStrategy = useMemo(
    () =>
      createSearchStrategy({
        pathname,
        navigate,
        searchCoordinator,
        progressiveWordStore,
        historyStore,
      }),
    [
      pathname,
      navigate,
      searchCoordinator,
      progressiveWordStore,
      historyStore,
    ]
  );

  useEffect(() => {
    publishSearchQuery(searchQuery);
  }, [searchQuery]);

  useEffect(() => {
    return subscribeSearchBox((value) => {
      setSearchQuery(value);
      setFocused(true);
      setSuggestions([]);
      setActiveIndex(-1);
      requestAnimationFrame(() => textareaRef.current?.focus());
    });
  }, []);

  useEffect(() => {
    if (searchStrategy.type !== 'word' || isSearchBoxHeld()) {
      return;
    }

    setSearchQuery(
      translateStore.currentSource ||
        zhEnStore.currentQuery ||
        progressiveWordStore.currentWord ||
        ''
    );
  }, [
    searchStrategy.type,
    progressiveWordStore.currentWord,
    zhEnStore.currentQuery,
    translateStore.currentSource,
  ]);

  useEffect(() => {
    if (searchStrategy.type !== 'history' || isSearchBoxHeld()) {
      return;
    }

    setSearchQuery(historyStore.searchQuery);
  }, [searchStrategy.type, historyStore.searchQuery]);

  useEffect(() => {
    if (!searchStrategy.requiresSpellCheck) {
      setShowSpellCheckDialog(false);
      setSpellCheckData(null);
    }
  }, [searchStrategy.requiresSpellCheck]);

  const collapseToPreview =
    searchStrategy.isBusy() ||
    (!focused && [...searchQuery.replace(/\s+/g, ' ').trim()].length > SEARCH_BAR_DISPLAY_CHARS);
  const displayValue = collapseToPreview ? truncateForSearchBar(searchQuery) : searchQuery;

  useEffect(() => {
    if (
      !suggestionsEnabled ||
      !focused ||
      isSettingsPage ||
      searchStrategy.type !== 'word' ||
      collapseToPreview ||
      !canSuggestQuery(searchQuery)
    ) {
      setSuggestions([]);
      setActiveIndex(-1);
      return;
    }

    const query = searchQuery.trim();
    let cancelled = false;
    setSuggestions([]);
    setActiveIndex(-1);
    const timer = window.setTimeout(() => {
      void api.suggestWords(query).then((words) => {
        if (cancelled || !suggestionsEnabledRef.current) {
          return;
        }
        setSuggestions(words);
        setActiveIndex((current) => (words.length === 0 ? -1 : Math.min(current, words.length - 1)));
      });
    }, 120);

    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [suggestionsEnabled, focused, isSettingsPage, searchStrategy.type, collapseToPreview, searchQuery]);

  const getPlaceholder = () => {
    return t(searchStrategy.placeholderKey);
  };

  const stopSuggestions = () => {
    suggestionsEnabledRef.current = false;
    setSuggestionsEnabled(false);
    setSuggestions([]);
    setActiveIndex(-1);
  };

  const handleInputChange = (value: string) => {
    const allowed = canSuggestQuery(value);
    suggestionsEnabledRef.current = allowed;
    setSuggestionsEnabled(allowed);
    setSearchQuery(value);
    setActiveIndex(-1);
    searchStrategy.handleInputChange?.(value);
  };

  const submitQuery = async (raw: string) => {
    const trimmed = raw.trim();
    if (!trimmed) return;

    stopSuggestions();

    const inputType = detectInputType(trimmed);
    if (searchStrategy.requiresSpellCheck && inputType === 'word' && !trimmed.includes('-')) {
      const spellCheck = await searchCoordinator.checkSpelling(trimmed);
      if (spellCheck) {
        setSpellCheckData(spellCheck);
        setShowSpellCheckDialog(true);
        return;
      }
    }

    await searchStrategy.performSearch(trimmed);
    if (searchStrategy.clearOnSubmit && inputType !== 'translate') {
      setSearchQuery('');
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await submitQuery(searchQuery);
  };

  const chooseSuggestion = async (word: string) => {
    stopSuggestions();
    setSearchQuery(word);
    fillSearchBox(word);
    await searchStrategy.performSearch(word);
  };

  const handleSelectWord = async (selectedWord: string) => {
    stopSuggestions();
    setShowSpellCheckDialog(false);
    setSpellCheckData(null);

    if (searchStrategy.performSuggestion) {
      await searchStrategy.performSuggestion(selectedWord);
    } else {
      await searchStrategy.performSearch(selectedWord);
    }

    if (searchStrategy.clearOnSubmit) {
      setSearchQuery('');
    }
  };

  const handleCancelSpellCheck = () => {
    setShowSpellCheckDialog(false);
    setSpellCheckData(null);
  };

  const handleClear = () => {
    stopSuggestions();
    setSearchQuery('');
    searchStrategy.handleInputChange?.('');
  };

  const canClearSearch = !isSettingsPage && searchQuery.length > 0;
  const technicalQuery = settingsStore.settings.technicalQuery ?? false;
  const technicalDomain = settingsStore.settings.technicalDomain?.trim() || DEFAULT_TECHNICAL_DOMAIN;
  const technicalHint = t('search.technicalHint', { domain: technicalDomain });
  const suggestionOpen = focused && suggestions.length > 0;

  const handleToggleTechnical = async () => {
    await settingsStore.updateSettings({ technicalQuery: !technicalQuery });
  };

  return (
    <>
      <div
        className={`relative flex items-center gap-2 flex-1 transition-all duration-300 ${
          isSettingsPage
            ? 'max-w-0 opacity-0 -translate-x-4 pointer-events-none'
            : 'w-full opacity-100 translate-x-0'
        }`}
      >
        <form
          onSubmit={handleSubmit}
          className="relative flex-1 h-9"
        >
          <textarea
            ref={textareaRef}
            rows={1}
            value={displayValue}
            readOnly={collapseToPreview}
            aria-autocomplete="list"
            aria-expanded={suggestionOpen}
            aria-controls={listId}
            aria-activedescendant={
              activeIndex >= 0 ? `${listId}-${activeIndex}` : undefined
            }
            onChange={(e) => {
              if (collapseToPreview) {
                return;
              }
              handleInputChange(e.target.value);
            }}
            onFocus={() => setFocused(true)}
            onBlur={() => setFocused(false)}
            onKeyDown={(e) => {
              if (suggestionOpen && e.key === 'ArrowDown') {
                e.preventDefault();
                setActiveIndex((current) => (current + 1) % suggestions.length);
                return;
              }
              if (suggestionOpen && e.key === 'ArrowUp') {
                e.preventDefault();
                setActiveIndex((current) =>
                  current <= 0 ? suggestions.length - 1 : current - 1
                );
                return;
              }
              if (e.key === 'Escape') {
                setSuggestions([]);
                setActiveIndex(-1);
                return;
              }
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                if (activeIndex >= 0 && suggestions[activeIndex]) {
                  void chooseSuggestion(suggestions[activeIndex]);
                  return;
                }
                e.currentTarget.form?.requestSubmit();
              }
            }}
            placeholder={getPlaceholder()}
            className="absolute top-0 left-0 z-20 w-full h-9 px-4 py-2 pr-16 border-2 border-input bg-card rounded-lg text-sm text-foreground outline-none focus:border-ring transition-colors resize-none overflow-hidden whitespace-nowrap leading-5 shadow-sm"
            disabled={isSettingsPage}
          />
          {suggestionOpen && (
            <ul
              id={listId}
              role="listbox"
              aria-label={t('search.suggestions')}
              className="absolute left-0 right-0 top-10 z-40 overflow-hidden rounded-lg border border-border bg-popover py-1 text-popover-foreground shadow-[0_16px_32px_-20px_oklch(0.28_0.04_250/0.55)]"
            >
              {suggestions.map((word, index) => {
                const selected = index === activeIndex;
                return (
                  <li key={word} role="presentation">
                    <button
                      id={`${listId}-${index}`}
                      type="button"
                      role="option"
                      aria-selected={selected}
                      className={`block w-full px-4 py-1.5 text-left text-sm ${
                        selected ? 'bg-accent text-accent-foreground' : 'hover:bg-accent/70'
                      }`}
                      onMouseDown={(event) => event.preventDefault()}
                      onMouseEnter={() => setActiveIndex(index)}
                      onClick={() => {
                        void chooseSuggestion(word);
                      }}
                    >
                      {word}
                    </button>
                  </li>
                );
              })}
            </ul>
          )}
          {canClearSearch && (
            <button
              type="button"
              data-clipboard-cancel=""
              onClick={handleClear}
              className="absolute right-10 top-2.5 z-30 text-muted-foreground hover:text-foreground transition-colors"
              aria-label={t('search.clear')}
            >
              <X className="w-4 h-4" />
            </button>
          )}
          <button
            type="submit"
            className="absolute right-3 top-2.5 z-30 text-muted-foreground hover:text-foreground transition-colors"
            disabled={isSettingsPage}
            aria-label={t('search.submit')}
          >
            <Search className="w-4 h-4" />
          </button>
        </form>
        <button
          type="button"
          onClick={handleToggleTechnical}
          disabled={isSettingsPage}
          aria-pressed={technicalQuery}
          aria-label={technicalHint}
          title={technicalHint}
          className={`shrink-0 inline-flex items-center gap-1 px-2 py-1.5 rounded-lg border text-xs font-medium transition-colors ${
            technicalQuery
              ? 'border-primary bg-primary text-primary-foreground'
              : 'border-input bg-card text-muted-foreground hover:text-foreground hover:border-ring'
          }`}
        >
          <Cpu className="w-3.5 h-3.5" />
          {t('search.technical')}
        </button>
      </div>

      <SpellCheckDialog
        open={searchStrategy.requiresSpellCheck && showSpellCheckDialog}
        spellCheckData={spellCheckData}
        onSelectWord={handleSelectWord}
        onCancel={handleCancelSpellCheck}
      />
    </>
  );
});
