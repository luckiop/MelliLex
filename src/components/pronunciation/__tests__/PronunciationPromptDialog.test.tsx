import { screen, fireEvent } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { PronunciationPromptDialog } from '../PronunciationPromptDialog';
import { renderWithProviders } from '../../../test/test-utils';
import {
  cancelPronunciationPrompt,
  requestPronunciationChoice,
} from '../../../services/pronunciationPrompt';

describe('PronunciationPromptDialog', () => {
  afterEach(() => {
    cancelPronunciationPrompt();
  });

  it('offers a download and the system voice', async () => {
    renderWithProviders(<PronunciationPromptDialog />);
    const choice = requestPronunciationChoice('choose');

    expect(await screen.findByText('Download a natural voice?')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Use system voice' }));

    await expect(choice).resolves.toBe('system');
  });

  it('resolves the download when that button is clicked', async () => {
    renderWithProviders(<PronunciationPromptDialog />);
    const choice = requestPronunciationChoice('choose');

    fireEvent.click(await screen.findByRole('button', { name: 'Download voice' }));

    await expect(choice).resolves.toBe('kokoro');
  });
});
