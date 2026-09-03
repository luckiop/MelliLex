import { screen, fireEvent } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { WordSection3RelatedView } from '../sections/WordSection3RelatedView';
import { renderWithProviders } from '../../test/test-utils';
import type { WordSection3Related } from '../../types';

const sampleRelated: WordSection3Related = {
  synonyms: ['articulate', 'fluent', 'expressive'],
  antonyms: ['inarticulate', 'tongue-tied'],
  collocations: [
    { phrase: 'eloquent speech', example: 'She delivered an eloquent speech.' },
    { phrase: 'eloquent plea', example: 'He made an eloquent plea for mercy.' },
  ],
};

describe('WordSection3RelatedView', () => {
  it('renders related sections and entries', () => {
    renderWithProviders(<WordSection3RelatedView data={sampleRelated} />);
    expect(screen.getByText('Related Words')).toBeInTheDocument();
    expect(screen.getByText('articulate')).toBeInTheDocument();
    expect(screen.getByText('inarticulate')).toBeInTheDocument();
    expect(screen.getByText('eloquent speech')).toBeInTheDocument();
  });

  it('calls onWordNavigate when an antonym is clicked', () => {
    const onNavigate = vi.fn();
    renderWithProviders(
      <WordSection3RelatedView data={sampleRelated} onWordNavigate={onNavigate} />
    );
    fireEvent.click(screen.getByText('tongue-tied'));
    expect(onNavigate).toHaveBeenCalledWith('tongue-tied');
  });

  it('hides synonyms section when empty', () => {
    renderWithProviders(
      <WordSection3RelatedView data={{ ...sampleRelated, synonyms: [] }} />
    );
    expect(screen.queryByText('Synonyms')).toBeNull();
  });

  it('hides antonyms section when empty', () => {
    renderWithProviders(
      <WordSection3RelatedView data={{ ...sampleRelated, antonyms: [] }} />
    );
    expect(screen.queryByText('Antonyms')).toBeNull();
  });

  it('hides Chinese related chips so they are not searched as zh-en', () => {
    renderWithProviders(
      <WordSection3RelatedView
        data={{
          ...sampleRelated,
          synonyms: ['mechanical', '机械的'],
          antonyms: ['manual'],
        }}
      />
    );
    expect(screen.getByText('mechanical')).toBeInTheDocument();
    expect(screen.queryByText('机械的')).toBeNull();
  });
});
