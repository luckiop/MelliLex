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
    expect(screen.getByText('Speech libraries')).toBeInTheDocument();
    expect(screen.getByText('Voice model')).toBeInTheDocument();
    expect(screen.getAllByText('Missing')).toHaveLength(2);
    fireEvent.click(screen.getByRole('button', { name: 'Use system voice' }));

    await expect(choice).resolves.toBe('system');
  });

  it('resolves the download when that button is clicked', async () => {
    renderWithProviders(<PronunciationPromptDialog />);
    const choice = requestPronunciationChoice('choose');

    fireEvent.click(await screen.findByRole('button', { name: 'Download and install' }));

    await expect(choice).resolves.toBe('kokoro');
  });

  it('offers to install only the missing speech libraries', async () => {
    renderWithProviders(<PronunciationPromptDialog />);
    requestPronunciationChoice('choose', '', {
      pluginReady: true,
      runtimeReady: false,
      modelReady: true,
    });

    expect(await screen.findByRole('button', { name: 'Install speech libraries' })).toBeEnabled();
    expect(screen.getByText('Installed')).toBeInTheDocument();
    expect(screen.getByText('Missing')).toBeInTheDocument();
  });

  it('disables installation when the speech component is absent', async () => {
    renderWithProviders(<PronunciationPromptDialog />);
    requestPronunciationChoice('choose', '', {
      pluginReady: false,
      runtimeReady: false,
      modelReady: false,
    });

    expect(await screen.findByRole('button', { name: 'Download and install' })).toBeDisabled();
    expect(screen.getByText('This app build does not include the speech component, so only the system voice is available.')).toBeInTheDocument();
  });
});
