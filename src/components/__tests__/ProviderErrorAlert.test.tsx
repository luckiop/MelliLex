import { screen, fireEvent } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ProviderErrorAlert } from '../ProviderErrorAlert';
import { renderWithProviders } from '../../test/test-utils';

// Mock useNavigate
const mockNavigate = vi.fn();
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return { ...actual, useNavigate: () => mockNavigate };
});

describe('ProviderErrorAlert', () => {
  beforeEach(() => {
    mockNavigate.mockClear();
  });

  it('renders nothing when error is null', () => {
    const { container } = renderWithProviders(
      <ProviderErrorAlert error={null} />
    );
    expect(container.querySelector('[role="alert"]')).toBeNull();
  });

  it('shows retry button when onRetry is provided', () => {
    const onRetry = vi.fn();
    renderWithProviders(
      <ProviderErrorAlert error="Failed" onRetry={onRetry} />
    );
    const retryBtn = screen.getByRole('button', { name: /try again/i });
    fireEvent.click(retryBtn);
    expect(onRetry).toHaveBeenCalledOnce();
  });

  it('does not show retry button when onRetry is not provided', () => {
    renderWithProviders(
      <ProviderErrorAlert error="Failed" />
    );
    expect(screen.queryByRole('button', { name: /try again/i })).toBeNull();
  });

  it('navigates to settings when settings button is clicked', () => {
    renderWithProviders(
      <ProviderErrorAlert
        error="Provider not configured"
        rawError={{ type: 'ProviderNotConfigured', message: 'Not configured' }}
      />
    );
    fireEvent.click(screen.getByRole('button', { name: /settings/i }));
    expect(mockNavigate).toHaveBeenCalledWith('/settings');
  });

  it('does not show settings button for non-config errors', () => {
    renderWithProviders(
      <ProviderErrorAlert error="Some random error" />
    );
    expect(screen.queryByRole('button', { name: /settings/i })).toBeNull();
  });

  it('shows the raw provider error when a lookup stalls', () => {
    const raw = 'DeepSeek chat completions: HTTP 200 OK received in 371ms, but the response body did not finish within 75s';
    renderWithProviders(<ProviderErrorAlert error={raw} />);
    expect(screen.getByText(/did not finish a reply in time/i)).toBeInTheDocument();
    expect(screen.getByText(/raw error/i)).toBeInTheDocument();
    expect(screen.getByText(raw)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /settings/i })).toBeInTheDocument();
  });

  it('detects provider config errors from message patterns', () => {
    renderWithProviders(
      <ProviderErrorAlert error="Invalid API key provided" />
    );
    // Should detect "API key" pattern as provider config error
    expect(screen.getByRole('button', { name: /settings/i })).toBeInTheDocument();
  });
});
