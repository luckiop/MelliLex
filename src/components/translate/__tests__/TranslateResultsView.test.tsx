import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { TranslateResultsView } from '../TranslateResultsView';
import { renderWithProviders, createMockRootStore } from '../../../test/test-utils';

describe('TranslateResultsView', () => {
  it('renders source, translation, and remark', () => {
    const store = createMockRootStore({
      translateStore: {
        currentSource: 'Shall I compare thee to a summer’s day?',
        result: {
          source: 'Shall I compare thee to a summer’s day?',
          translation: '我可否把你比作夏日？',
          remark: '这是莎士比亚十四行诗的开篇，语气典雅。',
        },
        isLoading: false,
        hasError: false,
        error: null,
      },
    });

    renderWithProviders(<TranslateResultsView />, { store });
    expect(screen.getByText('Translation')).toBeInTheDocument();
    expect(screen.getByText('Shall I compare thee to a summer’s day?')).toBeInTheDocument();
    expect(screen.getByText('我可否把你比作夏日？')).toBeInTheDocument();
    expect(screen.getByText('这是莎士比亚十四行诗的开篇，语气典雅。')).toBeInTheDocument();
  });

  it('hides remark when it is empty', () => {
    const store = createMockRootStore({
      translateStore: {
        currentSource: 'Press Save to continue.',
        result: {
          source: 'Press Save to continue.',
          translation: '请按「保存」继续。',
          remark: '  ',
        },
        isLoading: false,
        hasError: false,
        error: null,
      },
    });

    renderWithProviders(<TranslateResultsView />, { store });
    expect(screen.getByText('请按「保存」继续。')).toBeInTheDocument();
    expect(screen.queryByText('Remark')).toBeNull();
  });
});
